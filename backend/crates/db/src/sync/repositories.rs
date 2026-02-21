use async_trait::async_trait;
use uuid::Uuid;

use crate::sync::models::SyncWatermark;
use ovia_common::error::OviaResult;

#[async_trait]
pub trait SyncWatermarkRepository: Send + Sync {
    /// Get or create a watermark for a given org+source pair.
    async fn get_or_create(&self, org_id: Uuid, source: &str) -> OviaResult<SyncWatermark>;

    /// Atomically set status to 'running' only if currently 'idle' or 'failed'.
    /// Returns `None` if already running (lock not acquired).
    async fn acquire_lock(&self, org_id: Uuid, source: &str) -> OviaResult<Option<SyncWatermark>>;

    /// Mark a sync as completed, updating last_synced_at and optional cursor.
    async fn mark_completed(
        &self,
        id: Uuid,
        cursor_value: Option<&str>,
    ) -> OviaResult<SyncWatermark>;

    /// Mark a sync as failed with an error message.
    async fn mark_failed(&self, id: Uuid, error_message: &str) -> OviaResult<SyncWatermark>;
}
