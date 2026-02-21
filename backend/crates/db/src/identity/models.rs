use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LinkStatus {
    Auto,
    Verified,
    Conflict,
    Rejected,
    Ignored,
}

impl LinkStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Verified => "verified",
            Self::Conflict => "conflict",
            Self::Rejected => "rejected",
            Self::Ignored => "ignored",
        }
    }
}

impl FromStr for LinkStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "auto" => Ok(Self::Auto),
            "verified" => Ok(Self::Verified),
            "conflict" => Ok(Self::Conflict),
            "rejected" => Ok(Self::Rejected),
            "ignored" => Ok(Self::Ignored),
            _ => Err(format!("unknown link status: {value}")),
        }
    }
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
    pub raw_ref: Option<serde_json::Value>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConflictQueueFilter {
    pub min_confidence: Option<f32>,
    pub max_confidence: Option<f32>,
    pub sort_by: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkConfirmResult {
    pub confirmed: usize,
    pub failed: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictQueueStats {
    pub total: i64,
    pub avg_confidence: Option<f64>,
    pub oldest_created_at: Option<DateTime<Utc>>,
}
