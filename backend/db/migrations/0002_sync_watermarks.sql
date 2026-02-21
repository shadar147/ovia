-- Sync watermarks for incremental connector syncs

create table if not exists sync_watermarks (
  id uuid primary key default gen_random_uuid(),
  org_id uuid not null,
  source text not null,
  last_synced_at timestamptz,
  cursor_value text,
  status text not null default 'idle',  -- idle | running | failed
  error_message text,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create unique index if not exists sync_watermarks_org_source_uidx
  on sync_watermarks(org_id, source);
