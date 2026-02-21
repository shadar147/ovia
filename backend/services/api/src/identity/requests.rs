use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ConfirmRequest {
    pub link_id: Uuid,
    pub verified_by: String,
}

#[derive(Debug, Deserialize)]
pub struct RemapRequest {
    pub link_id: Uuid,
    pub new_person_id: Uuid,
    pub verified_by: String,
}

#[derive(Debug, Deserialize)]
pub struct SplitRequest {
    pub link_id: Uuid,
    pub verified_by: String,
}
