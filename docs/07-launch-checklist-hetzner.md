# Launch Checklist (Hetzner)

## Infrastructure
- [ ] Provision APP node (8 vCPU / 16 GB)
- [ ] Provision DB node (16 vCPU / 32 GB + 300-500 GB volume)
- [ ] Provision Redis node (2-3 vCPU / 4-8 GB)
- [ ] Configure private networking between nodes
- [ ] Configure DNS and TLS

## Security
- [ ] UFW + fail2ban
- [ ] SSH hardening (keys only, no password)
- [ ] service users + least privilege
- [ ] secret files with 600 permissions

## Data services
- [ ] PostgreSQL + pgvector installed
- [ ] backup schedule configured (daily + weekly)
- [ ] restore tested at least once

## Application
- [ ] Deploy API + workers
- [ ] Configure connectors (Jira/GitLab/Confluence)
- [ ] Run initial backfill
- [ ] Enable incremental sync schedules

## Intelligence
- [ ] Embeddings pipeline enabled
- [ ] RAG retrieval validated with citations
- [ ] Top management questions validated

## Operations
- [ ] Monitoring dashboards live
- [ ] Alert rules configured (sync failures, lag, DB health)
- [ ] Incident runbook documented
