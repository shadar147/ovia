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

## Next microtasks
1. Add DTOs for query filters/sort options used by `list_mappings`.
2. Implement read-only SQLx repository for `list_mappings`.
3. Add integration test fixture for mapping list query.
