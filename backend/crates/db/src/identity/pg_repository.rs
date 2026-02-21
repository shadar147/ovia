use std::str::FromStr;

use async_trait::async_trait;
use chrono::Utc;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row, Transaction};
use uuid::Uuid;

use crate::identity::models::{IdentityMappingFilter, LinkStatus, PersonIdentityLink};
use crate::identity::repositories::PersonIdentityLinkRepository;
use ovia_common::error::{OviaError, OviaResult};

pub struct PgIdentityRepository {
    pool: PgPool,
}

impl PgIdentityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_link_row(row: PgRow) -> OviaResult<PersonIdentityLink> {
        let status_raw: String = row.get("status");
        let status = LinkStatus::from_str(&status_raw).map_err(OviaError::Internal)?;

        Ok(PersonIdentityLink {
            id: row.get("id"),
            org_id: row.get("org_id"),
            person_id: row.get("person_id"),
            identity_id: row.get("identity_id"),
            status,
            confidence: row.get("confidence"),
            valid_from: row.get("valid_from"),
            valid_to: row.get("valid_to"),
            verified_by: row.get("verified_by"),
            verified_at: row.get("verified_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn append_event(
        tx: &mut Transaction<'_, Postgres>,
        org_id: Uuid,
        link_id: Uuid,
        action: &str,
        actor: &str,
        payload: Option<serde_json::Value>,
    ) -> OviaResult<()> {
        sqlx::query(
            "insert into identity_events (id, org_id, link_id, action, actor, payload, created_at)
             values ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(link_id)
        .bind(action)
        .bind(actor)
        .bind(payload)
        .bind(Utc::now())
        .execute(&mut **tx)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl PersonIdentityLinkRepository for PgIdentityRepository {
    async fn list_mappings(
        &self,
        org_id: Uuid,
        filter: IdentityMappingFilter,
    ) -> OviaResult<Vec<PersonIdentityLink>> {
        let mut qb = QueryBuilder::new(
            "select id, org_id, person_id, identity_id, status, confidence::float4 as confidence, valid_from, valid_to, verified_by, verified_at, created_at, updated_at \
             from person_identity_links where org_id = ",
        );

        qb.push_bind(org_id);

        if let Some(status) = filter.status {
            qb.push(" and status = ").push_bind(status.as_str());
        }
        if let Some(min_confidence) = filter.min_confidence {
            qb.push(" and confidence >= ").push_bind(min_confidence);
        }
        if let Some(max_confidence) = filter.max_confidence {
            qb.push(" and confidence <= ").push_bind(max_confidence);
        }

        qb.push(" order by created_at desc");
        qb.push(" limit ").push_bind(filter.limit.unwrap_or(50));
        qb.push(" offset ").push_bind(filter.offset.unwrap_or(0));

        let rows = qb
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| OviaError::Database(e.to_string()))?;

        rows.into_iter().map(Self::map_link_row).collect()
    }

    async fn confirm_mapping(
        &self,
        org_id: Uuid,
        link_id: Uuid,
        verified_by: &str,
    ) -> OviaResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| OviaError::Database(e.to_string()))?;

        let update_result = sqlx::query(
            "update person_identity_links
             set status = 'verified', verified_by = $1, verified_at = $2, updated_at = $2
             where org_id = $3 and id = $4 and valid_to is null",
        )
        .bind(verified_by)
        .bind(Utc::now())
        .bind(org_id)
        .bind(link_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        if update_result.rows_affected() == 0 {
            return Err(OviaError::NotFound(format!(
                "active link not found: {link_id}"
            )));
        }

        Self::append_event(&mut tx, org_id, link_id, "confirm", verified_by, None).await?;

        tx.commit()
            .await
            .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(())
    }

    async fn remap_mapping(
        &self,
        org_id: Uuid,
        link_id: Uuid,
        new_person_id: Uuid,
        verified_by: &str,
    ) -> OviaResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| OviaError::Database(e.to_string()))?;

        let row = sqlx::query(
            "select identity_id from person_identity_links
             where org_id = $1 and id = $2 and valid_to is null",
        )
        .bind(org_id)
        .bind(link_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        let identity_id: Uuid = match row {
            Some(r) => r.get("identity_id"),
            None => {
                return Err(OviaError::NotFound(format!(
                    "active link not found: {link_id}"
                )))
            }
        };

        let now = Utc::now();

        sqlx::query(
            "update person_identity_links
             set valid_to = $1, status = 'rejected', verified_by = $2, verified_at = $1, updated_at = $1
             where org_id = $3 and id = $4 and valid_to is null",
        )
        .bind(now)
        .bind(verified_by)
        .bind(org_id)
        .bind(link_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        let new_link_id = Uuid::new_v4();
        sqlx::query(
            "insert into person_identity_links
             (id, org_id, person_id, identity_id, status, confidence, valid_from, valid_to, verified_by, verified_at, created_at, updated_at)
             values ($1, $2, $3, $4, 'verified', 1.0, $5, null, $6, $5, $5, $5)",
        )
        .bind(new_link_id)
        .bind(org_id)
        .bind(new_person_id)
        .bind(identity_id)
        .bind(now)
        .bind(verified_by)
        .execute(&mut *tx)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        let payload = serde_json::json!({
            "old_link_id": link_id,
            "new_link_id": new_link_id,
            "new_person_id": new_person_id,
            "identity_id": identity_id,
        });
        Self::append_event(
            &mut tx,
            org_id,
            new_link_id,
            "remap",
            verified_by,
            Some(payload),
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(())
    }

    async fn split_mapping(
        &self,
        org_id: Uuid,
        link_id: Uuid,
        verified_by: &str,
    ) -> OviaResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| OviaError::Database(e.to_string()))?;

        let now = Utc::now();
        let update_result = sqlx::query(
            "update person_identity_links
             set status = 'conflict', verified_by = $1, verified_at = $2, updated_at = $2
             where org_id = $3 and id = $4 and valid_to is null",
        )
        .bind(verified_by)
        .bind(now)
        .bind(org_id)
        .bind(link_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        if update_result.rows_affected() == 0 {
            return Err(OviaError::NotFound(format!(
                "active link not found: {link_id}"
            )));
        }

        Self::append_event(&mut tx, org_id, link_id, "split", verified_by, None).await?;

        tx.commit()
            .await
            .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_pool;

    // ── Fixture helpers ──────────────────────────────────────────

    async fn test_repo() -> Option<(PgIdentityRepository, PgPool)> {
        let url = std::env::var("TEST_DATABASE_URL").ok()?;
        let pool = create_pool(&url).await.expect("db should connect");
        Some((PgIdentityRepository::new(pool.clone()), pool))
    }

    async fn insert_person(pool: &PgPool, org_id: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query("insert into people (id, org_id, display_name) values ($1, $2, 'test-person')")
            .bind(id)
            .bind(org_id)
            .execute(pool)
            .await
            .expect("insert person");
        id
    }

    async fn insert_identity(pool: &PgPool, org_id: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query("insert into identities (id, org_id, source) values ($1, $2, 'test-source')")
            .bind(id)
            .bind(org_id)
            .execute(pool)
            .await
            .expect("insert identity");
        id
    }

    async fn insert_link(
        pool: &PgPool,
        org_id: Uuid,
        person_id: Uuid,
        identity_id: Uuid,
        status: &str,
        confidence: f64,
    ) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "insert into person_identity_links \
             (id, org_id, person_id, identity_id, status, confidence) \
             values ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id)
        .bind(org_id)
        .bind(person_id)
        .bind(identity_id)
        .bind(status)
        .bind(confidence)
        .execute(pool)
        .await
        .expect("insert link");
        id
    }

    async fn insert_closed_link(
        pool: &PgPool,
        org_id: Uuid,
        person_id: Uuid,
        identity_id: Uuid,
    ) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "insert into person_identity_links \
             (id, org_id, person_id, identity_id, status, confidence, valid_to) \
             values ($1, $2, $3, $4, 'auto', 0.5, now())",
        )
        .bind(id)
        .bind(org_id)
        .bind(person_id)
        .bind(identity_id)
        .execute(pool)
        .await
        .expect("insert closed link");
        id
    }

    async fn count_events(pool: &PgPool, link_id: Uuid) -> i64 {
        sqlx::query_scalar::<_, i64>("select count(*) from identity_events where link_id = $1")
            .bind(link_id)
            .fetch_one(pool)
            .await
            .expect("count events")
    }

    async fn fetch_link_row(pool: &PgPool, link_id: Uuid) -> PgRow {
        sqlx::query("select * from person_identity_links where id = $1")
            .bind(link_id)
            .fetch_one(pool)
            .await
            .expect("fetch link row")
    }

    // ── list_mappings tests (MT-1002-07) ─────────────────────────

    #[tokio::test]
    async fn list_mappings_returns_empty_for_new_org() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };

