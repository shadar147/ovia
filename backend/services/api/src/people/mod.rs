pub mod handlers;
pub mod requests;
pub mod responses;

use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/team/people", get(handlers::list_people))
        .route("/team/people", post(handlers::create_person))
        .route("/team/people/{id}", get(handlers::get_person))
        .route("/team/people/{id}", put(handlers::update_person))
        .route("/team/people/{id}", delete(handlers::delete_person))
        .route("/team/people/{id}/link", post(handlers::link_identity))
}
