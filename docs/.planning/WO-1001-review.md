# WO-1001 Review — SQL migration baseline (done)

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
- Added reviewer query plan + migration apply checklist: `docs/16-identity-query-plan.md`.
- Migration replay executed on local Postgres container (clean apply + idempotent re-apply).
