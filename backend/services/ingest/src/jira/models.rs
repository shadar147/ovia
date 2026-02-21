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
