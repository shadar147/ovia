pub mod handlers;
pub mod responses;

use axum::routing::get;
use axum::Router;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/team/kpi", get(handlers::get_latest_kpi))
        .route("/team/kpi/history", get(handlers::list_kpi_history))
        .route("/team/kpi/risks", get(handlers::list_kpi_risks))
}
