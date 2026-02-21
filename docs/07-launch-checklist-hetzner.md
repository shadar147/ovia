# Launch Checklist (Hetzner + Docker Swarm)

Legend: `[x]` = code/config ready, `[ ]` = requires manual ops action at deploy time.

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
- [x] Create overlay networks — defined in `docker-compose.swarm.yml`
- [ ] Create Swarm secrets for Jira/GitLab/Confluence/API providers
- [x] Deploy stack files — `docker-compose.swarm.yml` ready

## Security
- [ ] UFW + fail2ban
- [ ] SSH hardening (keys only, no password)
- [ ] service users + least privilege
- [ ] rotate tokens and secrets policy

## Data services
- [x] PostgreSQL configured — compose service + 4 SQL migrations
- [x] backup schedule configured (daily + weekly) — `backup.sh` + backup service in compose
- [x] restore scripts ready — `restore.sh` + `verify-backup.sh`
- [ ] restore tested at least once on real data

## Application
- [x] Rust API + worker services — Dockerfile builds 5 binaries, compose deploys all
- [x] Connectors implemented (Jira/GitLab/Confluence) — watermark sync, retry/backoff
- [ ] Configure connector credentials (`.env`)
- [ ] Run initial backfill
- [ ] Enable incremental sync schedules

## Intelligence
- [x] KPI query service — `compute_delivery_health`, `compute_release_risk`
- [x] Ask API with citations — stub engine, ready for LLM integration
- [ ] Embeddings pipeline enabled
- [ ] RAG retrieval validated with real LLM
- [ ] Top management questions validated

## Operations
- [x] Monitoring dashboards — Grafana 9-panel dashboard provisioned
- [x] Alert rules configured — 6 Prometheus alert rules
- [x] Log aggregation — Loki + Promtail configured
- [x] Backup/restore runbook — `docs/15-backup-restore-runbook.md`
- [ ] Incident runbook documented
