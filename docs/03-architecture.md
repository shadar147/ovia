# Architecture (High Level)

## Logical layers
1. **Connectors layer**
   - Jira connector
   - GitLab connector
   - Confluence connector
2. **Ingestion/Sync layer**
   - full sync + incremental sync
   - webhook handlers (optional after MVP)
3. **Normalization layer**
   - map external entities into unified schema
4. **Storage layer**
   - PostgreSQL (operational + analytics tables)
   - pgvector (semantic retrieval)
   - Redis (queues/cache)
5. **Intelligence layer**
   - metrics engine
   - insights/risk engine
   - Q&A orchestration (RAG)
6. **Delivery layer**
   - API + Admin UI
   - Telegram bot interface
   - scheduled reports

## Data flow
1. Connector fetches source updates
2. Raw payload persisted for traceability
3. Normalizer transforms into canonical entities
4. Metrics engine computes aggregates and trends
5. Text content embedded into vector index
6. Query engine routes user question to:
   - structured analytics SQL,
   - semantic retrieval,
   - synthesis answer with citations

## Deployment baseline (Hetzner)
- App/worker nodes
- Dedicated Postgres node
- Redis node
- Reverse proxy + TLS
- Backup + monitoring stack
