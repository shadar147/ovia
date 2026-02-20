# WO-1001 Review — SQL migration baseline (partial)

## What changed
- Added conflict queue index for active links:
  - `person_identity_links_conflict_queue_idx`
- Added identity lookup indexes:
  - `identities_source_email_idx`
  - `identities_source_username_idx`
- Documented `status` semantics in migration comments.

## Why
- Speed up identity conflict triage queries and filter/sort operations.
- Improve lookup performance for matching engine pre-filters.

## Validation status
- SQL updated and committed.
- Added reviewer query plan + migration apply checklist: `docs/15-identity-query-plan.md`.
- Migration replay executed on local Postgres container (clean apply + idempotent re-apply).
- Created index snapshot: `docs/.planning/WO-1001-indexes.txt`.

## Next microtasks
1. (Optional) Run migration against existing schema snapshot from production-like dump.
2. Capture EXPLAIN ANALYZE output and attach to this review.
3. Move to OVIA-1002 repository layer implementation.
