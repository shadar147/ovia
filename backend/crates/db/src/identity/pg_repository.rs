use std::str::FromStr;

use async_trait::async_trait;
use sqlx::{postgres::PgRow, PgPool, QueryBuilder, Row};
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
        _org_id: Uuid,
        _link_id: Uuid,
        _verified_by: &str,
    ) -> OviaResult<()> {
        Err(OviaError::Internal(
            "confirm_mapping not implemented yet".to_string(),
        ))
    }

    async fn remap_mapping(
        &self,
        _org_id: Uuid,
        _link_id: Uuid,
        _new_person_id: Uuid,
        _verified_by: &str,
    ) -> OviaResult<()> {
        Err(OviaError::Internal(
            "remap_mapping not implemented yet".to_string(),
        ))
    }

    async fn split_mapping(
        &self,
        _org_id: Uuid,
        _link_id: Uuid,
        _verified_by: &str,
    ) -> OviaResult<()> {
        Err(OviaError::Internal(
            "split_mapping not implemented yet".to_string(),
        ))
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
