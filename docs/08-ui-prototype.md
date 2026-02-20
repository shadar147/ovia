# UI Prototype (v0) — Ovia

Goal: validate product expectations before implementation.

## Navigation
- Dashboard
- Team
- Ask Ovia
- Reports
- Settings

---

## 1) Dashboard (Exec View)

### Header
- Date range: `Last 30 days` (switcher)
- Scope: `All products` / `Mobile A` / `Mobile B` / `Mobile C`

### KPI Row
- Delivery Health Score
- Release Risk Score
- Throughput (issues done / week)
- Review Latency (MR median)

### Risk & Blockers
- Top blockers by aging
- Pipelines failing > N times
- Stale MRs linked to roadmap epics

### Narrative Insight Card
- “What changed this week”
- “Why it changed” (with source links)

---

## 2) Team → Identity Mapping (new critical screen)

Purpose: match one teammate across Jira/GitLab/Confluence identities.

### Filters
- Status: `all | unmatched | conflict | auto | verified`
- Team / Product / Role
- Confidence range slider

### Table
Columns:
1. Canonical person (name, email)
2. GitLab account
3. Jira account
4. Confluence account
5. Match status (`auto`, `verified`, `conflict`, `unmatched`)
6. Confidence score
7. Actions (`Confirm`, `Remap`, `Split`, `Ignore`)

### Right-side panel (on row click)
- Why matched (rules triggered)
- Similar candidates
- Activity preview by source
- Change history (audit)

### Bulk actions
- Confirm all with confidence > 0.95
- Export unresolved conflicts

---

## 3) Team → Person 360 View

For selected person:
- Jira: issues created/resolved, blockers, SLA
- GitLab: commits, MRs, review turnaround, pipeline impact
- Confluence: pages/updates/comments
- Cross-system timeline (single chronological feed)

Useful actions:
- “Show workload anomalies”
- “Show work not reflected in Jira”
- “Show potential hidden blockers”

---

## 4) Ask Ovia (Q&A)

### Input
- Free text question
- Context chips: `Jira`, `GitLab`, `Confluence`, `All`
- Filter chips: product/team/date

### Output
- Direct answer
- Confidence and assumptions
- Source citations with deep links
- Follow-up suggestions

Example:
- “Who drives Mobile B release and where are delays coming from?”

---

## 5) Reports
- Weekly exec summary
- Team digest
- Release readiness report
- Identity mapping quality report

---

## 6) Settings
- Source connections
- Sync schedules
- Role permissions
- Alerts routing (Telegram/Slack/email)

---

## UX acceptance criteria for prototype
- A manager can resolve identity conflicts in <10 minutes
- A person’s cross-system contribution is visible in one screen
- Every answer in Q&A contains traceable citations
- Dashboard explains trend shifts, not just numbers
