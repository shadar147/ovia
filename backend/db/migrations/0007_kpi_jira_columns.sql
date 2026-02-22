-- Add Jira-derived KPI columns to kpi_snapshots (Block 3A)
alter table kpi_snapshots add column if not exists blocker_count integer not null default 0;
alter table kpi_snapshots add column if not exists spillover_rate numeric(5,4);
alter table kpi_snapshots add column if not exists cycle_time_p50_hours numeric(8,2);
alter table kpi_snapshots add column if not exists cycle_time_p90_hours numeric(8,2);
