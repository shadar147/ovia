use async_trait::async_trait;
use chrono::{Duration, Utc};
use uuid::Uuid;

use ovia_db::jira::models::{JiraIssue as DbJiraIssue, JiraIssueTransition};
use ovia_db::jira::pg_repository::PgJiraRepository;
use ovia_db::sync::repositories::SyncWatermarkRepository;

use super::client::JiraClient;
use super::models::{JiraChangelogEntry, JiraIssue as ApiIssue};
use super::query::build_issue_search_jql;
use crate::connector::{Connector, SyncResult};

const SOURCE_NAME: &str = "jira_issues";

/// Convert an API issue to a DB issue row.
fn api_issue_to_db(org_id: Uuid, issue: &ApiIssue) -> DbJiraIssue {
    let now = Utc::now();
    let f = &issue.fields;
    let project_key = issue.key.split('-').next().unwrap_or("").to_string();
    let latest_sprint = f.latest_sprint();

    DbJiraIssue {
        id: Uuid::new_v4(),
        org_id,
        jira_key: issue.key.clone(),
        project_key,
        issue_type: f.issuetype.as_ref().map(|t| t.name.clone()),
        summary: f.summary.clone(),
        status: f.status.name.clone(),
        assignee_account_id: f.assignee.as_ref().map(|a| a.account_id.clone()),
        reporter_account_id: f.reporter.as_ref().map(|r| r.account_id.clone()),
        priority: f.priority.as_ref().map(|p| p.name.clone()),
        story_points: f.story_points.map(|sp| sp as f32),
        sprint_name: latest_sprint.map(|s| s.name.clone()),
        sprint_id: latest_sprint.map(|s| s.id),
        team_name: f.team_name(),
        labels: f.labels.clone(),
        created_at_jira: f.created,
        updated_at_jira: f.updated,
        resolved_at: f.resolution_date,
        raw_ref: serde_json::to_value(issue).ok(),
        created_at: now,
        updated_at: now,
    }
}

/// Extract status and sprint transitions from a changelog.
fn changelog_to_transitions(
    org_id: Uuid,
    issue_key: &str,
    entries: &[JiraChangelogEntry],
) -> Vec<JiraIssueTransition> {
    let now = Utc::now();
    let mut transitions = Vec::new();

    for entry in entries {
        for item in &entry.items {
            let field = item.field.to_lowercase();
            if field != "status" && field != "sprint" {
                continue;
            }

            transitions.push(JiraIssueTransition {
                id: Uuid::new_v4(),
                org_id,
                jira_key: issue_key.to_string(),
                field: field.clone(),
                from_value: item.from_string.clone(),
                to_value: item.to_string.clone(),
                author_account_id: entry.author.as_ref().map(|a| a.account_id.clone()),
                transitioned_at: entry.created,
                created_at: now,
            });
        }
    }

    transitions
}

pub struct JiraIssueSyncer<S> {
    org_id: Uuid,
    client: JiraClient,
    jira_repo: PgJiraRepository,
    sync_repo: S,
}

impl<S> JiraIssueSyncer<S>
where
    S: SyncWatermarkRepository,
{
    pub fn new(
        org_id: Uuid,
        client: JiraClient,
        jira_repo: PgJiraRepository,
        sync_repo: S,
    ) -> Self {
        Self {
            org_id,
            client,
            jira_repo,
            sync_repo,
        }
    }
}

