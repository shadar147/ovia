-- Ovia identity model v2

create table if not exists people (
  id uuid primary key,
  org_id uuid not null,
  display_name text not null,
  primary_email text,
  team text,
  role text,
  status text not null default 'active',
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table if not exists identities (
  id uuid primary key,
  org_id uuid not null,
  source text not null,
  external_id text,
  username text,
  email text,
  display_name text,
  is_service_account boolean not null default false,
  first_seen_at timestamptz,
  last_seen_at timestamptz,
  raw_ref jsonb,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create unique index if not exists identities_source_external_uidx
  on identities(org_id, source, external_id)
  where external_id is not null;

create table if not exists person_identity_links (
  id uuid primary key,
  org_id uuid not null,
  person_id uuid not null references people(id) on delete cascade,
  identity_id uuid not null references identities(id) on delete cascade,
  status text not null,
  confidence numeric(4,3) not null default 0,
  rule_trace jsonb,
  valid_from timestamptz,
  valid_to timestamptz,
  verified_by text,
  verified_at timestamptz,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create unique index if not exists person_identity_unique_active
  on person_identity_links(org_id, person_id, identity_id, coalesce(valid_to, 'infinity'::timestamptz));

create table if not exists identity_events (
  id uuid primary key,
  org_id uuid not null,
  link_id uuid not null references person_identity_links(id) on delete cascade,
  action text not null,
  actor text,
  payload jsonb,
  created_at timestamptz not null default now()
);
