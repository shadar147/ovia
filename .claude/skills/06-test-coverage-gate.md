# Skill: Test Coverage Gate

Goal: keep quality bar high.

Minimums:
- New modules require tests.
- Bug fixes require regression tests.
- Critical paths require integration tests.
- Every API endpoint requires at least one happy-path and one error-path test.
- Complex business logic (scoring, matching, state transitions) requires deterministic unit tests covering edge cases.

Suggested thresholds (initial):
- Line coverage >= 80%
- Branch coverage >= 70%

PR fails if:
- Tests missing for changed behavior.
- Flaky tests introduced.
- Coverage drops without approved reason.

DoD:
- Test report attached in PR summary.
