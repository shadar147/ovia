use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

pub struct OrgId(pub Uuid);

#[derive(Debug)]
pub struct OrgIdRejection(String);

impl IntoResponse for OrgIdRejection {
    fn into_response(self) -> Response {
        let body = serde_json::json!({ "error": self.0 });
        (StatusCode::BAD_REQUEST, axum::Json(body)).into_response()
    }
}

impl<S: Send + Sync> FromRequestParts<S> for OrgId {
    type Rejection = OrgIdRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("X-Org-Id")
            .ok_or_else(|| OrgIdRejection("missing X-Org-Id header".to_string()))?;

        let value = header
            .to_str()
            .map_err(|_| OrgIdRejection("invalid X-Org-Id header value".to_string()))?;

        let uuid = Uuid::parse_str(value)
            .map_err(|_| OrgIdRejection(format!("invalid UUID in X-Org-Id: {value}")))?;

        Ok(OrgId(uuid))
    }
}
