use axum::extract::{Query, State};
use axum::Json;
use ovia_common::error::OviaError;
use ovia_db::identity::models::IdentityMappingFilter;
use ovia_db::identity::repositories::PersonIdentityLinkRepository;

use crate::error::ApiError;
use crate::extractors::OrgId;
use crate::identity::requests::{ConfirmRequest, RemapRequest, SplitRequest};
use crate::identity::responses::{ListMappingsResponse, MutationResponse};
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

pub async fn list_mappings(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Query(filter): Query<IdentityMappingFilter>,
) -> Result<Json<ListMappingsResponse>, ApiError> {
    validate_filter(&filter)?;
    let data = state.identity_repo.list_mappings(org, filter).await?;
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
