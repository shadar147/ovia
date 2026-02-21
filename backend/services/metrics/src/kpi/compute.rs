/// Compute a delivery health score (0-100) from throughput, review latency, blockers, and spillover.
///
/// Weighted formula:
///   - throughput contributes 30% (normalized: capped at 100 items → 100 points)
///   - review_latency_median contributes 30% (inverse: 0h → 100, >=48h → 0)
///   - blocker_count contributes 20% (inverse: 0 → 100, >=10 → 0)
///   - spillover_rate contributes 20% (inverse: 0.0 → 100, >=1.0 → 0)
pub fn compute_delivery_health(
    throughput: i32,
    review_latency_median: f64,
    blocker_count: i32,
    spillover_rate: f64,
) -> f64 {
    let throughput_score = (throughput.clamp(0, 100) as f64 / 100.0) * 100.0;

    let latency_score = if review_latency_median <= 0.0 {
        100.0
    } else if review_latency_median >= 48.0 {
        0.0
    } else {
        (1.0 - review_latency_median / 48.0) * 100.0
    };

    let blocker_score = if blocker_count <= 0 {
        100.0
    } else if blocker_count >= 10 {
        0.0
    } else {
        (1.0 - blocker_count as f64 / 10.0) * 100.0
    };

    let spillover_score = if spillover_rate <= 0.0 {
        100.0
    } else if spillover_rate >= 1.0 {
        0.0
    } else {
        (1.0 - spillover_rate) * 100.0
    };

    let raw = throughput_score * 0.30
        + latency_score * 0.30
        + blocker_score * 0.20
        + spillover_score * 0.20;

    raw.clamp(0.0, 100.0)
}

/// Compute release risk as a label and score (0-100).
///
/// Factors:
///   - blocker_age_days: sum of days across all blockers
///   - failing_pipelines: count of failing CI pipelines
///   - stale_mr_pct: percentage of merge requests older than 7 days (0.0-1.0)
///
/// Returns ("low"|"medium"|"high", score 0-100).
pub fn compute_release_risk(
    blocker_age_days: &[i32],
    failing_pipelines: u32,
    stale_mr_pct: f64,
) -> (String, f64) {
    let total_blocker_age: f64 = blocker_age_days.iter().map(|d| *d as f64).sum();
    let blocker_count = blocker_age_days.len() as f64;

    // Blocker contribution: 40% weight
    // Each blocker adds risk, capped at a total of 100 from blockers alone
    let blocker_risk = ((blocker_count * 10.0) + (total_blocker_age * 0.5)).min(100.0);

    // Pipeline contribution: 30% weight
    // Each failing pipeline adds 20 points, capped at 100
    let pipeline_risk = (failing_pipelines as f64 * 20.0).min(100.0);

    // Stale MR contribution: 30% weight
    let stale_risk = (stale_mr_pct.clamp(0.0, 1.0)) * 100.0;

    let score = (blocker_risk * 0.40 + pipeline_risk * 0.30 + stale_risk * 0.30).clamp(0.0, 100.0);

    let label = if score >= 70.0 {
        "high"
    } else if score >= 35.0 {
        "medium"
    } else {
        "low"
    };

    (label.to_string(), score)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── compute_delivery_health tests ──────────────────────────────

    #[test]
    fn perfect_delivery_health() {
        let score = compute_delivery_health(100, 0.0, 0, 0.0);
        assert!((score - 100.0).abs() < 0.01);
    }

    #[test]
    fn zero_delivery_health() {
        let score = compute_delivery_health(0, 48.0, 10, 1.0);
        assert!((score - 0.0).abs() < 0.01);
    }

    #[test]
    fn mid_delivery_health() {
        let score = compute_delivery_health(50, 24.0, 5, 0.5);
        // throughput: 50/100 * 100 * 0.30 = 15.0
        // latency:   (1 - 24/48) * 100 * 0.30 = 15.0
        // blockers:  (1 - 5/10) * 100 * 0.20 = 10.0
        // spillover: (1 - 0.5) * 100 * 0.20 = 10.0
        // Total = 50.0
        assert!((score - 50.0).abs() < 0.01);
    }

    #[test]
    fn extreme_throughput_clamped() {
        let score = compute_delivery_health(500, 0.0, 0, 0.0);
        // throughput capped at 100
        assert!((score - 100.0).abs() < 0.01);
    }

    #[test]
    fn negative_throughput_clamped() {
        let score = compute_delivery_health(-10, 0.0, 0, 0.0);
        // throughput clamped to 0
        // 0 * 0.30 + 100 * 0.30 + 100 * 0.20 + 100 * 0.20 = 70.0
        assert!((score - 70.0).abs() < 0.01);
    }

    #[test]
    fn only_blockers_affect_health() {
        let score = compute_delivery_health(100, 0.0, 5, 0.0);
        // 100*0.30 + 100*0.30 + 50*0.20 + 100*0.20 = 30 + 30 + 10 + 20 = 90
        assert!((score - 90.0).abs() < 0.01);
    }

    // ── compute_release_risk tests ─────────────────────────────────

    #[test]
    fn no_risk_all_zeros() {
        let (label, score) = compute_release_risk(&[], 0, 0.0);
        assert_eq!(label, "low");
        assert!((score - 0.0).abs() < 0.01);
    }

    #[test]
    fn high_risk_many_blockers() {
        let (label, score) = compute_release_risk(&[30, 20, 10], 3, 0.8);
        // blocker_risk: min((3*10 + 60*0.5), 100) = min(60, 100) = 60
        // pipeline_risk: min(60, 100) = 60
        // stale_risk: 80
        // score = 60*0.4 + 60*0.3 + 80*0.3 = 24 + 18 + 24 = 66
        assert!(score > 35.0);
        assert!(label == "medium" || label == "high");
    }

    #[test]
    fn high_risk_extreme_values() {
        let (label, score) = compute_release_risk(&[100, 100, 100, 100, 100], 10, 1.0);
        assert_eq!(label, "high");
        assert!(score >= 70.0);
    }

    #[test]
    fn low_risk_minimal_issues() {
        let (label, score) = compute_release_risk(&[1], 0, 0.05);
        // blocker_risk: min(10 + 0.5, 100) = 10.5
        // pipeline_risk: 0
        // stale_risk: 5
        // score = 10.5*0.4 + 0 + 5*0.3 = 4.2 + 1.5 = 5.7
        assert_eq!(label, "low");
        assert!(score < 35.0);
    }

    #[test]
    fn medium_risk_moderate_pipelines() {
        let (label, score) = compute_release_risk(&[], 3, 0.3);
        // blocker_risk: 0
        // pipeline_risk: 60
        // stale_risk: 30
        // score = 0 + 60*0.3 + 30*0.3 = 18 + 9 = 27
        assert_eq!(label, "low");
        assert!(score < 35.0);
    }
}
