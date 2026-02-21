use axum::extract::{Query, State};
use axum::Json;
use ovia_common::error::OviaError;
use ovia_db::kpi::models::KpiFilter;
use ovia_db::kpi::repositories::KpiRepository;

use crate::error::ApiError;
use crate::extractors::OrgId;
use crate::kpi::responses::{KpiHistoryResponse, KpiRisksResponse, KpiSnapshotResponse};
use crate::AppState;

pub async fn get_latest_kpi(
    State(state): State<AppState>,
    OrgId(org): OrgId,
) -> Result<Json<KpiSnapshotResponse>, ApiError> {
    let snapshot = state
        .kpi_repo
        .get_latest(org)
        .await?
        .ok_or_else(|| OviaError::NotFound("no KPI snapshot found for this org".to_string()))?;

    Ok(Json(KpiSnapshotResponse { data: snapshot }))
}

pub async fn list_kpi_history(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Query(mut filter): Query<KpiFilter>,
) -> Result<Json<KpiHistoryResponse>, ApiError> {
    filter.org_id = Some(org);
    let data = state.kpi_repo.list_snapshots(filter).await?;
    let count = data.len();
    Ok(Json(KpiHistoryResponse { data, count }))
}

pub async fn list_kpi_risks(
    State(state): State<AppState>,
    OrgId(org): OrgId,
) -> Result<Json<KpiRisksResponse>, ApiError> {
    let snapshot = state
        .kpi_repo
        .get_latest(org)
        .await?
        .ok_or_else(|| OviaError::NotFound("no KPI snapshot found for this org".to_string()))?;

    let data = state.kpi_repo.list_risk_items(snapshot.id).await?;
    let count = data.len();
    Ok(Json(KpiRisksResponse { data, count }))
}
