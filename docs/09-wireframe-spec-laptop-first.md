# Ovia — Wireframe Spec (Laptop-first)

Goal: define implementation-ready wireframes for laptop screens first.

## Target viewport
- Primary: **1440 x 900**
- Secondary: **1366 x 768**
- Grid: 12 columns, 24px gutters, 80px side margins (desktop)
- Base spacing scale: 4 / 8 / 12 / 16 / 24 / 32

## Global layout
- Top bar height: 64px
- Left navigation width: 240px (collapsed 72px)
- Main content max width: 1200–1280px
- Right context panel (optional): 360px

## Global states (all screens)
- Loading: skeleton cards/tables
- Empty: contextual CTA + short explanation
- Error: retry + diagnostics id
- Permission denied: scoped explanation + request-access CTA

---

## Screen A — Dashboard (Exec)

### Layout zones
1. Header row (filters/actions)
   - Date range picker
   - Product selector (All / Mobile A/B/C)
   - Refresh / Export
2. KPI row (4 cards)
   - Delivery Health
   - Release Risk
   - Throughput
   - MR Review Latency
3. Mid row (2-column)
   - Left: trend charts (velocity, cycle time)
   - Right: blockers and stale MR lists
4. Bottom row
   - Narrative insight panel ("what changed + why")
   - Source citations links

### Interaction rules
- KPI card click filters lower widgets
- Chart point click drills to Team/Ask views
- All narrative statements must have source link anchors

---

## Screen B — Team Identity Mapping (critical)

### Layout zones
1. Header
   - Status filters (all/unmatched/conflict/auto/verified)
   - Team/product filter
   - Confidence slider
   - Bulk action button
2. Main table (sticky header)
   - Canonical person
   - GitLab account
   - Jira account
   - Confluence account
   - Match status
   - Confidence
   - Actions
3. Right-side detail drawer (on row select)
   - Match rationale
   - Candidate alternatives
   - Activity preview by source
   - Audit history

### Interaction rules
- Row select opens drawer without route change
- Confirm/remap/split updates status instantly (optimistic UI + rollback on fail)
- Bulk confirm enabled only for confidence threshold + no conflicts

### Table sizing (laptop)
- Default page size: 25 rows
- Row height: 52px
- Sticky first column for person identity

---

## Screen C — Team Person 360

### Layout zones
1. Header
   - Person identity card (canonical + mapped accounts)
   - Date range + product scope
2. Metrics row
   - Jira contribution metrics
   - GitLab contribution metrics
   - Confluence contribution metrics
3. Timeline panel (full width)
   - Unified chronological events feed
4. Insights panel
   - Workload anomalies
   - Hidden blocker hints

### Interaction rules
- Clicking timeline item deep-links to source object (issue/MR/page)
- Source toggles filter timeline by system

---

## Screen D — Ask Ovia

### Layout zones
1. Query box
   - Natural language input
   - Data source chips (Jira/GitLab/Confluence/All)
   - Date/team/product chips
2. Answer panel
   - Final answer
   - Confidence level and assumptions
   - Suggested follow-up prompts
3. Sources panel
   - Citation list with relevance score
   - Expand to raw excerpt

### Interaction rules
- Enter sends question, Shift+Enter new line
- Every answer requires at least one citation block
- "Show SQL/logic" visible for analytics-type answers

---

## Screen E — Reports

### Layout zones
- Report templates list (weekly exec/team/release readiness/identity quality)
- Schedule editor
- Last run status
- Delivery channels (Telegram/Slack/email)

### Interaction rules
- Dry-run preview before enabling schedule
- Failed reports show actionable error summary

---

## Screen F — Settings

### Sections
- Connections (Jira/GitLab/Confluence)
- Sync schedules
- Permissions/roles
- Alerts routing
- Retention policies

### Interaction rules
- Connection tests before save
- Secret fields masked and non-retrievable

---

## Component spec (shared)
- Data table (sortable, filterable, column hide/show)
- Metric card (value, delta, sparkline)
- Insight card (statement + evidence links)
- Source chip (system icon + label)
- Confidence badge (`high`/`medium`/`low`)

## Accessibility baseline
- Keyboard-navigable tables and drawers
- Visible focus states
- Color contrast WCAG AA
- Non-color status indicators (icons/text)

## Performance targets (laptop)
- Initial dashboard render: < 2.5s on warm cache
- Identity table filter response: < 500ms
- Ask Ovia first token: < 2.0s (best effort)

## Scope note
This spec is **laptop-first only**. Tablet/mobile adaptive behaviors are intentionally postponed to next phase.
