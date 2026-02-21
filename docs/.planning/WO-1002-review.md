# WO-1002 Review — Identity repository layer (done)

## What changed
- Added `crates/db/src/identity/models.rs` with core identity domain structs.
- Added `crates/db/src/identity/repositories.rs` with repository traits:
  - `PersonRepository`
  - `IdentityRepository`
  - `PersonIdentityLinkRepository`
  - `IdentityEventRepository`
- Exported identity module via `crates/db/src/lib.rs`.
- Updated `crates/db/Cargo.toml` with required deps (`uuid`, `serde`, `chrono`, `async-trait`).
- Added `PgIdentityRepository` (`crates/db/src/identity/pg_repository.rs`).
- Implemented `list_mappings` with SQLx `QueryBuilder` and filters (status, confidence, limit/offset).
- Added deterministic status conversion helpers (`LinkStatus::as_str`, `FromStr`).
- Implemented mutation transactions: `confirm_mapping`, `remap_mapping`, `split_mapping`.
- Added audit event persistence (`identity_events`) via shared `append_event` helper.
- Added `upsert_by_external_id` for idempotent connector inserts.
- Added `raw_ref: Option<serde_json::Value>` to `Identity` struct.

## Checks
- `cargo fmt --all` — passed
- `cargo clippy --all-targets --all-features -- -D warnings` — passed
- `cargo test --all` — 45 db tests passing

## Validation
- Integration tests for all CRUD + mutation paths.
- Bulk confirm, conflict queue filtering, sort, stats — all tested.