#[async_trait]
impl<S> Connector for JiraIssueSyncer<S>
where
    S: SyncWatermarkRepository,
{
    fn source_name(&self) -> &str {
        SOURCE_NAME
    }

    async fn sync(&self) -> Result<SyncResult, Box<dyn std::error::Error + Send + Sync>> {
        // Ensure watermark row exists
        self.sync_repo
            .get_or_create(self.org_id, SOURCE_NAME)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

        // Try to acquire lock
        let watermark = self
            .sync_repo
            .acquire_lock(self.org_id, SOURCE_NAME)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

        let watermark = match watermark {
            Some(wm) => wm,
            None => {
                tracing::info!(
                    "jira issue sync already running for org={}, skipping",
                    self.org_id
                );
                return Ok(SyncResult {
                    source: SOURCE_NAME.to_string(),
                    upserted: 0,
                    skipped: 0,
                    errors: 0,
                });
            }
        };

        // Determine updated_after: use cursor_value or fall back to sync_window_days
        let updated_after = watermark
            .cursor_value
            .as_deref()
            .and_then(|v| chrono::DateTime::parse_from_rfc3339(v).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| {
                Utc::now() - Duration::days(i64::from(self.client.config().sync_window_days))
            });

        let project_keys = &self.client.config().project_keys;
        let jql = build_issue_search_jql(project_keys, updated_after);
        tracing::info!(jql = %jql, "searching jira issues");

        // Fetch issues
        let issues = match self.client.search_issues(&jql).await {
            Ok(issues) => issues,
            Err(e) => {
                let msg = e.to_string();
                tracing::error!(error = %msg, "jira issue search failed");
                self.sync_repo
                    .mark_failed(watermark.id, &msg)
                    .await
                    .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
                return Err(Box::new(e));
            }
        };

        tracing::info!(count = issues.len(), "fetched jira issues");

        let mut upserted: usize = 0;
        let mut errors: usize = 0;

        for issue in &issues {
            // Upsert the issue itself
            let db_issue = api_issue_to_db(self.org_id, issue);
            match self.jira_repo.upsert_issue(&db_issue).await {
                Ok(_) => upserted += 1,
                Err(e) => {
                    tracing::warn!(
                        key = %issue.key,
                        error = %e,
                        "failed to upsert jira issue"
                    );
                    errors += 1;
                    continue;
                }
            }

            // Fetch and store changelog (replace strategy: delete old, insert new)
            match self.client.fetch_issue_changelog(&issue.key).await {
                Ok(entries) => {
                    let transitions = changelog_to_transitions(self.org_id, &issue.key, &entries);

                    if !transitions.is_empty() {
                        if let Err(e) = self
                            .jira_repo
                            .delete_transitions_for_issue(self.org_id, &issue.key)
                            .await
                        {
                            tracing::warn!(
                                key = %issue.key,
                                error = %e,
                                "failed to delete old transitions"
                            );
                        }

                        for t in &transitions {
                            if let Err(e) = self.jira_repo.insert_transition(t).await {
                                tracing::warn!(
                                    key = %issue.key,
                                    field = %t.field,
                                    error = %e,
                                    "failed to insert transition"
                                );
                                errors += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        key = %issue.key,
                        error = %e,
                        "failed to fetch changelog"
                    );
                    errors += 1;
                }
            }
        }

        // Mark completed with current timestamp as cursor for next incremental sync
        let cursor = Utc::now().to_rfc3339();
        self.sync_repo
            .mark_completed(watermark.id, Some(&cursor))
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

        let result = SyncResult {
            source: SOURCE_NAME.to_string(),
            upserted,
            skipped: 0,
            errors,
        };

        tracing::info!(?result, "jira issue sync completed");
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jira::client::{JiraClient, JiraClientConfig};
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_client_config(base_url: &str) -> JiraClientConfig {
        JiraClientConfig {
            base_url: base_url.to_string(),
            email: "test@example.com".to_string(),
            api_token: "token".to_string(),
            project_keys: vec!["BEE".to_string()],
            sync_window_days: 7,
            max_retries: 1,
            timeout_secs: 5,
        }
    }

    fn make_search_response(issues: Vec<serde_json::Value>) -> serde_json::Value {
        let total = issues.len();
        serde_json::json!({
            "startAt": 0,
            "maxResults": 50,
            "total": total,
            "issues": issues
        })
    }

    fn make_issue_json(key: &str, status: &str, story_points: Option<f64>) -> serde_json::Value {
        serde_json::json!({
            "key": key,
            "fields": {
                "summary": format!("Test issue {key}"),
                "status": { "name": status },
                "issuetype": { "name": "Story" },
                "assignee": { "accountId": "user-1" },
                "reporter": { "accountId": "user-2" },
                "priority": { "name": "Medium" },
                "labels": ["backend"],
                "created": "2026-02-10T10:00:00.000Z",
                "updated": "2026-02-20T15:00:00.000Z",
                "resolutiondate": null,
                "customfield_10016": story_points,
                "customfield_10020": [{
                    "id": 100,
                    "name": "Sprint 1",
                    "state": "active"
                }],
                "customfield_10001": "Team Alpha"
            }
        })
    }

    fn make_changelog_response(entries: Vec<serde_json::Value>) -> serde_json::Value {
        serde_json::json!({
            "startAt": 0,
            "maxResults": 100,
            "isLast": true,
            "values": entries
        })
    }

    fn make_status_transition(from: &str, to: &str) -> serde_json::Value {
        serde_json::json!({
            "author": { "accountId": "user-1" },
            "created": "2026-02-15T12:00:00.000Z",
            "items": [{
                "field": "status",
                "fromString": from,
                "toString": to
            }]
        })
    }

    // ── Model conversion tests ──────────────────────────────────

    #[test]
    fn api_issue_to_db_extracts_all_fields() {
        let json = make_issue_json("BEE-42", "In Progress", Some(5.0));
        let api_issue: ApiIssue = serde_json::from_value(json).unwrap();

        let org_id = Uuid::new_v4();
        let db_issue = api_issue_to_db(org_id, &api_issue);

        assert_eq!(db_issue.org_id, org_id);
        assert_eq!(db_issue.jira_key, "BEE-42");
        assert_eq!(db_issue.project_key, "BEE");
        assert_eq!(db_issue.status, "In Progress");
        assert_eq!(db_issue.issue_type.as_deref(), Some("Story"));
        assert_eq!(db_issue.assignee_account_id.as_deref(), Some("user-1"));
        assert_eq!(db_issue.reporter_account_id.as_deref(), Some("user-2"));
        assert_eq!(db_issue.priority.as_deref(), Some("Medium"));
        assert!((db_issue.story_points.unwrap() - 5.0).abs() < 0.01);
        assert_eq!(db_issue.sprint_name.as_deref(), Some("Sprint 1"));
        assert_eq!(db_issue.sprint_id, Some(100));
        assert_eq!(db_issue.team_name.as_deref(), Some("Team Alpha"));
        assert_eq!(db_issue.labels, vec!["backend"]);
        assert!(db_issue.created_at_jira.is_some());
        assert!(db_issue.resolved_at.is_none());
        assert!(db_issue.raw_ref.is_some());
    }

    #[test]
    fn changelog_extracts_status_and_sprint_transitions() {
        let entries_json = vec![
            serde_json::json!({
                "author": { "accountId": "user-1" },
                "created": "2026-02-12T10:00:00.000Z",
                "items": [
                    { "field": "status", "fromString": "To Do", "toString": "In Progress" },
                    { "field": "assignee", "fromString": null, "toString": "User 1" }
                ]
            }),
            serde_json::json!({
                "author": { "accountId": "user-2" },
                "created": "2026-02-14T14:00:00.000Z",
                "items": [
                    { "field": "Sprint", "fromString": "Sprint 0", "toString": "Sprint 1" }
                ]
            }),
        ];

        let entries: Vec<JiraChangelogEntry> =
            serde_json::from_value(serde_json::Value::Array(entries_json)).unwrap();

        let org_id = Uuid::new_v4();
        let transitions = changelog_to_transitions(org_id, "BEE-1", &entries);

        // Should get 2: status + sprint. Assignee is filtered out.
        assert_eq!(transitions.len(), 2);
        assert_eq!(transitions[0].field, "status");
        assert_eq!(transitions[0].from_value.as_deref(), Some("To Do"));
        assert_eq!(transitions[0].to_value.as_deref(), Some("In Progress"));
        assert_eq!(transitions[0].author_account_id.as_deref(), Some("user-1"));
        assert_eq!(transitions[1].field, "sprint");
        assert_eq!(transitions[1].from_value.as_deref(), Some("Sprint 0"));
        assert_eq!(transitions[1].to_value.as_deref(), Some("Sprint 1"));
        assert_eq!(transitions[1].author_account_id.as_deref(), Some("user-2"));
    }

    #[test]
    fn changelog_ignores_non_status_sprint_fields() {
        let entries_json = vec![serde_json::json!({
            "author": { "accountId": "user-1" },
            "created": "2026-02-12T10:00:00.000Z",
            "items": [
                { "field": "assignee", "fromString": null, "toString": "User 1" },
                { "field": "priority", "fromString": "Low", "toString": "High" },
                { "field": "labels", "fromString": "", "toString": "backend" }
            ]
        })];

        let entries: Vec<JiraChangelogEntry> =
            serde_json::from_value(serde_json::Value::Array(entries_json)).unwrap();

        let transitions = changelog_to_transitions(Uuid::new_v4(), "BEE-1", &entries);
        assert!(transitions.is_empty());
    }

    #[test]
    fn issue_fields_latest_sprint_prefers_active() {
        let json = serde_json::json!({
            "summary": "test",
            "status": { "name": "Open" },
            "labels": [],
            "customfield_10020": [
                { "id": 1, "name": "Sprint 1", "state": "closed" },
                { "id": 2, "name": "Sprint 2", "state": "active" },
                { "id": 3, "name": "Sprint 3", "state": "future" }
            ]
        });
        let fields: super::super::models::JiraIssueFields = serde_json::from_value(json).unwrap();

        let sprint = fields.latest_sprint().unwrap();
        assert_eq!(sprint.name, "Sprint 2");
        assert_eq!(sprint.id, 2);
    }

    #[test]
    fn issue_fields_latest_sprint_falls_back_to_last() {
        let json = serde_json::json!({
            "summary": "test",
            "status": { "name": "Open" },
            "labels": [],
            "customfield_10020": [
                { "id": 1, "name": "Sprint 1", "state": "closed" },
                { "id": 3, "name": "Sprint 3", "state": "future" }
            ]
        });
        let fields: super::super::models::JiraIssueFields = serde_json::from_value(json).unwrap();

        let sprint = fields.latest_sprint().unwrap();
        assert_eq!(sprint.name, "Sprint 3");
    }

    #[test]
    fn issue_fields_team_name_from_string() {
        let json = serde_json::json!({
            "summary": "test",
            "status": { "name": "Open" },
            "labels": [],
            "customfield_10001": "Team Alpha"
        });
        let fields: super::super::models::JiraIssueFields = serde_json::from_value(json).unwrap();
        assert_eq!(fields.team_name().as_deref(), Some("Team Alpha"));
    }

    #[test]
    fn issue_fields_team_name_from_object() {
        let json = serde_json::json!({
            "summary": "test",
            "status": { "name": "Open" },
            "labels": [],
            "customfield_10001": { "name": "Team Beta" }
        });
        let fields: super::super::models::JiraIssueFields = serde_json::from_value(json).unwrap();
        assert_eq!(fields.team_name().as_deref(), Some("Team Beta"));
    }

    #[test]
    fn issue_fields_team_name_none() {
        let json = serde_json::json!({
            "summary": "test",
            "status": { "name": "Open" },
            "labels": []
        });
        let fields: super::super::models::JiraIssueFields = serde_json::from_value(json).unwrap();
        assert!(fields.team_name().is_none());
    }

    // ── Deserialization tests ───────────────────────────────────

    #[test]
    fn search_response_deserializes() {
        let json = make_search_response(vec![
            make_issue_json("BEE-1", "Open", Some(3.0)),
            make_issue_json("BEE-2", "Done", None),
        ]);
        let response: super::super::models::JiraSearchResponse =
            serde_json::from_value(json).unwrap();
        assert_eq!(response.total, 2);
        assert_eq!(response.issues.len(), 2);
        assert_eq!(response.issues[0].key, "BEE-1");
    }

    #[test]
    fn changelog_response_deserializes() {
        let json = make_changelog_response(vec![make_status_transition("To Do", "In Progress")]);
        let response: super::super::models::JiraChangelogResponse =
            serde_json::from_value(json).unwrap();
        assert!(response.is_last);
        assert_eq!(response.values.len(), 1);
        assert_eq!(response.values[0].items.len(), 1);
        assert_eq!(response.values[0].items[0].field, "status");
    }

    #[test]
    fn minimal_issue_deserializes() {
        let json = serde_json::json!({
            "key": "BEE-1",
            "fields": {
                "summary": "Minimal issue",
                "status": { "name": "Open" },
                "labels": []
            }
        });
        let issue: ApiIssue = serde_json::from_value(json).unwrap();
        assert_eq!(issue.key, "BEE-1");
        assert!(issue.fields.assignee.is_none());
        assert!(issue.fields.story_points.is_none());
        assert!(issue.fields.sprints.is_none());
        assert!(issue.fields.team.is_none());
    }

    // ── Idempotency test (model level) ──────────────────────────

    #[test]
    fn api_issue_to_db_is_deterministic_except_id() {
        let json = make_issue_json("BEE-99", "Done", Some(8.0));
        let api_issue: ApiIssue = serde_json::from_value(json).unwrap();
        let org_id = Uuid::new_v4();

        let a = api_issue_to_db(org_id, &api_issue);
        let b = api_issue_to_db(org_id, &api_issue);

        // IDs differ (Uuid::new_v4), but all other fields match
        assert_ne!(a.id, b.id);
        assert_eq!(a.jira_key, b.jira_key);
        assert_eq!(a.status, b.status);
        assert_eq!(a.story_points, b.story_points);
        assert_eq!(a.sprint_name, b.sprint_name);
        assert_eq!(a.team_name, b.team_name);
        assert_eq!(a.org_id, b.org_id);
    }

    // ── Client integration tests (wiremock) ─────────────────────

    #[tokio::test]
    async fn search_issues_single_page() {
        let server = MockServer::start().await;

        let response = make_search_response(vec![make_issue_json("BEE-1", "Open", Some(3.0))]);

        Mock::given(method("GET"))
            .and(path("/rest/api/3/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response))
            .mount(&server)
            .await;

        let client = JiraClient::new(test_client_config(&server.uri())).unwrap();
        let issues = client.search_issues("project in (BEE)").await.unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].key, "BEE-1");
    }

    #[tokio::test]
    async fn search_issues_multiple_pages() {
        let server = MockServer::start().await;

        // Page 1: 50 issues (triggers next page)
        let issues_p1: Vec<serde_json::Value> = (0..50)
            .map(|i| make_issue_json(&format!("BEE-{i}"), "Open", None))
            .collect();
        let response_p1 = serde_json::json!({
            "startAt": 0,
            "maxResults": 50,
            "total": 60,
            "issues": issues_p1
        });

        // Mount page 2 first (more specific), then page 1 as fallback
        // Page 2: 10 issues (last page)
        let issues_p2: Vec<serde_json::Value> = (50..60)
            .map(|i| make_issue_json(&format!("BEE-{i}"), "Open", None))
            .collect();
        let response_p2 = serde_json::json!({
            "startAt": 50,
            "maxResults": 50,
            "total": 60,
            "issues": issues_p2
        });

        Mock::given(method("GET"))
            .and(path("/rest/api/3/search"))
            .and(query_param("startAt", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_p2))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/rest/api/3/search"))
            .and(query_param("startAt", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_p1))
            .mount(&server)
            .await;

        let client = JiraClient::new(test_client_config(&server.uri())).unwrap();
        let issues = client.search_issues("project in (BEE)").await.unwrap();
        assert_eq!(issues.len(), 60);
    }

    #[tokio::test]
    async fn fetch_changelog_single_page() {
        let server = MockServer::start().await;

        let response = make_changelog_response(vec![
            make_status_transition("To Do", "In Progress"),
            make_status_transition("In Progress", "Done"),
        ]);

        Mock::given(method("GET"))
            .and(path("/rest/api/3/issue/BEE-1/changelog"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response))
            .mount(&server)
            .await;

        let client = JiraClient::new(test_client_config(&server.uri())).unwrap();
        let entries = client.fetch_issue_changelog("BEE-1").await.unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[tokio::test]
    async fn search_issues_empty_result() {
        let server = MockServer::start().await;

        let response = serde_json::json!({
            "startAt": 0,
            "maxResults": 50,
            "total": 0,
            "issues": []
        });

        Mock::given(method("GET"))
            .and(path("/rest/api/3/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response))
            .mount(&server)
            .await;

        let client = JiraClient::new(test_client_config(&server.uri())).unwrap();
        let issues = client.search_issues("project in (BEE)").await.unwrap();
        assert!(issues.is_empty());
    }

    #[tokio::test]
    async fn fetch_changelog_empty() {
        let server = MockServer::start().await;

        let response = serde_json::json!({
            "startAt": 0,
            "maxResults": 100,
            "isLast": true,
            "values": []
        });

        Mock::given(method("GET"))
            .and(path("/rest/api/3/issue/BEE-1/changelog"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response))
            .mount(&server)
            .await;

        let client = JiraClient::new(test_client_config(&server.uri())).unwrap();
        let entries = client.fetch_issue_changelog("BEE-1").await.unwrap();
        assert!(entries.is_empty());
    }
}
