/// Throughput classification: maps Jira issue types and MR labels to
/// bug / feature / chore categories.
///
/// Priority order:
///   1. Jira issue_type mapping (authoritative)
///   2. GitLab label fallback
///   3. Unmatched → chore
///
/// Extend the slices below to cover additional issue types or labels.
/// Jira issue types classified as **bug** (EN + RU).
pub const BUG_ISSUE_TYPES: &[&str] = &["Bug", "Defect", "Баг", "Дефект", "Ошибка"];

/// Jira issue types classified as **feature** (EN + RU).
pub const FEATURE_ISSUE_TYPES: &[&str] = &[
    "Story",
    "Epic",
    "New Feature",
    "Improvement",
    "История",
    "Эпик",
    "Новая функция",
    "Улучшение",
];

/// GitLab MR labels classified as **bug** (case-insensitive match in query).
pub const BUG_LABELS: &[&str] = &["bug", "defect", "fix", "hotfix"];

/// GitLab MR labels classified as **feature** (case-insensitive match in query).
pub const FEATURE_LABELS: &[&str] = &["feature", "enhancement", "story"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bug_issue_types_contains_en() {
        assert!(BUG_ISSUE_TYPES.contains(&"Bug"));
        assert!(BUG_ISSUE_TYPES.contains(&"Defect"));
    }

    #[test]
    fn bug_issue_types_contains_ru() {
        assert!(BUG_ISSUE_TYPES.contains(&"Баг"));
        assert!(BUG_ISSUE_TYPES.contains(&"Дефект"));
        assert!(BUG_ISSUE_TYPES.contains(&"Ошибка"));
    }

    #[test]
    fn feature_issue_types_contains_en() {
        assert!(FEATURE_ISSUE_TYPES.contains(&"Story"));
        assert!(FEATURE_ISSUE_TYPES.contains(&"Epic"));
        assert!(FEATURE_ISSUE_TYPES.contains(&"New Feature"));
        assert!(FEATURE_ISSUE_TYPES.contains(&"Improvement"));
    }

    #[test]
    fn feature_issue_types_contains_ru() {
        assert!(FEATURE_ISSUE_TYPES.contains(&"История"));
        assert!(FEATURE_ISSUE_TYPES.contains(&"Эпик"));
        assert!(FEATURE_ISSUE_TYPES.contains(&"Новая функция"));
        assert!(FEATURE_ISSUE_TYPES.contains(&"Улучшение"));
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

    /// "Задача" is not in bug or feature lists → should fall through to chore.
    #[test]
    fn task_type_is_chore() {
        assert!(!BUG_ISSUE_TYPES.contains(&"Задача"));
        assert!(!FEATURE_ISSUE_TYPES.contains(&"Задача"));
        assert!(!BUG_ISSUE_TYPES.contains(&"Task"));
        assert!(!FEATURE_ISSUE_TYPES.contains(&"Task"));
    }
}
