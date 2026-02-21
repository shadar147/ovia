use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorerWeights {
    pub email_exact: f64,
    pub username_similarity: f64,
    pub display_name_similarity: f64,
    pub team_co_occurrence: f64,
    pub service_account_penalty: f64,
}

impl Default for ScorerWeights {
    fn default() -> Self {
        Self {
            email_exact: 0.40,
            username_similarity: 0.20,
            display_name_similarity: 0.20,
            team_co_occurrence: 0.10,
            service_account_penalty: 0.10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    pub auto_accept: f64,
    pub conflict_min: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            auto_accept: 0.85,
            conflict_min: 0.50,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MatchingConfig {
    pub weights: ScorerWeights,
    pub thresholds: Thresholds,
}
