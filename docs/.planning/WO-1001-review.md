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
- Full migration replay validation remains next microtask.

## Next microtasks
1. Run migration on clean DB.
2. Run migration against existing schema snapshot.
3. Add query examples + explain notes in docs.
