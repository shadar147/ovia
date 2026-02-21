use chrono::{DateTime, Utc};
use ovia_db::identity::models::PersonIdentityLink;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ListMappingsResponse {
    pub data: Vec<PersonIdentityLink>,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct MutationResponse {
    pub ok: bool,
}

#[derive(Debug, Serialize)]
pub struct ConflictQueueResponse {
    pub data: Vec<PersonIdentityLink>,
    pub count: usize,
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
