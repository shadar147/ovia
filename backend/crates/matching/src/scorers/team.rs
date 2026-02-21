use ovia_db::identity::models::{Identity, Person};

use crate::trace::ScorerResult;

use super::Scorer;

pub struct TeamCoOccurrenceScorer {
    pub weight: f64,
}

impl Scorer for TeamCoOccurrenceScorer {
    fn name(&self) -> &'static str {
        "team_co_occurrence"
    }

    fn score(&self, person: &Person, identity: &Identity) -> ScorerResult {
        let score = match &person.team {
            Some(team) => {
                let team_lower = team.trim().to_lowercase();
                if team_lower.is_empty() {
                    0.0
                } else {
                    let in_username = identity
                        .username
                        .as_deref()
                        .map(|u| u.to_lowercase().contains(&team_lower))
                        .unwrap_or(false);
                    let in_display = identity
                        .display_name
                        .as_deref()
                        .map(|d| d.to_lowercase().contains(&team_lower))
                        .unwrap_or(false);
                    if in_username || in_display {
                        0.5
                    } else {
                        0.0
                    }
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
                "person_team={:?} identity_username={:?} identity_display={:?}",
                person.team, identity.username, identity.display_name
            ),
        }
    }
}
