# Ovia autonomous execution mode

When owner is away:
- Continue through backlog from top priority to lower.
- Ask owner only on critical blockers:
  1) missing credentials/secrets
  2) irreversible infra action
  3) conflicting product requirement
- Otherwise proceed and log decisions in `docs/.planning/decision-log.md`.
