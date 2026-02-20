# Recommended Tech Stack

## Backend
- **TypeScript + NestJS** (modular API and workers)
- **Python micro-workers** (optional for analytics-heavy jobs)

## Data
- **PostgreSQL 16** (primary store)
- **pgvector** (semantic index)
- **Redis** (queues/cache)
- **S3-compatible object storage** (optional for payload archival)

## Queue/Jobs
- **BullMQ** (Redis-based job orchestration)

## Frontend
- **Next.js** admin panel (phase 2)
- Telegram as primary interface for MVP

## LLM/AI layer
- Pluggable provider interface (OpenAI/Anthropic)
- embeddings + answer synthesis abstraction
- citation-first answer policy

## Infra/DevOps
- Docker + Docker Compose (MVP)
- Terraform (phase 2)
- GitHub Actions CI
- Prometheus + Grafana + Loki

## Security baseline
- secrets in file/env vault (no chat exposure)
- least-privilege API tokens (read-only by default)
- encryption in transit (TLS)
- regular backup/restore drills
