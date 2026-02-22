# Ovia — Delivery Backlog (Claude Execution Plan)

Status legend: `todo | in_progress | review | done | blocked`

## Epic 0 — Project Foundation

### OVIA-0001 Rust workspace scaffold
- Status: `done`
- Priority: P0
- Owner: Claude
- Description:
  - Create Rust workspace structure for services: `api`, `ingest`, `metrics`, `rag`, `scheduler`.
  - Add shared crates for config, db, common types.
- Acceptance:
  - `cargo check` passes for full workspace.
  - Base README for workspace + local run notes.
  - Logging/tracing bootstrapped.
- Test requirements:
  - Smoke tests for service startup config parsing.

### OVIA-0002 CI quality gates
- Status: `done`
- Priority: P0
- Owner: Claude
- Description:
  - Add GitHub Actions for `fmt`, `clippy -D warnings`, `test`.
  - Add coverage report artifact.
- Acceptance:
  - CI runs on PR and main.
  - Fails on lint/test failures.

## Epic 1 — Identity Model v2 (core)

### OVIA-1001 SQL migration baseline
- Status: `done`
- Priority: P0
- Depends on: OVIA-0001
- Description:
  - Validate and refine `backend/db/migrations/0001_identity_v2.sql`.
  - Add missing indexes for listing/filtering identity conflicts.
- Acceptance:
  - Migration applies on clean DB and existing DB.
  - Index plan documented for key queries.

### OVIA-1002 Identity repository layer
- Status: `done`
- Priority: P0
- Depends on: OVIA-0001, OVIA-1001
- Description:
  - Implement repository interfaces for people/identities/links/events.
- Acceptance:
  - CRUD + list/filter methods.
  - Transaction-safe remap/split operations.
- Tests:
  - Integration tests with test DB.

### OVIA-1003 Identity API v1
- Status: `done`
- Priority: P0
- Depends on: OVIA-1002
- Description:
  - Endpoints:
    - `GET /team/identity-mappings`
    - `POST /team/identity-mappings/confirm`
    - `POST /team/identity-mappings/remap`
    - `POST /team/identity-mappings/split`
- Acceptance:
  - Request/response contracts match docs.
  - Validation + typed errors + audit events.
- Tests:
  - Handler tests for happy path + edge cases.

## Epic 2 — Matching Engine

### OVIA-2001 Matching rules v1
- Status: `done`
- Priority: P0
- Depends on: OVIA-1002
- Description:
  - Implement scoring:
    - exact email
    - username similarity
    - display name similarity
    - project/team co-occurrence
    - service-account exclusions
- Acceptance:
  - Score + rule trace returned for each suggestion.
  - Configurable thresholds: auto / conflict / reject.
- Tests:
  - Deterministic fixture tests across 15+ scenarios.

### OVIA-2002 Conflict queue workflow
- Status: `done`
- Priority: P1
- Depends on: OVIA-2001, OVIA-1003
- Description:
  - Add status transitions and queue filters for unresolved conflicts.
- Acceptance:
  - `conflict` rows visible in API with sort/filter.
  - Bulk confirm by threshold supported.

## Epic 3 — Connectors (MVP)

### OVIA-3001 Jira incremental sync
- Status: `done`
- Priority: P1
- Owner: Claude
- Description:
  - Jira Cloud user sync connector with paginated fetch, retry/backoff, idempotent upsert.
  - Sync watermark table for lock-based concurrency control.
  - `raw_ref` field added to Identity model for raw payload persistence.
  - `upsert_by_external_id` on `IdentityRepository` for conflict-free inserts.
- Acceptance:
  - watermark-based sync, idempotent upsert, retry/backoff.
  - raw payload persistence.
  - integration tests with mocked paginated API (wiremock).
- Tests:
  - 17 unit/integration tests: Jira models, client pagination, retry on 5xx, fail-fast on 4xx, sync orchestration, service account detection, raw_ref persistence.
  - 5 sync watermark repository tests: get_or_create, acquire_lock, concurrent lock rejection, mark_completed, mark_failed.

