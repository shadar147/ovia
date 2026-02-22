/// Throughput classification: maps Jira issue types and MR labels to
/// bug / feature / chore categories.
///
/// Priority order:
///   1. Jira issue_type mapping (authoritative)
///   2. GitLab label fallback
///   3. Unmatched → chore
///
/// Extend the slices below to cover additional issue types or labels.

/// Jira issue types classified as **bug**.
pub const BUG_ISSUE_TYPES: &[&str] = &["Bug", "Defect"];

/// Jira issue types classified as **feature**.
pub const FEATURE_ISSUE_TYPES: &[&str] = &["Story", "Epic", "New Feature", "Improvement"];

/// GitLab MR labels classified as **bug** (case-insensitive match in query).
pub const BUG_LABELS: &[&str] = &["bug", "defect", "fix", "hotfix"];

/// GitLab MR labels classified as **feature** (case-insensitive match in query).
pub const FEATURE_LABELS: &[&str] = &["feature", "enhancement", "story"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bug_issue_types_non_empty() {
        assert!(!BUG_ISSUE_TYPES.is_empty());
        assert!(BUG_ISSUE_TYPES.contains(&"Bug"));
    }

    #[test]
    fn feature_issue_types_non_empty() {
        assert!(!FEATURE_ISSUE_TYPES.is_empty());
        assert!(FEATURE_ISSUE_TYPES.contains(&"Story"));
    }

    #[test]
    fn bug_labels_non_empty() {
        assert!(!BUG_LABELS.is_empty());
        assert!(BUG_LABELS.contains(&"bug"));
    }

    #[test]
    fn feature_labels_non_empty() {
        assert!(!FEATURE_LABELS.is_empty());
        assert!(FEATURE_LABELS.contains(&"feature"));
    }

    #[test]
    fn no_overlap_between_bug_and_feature_issue_types() {
        for t in BUG_ISSUE_TYPES {
            assert!(
                !FEATURE_ISSUE_TYPES.contains(t),
                "{t} appears in both bug and feature issue types"
            );
        }
    }

    #[test]
    fn no_overlap_between_bug_and_feature_labels() {
        for l in BUG_LABELS {
            assert!(
                !FEATURE_LABELS.contains(l),
                "{l} appears in both bug and feature labels"
            );
        }
    }
}
