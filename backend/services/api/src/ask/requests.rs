use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskFilters {
    pub team: Option<String>,
    pub product: Option<String>,
    pub date_range: Option<String>,
    pub sources: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct AskRequest {
    pub query: String,
    pub filters: Option<AskFilters>,
}
