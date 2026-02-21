use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use ovia_common::error::OviaError;

pub struct ApiError(pub OviaError);

impl From<OviaError> for ApiError {
    fn from(err: OviaError) -> Self {
        Self(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self.0 {
            OviaError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            OviaError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            other => (StatusCode::INTERNAL_SERVER_ERROR, other.to_string()),
        };

        let body = serde_json::json!({ "error": message });
        (status, Json(body)).into_response()
    }
}
