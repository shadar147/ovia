use std::str::FromStr;

use async_trait::async_trait;
use chrono::Utc;
use sqlx::{postgres::PgRow, PgPool, Postgres, QueryBuilder, Row, Transaction};
use uuid::Uuid;

use crate::identity::models::{
    BulkConfirmResult, ConflictQueueFilter, ConflictQueueStats, Identity, IdentityMappingFilter,
    LinkStatus, Person, PersonFilter, PersonIdentityLink,
};
use crate::identity::repositories::{
    IdentityRepository, PersonIdentityLinkRepository, PersonRepository,
};
use ovia_common::error::{OviaError, OviaResult};

#[derive(Clone)]
pub struct PgIdentityRepository {
    pool: PgPool,
}

impl PgIdentityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    fn map_person_row(row: PgRow) -> Person {
        Person {
            id: row.get("id"),
            org_id: row.get("org_id"),
            display_name: row.get("display_name"),
            primary_email: row.get("primary_email"),
            avatar_url: row.get("avatar_url"),
            team: row.get("team"),
            role: row.get("role"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
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
impl PersonRepository for PgIdentityRepository {
    async fn get_by_id(&self, org_id: Uuid, id: Uuid) -> OviaResult<Option<Person>> {
        let row = sqlx::query(
            "select id, org_id, display_name, primary_email, avatar_url, team, role, status,
                    created_at, updated_at
             from people where org_id = $1 and id = $2",
        )
        .bind(org_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(Self::map_person_row(r))),
            None => Ok(None),
        }
    }

    async fn create(&self, person: Person) -> OviaResult<Person> {
        let row = sqlx::query(
            "insert into people (id, org_id, display_name, primary_email, avatar_url, team, role, status)
             values ($1, $2, $3, $4, $5, $6, $7, $8)
             returning id, org_id, display_name, primary_email, avatar_url, team, role, status,
                       created_at, updated_at",
        )
        .bind(person.id)
        .bind(person.org_id)
        .bind(&person.display_name)
        .bind(&person.primary_email)
        .bind(&person.avatar_url)
        .bind(&person.team)
        .bind(&person.role)
        .bind(&person.status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("duplicate key") || msg.contains("unique constraint") {
                OviaError::Validation(format!(
                    "person with this email already exists: {}",
                    person.primary_email.as_deref().unwrap_or("(none)")
                ))
            } else {
                OviaError::Database(msg)
            }
        })?;

        Ok(Self::map_person_row(row))
    }

    async fn update(&self, person: Person) -> OviaResult<Person> {
        let row = sqlx::query(
            "update people
             set display_name = $1, primary_email = $2, avatar_url = $3,
                 team = $4, role = $5, status = $6, updated_at = now()
             where id = $7 and org_id = $8
             returning id, org_id, display_name, primary_email, avatar_url, team, role, status,
                       created_at, updated_at",
        )
        .bind(&person.display_name)
        .bind(&person.primary_email)
        .bind(&person.avatar_url)
        .bind(&person.team)
        .bind(&person.role)
        .bind(&person.status)
        .bind(person.id)
        .bind(person.org_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Self::map_person_row(r)),
            None => Err(OviaError::NotFound(format!(
                "person not found: {}",
                person.id
            ))),
        }
    }

    async fn list(&self, org_id: Uuid, filter: PersonFilter) -> OviaResult<(Vec<Person>, i64)> {
        let status_filter = filter.status.as_deref().unwrap_or("active");

        let mut qb = QueryBuilder::new(
            "select id, org_id, display_name, primary_email, avatar_url, team, role, status, \
             created_at, updated_at from people where org_id = ",
        );
        qb.push_bind(org_id);
        qb.push(" and status = ").push_bind(status_filter);

        if let Some(ref team) = filter.team {
            qb.push(" and team = ").push_bind(team);
        }
        if let Some(ref search) = filter.search {
            let pattern = format!("%{search}%");
            qb.push(" and (display_name ilike ")
                .push_bind(pattern.clone())
                .push(" or primary_email ilike ")
                .push_bind(pattern)
                .push(")");
        }

        qb.push(" order by display_name asc");
        qb.push(" limit ").push_bind(filter.limit.unwrap_or(50));
        qb.push(" offset ").push_bind(filter.offset.unwrap_or(0));

        let rows = qb
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| OviaError::Database(e.to_string()))?;

        let people: Vec<Person> = rows.into_iter().map(Self::map_person_row).collect();

