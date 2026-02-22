use std::time::Duration;

use reqwest::{Client, StatusCode};

use super::models::JiraUser;

#[derive(Debug, Clone)]
pub struct JiraClientConfig {
    pub base_url: String,
    pub email: String,
    pub api_token: String,
    pub project_keys: Vec<String>,
    pub sync_window_days: u32,
    pub max_retries: u32,
    pub timeout_secs: u64,
}

impl JiraClientConfig {
    /// Load Jira config from environment.
    ///
    /// Returns `Ok(None)` if Jira is not configured (base URL / email / token missing).
    /// Returns `Err` if Jira IS configured but `JIRA_PROJECT_KEYS` is missing or empty
    /// (fail-fast on misconfiguration).
    pub fn from_env() -> Result<Option<Self>, String> {
        let base_url = match std::env::var("JIRA_BASE_URL").ok() {
            Some(v) => v,
            None => return Ok(None),
        };
        let email = match std::env::var("JIRA_EMAIL").ok() {
            Some(v) => v,
            None => return Ok(None),
        };
        let api_token = match std::env::var("JIRA_API_TOKEN").ok() {
            Some(v) => v,
            None => return Ok(None),
        };

        // Jira IS configured — JIRA_PROJECT_KEYS is now mandatory
        let project_keys = parse_csv_project_keys("JIRA_PROJECT_KEYS")?;

        let sync_window_days = std::env::var("JIRA_SYNC_WINDOW_DAYS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(7);
        let max_retries = std::env::var("JIRA_MAX_RETRIES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);
        let timeout_secs = std::env::var("JIRA_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        Ok(Some(Self {
            base_url,
            email,
            api_token,
            project_keys,
            sync_window_days,
            max_retries,
            timeout_secs,
        }))
    }
}

/// Parse a comma-separated list of Jira project keys from an env var.
/// Returns `Err` if the var is missing or all entries are blank after trimming.
pub fn parse_csv_project_keys(env_key: &str) -> Result<Vec<String>, String> {
    let raw = std::env::var(env_key).map_err(|_| {
        format!("{env_key} is required when Jira credentials are set, but not found")
    })?;

    let keys: Vec<String> = raw
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect();

    if keys.is_empty() {
        return Err(format!(
            "{env_key} is set but contains no valid project keys"
        ));
    }

    Ok(keys)
}

#[derive(Clone)]
pub struct JiraClient {
    client: Client,
    config: JiraClientConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum JiraClientError {
    #[error("HTTP {status}: {body}")]
    HttpError { status: StatusCode, body: String },

    #[error("request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("max retries exceeded after {attempts} attempts: {last_error}")]
    MaxRetriesExceeded { attempts: u32, last_error: String },
}

impl JiraClient {
    pub fn new(config: JiraClientConfig) -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()?;
        Ok(Self { client, config })
    }

    /// For testing: create a client pointing at a specific base URL (e.g., wiremock).
    #[cfg(test)]
    pub fn with_base_url(mut self, base_url: &str) -> Self {
        self.config.base_url = base_url.to_string();
        self
    }

    /// Fetch all users via paginated API, retrying transient errors.
    pub async fn fetch_all_users(&self) -> Result<Vec<JiraUser>, JiraClientError> {
        let max_results = 50;
        let mut start_at = 0;
        let mut all_users = Vec::new();

        loop {
            let url = format!(
                "{}/rest/api/3/users/search?startAt={}&maxResults={}",
                self.config.base_url, start_at, max_results
            );

            let page: Vec<JiraUser> = self.request_with_retry(&url).await?;
            let page_len = page.len();
            all_users.extend(page);

            if page_len < max_results {
                break;
            }
            start_at += max_results;
        }

        Ok(all_users)
    }

    async fn request_with_retry(&self, url: &str) -> Result<Vec<JiraUser>, JiraClientError> {
        let mut last_error = String::new();

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let backoff_secs = std::cmp::min(1u64 << attempt, 30);
                tracing::warn!(attempt, backoff_secs, "retrying after backoff");
                tokio::time::sleep(Duration::from_secs(backoff_secs)).await;
            }

