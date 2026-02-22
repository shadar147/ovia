use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use ovia_common::error::OviaError;
use ovia_db::identity::models::{Person, PersonFilter};
use ovia_db::identity::repositories::PersonRepository;
use sqlx::Row;
use uuid::Uuid;

use crate::error::ApiError;
use crate::extractors::OrgId;
use crate::people::requests::{CreatePersonRequest, LinkIdentityRequest, UpdatePersonRequest};
use crate::people::responses::{LinkResponse, ListPeopleResponse, PersonResponse};
use crate::AppState;

fn validate_email(email: &str) -> Result<(), OviaError> {
    if !email.contains('@') || !email.contains('.') {
        return Err(OviaError::Validation(format!(
            "invalid email format: {email}"
        )));
    }
    Ok(())
}

async fn identity_count_for_person(pool: &sqlx::PgPool, org_id: Uuid, person_id: Uuid) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "select count(*) from person_identity_links \
         where org_id = $1 and person_id = $2 and valid_to is null",
    )
    .bind(org_id)
    .bind(person_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0)
}

async fn identity_counts_for_people(
    pool: &sqlx::PgPool,
    org_id: Uuid,
    person_ids: &[Uuid],
) -> std::collections::HashMap<Uuid, i64> {
    if person_ids.is_empty() {
        return std::collections::HashMap::new();
    }
    let rows = sqlx::query(
        "select person_id, count(*) as cnt from person_identity_links \
         where org_id = $1 and person_id = any($2) and valid_to is null \
         group by person_id",
    )
    .bind(org_id)
    .bind(person_ids)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    rows.into_iter()
        .map(|r| {
            let pid: Uuid = r.get("person_id");
            let cnt: i64 = r.get("cnt");
            (pid, cnt)
        })
        .collect()
}

fn to_person_response(person: Person, identity_count: i64) -> PersonResponse {
    PersonResponse {
        id: person.id,
        display_name: person.display_name,
        primary_email: person.primary_email,
        avatar_url: person.avatar_url,
        team: person.team,
        role: person.role,
        status: person.status,
        identity_count,
        created_at: person.created_at,
        updated_at: person.updated_at,
    }
}

// ── Handlers ────────────────────────────────────────────────────

pub async fn list_people(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Query(filter): Query<PersonFilter>,
) -> Result<Json<ListPeopleResponse>, ApiError> {
    let (people, total) = PersonRepository::list(&state.identity_repo, org, filter).await?;
    let pool = state.identity_repo.pool();
    let person_ids: Vec<Uuid> = people.iter().map(|p| p.id).collect();
    let counts = identity_counts_for_people(pool, org, &person_ids).await;

    let data: Vec<PersonResponse> = people
        .into_iter()
        .map(|p| {
            let count = counts.get(&p.id).copied().unwrap_or(0);
            to_person_response(p, count)
        })
        .collect();

    let count = data.len();
    Ok(Json(ListPeopleResponse { data, count, total }))
}

pub async fn get_person(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Path(id): Path<Uuid>,
) -> Result<Json<PersonResponse>, ApiError> {
    let person = PersonRepository::get_by_id(&state.identity_repo, org, id)
        .await?
        .ok_or_else(|| ApiError(OviaError::NotFound(format!("person not found: {id}"))))?;

    let pool = state.identity_repo.pool();
    let count = identity_count_for_person(pool, org, id).await;
    Ok(Json(to_person_response(person, count)))
}

pub async fn create_person(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Json(body): Json<CreatePersonRequest>,
) -> Result<impl IntoResponse, ApiError> {
    if body.display_name.trim().is_empty() {
        return Err(ApiError(OviaError::Validation(
            "display_name must not be empty".to_string(),
        )));
    }
    if let Some(ref email) = body.primary_email {
        validate_email(email)?;
    }

    let person = Person {
        id: Uuid::new_v4(),
        org_id: org,
        display_name: body.display_name,
        primary_email: body.primary_email,
        avatar_url: body.avatar_url,
        team: body.team,
        role: body.role,
        status: "active".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created = PersonRepository::create(&state.identity_repo, person).await?;
    let resp = to_person_response(created, 0);
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn update_person(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdatePersonRequest>,
) -> Result<Json<PersonResponse>, ApiError> {
    let existing = PersonRepository::get_by_id(&state.identity_repo, org, id)
        .await?
        .ok_or_else(|| ApiError(OviaError::NotFound(format!("person not found: {id}"))))?;

    let display_name = body.display_name.unwrap_or(existing.display_name);
    if display_name.trim().is_empty() {
        return Err(ApiError(OviaError::Validation(
            "display_name must not be empty".to_string(),
        )));
    }

    let primary_email = if body.primary_email.is_some() {
        body.primary_email
    } else {
        existing.primary_email
    };
    if let Some(ref email) = primary_email {
        validate_email(email)?;
    }

    let person = Person {
        id,
        org_id: org,
        display_name,
        primary_email,
        avatar_url: if body.avatar_url.is_some() {
            body.avatar_url
        } else {
            existing.avatar_url
        },
        team: if body.team.is_some() {
            body.team
        } else {
            existing.team
        },
        role: if body.role.is_some() {
            body.role
        } else {
            existing.role
        },
        status: body.status.unwrap_or(existing.status),
        created_at: existing.created_at,
        updated_at: chrono::Utc::now(),
    };

    let updated = PersonRepository::update(&state.identity_repo, person).await?;
    let pool = state.identity_repo.pool();
    let count = identity_count_for_person(pool, org, id).await;
    Ok(Json(to_person_response(updated, count)))
}

pub async fn delete_person(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    PersonRepository::soft_delete(&state.identity_repo, org, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn link_identity(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Path(person_id): Path<Uuid>,
    Json(body): Json<LinkIdentityRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Verify person exists
    let _person = PersonRepository::get_by_id(&state.identity_repo, org, person_id)
        .await?
        .ok_or_else(|| {
            ApiError(OviaError::NotFound(format!(
                "person not found: {person_id}"
            )))
        })?;

    let link_status = body.status.as_deref().unwrap_or("verified");
    let confidence = body.confidence.unwrap_or(1.0);
    let link_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let pool = state.identity_repo.pool();
    let row = sqlx::query(
        "insert into person_identity_links \
         (id, org_id, person_id, identity_id, status, confidence, valid_from, verified_at, created_at, updated_at) \
         values ($1, $2, $3, $4, $5, $6, $7, $7, $7, $7) \
         returning id, person_id, identity_id, status, confidence::float8 as confidence",
    )
    .bind(link_id)
    .bind(org)
    .bind(person_id)
    .bind(body.identity_id)
    .bind(link_status)
    .bind(confidence)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("duplicate key") || msg.contains("unique constraint") {
            OviaError::Validation("identity is already linked to this person".to_string())
        } else if msg.contains("violates foreign key") {
            OviaError::NotFound(format!("identity not found: {}", body.identity_id))
        } else {
            OviaError::Database(msg)
        }
    })?;

    let resp = LinkResponse {
        id: row.get("id"),
        person_id: row.get("person_id"),
        identity_id: row.get("identity_id"),
        status: row.get("status"),
        confidence: row.get("confidence"),
    };

    Ok((StatusCode::CREATED, Json(resp)))
}
