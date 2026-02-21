create table if not exists ask_sessions (
  id uuid primary key default gen_random_uuid(),
  org_id uuid not null,
  query text not null,
  answer text,
  confidence text,  -- high | medium | low
  assumptions text,
  citations jsonb,  -- array of {source, url, excerpt}
  filters jsonb,    -- {team, product, date_range, sources}
  model text,
  prompt_tokens integer,
  completion_tokens integer,
  latency_ms integer,
  created_at timestamptz not null default now()
);

create index if not exists ask_sessions_org_idx on ask_sessions(org_id, created_at desc);
