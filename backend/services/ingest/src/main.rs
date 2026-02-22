mod confluence;
mod connector;
mod gitlab;
mod jira;
mod matching;

use ovia_config::init_tracing;

use crate::confluence::client::{ConfluenceClient, ConfluenceClientConfig};
use crate::confluence::sync::ConfluenceSyncer;
use crate::connector::Connector;
use crate::gitlab::client::{GitLabClient, GitLabClientConfig};
use crate::gitlab::mr_sync::GitLabMrPipelineSyncer;
use crate::gitlab::sync::GitLabSyncer;
use crate::jira::client::{JiraClient, JiraClientConfig};
use crate::jira::issue_sync::JiraIssueSyncer;
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
    // Fails fast if Jira creds are present but JIRA_PROJECT_KEYS is missing/empty
    match JiraClientConfig::from_env() {
        Ok(Some(jira_config)) => {
            tracing::info!(
                projects = ?jira_config.project_keys,
                window_days = jira_config.sync_window_days,
                "jira connector configured, starting sync"
            );

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

            // Jira Issue sync (separate watermark: "jira_issues")
            tracing::info!("starting jira issue sync");

            let issue_client =
                JiraClient::new(JiraClientConfig::from_env().unwrap().unwrap())
                    .expect("jira client for issue sync");
            let jira_repo =
                ovia_db::jira::pg_repository::PgJiraRepository::new(pool.clone());
            let issue_sync_repo =
                ovia_db::sync::pg_repository::PgSyncRepository::new(pool.clone());

            let issue_syncer =
                JiraIssueSyncer::new(org_id, issue_client, jira_repo, issue_sync_repo);

            match issue_syncer.sync().await {
                Ok(result) => {
                    tracing::info!(
                        source = result.source,
                        upserted = result.upserted,
                        errors = result.errors,
                        "jira issue sync completed"
                    );
                }
                Err(e) => {
                    tracing::error!(error = %e, "jira issue sync failed");
                }
            }
        }
        Ok(None) => {
            tracing::info!("no jira credentials found, skipping jira sync");
        }
        Err(e) => {
            panic!("jira configuration error (fail-fast): {e}");
        }
    }

    // GitLab connector (optional — only runs if GITLAB env vars are set)
    if let Some(gitlab_config) = GitLabClientConfig::from_env() {
        tracing::info!("gitlab connector configured, starting sync");

        let client = GitLabClient::new(gitlab_config).expect("failed to create gitlab client");
        let identity_repo =
            ovia_db::identity::pg_repository::PgIdentityRepository::new(pool.clone());
        let sync_repo = ovia_db::sync::pg_repository::PgSyncRepository::new(pool.clone());

        let syncer = GitLabSyncer::new(org_id, client, identity_repo, sync_repo);

        match syncer.sync().await {
            Ok(result) => {
                tracing::info!(
                    source = result.source,
                    upserted = result.upserted,
                    errors = result.errors,
                    "gitlab sync completed"
                );
            }
            Err(e) => {
                tracing::error!(error = %e, "gitlab sync failed");
            }
        }
        // GitLab MR/Pipeline sync (reuses same config)
        tracing::info!("starting gitlab MR/pipeline sync");

        let mr_client =
            GitLabClient::new(GitLabClientConfig::from_env().unwrap()).expect("gitlab client");
        let gitlab_repo = ovia_db::gitlab::pg_repository::PgGitlabRepository::new(pool.clone());
        let mr_sync_repo = ovia_db::sync::pg_repository::PgSyncRepository::new(pool.clone());

        let mr_syncer = GitLabMrPipelineSyncer::new(org_id, mr_client, gitlab_repo, mr_sync_repo);

        match mr_syncer.sync().await {
            Ok(result) => {
                tracing::info!(
                    source = result.source,
                    upserted = result.upserted,
                    errors = result.errors,
                    "gitlab MR/pipeline sync completed"
                );
            }
            Err(e) => {
                tracing::error!(error = %e, "gitlab MR/pipeline sync failed");
            }
        }
    } else {
        tracing::info!("no gitlab credentials found, skipping gitlab sync");
    }

    // Confluence connector (optional — only runs if CONFLUENCE env vars are set)
    if let Some(confluence_config) = ConfluenceClientConfig::from_env() {
        tracing::info!("confluence connector configured, starting sync");

        let client =
            ConfluenceClient::new(confluence_config).expect("failed to create confluence client");
        let identity_repo =
            ovia_db::identity::pg_repository::PgIdentityRepository::new(pool.clone());
        let sync_repo = ovia_db::sync::pg_repository::PgSyncRepository::new(pool.clone());

        let syncer = ConfluenceSyncer::new(org_id, client, identity_repo, sync_repo);

        match syncer.sync().await {
            Ok(result) => {
                tracing::info!(
                    source = result.source,
                    upserted = result.upserted,
                    errors = result.errors,
                    "confluence sync completed"
                );
            }
            Err(e) => {
                tracing::error!(error = %e, "confluence sync failed");
            }
        }
    } else {
        tracing::info!("no confluence credentials found, skipping confluence sync");
    }

    // ── Batch matching: link identities to people ──
    tracing::info!("starting batch matching");
    match matching::run_batch_matching(&pool, org_id).await {
        Ok(result) => {
            tracing::info!(
                people_created = result.people_created,
                links_created = result.links_created,
                auto = result.auto,
                conflict = result.conflict,
                rejected = result.rejected,
                "batch matching completed"
            );
        }
        Err(e) => {
            tracing::error!(error = %e, "batch matching failed");
        }
    }

    tracing::info!("ingest service finished");
}
