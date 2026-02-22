use chrono::NaiveDate;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::jira::models::{JiraIssue, JiraIssueTransition};
use ovia_common::error::{OviaError, OviaResult};

#[derive(Clone)]
pub struct PgJiraRepository {
    pool: PgPool,
}

impl PgJiraRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Upsert a Jira issue (idempotent on org_id + jira_key).
    pub async fn upsert_issue(&self, issue: &JiraIssue) -> OviaResult<()> {
        sqlx::query(
            "insert into jira_issues
             (id, org_id, jira_key, project_key, issue_type, summary, status,
              assignee_account_id, reporter_account_id, priority,
              story_points, sprint_name, sprint_id, team_name,
              labels, created_at_jira, updated_at_jira, resolved_at, raw_ref)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
             on conflict (org_id, jira_key) do update set
               issue_type = excluded.issue_type,
               summary = excluded.summary,
               status = excluded.status,
               assignee_account_id = excluded.assignee_account_id,
               reporter_account_id = excluded.reporter_account_id,
               priority = excluded.priority,
               story_points = excluded.story_points,
               sprint_name = excluded.sprint_name,
               sprint_id = excluded.sprint_id,
               team_name = excluded.team_name,
               labels = excluded.labels,
               created_at_jira = excluded.created_at_jira,
               updated_at_jira = excluded.updated_at_jira,
               resolved_at = excluded.resolved_at,
               raw_ref = excluded.raw_ref,
               updated_at = now()",
        )
        .bind(issue.id)
        .bind(issue.org_id)
        .bind(&issue.jira_key)
        .bind(&issue.project_key)
        .bind(&issue.issue_type)
        .bind(&issue.summary)
        .bind(&issue.status)
        .bind(&issue.assignee_account_id)
        .bind(&issue.reporter_account_id)
        .bind(&issue.priority)
        .bind(issue.story_points)
        .bind(&issue.sprint_name)
        .bind(issue.sprint_id)
        .bind(&issue.team_name)
        .bind(&issue.labels)
        .bind(issue.created_at_jira)
        .bind(issue.updated_at_jira)
        .bind(issue.resolved_at)
        .bind(&issue.raw_ref)
        .execute(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;
        Ok(())
    }

    /// Insert a transition row. Not idempotent — caller should avoid duplicates.
    pub async fn insert_transition(&self, t: &JiraIssueTransition) -> OviaResult<()> {
        sqlx::query(
            "insert into jira_issue_transitions
             (id, org_id, jira_key, field, from_value, to_value, author_account_id, transitioned_at)
             values ($1, $2, $3, $4, $5, $6, $7, $8)
             on conflict do nothing",
        )
        .bind(t.id)
        .bind(t.org_id)
        .bind(&t.jira_key)
        .bind(&t.field)
        .bind(&t.from_value)
        .bind(&t.to_value)
        .bind(&t.author_account_id)
        .bind(t.transitioned_at)
        .execute(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;
        Ok(())
    }

    // ── Jira KPI metrics queries ─────────────────────────────────

    /// Count open blocker issues (priority = 'Blocker' or 'Highest', not resolved).
    pub async fn count_open_blockers(&self, org_id: Uuid) -> OviaResult<i64> {
        let row = sqlx::query(
            "select count(*) as cnt from jira_issues
             where org_id = $1
               and priority in ('Blocker', 'Highest')
               and status not in ('Done', 'Closed', 'Resolved')",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;
        Ok(row.get::<i64, _>("cnt"))
    }

    /// List age in days for each open blocker (for release risk computation).
    pub async fn list_open_blocker_age_days(&self, org_id: Uuid) -> OviaResult<Vec<i32>> {
        let rows = sqlx::query(
            "select extract(day from (now() - created_at_jira))::integer as age_days
             from jira_issues
             where org_id = $1
               and priority in ('Blocker', 'Highest')
               and status not in ('Done', 'Closed', 'Resolved')
               and created_at_jira is not null",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(rows.iter().map(|r| r.get::<i32, _>("age_days")).collect())
    }

    /// Compute spillover rate: fraction of sprint-assigned issues that are unresolved.
    /// Returns 0.0 when no sprint-assigned issues exist.
    pub async fn spillover_rate(&self, org_id: Uuid) -> OviaResult<f64> {
        let row = sqlx::query(
            "select
               count(*) as total,
               count(*) filter (
                 where status not in ('Done', 'Closed', 'Resolved')
               ) as unresolved
             from jira_issues
             where org_id = $1
               and sprint_name is not null",
        )
        .bind(org_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        let total: i64 = row.get("total");
        let unresolved: i64 = row.get("unresolved");
        if total == 0 {
            return Ok(0.0);
        }
        Ok(unresolved as f64 / total as f64)
    }

    /// Get cycle times in hours for issues resolved in the given period.
    /// Cycle time = first "In Progress" transition → resolved_at.
    pub async fn get_cycle_times_hours(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> OviaResult<Vec<f64>> {
        let rows = sqlx::query(
            "select
               extract(epoch from (ji.resolved_at - t.started_at)) / 3600.0 as hours
             from jira_issues ji
             join lateral (
               select min(jit.transitioned_at) as started_at
               from jira_issue_transitions jit
               where jit.org_id = ji.org_id
                 and jit.jira_key = ji.jira_key
                 and jit.field = 'status'
                 and jit.to_value = 'In Progress'
             ) t on t.started_at is not null
             where ji.org_id = $1
               and ji.resolved_at is not null
               and ji.resolved_at >= $2::date
               and ji.resolved_at < $3::date
             order by hours",
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(rows.iter().map(|r| r.get::<f64, _>("hours")).collect())
    }

    /// Count resolved Jira issues in the given period.
    pub async fn count_resolved_issues(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> OviaResult<i64> {
        let row = sqlx::query(
            "select count(*) as cnt from jira_issues
             where org_id = $1
               and resolved_at >= $2::date
               and resolved_at < $3::date",
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;
        Ok(row.get::<i64, _>("cnt"))
    }

    /// Count resolved Jira issues matching any of the given issue types.
    pub async fn count_resolved_issues_by_types(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
        issue_types: &[&str],
    ) -> OviaResult<i64> {
        let types_vec: Vec<String> = issue_types.iter().map(|s| s.to_string()).collect();
        let row = sqlx::query(
            "select count(*) as cnt from jira_issues
             where org_id = $1
               and resolved_at >= $2::date
               and resolved_at < $3::date
               and issue_type = any($4)",
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .bind(&types_vec)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;
        Ok(row.get::<i64, _>("cnt"))
    }

    /// Count resolved Jira issues by issue_type in the given period.
    pub async fn count_resolved_issues_by_type(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
        issue_type: &str,
    ) -> OviaResult<i64> {
        let row = sqlx::query(
            "select count(*) as cnt from jira_issues
             where org_id = $1
               and resolved_at >= $2::date
               and resolved_at < $3::date
               and issue_type = $4",
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .bind(issue_type)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;
        Ok(row.get::<i64, _>("cnt"))
    }

    /// Delete all transitions for a given issue key (used before re-importing changelog).
    pub async fn delete_transitions_for_issue(
        &self,
        org_id: Uuid,
        jira_key: &str,
    ) -> OviaResult<u64> {
        let result =
            sqlx::query("delete from jira_issue_transitions where org_id = $1 and jira_key = $2")
                .bind(org_id)
                .bind(jira_key)
                .execute(&self.pool)
                .await
                .map_err(|e| OviaError::Database(e.to_string()))?;
        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_pool;
    use chrono::{Duration, Utc};

    async fn test_repo() -> Option<(PgJiraRepository, PgPool)> {
        let url = std::env::var("TEST_DATABASE_URL").ok()?;
        let pool = create_pool(&url).await.expect("db should connect");

        // Create tables inline for test isolation
        sqlx::query(
            "create table if not exists jira_issues (
              id uuid primary key default gen_random_uuid(),
              org_id uuid not null, jira_key text not null, project_key text not null,
              issue_type text, summary text not null, status text not null,
              assignee_account_id text, reporter_account_id text, priority text,
              story_points real, sprint_name text, sprint_id bigint, team_name text,
              labels text[] not null default '{}',
              created_at_jira timestamptz, updated_at_jira timestamptz, resolved_at timestamptz,
              raw_ref jsonb,
              created_at timestamptz not null default now(), updated_at timestamptz not null default now()
            )",
        )
        .execute(&pool)
        .await
        .ok()?;
        sqlx::query(
            "create unique index if not exists jira_issues_org_key_uidx on jira_issues(org_id, jira_key)",
        )
        .execute(&pool)
        .await
        .ok()?;

        sqlx::query(
            "create table if not exists jira_issue_transitions (
              id uuid primary key default gen_random_uuid(),
              org_id uuid not null, jira_key text not null, field text not null,
              from_value text, to_value text, author_account_id text,
              transitioned_at timestamptz not null,
              created_at timestamptz not null default now()
            )",
        )
        .execute(&pool)
        .await
        .ok()?;

        Some((PgJiraRepository::new(pool.clone()), pool))
    }

    fn make_issue(org_id: Uuid, key: &str) -> JiraIssue {
        let now = Utc::now();
        JiraIssue {
            id: Uuid::new_v4(),
            org_id,
            jira_key: key.to_string(),
            project_key: "BEE".to_string(),
            issue_type: Some("Story".to_string()),
            summary: format!("Test issue {key}"),
            status: "To Do".to_string(),
            assignee_account_id: Some("user-1".to_string()),
            reporter_account_id: Some("user-2".to_string()),
            priority: Some("Medium".to_string()),
            story_points: Some(3.0),
            sprint_name: Some("Sprint 1".to_string()),
            sprint_id: Some(100),
            team_name: Some("Team Alpha".to_string()),
            labels: vec!["backend".to_string()],
            created_at_jira: Some(now),
            updated_at_jira: Some(now),
            resolved_at: None,
            raw_ref: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn upsert_issue_inserts_and_updates() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let mut issue = make_issue(org, "BEE-1");
        repo.upsert_issue(&issue).await.expect("insert");

        issue.status = "In Progress".to_string();
        issue.story_points = Some(5.0);
        repo.upsert_issue(&issue).await.expect("update");
    }

    #[tokio::test]
    async fn upsert_issue_idempotent() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let issue = make_issue(org, "BEE-2");
        repo.upsert_issue(&issue).await.expect("first");
        repo.upsert_issue(&issue)
            .await
            .expect("second (idempotent)");
    }

    #[tokio::test]
    async fn insert_transition_and_delete() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let now = Utc::now();
        let t = JiraIssueTransition {
            id: Uuid::new_v4(),
            org_id: org,
            jira_key: "BEE-3".to_string(),
            field: "status".to_string(),
            from_value: Some("To Do".to_string()),
            to_value: Some("In Progress".to_string()),
            author_account_id: Some("user-1".to_string()),
            transitioned_at: now,
            created_at: now,
        };
        repo.insert_transition(&t).await.expect("insert transition");

        let deleted = repo
            .delete_transitions_for_issue(org, "BEE-3")
            .await
            .expect("delete");
        assert_eq!(deleted, 1);
    }

    // ── Jira KPI metrics tests ────────────────────────────────────

    #[tokio::test]
    async fn count_open_blockers_returns_correct_count() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();

        // Open blocker
        let mut b1 = make_issue(org, "BEE-B1");
        b1.priority = Some("Blocker".to_string());
        b1.status = "In Progress".to_string();
        repo.upsert_issue(&b1).await.expect("insert blocker");

        // Open highest-priority
        let mut b2 = make_issue(org, "BEE-B2");
        b2.priority = Some("Highest".to_string());
        b2.status = "To Do".to_string();
        repo.upsert_issue(&b2).await.expect("insert highest");

        // Resolved blocker (should not count)
        let mut b3 = make_issue(org, "BEE-B3");
        b3.priority = Some("Blocker".to_string());
        b3.status = "Done".to_string();
        b3.resolved_at = Some(Utc::now());
        repo.upsert_issue(&b3)
            .await
            .expect("insert resolved blocker");

        // Normal issue (should not count)
        let n1 = make_issue(org, "BEE-N1");
        repo.upsert_issue(&n1).await.expect("insert normal");

        let count = repo.count_open_blockers(org).await.expect("count");
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn count_open_blockers_returns_zero_when_none() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let count = repo.count_open_blockers(org).await.expect("count");
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn list_open_blocker_age_days_returns_ages() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();

        let mut b1 = make_issue(org, "BEE-AGE1");
        b1.priority = Some("Blocker".to_string());
        b1.status = "Open".to_string();
        b1.created_at_jira = Some(Utc::now() - Duration::days(5));
        repo.upsert_issue(&b1).await.expect("insert");

        let ages = repo.list_open_blocker_age_days(org).await.expect("ages");
        assert_eq!(ages.len(), 1);
        assert!(ages[0] >= 4); // at least 4 days (rounding)
    }

    #[tokio::test]
    async fn spillover_rate_with_mixed_issues() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();

        // 2 sprint issues: 1 done, 1 open → spillover = 0.5
        let mut done_issue = make_issue(org, "BEE-SP1");
        done_issue.sprint_name = Some("Sprint 10".to_string());
        done_issue.status = "Done".to_string();
        done_issue.resolved_at = Some(Utc::now());
        repo.upsert_issue(&done_issue).await.expect("insert done");

        let mut open_issue = make_issue(org, "BEE-SP2");
        open_issue.sprint_name = Some("Sprint 10".to_string());
        open_issue.status = "In Progress".to_string();
        repo.upsert_issue(&open_issue).await.expect("insert open");

        // Issue without sprint (should not count)
        let mut no_sprint = make_issue(org, "BEE-SP3");
        no_sprint.sprint_name = None;
        repo.upsert_issue(&no_sprint)
            .await
            .expect("insert no sprint");

        let rate = repo.spillover_rate(org).await.expect("rate");
        assert!((rate - 0.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn spillover_rate_returns_zero_when_no_sprint_issues() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let rate = repo.spillover_rate(org).await.expect("rate");
        assert!((rate - 0.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn get_cycle_times_hours_computes_durations() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let now = Utc::now();

        // Issue resolved 48h after going In Progress
        let mut issue = make_issue(org, "BEE-CT1");
        issue.status = "Done".to_string();
        issue.resolved_at = Some(now);
        repo.upsert_issue(&issue).await.expect("insert");

        let transition = JiraIssueTransition {
            id: Uuid::new_v4(),
            org_id: org,
            jira_key: "BEE-CT1".to_string(),
            field: "status".to_string(),
            from_value: Some("To Do".to_string()),
            to_value: Some("In Progress".to_string()),
            author_account_id: None,
            transitioned_at: now - Duration::hours(48),
            created_at: now - Duration::hours(48),
        };
        repo.insert_transition(&transition)
            .await
            .expect("insert transition");

        let today = now.date_naive();
        let from = today - chrono::Duration::days(1);
        let to = today + chrono::Duration::days(1);
        let times = repo
            .get_cycle_times_hours(org, from, to)
            .await
            .expect("cycle times");

        assert_eq!(times.len(), 1);
        assert!((times[0] - 48.0).abs() < 1.0);
    }

    #[tokio::test]
    async fn get_cycle_times_hours_empty_when_no_resolved() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let today = Utc::now().date_naive();
        let times = repo
            .get_cycle_times_hours(org, today, today + chrono::Duration::days(1))
            .await
            .expect("cycle times");
        assert!(times.is_empty());
    }

    #[tokio::test]
    async fn count_resolved_issues_and_by_type() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let now = Utc::now();

        let mut bug = make_issue(org, "BEE-R1");
        bug.issue_type = Some("Bug".to_string());
        bug.status = "Done".to_string();
        bug.resolved_at = Some(now);
        repo.upsert_issue(&bug).await.expect("insert bug");

        let mut story = make_issue(org, "BEE-R2");
        story.issue_type = Some("Story".to_string());
        story.status = "Done".to_string();
        story.resolved_at = Some(now);
        repo.upsert_issue(&story).await.expect("insert story");

        let mut task = make_issue(org, "BEE-R3");
        task.issue_type = Some("Task".to_string());
        task.status = "Done".to_string();
        task.resolved_at = Some(now);
        repo.upsert_issue(&task).await.expect("insert task");

        let today = now.date_naive();
        let from = today - chrono::Duration::days(1);
        let to = today + chrono::Duration::days(1);

        let total = repo
            .count_resolved_issues(org, from, to)
            .await
            .expect("total");
        assert_eq!(total, 3);

        let bugs = repo
            .count_resolved_issues_by_type(org, from, to, "Bug")
            .await
            .expect("bugs");
        assert_eq!(bugs, 1);

        let stories = repo
            .count_resolved_issues_by_type(org, from, to, "Story")
            .await
            .expect("stories");
        assert_eq!(stories, 1);
    }
}
