use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use ovia_db::identity::models::Identity;
use ovia_db::identity::repositories::IdentityRepository;
use ovia_db::sync::repositories::SyncWatermarkRepository;

use super::client::JiraClient;
use super::models::JiraUser;
use crate::connector::{Connector, SyncResult};

pub struct JiraSyncer<I, S> {
    org_id: Uuid,
    client: JiraClient,
    identity_repo: I,
    sync_repo: S,
}

impl<I, S> JiraSyncer<I, S>
where
    I: IdentityRepository,
    S: SyncWatermarkRepository,
{
    pub fn new(org_id: Uuid, client: JiraClient, identity_repo: I, sync_repo: S) -> Self {
        Self {
            org_id,
            client,
            identity_repo,
            sync_repo,
        }
    }

    fn jira_user_to_identity(&self, user: &JiraUser) -> Identity {
        Identity {
            id: Uuid::new_v4(),
            org_id: self.org_id,
            source: "jira".to_string(),
            external_id: Some(user.account_id.clone()),
            username: None,
            email: user.email_address.clone(),
            display_name: user.display_name.clone(),
            is_service_account: user.is_service_account(),
            first_seen_at: Some(Utc::now()),
            last_seen_at: Some(Utc::now()),
            raw_ref: serde_json::to_value(user).ok(),
        }
    }
}

