use chrono::Utc;
use ovia_db::identity::models::{Identity, LinkStatus, Person};
use ovia_matching::{evaluate, MatchingConfig};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug)]
pub struct MatchingResult {
    pub people_created: usize,
    pub links_created: usize,
    pub auto: usize,
    pub conflict: usize,
    pub rejected: usize,
}

pub async fn run_batch_matching(pool: &PgPool, org_id: Uuid) -> anyhow::Result<MatchingResult> {
    let config = MatchingConfig::default();
    let now = Utc::now();

    // 1. Fetch all identities for this org that do NOT have an active link
    let unlinked: Vec<Identity> = sqlx::query_as!(
        IdentityRow,
        r#"
        SELECT i.id, i.org_id, i.source, i.external_id, i.username, i.email,
               i.display_name, i.is_service_account, i.first_seen_at, i.last_seen_at, i.raw_ref
        FROM identities i
        WHERE i.org_id = $1
          AND i.is_service_account = false
          AND NOT EXISTS (
            SELECT 1 FROM person_identity_links pil
            WHERE pil.identity_id = i.id
              AND pil.valid_to IS NULL
              AND pil.status != 'rejected'
          )
        "#,
        org_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| r.into())
    .collect();

    tracing::info!(unlinked = unlinked.len(), "found unlinked identities");

    if unlinked.is_empty() {
        return Ok(MatchingResult {
            people_created: 0,
            links_created: 0,
            auto: 0,
            conflict: 0,
            rejected: 0,
        });
    }

    // 2. Fetch all existing people for this org
    let mut people: Vec<Person> = sqlx::query_as!(
        PersonRow,
        r#"
        SELECT id, org_id, display_name, primary_email, team, role, status,
               created_at, updated_at
        FROM people
        WHERE org_id = $1 AND status = 'active'
        "#,
        org_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| r.into())
    .collect();

    let mut result = MatchingResult {
        people_created: 0,
        links_created: 0,
        auto: 0,
        conflict: 0,
        rejected: 0,
    };

    for identity in &unlinked {
        // Try to find best match among existing people
        let mut best_person_id: Option<Uuid> = None;
        let mut best_match: Option<ovia_matching::MatchResult> = None;

        for person in &people {
            let m = evaluate(&config, person, identity);
            if m.status == LinkStatus::Rejected {
                continue;
            }
            if best_match
                .as_ref()
                .is_none_or(|b| m.confidence > b.confidence)
            {
                best_person_id = Some(person.id);
                best_match = Some(m);
            }
        }

        // If no existing person matched well, create a new person from the identity
        let (person_id, match_result) = if let (Some(pid), Some(mr)) = (best_person_id, best_match)
        {
            (pid, mr)
        } else {
            // Create a new person
            let display_name = identity
                .display_name
                .clone()
                .or_else(|| identity.username.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            let person_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO people (id, org_id, display_name, primary_email, status, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, 'active', $5, $5)",
            )
            .bind(person_id)
            .bind(org_id)
            .bind(&display_name)
            .bind(&identity.email)
            .bind(now)
            .execute(pool)
            .await?;

            result.people_created += 1;

            // Build a Person struct to evaluate
            let new_person = Person {
                id: person_id,
                org_id,
                display_name,
                primary_email: identity.email.clone(),
                team: None,
                role: None,
                status: "active".to_string(),
                created_at: now,
                updated_at: now,
            };

            let m = evaluate(&config, &new_person, identity);
            people.push(new_person);

            (person_id, m)
        };

        // 3. Insert the link
        let link_id = Uuid::new_v4();
        let rule_trace_json = serde_json::to_value(&match_result.rule_trace)?;

        sqlx::query(
            "INSERT INTO person_identity_links
             (id, org_id, person_id, identity_id, status, confidence, rule_trace, valid_from, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8, $8)",
        )
        .bind(link_id)
        .bind(org_id)
        .bind(person_id)
        .bind(identity.id)
        .bind(match_result.status.as_str())
        .bind(match_result.confidence)
        .bind(rule_trace_json)
        .bind(now)
        .execute(pool)
        .await?;

        match match_result.status {
            LinkStatus::Auto => result.auto += 1,
            LinkStatus::Conflict => result.conflict += 1,
            LinkStatus::Rejected => result.rejected += 1,
            _ => {}
        }
        result.links_created += 1;
    }

    Ok(result)
}

// Helper structs for sqlx::query_as! macro

#[allow(dead_code)]
struct IdentityRow {
    id: Uuid,
    org_id: Uuid,
    source: String,
    external_id: Option<String>,
    username: Option<String>,
    email: Option<String>,
    display_name: Option<String>,
    is_service_account: bool,
    first_seen_at: Option<chrono::DateTime<Utc>>,
    last_seen_at: Option<chrono::DateTime<Utc>>,
    raw_ref: Option<serde_json::Value>,
}

impl From<IdentityRow> for Identity {
    fn from(r: IdentityRow) -> Self {
        Identity {
            id: r.id,
            org_id: r.org_id,
            source: r.source,
            external_id: r.external_id,
            username: r.username,
            email: r.email,
            display_name: r.display_name,
            is_service_account: r.is_service_account,
            first_seen_at: r.first_seen_at,
            last_seen_at: r.last_seen_at,
            raw_ref: r.raw_ref,
        }
    }
}

#[allow(dead_code)]
struct PersonRow {
    id: Uuid,
    org_id: Uuid,
    display_name: String,
    primary_email: Option<String>,
    team: Option<String>,
    role: Option<String>,
    status: String,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<PersonRow> for Person {
    fn from(r: PersonRow) -> Self {
        Person {
            id: r.id,
            org_id: r.org_id,
            display_name: r.display_name,
            primary_email: r.primary_email,
            team: r.team,
            role: r.role,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
