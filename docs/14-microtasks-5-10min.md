# Ovia — Microtasks (5–10 min chunks)

Rule: each task should be completable in one focused sprint (<=10 min), with a clear output artifact.

## OVIA-0002 CI quality gates (decomposed)
- [x] MT-0002-01 Create `.github/workflows/ci.yml` skeleton with trigger on push/PR.
- [x] MT-0002-02 Add rust toolchain setup step.
- [x] MT-0002-03 Add `cargo fmt --all --check` step.
- [x] MT-0002-04 Add `cargo clippy --all-targets --all-features -- -D warnings` step.
- [x] MT-0002-05 Add `cargo test --all` step.
- [x] MT-0002-06 Add artifact upload for test logs.
- [x] MT-0002-07 Add README badge/link to CI status.
- [x] MT-0002-08 Validate workflow syntax locally with `act`-compatible lint (or `yamllint` if available).
- [x] MT-0002-09 Create `docs/.planning/WO-0002-review.md` template.

## OVIA-1001 SQL migration baseline (decomposed)
- [x] MT-1001-01 Add index for `person_identity_links(status, confidence)`.
- [x] MT-1001-02 Add index for `identities(org_id, source, email)` where email is not null.
- [x] MT-1001-03 Add index for `identities(org_id, source, username)` where username is not null.
- [x] MT-1001-04 Add comments on `status` semantics in SQL.
- [x] MT-1001-05 Add query example for conflict queue in docs.
- [x] MT-1001-06 Add migration apply check instructions in docs.

## OVIA-1002 Identity repository layer (decomposed)
- [x] MT-1002-01 Define repository trait file for `people`.
- [x] MT-1002-02 Define repository trait file for `identities`.
- [x] MT-1002-03 Define repository trait file for `person_identity_links`.
- [x] MT-1002-04 Define repository trait file for `identity_events`.
- [x] MT-1002-05 Add DTOs for list/filter requests.
- [x] MT-1002-06 Implement `list_mappings` query (read-only).
- [x] MT-1002-07 Add unit test fixture for mapping list.
- [x] MT-1002-08 Implement `confirm_mapping` transaction.
- [x] MT-1002-09 Add integration test for confirm mapping.
- [x] MT-1002-10 Implement `remap_mapping` transaction.
- [x] MT-1002-11 Add integration test for remap.
- [x] MT-1002-12 Implement `split_mapping` transaction.
- [x] MT-1002-13 Add integration test for split.

## OVIA-1003 Identity API v1 (decomposed)
- [x] MT-1003-01 Add route module for identity mappings.
- [x] MT-1003-02 Implement `GET /team/identity-mappings` handler.
- [x] MT-1003-03 Implement query validation for filters.
- [x] MT-1003-04 Add response schema structs.
- [x] MT-1003-05 Add `POST /confirm` handler.
- [x] MT-1003-06 Add `POST /remap` handler.
- [x] MT-1003-07 Add `POST /split` handler.
- [x] MT-1003-08 Emit audit event on each mutation.
- [x] MT-1003-09 Add handler tests for error mapping.

## OVIA-2001 Matching rules v1 (decomposed)
- [x] MT-2001-01 Define scoring config struct + defaults.
- [x] MT-2001-02 Implement exact-email scorer.
- [x] MT-2001-03 Implement username-similarity scorer.
- [x] MT-2001-04 Implement display-name scorer.
- [x] MT-2001-05 Implement team/project co-occurrence scorer.
- [x] MT-2001-06 Implement service-account penalty/exclusion.
- [x] MT-2001-07 Merge scorers into final confidence function.
- [x] MT-2001-08 Add rule-trace payload generation.
- [x] MT-2001-09 Add threshold classifier (`auto/conflict/reject`).
- [x] MT-2001-10 Add fixture tests (at least 15 scenarios).

## OVIA-2002 Conflict queue workflow (decomposed)
- [x] MT-2002-01 Add conflict queue query endpoint.
- [x] MT-2002-02 Add sort options (confidence asc, age desc).
- [x] MT-2002-03 Add bulk confirm endpoint.
- [x] MT-2002-04 Add CSV export formatter.
- [x] MT-2002-05 Add metrics counters for queue size.

## OVIA-5001 Swarm stack manifests (decomposed)
- [x] MT-5001-01 Create multi-stage `backend/Dockerfile` (builder + runtime).
- [x] MT-5001-02 Create `backend/.dockerignore`.
- [x] MT-5001-03 Create `backend/infra/docker-compose.swarm.yml` with all services.
- [x] MT-5001-04 Create `backend/infra/Caddyfile` for reverse proxy.
- [x] MT-5001-05 Create `backend/infra/init-db.sh` migration script.
- [x] MT-5001-06 Create `.env.example` with production variable template.
- [x] MT-5001-07 Update delivery backlog status to done.

