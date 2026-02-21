use std::collections::{HashMap, HashSet};

use axum::extract::{Query, State};
use axum::http::header;
use axum::response::IntoResponse;
use axum::Json;
use ovia_common::error::OviaError;
use ovia_db::identity::models::{ConflictQueueFilter, IdentityMappingFilter, PersonIdentityLink};
use ovia_db::identity::repositories::PersonIdentityLinkRepository;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::ApiError;
use crate::extractors::OrgId;
use crate::identity::formatters::format_conflicts_csv;
use crate::identity::requests::{BulkConfirmRequest, ConfirmRequest, RemapRequest, SplitRequest};
use crate::identity::responses::{
    BulkConfirmResponse, ConflictQueueResponse, ConflictQueueStatsResponse, EnrichedLink,
    IdentitySummary, ListMappingsResponse, MutationResponse, PersonSummary,
};
use crate::AppState;

fn validate_filter(filter: &IdentityMappingFilter) -> Result<(), OviaError> {
    if let Some(min) = filter.min_confidence {
        if !(0.0..=1.0).contains(&min) {
            return Err(OviaError::Validation(
                "min_confidence must be between 0.0 and 1.0".to_string(),
            ));
        }
    }
    if let Some(max) = filter.max_confidence {
        if !(0.0..=1.0).contains(&max) {
            return Err(OviaError::Validation(
                "max_confidence must be between 0.0 and 1.0".to_string(),
            ));
        }
    }
    if let (Some(min), Some(max)) = (filter.min_confidence, filter.max_confidence) {
        if min > max {
            return Err(OviaError::Validation(
                "min_confidence must not exceed max_confidence".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_verified_by(verified_by: &str) -> Result<(), OviaError> {
    if verified_by.trim().is_empty() {
        return Err(OviaError::Validation(
            "verified_by must not be empty".to_string(),
        ));
    }
    Ok(())
}

fn validate_conflict_filter(filter: &ConflictQueueFilter) -> Result<(), OviaError> {
    if let Some(min) = filter.min_confidence {
        if !(0.0..=1.0).contains(&min) {
            return Err(OviaError::Validation(
                "min_confidence must be between 0.0 and 1.0".to_string(),
            ));
        }
    }
    if let Some(max) = filter.max_confidence {
        if !(0.0..=1.0).contains(&max) {
            return Err(OviaError::Validation(
                "max_confidence must be between 0.0 and 1.0".to_string(),
            ));
        }
    }
    if let (Some(min), Some(max)) = (filter.min_confidence, filter.max_confidence) {
        if min > max {
            return Err(OviaError::Validation(
                "min_confidence must not exceed max_confidence".to_string(),
            ));
        }
    }
    if let Some(ref sort) = filter.sort_by {
        if sort != "confidence_asc" && sort != "age_desc" {
            return Err(OviaError::Validation(
                "sort_by must be 'confidence_asc' or 'age_desc'".to_string(),
            ));
        }
    }
    Ok(())
}

// ── Enrichment helpers ──────────────────────────────────────────

async fn fetch_people_by_ids(pool: &PgPool, ids: &[Uuid]) -> HashMap<Uuid, PersonSummary> {
    if ids.is_empty() {
        return HashMap::new();
    }
    let rows = sqlx::query(
        "select id, display_name, primary_email, team from people where id = any($1)",
    )
    .bind(ids)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .map(|r| {
            let id: Uuid = r.get("id");
            (
                id,
                PersonSummary {
                    id,
                    display_name: r.get("display_name"),
                    primary_email: r.get("primary_email"),
                    team: r.get("team"),
                },
            )
        })
        .collect()
}

async fn fetch_identities_by_ids(
    pool: &PgPool,
    ids: &[Uuid],
) -> HashMap<Uuid, IdentitySummary> {
    if ids.is_empty() {
        return HashMap::new();
    }
    let rows = sqlx::query(
        "select id, source, display_name, username, email, is_service_account \
         from identities where id = any($1)",
    )
    .bind(ids)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .map(|r| {
            let id: Uuid = r.get("id");
            (
                id,
                IdentitySummary {
                    id,
                    source: r.get("source"),
                    display_name: r.get("display_name"),
                    username: r.get("username"),
                    email: r.get("email"),
                    is_service_account: r.get("is_service_account"),
                },
            )
        })
        .collect()
}

async fn fetch_rule_traces(
    pool: &PgPool,
    link_ids: &[Uuid],
) -> HashMap<Uuid, serde_json::Value> {
    if link_ids.is_empty() {
        return HashMap::new();
    }
    let rows = sqlx::query(
        "select id, rule_trace from person_identity_links \
         where id = any($1) and rule_trace is not null",
    )
    .bind(link_ids)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .filter_map(|r| {
            let id: Uuid = r.get("id");
            let trace: Option<serde_json::Value> = r.get("rule_trace");
            trace.map(|t| (id, t))
        })
        .collect()
}

async fn enrich_links(pool: &PgPool, links: Vec<PersonIdentityLink>) -> Vec<EnrichedLink> {
    let person_ids: Vec<Uuid> = links
        .iter()
        .map(|l| l.person_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let identity_ids: Vec<Uuid> = links
        .iter()
        .map(|l| l.identity_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let link_ids: Vec<Uuid> = links.iter().map(|l| l.id).collect();

    let (people, identities, traces) = tokio::join!(
        fetch_people_by_ids(pool, &person_ids),
        fetch_identities_by_ids(pool, &identity_ids),
        fetch_rule_traces(pool, &link_ids),
    );

    links
        .into_iter()
        .map(|link| {
            let person = people.get(&link.person_id).cloned();
            let identity = identities.get(&link.identity_id).cloned();
            let rule_trace = traces.get(&link.id).cloned();
            EnrichedLink::from_link(link, rule_trace, person, identity)
        })
        .collect()
}

// ── Handlers ────────────────────────────────────────────────────

pub async fn list_mappings(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Query(filter): Query<IdentityMappingFilter>,
) -> Result<Json<ListMappingsResponse>, ApiError> {
    validate_filter(&filter)?;
    let links = state.identity_repo.list_mappings(org, filter).await?;
    let pool = state.identity_repo.pool();
    let data = enrich_links(pool, links).await;
    let count = data.len();
    Ok(Json(ListMappingsResponse { data, count }))
}

pub async fn confirm_mapping(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Json(body): Json<ConfirmRequest>,
) -> Result<Json<MutationResponse>, ApiError> {
    validate_verified_by(&body.verified_by)?;
    state
        .identity_repo
        .confirm_mapping(org, body.link_id, &body.verified_by)
        .await?;
    Ok(Json(MutationResponse { ok: true }))
}

pub async fn remap_mapping(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Json(body): Json<RemapRequest>,
) -> Result<Json<MutationResponse>, ApiError> {
    validate_verified_by(&body.verified_by)?;
    state
        .identity_repo
        .remap_mapping(org, body.link_id, body.new_person_id, &body.verified_by)
        .await?;
    Ok(Json(MutationResponse { ok: true }))
}

pub async fn split_mapping(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Json(body): Json<SplitRequest>,
) -> Result<Json<MutationResponse>, ApiError> {
    validate_verified_by(&body.verified_by)?;
    state
        .identity_repo
        .split_mapping(org, body.link_id, &body.verified_by)
        .await?;
    Ok(Json(MutationResponse { ok: true }))
}

pub async fn list_conflicts(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Query(filter): Query<ConflictQueueFilter>,
) -> Result<Json<ConflictQueueResponse>, ApiError> {
    validate_conflict_filter(&filter)?;
    let links = state.identity_repo.list_conflicts(org, filter).await?;
    let pool = state.identity_repo.pool();
    let data = enrich_links(pool, links).await;
    let count = data.len();
    Ok(Json(ConflictQueueResponse { data, count }))
}

pub async fn bulk_confirm_conflicts(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Json(body): Json<BulkConfirmRequest>,
) -> Result<Json<BulkConfirmResponse>, ApiError> {
    validate_verified_by(&body.verified_by)?;
    if body.link_ids.is_empty() {
        return Err(ApiError(OviaError::Validation(
            "link_ids must not be empty".to_string(),
        )));
    }
    let result = state
        .identity_repo
        .bulk_confirm_conflicts(org, body.link_ids, &body.verified_by)
        .await?;
    Ok(Json(BulkConfirmResponse {
        confirmed: result.confirmed,
        failed: result.failed,
    }))
}

pub async fn export_conflicts_csv(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Query(filter): Query<ConflictQueueFilter>,
) -> Result<impl IntoResponse, ApiError> {
    validate_conflict_filter(&filter)?;
    let data = state.identity_repo.list_conflicts(org, filter).await?;
    let csv = format_conflicts_csv(&data);
    Ok((
        [
            (header::CONTENT_TYPE, "text/csv"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"conflict-queue.csv\"",
            ),
        ],
        csv,
    ))
}

pub async fn conflict_queue_stats(
    State(state): State<AppState>,
    OrgId(org): OrgId,
) -> Result<Json<ConflictQueueStatsResponse>, ApiError> {
    let stats = state.identity_repo.conflict_queue_stats(org).await?;
    Ok(Json(ConflictQueueStatsResponse {
        total: stats.total,
        avg_confidence: stats.avg_confidence,
        oldest_created_at: stats.oldest_created_at,
    }))
}
