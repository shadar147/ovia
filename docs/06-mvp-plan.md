# MVP Plan (6 Weeks, Rust + Swarm)

## Week 1 — Foundation
- Rust monorepo scaffold (`api`, `workers`, `connectors`)
- env/secrets policy (Swarm secrets)
- Postgres schema + sqlx migrations
- Redis setup + queue conventions

## Week 2 — Connectors
- Jira connector + incremental sync
- GitLab connector + incremental sync
- raw payload persistence for traceability

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
- Docker Swarm stack manifests
- weekly summary jobs
- hardening, docs, runbook, handover

## Exit criteria
- stable daily sync
- top 10 management questions answerable
- weekly report generated automatically
- dashboard of ingestion health and lag
