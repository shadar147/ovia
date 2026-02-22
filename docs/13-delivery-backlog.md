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

---

## Execution policy
- Claude works one ticket at a time.
- Every ticket requires tests and a short PR summary.
- No merge without review gate (`08-pr-review-gatekeeper`).
