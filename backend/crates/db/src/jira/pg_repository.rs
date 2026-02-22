use sqlx::PgPool;
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

    /// Delete all transitions for a given issue key (used before re-importing changelog).
    pub async fn delete_transitions_for_issue(
        &self,
        org_id: Uuid,
        jira_key: &str,
    ) -> OviaResult<u64> {
        let result = sqlx::query(
            "delete from jira_issue_transitions where org_id = $1 and jira_key = $2",
        )
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
    use chrono::Utc;

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
        repo.upsert_issue(&issue).await.expect("second (idempotent)");
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
}
