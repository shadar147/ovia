pub mod handlers;
pub mod requests;
pub mod responses;

use axum::routing::{get, post};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/ask", post(handlers::post_ask))
        .route("/ask/history", get(handlers::list_ask_history))
        .route("/ask/{id}", get(handlers::get_ask_session))
}
