# Launch Checklist (Hetzner + Docker Swarm)

## Infrastructure
- [ ] Provision Swarm manager node
- [ ] Provision 1-2 Swarm worker nodes
- [ ] Provision DB node (16 vCPU / 32 GB + 300-500 GB volume)
- [ ] Provision Redis node (2-3 vCPU / 4-8 GB)
- [ ] Configure private networking between nodes
- [ ] Configure DNS and TLS

## Swarm setup
- [ ] `docker swarm init` on manager
- [ ] Join worker nodes with manager token
- [ ] Create overlay networks (`ovia-core`, `ovia-data`, `ovia-monitoring`)
- [ ] Create Swarm secrets for Jira/GitLab/Confluence/API providers
- [ ] Deploy stack files (`docker stack deploy`)

## Security
- [ ] UFW + fail2ban
- [ ] SSH hardening (keys only, no password)
- [ ] service users + least privilege
- [ ] rotate tokens and secrets policy

## Data services
- [ ] PostgreSQL + pgvector installed
- [ ] backup schedule configured (daily + weekly)
- [ ] restore tested at least once

## Application
- [ ] Deploy Rust API + worker services
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
