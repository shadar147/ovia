use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub source: String,
    pub url: Option<String>,
    pub excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskSession {
    pub id: Uuid,
    pub org_id: Uuid,
    pub query: String,
    pub answer: Option<String>,
    pub confidence: Option<String>,
    pub assumptions: Option<String>,
    pub citations: Option<Vec<Citation>>,
    pub filters: Option<serde_json::Value>,
    pub model: Option<String>,
    pub prompt_tokens: Option<i32>,
    pub completion_tokens: Option<i32>,
    pub latency_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AskFilter {
    pub org_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
