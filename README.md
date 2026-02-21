# Ovia

[![CI](https://github.com/shadar147/ovia/actions/workflows/ci.yml/badge.svg)](https://github.com/shadar147/ovia/actions/workflows/ci.yml)

Platform concept for engineering intelligence across Jira, GitLab, and Confluence (Rust backend, Docker Swarm deploy).

## Repo layout
- `backend/` — all backend code, migrations, and local infra
- `docs/` — product/architecture planning docs
- `prototype/` — clickable product prototype
- `design/` — wireframes and visual assets

## Backend quick start
```bash
cd backend
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

## Key docs
- `docs/01-business-vision.md`
- `docs/02-solution-overview.md`
- `docs/03-architecture.md`
- `docs/04-modules.md`
- `docs/05-tech-stack.md`
- `docs/06-mvp-plan.md`
- `docs/07-launch-checklist-hetzner.md`
- `docs/08-ui-prototype.md`
- `docs/09-wireframe-spec-laptop-first.md`
- `docs/10-user-flows.md`
- `docs/11-screen-details-v2.md`
- `docs/12-identity-model-v2.md`
- `docs/13-delivery-backlog.md`
- `docs/14-microtasks-5-10min.md`
- `docs/15-identity-query-plan.md`

## Backend references
- `backend/db/migrations/0001_identity_v2.sql`
- `backend/infra/docker-compose.dev.yml`
- `prototype/v1/README.md`
- `ops/skills/claude-relay/SKILL.md`
- `backend/scripts/claude-relay.sh`
