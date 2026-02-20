use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LinkStatus {
    Auto,
    Verified,
    Conflict,
    Rejected,
    Ignored,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: Uuid,
    pub org_id: Uuid,
    pub display_name: String,
    pub primary_email: Option<String>,
    pub team: Option<String>,
    pub role: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub id: Uuid,
    pub org_id: Uuid,
    pub source: String,
    pub external_id: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub is_service_account: bool,
    pub first_seen_at: Option<DateTime<Utc>>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonIdentityLink {
    pub id: Uuid,
    pub org_id: Uuid,
    pub person_id: Uuid,
    pub identity_id: Uuid,
    pub status: LinkStatus,
    pub confidence: f32,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_to: Option<DateTime<Utc>>,
    pub verified_by: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityEvent {
    pub id: Uuid,
    pub org_id: Uuid,
    pub link_id: Uuid,
    pub action: String,
    pub actor: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdentityMappingFilter {
    pub status: Option<LinkStatus>,
    pub min_confidence: Option<f32>,
    pub max_confidence: Option<f32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
