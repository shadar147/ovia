use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::Utc;
use ovia_common::error::OviaError;
use ovia_db::ask::models::{AskFilter, AskSession, Citation};
use ovia_db::ask::repositories::AskRepository;
use ovia_db::kpi::repositories::KpiRepository;
use uuid::Uuid;

use crate::error::ApiError;
use crate::extractors::OrgId;
use crate::AppState;

use super::requests::AskRequest;
use super::responses::{AskApiResponse, AskHistoryResponse, AskSessionResponse};

pub async fn post_ask(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Json(body): Json<AskRequest>,
) -> Result<Json<AskApiResponse>, ApiError> {
    if body.query.trim().is_empty() {
        return Err(ApiError(OviaError::Validation(
            "query must not be empty".to_string(),
        )));
    }

    let start = std::time::Instant::now();

    // Look up latest KPI data for the org
    let snapshot = state.kpi_repo.get_latest(org).await?;

    let (answer, confidence, citations) = if let Some(snap) = &snapshot {
        let answer = format!(
            "Based on the latest KPI snapshot (period {} to {}):\n\
             - Delivery health score: {}\n\
             - Release risk score: {}\n\
             - Total throughput: {} items ({} features, {} bugs, {} chores)\n\
             - Median review latency: {} hours\n\n\
             Note: This is an automated summary. LLM-powered analysis is pending integration.",
            snap.period_start,
            snap.period_end,
            snap.delivery_health_score
                .map_or("N/A".to_string(), |v| format!("{v:.1}")),
            snap.release_risk_score
                .map_or("N/A".to_string(), |v| format!("{v:.1}")),
            snap.throughput_total,
            snap.throughput_features,
            snap.throughput_bugs,
            snap.throughput_chores,
            snap.review_latency_median_hours
                .map_or("N/A".to_string(), |v| format!("{v:.1}")),
        );

        let citations = vec![Citation {
            source: "kpi_snapshot".to_string(),
            url: None,
            excerpt: format!(
                "delivery_health_score: {}, release_risk_score: {}, throughput_total: {}",
                snap.delivery_health_score
                    .map_or("null".to_string(), |v| format!("{v:.1}")),
                snap.release_risk_score
                    .map_or("null".to_string(), |v| format!("{v:.1}")),
                snap.throughput_total
            ),
        }];

        (answer, "medium".to_string(), citations)
    } else {
        let answer = format!(
            "No KPI data is available for this organization yet. \
             Please run a KPI computation first.\n\n\
             Your query was: \"{}\"\n\n\
             Note: This is an automated stub response. LLM-powered analysis is pending integration.",
            body.query
        );

        (answer, "low".to_string(), vec![])
    };

    let latency_ms = start.elapsed().as_millis() as i32;

    let filters_json = body
        .filters
        .as_ref()
        .map(|f| serde_json::to_value(f).unwrap_or_default());

    let session = AskSession {
        id: Uuid::new_v4(),
        org_id: org,
        query: body.query,
        answer: Some(answer.clone()),
        confidence: Some(confidence.clone()),
        assumptions: Some("Stub response based on latest KPI snapshot data.".to_string()),
        citations: Some(citations.clone()),
        filters: filters_json,
        model: Some("stub-v1".to_string()),
        prompt_tokens: Some(0),
        completion_tokens: Some(0),
        latency_ms: Some(latency_ms),
        created_at: Utc::now(),
    };

    let saved = state.ask_repo.save_session(session).await?;

    Ok(Json(AskApiResponse {
        session_id: saved.id,
        answer,
        confidence,
        assumptions: Some("Stub response based on latest KPI snapshot data.".to_string()),
        citations,
    }))
}

pub async fn get_ask_session(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Path(id): Path<Uuid>,
) -> Result<Json<AskSessionResponse>, ApiError> {
    let session = state
        .ask_repo
        .get_session(org, id)
        .await?
        .ok_or_else(|| OviaError::NotFound("session not found".to_string()))?;

    Ok(Json(AskSessionResponse { data: session }))
}

pub async fn list_ask_history(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Query(mut filter): Query<AskFilter>,
) -> Result<Json<AskHistoryResponse>, ApiError> {
    filter.org_id = Some(org);
    let data = state.ask_repo.list_sessions(filter).await?;
    let count = data.len();
    Ok(Json(AskHistoryResponse { data, count }))
}
