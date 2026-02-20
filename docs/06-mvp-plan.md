# MVP Plan (6 Weeks)

## Week 1 — Foundation
- repository scaffold
- env and secret handling
- Postgres schema + migrations
- Redis + queue setup

## Week 2 — Connectors
- Jira connector + incremental sync
- GitLab connector + incremental sync
- raw payload persistence

## Week 3 — Confluence + model
- Confluence sync
- canonical entity model
- relationship linking (issue↔MR↔page)

## Week 4 — Analytics
- baseline metrics pipeline
- scheduled aggregate jobs
- API endpoints for KPI retrieval

## Week 5 — Q&A (RAG)
- chunking and embeddings
- retrieval pipeline with metadata filters
- answer synthesis with citations

## Week 6 — Delivery
- Telegram integration
- weekly summary job
- hardening, docs, runbook, handover

## Exit criteria
- stable daily sync
- top 10 management questions answerable
- weekly report generated automatically
- dashboard of ingestion health and lag
