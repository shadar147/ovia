use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A user record from the GitLab REST API (`GET /api/v4/users`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabUser {
    pub id: u64,
    pub username: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub state: Option<String>,
    #[serde(default)]
    pub bot: Option<bool>,
}

impl GitLabUser {
    /// Returns `true` if the account is a bot/service account.
    pub fn is_service_account(&self) -> bool {
        self.bot == Some(true)
    }
}

/// A project record from the GitLab REST API (`GET /api/v4/projects`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabProject {
    pub id: u64,
    pub name: String,
    pub path_with_namespace: String,
    pub web_url: String,
}

/// Author embedded inside a merge request response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabMrAuthor {
    pub username: String,
}

/// A merge request record from the GitLab REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabMergeRequest {
    pub iid: u64,
    pub title: String,
    pub state: String,
    pub author: Option<GitLabMrAuthor>,
    #[serde(default)]
    pub labels: Vec<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
    pub web_url: String,
}

/// A pipeline record from the GitLab REST API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabPipeline {
    pub id: u64,
    pub status: String,
    #[serde(rename = "ref")]
    pub ref_name: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub duration: Option<i32>,
    pub web_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_user_is_not_service() {
        let user = GitLabUser {
            id: 1,
            username: "jdoe".to_string(),
            email: Some("jdoe@example.com".to_string()),
            name: Some("Jane Doe".to_string()),
            state: Some("active".to_string()),
            bot: Some(false),
        };
        assert!(!user.is_service_account());
    }

    #[test]
    fn bot_user_is_service() {
        let user = GitLabUser {
            id: 2,
            username: "project_bot".to_string(),
            email: None,
            name: Some("Project Bot".to_string()),
            state: Some("active".to_string()),
            bot: Some(true),
        };
        assert!(user.is_service_account());
    }

    #[test]
    fn missing_bot_field_is_not_service() {
        let user = GitLabUser {
            id: 3,
            username: "unknown".to_string(),
            email: None,
            name: None,
            state: None,
            bot: None,
        };
        assert!(!user.is_service_account());
    }

    #[test]
    fn deserialize_from_json() {
        let json = r#"{
            "id": 42,
            "username": "mia_k",
            "email": "mia@example.com",
            "name": "Mia Krystof",
            "state": "active",
            "bot": false
        }"#;
        let user: GitLabUser = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(user.id, 42);
        assert_eq!(user.username, "mia_k");
        assert_eq!(user.email.as_deref(), Some("mia@example.com"));
        assert_eq!(user.name.as_deref(), Some("Mia Krystof"));
        assert_eq!(user.state.as_deref(), Some("active"));
        assert!(!user.is_service_account());
    }

    #[test]
    fn deserialize_minimal() {
        let json = r#"{"id": 99, "username": "min"}"#;
        let user: GitLabUser = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(user.id, 99);
        assert_eq!(user.username, "min");
        assert!(user.email.is_none());
        assert!(user.name.is_none());
        assert!(user.state.is_none());
        assert!(user.bot.is_none());
        assert!(!user.is_service_account());
    }
}