### OVIA-3005 Jira Issue Sync — Block 1: Config + JQL Query Builder
- Status: `done`
- Priority: P1
- Owner: Claude
- Depends on: OVIA-3001
- Description:
  - `JIRA_PROJECT_KEYS` (CSV) env var added to `JiraClientConfig` with strict fail-fast validation.
  - `JIRA_SYNC_WINDOW_DAYS` env var (default 7) for bounded JQL window.
  - `JiraClientConfig::from_env()` returns `Result<Option<Self>, String>` — distinguishes "not configured" from "misconfigured".
  - `parse_csv_project_keys()` helper: trims whitespace, uppercases, rejects empty.
  - JQL query builder (`jira/query.rs`): `build_issue_search_jql()` generates `project in (...) AND updated >= "..." ORDER BY updated ASC`.
  - JQL escaping for keys with special characters.
  - `main.rs` updated to handle `Result<Option<_>>` with fail-fast panic on misconfiguration.
  - `.env.example` updated with `JIRA_PROJECT_KEYS` and `JIRA_SYNC_WINDOW_DAYS`.
- Acceptance:
  - All 73 ingest tests pass (15 new).
  - `cargo fmt --check` and `cargo clippy -D warnings` clean.
- Tests:
  - 8 config tests: CSV parse valid/single/empty/missing/whitespace, from_env none/fail-fast/success.
  - 7 JQL tests: single/multi project, bounded window, escape special chars, plain key, hyphen key, clause formatting.

### OVIA-3006 Jira Issue Sync — Block 2: Ingest Issues, Changelog, Analytics Fields
- Status: `done`
- Priority: P1
- Owner: Claude
- Depends on: OVIA-3005
- Description:
  - Migration 0006: `jira_issues` + `jira_issue_transitions` tables.
  - DB layer: `PgJiraRepository` with `upsert_issue`, `insert_transition`, `delete_transitions`.
  - API models: `JiraSearchResponse`, `JiraIssue`, `JiraChangelogResponse` with sprint/team/story_points.
  - Client: `search_issues` (paginated `/rest/api/3/search`), `fetch_issue_changelog` (paginated).
  - Generic `request_with_retry<T: DeserializeOwned>` replacing typed version.
  - `JiraIssueSyncer`: watermark-locked sync with JQL bounded by project keys + time window.
  - Changelog: extracts status + sprint transitions, replace strategy (delete old + insert new).
  - Analytics fields: `customfield_10016` (story_points), `customfield_10020` (sprint), `customfield_10001` (team).
- Acceptance:
  - All 141 tests pass (20 new: 17 issue_sync + 3 DB).
  - `cargo fmt --check` and `cargo clippy -D warnings` clean (after fmt fix commit).
- Tests:
  - 17 issue_sync tests: pagination, changelog extraction, sprint/team/story_points parsing, watermark lock, wiremock integration.
  - 3 DB tests: upsert_issue, insert_transition, delete_transitions.

### OVIA-3007 Jira Issue Sync — Block 3A: Compute Jira Metrics in KPI Pipeline
- Status: `done`
- Priority: P1
- Owner: Claude
- Depends on: OVIA-3006
- Description:
  - Migration 0007: add `blocker_count`, `spillover_rate`, `cycle_time_p50_hours`, `cycle_time_p90_hours` to `kpi_snapshots`.
  - `PgJiraRepository`: 6 new query methods (`count_open_blockers`, `list_open_blocker_age_days`, `spillover_rate`, `get_cycle_times_hours`, `count_resolved_issues`, `count_resolved_issues_by_type`).
  - `KpiService`: wire Jira metrics into `compute_and_save`, combine MR + Jira throughput, feed real blocker data into risk score.
  - `KpiSnapshot` model: 4 new optional fields (additive, non-breaking).
- Acceptance:
  - 8 new integration tests for Jira metrics queries.
  - All tests pass, clippy clean, fmt clean.
- Tests:
  - 8 Jira metric query tests in `ovia-db`.

