use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreatePersonRequest {
    pub display_name: String,
    pub primary_email: Option<String>,
    pub avatar_url: Option<String>,
    pub team: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePersonRequest {
    pub display_name: Option<String>,
    pub primary_email: Option<String>,
    pub avatar_url: Option<String>,
    pub team: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LinkIdentityRequest {
    pub identity_id: Uuid,
    pub status: Option<String>,
    pub confidence: Option<f64>,
}
