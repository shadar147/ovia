mod ask;
mod error;
mod extractors;
mod identity;
mod kpi;
mod people;

use axum::{
    http::{header, HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use ovia_common::types::ServiceInfo;
use ovia_config::{init_tracing, AppConfig};
use ovia_db::ask::pg_repository::PgAskRepository;
use ovia_db::identity::pg_repository::PgIdentityRepository;
use ovia_db::kpi::pg_repository::PgKpiRepository;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub identity_repo: PgIdentityRepository,
    pub kpi_repo: PgKpiRepository,
    pub ask_repo: PgAskRepository,
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn info() -> Json<ServiceInfo> {
    Json(ServiceInfo::new("ovia-api"))
}

async fn metrics() -> impl IntoResponse {
    let body = "\
# HELP ovia_up Service up indicator\n\
# TYPE ovia_up gauge\n\
ovia_up 1\n\
# HELP ovia_info Service info\n\
# TYPE ovia_info gauge\n\
ovia_info{service=\"ovia-api\",version=\"0.1.0\"} 1\n";

    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        body,
    )
}

fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:3000".parse::<HeaderValue>().unwrap(),
            "http://127.0.0.1:3000".parse::<HeaderValue>().unwrap(),
        ])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            "x-org-id".parse().unwrap(),
        ]);

    Router::new()
        .route("/health", get(health))
        .route("/info", get(info))
        .route("/metrics", get(metrics))
        .merge(identity::router())
        .merge(kpi::router())
        .merge(ask::router())
        .merge(people::router())
        .layer(cors)
        .with_state(state)
}

