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
            "select id, org_id, person_id, identity_id, status, confidence, valid_from, valid_to, verified_by, verified_at, created_at, updated_at \
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

    #[tokio::test]
    async fn list_mappings_returns_empty_for_new_org() {
        let database_url = match std::env::var("TEST_DATABASE_URL") {
            Ok(v) => v,
            Err(_) => return,
        };

        let pool = create_pool(&database_url)
            .await
            .expect("db should connect for integration test");
        let repo = PgIdentityRepository::new(pool);

        let results = repo
            .list_mappings(Uuid::new_v4(), IdentityMappingFilter::default())
            .await
            .expect("query should succeed");

        assert!(results.is_empty());
    }
}
