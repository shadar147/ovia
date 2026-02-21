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


## Next microtasks
1. Implement `confirm_mapping` transaction + audit event write.
2. Implement `remap_mapping` transaction path.
3. Implement `split_mapping` transaction path.