#[async_trait]
impl<I, S> Connector for JiraSyncer<I, S>
where
    I: IdentityRepository,
    S: SyncWatermarkRepository,
{
    fn source_name(&self) -> &str {
        "jira"
    }

    async fn sync(&self) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        // Ensure watermark row exists
        self.sync_repo
            .get_or_create(self.org_id, "jira")
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

        // Try to acquire lock
        let watermark = self
            .sync_repo
            .acquire_lock(self.org_id, "jira")
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

        let watermark = match watermark {
            Some(wm) => wm,
            None => {
                tracing::info!(
                    "jira sync already running for org={}, skipping",
                    self.org_id
                );
                return Ok(SyncResult {
                    source: "jira".to_string(),
                    upserted: 0,
                    skipped: 0,
                    errors: 0,
                });
            }
        };

        // Fetch all users
        let users = match self.client.fetch_all_users().await {
            Ok(users) => users,
            Err(e) => {
                let msg = e.to_string();
                tracing::error!(error = %msg, "jira user fetch failed");
                self.sync_repo
                    .mark_failed(watermark.id, &msg)
                    .await
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
                return Err(Box::new(e));
            }
        };

        tracing::info!(count = users.len(), "fetched jira users");

        // Upsert each user
        let mut upserted = 0;
        let mut errors = 0;

        for user in &users {
            let identity = self.jira_user_to_identity(user);
            match self.identity_repo.upsert_by_external_id(identity).await {
                Ok(_) => upserted += 1,
                Err(e) => {
                    tracing::warn!(
                        account_id = %user.account_id,
                        error = %e,
                        "failed to upsert jira user"
                    );
                    errors += 1;
                }
            }
        }

        // Mark completed
        self.sync_repo
            .mark_completed(watermark.id, None)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

        let result = SyncResult {
            source: "jira".to_string(),
            upserted,
            skipped: 0,
            errors,
        };

        tracing::info!(?result, "jira sync completed");
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jira::client::{JiraClient, JiraClientConfig};
    use ovia_db::identity::models::Identity;
    use ovia_db::sync::models::SyncWatermark;
    use std::sync::{Arc, Mutex};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // ── Mock IdentityRepository ─────────────────────────────────

    #[derive(Clone)]
    struct MockIdentityRepo {
        upserted: Arc<Mutex<Vec<Identity>>>,
    }

    impl MockIdentityRepo {
        fn new() -> Self {
            Self {
                upserted: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl IdentityRepository for MockIdentityRepo {
        async fn get_by_id(
            &self,
            _org_id: Uuid,
            _id: Uuid,
        ) -> ovia_common::error::OviaResult<Option<Identity>> {
            Ok(None)
        }

        async fn create(&self, identity: Identity) -> ovia_common::error::OviaResult<Identity> {
            Ok(identity)
        }

        async fn update(&self, identity: Identity) -> ovia_common::error::OviaResult<Identity> {
            Ok(identity)
        }

        async fn upsert_by_external_id(
            &self,
            identity: Identity,
        ) -> ovia_common::error::OviaResult<Identity> {
            self.upserted.lock().unwrap().push(identity.clone());
            Ok(identity)
        }
    }

    // ── Mock SyncWatermarkRepository ────────────────────────────

    struct MockSyncRepo {
        lock_available: bool,
    }

    impl MockSyncRepo {
        fn new(lock_available: bool) -> Self {
            Self { lock_available }
        }

        fn dummy_watermark() -> SyncWatermark {
            SyncWatermark {
                id: Uuid::new_v4(),
                org_id: Uuid::new_v4(),
                source: "jira".to_string(),
                last_synced_at: None,
                cursor_value: None,
                status: "running".to_string(),
                error_message: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }
        }
    }

    #[async_trait]
    impl SyncWatermarkRepository for MockSyncRepo {
        async fn get_or_create(
            &self,
            _org_id: Uuid,
            _source: &str,
        ) -> ovia_common::error::OviaResult<SyncWatermark> {
            Ok(Self::dummy_watermark())
        }

        async fn acquire_lock(
            &self,
            _org_id: Uuid,
            _source: &str,
        ) -> ovia_common::error::OviaResult<Option<SyncWatermark>> {
            if self.lock_available {
                Ok(Some(Self::dummy_watermark()))
            } else {
                Ok(None)
            }
        }

        async fn mark_completed(
            &self,
            _id: Uuid,
            _cursor_value: Option<&str>,
        ) -> ovia_common::error::OviaResult<SyncWatermark> {
            Ok(Self::dummy_watermark())
        }

        async fn mark_failed(
            &self,
            _id: Uuid,
            _error_message: &str,
        ) -> ovia_common::error::OviaResult<SyncWatermark> {
            Ok(Self::dummy_watermark())
        }
    }

    fn make_jira_users(count: usize) -> Vec<serde_json::Value> {
        (0..count)
            .map(|i| {
                serde_json::json!({
                    "accountId": format!("user-{i}"),
                    "emailAddress": format!("user{i}@example.com"),
                    "displayName": format!("User {i}"),
                    "active": true,
                    "accountType": "atlassian"
                })
            })
            .collect()
    }

    fn test_client_config(base_url: &str) -> JiraClientConfig {
        JiraClientConfig {
            base_url: base_url.to_string(),
            email: "test@example.com".to_string(),
            api_token: "token".to_string(),
            project_keys: vec!["PROJ".to_string()],
            sync_window_days: 7,
            max_retries: 1,
            timeout_secs: 5,
        }
    }

    #[tokio::test]
    async fn sync_upserts_all_users() {
        let server = MockServer::start().await;
        let users = make_jira_users(3);

        Mock::given(method("GET"))
            .and(path("/rest/api/3/users/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&users))
            .mount(&server)
            .await;

        let client = JiraClient::new(test_client_config(&server.uri())).unwrap();
        let identity_repo = MockIdentityRepo::new();
        let sync_repo = MockSyncRepo::new(true);

        let syncer = JiraSyncer::new(Uuid::new_v4(), client, identity_repo, sync_repo);
        let result = syncer.sync().await.expect("sync should succeed");

        assert_eq!(result.source, "jira");
        assert_eq!(result.upserted, 3);
        assert_eq!(result.errors, 0);
    }

    #[tokio::test]
    async fn sync_skips_when_lock_unavailable() {
        let server = MockServer::start().await;

        let client = JiraClient::new(test_client_config(&server.uri())).unwrap();
        let identity_repo = MockIdentityRepo::new();
        let sync_repo = MockSyncRepo::new(false);

        let syncer = JiraSyncer::new(Uuid::new_v4(), client, identity_repo, sync_repo);
        let result = syncer.sync().await.expect("sync should succeed");

        assert_eq!(result.upserted, 0);
        assert_eq!(result.skipped, 0);
    }

    #[tokio::test]
    async fn sync_marks_service_accounts() {
        let server = MockServer::start().await;
        let users = vec![
            serde_json::json!({
                "accountId": "human-1",
                "emailAddress": "human@example.com",
                "displayName": "Human User",
                "active": true,
                "accountType": "atlassian"
            }),
            serde_json::json!({
                "accountId": "app-1",
                "displayName": "Bot App",
                "active": true,
                "accountType": "app"
            }),
        ];

        Mock::given(method("GET"))
            .and(path("/rest/api/3/users/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&users))
            .mount(&server)
            .await;

        let client = JiraClient::new(test_client_config(&server.uri())).unwrap();
        let identity_repo = MockIdentityRepo::new();
        let sync_repo = MockSyncRepo::new(true);

        let syncer = JiraSyncer::new(Uuid::new_v4(), client, identity_repo.clone(), sync_repo);
        syncer.sync().await.expect("sync should succeed");

        let upserted = identity_repo.upserted.lock().unwrap();
        assert_eq!(upserted.len(), 2);

        let human = upserted
            .iter()
            .find(|i| i.external_id.as_deref() == Some("human-1"))
            .unwrap();
        assert!(!human.is_service_account);

        let app = upserted
            .iter()
            .find(|i| i.external_id.as_deref() == Some("app-1"))
            .unwrap();
        assert!(app.is_service_account);
    }

    #[tokio::test]
    async fn sync_sets_raw_ref() {
        let server = MockServer::start().await;
        let users = make_jira_users(1);

        Mock::given(method("GET"))
            .and(path("/rest/api/3/users/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&users))
            .mount(&server)
            .await;

        let client = JiraClient::new(test_client_config(&server.uri())).unwrap();
        let identity_repo = MockIdentityRepo::new();
        let sync_repo = MockSyncRepo::new(true);

        let syncer = JiraSyncer::new(Uuid::new_v4(), client, identity_repo.clone(), sync_repo);
        syncer.sync().await.expect("sync should succeed");

        let upserted = identity_repo.upserted.lock().unwrap();
        assert_eq!(upserted.len(), 1);
        let raw = upserted[0].raw_ref.as_ref().expect("raw_ref should be set");
        assert_eq!(raw["accountId"], "user-0");
    }
}