## OVIA-3001 Jira incremental sync (decomposed)
- [x] MT-3001-01 Add `raw_ref: Option<serde_json::Value>` to `Identity` struct (align Rust model with SQL).
- [x] MT-3001-02 Add `raw_ref: None` to `make_identity` test helper in matching engine.
- [x] MT-3001-03 Create `0002_sync_watermarks.sql` migration.
- [x] MT-3001-04 Create `db/src/sync/models.rs` — `SyncWatermark` struct.
- [x] MT-3001-05 Create `db/src/sync/repositories.rs` — `SyncWatermarkRepository` trait.
- [x] MT-3001-06 Create `db/src/sync/pg_repository.rs` — Postgres implementation + tests.
- [x] MT-3001-07 Add `upsert_by_external_id` to `IdentityRepository` trait.
- [x] MT-3001-08 Implement `IdentityRepository` for `PgIdentityRepository` (get_by_id, create, update, upsert).
- [x] MT-3001-09 Add workspace deps: `reqwest`, `base64`, `wiremock`.
- [x] MT-3001-10 Create `ingest/src/connector.rs` — shared `Connector` trait + `SyncResult`.
- [x] MT-3001-11 Create `ingest/src/jira/models.rs` — `JiraUser` struct + `is_service_account()` + unit tests.
- [x] MT-3001-12 Create `ingest/src/jira/client.rs` — paginated fetch, retry/backoff, wiremock tests.
- [x] MT-3001-13 Create `ingest/src/jira/sync.rs` — `JiraSyncer` with lock/fetch/upsert cycle + mock tests.
- [x] MT-3001-14 Wire up `ingest/src/main.rs` — load config, build client, run sync.
- [x] MT-3001-15 Update `.env.example` with Jira env vars.
- [x] MT-3001-16 Run `cargo fmt`, `cargo clippy -D warnings`, `cargo test --all` — all green.
- [x] MT-3001-17 Update delivery backlog and microtasks docs.

## OVIA-3002 GitLab incremental sync (decomposed)
- [x] MT-3002-01 Create `ingest/src/gitlab/models.rs` — `GitLabUser` struct + `is_service_account()` + unit tests.
- [x] MT-3002-02 Create `ingest/src/gitlab/client.rs` — paginated fetch via `PRIVATE-TOKEN`, `x-next-page` pagination, retry/backoff, wiremock tests.
- [x] MT-3002-03 Create `ingest/src/gitlab/sync.rs` — `GitLabSyncer` with lock/fetch/upsert cycle + mock tests.
- [x] MT-3002-04 Wire up `ingest/src/main.rs` — load config, build client, run sync after Jira block.
- [x] MT-3002-05 Update `.env.example` with GitLab env vars.
- [x] MT-3002-06 Run `cargo fmt`, `cargo clippy -D warnings`, `cargo test --all` — all green.
- [x] MT-3002-07 Update delivery backlog and microtasks docs.

## OVIA-3003 Confluence incremental sync (decomposed)
- [x] MT-3003-01 Create `ingest/src/confluence/models.rs` — `ConfluenceUser` + `ConfluencePageResponse` structs, `is_service_account()`, `effective_display_name()` + unit tests.
- [x] MT-3003-02 Create `ingest/src/confluence/client.rs` — paginated fetch via Basic auth, group-member endpoint, retry/backoff, wiremock tests.
- [x] MT-3003-03 Create `ingest/src/confluence/sync.rs` — `ConfluenceSyncer` with lock/fetch/upsert cycle + mock tests.
- [x] MT-3003-04 Wire up `ingest/src/main.rs` — load config, build client, run sync after GitLab block.
- [x] MT-3003-05 Update `.env.example` with Confluence env vars.
- [x] MT-3003-06 Run `cargo fmt`, `cargo clippy -D warnings`, `cargo test --all` — all green.
- [x] MT-3003-07 Update delivery backlog and microtasks docs.

