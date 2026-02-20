# Skill: PR Review Gatekeeper

Goal: enforce consistent merge quality.

Review checklist:
- Architecture fit: yes/no
- Data model correctness: yes/no
- Security/secrets handling: yes/no
- Test completeness: yes/no
- Performance risk checked: yes/no
- Observability added: yes/no
- Docs updated: yes/no

Reject PR if any critical item is no.

Required PR summary format:
1. What changed
2. Why
3. Risks
4. How tested
5. Rollback plan
