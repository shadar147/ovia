use ovia_db::kpi::models::{KpiSnapshot, RiskItem};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct KpiSnapshotResponse {
    pub data: KpiSnapshot,
}

#[derive(Debug, Serialize)]
pub struct KpiHistoryResponse {
    pub data: Vec<KpiSnapshot>,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct KpiRisksResponse {
    pub data: Vec<RiskItem>,
    pub count: usize,
}