### OVIA-3008 Jira Issue Sync — Block 3B: Integrate Jira Metrics into /team/kpi API
- Status: `done`
- Priority: P1
- Owner: Claude
- Depends on: OVIA-3007
- Description:
  - Wire `blocker_count`, `spillover_rate`, `cycle_time_p50_hours`, `cycle_time_p90_hours` into KPI API response contract (additive, non-breaking).
  - All three endpoints (`/team/kpi`, `/team/kpi/history`, `/team/kpi/risks`) now return Jira-derived fields.
  - Updated test fixtures to include Jira metric values.
- Acceptance:
  - 203 total tests pass across all crates (59 db + 90 ingest + 16 metrics + 34 api + 4 rag).
  - `cargo fmt --check` and `cargo clippy -D warnings` clean.
- Tests:
  - `kpi_response_contract_includes_all_fields` contract test.
  - `kpi_history_includes_jira_metrics` test.
  - Updated assertions in `kpi_latest` test.

### OVIA-3002 GitLab incremental sync
- Status: `done`
- Priority: P1
- Owner: Claude
- Description:
  - GitLab Cloud user sync connector with paginated fetch via `PRIVATE-TOKEN` auth.
  - Pagination via `x-next-page` response header.
  - Retry/backoff on 429 and 5xx, fail-fast on 4xx.
  - Idempotent upsert via `upsert_by_external_id`.
  - Bot detection via `bot` field on GitLab user records.
  - `raw_ref` field populated with full GitLab user payload.
- Acceptance:
  - watermark-based sync, idempotent upsert, retry/backoff.
  - raw payload persistence.
  - integration tests with mocked paginated API (wiremock).
- Tests:
  - 5 model tests: human user, bot user, missing fields, JSON deserialization, minimal deserialization.
  - 7 client tests: single page, multi-page pagination, retry on 500, fail-fast on 401, empty response, PRIVATE-TOKEN header, max retries exceeded.
  - 4 sync tests: upsert all users, skip when lock unavailable, bot service account detection, raw_ref persistence.

### OVIA-3003 Confluence incremental sync
- Status: `done`
- Priority: P1
- Owner: Claude
- Description:
  - Confluence Cloud user sync connector using group-member API with paginated fetch, retry/backoff, idempotent upsert.
  - Basic auth (shared Atlassian identity), `accountType` detection for service accounts.
  - `effective_display_name()` fallback from `displayName` to `publicName`.
  - `raw_ref` persistence for raw payload.
- Acceptance:
  - watermark-based sync, idempotent upsert, retry/backoff.
  - raw payload persistence.
  - integration tests with mocked paginated API (wiremock).
- Tests:
  - 9 model tests: human user, app user, missing fields, JSON deserialization, minimal deserialization, effective_display_name preference, fallback, none case, page response deserialization.
  - 7 client tests: single page, multi-page pagination, retry on 500, fail-fast on 401, empty response, basic auth header, max retries exceeded.
  - 4 sync tests: upsert all users, skip when lock unavailable, app service account detection, raw_ref persistence.

### OVIA-3004 GitLab MR/Pipeline sync + real KPI computation
- Status: `done`
- Priority: P1
- Owner: Claude
- Depends on: OVIA-3002, OVIA-4001
- Description:
  - New DB tables: `gitlab_projects`, `gitlab_merge_requests`, `gitlab_pipelines` (migration 0005).
  - DB repository layer in `ovia-db` gitlab module: upsert helpers + KPI query helpers (count merged MRs, count by label, review durations, pipeline counts, stale MR percentage, stale MR listing, failed pipeline listing).
  - Extended GitLab client with generic paginated fetch and 4 new methods: `fetch_all_projects`, `fetch_merged_mrs`, `fetch_open_mrs`, `fetch_pipelines`.
  - New API response structs: `GitLabProject`, `GitLabMergeRequest`, `GitLabMrAuthor`, `GitLabPipeline`.
  - MR/Pipeline syncer (`GitLabMrPipelineSyncer`) with separate watermark lock (`gitlab_mr_pipeline`), incremental sync via `cursor_value` as `updated_after`.
  - KPI service rewritten to compute real metrics from GitLab data: throughput from merged MRs, review latency from actual durations, risk from failing pipelines and stale MRs.
  - Risk item generation from stale open MRs (>7 days) and failed pipelines.