        // Count query
        let mut cqb = QueryBuilder::new("select count(*) from people where org_id = ");
        cqb.push_bind(org_id);
        cqb.push(" and status = ").push_bind(status_filter);

        if let Some(ref team) = filter.team {
            cqb.push(" and team = ").push_bind(team);
        }
        if let Some(ref search) = filter.search {
            let pattern = format!("%{search}%");
            cqb.push(" and (display_name ilike ")
                .push_bind(pattern.clone())
                .push(" or primary_email ilike ")
                .push_bind(pattern)
                .push(")");
        }

        let total: i64 = cqb
            .build()
            .fetch_one(&self.pool)
            .await
            .map_err(|e| OviaError::Database(e.to_string()))?
            .get(0);

        Ok((people, total))
    }

    async fn list_by_ids(&self, org_id: Uuid, ids: &[Uuid]) -> OviaResult<Vec<Person>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let rows = sqlx::query(
            "select id, org_id, display_name, primary_email, avatar_url, team, role, status,
                    created_at, updated_at
             from people where org_id = $1 and id = any($2)",
        )
        .bind(org_id)
        .bind(ids)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(Self::map_person_row).collect())
    }

    async fn soft_delete(&self, org_id: Uuid, id: Uuid) -> OviaResult<()> {
        let result = sqlx::query(
            "update people set status = 'inactive', updated_at = now()
             where org_id = $1 and id = $2 and status != 'inactive'",
        )
        .bind(org_id)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(OviaError::NotFound(format!("person not found: {id}")));
        }
        Ok(())
    }
}

