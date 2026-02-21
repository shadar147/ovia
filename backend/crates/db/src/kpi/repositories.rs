use async_trait::async_trait;
use uuid::Uuid;

use crate::kpi::models::{KpiFilter, KpiSnapshot, RiskItem};
use ovia_common::error::OviaResult;

#[async_trait]
pub trait KpiRepository: Send + Sync {
    async fn save_snapshot(&self, snapshot: KpiSnapshot) -> OviaResult<KpiSnapshot>;
    async fn get_latest(&self, org_id: Uuid) -> OviaResult<Option<KpiSnapshot>>;
    async fn list_snapshots(&self, filter: KpiFilter) -> OviaResult<Vec<KpiSnapshot>>;
    async fn save_risk_items(&self, items: Vec<RiskItem>) -> OviaResult<Vec<RiskItem>>;
    async fn list_risk_items(&self, snapshot_id: Uuid) -> OviaResult<Vec<RiskItem>>;
}
