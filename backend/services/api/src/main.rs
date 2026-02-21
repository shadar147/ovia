mod error;
mod extractors;
mod identity;

use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use ovia_common::types::ServiceInfo;
use ovia_config::{init_tracing, AppConfig};
use ovia_db::identity::pg_repository::PgIdentityRepository;
use std::net::SocketAddr;

#[derive(Clone)]
pub struct AppState {
    pub identity_repo: PgIdentityRepository,
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
    Router::new()
        .route("/health", get(health))
        .route("/info", get(info))
        .route("/metrics", get(metrics))
        .merge(identity::router())
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
        identity_repo: PgIdentityRepository::new(pool),
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
    use sqlx::PgPool;
    use tower::ServiceExt;
    use uuid::Uuid;

    async fn test_state() -> Option<(AppState, PgPool)> {
        let url = std::env::var("TEST_DATABASE_URL").ok()?;
        let pool = ovia_db::create_pool(&url).await.expect("db should connect");
        let state = AppState {
            identity_repo: PgIdentityRepository::new(pool.clone()),
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
}
