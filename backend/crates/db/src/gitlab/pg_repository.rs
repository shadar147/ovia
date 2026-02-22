use chrono::NaiveDate;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::gitlab::models::{
    GitlabMergeRequest, GitlabPipeline, GitlabProject, ReviewDurationRow, StaleMrRow,
};
use ovia_common::error::{OviaError, OviaResult};

#[derive(Clone)]
pub struct PgGitlabRepository {
    pool: PgPool,
}

impl PgGitlabRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ── Upsert helpers ──────────────────────────────────────────────

    pub async fn upsert_project(&self, p: &GitlabProject) -> OviaResult<()> {
        sqlx::query(
            "insert into gitlab_projects (id, org_id, gitlab_id, name, path_with_namespace, web_url)
             values ($1, $2, $3, $4, $5, $6)
             on conflict (org_id, gitlab_id) do update set
               name = excluded.name,
               path_with_namespace = excluded.path_with_namespace,
               web_url = excluded.web_url,
               updated_at = now()",
        )
        .bind(p.id)
        .bind(p.org_id)
        .bind(p.gitlab_id)
        .bind(&p.name)
        .bind(&p.path_with_namespace)
        .bind(&p.web_url)
        .execute(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn upsert_merge_request(&self, mr: &GitlabMergeRequest) -> OviaResult<()> {
        sqlx::query(
            "insert into gitlab_merge_requests
             (id, org_id, gitlab_project_id, gitlab_mr_iid, title, state, author_username,
              labels, created_at_gl, merged_at, web_url)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             on conflict (org_id, gitlab_project_id, gitlab_mr_iid) do update set
               title = excluded.title,
               state = excluded.state,
               author_username = excluded.author_username,
               labels = excluded.labels,
               created_at_gl = excluded.created_at_gl,
               merged_at = excluded.merged_at,
               web_url = excluded.web_url,
               updated_at = now()",
        )
        .bind(mr.id)
        .bind(mr.org_id)
        .bind(mr.gitlab_project_id)
        .bind(mr.gitlab_mr_iid)
        .bind(&mr.title)
        .bind(&mr.state)
        .bind(&mr.author_username)
        .bind(&mr.labels)
        .bind(mr.created_at_gl)
        .bind(mr.merged_at)
        .bind(&mr.web_url)
        .execute(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn upsert_pipeline(&self, p: &GitlabPipeline) -> OviaResult<()> {
        sqlx::query(
            "insert into gitlab_pipelines
             (id, org_id, gitlab_project_id, gitlab_pipeline_id, status, ref_name,
              created_at_gl, finished_at_gl, duration_secs, web_url)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             on conflict (org_id, gitlab_pipeline_id) do update set
               status = excluded.status,
               ref_name = excluded.ref_name,
               created_at_gl = excluded.created_at_gl,
               finished_at_gl = excluded.finished_at_gl,
               duration_secs = excluded.duration_secs,
               web_url = excluded.web_url,
               updated_at = now()",
        )
        .bind(p.id)
        .bind(p.org_id)
        .bind(p.gitlab_project_id)
        .bind(p.gitlab_pipeline_id)
        .bind(&p.status)
        .bind(&p.ref_name)
        .bind(p.created_at_gl)
        .bind(p.finished_at_gl)
        .bind(p.duration_secs)
        .bind(&p.web_url)
        .execute(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;
        Ok(())
    }

    // ── KPI query helpers ───────────────────────────────────────────

    /// Count merged MRs in [from, to].
    pub async fn count_merged_mrs(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> OviaResult<i64> {
        let count: i64 = sqlx::query_scalar(
            "select count(*) from gitlab_merge_requests
             where org_id = $1 and state = 'merged'
               and merged_at >= $2::date and merged_at < ($3::date + interval '1 day')",
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;
        Ok(count)
    }

    /// Count merged MRs that contain any of the given labels.
    pub async fn count_merged_mrs_by_labels(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
        labels: &[&str],
    ) -> OviaResult<i64> {
        let labels_vec: Vec<String> = labels.iter().map(|s| s.to_string()).collect();
        let count: i64 = sqlx::query_scalar(
            "select count(*) from gitlab_merge_requests
             where org_id = $1 and state = 'merged'
               and merged_at >= $2::date and merged_at < ($3::date + interval '1 day')
               and labels && $4",
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .bind(&labels_vec)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;
        Ok(count)
    }

    /// Count merged MRs that contain a given label.
    pub async fn count_merged_mrs_by_label(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
        label: &str,
    ) -> OviaResult<i64> {
        let count: i64 = sqlx::query_scalar(
            "select count(*) from gitlab_merge_requests
             where org_id = $1 and state = 'merged'
               and merged_at >= $2::date and merged_at < ($3::date + interval '1 day')
               and $4 = any(labels)",
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .bind(label)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;
        Ok(count)
    }

    /// Return review durations in hours (merged_at - created_at_gl) sorted ascending.
    pub async fn get_review_durations_hours(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> OviaResult<Vec<ReviewDurationRow>> {
        let rows = sqlx::query(
            "select (extract(epoch from (merged_at - created_at_gl)) / 3600.0)::float8 as hours
             from gitlab_merge_requests
             where org_id = $1 and state = 'merged'
               and merged_at >= $2::date and merged_at < ($3::date + interval '1 day')
               and created_at_gl is not null
             order by hours asc",
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| ReviewDurationRow {
                hours: r.get::<f64, _>("hours"),
            })
            .collect())
    }

    /// Count pipelines with a given status in [from, to].
    pub async fn count_pipelines_by_status(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
        status: &str,
    ) -> OviaResult<i64> {
        let count: i64 = sqlx::query_scalar(
            "select count(*) from gitlab_pipelines
             where org_id = $1 and status = $4
               and created_at_gl >= $2::date and created_at_gl < ($3::date + interval '1 day')",
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;
        Ok(count)
    }

    /// Percentage of open MRs older than `stale_days` relative to all open MRs.
    pub async fn stale_mr_percentage(&self, org_id: Uuid, stale_days: i32) -> OviaResult<f64> {
        let row = sqlx::query(
            "select
               count(*) filter (where created_at_gl < now() - ($2 || ' days')::interval) as stale,
               count(*) as total
             from gitlab_merge_requests
             where org_id = $1 and state = 'opened'",
        )
        .bind(org_id)
        .bind(stale_days)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        let stale: i64 = row.get("stale");
        let total: i64 = row.get("total");
        if total == 0 {
            return Ok(0.0);
        }
        Ok(stale as f64 / total as f64)
    }

    /// List open MRs older than `stale_days` for risk item generation.
    pub async fn list_stale_open_mrs(
        &self,
        org_id: Uuid,
        stale_days: i32,
    ) -> OviaResult<Vec<StaleMrRow>> {
        let rows = sqlx::query(
            "select gitlab_mr_iid, gitlab_project_id, title, author_username,
                    extract(day from now() - created_at_gl)::int as age_days,
                    web_url
             from gitlab_merge_requests
             where org_id = $1 and state = 'opened'
               and created_at_gl < now() - ($2 || ' days')::interval
             order by created_at_gl asc",
        )
        .bind(org_id)
        .bind(stale_days)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(rows
            .iter()
            .map(|r| StaleMrRow {
                gitlab_mr_iid: r.get("gitlab_mr_iid"),
                gitlab_project_id: r.get("gitlab_project_id"),
                title: r.get("title"),
                author_username: r.get("author_username"),
                age_days: r.get("age_days"),
                web_url: r.get("web_url"),
            })
            .collect())
    }

    /// List failed pipelines in a period for risk item generation.
    pub async fn list_failed_pipelines(
        &self,
        org_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> OviaResult<Vec<GitlabPipeline>> {
        let rows = sqlx::query(
            "select id, org_id, gitlab_project_id, gitlab_pipeline_id, status, ref_name,
                    created_at_gl, finished_at_gl, duration_secs, web_url, created_at, updated_at
             from gitlab_pipelines
             where org_id = $1 and status = 'failed'
               and created_at_gl >= $2::date and created_at_gl < ($3::date + interval '1 day')
             order by created_at_gl desc",
        )
        .bind(org_id)
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OviaError::Database(e.to_string()))?;

        Ok(rows.iter().map(map_pipeline_row).collect())
    }
}

fn map_pipeline_row(row: &sqlx::postgres::PgRow) -> GitlabPipeline {
    GitlabPipeline {
        id: row.get("id"),
        org_id: row.get("org_id"),
        gitlab_project_id: row.get("gitlab_project_id"),
        gitlab_pipeline_id: row.get("gitlab_pipeline_id"),
        status: row.get("status"),
        ref_name: row.get("ref_name"),
        created_at_gl: row.get("created_at_gl"),
        finished_at_gl: row.get("finished_at_gl"),
        duration_secs: row.get("duration_secs"),
        web_url: row.get("web_url"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_pool;
    use chrono::{DateTime, Utc};

    async fn test_repo() -> Option<(PgGitlabRepository, PgPool)> {
        let url = std::env::var("TEST_DATABASE_URL").ok()?;
        let pool = create_pool(&url).await.expect("db should connect");

        // Apply migration inline for test isolation
        sqlx::query(
            "create table if not exists gitlab_projects (
              id uuid primary key default gen_random_uuid(),
              org_id uuid not null, gitlab_id bigint not null,
              name text not null, path_with_namespace text not null, web_url text not null,
              created_at timestamptz not null default now(), updated_at timestamptz not null default now()
            )",
        )
        .execute(&pool)
        .await
        .ok()?;
        sqlx::query(
            "create unique index if not exists gitlab_projects_org_gl_uidx on gitlab_projects(org_id, gitlab_id)",
        )
        .execute(&pool)
        .await
        .ok()?;

        sqlx::query(
            "create table if not exists gitlab_merge_requests (
              id uuid primary key default gen_random_uuid(),
              org_id uuid not null, gitlab_project_id bigint not null, gitlab_mr_iid bigint not null,
              title text not null, state text not null, author_username text,
              labels text[] not null default '{}',
              created_at_gl timestamptz, merged_at timestamptz, web_url text not null,
              created_at timestamptz not null default now(), updated_at timestamptz not null default now()
            )",
        )
        .execute(&pool)
        .await
        .ok()?;
        sqlx::query(
            "create unique index if not exists gitlab_mrs_org_proj_iid_uidx on gitlab_merge_requests(org_id, gitlab_project_id, gitlab_mr_iid)",
        )
        .execute(&pool)
        .await
        .ok()?;

        sqlx::query(
            "create table if not exists gitlab_pipelines (
              id uuid primary key default gen_random_uuid(),
              org_id uuid not null, gitlab_project_id bigint not null, gitlab_pipeline_id bigint not null,
              status text not null, ref_name text,
              created_at_gl timestamptz, finished_at_gl timestamptz, duration_secs integer,
              web_url text not null,
              created_at timestamptz not null default now(), updated_at timestamptz not null default now()
            )",
        )
        .execute(&pool)
        .await
        .ok()?;
        sqlx::query(
            "create unique index if not exists gitlab_pipelines_org_gl_uidx on gitlab_pipelines(org_id, gitlab_pipeline_id)",
        )
        .execute(&pool)
        .await
        .ok()?;

        Some((PgGitlabRepository::new(pool.clone()), pool))
    }

    fn make_project(org_id: Uuid, gitlab_id: i64) -> GitlabProject {
        let now = Utc::now();
        GitlabProject {
            id: Uuid::new_v4(),
            org_id,
            gitlab_id,
            name: format!("project-{gitlab_id}"),
            path_with_namespace: format!("group/project-{gitlab_id}"),
            web_url: format!("https://gitlab.example.com/group/project-{gitlab_id}"),
            created_at: now,
            updated_at: now,
        }
    }

    fn make_mr(
        org_id: Uuid,
        project_id: i64,
        iid: i64,
        state: &str,
        labels: Vec<String>,
        created: DateTime<Utc>,
        merged: Option<DateTime<Utc>>,
    ) -> GitlabMergeRequest {
        let now = Utc::now();
        GitlabMergeRequest {
            id: Uuid::new_v4(),
            org_id,
            gitlab_project_id: project_id,
            gitlab_mr_iid: iid,
            title: format!("MR !{iid}"),
            state: state.to_string(),
            author_username: Some("dev".to_string()),
            labels,
            created_at_gl: Some(created),
            merged_at: merged,
            web_url: format!("https://gitlab.example.com/group/project/merge_requests/{iid}"),
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn upsert_project_inserts_and_updates() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let mut p = make_project(org, 100);
        repo.upsert_project(&p).await.expect("insert");

        p.name = "renamed".to_string();
        repo.upsert_project(&p).await.expect("update");
    }

    #[tokio::test]
    async fn count_merged_mrs_returns_correct_count() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let now = Utc::now();
        let yesterday = now - chrono::Duration::hours(12);

        let mr1 = make_mr(org, 1, 1, "merged", vec![], yesterday, Some(now));
        let mr2 = make_mr(
            org,
            1,
            2,
            "merged",
            vec!["bug".into()],
            yesterday,
            Some(now),
        );
        let mr3 = make_mr(org, 1, 3, "opened", vec![], yesterday, None);

        repo.upsert_merge_request(&mr1).await.expect("upsert mr1");
        repo.upsert_merge_request(&mr2).await.expect("upsert mr2");
        repo.upsert_merge_request(&mr3).await.expect("upsert mr3");

        let today = now.date_naive();
        let count = repo
            .count_merged_mrs(org, today, today)
            .await
            .expect("count");
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn count_merged_mrs_by_label_filters() {
        let (repo, _pool) = match test_repo().await {
            Some(r) => r,
            None => return,
        };
        let org = Uuid::new_v4();
        let now = Utc::now();
        let yesterday = now - chrono::Duration::hours(12);

        let mr1 = make_mr(
            org,
            1,
            10,
            "merged",
            vec!["bug".into()],
            yesterday,
            Some(now),
        );
        let mr2 = make_mr(
            org,
            1,
            11,
            "merged",
            vec!["feature".into()],
            yesterday,
            Some(now),
        );

        repo.upsert_merge_request(&mr1).await.expect("mr1");
        repo.upsert_merge_request(&mr2).await.expect("mr2");

        let today = now.date_naive();
        let bugs = repo
            .count_merged_mrs_by_label(org, today, today, "bug")
            .await
            .expect("bugs");
        assert_eq!(bugs, 1);
    }
}
