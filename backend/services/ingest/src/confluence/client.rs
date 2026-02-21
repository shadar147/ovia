use std::time::Duration;

use reqwest::{Client, StatusCode};

use super::models::{ConfluencePageResponse, ConfluenceUser};

#[derive(Debug, Clone)]
pub struct ConfluenceClientConfig {
    pub base_url: String,
    pub email: String,
    pub api_token: String,
    pub max_retries: u32,
    pub timeout_secs: u64,
}

impl ConfluenceClientConfig {
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var("CONFLUENCE_BASE_URL").ok()?;
        let email = std::env::var("CONFLUENCE_EMAIL").ok()?;
        let api_token = std::env::var("CONFLUENCE_API_TOKEN").ok()?;
        let max_retries = std::env::var("CONFLUENCE_MAX_RETRIES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);
        let timeout_secs = std::env::var("CONFLUENCE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        Some(Self {
            base_url,
            email,
            api_token,
            max_retries,
            timeout_secs,
        })
    }
}

#[derive(Clone)]
pub struct ConfluenceClient {
    client: Client,
    config: ConfluenceClientConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfluenceClientError {
    #[error("HTTP {status}: {body}")]
    HttpError { status: StatusCode, body: String },

    #[error("request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("max retries exceeded after {attempts} attempts: {last_error}")]
    MaxRetriesExceeded { attempts: u32, last_error: String },
}

impl ConfluenceClient {
    pub fn new(config: ConfluenceClientConfig) -> Result<Self, reqwest::Error> {
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

    /// Fetch all users via paginated group-member API, retrying transient errors.
    pub async fn fetch_all_users(&self) -> Result<Vec<ConfluenceUser>, ConfluenceClientError> {
        let limit = 50;
        let mut start = 0;
        let mut all_users = Vec::new();

        loop {
            let url = format!(
                "{}/wiki/rest/api/group/confluence-users/member?limit={}&start={}",
                self.config.base_url, limit, start
            );

            let page: ConfluencePageResponse = self.request_with_retry(&url).await?;
            let page_size = page.size;
            all_users.extend(page.results);

            if page_size < limit {
                break;
            }
            start += limit;
        }

        Ok(all_users)
    }

    async fn request_with_retry(
        &self,
        url: &str,
    ) -> Result<ConfluencePageResponse, ConfluenceClientError> {
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
                    return Err(ConfluenceClientError::RequestError(e));
                }
            };

            let status = response.status();

            if status.is_success() {
                return response
                    .json::<ConfluencePageResponse>()
                    .await
                    .map_err(ConfluenceClientError::RequestError);
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
            return Err(ConfluenceClientError::HttpError { status, body });
        }

        Err(ConfluenceClientError::MaxRetriesExceeded {
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

    fn test_config() -> ConfluenceClientConfig {
        ConfluenceClientConfig {
            base_url: "http://localhost".to_string(),
            email: "test@example.com".to_string(),
            api_token: "fake-token".to_string(),
            max_retries: 2,
            timeout_secs: 5,
        }
    }

    fn make_page_response(count: usize, offset: usize, limit: usize) -> serde_json::Value {
        let results: Vec<serde_json::Value> = (0..count)
            .map(|i| {
                serde_json::json!({
                    "accountId": format!("user-{}", i + offset),
                    "email": format!("user{}@example.com", i + offset),
                    "displayName": format!("User {}", i + offset),
                    "publicName": format!("User {}", i + offset),
                    "accountType": "atlassian"
                })
            })
            .collect();

        serde_json::json!({
            "results": results,
            "start": offset,
            "limit": limit,
            "size": count
        })
    }

    #[tokio::test]
    async fn fetch_single_page() {
        let server = MockServer::start().await;
        let page = make_page_response(3, 0, 50);

        Mock::given(method("GET"))
            .and(path("/wiki/rest/api/group/confluence-users/member"))
            .and(query_param("start", "0"))
            .and(query_param("limit", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&page))
            .mount(&server)
            .await;

        let client = ConfluenceClient::new(test_config())
            .unwrap()
            .with_base_url(&server.uri());

        let result = client.fetch_all_users().await.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].account_id, "user-0");
    }

    #[tokio::test]
    async fn fetch_multiple_pages() {
        let server = MockServer::start().await;

        // Page 1: 50 users (full page -> triggers next)
        let page1 = make_page_response(50, 0, 50);
        Mock::given(method("GET"))
            .and(path("/wiki/rest/api/group/confluence-users/member"))
            .and(query_param("start", "0"))
            .and(query_param("limit", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&page1))
            .mount(&server)
            .await;

        // Page 2: 10 users (partial -> last page)
        let page2 = make_page_response(10, 50, 50);
        Mock::given(method("GET"))
            .and(path("/wiki/rest/api/group/confluence-users/member"))
            .and(query_param("start", "50"))
            .and(query_param("limit", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&page2))
            .mount(&server)
            .await;

        let client = ConfluenceClient::new(test_config())
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
        let page = make_page_response(2, 0, 50);

        Mock::given(method("GET"))
            .and(path("/wiki/rest/api/group/confluence-users/member"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/wiki/rest/api/group/confluence-users/member"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&page))
            .mount(&server)
            .await;

        let mut config = test_config();
        config.max_retries = 2;
        let client = ConfluenceClient::new(config)
            .unwrap()
            .with_base_url(&server.uri());

        let result = client.fetch_all_users().await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn fails_fast_on_401() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/wiki/rest/api/group/confluence-users/member"))
            .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
            .mount(&server)
            .await;

        let client = ConfluenceClient::new(test_config())
            .unwrap()
            .with_base_url(&server.uri());

        let err = client.fetch_all_users().await.unwrap_err();
        match err {
            ConfluenceClientError::HttpError { status, body } => {
                assert_eq!(status, StatusCode::UNAUTHORIZED);
                assert_eq!(body, "unauthorized");
            }
            other => panic!("expected HttpError, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn empty_response() {
        let server = MockServer::start().await;
        let page = serde_json::json!({
            "results": [],
            "start": 0,
            "limit": 50,
            "size": 0
        });

        Mock::given(method("GET"))
            .and(path("/wiki/rest/api/group/confluence-users/member"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&page))
            .mount(&server)
            .await;

        let client = ConfluenceClient::new(test_config())
            .unwrap()
            .with_base_url(&server.uri());

        let result = client.fetch_all_users().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn uses_basic_auth() {
        let server = MockServer::start().await;
        let page = serde_json::json!({
            "results": [],
            "start": 0,
            "limit": 50,
            "size": 0
        });

        Mock::given(method("GET"))
            .and(path("/wiki/rest/api/group/confluence-users/member"))
            .and(wiremock::matchers::header_exists("Authorization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&page))
            .expect(1)
            .mount(&server)
            .await;

        let client = ConfluenceClient::new(test_config())
            .unwrap()
            .with_base_url(&server.uri());

        client.fetch_all_users().await.unwrap();
    }

    #[tokio::test]
    async fn max_retries_exceeded() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/wiki/rest/api/group/confluence-users/member"))
            .respond_with(ResponseTemplate::new(500).set_body_string("always failing"))
            .mount(&server)
            .await;

        let mut config = test_config();
        config.max_retries = 1;
        let client = ConfluenceClient::new(config)
            .unwrap()
            .with_base_url(&server.uri());

        let err = client.fetch_all_users().await.unwrap_err();
        assert!(matches!(
            err,
            ConfluenceClientError::MaxRetriesExceeded { .. }
        ));
    }
}
