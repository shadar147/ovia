-- KPI Dashboard seed data
-- 12 weekly snapshots + 7 risk items for org 00000000-0000-0000-0000-000000000001
--
-- Story: team had a holiday dip (Dec), recovered strongly through Jan-Feb.
-- Health 62→78, risk 45→24, latency 8.2h→4.1h, throughput 32→48.

BEGIN;

-- Use a fixed org_id matching the frontend default
\set org_id '00000000-0000-0000-0000-000000000001'

-- Clean existing data for idempotent re-runs
DELETE FROM risk_items WHERE org_id = :'org_id';
DELETE FROM kpi_snapshots WHERE org_id = :'org_id';

-- Week 1: Nov 24-30 — baseline before holiday slowdown
INSERT INTO kpi_snapshots (id, org_id, period_start, period_end, delivery_health_score, release_risk_score, throughput_total, throughput_bugs, throughput_features, throughput_chores, review_latency_median_hours, review_latency_p90_hours, computed_at)
VALUES ('a0000000-0000-0000-0000-000000000001', :'org_id', '2025-11-24', '2025-11-30', 62.3, 45.0, 32, 8, 18, 6, 8.2, 18.5, '2025-11-30T23:00:00Z');

-- Week 2: Dec 1-7 — slight dip starting
INSERT INTO kpi_snapshots (id, org_id, period_start, period_end, delivery_health_score, release_risk_score, throughput_total, throughput_bugs, throughput_features, throughput_chores, review_latency_median_hours, review_latency_p90_hours, computed_at)
VALUES ('a0000000-0000-0000-0000-000000000002', :'org_id', '2025-12-01', '2025-12-07', 60.1, 46.5, 28, 7, 15, 6, 8.8, 19.2, '2025-12-07T23:00:00Z');

-- Week 3: Dec 8-14 — continued decline
INSERT INTO kpi_snapshots (id, org_id, period_start, period_end, delivery_health_score, release_risk_score, throughput_total, throughput_bugs, throughput_features, throughput_chores, review_latency_median_hours, review_latency_p90_hours, computed_at)
VALUES ('a0000000-0000-0000-0000-000000000003', :'org_id', '2025-12-08', '2025-12-14', 59.0, 47.0, 25, 6, 14, 5, 9.1, 20.0, '2025-12-14T23:00:00Z');

-- Week 4: Dec 15-21 — pre-holiday low
INSERT INTO kpi_snapshots (id, org_id, period_start, period_end, delivery_health_score, release_risk_score, throughput_total, throughput_bugs, throughput_features, throughput_chores, review_latency_median_hours, review_latency_p90_hours, computed_at)
VALUES ('a0000000-0000-0000-0000-000000000004', :'org_id', '2025-12-15', '2025-12-21', 57.5, 49.0, 20, 5, 11, 4, 9.5, 21.3, '2025-12-21T23:00:00Z');

-- Week 5: Dec 22-28 — holiday trough
INSERT INTO kpi_snapshots (id, org_id, period_start, period_end, delivery_health_score, release_risk_score, throughput_total, throughput_bugs, throughput_features, throughput_chores, review_latency_median_hours, review_latency_p90_hours, computed_at)
VALUES ('a0000000-0000-0000-0000-000000000005', :'org_id', '2025-12-22', '2025-12-28', 58.2, 48.0, 18, 4, 10, 4, 9.0, 20.8, '2025-12-28T23:00:00Z');

-- Week 6: Dec 29 - Jan 4 — slow recovery
INSERT INTO kpi_snapshots (id, org_id, period_start, period_end, delivery_health_score, release_risk_score, throughput_total, throughput_bugs, throughput_features, throughput_chores, review_latency_median_hours, review_latency_p90_hours, computed_at)
VALUES ('a0000000-0000-0000-0000-000000000006', :'org_id', '2025-12-29', '2026-01-04', 60.8, 44.0, 24, 6, 13, 5, 7.8, 17.5, '2026-01-04T23:00:00Z');

-- Week 7: Jan 5-11 — picking up pace
INSERT INTO kpi_snapshots (id, org_id, period_start, period_end, delivery_health_score, release_risk_score, throughput_total, throughput_bugs, throughput_features, throughput_chores, review_latency_median_hours, review_latency_p90_hours, computed_at)
VALUES ('a0000000-0000-0000-0000-000000000007', :'org_id', '2026-01-05', '2026-01-11', 64.5, 40.0, 34, 9, 18, 7, 6.8, 15.2, '2026-01-11T23:00:00Z');

-- Week 8: Jan 12-18 — strong recovery
INSERT INTO kpi_snapshots (id, org_id, period_start, period_end, delivery_health_score, release_risk_score, throughput_total, throughput_bugs, throughput_features, throughput_chores, review_latency_median_hours, review_latency_p90_hours, computed_at)
VALUES ('a0000000-0000-0000-0000-000000000008', :'org_id', '2026-01-12', '2026-01-18', 68.9, 35.0, 42, 10, 24, 8, 5.8, 13.0, '2026-01-18T23:00:00Z');

