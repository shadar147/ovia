-- Add avatar_url column to people table
alter table people add column if not exists avatar_url text;
