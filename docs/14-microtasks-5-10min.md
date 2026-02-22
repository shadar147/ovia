# Ovia — Microtasks (5–10 min chunks)

Rule: each task should be completable in one focused sprint (<=10 min), with a clear output artifact.

## OVIA-0002 CI quality gates (decomposed)
- [ ] MT-0002-01 Create `.github/workflows/ci.yml` skeleton with trigger on push/PR.
- [ ] MT-0002-02 Add rust toolchain setup step.
- [ ] MT-0002-03 Add `cargo fmt --all --check` step.
- [ ] MT-0002-04 Add `cargo clippy --all-targets --all-features -- -D warnings` step.
- [ ] MT-0002-05 Add `cargo test --all` step.
- [ ] MT-0002-06 Add artifact upload for test logs.
- [ ] MT-0002-07 Add README badge/link to CI status.
- [ ] MT-0002-08 Validate workflow syntax locally with `act`-compatible lint (or `yamllint` if available).
- [ ] MT-0002-09 Create `docs/.planning/WO-0002-review.md` template.

## OVIA-1001 SQL migration baseline (decomposed)
- [ ] MT-1001-01 Add index for `person_identity_links(status, confidence)`.
- [ ] MT-1001-02 Add index for `identities(org_id, source, email)` where email is not null.
- [ ] MT-1001-03 Add index for `identities(org_id, source, username)` where username is not null.
- [ ] MT-1001-04 Add comments on `status` semantics in SQL.
- [ ] MT-1001-05 Add query example for conflict queue in docs.
- [ ] MT-1001-06 Add migration apply check instructions in docs.

## OVIA-1002 Identity repository layer (decomposed)
- [ ] MT-1002-01 Define repository trait file for `people`.
- [ ] MT-1002-02 Define repository trait file for `identities`.
- [ ] MT-1002-03 Define repository trait file for `person_identity_links`.
- [ ] MT-1002-04 Define repository trait file for `identity_events`.
- [ ] MT-1002-05 Add DTOs for list/filter requests.
- [ ] MT-1002-06 Implement `list_mappings` query (read-only).
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

## Operating cadence
- One commit every 1–2 microtasks (max ~10 minutes work).
- Each commit includes:
  - What changed
  - Checks run
  - Next microtask
