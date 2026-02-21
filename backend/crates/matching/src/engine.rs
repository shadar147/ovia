use ovia_db::identity::models::{Identity, LinkStatus, Person};

use crate::config::MatchingConfig;
use crate::scorers::display_name::DisplayNameSimilarityScorer;
use crate::scorers::email::EmailExactScorer;
use crate::scorers::service::ServiceAccountScorer;
use crate::scorers::team::TeamCoOccurrenceScorer;
use crate::scorers::username::UsernameSimilarityScorer;
use crate::scorers::Scorer;
use crate::trace::{RuleTrace, ScorerResult};

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub confidence: f64,
    pub status: LinkStatus,
    pub rule_trace: RuleTrace,
}

fn classify(confidence: f64, config: &MatchingConfig) -> LinkStatus {
    if confidence >= config.thresholds.auto_accept {
        LinkStatus::Auto
    } else if confidence >= config.thresholds.conflict_min {
        LinkStatus::Conflict
    } else {
        LinkStatus::Rejected
    }
}

pub fn evaluate(config: &MatchingConfig, person: &Person, identity: &Identity) -> MatchResult {
    let scorers: Vec<Box<dyn Scorer>> = vec![
        Box::new(EmailExactScorer {
            weight: config.weights.email_exact,
        }),
        Box::new(UsernameSimilarityScorer {
            weight: config.weights.username_similarity,
        }),
        Box::new(DisplayNameSimilarityScorer {
            weight: config.weights.display_name_similarity,
        }),
        Box::new(TeamCoOccurrenceScorer {
            weight: config.weights.team_co_occurrence,
        }),
        Box::new(ServiceAccountScorer {
            weight: config.weights.service_account_penalty,
        }),
    ];

    let results: Vec<ScorerResult> = scorers.iter().map(|s| s.score(person, identity)).collect();

    let raw_total: f64 = results.iter().map(|r| r.weighted_score).sum();
    let weight_sum: f64 = results.iter().map(|r| r.weight).sum();

    let confidence = if weight_sum > 0.0 {
        (raw_total / weight_sum).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let status = classify(confidence, config);

    let rule_trace = RuleTrace {
        scorers: results,
        raw_total,
        weight_sum,
        confidence,
        classification: status.as_str().to_string(),
    };

    MatchResult {
        confidence,
        status,
        rule_trace,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ScorerWeights, Thresholds};
    use chrono::Utc;
    use uuid::Uuid;

    fn make_person(display_name: &str, email: Option<&str>, team: Option<&str>) -> Person {
        Person {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            display_name: display_name.to_string(),
            primary_email: email.map(|s| s.to_string()),
            team: team.map(|s| s.to_string()),
            role: None,
            status: "active".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_identity(
        username: Option<&str>,
        email: Option<&str>,
        display_name: Option<&str>,
        is_service_account: bool,
    ) -> Identity {
        Identity {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            source: "gitlab".to_string(),
            external_id: None,
            username: username.map(|s| s.to_string()),
            email: email.map(|s| s.to_string()),
            display_name: display_name.map(|s| s.to_string()),
            is_service_account,
            first_seen_at: None,
            last_seen_at: None,
        }
    }

    #[test]
    fn t01_perfect_match_all_fields() {
        let cfg = MatchingConfig::default();
        let person = make_person("John Smith", Some("john.smith@corp.com"), None);
        let identity = make_identity(
            Some("john.smith"),
            Some("john.smith@corp.com"),
            Some("John Smith"),
            false,
        );
        let result = evaluate(&cfg, &person, &identity);
        // email=1.0, username=1.0, display=1.0, team=0.0 (no team), service=1.0
        // weighted = 0.40 + 0.20 + 0.20 + 0 + 0.10 = 0.90 → Auto (>= 0.85)
        assert!(
            result.confidence >= cfg.thresholds.auto_accept,
            "confidence={}",
            result.confidence
        );
        assert_eq!(result.status, LinkStatus::Auto);
    }

    #[test]
    fn t02_exact_email_only() {
        let cfg = MatchingConfig::default();
        let person = make_person("John Smith", Some("john@co.com"), None);
        let identity = make_identity(None, Some("john@co.com"), None, false);
        let result = evaluate(&cfg, &person, &identity);
        // email_exact=0.40*1.0, service=0.10*1.0, rest=0 → (0.50/1.0)=0.50
        // But username scorer gets email local "john" vs None → 0, display_name 0, team 0
        // So weighted = 0.40 + 0.10 = 0.50, sum=1.0 → confidence=0.50
        // Actually with default weights that's exactly 0.50 which equals conflict_min
        // Let's just check it's >= conflict_min
        assert!(
            result.confidence >= cfg.thresholds.conflict_min,
            "confidence={}",
            result.confidence
        );
    }

    #[test]
    fn t03_no_email_username_match() {
        let cfg = MatchingConfig::default();
        let person = make_person("Ivan M", Some("ivan.m@corp.com"), None);
        let identity = make_identity(Some("ivan.m"), None, None, false);
        let result = evaluate(&cfg, &person, &identity);
        // username should score high (jaro_winkler "ivan.m" vs "ivan.m" = 1.0)
        // email_exact=0 (identity has no email), service=1.0
        // weighted = 0.20*1.0 + 0.10*1.0 = 0.30, sum=1.0 → 0.30 → Rejected
        // Actually conflict range starts at 0.50, so this is Rejected
        assert!(result.confidence > 0.0);
        assert!(result.confidence < cfg.thresholds.auto_accept);
    }

    #[test]
    fn t04_display_name_close() {
        let cfg = MatchingConfig::default();
        let person = make_person("John Smith", Some("john@co.com"), None);
        let identity = make_identity(
            Some("jsmith"),
            Some("john@co.com"),
            Some("John Smyth"),
            false,
        );
        let result = evaluate(&cfg, &person, &identity);
        // email exact match, display name close (JW ~ 0.93), username partial, service ok
        assert!(
            result.confidence >= cfg.thresholds.conflict_min,
            "confidence={}",
            result.confidence
        );
    }

    #[test]
    fn t05_service_account_rejection() {
        let cfg = MatchingConfig::default();
        let person = make_person("CI Bot", Some("ci@corp.com"), None);
        let identity = make_identity(Some("ci-bot"), Some("ci@corp.com"), Some("CI Bot"), true);
        let result = evaluate(&cfg, &person, &identity);
        // service_account_penalty score=0.0, drags confidence down
        // email=1.0*0.40, username ~high*0.20, display=1.0*0.20, team=0*0.10, service=0.0*0.10
        // Should be below perfect but may still be Auto since service is only 0.10 weight
        // Actually: 0.40 + ~0.14 + 0.20 + 0 + 0 = ~0.74 → Conflict (below 0.85)
        assert!(
            result.confidence < 1.0,
            "service penalty should reduce confidence"
        );
        // The key assertion: a service account with matching fields should NOT be Auto
        // with default weights the penalty is small (0.10), but it should drag below Auto
        assert!(
            result.confidence < cfg.thresholds.auto_accept,
            "confidence={} should be < auto_accept={}",
            result.confidence,
            cfg.thresholds.auto_accept
        );
    }

    #[test]
    fn t06_completely_different() {
        let cfg = MatchingConfig::default();
        let person = make_person("Alice Johnson", Some("alice@example.com"), Some("backend"));
        let identity = make_identity(
            Some("bob-xyz"),
            Some("bob@other.org"),
            Some("Robert Chen"),
            false,
        );
        let result = evaluate(&cfg, &person, &identity);
        assert!(result.confidence < 0.5, "confidence={}", result.confidence);
        assert_eq!(result.status, LinkStatus::Rejected);
    }

    #[test]
    fn t07_case_insensitive_email() {
        let cfg = MatchingConfig::default();
        let person = make_person("John", Some("John@Co.Com"), None);
        let identity = make_identity(Some("john"), Some("john@co.com"), Some("John"), false);
        let result = evaluate(&cfg, &person, &identity);
        // email match (case insensitive), display match, username match, service ok
        assert!(
            result.confidence >= cfg.thresholds.auto_accept,
            "confidence={}",
            result.confidence
        );
        assert_eq!(result.status, LinkStatus::Auto);
    }

    #[test]
    fn t08_all_fields_missing() {
        let cfg = MatchingConfig::default();
        let person = make_person("", None, None);
        let identity = make_identity(None, None, None, false);
        let result = evaluate(&cfg, &person, &identity);
        // Only service_account_penalty scores 1.0 (not a service account)
        // Everything else is 0 → confidence = 0.10/1.0 = 0.10
        assert!(result.confidence < cfg.thresholds.conflict_min);
        assert_eq!(result.status, LinkStatus::Rejected);
    }

    #[test]
    fn t09_username_from_email_match() {
        let cfg = MatchingConfig::default();
        let person = make_person("Ivan", Some("ivan.m@corp.com"), None);
        let identity = make_identity(Some("ivan.m"), None, None, false);
        let result = evaluate(&cfg, &person, &identity);
        // username scorer: jaro_winkler("ivan.m", "ivan.m") = 1.0
        let username_scorer = result
            .rule_trace
            .scorers
            .iter()
            .find(|s| s.rule == "username_similarity")
            .unwrap();
        assert!(
            (username_scorer.score - 1.0).abs() < f64::EPSILON,
            "score={}",
            username_scorer.score
        );
    }

    #[test]
    fn t10_partial_username() {
        let cfg = MatchingConfig::default();
        let person = make_person("Ivan Malinov", Some("ivan.malinov@corp.com"), None);
        let identity = make_identity(Some("imalinov"), None, None, false);
        let result = evaluate(&cfg, &person, &identity);
        let username_scorer = result
            .rule_trace
            .scorers
            .iter()
            .find(|s| s.rule == "username_similarity")
            .unwrap();
        // "ivan.malinov" vs "imalinov" → partial JW match, mid-range
        assert!(
            username_scorer.score > 0.3 && username_scorer.score < 0.95,
            "score={}",
            username_scorer.score
        );
    }

    #[test]
    fn t11_team_in_username() {
        let cfg = MatchingConfig::default();
        let person = make_person("Bot", None, Some("platform"));
        let identity = make_identity(Some("platform-bot"), None, None, false);
        let result = evaluate(&cfg, &person, &identity);
        let team_scorer = result
            .rule_trace
            .scorers
            .iter()
            .find(|s| s.rule == "team_co_occurrence")
            .unwrap();
        assert!(
            (team_scorer.score - 0.5).abs() < f64::EPSILON,
            "score={}",
            team_scorer.score
        );
    }

    #[test]
    fn t12_team_no_overlap() {
        let cfg = MatchingConfig::default();
        let person = make_person("Dev", None, Some("backend"));
        let identity = make_identity(Some("frontend-dev"), None, None, false);
        let result = evaluate(&cfg, &person, &identity);
        let team_scorer = result
            .rule_trace
            .scorers
            .iter()
            .find(|s| s.rule == "team_co_occurrence")
            .unwrap();
        assert!(
            team_scorer.score.abs() < f64::EPSILON,
            "score={}",
            team_scorer.score
        );
    }

    #[test]
    fn t13_custom_thresholds_lower() {
        let cfg = MatchingConfig {
            weights: ScorerWeights::default(),
            thresholds: Thresholds {
                auto_accept: 0.70,
                conflict_min: 0.30,
            },
        };
        let person = make_person("John", Some("john@co.com"), None);
        let identity = make_identity(Some("john"), Some("john@co.com"), None, false);
        let result = evaluate(&cfg, &person, &identity);
        // email match (0.40) + username high (0.20*~1.0) + service (0.10) = ~0.70
        assert_eq!(
            result.status,
            LinkStatus::Auto,
            "confidence={} should be Auto with lowered threshold",
            result.confidence
        );
    }

    #[test]
    fn t14_custom_weights_email_dominant() {
        let cfg = MatchingConfig {
            weights: ScorerWeights {
                email_exact: 0.90,
                username_similarity: 0.025,
                display_name_similarity: 0.025,
                team_co_occurrence: 0.025,
                service_account_penalty: 0.025,
            },
            thresholds: Thresholds::default(),
        };
        let person = make_person("X", Some("x@y.com"), None);
        let identity = make_identity(None, Some("x@y.com"), None, false);
        let result = evaluate(&cfg, &person, &identity);
        // email=0.90*1.0, service=0.025*1.0, rest=0 → (0.925/1.0) = 0.925
        assert!(result.confidence > 0.90, "confidence={}", result.confidence);
        assert_eq!(result.status, LinkStatus::Auto);
    }

    #[test]
    fn t15_whitespace_trimmed_email() {
        let cfg = MatchingConfig::default();
        let person = make_person("John", Some("  john@co.com  "), None);
        let identity = make_identity(None, Some("john@co.com"), None, false);
        let result = evaluate(&cfg, &person, &identity);
        let email_scorer = result
            .rule_trace
            .scorers
            .iter()
            .find(|s| s.rule == "email_exact")
            .unwrap();
        assert!(
            (email_scorer.score - 1.0).abs() < f64::EPSILON,
            "trimmed emails should match exactly, score={}",
            email_scorer.score
        );
    }

    #[test]
    fn t16_default_config_valid() {
        let cfg = MatchingConfig::default();
        let weight_sum = cfg.weights.email_exact
            + cfg.weights.username_similarity
            + cfg.weights.display_name_similarity
            + cfg.weights.team_co_occurrence
            + cfg.weights.service_account_penalty;
        assert!(
            (weight_sum - 1.0).abs() < 0.01,
            "weights should sum to ~1.0, got {}",
            weight_sum
        );
        assert!(cfg.thresholds.auto_accept > cfg.thresholds.conflict_min);
        assert!(cfg.thresholds.auto_accept <= 1.0);
        assert!(cfg.thresholds.conflict_min >= 0.0);
    }

    #[test]
    fn t17_rule_trace_has_all_scorers() {
        let cfg = MatchingConfig::default();
        let person = make_person("Test", Some("t@t.com"), None);
        let identity = make_identity(Some("t"), Some("t@t.com"), Some("Test"), false);
        let result = evaluate(&cfg, &person, &identity);
        assert_eq!(
            result.rule_trace.scorers.len(),
            5,
            "should have exactly 5 scorer results"
        );
        let names: Vec<&str> = result
            .rule_trace
            .scorers
            .iter()
            .map(|s| s.rule.as_str())
            .collect();
        assert!(names.contains(&"email_exact"));
        assert!(names.contains(&"username_similarity"));
        assert!(names.contains(&"display_name_similarity"));
        assert!(names.contains(&"team_co_occurrence"));
        assert!(names.contains(&"service_account_penalty"));
    }
}
