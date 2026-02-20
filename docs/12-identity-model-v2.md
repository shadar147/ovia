# Ovia — Identity Model v2 (People + Historical Accounts)

## Why v2
Real-world teams often use multiple identities per system over time (different laptops, emails, usernames, service accounts). A strict `1 person -> 1 account per source` model loses data and causes misattribution.

## Core design

### 1) `people` (canonical human profile)
Single internal person record.

Fields:
- `id` (uuid)
- `org_id`
- `display_name`
- `primary_email`
- `team`
- `role`
- `status` (`active|inactive`)
- `created_at`, `updated_at`

### 2) `identities` (all discovered accounts/aliases)
Every observed identity from connected systems and git history.

Fields:
- `id` (uuid)
- `org_id`
- `source` (`gitlab|jira|confluence|git_commit_author|git_commit_committer|git_config_snapshot|service_account`)
- `external_id` (nullable; source-native id)
- `username` (nullable)
- `email` (nullable)
- `display_name` (nullable)
- `is_service_account` (bool)
- `first_seen_at`, `last_seen_at`
- `raw_ref` (jsonb)

### 3) `person_identity_links` (many-to-many link table)
Links a canonical person to one or more identities.

Fields:
- `id` (uuid)
- `org_id`
- `person_id` (fk -> people)
- `identity_id` (fk -> identities)
- `status` (`auto|verified|conflict|rejected|ignored`)
- `confidence` (0..1)
- `rule_trace` (jsonb)
- `valid_from`, `valid_to` (temporal history)
- `verified_by`, `verified_at`
- `created_at`, `updated_at`

### 4) `identity_events` (audit trail)
Who changed mapping and why.

Fields:
- `id` (uuid)
- `org_id`
- `link_id`
- `action` (`auto_match|confirm|remap|split|reject|ignore`)
- `actor`
- `payload` (jsonb)
- `created_at`

## Matching strategy
1. Exact email match
2. Username similarity
3. Display-name similarity
4. Historical co-occurrence in same project/team context
5. Service-account heuristics exclusion

Output: suggested links with confidence score.

## Operational policy
- Analytics include: `verified` + `auto(confidence>=threshold)`
- `conflict` and low-confidence links are excluded from management KPIs by default
- All link changes are auditable and reversible

## UI implications
Team Identity screen should show:
- canonical person
- linked identities count
- aliases preview (top 3)
- conflict indicator
- last verification date

## Example
Person "Ivan Malinov" can be linked to:
- `gitlab: imalinov`
- `git_commit_author: Ivan M <ivan@oldmail.com>`
- `git_commit_author: Ivan Malinov <ivan@corp.com>`
- `jira: ivan.m`
- `confluence: ivan.m`

All of these remain separate identities, but map to one canonical person.
