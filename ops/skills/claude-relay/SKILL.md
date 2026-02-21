---
name: claude-relay
description: Delegate coding tasks to Claude CLI from the current repository, enforce clean code + tests, and return concise execution reports with changed files, checks, and risks. Use when user asks to have Claude implement code while this agent reviews/coordinates.
---

# Claude Relay Skill

## Objective
Delegate implementation to Claude CLI and keep this agent as coordinator/reviewer.

## Workflow
1. Confirm task scope in one short sentence.
2. Build execution prompt with:
   - context files to read first
   - acceptance criteria
   - required checks (`fmt`, `clippy`, `test`)
   - required output report path
3. Run Claude in repo root with non-interactive mode.
4. Review produced diff and run checks locally.
5. Commit/push only when checks pass.
6. Reply to user with commit hash and short summary.

## Prompt template
Use this structure when delegating:

- Task:
- Constraints:
- Files to read first:
- Acceptance criteria:
- Required checks:
- Required report file:

## Quality gate
Never mark task done unless all are true:
- code compiles
- tests pass
- formatting/lints pass
- report exists with risks/follow-ups

## Reporting format to user
- commit: `<hash> — <title>`
- done: bullet list
- checks: pass/fail
- risks: bullet list
- next: one line
