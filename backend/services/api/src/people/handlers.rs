use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use ovia_common::error::OviaError;
use ovia_db::identity::models::{Person, PersonFilter};
use ovia_db::identity::repositories::{IdentityRepository, PersonRepository};
use sqlx::Row;
use uuid::Uuid;

use crate::error::ApiError;
use crate::extractors::OrgId;
use crate::people::requests::{CreatePersonRequest, LinkIdentityRequest, UpdatePersonRequest};
use crate::people::responses::{
    LinkResponse, LinkedIdentitiesResponse, LinkedIdentityResponse, ListPeopleResponse,
    PersonResponse,
};
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
    // Verify person exists in this org
    let _person = PersonRepository::get_by_id(&state.identity_repo, org, person_id)
        .await?
        .ok_or_else(|| {
            ApiError(OviaError::NotFound(format!(
                "person not found: {person_id}"
            )))
        })?;

    // Verify identity exists in same org
    let _identity = IdentityRepository::get_by_id(&state.identity_repo, org, body.identity_id)
        .await?
        .ok_or_else(|| {
            ApiError(OviaError::NotFound(format!(
                "identity not found: {}",
                body.identity_id
            )))
        })?;

    let pool = state.identity_repo.pool();

    // Check if identity is already actively linked to another person
    let existing = sqlx::query(
        "select person_id from person_identity_links \
         where org_id = $1 and identity_id = $2 and valid_to is null \
         limit 1",
    )
    .bind(org)
    .bind(body.identity_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| OviaError::Database(e.to_string()))?;

    if let Some(row) = existing {
        let existing_person_id: Uuid = row.get("person_id");
        if existing_person_id == person_id {
            return Err(ApiError(OviaError::Validation(
                "identity is already linked to this person".to_string(),
            )));
        }
        return Err(ApiError(OviaError::Conflict(format!(
            "identity {} is already linked to person {existing_person_id}; use remap to reassign",
            body.identity_id
        ))));
    }

    let link_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    // Use a transaction to insert link + audit event atomically
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

    let row = sqlx::query(
        "insert into person_identity_links \
         (id, org_id, person_id, identity_id, status, confidence, valid_from, verified_by, verified_at, created_at, updated_at) \
         values ($1, $2, $3, $4, 'verified', 1.0, $5, 'manual', $5, $5, $5) \
         returning id, person_id, identity_id, status, confidence::float8 as confidence",
    )
    .bind(link_id)
    .bind(org)
    .bind(person_id)
    .bind(body.identity_id)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| OviaError::Database(e.to_string()))?;

    // Emit audit event
    sqlx::query(
        "insert into identity_events (id, org_id, link_id, action, actor, payload, created_at) \
         values ($1, $2, $3, 'manual_link', 'manual', $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(org)
    .bind(link_id)
    .bind(serde_json::json!({
        "person_id": person_id,
        "identity_id": body.identity_id,
    }))
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| OviaError::Database(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

    let resp = LinkResponse {
        id: row.get("id"),
        person_id: row.get("person_id"),
        identity_id: row.get("identity_id"),
        status: row.get("status"),
        confidence: row.get("confidence"),
    };

    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn unlink_identity(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Path((person_id, identity_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let pool = state.identity_repo.pool();
    let now = chrono::Utc::now();

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

    let row = sqlx::query(
        "update person_identity_links \
         set valid_to = $1, updated_at = $1 \
         where org_id = $2 and person_id = $3 and identity_id = $4 and valid_to is null \
         returning id",
    )
    .bind(now)
    .bind(org)
    .bind(person_id)
    .bind(identity_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| OviaError::Database(e.to_string()))?;

    let link_id: Uuid = match row {
        Some(r) => r.get("id"),
        None => {
            return Err(ApiError(OviaError::NotFound(format!(
                "active link not found for person {person_id} and identity {identity_id}"
            ))));
        }
    };

    // Emit audit event
    sqlx::query(
        "insert into identity_events (id, org_id, link_id, action, actor, payload, created_at) \
         values ($1, $2, $3, 'manual_unlink', 'manual', $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(org)
    .bind(link_id)
    .bind(serde_json::json!({
        "person_id": person_id,
        "identity_id": identity_id,
    }))
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| OviaError::Database(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_person_identities(
    State(state): State<AppState>,
    OrgId(org): OrgId,
    Path(person_id): Path<Uuid>,
) -> Result<Json<LinkedIdentitiesResponse>, ApiError> {
    // Verify person exists
    let _person = PersonRepository::get_by_id(&state.identity_repo, org, person_id)
        .await?
        .ok_or_else(|| {
            ApiError(OviaError::NotFound(format!(
                "person not found: {person_id}"
            )))
        })?;

    let pool = state.identity_repo.pool();
    let rows = sqlx::query(
        "select pil.id as link_id, pil.identity_id, pil.status, \
                pil.confidence::float8 as confidence, pil.created_at as linked_at, \
                i.source, i.username, i.email, i.display_name \
         from person_identity_links pil \
         join identities i on pil.identity_id = i.id \
         where pil.org_id = $1 and pil.person_id = $2 and pil.valid_to is null \
         order by pil.created_at desc",
    )
    .bind(org)
    .bind(person_id)
    .fetch_all(pool)
    .await
    .map_err(|e| OviaError::Database(e.to_string()))?;

    let data: Vec<LinkedIdentityResponse> = rows
        .into_iter()
        .map(|r| LinkedIdentityResponse {
            link_id: r.get("link_id"),
            identity_id: r.get("identity_id"),
            source: r.get("source"),
            username: r.get("username"),
            email: r.get("email"),
            display_name: r.get("display_name"),
            status: r.get("status"),
            confidence: r.get("confidence"),
            linked_at: r.get("linked_at"),
        })
        .collect();

    let count = data.len();
    Ok(Json(LinkedIdentitiesResponse { data, count }))
}