- Acceptance:
  - All 122 tests pass across ovia-db (48), ovia-ingest (58), ovia-metrics (16).
  - Clean build with no warnings.
  - New tests: 3 DB repo tests, 4 client tests, 4 syncer tests, 4 percentile unit tests.
- Tests:
  - DB: upsert_project, count_merged_mrs, count_merged_mrs_by_label.
  - Client: fetch_projects pagination, fetch_merged_mrs, fetch_pipelines, API model deserialization.
  - Syncer: lock-skip, model deserialization.
  - KPI: percentile median, p90, empty, single value.

### OVIA-3009 Jira Dashboard KPI Exposure + Risk Pagination (Block A)
- Status: `done`
- Priority: P1
- Owner: Claude
- Depends on: OVIA-3008
- Description:
  - Expose `blocker_count`, `spillover_rate`, `cycle_time_p50_hours`, `cycle_time_p90_hours` in frontend `KpiSnapshot` type.
  - Add 3 new KPI cards to dashboard: Blockers, Spillover Rate, Cycle Time.
  - Add client-side pagination (20/page) to RiskTable with prev/next navigation.
  - Update i18n messages (en + ru) for new cards and pagination controls.
  - Update all chart test fixtures with new Jira KPI fields.
- Acceptance:
  - New KPI cards render with correct data.
  - Risk table paginates at 20 rows with boundary handling.
  - 3 new pagination tests + updated fixture tests pass.

### OVIA-3010 Extensible Throughput Classification (Block B)
- Status: `done`
- Priority: P1
- Owner: Claude
- Depends on: OVIA-3008
- Description:
  - New `classify.rs` module with configurable bug/feature/chore mapping strategy.
  - Jira issue type mapping: Bug/Defect → bug; Story/Epic/New Feature/Improvement → feature.
  - GitLab label fallback: bug/defect/fix/hotfix → bug; feature/enhancement/story → feature.
  - Unmatched → chore.
  - `count_resolved_issues_by_types` (multi-type) added to `PgJiraRepository`.
  - `count_merged_mrs_by_labels` (multi-label) added to `PgGitlabRepository`.
  - KPI service updated to use expanded mappings for throughput breakdown.
- Acceptance:
  - Classification invariant tests pass (no overlap between bug/feature sets, non-empty sets).
  - All existing tests pass.

### OVIA-3011 Jira Identities Ingest from Issues (Block C)
- Status: `done`
- Priority: P1
- Owner: Claude
- Depends on: OVIA-3006
- Description:
  - Enrich `JiraIssueSyncer` to collect unique user refs (assignee + reporter) from synced issues.
  - Upsert collected users as identities (`source=jira`).
  - Dedup by `accountId`, null-safe for missing fields, marks app accounts as service accounts.
  - Passes `IdentityRepository` + `OrgId` into issue syncer for identity upsert.
- Acceptance:
  - 5 new unit tests, 95 total ingest tests pass.
  - `cargo fmt --check` and `cargo clippy -D warnings` clean.
- Tests:
  - `extracts_assignee_and_reporter_identities`
  - `deduplicates_identities_by_account_id`
  - `skips_null_assignee_reporter`
  - `marks_app_accounts_as_service_accounts`
  - `handles_missing_display_name`

### OVIA-CI-001 Clippy doc-comment lint fix
- Status: `done`
- Priority: P0
- Owner: Claude
- Description:
  - Fix `empty-line-after-doc-comments` clippy lint in `classify.rs` (commit `d5ab246`).
- Acceptance:
  - `cargo clippy -D warnings` clean.

## Epic 4 — Analytics + Ask Ovia

