mod connector;
mod jira;

use ovia_config::init_tracing;

use crate::connector::Connector;
use crate::jira::client::{JiraClient, JiraClientConfig};
use crate::jira::sync::JiraSyncer;

#[tokio::main]
async fn main() {
    init_tracing("info");
    let _ = dotenvy::dotenv();

    tracing::info!(service = "ovia-ingest", "starting");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = ovia_db::create_pool(&database_url)
        .await
        .expect("failed to connect to database");

    // Default org_id — in production this would come from a config or multi-tenant loop
    let org_id: uuid::Uuid = std::env::var("ORG_ID")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(uuid::Uuid::new_v4);

    // Jira connector (optional — only runs if JIRA env vars are set)
    if let Some(jira_config) = JiraClientConfig::from_env() {
        tracing::info!("jira connector configured, starting sync");

        let client = JiraClient::new(jira_config).expect("failed to create jira client");
        let identity_repo =
            ovia_db::identity::pg_repository::PgIdentityRepository::new(pool.clone());
        let sync_repo = ovia_db::sync::pg_repository::PgSyncRepository::new(pool.clone());

        let syncer = JiraSyncer::new(org_id, client, identity_repo, sync_repo);

        match syncer.sync().await {
            Ok(result) => {
                tracing::info!(
                    source = result.source,
                    upserted = result.upserted,
                    errors = result.errors,
                    "jira sync completed"
                );
            }
            Err(e) => {
                tracing::error!(error = %e, "jira sync failed");
            }
        }
    } else {
        tracing::info!("no jira credentials found, skipping jira sync");
    }

    tracing::info!("ingest service finished");
}
