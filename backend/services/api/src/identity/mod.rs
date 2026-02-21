pub mod handlers;
pub mod requests;
pub mod responses;

use axum::routing::{get, post};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/team/identity-mappings", get(handlers::list_mappings))
        .route(
            "/team/identity-mappings/confirm",
            post(handlers::confirm_mapping),
        )
        .route(
            "/team/identity-mappings/remap",
            post(handlers::remap_mapping),
        )
        .route(
            "/team/identity-mappings/split",
            post(handlers::split_mapping),
        )
}
