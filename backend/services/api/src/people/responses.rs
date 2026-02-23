use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct PersonResponse {
    pub id: Uuid,
    pub display_name: String,
    pub primary_email: Option<String>,
    pub avatar_url: Option<String>,
    pub team: Option<String>,
    pub role: Option<String>,
    pub status: String,
    pub identity_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct ListPeopleResponse {
    pub data: Vec<PersonResponse>,
    pub count: usize,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct LinkResponse {
    pub id: Uuid,
    pub person_id: Uuid,
    pub identity_id: Uuid,
    pub status: String,
    pub confidence: f64,
}

#[derive(Debug, Serialize)]
pub struct LinkedIdentityResponse {
    pub link_id: Uuid,
    pub identity_id: Uuid,
    pub source: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub status: String,
    pub confidence: f64,
    pub linked_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct LinkedIdentitiesResponse {
    pub data: Vec<LinkedIdentityResponse>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActivityItem {
    pub id: String,
    pub source: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub title: String,
    pub url: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ActivityListResponse {
    pub data: Vec<ActivityItem>,
    pub count: usize,
    pub total: i64,
}