#[tokio::main]
async fn main() {
    init_tracing("info");

    let config = AppConfig::from_env().expect("failed to load config");
    tracing::info!(service = "ovia-api", "starting");

    let pool = ovia_db::create_pool(&config.database_url)
        .await
        .expect("failed to create database pool");

    let state = AppState {
        identity_repo: PgIdentityRepository::new(pool.clone()),
        kpi_repo: PgKpiRepository::new(pool.clone()),
        ask_repo: PgAskRepository::new(pool),
    };

    let app = build_router(state);
    let addr: SocketAddr = config.bind_addr().parse().expect("invalid bind address");

    tracing::info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");
    axum::serve(listener, app).await.expect("server error");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use sqlx::{PgPool, Row};
    use tower::ServiceExt;
    use uuid::Uuid;

    async fn test_state() -> Option<(AppState, PgPool)> {
        let url = std::env::var("TEST_DATABASE_URL").ok()?;
        let pool = ovia_db::create_pool(&url).await.expect("db should connect");
        let state = AppState {
            identity_repo: PgIdentityRepository::new(pool.clone()),
            kpi_repo: PgKpiRepository::new(pool.clone()),
            ask_repo: PgAskRepository::new(pool.clone()),
        };
        Some((state, pool))
    }

    async fn insert_person(pool: &PgPool, org_id: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query("insert into people (id, org_id, display_name) values ($1, $2, 'test-person')")
            .bind(id)
            .bind(org_id)
            .execute(pool)
            .await
            .expect("insert person");
        id
    }

    async fn insert_identity(pool: &PgPool, org_id: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query("insert into identities (id, org_id, source) values ($1, $2, 'test-source')")
            .bind(id)
            .bind(org_id)
            .execute(pool)
            .await
            .expect("insert identity");
        id
    }

    async fn insert_link(pool: &PgPool, org_id: Uuid, person_id: Uuid, identity_id: Uuid) -> Uuid {
        insert_link_with(pool, org_id, person_id, identity_id, "auto", 0.8).await
    }

    async fn insert_link_with(
        pool: &PgPool,
        org_id: Uuid,
        person_id: Uuid,
        identity_id: Uuid,
        status: &str,
        confidence: f64,
    ) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "insert into person_identity_links \
             (id, org_id, person_id, identity_id, status, confidence) \
             values ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id)
        .bind(org_id)
        .bind(person_id)
        .bind(identity_id)
        .bind(status)
        .bind(confidence)
        .execute(pool)
        .await
        .expect("insert link");
        id
    }

    async fn read_body(resp: axum::http::Response<Body>) -> serde_json::Value {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    async fn read_body_string(resp: axum::http::Response<Body>) -> String {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    // ── Health / Info (no DB needed) ────────────────────────────────

    #[tokio::test]
    async fn health_returns_ok() {
        let (state, _pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let app = build_router(state);
        let resp = app
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn metrics_returns_prometheus_format() {
        let (state, _pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let app = build_router(state);
        let resp = app
            .oneshot(Request::get("/metrics").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap(),
            "text/plain; version=0.0.4; charset=utf-8"
        );
        let body = read_body_string(resp).await;
        assert!(body.contains("ovia_up 1"));
        assert!(body.contains("ovia_info{service=\"ovia-api\",version=\"0.1.0\"} 1"));
    }

    #[tokio::test]
    async fn info_returns_service_name() {
        let (state, _pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let app = build_router(state);
        let resp = app
            .oneshot(Request::get("/info").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // ── GET /team/identity-mappings ─────────────────────────────────

    #[tokio::test]
    async fn list_empty_org_returns_empty() {
        let (state, _pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let app = build_router(state);
        let org = Uuid::new_v4();
        let resp = app
            .oneshot(
                Request::get("/team/identity-mappings")
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_body(resp).await;
        assert_eq!(body["data"], serde_json::json!([]));
        assert_eq!(body["count"], 0);
    }

    #[tokio::test]
    async fn list_missing_org_id_returns_400() {
        let (state, _pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let app = build_router(state);
        let resp = app
            .oneshot(
                Request::get("/team/identity-mappings")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = read_body(resp).await;
        assert!(body["error"].as_str().unwrap().contains("X-Org-Id"));
    }

    #[tokio::test]
    async fn list_invalid_uuid_returns_400() {
        let (state, _pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let app = build_router(state);
        let resp = app
            .oneshot(
                Request::get("/team/identity-mappings")
                    .header("X-Org-Id", "not-a-uuid")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = read_body(resp).await;
        assert!(body["error"].as_str().unwrap().contains("UUID"));
    }

    #[tokio::test]
    async fn list_bad_confidence_range_returns_400() {
        let (state, _pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let app = build_router(state);
        let org = Uuid::new_v4();
        let resp = app
            .oneshot(
                Request::get("/team/identity-mappings?min_confidence=0.9&max_confidence=0.1")
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = read_body(resp).await;
        assert!(body["error"].as_str().unwrap().contains("confidence"));
    }

    // ── POST /team/identity-mappings/confirm ────────────────────────

    #[tokio::test]
    async fn confirm_not_found_returns_404() {
        let (state, _pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let app = build_router(state);
        let org = Uuid::new_v4();
        let body = serde_json::json!({
            "link_id": Uuid::new_v4(),
            "verified_by": "tester"
        });
        let resp = app
            .oneshot(
                Request::post("/team/identity-mappings/confirm")
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn confirm_empty_verified_by_returns_400() {
        let (state, _pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let app = build_router(state);
        let org = Uuid::new_v4();
        let body = serde_json::json!({
            "link_id": Uuid::new_v4(),
            "verified_by": ""
        });
        let resp = app
            .oneshot(
                Request::post("/team/identity-mappings/confirm")
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let resp_body = read_body(resp).await;
        assert!(resp_body["error"].as_str().unwrap().contains("verified_by"));
    }

    #[tokio::test]
    async fn confirm_happy_path() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let org = Uuid::new_v4();
        let person = insert_person(&pool, org).await;
        let identity = insert_identity(&pool, org).await;
        let link_id = insert_link(&pool, org, person, identity).await;

        let app = build_router(state);
        let body = serde_json::json!({
            "link_id": link_id,
            "verified_by": "tester"
        });
        let resp = app
            .oneshot(
                Request::post("/team/identity-mappings/confirm")
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let resp_body = read_body(resp).await;
        assert_eq!(resp_body["ok"], true);
    }

    // ── POST /team/identity-mappings/remap ──────────────────────────

    #[tokio::test]
    async fn remap_not_found_returns_404() {
        let (state, _pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let app = build_router(state);
        let org = Uuid::new_v4();
        let body = serde_json::json!({
            "link_id": Uuid::new_v4(),
            "new_person_id": Uuid::new_v4(),
            "verified_by": "tester"
        });
        let resp = app
            .oneshot(
                Request::post("/team/identity-mappings/remap")
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn remap_happy_path() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let org = Uuid::new_v4();
        let person = insert_person(&pool, org).await;
        let new_person = insert_person(&pool, org).await;
        let identity = insert_identity(&pool, org).await;
        let link_id = insert_link(&pool, org, person, identity).await;

        let app = build_router(state);
        let body = serde_json::json!({
            "link_id": link_id,
            "new_person_id": new_person,
            "verified_by": "tester"
        });
        let resp = app
            .oneshot(
                Request::post("/team/identity-mappings/remap")
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let resp_body = read_body(resp).await;
        assert_eq!(resp_body["ok"], true);
    }

    // ── POST /team/identity-mappings/split ──────────────────────────

    #[tokio::test]
    async fn split_not_found_returns_404() {
        let (state, _pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let app = build_router(state);
        let org = Uuid::new_v4();
        let body = serde_json::json!({
            "link_id": Uuid::new_v4(),
            "verified_by": "tester"
        });
        let resp = app
            .oneshot(
                Request::post("/team/identity-mappings/split")
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn split_happy_path() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let org = Uuid::new_v4();
        let person = insert_person(&pool, org).await;
        let identity = insert_identity(&pool, org).await;
        let link_id = insert_link(&pool, org, person, identity).await;

        let app = build_router(state);
        let body = serde_json::json!({
            "link_id": link_id,
            "verified_by": "tester"
        });
        let resp = app
            .oneshot(
                Request::post("/team/identity-mappings/split")
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let resp_body = read_body(resp).await;
        assert_eq!(resp_body["ok"], true);
    }

    // ── GET /team/conflict-queue ──────────────────────────────────────

    #[tokio::test]
    async fn conflict_queue_empty_returns_empty() {
        let (state, _pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let app = build_router(state);
        let org = Uuid::new_v4();
        let resp = app
            .oneshot(
                Request::get("/team/conflict-queue")
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_body(resp).await;
        assert_eq!(body["data"], serde_json::json!([]));
        assert_eq!(body["count"], 0);
    }

    #[tokio::test]
    async fn conflict_queue_with_data() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let org = Uuid::new_v4();
        let person = insert_person(&pool, org).await;
        let identity = insert_identity(&pool, org).await;
        insert_link_with(&pool, org, person, identity, "conflict", 0.5).await;

        let app = build_router(state);
        let resp = app
            .oneshot(
                Request::get("/team/conflict-queue")
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_body(resp).await;
        assert_eq!(body["count"], 1);
        assert_eq!(body["data"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn conflict_queue_invalid_sort_returns_400() {
        let (state, _pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let app = build_router(state);
        let org = Uuid::new_v4();
        let resp = app
            .oneshot(
                Request::get("/team/conflict-queue?sort_by=invalid")
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = read_body(resp).await;
        assert!(body["error"].as_str().unwrap().contains("sort_by"));
    }

    // ── POST /team/conflict-queue/bulk-confirm ────────────────────────

    #[tokio::test]
    async fn bulk_confirm_returns_result() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let org = Uuid::new_v4();
        let person = insert_person(&pool, org).await;
        let identity = insert_identity(&pool, org).await;
        let link_id = insert_link_with(&pool, org, person, identity, "conflict", 0.5).await;

        let app = build_router(state);
        let body = serde_json::json!({
            "link_ids": [link_id],
            "verified_by": "tester"
        });
        let resp = app
            .oneshot(
                Request::post("/team/conflict-queue/bulk-confirm")
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let resp_body = read_body(resp).await;
        assert_eq!(resp_body["confirmed"], 1);
        assert_eq!(resp_body["failed"], serde_json::json!([]));
    }

    #[tokio::test]
    async fn bulk_confirm_empty_verified_by_returns_400() {
        let (state, _pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let app = build_router(state);
        let org = Uuid::new_v4();
        let body = serde_json::json!({
            "link_ids": [Uuid::new_v4()],
            "verified_by": ""
        });
        let resp = app
            .oneshot(
                Request::post("/team/conflict-queue/bulk-confirm")
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let resp_body = read_body(resp).await;
        assert!(resp_body["error"].as_str().unwrap().contains("verified_by"));
    }

    #[tokio::test]
    async fn bulk_confirm_empty_link_ids_returns_400() {
        let (state, _pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let app = build_router(state);
        let org = Uuid::new_v4();
        let body = serde_json::json!({
            "link_ids": [],
            "verified_by": "tester"
        });
        let resp = app
            .oneshot(
                Request::post("/team/conflict-queue/bulk-confirm")
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let resp_body = read_body(resp).await;
        assert!(resp_body["error"].as_str().unwrap().contains("link_ids"));
    }

    // ── GET /team/conflict-queue/export ───────────────────────────────

    #[tokio::test]
    async fn conflict_queue_export_csv() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let org = Uuid::new_v4();
        let person = insert_person(&pool, org).await;
        let identity = insert_identity(&pool, org).await;
        insert_link_with(&pool, org, person, identity, "conflict", 0.5).await;

        let app = build_router(state);
        let resp = app
            .oneshot(
                Request::get("/team/conflict-queue/export")
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get("content-type")
                .unwrap()
                .to_str()
                .unwrap(),
            "text/csv"
        );
        let body = read_body_string(resp).await;
        let lines: Vec<&str> = body.lines().collect();
        assert_eq!(
            lines[0],
            "id,person_id,identity_id,status,confidence,created_at"
        );
        assert!(lines.len() >= 2);
    }

    // ── GET /team/conflict-queue/stats ────────────────────────────────

    #[tokio::test]
    async fn conflict_queue_stats_returns_counts() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        let org = Uuid::new_v4();
        let person = insert_person(&pool, org).await;
        let identity = insert_identity(&pool, org).await;
        insert_link_with(&pool, org, person, identity, "conflict", 0.5).await;

        let app = build_router(state);
        let resp = app
            .oneshot(
                Request::get("/team/conflict-queue/stats")
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_body(resp).await;
        assert_eq!(body["total"], 1);
        assert!(body["avg_confidence"].as_f64().is_some());
        assert!(body["oldest_created_at"].as_str().is_some());
    }

    // ── KPI endpoint tests ───────────────────────────────────────────

    async fn ensure_kpi_tables(pool: &PgPool) {
        sqlx::query(
            "create table if not exists kpi_snapshots (
              id uuid primary key default gen_random_uuid(),
              org_id uuid not null,
              period_start date not null,
              period_end date not null,
              delivery_health_score numeric(5,2),
              release_risk_score numeric(5,2),
              throughput_total integer not null default 0,
              throughput_bugs integer not null default 0,
              throughput_features integer not null default 0,
              throughput_chores integer not null default 0,
              review_latency_median_hours numeric(8,2),
              review_latency_p90_hours numeric(8,2),
              computed_at timestamptz not null default now(),
              created_at timestamptz not null default now()
            )",
        )
        .execute(pool)
        .await
        .expect("create kpi_snapshots");

        sqlx::query(
            "create unique index if not exists kpi_snapshots_org_period_uidx
             on kpi_snapshots(org_id, period_start, period_end)",
        )
        .execute(pool)
        .await
        .expect("create kpi index");

        // Jira KPI columns (migration 0007)
        for stmt in &[
            "alter table kpi_snapshots add column if not exists blocker_count integer not null default 0",
            "alter table kpi_snapshots add column if not exists spillover_rate numeric(5,4)",
            "alter table kpi_snapshots add column if not exists cycle_time_p50_hours numeric(8,2)",
            "alter table kpi_snapshots add column if not exists cycle_time_p90_hours numeric(8,2)",
        ] {
            sqlx::query(stmt)
                .execute(pool)
                .await
                .expect("alter kpi_snapshots for jira columns");
        }

        sqlx::query(
            "create table if not exists risk_items (
              id uuid primary key default gen_random_uuid(),
              org_id uuid not null,
              snapshot_id uuid not null references kpi_snapshots(id) on delete cascade,
              entity_type text not null,
              title text not null,
              owner text,
              age_days integer not null default 0,
              impact_scope text,
              status text not null,
              source_url text,
              created_at timestamptz not null default now()
            )",
        )
        .execute(pool)
        .await
        .expect("create risk_items");
    }

    async fn insert_kpi_snapshot(pool: &PgPool, org_id: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "insert into kpi_snapshots (id, org_id, period_start, period_end, \
             delivery_health_score, release_risk_score, throughput_total, throughput_bugs, \
             throughput_features, throughput_chores, review_latency_median_hours, \
             blocker_count, spillover_rate, cycle_time_p50_hours, cycle_time_p90_hours) \
             values ($1, $2, '2026-02-01', '2026-02-14', 75.5, 30.0, 42, 10, 25, 7, 4.5, \
             3, 0.25, 36.0, 72.0)",
        )
        .bind(id)
        .bind(org_id)
        .execute(pool)
        .await
        .expect("insert kpi snapshot");
        id
    }

    #[tokio::test]
    async fn kpi_latest_returns_snapshot() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_kpi_tables(&pool).await;
        let org = Uuid::new_v4();
        insert_kpi_snapshot(&pool, org).await;

        let app = build_router(state);
        let resp = app
            .oneshot(
                Request::get("/team/kpi")
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_body(resp).await;
        let data = &body["data"];
        assert_eq!(data["throughput_total"], 42);
        // Jira metrics present in response
        assert_eq!(data["blocker_count"], 3);
        assert!((data["spillover_rate"].as_f64().unwrap() - 0.25).abs() < 0.01);
        assert!((data["cycle_time_p50_hours"].as_f64().unwrap() - 36.0).abs() < 0.1);
        assert!((data["cycle_time_p90_hours"].as_f64().unwrap() - 72.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn kpi_latest_returns_404_when_empty() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_kpi_tables(&pool).await;
        let org = Uuid::new_v4();

        let app = build_router(state);
        let resp = app
            .oneshot(
                Request::get("/team/kpi")
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn kpi_history_returns_list() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_kpi_tables(&pool).await;
        let org = Uuid::new_v4();
        insert_kpi_snapshot(&pool, org).await;

        let app = build_router(state);
        let resp = app
            .oneshot(
                Request::get("/team/kpi/history")
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_body(resp).await;
        assert_eq!(body["count"], 1);
        assert_eq!(body["data"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn kpi_response_contract_includes_all_fields() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_kpi_tables(&pool).await;
        let org = Uuid::new_v4();
        insert_kpi_snapshot(&pool, org).await;

        let app = build_router(state);
        let resp = app
            .oneshot(
                Request::get("/team/kpi")
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_body(resp).await;
        let data = &body["data"];

        // All KPI fields must be present in the contract
        let required_fields = [
            "id",
            "org_id",
            "period_start",
            "period_end",
            "delivery_health_score",
            "release_risk_score",
            "throughput_total",
            "throughput_bugs",
            "throughput_features",
            "throughput_chores",
            "review_latency_median_hours",
            "review_latency_p90_hours",
            "blocker_count",
            "spillover_rate",
            "cycle_time_p50_hours",
            "cycle_time_p90_hours",
            "computed_at",
            "created_at",
        ];
        for field in &required_fields {
            assert!(
                !data[field].is_null() || data.get(field).is_some(),
                "field '{}' missing from KPI response",
                field
            );
        }
    }

    #[tokio::test]
    async fn kpi_history_includes_jira_metrics() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_kpi_tables(&pool).await;
        let org = Uuid::new_v4();
        insert_kpi_snapshot(&pool, org).await;

        let app = build_router(state);
        let resp = app
            .oneshot(
                Request::get("/team/kpi/history")
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_body(resp).await;
        let item = &body["data"][0];
        assert_eq!(item["blocker_count"], 3);
        assert!(item["spillover_rate"].as_f64().is_some());
        assert!(item["cycle_time_p50_hours"].as_f64().is_some());
    }

    #[tokio::test]
    async fn kpi_risks_returns_404_when_no_snapshot() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_kpi_tables(&pool).await;
        let org = Uuid::new_v4();

        let app = build_router(state);
        let resp = app
            .oneshot(
                Request::get("/team/kpi/risks")
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // ── Ask endpoint tests ───────────────────────────────────────────

    async fn ensure_ask_tables(pool: &PgPool) {
        sqlx::query(
            "create table if not exists ask_sessions (
              id uuid primary key default gen_random_uuid(),
              org_id uuid not null,
              query text not null,
              answer text,
              confidence text,
              assumptions text,
              citations jsonb,
              filters jsonb,
              model text,
              prompt_tokens integer,
              completion_tokens integer,
              latency_ms integer,
              created_at timestamptz not null default now()
            )",
        )
        .execute(pool)
        .await
        .expect("create ask_sessions");
    }

    #[tokio::test]
    async fn ask_post_returns_stub_response() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_kpi_tables(&pool).await;
        ensure_ask_tables(&pool).await;
        let org = Uuid::new_v4();

        let app = build_router(state);
        let body = serde_json::json!({
            "query": "What is our delivery health?"
        });
        let resp = app
            .oneshot(
                Request::post("/ask")
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let resp_body = read_body(resp).await;
        assert!(resp_body["answer"].as_str().is_some());
        assert!(resp_body["confidence"].as_str().is_some());
        assert!(resp_body["session_id"].as_str().is_some());
    }

    #[tokio::test]
    async fn ask_post_empty_query_returns_400() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_kpi_tables(&pool).await;
        ensure_ask_tables(&pool).await;
        let org = Uuid::new_v4();

        let app = build_router(state);
        let body = serde_json::json!({
            "query": ""
        });
        let resp = app
            .oneshot(
                Request::post("/ask")
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let resp_body = read_body(resp).await;
        assert!(resp_body["error"].as_str().unwrap().contains("query"));
    }

    #[tokio::test]
    async fn ask_get_session_returns_404_for_nonexistent() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_ask_tables(&pool).await;
        let org = Uuid::new_v4();

        let app = build_router(state);
        let resp = app
            .oneshot(
                Request::get(format!("/ask/{}", Uuid::new_v4()))
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn ask_history_returns_empty_list() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_ask_tables(&pool).await;
        let org = Uuid::new_v4();

        let app = build_router(state);
        let resp = app
            .oneshot(
                Request::get("/ask/history")
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_body(resp).await;
        assert_eq!(body["count"], 0);
        assert_eq!(body["data"], serde_json::json!([]));
    }

    // ── People CRUD endpoint tests ──────────────────────────────────

    async fn ensure_avatar_column(pool: &PgPool) {
        sqlx::query("alter table people add column if not exists avatar_url text")
            .execute(pool)
            .await
            .expect("add avatar_url column");
    }

    #[tokio::test]
    async fn people_list_empty_returns_empty() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_avatar_column(&pool).await;
        let app = build_router(state);
        let org = Uuid::new_v4();
        let resp = app
            .oneshot(
                Request::get("/team/people")
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_body(resp).await;
        assert_eq!(body["data"], serde_json::json!([]));
        assert_eq!(body["count"], 0);
        assert_eq!(body["total"], 0);
    }

    #[tokio::test]
    async fn people_create_happy_path() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_avatar_column(&pool).await;
        let app = build_router(state);
        let org = Uuid::new_v4();
        let body = serde_json::json!({
            "display_name": "Alice Smith",
            "primary_email": "alice@example.com",
            "team": "Platform",
            "role": "Engineer"
        });
        let resp = app
            .oneshot(
                Request::post("/team/people")
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let resp_body = read_body(resp).await;
        assert_eq!(resp_body["display_name"], "Alice Smith");
        assert_eq!(resp_body["primary_email"], "alice@example.com");
        assert_eq!(resp_body["team"], "Platform");
        assert_eq!(resp_body["role"], "Engineer");
        assert_eq!(resp_body["status"], "active");
        assert_eq!(resp_body["identity_count"], 0);
        assert!(resp_body["id"].as_str().is_some());
    }

    #[tokio::test]
    async fn people_create_empty_name_returns_400() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_avatar_column(&pool).await;
        let app = build_router(state);
        let org = Uuid::new_v4();
        let body = serde_json::json!({
            "display_name": "",
            "primary_email": "bob@example.com"
        });
        let resp = app
            .oneshot(
                Request::post("/team/people")
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let resp_body = read_body(resp).await;
        assert!(resp_body["error"]
            .as_str()
            .unwrap()
            .contains("display_name"));
    }

    #[tokio::test]
    async fn people_create_invalid_email_returns_400() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_avatar_column(&pool).await;
        let app = build_router(state);
        let org = Uuid::new_v4();
        let body = serde_json::json!({
            "display_name": "Bob",
            "primary_email": "not-an-email"
        });
        let resp = app
            .oneshot(
                Request::post("/team/people")
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let resp_body = read_body(resp).await;
        assert!(resp_body["error"].as_str().unwrap().contains("email"));
    }

    #[tokio::test]
    async fn people_get_returns_person_with_identity_count() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_avatar_column(&pool).await;
        let org = Uuid::new_v4();
        let person_id = insert_person(&pool, org).await;
        let identity_id = insert_identity(&pool, org).await;
        insert_link(&pool, org, person_id, identity_id).await;

        let app = build_router(state);
        let resp = app
            .oneshot(
                Request::get(format!("/team/people/{person_id}"))
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_body(resp).await;
        assert_eq!(body["id"], person_id.to_string());
        assert_eq!(body["identity_count"], 1);
    }

    #[tokio::test]
    async fn people_get_not_found_returns_404() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_avatar_column(&pool).await;
        let app = build_router(state);
        let org = Uuid::new_v4();
        let resp = app
            .oneshot(
                Request::get(format!("/team/people/{}", Uuid::new_v4()))
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn people_update_happy_path() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_avatar_column(&pool).await;
        let org = Uuid::new_v4();
        let person_id = insert_person(&pool, org).await;

        let app = build_router(state);
        let body = serde_json::json!({
            "display_name": "Updated Name",
            "team": "Backend"
        });
        let resp = app
            .oneshot(
                Request::put(format!("/team/people/{person_id}"))
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let resp_body = read_body(resp).await;
        assert_eq!(resp_body["display_name"], "Updated Name");
        assert_eq!(resp_body["team"], "Backend");
    }

    #[tokio::test]
    async fn people_update_not_found_returns_404() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_avatar_column(&pool).await;
        let app = build_router(state);
        let org = Uuid::new_v4();
        let body = serde_json::json!({
            "display_name": "Ghost"
        });
        let resp = app
            .oneshot(
                Request::put(format!("/team/people/{}", Uuid::new_v4()))
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn people_delete_soft_deletes() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_avatar_column(&pool).await;
        let org = Uuid::new_v4();
        let person_id = insert_person(&pool, org).await;

        let app = build_router(state.clone());
        let resp = app
            .oneshot(
                Request::delete(format!("/team/people/{person_id}"))
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        // Verify still in DB but inactive
        let row = sqlx::query("select status from people where id = $1")
            .bind(person_id)
            .fetch_one(&pool)
            .await
            .expect("person should still exist");
        let status: String = row.get("status");
        assert_eq!(status, "inactive");
    }

    #[tokio::test]
    async fn people_delete_not_found_returns_404() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_avatar_column(&pool).await;
        let app = build_router(state);
        let org = Uuid::new_v4();
        let resp = app
            .oneshot(
                Request::delete(format!("/team/people/{}", Uuid::new_v4()))
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn people_list_filters_inactive_by_default() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_avatar_column(&pool).await;
        let org = Uuid::new_v4();
        let person_id = insert_person(&pool, org).await;

        // Soft delete the person
        sqlx::query("update people set status = 'inactive' where id = $1")
            .bind(person_id)
            .execute(&pool)
            .await
            .expect("soft delete");

        let app = build_router(state);
        let resp = app
            .oneshot(
                Request::get("/team/people")
                    .header("X-Org-Id", org.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = read_body(resp).await;
        assert_eq!(body["count"], 0);
    }

    #[tokio::test]
    async fn people_link_identity_happy_path() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_avatar_column(&pool).await;
        let org = Uuid::new_v4();
        let person_id = insert_person(&pool, org).await;
        let identity_id = insert_identity(&pool, org).await;

        let app = build_router(state);
        let body = serde_json::json!({
            "identity_id": identity_id
        });
        let resp = app
            .oneshot(
                Request::post(format!("/team/people/{person_id}/link"))
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let resp_body = read_body(resp).await;
        assert_eq!(resp_body["person_id"], person_id.to_string());
        assert_eq!(resp_body["identity_id"], identity_id.to_string());
        assert_eq!(resp_body["status"], "verified");
    }

    #[tokio::test]
    async fn people_link_identity_person_not_found_returns_404() {
        let (state, pool) = match test_state().await {
            Some(s) => s,
            None => return,
        };
        ensure_avatar_column(&pool).await;
        let org = Uuid::new_v4();
        let identity_id = insert_identity(&pool, org).await;

        let app = build_router(state);
        let body = serde_json::json!({
            "identity_id": identity_id
        });
        let resp = app
            .oneshot(
                Request::post(format!("/team/people/{}/link", Uuid::new_v4()))
                    .header("X-Org-Id", org.to_string())
                    .header("Content-Type", "application/json")
                    .body(Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