        let results = repo
            .list_mappings(Uuid::new_v4(), IdentityMappingFilter::default())
            .await
            .expect("query should succeed");

        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn list_mappings_returns_inserted_links() {
        let (repo, pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let p1 = insert_person(&pool, org).await;
        let p2 = insert_person(&pool, org).await;
        let i1 = insert_identity(&pool, org).await;
        let i2 = insert_identity(&pool, org).await;

        let _l1 = insert_link(&pool, org, p1, i1, "auto", 0.8).await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let l2 = insert_link(&pool, org, p2, i2, "auto", 0.9).await;

        let results = repo
            .list_mappings(org, IdentityMappingFilter::default())
            .await
            .expect("list_mappings should succeed");

        assert_eq!(results.len(), 2);
        // Ordered by created_at DESC — l2 (inserted later) comes first
        assert_eq!(results[0].id, l2);
    }

    #[tokio::test]
    async fn list_mappings_filters_by_status() {
        let (repo, pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let p1 = insert_person(&pool, org).await;
        let p2 = insert_person(&pool, org).await;
        let i1 = insert_identity(&pool, org).await;
        let i2 = insert_identity(&pool, org).await;

        insert_link(&pool, org, p1, i1, "auto", 0.7).await;
        insert_link(&pool, org, p2, i2, "verified", 0.9).await;

        let filter = IdentityMappingFilter {
            status: Some(LinkStatus::Auto),
            ..Default::default()
        };
        let results = repo
            .list_mappings(org, filter)
            .await
            .expect("should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, LinkStatus::Auto);
    }

    #[tokio::test]
    async fn list_mappings_filters_by_confidence_range() {
        let (repo, pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let p1 = insert_person(&pool, org).await;
        let p2 = insert_person(&pool, org).await;
        let p3 = insert_person(&pool, org).await;
        let i1 = insert_identity(&pool, org).await;
        let i2 = insert_identity(&pool, org).await;
        let i3 = insert_identity(&pool, org).await;

        insert_link(&pool, org, p1, i1, "auto", 0.3).await;
        insert_link(&pool, org, p2, i2, "auto", 0.6).await;
        insert_link(&pool, org, p3, i3, "auto", 0.9).await;

        let filter = IdentityMappingFilter {
            min_confidence: Some(0.5),
            max_confidence: Some(0.8),
            ..Default::default()
        };
        let results = repo
            .list_mappings(org, filter)
            .await
            .expect("should succeed");

        assert_eq!(results.len(), 1);
        assert!((results[0].confidence - 0.6).abs() < 0.01);
    }

    #[tokio::test]
    async fn list_mappings_respects_limit_and_offset() {
        let (repo, pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        for _ in 0..3 {
            let p = insert_person(&pool, org).await;
            let i = insert_identity(&pool, org).await;
            insert_link(&pool, org, p, i, "auto", 0.7).await;
        }

        let filter = IdentityMappingFilter {
            limit: Some(2),
            ..Default::default()
        };
        let results = repo
            .list_mappings(org, filter)
            .await
            .expect("should succeed");
        assert_eq!(results.len(), 2);

        let filter = IdentityMappingFilter {
            limit: Some(2),
            offset: Some(2),
            ..Default::default()
        };
        let results = repo
            .list_mappings(org, filter)
            .await
            .expect("should succeed");
        assert_eq!(results.len(), 1);
    }

    // ── confirm_mapping tests (MT-1002-09) ───────────────────────

    #[tokio::test]
    async fn confirm_mapping_happy_path() {
        let (repo, pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let p = insert_person(&pool, org).await;
        let i = insert_identity(&pool, org).await;
        let link_id = insert_link(&pool, org, p, i, "auto", 0.8).await;

        repo.confirm_mapping(org, link_id, "test-reviewer")
            .await
            .expect("confirm should succeed");

        let row = fetch_link_row(&pool, link_id).await;
        let status: String = row.get("status");
        let verified_by: Option<String> = row.get("verified_by");
        assert_eq!(status, "verified");
        assert_eq!(verified_by.as_deref(), Some("test-reviewer"));

        assert_eq!(count_events(&pool, link_id).await, 1);
    }

    #[tokio::test]
    async fn confirm_mapping_not_found_nonexistent() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let result = repo
            .confirm_mapping(Uuid::new_v4(), Uuid::new_v4(), "actor")
            .await;
        assert!(matches!(result, Err(OviaError::NotFound(_))));
    }

    #[tokio::test]
    async fn confirm_mapping_not_found_closed_link() {
        let (repo, pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let p = insert_person(&pool, org).await;
        let i = insert_identity(&pool, org).await;
        let link_id = insert_closed_link(&pool, org, p, i).await;

        let result = repo.confirm_mapping(org, link_id, "actor").await;
        assert!(matches!(result, Err(OviaError::NotFound(_))));
    }

    // ── remap_mapping tests (MT-1002-11) ─────────────────────────

    #[tokio::test]
    async fn remap_mapping_happy_path() {
        let (repo, pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let old_person = insert_person(&pool, org).await;
        let new_person = insert_person(&pool, org).await;
        let identity = insert_identity(&pool, org).await;
        let old_link = insert_link(&pool, org, old_person, identity, "auto", 0.5).await;

        repo.remap_mapping(org, old_link, new_person, "test-reviewer")
            .await
            .expect("remap should succeed");

        // Old link: status=rejected, valid_to set
        let old_row = fetch_link_row(&pool, old_link).await;
        let old_status: String = old_row.get("status");
        let old_valid_to: Option<chrono::DateTime<Utc>> = old_row.get("valid_to");
        assert_eq!(old_status, "rejected");
        assert!(old_valid_to.is_some());

        // New link: status=verified, person_id=new_person, confidence=1.0
        let new_row = sqlx::query(
            "select id, status, person_id, confidence::float4 as confidence \
             from person_identity_links \
             where org_id = $1 and person_id = $2 and identity_id = $3 and valid_to is null",
        )
        .bind(org)
        .bind(new_person)
        .bind(identity)
        .fetch_one(&pool)
        .await
        .expect("new link should exist");

        let new_status: String = new_row.get("status");
        let new_person_id: Uuid = new_row.get("person_id");
        let new_confidence: f32 = new_row.get("confidence");
        let new_link_id: Uuid = new_row.get("id");
        assert_eq!(new_status, "verified");
        assert_eq!(new_person_id, new_person);
        assert!((new_confidence - 1.0).abs() < 0.01);

        // Audit event on new link with remap action + JSON payload
        let event_row =
            sqlx::query("select action, payload from identity_events where link_id = $1")
                .bind(new_link_id)
                .fetch_one(&pool)
                .await
                .expect("event should exist");

        let action: String = event_row.get("action");
        let payload: Option<serde_json::Value> = event_row.get("payload");
        assert_eq!(action, "remap");
        assert!(payload.is_some());
    }

    #[tokio::test]
    async fn remap_mapping_not_found_nonexistent() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let result = repo
            .remap_mapping(Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4(), "actor")
            .await;
        assert!(matches!(result, Err(OviaError::NotFound(_))));
    }

    #[tokio::test]
    async fn remap_mapping_not_found_closed_link() {
        let (repo, pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let p = insert_person(&pool, org).await;
        let i = insert_identity(&pool, org).await;
        let link_id = insert_closed_link(&pool, org, p, i).await;

        let result = repo
            .remap_mapping(org, link_id, Uuid::new_v4(), "actor")
            .await;
        assert!(matches!(result, Err(OviaError::NotFound(_))));
    }

    // ── split_mapping tests (MT-1002-13) ─────────────────────────

    #[tokio::test]
    async fn split_mapping_happy_path() {
        let (repo, pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let p = insert_person(&pool, org).await;
        let i = insert_identity(&pool, org).await;
        let link_id = insert_link(&pool, org, p, i, "auto", 0.6).await;

        repo.split_mapping(org, link_id, "test-reviewer")
            .await
            .expect("split should succeed");

        let row = fetch_link_row(&pool, link_id).await;
        let status: String = row.get("status");
        let valid_to: Option<chrono::DateTime<Utc>> = row.get("valid_to");
        assert_eq!(status, "conflict");
        assert!(valid_to.is_none()); // valid_to stays NULL

        assert_eq!(count_events(&pool, link_id).await, 1);
    }

    #[tokio::test]
    async fn split_mapping_not_found_nonexistent() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let result = repo
            .split_mapping(Uuid::new_v4(), Uuid::new_v4(), "actor")
            .await;
        assert!(matches!(result, Err(OviaError::NotFound(_))));
    }

    #[tokio::test]
    async fn split_mapping_not_found_closed_link() {
        let (repo, pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let p = insert_person(&pool, org).await;
        let i = insert_identity(&pool, org).await;
        let link_id = insert_closed_link(&pool, org, p, i).await;

        let result = repo.split_mapping(org, link_id, "actor").await;
        assert!(matches!(result, Err(OviaError::NotFound(_))));
    }
}
