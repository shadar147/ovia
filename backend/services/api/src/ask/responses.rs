use ovia_db::ask::models::{AskSession, Citation};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct AskApiResponse {
    pub session_id: Uuid,
    pub answer: String,
    pub confidence: String,
    pub assumptions: Option<String>,
    pub citations: Vec<Citation>,
}

#[derive(Debug, Serialize)]
pub struct AskSessionResponse {
    pub data: AskSession,
}

#[derive(Debug, Serialize)]
pub struct AskHistoryResponse {
    pub data: Vec<AskSession>,
    pub count: usize,
}
