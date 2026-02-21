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