            let response = match self
                .client
                .get(url)
                .basic_auth(&self.config.email, Some(&self.config.api_token))
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    last_error = e.to_string();
                    if e.is_timeout() || e.is_connect() {
                        continue;
                    }
                    return Err(JiraClientError::RequestError(e));
                }
            };

            let status = response.status();

            if status.is_success() {
                return response
                    .json::<Vec<JiraUser>>()
                    .await
                    .map_err(JiraClientError::RequestError);
            }

            // Honor Retry-After header for 429
            if status == StatusCode::TOO_MANY_REQUESTS {
                if let Some(retry_after) = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                {
                    let wait = std::cmp::min(retry_after, 60);
                    tracing::warn!(wait, "rate-limited, waiting Retry-After");
                    tokio::time::sleep(Duration::from_secs(wait)).await;
                }
                last_error = "429 Too Many Requests".to_string();
                continue;
            }

            // Retry on 5xx
            if status.is_server_error() {
                let body = response.text().await.unwrap_or_default();
                last_error = format!("{status}: {body}");
                continue;
            }

            // Fail fast on 4xx (except 429 handled above)
            let body = response.text().await.unwrap_or_default();
            return Err(JiraClientError::HttpError { status, body });
        }

        Err(JiraClientError::MaxRetriesExceeded {
            attempts: self.config.max_retries + 1,
            last_error,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_config() -> JiraClientConfig {
        JiraClientConfig {
            base_url: "http://localhost".to_string(),
            email: "test@example.com".to_string(),
            api_token: "fake-token".to_string(),
            project_keys: vec!["PROJ".to_string()],
            sync_window_days: 7,
            max_retries: 2,
            timeout_secs: 5,
        }
    }

    fn make_users(count: usize, offset: usize) -> Vec<serde_json::Value> {
        (0..count)
            .map(|i| {
                serde_json::json!({
                    "accountId": format!("user-{}", i + offset),
                    "emailAddress": format!("user{}@example.com", i + offset),
                    "displayName": format!("User {}", i + offset),
                    "active": true,
                    "accountType": "atlassian"
                })
            })
            .collect()
    }

    #[tokio::test]
    async fn fetch_single_page() {
        let server = MockServer::start().await;
        let users = make_users(3, 0);

        Mock::given(method("GET"))
            .and(path("/rest/api/3/users/search"))
            .and(query_param("startAt", "0"))
            .and(query_param("maxResults", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&users))
            .mount(&server)
            .await;

        let client = JiraClient::new(test_config())
            .unwrap()
            .with_base_url(&server.uri());

        let result = client.fetch_all_users().await.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].account_id, "user-0");
    }

    #[tokio::test]
    async fn fetch_multiple_pages() {
        let server = MockServer::start().await;

        // Page 1: 50 users (full page → triggers next)
        let page1 = make_users(50, 0);
        Mock::given(method("GET"))
            .and(path("/rest/api/3/users/search"))
            .and(query_param("startAt", "0"))
            .and(query_param("maxResults", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&page1))
            .mount(&server)
            .await;

        // Page 2: 10 users (partial → last page)
        let page2 = make_users(10, 50);
        Mock::given(method("GET"))
            .and(path("/rest/api/3/users/search"))
            .and(query_param("startAt", "50"))
            .and(query_param("maxResults", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&page2))
            .mount(&server)
            .await;

        let client = JiraClient::new(test_config())
            .unwrap()
            .with_base_url(&server.uri());

        let result = client.fetch_all_users().await.unwrap();
        assert_eq!(result.len(), 60);
        assert_eq!(result[0].account_id, "user-0");
        assert_eq!(result[50].account_id, "user-50");
    }

    #[tokio::test]
    async fn retries_on_500() {
        let server = MockServer::start().await;
        let users = make_users(2, 0);

        Mock::given(method("GET"))
            .and(path("/rest/api/3/users/search"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/rest/api/3/users/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&users))
            .mount(&server)
            .await;

        let mut config = test_config();
        config.max_retries = 2;
        let client = JiraClient::new(config)
            .unwrap()
            .with_base_url(&server.uri());

        let result = client.fetch_all_users().await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn fails_fast_on_401() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/api/3/users/search"))
            .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
            .mount(&server)
            .await;

        let client = JiraClient::new(test_config())
            .unwrap()
            .with_base_url(&server.uri());

        let err = client.fetch_all_users().await.unwrap_err();
        match err {
            JiraClientError::HttpError { status, body } => {
                assert_eq!(status, StatusCode::UNAUTHORIZED);
                assert_eq!(body, "unauthorized");
            }
            other => panic!("expected HttpError, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn fails_fast_on_403() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/api/3/users/search"))
            .respond_with(ResponseTemplate::new(403).set_body_string("forbidden"))
            .mount(&server)
            .await;

        let client = JiraClient::new(test_config())
            .unwrap()
            .with_base_url(&server.uri());

        let err = client.fetch_all_users().await.unwrap_err();
        assert!(matches!(err, JiraClientError::HttpError { .. }));
    }

    #[tokio::test]
    async fn max_retries_exceeded() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/api/3/users/search"))
            .respond_with(ResponseTemplate::new(500).set_body_string("always failing"))
            .mount(&server)
            .await;

        let mut config = test_config();
        config.max_retries = 1;
        let client = JiraClient::new(config)
            .unwrap()
            .with_base_url(&server.uri());

        let err = client.fetch_all_users().await.unwrap_err();
        assert!(matches!(err, JiraClientError::MaxRetriesExceeded { .. }));
    }

    #[tokio::test]
    async fn empty_response() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/api/3/users/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Vec::<serde_json::Value>::new()))
            .mount(&server)
            .await;

        let client = JiraClient::new(test_config())
            .unwrap()
            .with_base_url(&server.uri());

        let result = client.fetch_all_users().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn uses_basic_auth() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/api/3/users/search"))
            .and(wiremock::matchers::header_exists("Authorization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Vec::<serde_json::Value>::new()))
            .expect(1)
            .mount(&server)
            .await;

        let client = JiraClient::new(test_config())
            .unwrap()
            .with_base_url(&server.uri());

        client.fetch_all_users().await.unwrap();
    }

    // ── CSV parser tests ─────────────────────────────────────────

    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn parse_csv_valid_multiple_keys() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var("_TEST_KEYS", "PROJ, TEAM ,ops");
        let keys = super::parse_csv_project_keys("_TEST_KEYS").unwrap();
        assert_eq!(keys, vec!["PROJ", "TEAM", "OPS"]);
        std::env::remove_var("_TEST_KEYS");
    }

    #[test]
    fn parse_csv_single_key() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var("_TEST_KEYS2", "SINGLE");
        let keys = super::parse_csv_project_keys("_TEST_KEYS2").unwrap();
        assert_eq!(keys, vec!["SINGLE"]);
        std::env::remove_var("_TEST_KEYS2");
    }

    #[test]
    fn parse_csv_empty_value_fails() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var("_TEST_KEYS3", "  , , ");
        let err = super::parse_csv_project_keys("_TEST_KEYS3").unwrap_err();
        assert!(err.contains("no valid project keys"), "got: {err}");
        std::env::remove_var("_TEST_KEYS3");
    }

    #[test]
    fn parse_csv_missing_var_fails() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var("_TEST_KEYS_MISSING");
        let err = super::parse_csv_project_keys("_TEST_KEYS_MISSING").unwrap_err();
        assert!(err.contains("required"), "got: {err}");
    }

    #[test]
    fn parse_csv_trims_whitespace_and_uppercases() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var("_TEST_KEYS4", "  alpha , Beta,GAMMA  ");
        let keys = super::parse_csv_project_keys("_TEST_KEYS4").unwrap();
        assert_eq!(keys, vec!["ALPHA", "BETA", "GAMMA"]);
        std::env::remove_var("_TEST_KEYS4");
    }

    #[test]
    fn from_env_returns_none_when_no_jira_creds() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var("JIRA_BASE_URL");
        std::env::remove_var("JIRA_EMAIL");
        std::env::remove_var("JIRA_API_TOKEN");
        std::env::remove_var("JIRA_PROJECT_KEYS");
        let result = JiraClientConfig::from_env().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn from_env_fails_when_creds_set_but_no_project_keys() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var("JIRA_BASE_URL", "https://test.atlassian.net");
        std::env::set_var("JIRA_EMAIL", "a@b.com");
        std::env::set_var("JIRA_API_TOKEN", "tok");
        std::env::remove_var("JIRA_PROJECT_KEYS");
        let err = JiraClientConfig::from_env().unwrap_err();
        assert!(err.contains("JIRA_PROJECT_KEYS"), "got: {err}");
        std::env::remove_var("JIRA_BASE_URL");
        std::env::remove_var("JIRA_EMAIL");
        std::env::remove_var("JIRA_API_TOKEN");
    }

    #[test]
    fn from_env_succeeds_with_all_vars() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var("JIRA_BASE_URL", "https://test.atlassian.net");
        std::env::set_var("JIRA_EMAIL", "a@b.com");
        std::env::set_var("JIRA_API_TOKEN", "tok");
        std::env::set_var("JIRA_PROJECT_KEYS", "DEV,OPS");
        std::env::set_var("JIRA_SYNC_WINDOW_DAYS", "14");
        let cfg = JiraClientConfig::from_env().unwrap().unwrap();
        assert_eq!(cfg.project_keys, vec!["DEV", "OPS"]);
        assert_eq!(cfg.sync_window_days, 14);
        std::env::remove_var("JIRA_BASE_URL");
        std::env::remove_var("JIRA_EMAIL");
        std::env::remove_var("JIRA_API_TOKEN");
        std::env::remove_var("JIRA_PROJECT_KEYS");
        std::env::remove_var("JIRA_SYNC_WINDOW_DAYS");
    }
}
