# WO-0001 Review — Rust workspace scaffold

**Date:** 2026-02-20
**Primary skill:** 02-rust-backend-engineer
**Reviewer skill:** 08-pr-review-gatekeeper

---

## What changed

### Workspace root
- `Cargo.toml` — workspace definition with resolver v2, eight member crates, shared dependency versions.
- `.gitignore` — added `/target/`, `**/*.rs.bk`, `Cargo.lock`.

### Shared crates (`crates/`)

| Crate | Purpose |
|-------|---------|
| `ovia-common` | Typed error enum (`OviaError`), result alias, `ServiceInfo` struct. |
| `ovia-config` | `AppConfig::from_env()` — env-based config loader with `.env` support; `init_tracing()` — tracing subscriber bootstrap with `LOG_LEVEL`/`RUST_LOG` filtering. |
| `ovia-db` | `create_pool()` — Postgres connection pool via sqlx `PgPoolOptions`. |

### Service binaries (`services/`)

| Service | Binary | Description |
|---------|--------|-------------|
| `ovia-api` | `ovia-api` | Axum HTTP server with `/health` and `/info` endpoints. |
| `ovia-ingest` | `ovia-ingest` | Placeholder connector worker; awaits ctrl-c. |
| `ovia-metrics` | `ovia-metrics` | Placeholder analytics worker; awaits ctrl-c. |
| `ovia-rag` | `ovia-rag` | Placeholder RAG indexer; awaits ctrl-c. |
| `ovia-scheduler` | `ovia-scheduler` | Placeholder cron orchestrator; awaits ctrl-c. |

All services share the same startup pattern: init tracing, load config from env, log service name, run main loop.

---

## Test evidence

```
cargo fmt --all                                        # clean, no changes
cargo clippy --all-targets --all-features -- -D warnings  # 0 warnings
cargo test --all                                       # 6 passed, 0 failed
```

Tests by crate:

| Crate | Tests | Status |
|-------|-------|--------|
| `ovia-api` | `health_returns_ok`, `info_returns_service_name` | pass |
| `ovia-config` | `config_from_env_succeeds_with_required_vars`, `config_from_env_fails_without_database_url`, `bind_addr_formats_correctly` | pass |
| `ovia-db` | `create_pool_fails_with_invalid_url` | pass |

---

## PR review checklist (08-pr-review-gatekeeper)

| Item | Status | Notes |
|------|--------|-------|
| Architecture fit | yes | Services match `docs/03-architecture.md` runtime services (api, ingest, metrics, rag, scheduler). Shared crates follow layered separation. |
| Data model correctness | yes | No data models introduced (scaffold only). Identity model v2 preserved for OVIA-1001+. |
| Security/secrets handling | yes | Config reads from env vars; `.env` in `.gitignore`; no secrets hardcoded. |
| Test completeness | yes | Smoke tests for config parsing, health endpoint, DB pool creation. |
| Performance risk checked | yes | No performance-critical paths; connection pool defaults are conservative (max 10). |
| Observability added | yes | Tracing subscriber bootstrapped with env-based log-level filtering. |
| Docs updated | yes | This review document. |

---

## Risks

1. **No integration tests with live DB** — `create_pool` test uses an invalid URL to verify error handling. Full integration tests depend on OVIA-1001 (migration baseline) and a test Postgres instance.
2. **Dependency surface** — sqlx pulls in sqlite/mysql features by default; future optimization could use feature flags to trim compile targets.
3. **Config tests use `env::set_var`** — these are not thread-safe; acceptable for sequential test execution but may need isolation if test parallelism is increased.

---

## Follow-ups

| Ticket | Description |
|--------|-------------|
| OVIA-0002 | CI quality gates — GitHub Actions for fmt/clippy/test. |
| OVIA-1001 | SQL migration baseline — first real DB interaction. |
| Future | Trim sqlx feature flags to postgres-only to reduce compile time. |
| Future | Add `rustfmt.toml` and `clippy.toml` for project-wide style config. |
