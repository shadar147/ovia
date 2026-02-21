use chrono::{DateTime, Utc};
use ovia_db::identity::models::{LinkStatus, PersonIdentityLink};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct MutationResponse {
    pub ok: bool,
}

#[derive(Debug, Serialize)]
pub struct BulkConfirmResponse {
    pub confirmed: usize,
    pub failed: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ConflictQueueStatsResponse {
    pub total: i64,
    pub avg_confidence: Option<f64>,
    pub oldest_created_at: Option<DateTime<Utc>>,
}

// ── Enriched link with person/identity details ──────────────────

#[derive(Debug, Clone, Serialize)]
pub struct PersonSummary {
    pub id: Uuid,
    pub display_name: String,
    pub primary_email: Option<String>,
    pub team: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IdentitySummary {
    pub id: Uuid,
    pub source: String,
    pub display_name: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub is_service_account: bool,
}

#[derive(Debug, Serialize)]
pub struct EnrichedLink {
    pub id: Uuid,
    pub person_id: Uuid,
    pub identity_id: Uuid,
    pub status: LinkStatus,
    pub confidence: f32,
    pub rule_trace: Option<serde_json::Value>,
    pub verified_by: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub person: Option<PersonSummary>,
    pub identity: Option<IdentitySummary>,
}

impl EnrichedLink {
    pub fn from_link(
        link: PersonIdentityLink,
        rule_trace: Option<serde_json::Value>,
        person: Option<PersonSummary>,
        identity: Option<IdentitySummary>,
    ) -> Self {
        Self {
            id: link.id,
            person_id: link.person_id,
            identity_id: link.identity_id,
            status: link.status,
            confidence: link.confidence,
            rule_trace,
            verified_by: link.verified_by,
            verified_at: link.verified_at,
            created_at: link.created_at,
            updated_at: link.updated_at,
            person,
            identity,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ListMappingsResponse {
    pub data: Vec<EnrichedLink>,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct ConflictQueueResponse {
    pub data: Vec<EnrichedLink>,
    pub count: usize,
}
