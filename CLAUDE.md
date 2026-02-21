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
1. **Dashboard + mock data** — KPI dashboard with charts on generated data
2. **Jira Issues sync** — tasks, sprints, statuses, cycle time (needs JIRA_BASE_URL, JIRA_EMAIL, JIRA_API_TOKEN)
3. **GitLab MRs sync** — merge requests, reviews, CI/CD (needs GITLAB_URL, GITLAB_TOKEN)
4. **Person 360 page** — individual profile with all identities and activity
