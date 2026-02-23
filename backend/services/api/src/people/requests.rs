use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreatePersonRequest {
    pub display_name: String,
    pub primary_email: Option<String>,
    pub avatar_url: Option<String>,
    pub team: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePersonRequest {
    pub display_name: Option<String>,
    pub primary_email: Option<String>,
    pub avatar_url: Option<String>,
    pub team: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LinkIdentityRequest {
    pub identity_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ActivityFilter {
    pub period: Option<String>,       // 7d, 30d, 90d
    pub source: Option<String>,       // gitlab, jira, identity, all
    #[serde(rename = "type")]
    pub activity_type: Option<String>, // merge_request, issue, identity_event, all
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
