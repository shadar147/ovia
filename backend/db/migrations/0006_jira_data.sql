-- Jira issues and status transitions for cycle-time and analytics.

create table if not exists jira_issues (
  id uuid primary key default gen_random_uuid(),
  org_id uuid not null,
  jira_key text not null,           -- e.g. "BEE-123"
  project_key text not null,        -- e.g. "BEE"
  issue_type text,                  -- "Story", "Bug", "Task", etc.
  summary text not null,
  status text not null,             -- current status
  assignee_account_id text,
  reporter_account_id text,
  priority text,
  story_points real,                -- customfield_10016
  sprint_name text,                 -- latest sprint from customfield_10020
  sprint_id bigint,                 -- latest sprint id
  team_name text,                   -- customfield_10001
  labels text[] not null default '{}',
  created_at_jira timestamptz,
  updated_at_jira timestamptz,
  resolved_at timestamptz,
  raw_ref jsonb,
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create unique index if not exists jira_issues_org_key_uidx
  on jira_issues(org_id, jira_key);

create index if not exists jira_issues_org_project_idx
  on jira_issues(org_id, project_key);

create index if not exists jira_issues_org_status_idx
  on jira_issues(org_id, status);

-- Status transitions extracted from issue changelog.
create table if not exists jira_issue_transitions (
  id uuid primary key default gen_random_uuid(),
  org_id uuid not null,
  jira_key text not null,
  field text not null,              -- "status" or "sprint"
  from_value text,
  to_value text,
  author_account_id text,
  transitioned_at timestamptz not null,
  created_at timestamptz not null default now()
);

create index if not exists jira_transitions_org_key_idx
  on jira_issue_transitions(org_id, jira_key);

create index if not exists jira_transitions_org_field_idx
  on jira_issue_transitions(org_id, field, transitioned_at);