## OVIA-4001 KPI query service (decomposed)
- [x] MT-4001-01 Create `0003_kpi_snapshots.sql` migration with kpi_snapshots and risk_items tables.
- [x] MT-4001-02 Create `db/src/kpi/models.rs` — KpiSnapshot, RiskItem, KpiFilter structs.
- [x] MT-4001-03 Create `db/src/kpi/repositories.rs` — KpiRepository trait with save/get/list methods.
- [x] MT-4001-04 Create `db/src/kpi/pg_repository.rs` — Postgres implementation with upsert support.
- [x] MT-4001-05 Add KPI repo integration tests (7 tests: save+get, none for new org, filter by org, filter by date, risk items, empty risks, upsert).
- [x] MT-4001-06 Create `metrics/src/kpi/compute.rs` — compute_delivery_health + compute_release_risk pure functions.
- [x] MT-4001-07 Add compute unit tests (11 tests: perfect/zero/mid health, edge cases, risk levels).
- [x] MT-4001-08 Create `metrics/src/kpi/service.rs` — KpiService with compute_and_save from DB stats.
- [x] MT-4001-09 Wire up `metrics/src/main.rs` — one-shot KPI computation with configurable ORG_ID.
- [x] MT-4001-10 Create `api/src/kpi/` module — handlers, responses, route registration.
- [x] MT-4001-11 Add API handler tests (4 tests: latest snapshot, 404 when empty, history list, risks 404).
- [x] MT-4001-12 Run `cargo fmt`, `cargo clippy -D warnings`, `cargo test --all` — all green.
- [x] MT-4001-13 Update delivery backlog and microtasks docs.

## OVIA-4002 Ask API with citations (decomposed)
- [x] MT-4002-01 Create `0004_ask_sessions.sql` migration with ask_sessions table.
- [x] MT-4002-02 Create `db/src/ask/models.rs` — AskSession, Citation, AskFilter structs.
- [x] MT-4002-03 Create `db/src/ask/repositories.rs` — AskRepository trait.
- [x] MT-4002-04 Create `db/src/ask/pg_repository.rs` — Postgres implementation with JSONB citations.
- [x] MT-4002-05 Add Ask repo integration tests (6 tests: save+get, nonexistent, wrong org, filter by org, limit/offset, no optional fields).
- [x] MT-4002-06 Create `rag/src/ask/engine.rs` — AskEngine stub with KPI data lookup and citation generation.
- [x] MT-4002-07 Create `rag/src/ask/filters.rs` — AskFilters struct.
- [x] MT-4002-08 Add Ask engine unit tests (4 tests: with KPI data, without data, session saved, with filters).
- [x] MT-4002-09 Wire up `rag/src/main.rs` — axum server with POST /ask, GET /ask/:id, GET /ask/history.
- [x] MT-4002-10 Create `api/src/ask/` module — handlers, requests, responses, route registration.
- [x] MT-4002-11 Add API handler tests (4 tests: stub response, empty query 400, session 404, history empty).
- [x] MT-4002-12 Run `cargo fmt`, `cargo clippy -D warnings`, `cargo test --all` — all green.
- [x] MT-4002-13 Update delivery backlog and microtasks docs.

