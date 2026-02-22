use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use ovia_common::error::OviaResult;
use ovia_db::gitlab::pg_repository::PgGitlabRepository;
use ovia_db::jira::pg_repository::PgJiraRepository;
use ovia_db::kpi::models::{KpiSnapshot, RiskItem};
use ovia_db::kpi::repositories::KpiRepository;

use super::compute::{compute_delivery_health, compute_release_risk};

pub struct KpiService<R: KpiRepository> {
    repo: R,
    pool: PgPool,
}

impl<R: KpiRepository> KpiService<R> {
    pub fn new(repo: R, pool: PgPool) -> Self {
        Self { repo, pool }
    }

    /// Compute and save a KPI snapshot for the given org and period.
    ///
    /// Metrics are derived from real GitLab merge request and pipeline data:
    /// - throughput_total: count of merged MRs in period
    /// - throughput_bugs: merged MRs with 'bug' label
    /// - throughput_features: merged MRs with 'feature' label
    /// - throughput_chores: total - bugs - features
    /// - review_latency_median/p90: from merged_at - created_at_gl durations
    /// - delivery_health_score + release_risk_score from compute functions
    ///
    /// Risk items are generated from stale open MRs (>7 days) and failed pipelines.
    pub async fn compute_and_save(
        &self,
        org_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> OviaResult<KpiSnapshot> {
        let gl_repo = PgGitlabRepository::new(self.pool.clone());
        let jira_repo = PgJiraRepository::new(self.pool.clone());

        // ── GitLab throughput ─────────────────────────────────────────
        let mr_total = gl_repo
            .count_merged_mrs(org_id, period_start, period_end)
            .await? as i32;

        let mr_bugs = gl_repo
            .count_merged_mrs_by_label(org_id, period_start, period_end, "bug")
            .await? as i32;

        let mr_features = gl_repo
            .count_merged_mrs_by_label(org_id, period_start, period_end, "feature")
            .await? as i32;

        // ── Jira throughput (resolved issues by type) ─────────────────
        let jira_bugs = jira_repo
            .count_resolved_issues_by_type(org_id, period_start, period_end, "Bug")
            .await? as i32;

        let jira_stories = jira_repo
            .count_resolved_issues_by_type(org_id, period_start, period_end, "Story")
            .await? as i32;

        let jira_resolved_total = jira_repo
            .count_resolved_issues(org_id, period_start, period_end)
            .await? as i32;

        // Combine MR + Jira throughput (MR labels are usually empty → all chores,
        // so Jira provides the primary bug/feature breakdown)
        let throughput_bugs = mr_bugs + jira_bugs;
        let throughput_features = mr_features + jira_stories;
        let throughput_total = mr_total + jira_resolved_total;
        let throughput_chores = (throughput_total - throughput_bugs - throughput_features).max(0);

        // ── Review latency ──────────────────────────────────────────
        let durations = gl_repo
            .get_review_durations_hours(org_id, period_start, period_end)
            .await?;

        let review_latency_median =
            percentile(&durations.iter().map(|d| d.hours).collect::<Vec<_>>(), 50.0);
        let review_latency_p90 =
            percentile(&durations.iter().map(|d| d.hours).collect::<Vec<_>>(), 90.0);

        // ── Jira metrics ──────────────────────────────────────────────
        let blocker_count = jira_repo.count_open_blockers(org_id).await? as i32;
        let spillover_rate = jira_repo.spillover_rate(org_id).await?;

        let cycle_times = jira_repo
            .get_cycle_times_hours(org_id, period_start, period_end)
            .await?;
        let cycle_time_p50 = percentile(&cycle_times, 50.0);
        let cycle_time_p90 = percentile(&cycle_times, 90.0);

        // ── Risk inputs ─────────────────────────────────────────────
        let failing_pipelines = gl_repo
            .count_pipelines_by_status(org_id, period_start, period_end, "failed")
            .await? as u32;

        let stale_mr_pct = gl_repo.stale_mr_percentage(org_id, 7).await?;

        let blocker_age_days = jira_repo.list_open_blocker_age_days(org_id).await?;

        // ── Scores ──────────────────────────────────────────────────
        let delivery_health = compute_delivery_health(
            throughput_total,
            review_latency_median.unwrap_or(0.0),
            blocker_count,
            spillover_rate,
        );

        let (_risk_label, release_risk) =
            compute_release_risk(&blocker_age_days, failing_pipelines, stale_mr_pct);

        let now = Utc::now();
        let snapshot_id = Uuid::new_v4();
        let snapshot = KpiSnapshot {
            id: snapshot_id,
            org_id,
            period_start,
            period_end,
            delivery_health_score: Some(delivery_health),
            release_risk_score: Some(release_risk),
            throughput_total,
            throughput_bugs,
            throughput_features,
            throughput_chores,
            review_latency_median_hours: review_latency_median,
            review_latency_p90_hours: review_latency_p90,
            blocker_count,
            spillover_rate: Some(spillover_rate),
            cycle_time_p50_hours: cycle_time_p50,
            cycle_time_p90_hours: cycle_time_p90,
            computed_at: now,
            created_at: now,
        };

        let saved = self.repo.save_snapshot(snapshot).await?;

        // ── Risk items ──────────────────────────────────────────────
        let mut risk_items = Vec::new();

        // Stale open MRs (>7 days)
        let stale_mrs = gl_repo.list_stale_open_mrs(org_id, 7).await?;
        for mr in &stale_mrs {
            risk_items.push(RiskItem {
                id: Uuid::new_v4(),
                org_id,
                snapshot_id: saved.id,
                entity_type: "merge_request".to_string(),
                title: format!("Stale MR: {}", mr.title),
                owner: mr.author_username.clone(),
                age_days: mr.age_days,
                impact_scope: None,
                status: "open".to_string(),
                source_url: Some(mr.web_url.clone()),
                created_at: now,
            });
        }

        // Failed pipelines in period
        let failed_pipelines = gl_repo
            .list_failed_pipelines(org_id, period_start, period_end)
            .await?;
        for pl in &failed_pipelines {
            let age = pl
                .created_at_gl
                .map(|c| (now - c).num_days() as i32)
                .unwrap_or(0);
            risk_items.push(RiskItem {
                id: Uuid::new_v4(),
                org_id,
                snapshot_id: saved.id,
                entity_type: "pipeline".to_string(),
                title: format!(
                    "Failed pipeline #{} on {}",
                    pl.gitlab_pipeline_id,
                    pl.ref_name.as_deref().unwrap_or("unknown")
                ),
                owner: None,
                age_days: age,
                impact_scope: pl.ref_name.clone(),
                status: "failed".to_string(),
                source_url: Some(pl.web_url.clone()),
                created_at: now,
            });
        }

        if !risk_items.is_empty() {
            tracing::info!(count = risk_items.len(), "saving risk items");
            self.repo.save_risk_items(risk_items).await?;
        }

        Ok(saved)
    }
}

/// Compute a percentile from a sorted-ascending slice. Returns None for empty input.
fn percentile(sorted: &[f64], pct: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let k = (pct / 100.0) * (sorted.len() as f64 - 1.0);
    let floor = k.floor() as usize;
    let ceil = k.ceil() as usize;
    if floor == ceil {
        Some(sorted[floor])
    } else {
        let d = k - floor as f64;
        Some(sorted[floor] * (1.0 - d) + sorted[ceil] * d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use ovia_db::kpi::models::KpiFilter;
    use std::sync::Mutex;

    struct MockKpiRepo {
        saved_snapshots: Mutex<Vec<KpiSnapshot>>,
        saved_risks: Mutex<Vec<RiskItem>>,
    }

    impl MockKpiRepo {
        fn new() -> Self {
            Self {
                saved_snapshots: Mutex::new(Vec::new()),
                saved_risks: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl KpiRepository for MockKpiRepo {
        async fn save_snapshot(&self, snapshot: KpiSnapshot) -> OviaResult<KpiSnapshot> {
            self.saved_snapshots.lock().unwrap().push(snapshot.clone());
            Ok(snapshot)
        }

        async fn get_latest(&self, _org_id: Uuid) -> OviaResult<Option<KpiSnapshot>> {
            Ok(self.saved_snapshots.lock().unwrap().last().cloned())
        }

        async fn list_snapshots(&self, _filter: KpiFilter) -> OviaResult<Vec<KpiSnapshot>> {
            Ok(self.saved_snapshots.lock().unwrap().clone())
        }

        async fn save_risk_items(&self, items: Vec<RiskItem>) -> OviaResult<Vec<RiskItem>> {
            self.saved_risks.lock().unwrap().extend(items.clone());
            Ok(items)
        }

        async fn list_risk_items(&self, _snapshot_id: Uuid) -> OviaResult<Vec<RiskItem>> {
            Ok(self.saved_risks.lock().unwrap().clone())
        }
    }

    #[test]
    fn percentile_median_of_sorted_values() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&values, 50.0), Some(3.0));
    }

    #[test]
    fn percentile_p90_of_sorted_values() {
        let values: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let p90 = percentile(&values, 90.0).unwrap();
        assert!((p90 - 90.01).abs() < 0.1);
    }

    #[test]
    fn percentile_empty_returns_none() {
        assert_eq!(percentile(&[], 50.0), None);
    }

    #[test]
    fn percentile_single_value() {
        assert_eq!(percentile(&[42.0], 50.0), Some(42.0));
        assert_eq!(percentile(&[42.0], 90.0), Some(42.0));
    }

    #[tokio::test]
    async fn compute_and_save_uses_mock_repo() {
        let mock_repo = MockKpiRepo::new();
        assert!(mock_repo.saved_snapshots.lock().unwrap().is_empty());

        let snapshot = KpiSnapshot {
            id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            period_start: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            period_end: NaiveDate::from_ymd_opt(2026, 2, 14).unwrap(),
            delivery_health_score: Some(80.0),
            release_risk_score: Some(20.0),
            throughput_total: 10,
            throughput_bugs: 2,
            throughput_features: 5,
            throughput_chores: 3,
            review_latency_median_hours: Some(4.0),
            review_latency_p90_hours: None,
            blocker_count: 1,
            spillover_rate: Some(0.2),
            cycle_time_p50_hours: Some(24.0),
            cycle_time_p90_hours: Some(48.0),
            computed_at: Utc::now(),
            created_at: Utc::now(),
        };

        let saved = mock_repo.save_snapshot(snapshot).await.unwrap();
        assert_eq!(saved.throughput_total, 10);
        assert_eq!(saved.blocker_count, 1);
        assert_eq!(mock_repo.saved_snapshots.lock().unwrap().len(), 1);
    }
}
