use ovia_db::identity::models::{Identity, Person};

use crate::trace::ScorerResult;

use super::Scorer;

pub struct DisplayNameSimilarityScorer {
    pub weight: f64,
}

impl Scorer for DisplayNameSimilarityScorer {
    fn name(&self) -> &'static str {
        "display_name_similarity"
    }

    fn score(&self, person: &Person, identity: &Identity) -> ScorerResult {
        let score = match &identity.display_name {
            Some(id_name) => {
                let pn = person.display_name.trim().to_lowercase();
                let idn = id_name.trim().to_lowercase();
                if pn.is_empty() || idn.is_empty() {
                    0.0
                } else {
                    strsim::jaro_winkler(&pn, &idn)
                }
            }
            None => 0.0,
        };

        ScorerResult {
            rule: self.name().to_string(),
            score,
            weight: self.weight,
            weighted_score: score * self.weight,
            detail: format!(
                "person_name={:?} identity_name={:?}",
                person.display_name, identity.display_name
            ),
        }
    }
}
