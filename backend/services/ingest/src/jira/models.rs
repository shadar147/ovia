use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A user record from the Jira Cloud REST API (`/rest/api/3/users/search`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraUser {
    pub account_id: String,
    pub email_address: Option<String>,
    pub display_name: Option<String>,
    #[serde(default)]
    pub active: bool,
    pub account_type: Option<String>,
}

impl JiraUser {
    /// Returns `true` if the account looks like a service/bot account.
    pub fn is_service_account(&self) -> bool {
        matches!(self.account_type.as_deref(), Some("app"))
    }
}

// ── Issue search API response types ─────────────────────────────

/// Top-level response from `/rest/api/3/search`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraSearchResponse {
    #[allow(dead_code)]
    pub start_at: usize,
    #[allow(dead_code)]
    pub max_results: usize,
    pub total: usize,
    #[serde(default)]
    pub issues: Vec<JiraIssue>,
}

/// A single issue from the search response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssue {
    pub key: String,
    pub fields: JiraIssueFields,
}

/// Issue fields we care about.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssueFields {
    pub summary: String,
    pub status: JiraStatus,
    #[serde(default)]
    pub issuetype: Option<JiraIssueType>,
    #[serde(default)]
    pub assignee: Option<JiraUserRef>,
    #[serde(default)]
    pub reporter: Option<JiraUserRef>,
    #[serde(default)]
    pub priority: Option<JiraPriority>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub created: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated: Option<DateTime<Utc>>,
    #[serde(default)]
    pub resolution_date: Option<DateTime<Utc>>,
    /// Story points — customfield_10016
    #[serde(default, rename = "customfield_10016")]
    pub story_points: Option<f64>,
    /// Sprint — customfield_10020 (array of sprint objects)
    #[serde(default, rename = "customfield_10020")]
    pub sprints: Option<Vec<JiraSprint>>,
    /// Team — customfield_10001 (string or object with name)
    #[serde(default, rename = "customfield_10001")]
    pub team: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraStatus {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssueType {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraUserRef {
    pub account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraPriority {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraSprint {
    pub id: i64,
    pub name: String,
    pub state: Option<String>,
}

impl JiraIssueFields {
    /// Extract the latest (active or most recent) sprint.
    pub fn latest_sprint(&self) -> Option<&JiraSprint> {
        let sprints = self.sprints.as_ref()?;
        // Prefer active sprint, fall back to last in array
        sprints
            .iter()
            .find(|s| s.state.as_deref() == Some("active"))
            .or_else(|| sprints.last())
    }

    /// Extract team name from customfield_10001 (can be string or object with name).
    pub fn team_name(&self) -> Option<String> {
        let val = self.team.as_ref()?;
        if let Some(s) = val.as_str() {
            return Some(s.to_string());
        }
        val.get("name")
            .and_then(|n| n.as_str())
            .map(|s| s.to_string())
    }
}

// ── Changelog API response types ────────────────────────────────

/// Response from `/rest/api/3/issue/{key}/changelog`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraChangelogResponse {
    #[allow(dead_code)]
    pub start_at: usize,
    #[allow(dead_code)]
    pub max_results: usize,
    #[serde(default)]
    pub is_last: bool,
    #[serde(default)]
    pub values: Vec<JiraChangelogEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraChangelogEntry {
    pub author: Option<JiraUserRef>,
    pub created: DateTime<Utc>,
    #[serde(default)]
    pub items: Vec<JiraChangelogItem>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JiraChangelogItem {
    pub field: String,
    pub from_string: Option<String>,
    pub to_string: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_account_is_not_service() {
        let user = JiraUser {
            account_id: "abc123".to_string(),
            email_address: Some("user@example.com".to_string()),
            display_name: Some("Test User".to_string()),
            active: true,
            account_type: Some("atlassian".to_string()),
        };
        assert!(!user.is_service_account());
    }

    #[test]
    fn app_account_is_service() {
        let user = JiraUser {
            account_id: "app-123".to_string(),
            email_address: None,
            display_name: Some("My App".to_string()),
            active: true,
            account_type: Some("app".to_string()),
        };
        assert!(user.is_service_account());
    }

    #[test]
    fn missing_account_type_is_not_service() {
        let user = JiraUser {
            account_id: "xyz".to_string(),
            email_address: None,
            display_name: None,
            active: true,
            account_type: None,
        };
        assert!(!user.is_service_account());
    }

    #[test]
    fn deserialize_from_json() {
        let json = r#"{
            "accountId": "5b10ac8d82e05b22cc7d4ef5",
            "emailAddress": "mia@example.com",
            "displayName": "Mia Krystof",
            "active": true,
            "accountType": "atlassian"
        }"#;
        let user: JiraUser = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(user.account_id, "5b10ac8d82e05b22cc7d4ef5");
        assert_eq!(user.email_address.as_deref(), Some("mia@example.com"));
        assert!(user.active);
        assert!(!user.is_service_account());
    }

    #[test]
    fn deserialize_minimal() {
        let json = r#"{"accountId": "min", "active": false}"#;
        let user: JiraUser = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(user.account_id, "min");
        assert!(!user.active);
        assert!(user.email_address.is_none());
    }
}