#[async_trait]
impl IdentityRepository for PgIdentityRepository {
    async fn get_by_id(&self, org_id: Uuid, id: Uuid) -> OviaResult<Option<Identity>> {
        let row = sqlx::query(
            "select id, org_id, source, external_id, username, email, display_name,
                    is_service_account, first_seen_at, last_seen_at, raw_ref
             from identities where org_id = $1 and id = $2",
        )
        .bind(org_id)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(Identity {
                id: r.get("id"),
                org_id: r.get("org_id"),
                source: r.get("source"),
                external_id: r.get("external_id"),
                username: r.get("username"),
                email: r.get("email"),
                display_name: r.get("display_name"),
                is_service_account: r.get("is_service_account"),
                first_seen_at: r.get("first_seen_at"),
                last_seen_at: r.get("last_seen_at"),
                raw_ref: r.get("raw_ref"),
            })),
            None => Ok(None),
        }
    }

    async fn create(&self, identity: Identity) -> OviaResult<Identity> {
        let row = sqlx::query(
            "insert into identities (id, org_id, source, external_id, username, email, display_name,
                                     is_service_account, first_seen_at, last_seen_at, raw_ref)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             returning id, org_id, source, external_id, username, email, display_name,
                       is_service_account, first_seen_at, last_seen_at, raw_ref",
        )
        .bind(identity.id)
        .bind(identity.org_id)
        .bind(&identity.source)
        .bind(&identity.external_id)
        .bind(&identity.username)
        .bind(&identity.email)
        .bind(&identity.display_name)
        .bind(identity.is_service_account)
        .bind(identity.first_seen_at)
        .bind(identity.last_seen_at)
        .bind(&identity.raw_ref)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(Identity {
            id: row.get("id"),
            org_id: row.get("org_id"),
            source: row.get("source"),
            external_id: row.get("external_id"),
            username: row.get("username"),
            email: row.get("email"),
            display_name: row.get("display_name"),
            is_service_account: row.get("is_service_account"),
            first_seen_at: row.get("first_seen_at"),
            last_seen_at: row.get("last_seen_at"),
            raw_ref: row.get("raw_ref"),
        })
    }

    async fn update(&self, identity: Identity) -> OviaResult<Identity> {
        let row = sqlx::query(
            "update identities
             set username = $1, email = $2, display_name = $3, is_service_account = $4,
                 last_seen_at = $5, raw_ref = $6, updated_at = now()
             where id = $7 and org_id = $8
             returning id, org_id, source, external_id, username, email, display_name,
                       is_service_account, first_seen_at, last_seen_at, raw_ref",
        )
        .bind(&identity.username)
        .bind(&identity.email)
        .bind(&identity.display_name)
        .bind(identity.is_service_account)
        .bind(identity.last_seen_at)
        .bind(&identity.raw_ref)
        .bind(identity.id)
        .bind(identity.org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(Identity {
            id: row.get("id"),
            org_id: row.get("org_id"),
            source: row.get("source"),
            external_id: row.get("external_id"),
            username: row.get("username"),
            email: row.get("email"),
            display_name: row.get("display_name"),
            is_service_account: row.get("is_service_account"),
            first_seen_at: row.get("first_seen_at"),
            last_seen_at: row.get("last_seen_at"),
            raw_ref: row.get("raw_ref"),
        })
    }

    async fn upsert_by_external_id(&self, identity: Identity) -> OviaResult<Identity> {
        let now = Utc::now();
        let row = sqlx::query(
            "insert into identities (id, org_id, source, external_id, username, email, display_name,
                                     is_service_account, first_seen_at, last_seen_at, raw_ref, created_at, updated_at)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $12)
             on conflict (org_id, source, external_id) where external_id is not null
             do update set
               email = excluded.email,
               display_name = excluded.display_name,
               username = excluded.username,
               is_service_account = excluded.is_service_account,
               last_seen_at = excluded.last_seen_at,
               raw_ref = excluded.raw_ref,
               updated_at = excluded.updated_at
             returning id, org_id, source, external_id, username, email, display_name,
                       is_service_account, first_seen_at, last_seen_at, raw_ref",
        )
        .bind(identity.id)
        .bind(identity.org_id)
        .bind(&identity.source)
        .bind(&identity.external_id)
        .bind(&identity.username)
        .bind(&identity.email)
        .bind(&identity.display_name)
        .bind(identity.is_service_account)
        .bind(identity.first_seen_at.unwrap_or(now))
        .bind(identity.last_seen_at.unwrap_or(now))
        .bind(&identity.raw_ref)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(Identity {
            id: row.get("id"),
            org_id: row.get("org_id"),
            source: row.get("source"),
            external_id: row.get("external_id"),
            username: row.get("username"),
            email: row.get("email"),
            display_name: row.get("display_name"),
            is_service_account: row.get("is_service_account"),
            first_seen_at: row.get("first_seen_at"),
            last_seen_at: row.get("last_seen_at"),
            raw_ref: row.get("raw_ref"),
        })
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

    async fn list_conflicts(
        &self,
        org_id: Uuid,
        filter: ConflictQueueFilter,
    ) -> OviaResult<Vec<PersonIdentityLink>> {
        let mut qb = QueryBuilder::new(
            "select id, org_id, person_id, identity_id, status, confidence::float4 as confidence, \
             valid_from, valid_to, verified_by, verified_at, created_at, updated_at \
             from person_identity_links where org_id = ",
        );

        qb.push_bind(org_id);
        qb.push(" and status = 'conflict' and valid_to is null");

        if let Some(min_confidence) = filter.min_confidence {
            qb.push(" and confidence >= ").push_bind(min_confidence);
        }
        if let Some(max_confidence) = filter.max_confidence {
            qb.push(" and confidence <= ").push_bind(max_confidence);
        }

        match filter.sort_by.as_deref() {
            Some("confidence_asc") => qb.push(" order by confidence asc"),
            _ => qb.push(" order by created_at desc"),
        };

        qb.push(" limit ").push_bind(filter.limit.unwrap_or(50));
        qb.push(" offset ").push_bind(filter.offset.unwrap_or(0));

        let rows = qb
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| OviaError::Database(e.to_string()))?;

        rows.into_iter().map(Self::map_link_row).collect()
    }

    async fn bulk_confirm_conflicts(
        &self,
        org_id: Uuid,
        link_ids: Vec<Uuid>,
        verified_by: &str,
    ) -> OviaResult<BulkConfirmResult> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| OviaError::Database(e.to_string()))?;

        let now = Utc::now();
        let mut confirmed: usize = 0;
        let mut failed: Vec<Uuid> = Vec::new();

        for link_id in link_ids {
            let result = sqlx::query(
                "update person_identity_links \
                 set status = 'verified', verified_by = $1, verified_at = $2, updated_at = $2 \
                 where org_id = $3 and id = $4 and status = 'conflict' and valid_to is null",
            )
            .bind(verified_by)
            .bind(now)
            .bind(org_id)
            .bind(link_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| OviaError::Database(e.to_string()))?;

            if result.rows_affected() == 1 {
                Self::append_event(&mut tx, org_id, link_id, "bulk_confirm", verified_by, None)
                    .await?;
                confirmed += 1;
            } else {
                failed.push(link_id);
            }
        }

        tx.commit()
            .await
            .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(BulkConfirmResult { confirmed, failed })
    }

    async fn conflict_queue_stats(&self, org_id: Uuid) -> OviaResult<ConflictQueueStats> {
        let row = sqlx::query(
            "select count(*) as total, avg(confidence::float8) as avg_confidence, \
             min(created_at) as oldest_created_at \
             from person_identity_links \
             where org_id = $1 and status = 'conflict' and valid_to is null",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(ConflictQueueStats {
            total: row.get("total"),
            avg_confidence: row.get("avg_confidence"),
            oldest_created_at: row.get("oldest_created_at"),
        })
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

    // ── list_conflicts tests (MT-2002-01, MT-2002-02) ─────────────

    #[tokio::test]
    async fn list_conflicts_returns_only_conflict_status() {
        let (repo, pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let p1 = insert_person(&pool, org).await;
        let p2 = insert_person(&pool, org).await;
        let i1 = insert_identity(&pool, org).await;
        let i2 = insert_identity(&pool, org).await;

        insert_link(&pool, org, p1, i1, "auto", 0.8).await;
        let conflict_id = insert_link(&pool, org, p2, i2, "conflict", 0.5).await;

        let results = repo
            .list_conflicts(org, ConflictQueueFilter::default())
            .await
            .expect("list_conflicts should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, conflict_id);
        assert_eq!(results[0].status, LinkStatus::Conflict);
    }

    #[tokio::test]
    async fn list_conflicts_excludes_closed_links() {
        let (repo, pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let p = insert_person(&pool, org).await;
        let i = insert_identity(&pool, org).await;

        // Insert a conflict link that has valid_to set (closed)
        let id = Uuid::new_v4();
        sqlx::query(
            "insert into person_identity_links \
             (id, org_id, person_id, identity_id, status, confidence, valid_to) \
             values ($1, $2, $3, $4, 'conflict', 0.5, now())",
        )
        .bind(id)
        .bind(org)
        .bind(p)
        .bind(i)
        .execute(&pool)
        .await
        .expect("insert closed conflict link");

        let results = repo
            .list_conflicts(org, ConflictQueueFilter::default())
            .await
            .expect("list_conflicts should succeed");

        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn list_conflicts_sort_confidence_asc() {
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

        insert_link(&pool, org, p1, i1, "conflict", 0.8).await;
        insert_link(&pool, org, p2, i2, "conflict", 0.3).await;
        insert_link(&pool, org, p3, i3, "conflict", 0.6).await;

        let filter = ConflictQueueFilter {
            sort_by: Some("confidence_asc".to_string()),
            ..Default::default()
        };
        let results = repo
            .list_conflicts(org, filter)
            .await
            .expect("should succeed");

        assert_eq!(results.len(), 3);
        assert!((results[0].confidence - 0.3).abs() < 0.01);
        assert!((results[1].confidence - 0.6).abs() < 0.01);
        assert!((results[2].confidence - 0.8).abs() < 0.01);
    }

    #[tokio::test]
    async fn list_conflicts_sort_age_desc() {
        let (repo, pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();

        let p1 = insert_person(&pool, org).await;
        let i1 = insert_identity(&pool, org).await;
        let first = insert_link(&pool, org, p1, i1, "conflict", 0.5).await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let p2 = insert_person(&pool, org).await;
        let i2 = insert_identity(&pool, org).await;
        let second = insert_link(&pool, org, p2, i2, "conflict", 0.6).await;

        let filter = ConflictQueueFilter {
            sort_by: Some("age_desc".to_string()),
            ..Default::default()
        };
        let results = repo
            .list_conflicts(org, filter)
            .await
            .expect("should succeed");

        assert_eq!(results.len(), 2);
        // Newest first (created_at desc)
        assert_eq!(results[0].id, second);
        assert_eq!(results[1].id, first);
    }

    #[tokio::test]
    async fn list_conflicts_filters_by_confidence_range() {
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

        insert_link(&pool, org, p1, i1, "conflict", 0.3).await;
        insert_link(&pool, org, p2, i2, "conflict", 0.6).await;
        insert_link(&pool, org, p3, i3, "conflict", 0.9).await;

        let filter = ConflictQueueFilter {
            min_confidence: Some(0.5),
            max_confidence: Some(0.7),
            ..Default::default()
        };
        let results = repo
            .list_conflicts(org, filter)
            .await
            .expect("should succeed");

        assert_eq!(results.len(), 1);
        assert!((results[0].confidence - 0.6).abs() < 0.01);
    }

    #[tokio::test]
    async fn list_conflicts_respects_limit_offset() {
        let (repo, pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();

        for _ in 0..3 {
            let p = insert_person(&pool, org).await;
            let i = insert_identity(&pool, org).await;
            insert_link(&pool, org, p, i, "conflict", 0.5).await;
        }

        let filter = ConflictQueueFilter {
            limit: Some(2),
            ..Default::default()
        };
        let results = repo
            .list_conflicts(org, filter)
            .await
            .expect("should succeed");
        assert_eq!(results.len(), 2);

        let filter = ConflictQueueFilter {
            limit: Some(2),
            offset: Some(2),
            ..Default::default()
        };
        let results = repo
            .list_conflicts(org, filter)
            .await
            .expect("should succeed");
        assert_eq!(results.len(), 1);
    }

    // ── bulk_confirm_conflicts tests (MT-2002-03) ─────────────────

    #[tokio::test]
    async fn bulk_confirm_happy_path() {
        let (repo, pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();

        let p1 = insert_person(&pool, org).await;
        let p2 = insert_person(&pool, org).await;
        let i1 = insert_identity(&pool, org).await;
        let i2 = insert_identity(&pool, org).await;

        let l1 = insert_link(&pool, org, p1, i1, "conflict", 0.5).await;
        let l2 = insert_link(&pool, org, p2, i2, "conflict", 0.6).await;

        let result = repo
            .bulk_confirm_conflicts(org, vec![l1, l2], "reviewer")
            .await
            .expect("bulk confirm should succeed");

        assert_eq!(result.confirmed, 2);
        assert!(result.failed.is_empty());

        // Verify status changed
        let row1 = fetch_link_row(&pool, l1).await;
        let status1: String = row1.get("status");
        assert_eq!(status1, "verified");

        let row2 = fetch_link_row(&pool, l2).await;
        let status2: String = row2.get("status");
        assert_eq!(status2, "verified");

        // Verify audit events
        assert_eq!(count_events(&pool, l1).await, 1);
        assert_eq!(count_events(&pool, l2).await, 1);
    }

    #[tokio::test]
    async fn bulk_confirm_partial_failure() {
        let (repo, pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();

        let p = insert_person(&pool, org).await;
        let i = insert_identity(&pool, org).await;

        let valid_id = insert_link(&pool, org, p, i, "conflict", 0.5).await;
        let invalid_id = Uuid::new_v4(); // does not exist

        let result = repo
            .bulk_confirm_conflicts(org, vec![valid_id, invalid_id], "reviewer")
            .await
            .expect("bulk confirm should succeed");

        assert_eq!(result.confirmed, 1);
        assert_eq!(result.failed, vec![invalid_id]);
    }

    #[tokio::test]
    async fn bulk_confirm_empty_list() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };

        let result = repo
            .bulk_confirm_conflicts(Uuid::new_v4(), vec![], "reviewer")
            .await
            .expect("bulk confirm empty should succeed");

        assert_eq!(result.confirmed, 0);
        assert!(result.failed.is_empty());
    }

    // ── conflict_queue_stats tests (MT-2002-05) ───────────────────

    #[tokio::test]
    async fn conflict_queue_stats_empty() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };

        let stats = repo
            .conflict_queue_stats(Uuid::new_v4())
            .await
            .expect("stats should succeed");

        assert_eq!(stats.total, 0);
        assert!(stats.avg_confidence.is_none());
        assert!(stats.oldest_created_at.is_none());
    }

    #[tokio::test]
    async fn conflict_queue_stats_with_data() {
        let (repo, pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();

        let p1 = insert_person(&pool, org).await;
        let p2 = insert_person(&pool, org).await;
        let i1 = insert_identity(&pool, org).await;
        let i2 = insert_identity(&pool, org).await;

        insert_link(&pool, org, p1, i1, "conflict", 0.4).await;
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        insert_link(&pool, org, p2, i2, "conflict", 0.6).await;

        // Also insert a non-conflict link — should not affect stats
        let p3 = insert_person(&pool, org).await;
        let i3 = insert_identity(&pool, org).await;
        insert_link(&pool, org, p3, i3, "auto", 0.9).await;

        let stats = repo
            .conflict_queue_stats(org)
            .await
            .expect("stats should succeed");

        assert_eq!(stats.total, 2);
        let avg = stats.avg_confidence.expect("should have avg");
        assert!((avg - 0.5).abs() < 0.01);
        assert!(stats.oldest_created_at.is_some());
    }
}
