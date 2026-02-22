# CLAUDE.md — Ovia execution policy

Use these local skills from `.claude/skills/`.

Global standards (always apply):
1. Keep code clean, explicit, and production-oriented.
2. Write tests for every new behavior and bug fix.
3. Do not merge if tests fail.
4. Preserve architecture constraints from `docs/03-architecture.md` and identity model v2 from `docs/12-identity-model-v2.md`.
5. Prefer small PRs with clear acceptance criteria.

Execution flow for every task:
- Read task + acceptance criteria.
- Select one primary skill and one reviewer skill.
- Implement.
- Run lint/tests.
- Self-review against DoD checklist.
- Update task status in `docs/13-delivery-backlog.md` and check off completed microtasks in `docs/14-microtasks-5-10min.md`.

## Current roadmap priority (agreed 2026-02-21)
1. ~~**GitLab MRs sync**~~ — **DONE** (OVIA-3004). 104 projects, 3806 MRs, 4785 pipelines synced. Real KPI computed.
2. **Dashboard + mock data** — KPI dashboard with charts. Backend API ready (`GET /team/kpi`), frontend needs to wire real data.
3. **Jira Issues sync** — tasks, sprints, statuses, cycle time (needs JIRA_BASE_URL, JIRA_EMAIL, JIRA_API_TOKEN). Will unlock `blocker_count` and `spillover_rate` in KPI.
4. **Person 360 page** — individual profile with all identities and activity

## Session notes (2026-02-21)
- `.env` lives in project root AND `backend/` (copied for cargo). Contains real GitLab creds for gitlab.beeezone.com.
- `.env` is NOT in git (secrets). `.env.example` IS committed in `backend/`.
- DB runs in Docker container `ovia-postgres` (user: ovia, db: ovia, pass: ovia_dev_password, port: 5432).
- `psql` not installed on host — use `docker exec ovia-postgres psql -U ovia -d ovia` instead.
- CI uses `SQLX_OFFLINE=true` with `.sqlx/` query cache. Run `cargo sqlx prepare --workspace` after changing any `sqlx::query_as!` macros.
- KPI throughput_bugs/features = 0 because GitLab MRs have no `bug`/`feature` labels yet. All count as chores.
- Review latency P90 is high (~410h) due to long-lived MRs. 232 stale MR risk items + 20 failed pipeline risk items generated.
- Pre-existing `ovia-config` tests fail locally (env-dependent) — this is expected, not a regression.
