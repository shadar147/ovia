# Skill: Data Model Migrator

Goal: evolve PostgreSQL schema safely.

Rules:
- Write forward-only migrations.
- Preserve data and backwards compatibility where possible.
- Add indexes for expected query patterns.
- Add comments for non-obvious constraints.

Identity-specific checks:
- Keep many-to-many person/identity model.
- Keep temporal validity (`valid_from`, `valid_to`) semantics intact.
- Keep auditability for mapping changes.

DoD:
- Migration applies on clean DB.
- Migration applies on realistic existing DB.
- Query plan reviewed for key list/report queries.