-- Week 9: Jan 19-25 — momentum building
INSERT INTO kpi_snapshots (id, org_id, period_start, period_end, delivery_health_score, release_risk_score, throughput_total, throughput_bugs, throughput_features, throughput_chores, review_latency_median_hours, review_latency_p90_hours, computed_at)
VALUES ('a0000000-0000-0000-0000-000000000009', :'org_id', '2026-01-19', '2026-01-25', 71.2, 31.0, 44, 11, 25, 8, 5.2, 11.8, '2026-01-25T23:00:00Z');

-- Week 10: Jan 26 - Feb 1 — sustained improvement
INSERT INTO kpi_snapshots (id, org_id, period_start, period_end, delivery_health_score, release_risk_score, throughput_total, throughput_bugs, throughput_features, throughput_chores, review_latency_median_hours, review_latency_p90_hours, computed_at)
VALUES ('a0000000-0000-0000-0000-000000000010', :'org_id', '2026-01-26', '2026-02-01', 73.6, 28.0, 45, 10, 27, 8, 4.9, 10.5, '2026-02-01T23:00:00Z');

-- Week 11: Feb 2-8 — nearing target
INSERT INTO kpi_snapshots (id, org_id, period_start, period_end, delivery_health_score, release_risk_score, throughput_total, throughput_bugs, throughput_features, throughput_chores, review_latency_median_hours, review_latency_p90_hours, computed_at)
VALUES ('a0000000-0000-0000-0000-000000000011', :'org_id', '2026-02-02', '2026-02-08', 75.8, 26.0, 46, 9, 29, 8, 4.5, 9.8, '2026-02-08T23:00:00Z');

-- Week 12: Feb 10-16 — current/latest (best week)
INSERT INTO kpi_snapshots (id, org_id, period_start, period_end, delivery_health_score, release_risk_score, throughput_total, throughput_bugs, throughput_features, throughput_chores, review_latency_median_hours, review_latency_p90_hours, computed_at)
VALUES ('a0000000-0000-0000-0000-000000000012', :'org_id', '2026-02-10', '2026-02-16', 78.4, 24.0, 48, 8, 31, 9, 4.1, 8.9, '2026-02-16T23:00:00Z');

-- Risk items for the latest snapshot (week 12)
INSERT INTO risk_items (id, org_id, snapshot_id, entity_type, title, owner, age_days, impact_scope, status, source_url)
VALUES
  ('b0000000-0000-0000-0000-000000000001', :'org_id', 'a0000000-0000-0000-0000-000000000012',
   'pull_request', 'Stale PR: Refactor auth middleware', 'alice.chen', 12, 'auth-service', 'open',
   'https://gitlab.example.com/ovia/backend/-/merge_requests/187'),

  ('b0000000-0000-0000-0000-000000000002', :'org_id', 'a0000000-0000-0000-0000-000000000012',
   'issue', 'Blocker: Database migration deadlock on deploy', NULL, 5, 'infra', 'blocked',
   'https://jira.example.com/browse/OV-342'),

  ('b0000000-0000-0000-0000-000000000003', :'org_id', 'a0000000-0000-0000-0000-000000000012',
   'pipeline', 'CI pipeline failing on integration tests', 'bob.smith', 3, 'ci-cd', 'failing',
   'https://gitlab.example.com/ovia/backend/-/pipelines/8921'),

  ('b0000000-0000-0000-0000-000000000004', :'org_id', 'a0000000-0000-0000-0000-000000000012',
   'issue', 'Unassigned: Performance regression in search API', NULL, 8, 'search-service', 'open',
   'https://jira.example.com/browse/OV-338'),

  ('b0000000-0000-0000-0000-000000000005', :'org_id', 'a0000000-0000-0000-0000-000000000012',
   'pull_request', 'Stale PR: Add rate limiting to public endpoints', 'carol.davis', 15, 'api-gateway', 'open',
   'https://gitlab.example.com/ovia/backend/-/merge_requests/172'),

  ('b0000000-0000-0000-0000-000000000006', :'org_id', 'a0000000-0000-0000-0000-000000000012',
   'issue', 'Bug: Incorrect timezone handling in reports', 'dave.kumar', 6, 'reporting', 'in_progress',
   'https://jira.example.com/browse/OV-345'),

  ('b0000000-0000-0000-0000-000000000007', :'org_id', 'a0000000-0000-0000-0000-000000000012',
   'pull_request', 'Large PR: Identity resolution v2 merge', 'alice.chen', 9, 'identity-service', 'review_needed',
   'https://gitlab.example.com/ovia/backend/-/merge_requests/181');

COMMIT;
