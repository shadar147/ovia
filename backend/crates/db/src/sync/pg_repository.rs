use async_trait::async_trait;
use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::sync::models::SyncWatermark;
use crate::sync::repositories::SyncWatermarkRepository;
use ovia_common::error::{OviaError, OviaResult};

#[derive(Clone)]
pub struct PgSyncRepository {
    pool: PgPool,
}

impl PgSyncRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row(row: sqlx::postgres::PgRow) -> OviaResult<SyncWatermark> {
        Ok(SyncWatermark {
            id: row.get("id"),
            org_id: row.get("org_id"),
            source: row.get("source"),
            last_synced_at: row.get("last_synced_at"),
            cursor_value: row.get("cursor_value"),
            status: row.get("status"),
            error_message: row.get("error_message"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}

#[async_trait]
impl SyncWatermarkRepository for PgSyncRepository {
    async fn get_or_create(&self, org_id: Uuid, source: &str) -> OviaResult<SyncWatermark> {
        let row = sqlx::query(
            "insert into sync_watermarks (id, org_id, source)
             values ($1, $2, $3)
             on conflict (org_id, source) do update set updated_at = now()
             returning id, org_id, source, last_synced_at, cursor_value, status, error_message, created_at, updated_at",
        )
        .bind(Uuid::new_v4())
        .bind(org_id)
        .bind(source)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Self::map_row(row)
    }

    async fn acquire_lock(&self, org_id: Uuid, source: &str) -> OviaResult<Option<SyncWatermark>> {
        let row = sqlx::query(
            "update sync_watermarks
             set status = 'running', error_message = null, updated_at = $1
             where org_id = $2 and source = $3 and status != 'running'
             returning id, org_id, source, last_synced_at, cursor_value, status, error_message, created_at, updated_at",
        )
        .bind(Utc::now())
        .bind(org_id)
        .bind(source)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(Self::map_row(r)?)),
            None => Ok(None),
        }
    }

    async fn mark_completed(
        &self,
        id: Uuid,
        cursor_value: Option<&str>,
    ) -> OviaResult<SyncWatermark> {
        let now = Utc::now();
        let row = sqlx::query(
            "update sync_watermarks
             set status = 'idle', last_synced_at = $1, cursor_value = $2, error_message = null, updated_at = $1
             where id = $3
             returning id, org_id, source, last_synced_at, cursor_value, status, error_message, created_at, updated_at",
        )
        .bind(now)
        .bind(cursor_value)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Self::map_row(row)
    }

    async fn mark_failed(&self, id: Uuid, error_message: &str) -> OviaResult<SyncWatermark> {
        let row = sqlx::query(
            "update sync_watermarks
             set status = 'failed', error_message = $1, updated_at = $2
             where id = $3
             returning id, org_id, source, last_synced_at, cursor_value, status, error_message, created_at, updated_at",
        )
        .bind(error_message)
        .bind(Utc::now())
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Self::map_row(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_pool;

    async fn test_repo() -> Option<(PgSyncRepository, PgPool)> {
        let url = std::env::var("TEST_DATABASE_URL").ok()?;
        let pool = create_pool(&url).await.expect("db should connect");

        // Ensure the watermarks table exists
        sqlx::query(
            "create table if not exists sync_watermarks (
               id uuid primary key default gen_random_uuid(),
               org_id uuid not null,
               source text not null,
               last_synced_at timestamptz,
               cursor_value text,
               status text not null default 'idle',
               error_message text,
               created_at timestamptz not null default now(),
               updated_at timestamptz not null default now()
             )",
        )
        .execute(&pool)
        .await
        .ok()?;

        sqlx::query(
            "create unique index if not exists sync_watermarks_org_source_uidx
             on sync_watermarks(org_id, source)",
        )
        .execute(&pool)
        .await
        .ok()?;

        Some((PgSyncRepository::new(pool.clone()), pool))
    }

    #[tokio::test]
    async fn get_or_create_inserts_new() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let wm = repo.get_or_create(org, "jira").await.expect("should work");
        assert_eq!(wm.org_id, org);
        assert_eq!(wm.source, "jira");
        assert_eq!(wm.status, "idle");
        assert!(wm.last_synced_at.is_none());
    }

    #[tokio::test]
    async fn get_or_create_returns_existing() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let wm1 = repo.get_or_create(org, "jira").await.expect("first");
        let wm2 = repo.get_or_create(org, "jira").await.expect("second");
        assert_eq!(wm1.id, wm2.id);
    }

    #[tokio::test]
    async fn acquire_lock_succeeds_when_idle() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        repo.get_or_create(org, "jira").await.expect("create");
        let lock = repo.acquire_lock(org, "jira").await.expect("lock");
        assert!(lock.is_some());
        assert_eq!(lock.unwrap().status, "running");
    }

    #[tokio::test]
    async fn acquire_lock_fails_when_already_running() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        repo.get_or_create(org, "jira").await.expect("create");
        repo.acquire_lock(org, "jira").await.expect("first lock");
        let second = repo.acquire_lock(org, "jira").await.expect("second lock");
        assert!(second.is_none());
    }

    #[tokio::test]
    async fn mark_completed_resets_to_idle() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        repo.get_or_create(org, "jira").await.expect("create");
        let lock = repo
            .acquire_lock(org, "jira")
            .await
            .expect("lock")
            .expect("should acquire");
        let completed = repo
            .mark_completed(lock.id, Some("cursor-123"))
            .await
            .expect("mark completed");
        assert_eq!(completed.status, "idle");
        assert!(completed.last_synced_at.is_some());
        assert_eq!(completed.cursor_value.as_deref(), Some("cursor-123"));
    }

    #[tokio::test]
    async fn mark_failed_sets_error() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        repo.get_or_create(org, "jira").await.expect("create");
        let lock = repo
            .acquire_lock(org, "jira")
            .await
            .expect("lock")
            .expect("should acquire");
        let failed = repo
            .mark_failed(lock.id, "connection timeout")
            .await
            .expect("mark failed");
        assert_eq!(failed.status, "failed");
        assert_eq!(failed.error_message.as_deref(), Some("connection timeout"));
    }
}
