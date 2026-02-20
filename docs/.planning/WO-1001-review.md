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
- Full migration replay validation remains blocked until PostgreSQL runtime is available on host.

## Next microtasks
1. Provision PostgreSQL runtime (or temporary remote DB) for migration replay.
2. Run migration on clean DB.
3. Run migration against existing schema snapshot.
4. Capture EXPLAIN ANALYZE output and attach to this review.
