mod ask;

use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use ovia_common::error::OviaError;
use ovia_config::{init_tracing, AppConfig};
use ovia_db::ask::models::AskFilter;
use ovia_db::ask::pg_repository::PgAskRepository;
use ovia_db::ask::repositories::AskRepository;
use ovia_db::kpi::pg_repository::PgKpiRepository;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

use ask::engine::{AskEngine, AskResponse};
use ask::filters::AskFilters;

#[derive(Clone)]
struct RagState {
    ask_repo: PgAskRepository,
    kpi_repo: PgKpiRepository,
}

// ── Request/Response types ──────────────────────────────────────

#[derive(Debug, Deserialize)]
struct AskRequest {
    query: String,
    filters: Option<AskFilters>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

fn extract_org_id(
    headers: &axum::http::HeaderMap,
) -> Result<Uuid, (axum::http::StatusCode, Json<ErrorResponse>)> {
    let header = headers.get("X-Org-Id").ok_or_else(|| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "missing X-Org-Id header".to_string(),
            }),
        )
    })?;

    let value = header.to_str().map_err(|_| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "invalid X-Org-Id header value".to_string(),
            }),
        )
    })?;

    Uuid::parse_str(value).map_err(|_| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("invalid UUID in X-Org-Id: {value}"),
            }),
        )
    })
}

fn map_err(e: OviaError) -> (axum::http::StatusCode, Json<ErrorResponse>) {
    let (status, message) = match &e {
        OviaError::NotFound(msg) => (axum::http::StatusCode::NOT_FOUND, msg.clone()),
        OviaError::Validation(msg) => (axum::http::StatusCode::BAD_REQUEST, msg.clone()),
        other => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            other.to_string(),
        ),
    };
    (status, Json(ErrorResponse { error: message }))
}

// ── Handlers ────────────────────────────────────────────────────

async fn post_ask(
    State(state): State<RagState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<AskRequest>,
) -> Result<Json<AskResponse>, (axum::http::StatusCode, Json<ErrorResponse>)> {
    let org_id = extract_org_id(&headers)?;

    if body.query.trim().is_empty() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "query must not be empty".to_string(),
            }),
        ));
    }

    let engine = AskEngine::new(state.ask_repo.clone(), state.kpi_repo.clone());
    let response = engine
        .answer(org_id, &body.query, body.filters)
        .await
        .map_err(map_err)?;

    Ok(Json(response))
}

#[derive(Debug, Serialize)]
struct SessionResponse {
    data: ovia_db::ask::models::AskSession,
}

async fn get_ask_session(
    State(state): State<RagState>,
    headers: axum::http::HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<SessionResponse>, (axum::http::StatusCode, Json<ErrorResponse>)> {
    let org_id = extract_org_id(&headers)?;

    let session = state
        .ask_repo
        .get_session(org_id, id)
        .await
        .map_err(map_err)?
        .ok_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "session not found".to_string(),
                }),
            )
        })?;

    Ok(Json(SessionResponse { data: session }))
}

#[derive(Debug, Serialize)]
struct SessionListResponse {
    data: Vec<ovia_db::ask::models::AskSession>,
    count: usize,
}

async fn list_ask_history(
    State(state): State<RagState>,
    headers: axum::http::HeaderMap,
    Query(mut filter): Query<AskFilter>,
) -> Result<Json<SessionListResponse>, (axum::http::StatusCode, Json<ErrorResponse>)> {
    let org_id = extract_org_id(&headers)?;
    filter.org_id = Some(org_id);

    let data = state
        .ask_repo
        .list_sessions(filter)
        .await
        .map_err(map_err)?;
    let count = data.len();

    Ok(Json(SessionListResponse { data, count }))
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

fn build_router(state: RagState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ask", post(post_ask))
        .route("/ask/history", get(list_ask_history))
        .route("/ask/{id}", get(get_ask_session))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    init_tracing("info");

    let config = AppConfig::from_env().expect("failed to load config");
    tracing::info!(service = "ovia-rag", "starting");

    let pool = ovia_db::create_pool(&config.database_url)
        .await
        .expect("failed to create database pool");

    let state = RagState {
        ask_repo: PgAskRepository::new(pool.clone()),
        kpi_repo: PgKpiRepository::new(pool),
    };

    let rag_port = std::env::var("RAG_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(config.port + 2);

    let addr: SocketAddr = format!("{}:{}", config.host, rag_port)
        .parse()
        .expect("invalid bind address");

    let app = build_router(state);

    tracing::info!(%addr, "rag service listening");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");
    axum::serve(listener, app).await.expect("server error");
}