### OVIA-4001 KPI query service
- Status: `done`
- Priority: P1
- Owner: Claude
- Depends on: Epic 3
- Description:
  - KPI snapshots table + risk items table (migrations 0003).
  - DB repository layer: save, get_latest, list, upsert-on-conflict for snapshots; save/list for risk items.
  - Pure KPI computation functions: `compute_delivery_health` (weighted 0-100) and `compute_release_risk` (label + score).
  - KPI service in metrics: one-shot compute-and-save from identity/link stats.
  - API endpoints: `GET /team/kpi`, `GET /team/kpi/history`, `GET /team/kpi/risks`.
- Acceptance:
  - 7 KPI repo integration tests, 11 compute unit tests, 1 service mock test, 4 API handler tests.
  - All passing, clippy clean, fmt clean.

### OVIA-4002 Ask API contract with citations
- Status: `done`
- Priority: P1
- Owner: Claude
- Depends on: OVIA-4001
- Description:
  - Ask sessions table (migration 0004).
  - DB repository layer: save, get, list sessions with citations as JSONB.
  - Stub Ask engine in RAG service: looks up KPI data, formats structured answer with citations.
  - RAG service as axum server: `POST /ask`, `GET /ask/:id`, `GET /ask/history`.
  - API gateway endpoints: `POST /ask`, `GET /ask/:id`, `GET /ask/history` with local stub engine.
  - All responses include confidence level, assumptions, and citations pointing to real data.
- Acceptance:
  - 6 Ask repo integration tests, 4 Ask engine unit tests, 4 API handler tests.
  - All passing, clippy clean, fmt clean.

## Epic 5 — Deployment & Ops

### OVIA-5001 Swarm stack manifests
- Status: `done`
- Priority: P1
- Owner: Claude
- Description:
  - Multi-stage Dockerfile for all Rust services (single image, 5 binaries).
  - Docker Swarm compose with postgres, redis, caddy, migrate init, all 5 services.
  - Caddyfile for reverse proxy with auto-TLS.
  - DB migration init script.
  - `.env.example` for production secrets.
  - `.dockerignore` for build context.
- Acceptance:
  - `docker compose -f backend/infra/docker-compose.swarm.yml config` validates.
  - `docker build -t ovia-backend backend/` succeeds.
  - All 5 binaries present in built image.

