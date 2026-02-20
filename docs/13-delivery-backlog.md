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
  - Validate and refine `db/migrations/0001_identity_v2.sql`.
  - Add missing indexes for listing/filtering identity conflicts.
- Acceptance:
  - Migration applies on clean DB and existing DB.
  - Index plan documented for key queries.

### OVIA-1002 Identity repository layer
- Status: `in_progress`
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
- Status: `todo`
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
- Status: `todo`
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
- Status: `todo`
- Priority: P1
- Depends on: OVIA-2001, OVIA-1003
- Description:
  - Add status transitions and queue filters for unresolved conflicts.
- Acceptance:
  - `conflict` rows visible in API with sort/filter.
  - Bulk confirm by threshold supported.

## Epic 3 — Connectors (MVP)

### OVIA-3001 Jira incremental sync
### OVIA-3002 GitLab incremental sync
### OVIA-3003 Confluence incremental sync
- Status: `todo`
- Priority: P1
- Acceptance (all):
  - watermark-based sync, idempotent upsert, retry/backoff.
  - raw payload persistence.
  - integration tests with mocked paginated API.

## Epic 4 — Analytics + Ask Ovia

### OVIA-4001 KPI query service
- Status: `todo`
- Priority: P1
- Depends on: Epic 3

### OVIA-4002 Ask API contract with citations
- Status: `todo`
- Priority: P1
- Depends on: OVIA-4001

## Epic 5 — Deployment & Ops

### OVIA-5001 Swarm stack manifests
### OVIA-5002 Monitoring baseline
### OVIA-5003 Backup/restore runbook
- Status: `todo`
- Priority: P1

---

## Execution policy
- Claude works one ticket at a time.
- Every ticket requires tests and a short PR summary.
- No merge without review gate (`08-pr-review-gatekeeper`).
