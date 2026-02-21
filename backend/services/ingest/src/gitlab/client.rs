use std::time::Duration;

use reqwest::{Client, StatusCode};

use super::models::GitLabUser;

#[derive(Debug, Clone)]
pub struct GitLabClientConfig {
    pub base_url: String,
    pub private_token: String,
    pub max_retries: u32,
    pub timeout_secs: u64,
}

impl GitLabClientConfig {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var("GITLAB_BASE_URL").ok()?;
        let private_token = std::env::var("GITLAB_PRIVATE_TOKEN").ok()?;
        let max_retries = std::env::var("GITLAB_MAX_RETRIES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);
        let timeout_secs = std::env::var("GITLAB_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        Some(Self {
            base_url,
            private_token,
            max_retries,
            timeout_secs,
        })
    }
}

#[derive(Clone)]
pub struct GitLabClient {
    client: Client,
    config: GitLabClientConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum GitLabClientError {
    #[error("HTTP {status}: {body}")]
    HttpError { status: StatusCode, body: String },

    #[error("request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("max retries exceeded after {attempts} attempts: {last_error}")]
    MaxRetriesExceeded { attempts: u32, last_error: String },
}

impl GitLabClient {
    pub fn new(config: GitLabClientConfig) -> Result<Self, reqwest::Error> {
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
    pub async fn fetch_all_users(&self) -> Result<Vec<GitLabUser>, GitLabClientError> {
        let per_page = 100;
        let mut page: u64 = 1;
        let mut all_users = Vec::new();

        loop {
            let url = format!(
                "{}/api/v4/users?per_page={}&page={}",
                self.config.base_url, per_page, page
            );

            let (users, next_page) = self.request_with_retry(&url).await?;
            all_users.extend(users);

            match next_page {
                Some(np) if !np.is_empty() => page = np.parse::<u64>().unwrap_or(page + 1),
                _ => break,
            }
        }

        Ok(all_users)
    }

    async fn request_with_retry(
        &self,
        url: &str,
    ) -> Result<(Vec<GitLabUser>, Option<String>), GitLabClientError> {
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
                .header("PRIVATE-TOKEN", &self.config.private_token)
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    last_error = e.to_string();
                    if e.is_timeout() || e.is_connect() {
                        continue;
                    }
                    return Err(GitLabClientError::RequestError(e));
                }
            };

            let status = response.status();

            if status.is_success() {
                let next_page = response
                    .headers()
                    .get("x-next-page")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string());

                let users = response
                    .json::<Vec<GitLabUser>>()
                    .await
                    .map_err(GitLabClientError::RequestError)?;

                return Ok((users, next_page));
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
            return Err(GitLabClientError::HttpError { status, body });
        }

        Err(GitLabClientError::MaxRetriesExceeded {
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

    fn test_config() -> GitLabClientConfig {
        GitLabClientConfig {
            base_url: "http://localhost".to_string(),
            private_token: "glpat-test-token".to_string(),
            max_retries: 2,
            timeout_secs: 5,
        }
    }

    fn make_users(count: usize, offset: usize) -> Vec<serde_json::Value> {
        (0..count)
            .map(|i| {
                serde_json::json!({
                    "id": i + offset,
                    "username": format!("user_{}", i + offset),
                    "email": format!("user{}@example.com", i + offset),
                    "name": format!("User {}", i + offset),
                    "state": "active",
                    "bot": false
                })
            })
            .collect()
    }

    #[tokio::test]
    async fn fetch_single_page() {
        let server = MockServer::start().await;
        let users = make_users(3, 0);

        Mock::given(method("GET"))
            .and(path("/api/v4/users"))
            .and(query_param("page", "1"))
            .and(query_param("per_page", "100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&users))
            .mount(&server)
            .await;

        let client = GitLabClient::new(test_config())
            .unwrap()
            .with_base_url(&server.uri());

        let result = client.fetch_all_users().await.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].username, "user_0");
    }

    #[tokio::test]
    async fn fetch_multiple_pages() {
        let server = MockServer::start().await;

        // Page 1: returns x-next-page header pointing to page 2
        let page1 = make_users(3, 0);
        Mock::given(method("GET"))
            .and(path("/api/v4/users"))
            .and(query_param("page", "1"))
            .and(query_param("per_page", "100"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(&page1)
                    .append_header("x-next-page", "2"),
            )
            .mount(&server)
            .await;

        // Page 2: no x-next-page header → last page
        let page2 = make_users(2, 3);
        Mock::given(method("GET"))
            .and(path("/api/v4/users"))
            .and(query_param("page", "2"))
            .and(query_param("per_page", "100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&page2))
            .mount(&server)
            .await;

        let client = GitLabClient::new(test_config())
            .unwrap()
            .with_base_url(&server.uri());

        let result = client.fetch_all_users().await.unwrap();
        assert_eq!(result.len(), 5);
        assert_eq!(result[0].username, "user_0");
        assert_eq!(result[3].username, "user_3");
    }

    #[tokio::test]
    async fn retries_on_500() {
        let server = MockServer::start().await;
        let users = make_users(2, 0);

        Mock::given(method("GET"))
            .and(path("/api/v4/users"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v4/users"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&users))
            .mount(&server)
            .await;

        let mut config = test_config();
        config.max_retries = 2;
        let client = GitLabClient::new(config)
            .unwrap()
            .with_base_url(&server.uri());

        let result = client.fetch_all_users().await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn fails_fast_on_401() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v4/users"))
            .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
            .mount(&server)
            .await;

        let client = GitLabClient::new(test_config())
            .unwrap()
            .with_base_url(&server.uri());

        let err = client.fetch_all_users().await.unwrap_err();
        match err {
            GitLabClientError::HttpError { status, body } => {
                assert_eq!(status, StatusCode::UNAUTHORIZED);
                assert_eq!(body, "unauthorized");
            }
            other => panic!("expected HttpError, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn empty_response() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v4/users"))
            .respond_with(ResponseTemplate::new(200).set_body_json(Vec::<serde_json::Value>::new()))
            .mount(&server)
            .await;

        let client = GitLabClient::new(test_config())
            .unwrap()
            .with_base_url(&server.uri());

        let result = client.fetch_all_users().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn uses_private_token_header() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v4/users"))
            .and(wiremock::matchers::header(
                "PRIVATE-TOKEN",
                "glpat-test-token",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(Vec::<serde_json::Value>::new()))
            .expect(1)
            .mount(&server)
            .await;

        let client = GitLabClient::new(test_config())
            .unwrap()
            .with_base_url(&server.uri());

        client.fetch_all_users().await.unwrap();
    }

    #[tokio::test]
    async fn max_retries_exceeded() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v4/users"))
            .respond_with(ResponseTemplate::new(500).set_body_string("always failing"))
            .mount(&server)
            .await;

        let mut config = test_config();
        config.max_retries = 1;
        let client = GitLabClient::new(config)
            .unwrap()
            .with_base_url(&server.uri());

        let err = client.fetch_all_users().await.unwrap_err();
        assert!(matches!(err, GitLabClientError::MaxRetriesExceeded { .. }));
    }
}
