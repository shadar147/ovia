use ovia_db::identity::models::PersonIdentityLink;

pub fn format_conflicts_csv(links: &[PersonIdentityLink]) -> String {
    let mut out = String::from("id,person_id,identity_id,status,confidence,created_at\n");
    for link in links {
        out.push_str(&format!(
            "{},{},{},{},{},{}\n",
            link.id,
            link.person_id,
            link.identity_id,
            link.status.as_str(),
            link.confidence,
            link.created_at.to_rfc3339(),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use ovia_db::identity::models::LinkStatus;
    use uuid::Uuid;

    #[test]
    fn csv_format_produces_header_and_rows() {
        let link = PersonIdentityLink {
            id: Uuid::nil(),
            org_id: Uuid::nil(),
            person_id: Uuid::nil(),
            identity_id: Uuid::nil(),
            status: LinkStatus::Conflict,
            confidence: 0.75,
            valid_from: None,
            valid_to: None,
            verified_by: None,
            verified_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let csv = format_conflicts_csv(&[link]);
        let lines: Vec<&str> = csv.lines().collect();

        assert_eq!(
            lines[0],
            "id,person_id,identity_id,status,confidence,created_at"
        );
        assert!(lines[1].starts_with("00000000-0000-0000-0000-000000000000"));
        assert!(lines[1].contains("conflict"));
        assert!(lines[1].contains("0.75"));
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn csv_format_empty_produces_only_header() {
        let csv = format_conflicts_csv(&[]);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0],
            "id,person_id,identity_id,status,confidence,created_at"
        );
    }
}
