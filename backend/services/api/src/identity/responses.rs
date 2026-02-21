use ovia_db::identity::models::PersonIdentityLink;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ListMappingsResponse {
    pub data: Vec<PersonIdentityLink>,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct MutationResponse {
    pub ok: bool,
}
