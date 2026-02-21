create table if not exists kpi_snapshots (
  id uuid primary key default gen_random_uuid(),
  org_id uuid not null,
  period_start date not null,
  period_end date not null,
  delivery_health_score numeric(5,2),
  release_risk_score numeric(5,2),
  throughput_total integer not null default 0,
  throughput_bugs integer not null default 0,
  throughput_features integer not null default 0,
  throughput_chores integer not null default 0,
  review_latency_median_hours numeric(8,2),
  review_latency_p90_hours numeric(8,2),
  computed_at timestamptz not null default now(),
  created_at timestamptz not null default now()
);

create unique index if not exists kpi_snapshots_org_period_uidx
  on kpi_snapshots(org_id, period_start, period_end);

create table if not exists risk_items (
  id uuid primary key default gen_random_uuid(),
  org_id uuid not null,
  snapshot_id uuid not null references kpi_snapshots(id) on delete cascade,
  entity_type text not null,
  title text not null,
  owner text,
  age_days integer not null default 0,
  impact_scope text,
  status text not null,
  source_url text,
  created_at timestamptz not null default now()
);

create index if not exists risk_items_snapshot_idx on risk_items(snapshot_id);
