use ovia_db::identity::models::{Identity, Person};

use crate::trace::ScorerResult;

use super::Scorer;

pub struct UsernameSimilarityScorer {
    pub weight: f64,
}

/// Extract the local part of an email address (before the @).
fn username_from_email(email: &str) -> &str {
    email.split('@').next().unwrap_or(email)
}

impl Scorer for UsernameSimilarityScorer {
    fn name(&self) -> &'static str {
        "username_similarity"
    }

    fn score(&self, person: &Person, identity: &Identity) -> ScorerResult {
        let score = match (&person.primary_email, &identity.username) {
            (Some(email), Some(uname)) => {
                let local = username_from_email(email.trim());
                let uname = uname.trim();
                if local.is_empty() || uname.is_empty() {
                    0.0
                } else {
                    strsim::jaro_winkler(&local.to_lowercase(), &uname.to_lowercase())
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
                "email_local={:?} identity_username={:?}",
                person.primary_email.as_deref().map(username_from_email),
                identity.username
            ),
        }
    }
}
