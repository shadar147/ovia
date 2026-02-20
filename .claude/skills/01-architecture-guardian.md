# Skill: Architecture Guardian

Goal: keep implementation aligned with Ovia architecture and scope.

Must enforce:
- Keep services separated by concern: API, ingest, metrics, rag, scheduler.
- Treat identity model v2 as source of truth (`people`, `identities`, `person_identity_links`, `identity_events`).
- Keep write access off by default for external systems.
- Keep every insight answer citation-backed.

DoD:
- Changes map to existing docs or include doc updates.
- No architecture shortcuts without explicit ADR note.