### OVIA-5002 Monitoring baseline
- Status: `done`
- Priority: P1
- Owner: Claude
- Description:
  - Prometheus config with scrape targets for ovia-api, ovia-metrics, ovia-ingest, postgres-exporter, redis-exporter, node-exporter.
  - Alert rules: HighSyncFailureRate, ApiHighLatency, DatabaseConnectionPoolExhausted, HighErrorRate, DiskSpaceRunningLow, PostgresReplicationLag.
  - Grafana auto-provisioning: Prometheus + Loki datasources, Ovia Overview dashboard with uptime, request rate, error rate, latency percentiles, DB pool, sync status, system resources.
  - Loki local config with TSDB store and 7-day retention.
  - Promtail config for Docker container log collection via socket.
  - Docker Compose services: prometheus, grafana, loki, promtail, node-exporter with proper deploy constraints and volumes.
  - Caddyfile reverse proxy for Grafana at /grafana/*.
  - Stub /metrics endpoint on ovia-api returning Prometheus text format.
  - .env.example updated with GRAFANA_ADMIN_USER and GRAFANA_ADMIN_PASSWORD.
- Acceptance:
  - `docker compose -f backend/infra/docker-compose.swarm.yml config` validates.
  - All YAML/JSON configs are syntactically valid.
  - /metrics endpoint returns Prometheus text exposition format.
- Tests:
  - Handler test for /metrics endpoint verifying content-type and body content.

### OVIA-5003 Backup/restore runbook
- Status: `done`
- Priority: P1
- Owner: Claude
- Description:
  - Backup script (`backup.sh`): daily + weekly pg_dump with configurable retention.
  - Restore script (`restore.sh`): pg_restore with optional drop/recreate and row-count verification.
  - Verification script (`verify-backup.sh`): non-destructive archive validation.
  - Docker Swarm backup service running on 24-hour loop.
  - Comprehensive runbook covering automated backups, manual procedures, restore scenarios, monitoring, testing drills, and troubleshooting.
- Acceptance:
  - All scripts pass `bash -n` syntax check.
  - `docker compose -f backend/infra/docker-compose.swarm.yml config` validates.
  - Runbook covers same-host restore, new-host restore, and PITR (future).

## Epic 6 — People Management & Multi-Identity Mapping

### OVIA-6001 People CRUD API (Backend)
- Status: `done`
- Priority: P0
- Owner: Claude
- Depends on: OVIA-1002
- Description:
  - Extend `PersonRepository` trait with `list(org_id, filters)`, `soft_delete(org_id, id)`.
  - Implement `PgPersonRepository` for all CRUD methods (create, get_by_id, update, list, soft_delete).
  - List endpoint supports filters: `team`, `status`, `search` (display_name/email substring), pagination (`limit`/`offset`).
  - Soft delete sets `status = 'inactive'`, does not remove row.
  - API endpoints:
    - `GET /team/people` — paginated list with filters.
    - `GET /team/people/:id` — single person with linked identity count.
    - `POST /team/people` — create manual person.
    - `PUT /team/people/:id` — update person fields.
    - `DELETE /team/people/:id` — soft delete (set inactive).
  - Response contract: `PersonResponse { id, display_name, primary_email, team, role, status, identity_count, created_at, updated_at }`.
- Acceptance:
  - All 5 endpoints return correct status codes and payloads.
  - Soft delete preserves row, filters out inactive by default.
  - Pagination works with `limit`/`offset` + total count header.
  - Validation: display_name required, email format check.
- Tests:
  - DB: list with filters, soft delete, pagination boundary.
  - API: 5 handler tests (list, get, create, update, delete) + 3 error cases (404, duplicate email, validation).

### OVIA-6002 Manual Identity Linking API (Backend)
- Status: `todo`
- Priority: P0
- Owner: Claude
- Depends on: OVIA-6001, OVIA-1003
- Description:
  - API endpoints for manual identity-to-person linking:
    - `POST /team/people/:id/identities` — link existing identity to person (creates `person_identity_link` with `status=verified`, `confidence=1.0`, `verified_by=manual`).
    - `DELETE /team/people/:id/identities/:identity_id` — unlink identity from person (sets `valid_to=now()`, emits audit event).
    - `GET /team/people/:id/identities` — list all linked identities for a person (with source, username, email, status, linked_at).
  - Emit `identity_event` for every link/unlink action (`action=manual_link` / `action=manual_unlink`).
  - Validate: identity must exist, must belong to same org, cannot link already-linked identity without explicit remap.
  - Support linking unmatched identities (those without any person_identity_link).
- Acceptance:
  - Link/unlink operations are transactional with audit trail.
  - Cannot link identity to two persons simultaneously (must remap).
  - Unlinked identities appear in "orphan" pool for re-linking.
- Tests:
  - 4 handler tests: link, unlink, list identities, link-already-linked error.
  - 2 integration tests: audit event emission, concurrent link conflict.

### OVIA-6003 People List Page (Frontend)
- Status: `todo`
- Priority: P0
- Owner: Claude
- Depends on: OVIA-6001
- Description:
  - New route: `/team/people` with page component.
  - Data table with columns: Name, Email, Team, Role, Status, Identities (count badge), Actions.
  - Search bar (debounced, searches display_name + email).
  - Filter chips: team (dropdown from distinct values), status (active/inactive/all).
  - Pagination controls (reuse risk-table pattern).
  - Row click navigates to Person 360 (`/team/people/:id`).
  - "Add Person" button opens create dialog.
  - Add navigation link in sidebar.
  - i18n: en + ru messages for all labels and actions.
- Acceptance:
  - Table renders with real API data.
  - Search debounce ≤300ms, filters update URL params.
  - Empty state shown when no results.
  - Responsive layout (table scrolls on mobile).
- Tests:
  - Render test with mock data.
  - Search filter test.
  - Pagination test.

### OVIA-6004 Person Create/Edit Dialog (Frontend)
- Status: `todo`
- Priority: P1
- Owner: Claude
- Depends on: OVIA-6003
- Description:
  - Modal dialog for creating/editing a person.
  - Fields: display_name (required), primary_email, team (dropdown/free-text), role (dropdown/free-text), status.
  - Validation: display_name non-empty, email format if provided.
  - On save: `POST /team/people` (create) or `PUT /team/people/:id` (edit).
  - Success toast + table refresh.
  - Delete confirmation dialog → `DELETE /team/people/:id`.
  - i18n: en + ru for form labels, validation messages, confirmation text.
- Acceptance:
  - Form validates before submit.
  - Create and edit flows work end-to-end.
  - Delete shows confirmation, updates table on success.
- Tests:
  - Form validation test (empty name, invalid email).
  - Create submit test with mock API.
  - Edit pre-fill test.

### OVIA-6005 Multi-Identity Mapping UI (Frontend)
- Status: `todo`
- Priority: P1
- Owner: Claude
- Depends on: OVIA-6002, OVIA-6004
- Description:
  - Identity linking panel within Person 360 or as standalone section.
  - Shows all linked identities for a person with: source icon, username, email, status badge, linked date.
  - "Link Identity" button opens search dialog:
    - Search orphan identities by username/email/source.
    - Select → calls `POST /team/people/:id/identities`.
  - "Unlink" action per identity row → calls `DELETE /team/people/:id/identities/:identity_id` with confirmation.
  - Visual indicator for identity sources (GitLab, Jira, Confluence, Git).
  - i18n: en + ru.
- Acceptance:
  - Link/unlink reflected immediately in UI (optimistic update + refetch).
  - Search filters work for orphan identities.
  - Source icons render correctly for all known sources.
- Tests:
  - Link identity flow test.
  - Unlink with confirmation test.
  - Orphan search rendering test.

## Epic 7 — Person 360 Profile & Activity Timeline

### OVIA-7001 Person 360 Backend API
- Status: `todo`
- Priority: P1
- Owner: Claude
- Depends on: OVIA-6001, OVIA-6002
- Description:
  - `GET /team/people/:id/profile` — full profile with:
    - Person fields (display_name, email, team, role, status).
    - All linked identities with source, username, email, status, confidence.
    - Summary stats: total_mrs, total_issues, active_days_30d.
  - `GET /team/people/:id/activity` — unified activity timeline:
    - Sources: `gitlab_merge_requests` (via person→identity→gitlab author), `jira_issues` (via person→identity→jira assignee/reporter), `identity_events` (mapping changes).
    - Query params:
      - `period`: `7d|30d|90d|custom` (with `from`/`to` for custom).
      - `source`: `gitlab|jira|confluence|all` (comma-separated multi-select).
      - `type`: `merge_request|issue|identity_event|all`.
      - `limit`/`offset` for pagination.
    - Response: `ActivityItem { id, source, type, title, url, timestamp, metadata }`.
  - Activity query joins person→identity links→source tables via identity external_id matching.
  - New DB queries in `PgGitlabRepository` and `PgJiraRepository`: `list_activity_by_identity_ids(ids, filters)`.
- Acceptance:
  - Profile returns all linked identities with stats.
  - Activity timeline returns unified, chronologically sorted items.
  - Filters work correctly (period, source, type).
  - Pagination works with total count.
- Tests:
  - DB: 4 activity query tests (gitlab MRs by identity, jira issues by identity, combined, empty).
  - API: 3 handler tests (profile, activity with filters, activity pagination).

### OVIA-7002 Person 360 Page (Frontend)
- Status: `todo`
- Priority: P1
- Owner: Claude
- Depends on: OVIA-7001, OVIA-6005
- Description:
  - New route: `/team/people/:id` — Person 360 page.
  - Layout sections:
    1. **Header**: display_name, email, team, role, status badge, edit button.
    2. **Identities panel**: linked identities list with source icons (reuse OVIA-6005 component).
    3. **Stats row**: total MRs, total issues, active days (30d) as compact cards.
    4. **Activity timeline**: chronological feed with source icon, type badge, title, timestamp.
  - Activity timeline infinite scroll or "Load more" button.
  - Breadcrumb navigation: People → Person Name.
  - i18n: en + ru for all labels, empty states.
- Acceptance:
  - Page loads with profile data and activity feed.
  - Identities panel shows all linked sources.
  - Stats reflect real computed values.
  - Breadcrumb navigation works.
- Tests:
  - Profile render test with mock data.
  - Activity timeline render test.
  - Empty state test (no activity).

### OVIA-7003 Activity Timeline Filters (Frontend)
- Status: `todo`
- Priority: P2
- Owner: Claude
- Depends on: OVIA-7002
- Description:
  - Filter bar above activity timeline:
    - Period selector: 7d / 30d / 90d / Custom date range picker.
    - Source multi-select: GitLab, Jira, Confluence (with checkboxes).
    - Type multi-select: Merge Requests, Issues, Identity Events.
  - Filters persist in URL query params for shareability.
  - Debounced re-fetch on filter change.
  - "Clear filters" button resets to defaults (30d, all sources, all types).
  - i18n: en + ru for filter labels, period names.
- Acceptance:
  - All filter combinations work correctly.
  - URL updates on filter change, page loads with filters from URL.
  - Clear button resets all filters.
- Tests:
  - Filter URL sync test.
  - Period selector test.
  - Source multi-select test.

## Epic 8 — Confluence Content Integration (Optional)

### OVIA-8001 Confluence Page Sync (Backend)
- Status: `todo`
- Priority: P2
- Owner: Claude
- Depends on: OVIA-3003
- Description:
  - Migration `0008_confluence_pages.sql`:
    - `confluence_pages` table: `id (uuid)`, `org_id`, `external_id (text)`, `space_key (text)`, `title (text)`, `author_account_id (text)`, `last_modifier_account_id (text)`, `status (text)`, `created_at_source (timestamptz)`, `updated_at_source (timestamptz)`, `version (int)`, `url (text)`, `created_at`, `updated_at`.
    - Indexes on `(org_id, space_key)`, `(org_id, author_account_id)`, `(org_id, updated_at_source)`.
  - Extend `ConfluenceClient` with `fetch_pages(space_key, updated_since)` — paginated `/wiki/rest/api/content` with expand=version,history.
  - New `ConfluencePageSyncer`:
    - Watermark-locked sync per space.
    - Upsert pages by external_id.
    - Extract author/modifier account IDs for identity linking.
  - DB repository: `PgConfluenceRepository` with `upsert_page`, `list_pages_by_author_ids(identity_ids, filters)`.
  - Config: `CONFLUENCE_SPACE_KEYS` (CSV) env var for target spaces.
- Acceptance:
  - Pages synced with author/modifier metadata.
  - Watermark-based incremental sync.
  - Author account IDs linkable to existing Confluence identities.
- Tests:
  - 3 client tests: page fetch pagination, expand fields, retry.
  - 3 DB tests: upsert page, list by author, filter by date range.
  - 3 sync tests: lock-skip, incremental via cursor, author extraction.

### OVIA-8002 Confluence Activity in Person 360
- Status: `todo`
- Priority: P2
- Owner: Claude
- Depends on: OVIA-8001, OVIA-7001
- Description:
  - Extend Person 360 activity query to include Confluence page edits.
  - `PgConfluenceRepository::list_activity_by_identity_ids(ids, filters)` — returns page create/edit events.
  - Activity item: `{ source: "confluence", type: "page_edit", title: page_title, url: page_url, timestamp: updated_at_source }`.
  - Frontend: Confluence icon in activity timeline, source filter includes "Confluence" option.
  - Confluence activity appears in unified chronological feed alongside GitLab/Jira items.
- Acceptance:
  - Confluence page edits appear in activity timeline.
  - Source filter correctly includes/excludes Confluence items.
  - Confluence icon renders in timeline.
- Tests:
  - DB: activity query returns confluence pages for linked identities.
  - API: activity endpoint includes confluence items when source=confluence.
  - Frontend: confluence icon render test.

---

## Execution policy
- Claude works one ticket at a time.
- Every ticket requires tests and a short PR summary.
- No merge without review gate (`08-pr-review-gatekeeper`).
