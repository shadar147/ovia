use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitlabProject {
    pub id: Uuid,
    pub org_id: Uuid,
    pub gitlab_id: i64,
    pub name: String,
    pub path_with_namespace: String,
    pub web_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitlabMergeRequest {
    pub id: Uuid,
    pub org_id: Uuid,
    pub gitlab_project_id: i64,
    pub gitlab_mr_iid: i64,
    pub title: String,
    pub state: String,
    pub author_username: Option<String>,
    pub labels: Vec<String>,
    pub created_at_gl: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
    pub web_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitlabPipeline {
    pub id: Uuid,
    pub org_id: Uuid,
    pub gitlab_project_id: i64,
    pub gitlab_pipeline_id: i64,
    pub status: String,
    pub ref_name: Option<String>,
    pub created_at_gl: Option<DateTime<Utc>>,
    pub finished_at_gl: Option<DateTime<Utc>>,
    pub duration_secs: Option<i32>,
    pub web_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Row returned by the review-duration query (created_at_gl → merged_at in hours).
#[derive(Debug, Clone)]
pub struct ReviewDurationRow {
    pub hours: f64,
}

/// Row returned by stale-MR listing for risk item generation.
#[derive(Debug, Clone)]
pub struct StaleMrRow {
    pub gitlab_mr_iid: i64,
    pub gitlab_project_id: i64,
    pub title: String,
    pub author_username: Option<String>,
    pub age_days: i32,
    pub web_url: String,
}
