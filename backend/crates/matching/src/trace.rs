use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorerResult {
    pub rule: String,
    pub score: f64,
    pub weight: f64,
    pub weighted_score: f64,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleTrace {
    pub scorers: Vec<ScorerResult>,
    pub raw_total: f64,
    pub weight_sum: f64,
    pub confidence: f64,
    pub classification: String,
}
