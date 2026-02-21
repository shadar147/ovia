#!/usr/bin/env bash
set -euo pipefail

if ! command -v claude >/dev/null 2>&1; then
  echo "claude CLI not found in PATH" >&2
  exit 1
fi

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 '<task text>'" >&2
  exit 1
fi

TASK="$1"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPORT_DIR="$REPO_ROOT/docs/.planning"
REPORT_FILE="$REPORT_DIR/CLAUDE-RELAY-REPORT.md"
mkdir -p "$REPORT_DIR"

PROMPT=$(cat <<EOF
You are executing a coding task in repository: $REPO_ROOT

Task:
$TASK

Constraints:
- Keep changes minimal and clean.
- Follow existing architecture/docs.
- Add/update tests for changed behavior.

Files to read first:
- CLAUDE.md
- .claude/skills/00-skill-index.md
- docs/13-delivery-backlog.md
- docs/14-microtasks-5-10min.md

Acceptance criteria:
- Task implemented end-to-end.
- Relevant tests added/updated.
- No unrelated refactors.

Required checks:
- cargo fmt --all --check
- cargo clippy --all-targets --all-features -- -D warnings
- cargo test --all

Required report file:
- $REPORT_FILE
Include:
1) What changed
2) Files changed
3) Checks run + results
4) Risks/follow-ups
EOF
)

cd "$REPO_ROOT"
claude -p --permission-mode dontAsk --allowedTools Write,Bash,Read,Edit "$PROMPT"

echo "Claude relay finished. Report: $REPORT_FILE"