## OVIA-5002 Monitoring baseline (decomposed)
- [x] MT-5002-01 Create `backend/infra/monitoring/prometheus.yml` with scrape configs for all services.
- [x] MT-5002-02 Create `backend/infra/monitoring/alert_rules.yml` with 6 alert rules.
- [x] MT-5002-03 Create `backend/infra/monitoring/loki-config.yml` local config.
- [x] MT-5002-04 Create `backend/infra/monitoring/promtail-config.yml` for Docker log collection.
- [x] MT-5002-05 Create Grafana datasource provisioning (Prometheus + Loki).
- [x] MT-5002-06 Create Grafana dashboard provisioning config.
- [x] MT-5002-07 Create `ovia-overview.json` Grafana dashboard with 9 panels.
- [x] MT-5002-08 Add prometheus, grafana, loki, promtail, node-exporter to docker-compose.swarm.yml.
- [x] MT-5002-09 Add prometheus_data, grafana_data, loki_data volumes.
- [x] MT-5002-10 Update Caddyfile with /grafana/* reverse proxy.
- [x] MT-5002-11 Update `.env.example` with Grafana credentials.
- [x] MT-5002-12 Add stub `/metrics` endpoint to ovia-api with Prometheus text format.
- [x] MT-5002-13 Add test for `/metrics` endpoint.
- [x] MT-5002-14 Update delivery backlog and microtasks docs.

## OVIA-5003 Backup/restore runbook (decomposed)
- [x] MT-5003-01 Create `backend/infra/backup/backup.sh` — daily + weekly pg_dump with retention cleanup.
- [x] MT-5003-02 Create `backend/infra/backup/restore.sh` — pg_restore with --drop flag and row-count verification.
- [x] MT-5003-03 Create `backend/infra/backup/verify-backup.sh` — non-destructive archive validation.
- [x] MT-5003-04 Add backup service to `docker-compose.swarm.yml` with volume and placement constraint.
- [x] MT-5003-05 Update `.env.example` with backup retention variables.
- [x] MT-5003-06 Write `docs/15-backup-restore-runbook.md` covering all restore scenarios and operational procedures.
- [x] MT-5003-07 Update delivery backlog and microtasks docs.

## OVIA-3004 GitLab MR/Pipeline sync + real KPI (decomposed)
- [x] MT-3004-01 Create `0005_gitlab_data.sql` migration with gitlab_projects, gitlab_merge_requests, gitlab_pipelines tables.
- [x] MT-3004-02 Create `db/src/gitlab/models.rs` — GitlabProject, GitlabMergeRequest, GitlabPipeline, ReviewDurationRow, StaleMrRow structs.
- [x] MT-3004-03 Create `db/src/gitlab/pg_repository.rs` — PgGitlabRepository with upsert helpers + KPI query helpers.
- [x] MT-3004-04 Wire `pub mod gitlab` into `db/src/lib.rs`.
- [x] MT-3004-05 Add API response structs to `ingest/src/gitlab/models.rs` — GitLabProject, GitLabMergeRequest, GitLabMrAuthor, GitLabPipeline.
- [x] MT-3004-06 Refactor `ingest/src/gitlab/client.rs` — generic `fetch_all_pages<T>` + `request_with_retry<T>`, add `fetch_all_projects`, `fetch_merged_mrs`, `fetch_open_mrs`, `fetch_pipelines`.
- [x] MT-3004-07 Create `ingest/src/gitlab/mr_sync.rs` — GitLabMrPipelineSyncer with watermark lock, project/MR/pipeline sync, incremental via cursor.
- [x] MT-3004-08 Wire `pub mod mr_sync` into `ingest/src/gitlab/mod.rs`.
- [x] MT-3004-09 Wire MR/pipeline sync into `ingest/src/main.rs` after identity sync.
- [x] MT-3004-10 Rewrite `metrics/src/kpi/service.rs` — real GitLab queries for throughput, review latency, risk scores, risk item generation.
- [x] MT-3004-11 Add percentile helper function with unit tests.
- [x] MT-3004-12 Run `cargo build` — clean with no warnings.
- [x] MT-3004-13 Run `cargo test -p ovia-db -p ovia-ingest -p ovia-metrics` — all 122 tests pass.
- [x] MT-3004-14 Update delivery backlog and microtasks docs.

## OVIA-3005 Jira Issue Sync — Block 1 (decomposed)
- [x] MT-3005-01 Add `project_keys: Vec<String>` and `sync_window_days: u32` to `JiraClientConfig`.
- [x] MT-3005-02 Change `JiraClientConfig::from_env()` to `Result<Option<Self>, String>` with fail-fast on missing `JIRA_PROJECT_KEYS`.
- [x] MT-3005-03 Implement `parse_csv_project_keys()` helper with trim/uppercase/empty-reject.
- [x] MT-3005-04 Add 8 tests for CSV parser and from_env behavior.
- [x] MT-3005-05 Create `jira/query.rs` — `build_issue_search_jql()` with bounded window and project filter.
- [x] MT-3005-06 Add 7 JQL builder tests: single/multi project, bounded window, escaping.
- [x] MT-3005-07 Update `main.rs` to handle `Result<Option<_>>` with fail-fast panic.
- [x] MT-3005-08 Update `.env.example` with `JIRA_PROJECT_KEYS` and `JIRA_SYNC_WINDOW_DAYS`.
- [x] MT-3005-09 Update sync.rs test helpers for new config fields.
- [x] MT-3005-10 Run `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test -p ovia-ingest` — all green (73 tests).
- [x] MT-3005-11 Update delivery backlog and microtasks docs.

## OVIA-3006 Jira Issue Sync — Block 2 (decomposed)
- [x] MT-3006-01 Create `0006_jira_issues.sql` migration with `jira_issues` + `jira_issue_transitions` tables.
- [x] MT-3006-02 Create `db/src/jira/models.rs` — `JiraIssueRow`, `JiraIssueTransitionRow` structs.
- [x] MT-3006-03 Create `db/src/jira/pg_repository.rs` — `PgJiraRepository` with `upsert_issue`, `insert_transition`, `delete_transitions`.
- [x] MT-3006-04 Add API models: `JiraSearchResponse`, `JiraIssue`, `JiraChangelogResponse` with sprint/team/story_points.
- [x] MT-3006-05 Add `search_issues()` to `JiraClient` — paginated `/rest/api/3/search` with JQL.
- [x] MT-3006-06 Add `fetch_issue_changelog()` to `JiraClient` — paginated changelog endpoint.
- [x] MT-3006-07 Refactor client to generic `request_with_retry<T: DeserializeOwned>`.
- [x] MT-3006-08 Create `ingest/src/jira/issue_sync.rs` — `JiraIssueSyncer` with watermark lock + JQL window.
- [x] MT-3006-09 Implement changelog extraction: status + sprint transitions, replace strategy.
- [x] MT-3006-10 Parse analytics fields: `customfield_10016` (story_points), `customfield_10020` (sprint), `customfield_10001` (team).
- [x] MT-3006-11 Wire issue sync into `ingest/src/main.rs` after identity sync.
- [x] MT-3006-12 Add 17 issue_sync tests + 3 DB tests (20 new, 141 total).
- [x] MT-3006-13 Run `cargo fmt --check`, `cargo clippy -D warnings` — clean (after fmt fix).
- [x] MT-3006-14 Update delivery backlog and microtasks docs.

## OVIA-3007 Jira Issue Sync — Block 3A (decomposed)
- [x] MT-3007-01 Create `0007_kpi_jira_columns.sql` migration — add 4 Jira columns to `kpi_snapshots`.
- [x] MT-3007-02 Add `count_open_blockers()` to `PgJiraRepository`.
- [x] MT-3007-03 Add `list_open_blocker_age_days()` to `PgJiraRepository`.
- [x] MT-3007-04 Add `spillover_rate()` to `PgJiraRepository`.
- [x] MT-3007-05 Add `get_cycle_times_hours()` to `PgJiraRepository`.
- [x] MT-3007-06 Add `count_resolved_issues()` and `count_resolved_issues_by_type()` to `PgJiraRepository`.
- [x] MT-3007-07 Wire Jira metrics into `KpiService::compute_and_save` — combine MR + Jira throughput, feed blockers into risk.
- [x] MT-3007-08 Update `KpiSnapshot` model with 4 new optional fields.
- [x] MT-3007-09 Add 8 integration tests for Jira metric queries.
- [x] MT-3007-10 Run `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` — all green.

## OVIA-3008 Jira Issue Sync — Block 3B (decomposed)
- [x] MT-3008-01 Wire `blocker_count`, `spillover_rate`, `cycle_time_p50_hours`, `cycle_time_p90_hours` into KPI API response.
- [x] MT-3008-02 Update `/team/kpi`, `/team/kpi/history`, `/team/kpi/risks` endpoints with Jira fields.
- [x] MT-3008-03 Update test fixtures with Jira metric values.
- [x] MT-3008-04 Add `kpi_response_contract_includes_all_fields` contract test.
- [x] MT-3008-05 Add `kpi_history_includes_jira_metrics` test.
- [x] MT-3008-06 Verify 203 total tests pass (59 db + 90 ingest + 16 metrics + 34 api + 4 rag).
- [x] MT-3008-07 Update delivery backlog and microtasks docs.

## OVIA-3009 Dashboard Jira KPI + Risk Pagination — Block A (decomposed)
- [x] MT-3009-01 Add `blocker_count`, `spillover_rate`, `cycle_time_p50_hours`, `cycle_time_p90_hours` to frontend `KpiSnapshot` type.
- [x] MT-3009-02 Add 3 new KPI cards (Blockers, Spillover Rate, Cycle Time) to `kpi-cards-row.tsx`.
- [x] MT-3009-03 Add client-side pagination (20/page) to `risk-table.tsx` with prev/next navigation.
- [x] MT-3009-04 Update i18n messages (en + ru) for new KPI cards and pagination controls.
- [x] MT-3009-05 Update all chart test fixtures with new Jira KPI fields.
- [x] MT-3009-06 Add pagination tests (page size, navigation, boundary).

## OVIA-3010 Extensible Throughput Classification — Block B (decomposed)
- [x] MT-3010-01 Create `metrics/src/kpi/classify.rs` — bug/feature/chore mapping with Jira type + GitLab label strategy.
- [x] MT-3010-02 Add `count_resolved_issues_by_types` (multi-type) to `PgJiraRepository`.
- [x] MT-3010-03 Add `count_merged_mrs_by_labels` (multi-label) to `PgGitlabRepository`.
- [x] MT-3010-04 Update KPI service to use expanded classification mappings.
- [x] MT-3010-05 Add classification invariant tests (no overlap, non-empty sets).

## OVIA-3011 Jira Identities Ingest from Issues — Block C (decomposed)
- [x] MT-3011-01 Enrich `JiraIssueSyncer` to collect unique user refs (assignee + reporter).
- [x] MT-3011-02 Upsert collected users as identities (`source=jira`) with dedup by `accountId`.
- [x] MT-3011-03 Handle null-safe fields and mark app accounts as service accounts.
- [x] MT-3011-04 Pass `IdentityRepository` + `OrgId` into issue syncer for identity upsert.
- [x] MT-3011-05 Add 5 new unit tests for identity extraction scenarios.

## OVIA-CI-001 Clippy lint fix (decomposed)
- [x] MT-CI-001-01 Remove empty line after doc-comments in `classify.rs` (commit `d5ab246`).

## OVIA-6001 People CRUD API — Backend (decomposed)
- [ ] MT-6001-01 Extend `PersonRepository` trait: add `list(org_id, filters) -> Vec<Person>` and `soft_delete(org_id, id) -> Result`.
- [ ] MT-6001-02 Add `PersonFilter` struct: `team: Option<String>`, `status: Option<String>`, `search: Option<String>`, `limit`, `offset`.
- [ ] MT-6001-03 Implement `PgPersonRepository::list()` with search substring match on display_name + email, team/status filters, LIMIT/OFFSET.
- [ ] MT-6001-04 Implement `PgPersonRepository::soft_delete()` — set `status='inactive'`, `updated_at=now()`.
- [ ] MT-6001-05 Add DB integration tests: list with filters (3 tests), soft_delete + re-list (1 test), pagination boundary (1 test).
- [ ] MT-6001-06 Create `api/src/people/` module — `mod.rs`, `handlers.rs`, `requests.rs`, `responses.rs`.
- [ ] MT-6001-07 Define `PersonResponse` struct: `id, display_name, primary_email, team, role, status, identity_count, created_at, updated_at`.
- [ ] MT-6001-08 Implement `GET /team/people` handler with filter query params + pagination.
- [ ] MT-6001-09 Implement `GET /team/people/:id` handler with identity count sub-query.
- [ ] MT-6001-10 Implement `POST /team/people` handler with validation (display_name required, email format).
- [ ] MT-6001-11 Implement `PUT /team/people/:id` handler with partial update support.
- [ ] MT-6001-12 Implement `DELETE /team/people/:id` handler — calls soft_delete, returns 204.
- [ ] MT-6001-13 Register `/team/people` routes in API router.
- [ ] MT-6001-14 Add 5 handler tests: list (200), get (200), create (201), update (200), delete (204).
- [ ] MT-6001-15 Add 3 error handler tests: get 404, create validation 400, duplicate email 409.
- [ ] MT-6001-16 Run `cargo sqlx prepare --workspace` to update `.sqlx/` offline cache.
- [ ] MT-6001-17 Run `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --all` — all green.
- [ ] MT-6001-18 Update delivery backlog status to `done`.

## OVIA-6002 Manual Identity Linking API — Backend (decomposed)
- [ ] MT-6002-01 Implement `POST /team/people/:id/identities` handler — validate person + identity exist, same org, not already linked.
- [ ] MT-6002-02 Create `person_identity_link` with `status=verified`, `confidence=1.0`, `verified_by='manual'` in handler logic.
- [ ] MT-6002-03 Emit `identity_event` with `action=manual_link` on successful link.
- [ ] MT-6002-04 Implement `DELETE /team/people/:id/identities/:identity_id` handler — set `valid_to=now()`, emit `manual_unlink` event.
- [ ] MT-6002-05 Implement `GET /team/people/:id/identities` handler — list linked identities with source, username, email, status, linked_at.
- [ ] MT-6002-06 Add validation: reject link if identity already linked to another person (return 409 with remap hint).
- [ ] MT-6002-07 Register `/team/people/:id/identities` routes in API router.
- [ ] MT-6002-08 Add 4 handler tests: link (201), unlink (204), list identities (200), link-already-linked (409).
- [ ] MT-6002-09 Add 2 integration tests: audit event emission check, concurrent link conflict.
- [ ] MT-6002-10 Run `cargo sqlx prepare --workspace`, `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --all` — all green.
- [ ] MT-6002-11 Update delivery backlog status to `done`.

## OVIA-6003 People List Page — Frontend (decomposed)
- [ ] MT-6003-01 Create route `/team/people` — add `app/(app)/team/people/page.tsx`.
- [ ] MT-6003-02 Add sidebar navigation link "People" with users icon (between Team Identity and Reports).
- [ ] MT-6003-03 Add API client functions: `fetchPeople(filters)`, `createPerson(data)`, `updatePerson(id, data)`, `deletePerson(id)`.
- [ ] MT-6003-04 Add `Person`, `PersonFilter`, `PersonListResponse` types to `lib/api/types.ts`.
- [ ] MT-6003-05 Create `PeopleTable` component — columns: Name, Email, Team, Role, Status badge, Identities count badge, Actions menu.
- [ ] MT-6003-06 Add search bar with 300ms debounce, updates `search` URL param.
- [ ] MT-6003-07 Add filter chips: team dropdown (fetched from distinct values), status toggle (active/inactive/all).
- [ ] MT-6003-08 Add pagination controls (reuse risk-table pattern: 20/page, prev/next).
- [ ] MT-6003-09 Row click handler: navigate to `/team/people/:id`.
- [ ] MT-6003-10 Add "Add Person" button in page header.
- [ ] MT-6003-11 Add i18n messages (en + ru): page title, column headers, empty state, filter labels.
- [ ] MT-6003-12 Add render test with mock data, search filter test, pagination test.
- [ ] MT-6003-13 Update delivery backlog status to `done`.

## OVIA-6004 Person Create/Edit Dialog — Frontend (decomposed)
- [ ] MT-6004-01 Create `PersonFormDialog` component — modal with fields: display_name (required), primary_email, team, role, status.
- [ ] MT-6004-02 Add client-side validation: display_name non-empty, email format regex if provided.
- [ ] MT-6004-03 Wire create mode: "Add Person" button → dialog → `POST /team/people` → success toast → table refresh.
- [ ] MT-6004-04 Wire edit mode: row action "Edit" → pre-filled dialog → `PUT /team/people/:id` → success toast → table refresh.
- [ ] MT-6004-05 Add delete confirmation dialog: row action "Delete" → confirm modal → `DELETE /team/people/:id` → toast → table refresh.
- [ ] MT-6004-06 Add i18n messages (en + ru): form labels, validation errors, confirmation text, success/error toasts.
- [ ] MT-6004-07 Add tests: form validation (empty name, invalid email), create submit mock, edit pre-fill.
- [ ] MT-6004-08 Update delivery backlog status to `done`.

## OVIA-6005 Multi-Identity Mapping UI — Frontend (decomposed)
- [ ] MT-6005-01 Create `IdentityLinkPanel` component — shows linked identities with source icon, username, email, status badge, linked date.
- [ ] MT-6005-02 Add source icon mapping: GitLab (git-merge), Jira (ticket), Confluence (file-text), Git (git-commit).
- [ ] MT-6005-03 Add API client: `fetchPersonIdentities(personId)`, `linkIdentity(personId, identityId)`, `unlinkIdentity(personId, identityId)`.
- [ ] MT-6005-04 Create "Link Identity" search dialog — search orphan identities by username/email/source with typeahead.
- [ ] MT-6005-05 Wire link flow: search → select → `POST /team/people/:id/identities` → optimistic UI update → refetch.
- [ ] MT-6005-06 Wire unlink flow: row "Unlink" button → confirmation dialog → `DELETE` → optimistic update → refetch.
- [ ] MT-6005-07 Add i18n messages (en + ru): panel title, link/unlink labels, search placeholder, confirmation text, empty state.
- [ ] MT-6005-08 Add tests: link identity flow, unlink with confirmation, orphan search rendering.
- [ ] MT-6005-09 Update delivery backlog status to `done`.

## OVIA-7001 Person 360 Backend API (decomposed)
- [ ] MT-7001-01 Implement `GET /team/people/:id/profile` handler — return person + all linked identities + summary stats.
- [ ] MT-7001-02 Add `PersonProfileResponse` struct: person fields + `identities: Vec<LinkedIdentity>` + `stats: ProfileStats`.
- [ ] MT-7001-03 Add `ProfileStats` struct: `total_mrs: i64`, `total_issues: i64`, `active_days_30d: i64`.
- [ ] MT-7001-04 Implement stats queries: count MRs by author identity IDs, count issues by assignee identity IDs, count distinct active days.
- [ ] MT-7001-05 Implement `GET /team/people/:id/activity` handler with query params: `period`, `from`, `to`, `source`, `type`, `limit`, `offset`.
- [ ] MT-7001-06 Define `ActivityItem` struct: `id, source, activity_type, title, url, timestamp, metadata`.
- [ ] MT-7001-07 Add `PgGitlabRepository::list_activity_by_identity_ids(ids, period, limit, offset)` — returns MR activity items.
- [ ] MT-7001-08 Add `PgJiraRepository::list_activity_by_identity_ids(ids, period, limit, offset)` — returns issue activity items.
- [ ] MT-7001-09 Implement unified activity merge: query all sources in parallel, merge by timestamp desc, apply pagination.
- [ ] MT-7001-10 Register `/team/people/:id/profile` and `/team/people/:id/activity` routes.
- [ ] MT-7001-11 Add 4 DB tests: gitlab MRs by identity IDs, jira issues by identity IDs, combined query, empty result.
- [ ] MT-7001-12 Add 3 API handler tests: profile with identities+stats, activity with filters, activity pagination.
- [ ] MT-7001-13 Run `cargo sqlx prepare --workspace`, `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --all` — all green.
- [ ] MT-7001-14 Update delivery backlog status to `done`.

## OVIA-7002 Person 360 Page — Frontend (decomposed)
- [ ] MT-7002-01 Create route `/team/people/:id` — add `app/(app)/team/people/[id]/page.tsx`.
- [ ] MT-7002-02 Add API client: `fetchPersonProfile(id)`, `fetchPersonActivity(id, filters)`.
- [ ] MT-7002-03 Create `PersonHeader` component: display_name, email, team badge, role, status badge, "Edit" button.
- [ ] MT-7002-04 Create `PersonStats` component: 3 compact cards (Total MRs, Total Issues, Active Days 30d).
- [ ] MT-7002-05 Embed `IdentityLinkPanel` from OVIA-6005 in profile page.
- [ ] MT-7002-06 Create `ActivityTimeline` component: chronological feed with source icon, type badge, title (clickable link), relative timestamp.
- [ ] MT-7002-07 Add "Load more" button at bottom of timeline (increments offset, appends items).
- [ ] MT-7002-08 Add breadcrumb navigation: People → Person Name.
- [ ] MT-7002-09 Add i18n messages (en + ru): section headers, stats labels, empty states, breadcrumb.
- [ ] MT-7002-10 Add tests: profile render with mock data, activity timeline render, empty state.
- [ ] MT-7002-11 Update delivery backlog status to `done`.

## OVIA-7003 Activity Timeline Filters — Frontend (decomposed)
- [ ] MT-7003-01 Create `ActivityFilters` component — filter bar above activity timeline.
- [ ] MT-7003-02 Add period selector: 7d / 30d / 90d / Custom (date range picker).
- [ ] MT-7003-03 Add source multi-select checkboxes: GitLab, Jira, Confluence.
- [ ] MT-7003-04 Add type multi-select checkboxes: Merge Requests, Issues, Identity Events.
- [ ] MT-7003-05 Persist filters in URL query params (`?period=30d&source=gitlab,jira&type=merge_request`).
- [ ] MT-7003-06 Debounced re-fetch on filter change (300ms).
- [ ] MT-7003-07 Add "Clear filters" button — resets to defaults (30d, all sources, all types).
- [ ] MT-7003-08 Add i18n messages (en + ru): filter labels, period names, source names, type names, clear button.
- [ ] MT-7003-09 Add tests: filter URL sync, period selector, source multi-select.
- [ ] MT-7003-10 Update delivery backlog status to `done`.

## OVIA-8001 Confluence Page Sync — Backend (decomposed)
- [ ] MT-8001-01 Create `0008_confluence_pages.sql` migration: `confluence_pages` table with indexes on `(org_id, space_key)`, `(org_id, author_account_id)`, `(org_id, updated_at_source)`.
- [ ] MT-8001-02 Create `db/src/confluence/models.rs` — `ConfluencePageRow` struct.
- [ ] MT-8001-03 Create `db/src/confluence/pg_repository.rs` — `PgConfluenceRepository` with `upsert_page`, `list_pages_by_author_ids(ids, filters)`.
- [ ] MT-8001-04 Wire `pub mod confluence` into `db/src/lib.rs`.
- [ ] MT-8001-05 Add `CONFLUENCE_SPACE_KEYS` (CSV) env var to `ConfluenceClientConfig` with validation.
- [ ] MT-8001-06 Extend `ConfluenceClient` with `fetch_pages(space_key, updated_since)` — paginated `/wiki/rest/api/content` with `expand=version,history`.
- [ ] MT-8001-07 Create `ConfluencePageSyncer` — watermark-locked sync per space, upsert pages, extract author/modifier account IDs.
- [ ] MT-8001-08 Wire page sync into `ingest/src/main.rs` after identity sync.
- [ ] MT-8001-09 Update `.env.example` with `CONFLUENCE_SPACE_KEYS`.
- [ ] MT-8001-10 Add 3 client tests: page fetch pagination, expand fields, retry on 5xx.
- [ ] MT-8001-11 Add 3 DB tests: upsert page, list by author IDs, filter by date range.
- [ ] MT-8001-12 Add 3 sync tests: lock-skip, incremental via cursor, author extraction.
- [ ] MT-8001-13 Run `cargo sqlx prepare --workspace`, `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --all` — all green.
- [ ] MT-8001-14 Update delivery backlog status to `done`.

## OVIA-8002 Confluence Activity in Person 360 (decomposed)
- [ ] MT-8002-01 Add `PgConfluenceRepository::list_activity_by_identity_ids(ids, filters)` — returns page create/edit as ActivityItem.
- [ ] MT-8002-02 Extend Person 360 activity query to include Confluence source in unified merge.
- [ ] MT-8002-03 Add Confluence icon to `ActivityTimeline` component source icon mapping.
- [ ] MT-8002-04 Add "Confluence" option to source multi-select in `ActivityFilters`.
- [ ] MT-8002-05 Add DB test: activity query returns confluence pages for linked identities.
- [ ] MT-8002-06 Add API test: activity endpoint includes confluence items when `source=confluence`.
- [ ] MT-8002-07 Add frontend test: confluence icon renders in timeline.
- [ ] MT-8002-08 Update i18n (en + ru): "Confluence" source label, page_edit type label.
- [ ] MT-8002-09 Update delivery backlog status to `done`.

## Operating cadence
- One commit every 1–2 microtasks (max ~10 minutes work).
- Each commit includes:
  - What changed
  - Checks run
  - Next microtask
