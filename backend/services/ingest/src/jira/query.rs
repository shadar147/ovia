use chrono::{DateTime, Utc};

/// Build a JQL query for issue search bounded by project keys and a time window.
///
/// Generates: `project in (KEY1, KEY2) AND updated >= "2026-02-15 00:00" ORDER BY updated ASC`
#[allow(dead_code)] // consumed in Block 2 (issue sync)
pub fn build_issue_search_jql(project_keys: &[String], updated_after: DateTime<Utc>) -> String {
    let projects_clause = project_in_clause(project_keys);
    let updated_clause = format!("updated >= \"{}\"", updated_after.format("%Y-%m-%d %H:%M"));
    format!("{projects_clause} AND {updated_clause} ORDER BY updated ASC")
}

/// Build the `project in (...)` JQL clause from a list of project keys.
fn project_in_clause(keys: &[String]) -> String {
    let escaped: Vec<String> = keys.iter().map(|k| escape_jql_value(k)).collect();
    format!("project in ({})", escaped.join(", "))
}

/// Escape a JQL value — wrap in quotes if it contains special characters.
fn escape_jql_value(value: &str) -> String {
    if value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        value.to_string()
    } else {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn single_project_key() {
        let keys = vec!["DEV".to_string()];
        let after = Utc.with_ymd_and_hms(2026, 2, 15, 0, 0, 0).unwrap();
        let jql = build_issue_search_jql(&keys, after);
        assert_eq!(
            jql,
            "project in (DEV) AND updated >= \"2026-02-15 00:00\" ORDER BY updated ASC"
        );
    }

    #[test]
    fn multiple_project_keys() {
        let keys = vec!["DEV".to_string(), "OPS".to_string(), "INFRA".to_string()];
        let after = Utc.with_ymd_and_hms(2026, 1, 1, 12, 30, 0).unwrap();
        let jql = build_issue_search_jql(&keys, after);
        assert_eq!(
            jql,
            "project in (DEV, OPS, INFRA) AND updated >= \"2026-01-01 12:30\" ORDER BY updated ASC"
        );
    }

    #[test]
    fn bounded_window_from_days_ago() {
        let keys = vec!["TEAM".to_string()];
        let days_ago = 7u32;
        let after = Utc::now() - chrono::Duration::days(i64::from(days_ago));
        let jql = build_issue_search_jql(&keys, after);
        assert!(jql.starts_with("project in (TEAM) AND updated >= \""));
        assert!(jql.ends_with("ORDER BY updated ASC"));
    }

    #[test]
    fn escape_special_chars_in_key() {
        let keys = vec!["MY-PROJECT".to_string()];
        let after = Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap();
        let jql = build_issue_search_jql(&keys, after);
        assert!(jql.contains("\"MY-PROJECT\""), "got: {jql}");
    }

    #[test]
    fn plain_alphanumeric_key_not_quoted() {
        let result = escape_jql_value("DEV");
        assert_eq!(result, "DEV");
    }

    #[test]
    fn key_with_hyphen_is_quoted() {
        let result = escape_jql_value("MY-PROJ");
        assert_eq!(result, "\"MY-PROJ\"");
    }

    #[test]
    fn project_in_clause_formatting() {
        let keys = vec!["A".to_string(), "B".to_string()];
        let clause = project_in_clause(&keys);
        assert_eq!(clause, "project in (A, B)");
    }
}
