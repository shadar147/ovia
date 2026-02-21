-- GitLab projects, merge requests, and pipelines for KPI computation.

create table if not exists gitlab_projects (
  id uuid primary key default gen_random_uuid(),
  org_id uuid not null,
  gitlab_id bigint not null,
  name text not null,
  path_with_namespace text not null,
  web_url text not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create unique index if not exists gitlab_projects_org_gl_uidx
  on gitlab_projects(org_id, gitlab_id);

create table if not exists gitlab_merge_requests (
  id uuid primary key default gen_random_uuid(),
  org_id uuid not null,
  gitlab_project_id bigint not null,
  gitlab_mr_iid bigint not null,
  title text not null,
  state text not null,
  author_username text,
  labels text[] not null default '{}',
  created_at_gl timestamptz,
  merged_at timestamptz,
  web_url text not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create unique index if not exists gitlab_mrs_org_proj_iid_uidx
  on gitlab_merge_requests(org_id, gitlab_project_id, gitlab_mr_iid);

create table if not exists gitlab_pipelines (
  id uuid primary key default gen_random_uuid(),
  org_id uuid not null,
  gitlab_project_id bigint not null,
  gitlab_pipeline_id bigint not null,
  status text not null,
  ref_name text,
  created_at_gl timestamptz,
  finished_at_gl timestamptz,
  duration_secs integer,
  web_url text not null,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create unique index if not exists gitlab_pipelines_org_gl_uidx
  on gitlab_pipelines(org_id, gitlab_pipeline_id);
