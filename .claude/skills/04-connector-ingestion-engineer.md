# Skill: Connector & Ingestion Engineer

Goal: ingest Jira/GitLab/Confluence reliably.

Rules:
- Implement incremental sync by watermark.
- Handle retries/backoff/rate-limits.
- Persist raw payload for traceability.
- Ensure idempotent upserts.
- Add dead-letter handling for failed jobs.

Tests:
- Mock API pagination and rate-limit responses.
- Validate deduplication and re-run safety.
- Validate watermark progression.

DoD:
- Re-running same sync window produces no duplicates.
- Sync failure is observable and recoverable.
