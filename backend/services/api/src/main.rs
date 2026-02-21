use axum::{routing::get, Json, Router};
use ovia_common::types::ServiceInfo;
use ovia_config::{init_tracing, AppConfig};
use std::net::SocketAddr;

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn info() -> Json<ServiceInfo> {
    Json(ServiceInfo::new("ovia-api"))
}

fn build_router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/info", get(info))
}

#[tokio::main]
async fn main() {
    init_tracing("info");

    let config = AppConfig::from_env().expect("failed to load config");
    tracing::info!(service = "ovia-api", "starting");

    let app = build_router();
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
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_returns_ok() {
        let app = build_router();
        let resp = app
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn info_returns_service_name() {
        let app = build_router();
        let resp = app
            .oneshot(Request::get("/info").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
