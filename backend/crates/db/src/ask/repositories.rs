use async_trait::async_trait;
use uuid::Uuid;

use crate::ask::models::{AskFilter, AskSession};
use ovia_common::error::OviaResult;

#[async_trait]
pub trait AskRepository: Send + Sync {
    async fn save_session(&self, session: AskSession) -> OviaResult<AskSession>;
    async fn get_session(&self, org_id: Uuid, id: Uuid) -> OviaResult<Option<AskSession>>;
    async fn list_sessions(&self, filter: AskFilter) -> OviaResult<Vec<AskSession>>;
}
