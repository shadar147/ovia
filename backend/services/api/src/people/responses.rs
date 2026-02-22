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
