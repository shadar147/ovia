use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssue {
    pub id: Uuid,
    pub org_id: Uuid,
    pub jira_key: String,
    pub project_key: String,
    pub issue_type: Option<String>,
    pub summary: String,
    pub status: String,
    pub assignee_account_id: Option<String>,
    pub reporter_account_id: Option<String>,
    pub priority: Option<String>,
    pub story_points: Option<f32>,
    pub sprint_name: Option<String>,
    pub sprint_id: Option<i64>,
    pub team_name: Option<String>,
    pub labels: Vec<String>,
    pub created_at_jira: Option<DateTime<Utc>>,
    pub updated_at_jira: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub raw_ref: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssueTransition {
    pub id: Uuid,
    pub org_id: Uuid,
    pub jira_key: String,
    pub field: String,
    pub from_value: Option<String>,
    pub to_value: Option<String>,
    pub author_account_id: Option<String>,
    pub transitioned_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
