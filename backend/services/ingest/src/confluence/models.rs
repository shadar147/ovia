use serde::{Deserialize, Serialize};

/// A user record from the Confluence Cloud REST API
/// (`/wiki/rest/api/group/confluence-users/member`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfluenceUser {
    pub account_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub public_name: Option<String>,
    pub account_type: Option<String>,
}

impl ConfluenceUser {
    /// Returns `true` if the account looks like a service/bot account.
    pub fn is_service_account(&self) -> bool {
        matches!(self.account_type.as_deref(), Some("app"))
    }

    /// Returns `display_name` if present, falling back to `public_name`.
    pub fn effective_display_name(&self) -> Option<&str> {
        self.display_name.as_deref().or(self.public_name.as_deref())
    }
}

/// Paginated response from the Confluence group-member endpoint.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ConfluencePageResponse {
    pub results: Vec<ConfluenceUser>,
    pub start: usize,
    pub limit: usize,
    pub size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_account_is_not_service() {
        let user = ConfluenceUser {
            account_id: "abc123".to_string(),
            email: Some("user@example.com".to_string()),
            display_name: Some("Test User".to_string()),
            public_name: Some("Test User".to_string()),
            account_type: Some("atlassian".to_string()),
        };
        assert!(!user.is_service_account());
    }

    #[test]
    fn app_account_is_service() {
        let user = ConfluenceUser {
            account_id: "app-123".to_string(),
            email: None,
            display_name: Some("My App".to_string()),
            public_name: None,
            account_type: Some("app".to_string()),
        };
        assert!(user.is_service_account());
    }

    #[test]
    fn missing_account_type_is_not_service() {
        let user = ConfluenceUser {
            account_id: "xyz".to_string(),
            email: None,
            display_name: None,
            public_name: None,
            account_type: None,
        };
        assert!(!user.is_service_account());
    }

    #[test]
    fn deserialize_from_json() {
        let json = r#"{
            "accountId": "5b10ac8d82e05b22cc7d4ef5",
            "email": "mia@example.com",
            "displayName": "Mia Krystof",
            "publicName": "Mia K",
            "accountType": "atlassian"
        }"#;
        let user: ConfluenceUser = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(user.account_id, "5b10ac8d82e05b22cc7d4ef5");
        assert_eq!(user.email.as_deref(), Some("mia@example.com"));
        assert!(!user.is_service_account());
    }

    #[test]
    fn deserialize_minimal() {
        let json = r#"{"accountId": "min"}"#;
        let user: ConfluenceUser = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(user.account_id, "min");
        assert!(user.email.is_none());
        assert!(user.display_name.is_none());
    }

    #[test]
    fn effective_display_name_prefers_display_name() {
        let user = ConfluenceUser {
            account_id: "u1".to_string(),
            email: None,
            display_name: Some("Display".to_string()),
            public_name: Some("Public".to_string()),
            account_type: None,
        };
        assert_eq!(user.effective_display_name(), Some("Display"));
    }

    #[test]
    fn effective_display_name_falls_back_to_public_name() {
        let user = ConfluenceUser {
            account_id: "u2".to_string(),
            email: None,
            display_name: None,
            public_name: Some("Public Name".to_string()),
            account_type: None,
        };
        assert_eq!(user.effective_display_name(), Some("Public Name"));
    }

    #[test]
    fn effective_display_name_returns_none_when_both_missing() {
        let user = ConfluenceUser {
            account_id: "u3".to_string(),
            email: None,
            display_name: None,
            public_name: None,
            account_type: None,
        };
        assert_eq!(user.effective_display_name(), None);
    }

    #[test]
    fn deserialize_page_response() {
        let json = r#"{
            "results": [
                {
                    "accountId": "user-1",
                    "email": "user1@example.com",
                    "displayName": "User One",
                    "accountType": "atlassian"
                }
            ],
            "start": 0,
            "limit": 50,
            "size": 1
        }"#;
        let page: ConfluencePageResponse = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(page.results.len(), 1);
        assert_eq!(page.start, 0);
        assert_eq!(page.limit, 50);
        assert_eq!(page.size, 1);
    }
}
