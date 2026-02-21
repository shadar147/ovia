use ovia_db::identity::models::{Identity, Person};

use crate::trace::ScorerResult;

use super::Scorer;

pub struct EmailExactScorer {
    pub weight: f64,
}

impl Scorer for EmailExactScorer {
    fn name(&self) -> &'static str {
        "email_exact"
    }

    fn score(&self, person: &Person, identity: &Identity) -> ScorerResult {
        let score = match (&person.primary_email, &identity.email) {
            (Some(pe), Some(ie)) => {
                let pe = pe.trim().to_lowercase();
                let ie = ie.trim().to_lowercase();
                if pe == ie {
                    1.0
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };

        ScorerResult {
            rule: self.name().to_string(),
            score,
            weight: self.weight,
            weighted_score: score * self.weight,
            detail: format!(
                "person_email={:?} identity_email={:?}",
                person.primary_email, identity.email
            ),
        }
    }
}
