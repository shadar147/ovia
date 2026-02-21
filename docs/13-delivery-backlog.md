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
### OVIA-5003 Backup/restore runbook
- Status: `todo`
- Priority: P1

---

## Execution policy
- Claude works one ticket at a time.
- Every ticket requires tests and a short PR summary.
- No merge without review gate (`08-pr-review-gatekeeper`).
