# Decision log

## 2026-02-20
- Initialized Claude execution backlog and work-order system.
- Chosen sequence start: Foundation -> Identity v2 -> Matching.

## 2026-02-21
- Completed Epic 0–2 (Foundation, Identity Model v2, Matching Engine).
- Epic 3 (Connectors): Jira implemented first, then GitLab + Confluence delegated to parallel subagents.
- Shared connector pattern: `Connector` trait, watermark-based sync, idempotent upsert, retry/backoff.
- Epic 4 + 5 delegated to 3 parallel subagents with worktree isolation to avoid conflicts.
- Ask API uses stub engine (no real LLM) — returns structured answers from KPI data with citations. Ready for Claude API integration.
- Monitoring stack: Prometheus + Grafana + Loki (no Alertmanager yet — alerts defined but no receiver).
- Backup: pg_dump-based (no WAL PITR) — sufficient for MVP scale.
- All 15 backlog tickets completed. 166 tests passing.
