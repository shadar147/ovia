export const messages = {
  // ── Navigation ──
  "nav.dashboard": "Dashboard",
  "nav.identityMapping": "Identity Mapping",
  "nav.askOvia": "Ask Ovia",
  "nav.reports": "Reports",
  "nav.settings": "Settings",

  // ── Topbar ──
  "topbar.org": "org: {org}",

  // ── Dashboard ──
  "dashboard.title": "Dashboard",
  "dashboard.loading": "Loading metrics...",
  "dashboard.weekOf": "Week of {period}",
  "dashboard.failedToLoad": "Failed to load KPI data",

  // ── Dashboard charts ──
  "dashboard.throughputTrend": "Throughput Trend",
  "dashboard.throughputTrendDesc": "Completed items per week, stacked by type: features, bugs, chores",
  "dashboard.latencyTrend": "Review Latency Trend",
  "dashboard.latencyTrendDesc": "Time from MR opened to first review. Median (solid) and P90 (dashed)",
  "dashboard.healthOverTime": "Delivery Health Over Time",
  "dashboard.healthOverTimeDesc": "Composite score (0-100) = 30% throughput + 30% review latency + 20% blocker count + 20% spillover rate. Above 80 = Healthy, 60-80 = At Risk, below 60 = Critical",
  "dashboard.topRisks": "Top Risks & Blockers",
  "dashboard.topRisksDesc": "Active blockers, stale PRs, and failing pipelines from the latest snapshot. Risk score = 40% blocker age + 30% failing pipelines + 30% stale MR %",
  "dashboard.throughputMix": "Throughput Mix",
  "dashboard.throughputMixDesc": "Current week breakdown by work item type",

  // ── KPI cards ──
  "kpi.deliveryHealth": "Delivery Health",
  "kpi.noData": "No data",
  "kpi.releaseRisk": "Release Risk",
  "kpi.lowerIsBetter": "Lower is better",
  "kpi.throughput": "Throughput",
  "kpi.itemsDelivered": "items delivered",
  "kpi.reviewLatency": "Review Latency",
  "kpi.vsPrevWeek": "vs prev week",

  // ── KPI descriptions ──
  "kpi.deliveryHealthDesc": "Weighted score 0-100:\n30% Throughput (capped at 100 items)\n30% Review Latency (0h=100, 48h+=0)\n20% Blockers (0=100, 10+=0)\n20% Spillover Rate (0%=100, 100%=0)",
  "kpi.releaseRiskDesc": "Weighted score 0-100:\n40% Blocker age (count x10 + total days x0.5)\n30% Failing pipelines (each = 20 pts)\n30% Stale MR percentage\nLabels: <35 Low, 35-70 Medium, >70 High",
  "kpi.throughputDesc": "Total completed work items per period.\nBreakdown: features + bugs + chores.\nSource: closed Jira issues / MRs merged.",
  "kpi.latencyDesc": "Median time from MR opened to first review.\nP90 = 90th percentile (worst 10% of reviews).\nTarget: median <4h, P90 <12h.",

  // ── Health ──
  "health.healthy": "Healthy",
  "health.atRisk": "At Risk",
  "health.critical": "Critical",

  // ── Chart legends ──
  "chart.features": "Features",
  "chart.bugs": "Bugs",
  "chart.chores": "Chores",
  "chart.median": "Median",
  "chart.p90": "P90",
  "chart.deliveryHealth": "Delivery Health",
  "chart.hours": "Hours",
  "chart.healthyThreshold": "Healthy (80)",
  "chart.atRiskThreshold": "At Risk (60)",

  // ── Risk table ──
  "risk.type": "Type",
  "risk.title": "Title",
  "risk.owner": "Owner",
  "risk.age": "Age",
  "risk.status": "Status",
  "risk.noRisks": "No risks detected — great work!",
  "risk.unassigned": "Unassigned",

  // ── Identity page ──
  "identity.title": "Identity Mapping",
  "identity.subtitle": "Match identities across Jira, GitLab, and Confluence to canonical people.",
  "identity.bulkConfirm": "Bulk Confirm ({count})",
  "identity.exportCsv": "Export CSV",
  "identity.noMappings": "No mappings",
  "identity.noMappingsDesc": "Run a sync first to populate identity mappings.",
  "identity.noAutoLinks": "No auto-matched links to confirm",
  "identity.remapSoon": "Remap modal coming soon",
  "identity.exportFailed": "Failed to export CSV",
  "identity.failedToLoad": "Failed to load mappings",

  // ── Identity filters ──
  "identity.filterStatus": "Status",
  "identity.filterAll": "All",
  "identity.filterConfidence": "Confidence: {min}% – {max}%",
  "identity.filterReset": "Reset",

  // ── Identity table ──
  "identity.colPerson": "Person",
  "identity.colIdentity": "Identity",
  "identity.colStatus": "Status",
  "identity.colConfidence": "Confidence",
  "identity.colUpdated": "Updated",
  "identity.noMappingsFound": "No mappings found.",
  "identity.totalCount": "{count} total",
  "identity.previous": "Previous",
  "identity.next": "Next",

  // ── Identity drawer ──
  "identity.drawerTitle": "Mapping Details",
  "identity.person": "Person",
  "identity.identity": "Identity",
  "identity.matchRationale": "Match Rationale",
  "identity.totalConfidence": "Total confidence",
  "identity.audit": "Audit",
  "identity.created": "Created: {date}",
  "identity.updated": "Updated: {date}",
  "identity.confirm": "Confirm",
  "identity.remap": "Remap",
  "identity.split": "Split",
  "identity.unknown": "Unknown",
  "identity.serviceAccount": "Service Account",

  // ── Status badges ──
  "status.auto": "Auto",
  "status.verified": "Verified",
  "status.conflict": "Conflict",
  "status.rejected": "Rejected",
  "status.split": "Split",

  // ── State components ──
  "state.error": "Error",
  "state.somethingWrong": "Something went wrong.",
  "state.retry": "Retry",
  "state.noData": "No data",
  "state.nothingToShow": "There's nothing to show here yet.",

  // ── Stub pages ──
  "ask.title": "Ask Ovia",
  "ask.description": "Ask questions with citations — coming in Phase 4.",
  "reports.title": "Reports",
  "reports.description": "Report templates and scheduling — coming in Phase 5.",
  "settings.title": "Settings",
  "settings.description": "Connections, sync policy, roles — coming in Phase 5.",
} as const;
