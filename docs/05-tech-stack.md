# Recommended Tech Stack (Rust + Docker Swarm)

## Backend
- **Rust (stable)** as primary backend language
- **axum** for HTTP API + webhooks
- **tokio** async runtime
- **sqlx** for PostgreSQL access and migrations
- **serde** for schema-safe payload mapping

## Data
- **PostgreSQL 16** (primary store)
- **pgvector** (semantic index)
- **Redis** (queues/cache/locks)
- **S3-compatible object storage** (optional for raw payload archive)

## Queue/Jobs
- **Redis Streams / BullMQ-equivalent in Rust** (job orchestration)
- **Scheduled workers** for sync + analytics recalculation

## Frontend
- **Next.js** admin panel (phase 2)
- Telegram as primary interface for MVP

## LLM layer
- Pluggable provider adapters (OpenAI/Anthropic)
- embeddings + answer synthesis abstraction
- citation-first answer policy

## Infra/DevOps
- **Docker + Docker Swarm** for deployment
- Swarm stacks for `api`, `workers`, `ingestion`, `monitoring`
- GitHub Actions CI
- Prometheus + Grafana + Loki
- Terraform (phase 2, optional)

## Security baseline
- secrets in Swarm secrets / env files (no chat exposure)
- least-privilege API tokens (read-only by default)
- encryption in transit (TLS)
- regular backup/restore drills
