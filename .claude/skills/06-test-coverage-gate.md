# Skill: Test Coverage Gate

Goal: keep quality bar high.

Minimums:
- New modules require tests.
- Bug fixes require regression tests.
- Critical paths require integration tests.

Suggested thresholds (initial):
- Line coverage >= 80%
- Branch coverage >= 70%

PR fails if:
- Tests missing for changed behavior.
- Flaky tests introduced.
- Coverage drops without approved reason.

DoD:
- Test report attached in PR summary.
