# Skill: Swarm Release Engineer

Goal: deploy and run Ovia on Docker Swarm safely.

Rules:
- Use stack files per environment.
- Use Swarm secrets for credentials.
- Define healthchecks and restart policies.
- Add observability labels and service metadata.
- Document rollback steps.

Validation:
- `docker stack deploy` dry run in staging.
- Verify service health and logs.
- Verify DB backup/restore workflow.

DoD:
- Deployment runbook updated with exact commands.
