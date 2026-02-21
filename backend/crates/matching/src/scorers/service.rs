use ovia_db::identity::models::{Identity, Person};

use crate::trace::ScorerResult;

use super::Scorer;

pub struct ServiceAccountScorer {
    pub weight: f64,
}

impl Scorer for ServiceAccountScorer {
    fn name(&self) -> &'static str {
        "service_account_penalty"
    }

    fn score(&self, _person: &Person, identity: &Identity) -> ScorerResult {
        let score = if identity.is_service_account {
            0.0
        } else {
            1.0
        };

        ScorerResult {
            rule: self.name().to_string(),
            score,
            weight: self.weight,
            weighted_score: score * self.weight,
            detail: format!("is_service_account={}", identity.is_service_account),
        }
    }
}
