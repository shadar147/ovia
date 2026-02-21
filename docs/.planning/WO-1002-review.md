# WO-1002 Review — Identity repository layer (in progress)

## What changed (this slice)
- Added `crates/db/src/identity/models.rs` with core identity domain structs.
- Added `crates/db/src/identity/repositories.rs` with repository traits:
  - `PersonRepository`
  - `IdentityRepository`
  - `PersonIdentityLinkRepository`
  - `IdentityEventRepository`
- Exported identity module via `crates/db/src/lib.rs`.
- Updated `crates/db/Cargo.toml` with required deps (`uuid`, `serde`, `chrono`, `async-trait`).

## Checks
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all`

All passed locally.

## What changed (this slice)
- Added `PgIdentityRepository` (`crates/db/src/identity/pg_repository.rs`).
- Implemented read-only `list_mappings` with SQLx `QueryBuilder` and filters:
  - `status`
  - `min_confidence`
  - `max_confidence`
  - `limit`/`offset`
- Added deterministic status conversion helpers (`LinkStatus::as_str`, `FromStr`).
- Added integration-style repo test (runs when `TEST_DATABASE_URL` is set).


## What changed (this slice)
- Implemented mutation transactions in `backend/crates/db/src/identity/pg_repository.rs`:
  - `confirm_mapping`
  - `remap_mapping`
  - `split_mapping`
- Added audit event persistence (`identity_events`) via shared `append_event` helper.
- Added `NotFound` guards when target active link is missing.
- Added `serde_json` dependency to `ovia-db` for structured event payloads.

## Next microtasks
1. Add integration tests for `confirm/remap/split` mutation paths with fixture setup.
2. Add event payload assertions for remap flow.
3. Mark OVIA-1002 done after mutation tests are in place.
