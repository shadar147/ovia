pub mod ask;
pub mod gitlab;
pub mod identity;
pub mod kpi;
pub mod sync;

use ovia_common::error::{OviaError, OviaResult};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

/// Create a Postgres connection pool from a database URL.
pub async fn create_pool(database_url: &str) -> OviaResult<PgPool> {
    tracing::info!("connecting to database");
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_pool_fails_with_invalid_url() {
        let result = create_pool("postgres://invalid:5432/nonexistent").await;
        assert!(result.is_err());
    }
}
