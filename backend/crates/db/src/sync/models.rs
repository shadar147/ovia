use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncWatermark {
    pub id: Uuid,
    pub org_id: Uuid,
    pub source: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub cursor_value: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
