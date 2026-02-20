# Ovia — User Flows (Laptop-first)

## 1) CTO flow — Weekly executive review (10–15 min)
**Goal:** quickly understand delivery health and release risk across 3 mobile apps.

1. Open **Dashboard**
2. Set range to `Last 14 days`, scope `All products`
3. Check KPI cards: Delivery Health, Release Risk, Throughput, Review Latency
4. Open blockers widget and inspect top aging blockers
5. Open narrative insights and validate evidence links
6. Export weekly summary to Telegram/Slack

**Success criteria:** CTO can answer “Are we on track?” in < 10 minutes.

---

## 2) Engineering Manager flow — Resolve identity conflicts
**Goal:** ensure cross-system teammate mapping is reliable.

1. Open **Team → Identity Mapping**
2. Filter status `conflict` + confidence `< 0.85`
3. Review each row in right-side drawer
4. Confirm/remap/split ambiguous matches
5. Run bulk confirm for confidence `> 0.95`
6. Export unresolved list and assign owners

**Success criteria:** manager resolves most conflicts in one pass (< 10 min).

---

## 3) PM flow — Release readiness check
**Goal:** identify blockers and hidden risks for release X.

1. Dashboard scope `Mobile B`, range `current sprint`
2. Open `stale MRs` + `pipeline failures` widgets
3. Jump to Team Person 360 for top-risk contributors
4. Ask Ovia: “What blocks release X and what changed this week?”
5. Review answer citations and share report link

**Success criteria:** PM gets action list with owners and evidence.

---

## 4) Tech Lead flow — Person 360 review
**Goal:** understand a teammate’s cross-system contribution.

1. Team page → open person row
2. Inspect Jira/GitLab/Confluence metrics
3. Open unified timeline for last 30 days
4. Filter by source to isolate anomalies
5. Create follow-up question in Ask Ovia

**Success criteria:** lead can explain “who does what” with data.

---

## 5) Ops flow — Scheduled report reliability
**Goal:** ensure weekly reports deliver on time.

1. Open Reports
2. Inspect last run statuses
3. Open failed run diagnostics, retry dry-run
4. Validate delivery channels
5. Confirm next schedule and alert thresholds

**Success criteria:** report failure triaged in < 5 minutes.

---

## 6) Q&A flow — Evidence-first answer
**Goal:** answer management question with verifiable sources.

1. Open Ask Ovia
2. Enter question + source filters
3. Review answer + confidence + assumptions
4. Open citations and verify source excerpts
5. Save answer as report snippet

**Success criteria:** every decision-facing answer includes traceable citations.
