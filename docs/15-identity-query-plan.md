# Identity Query Plan (OVIA-1001)

## Primary reviewer queries

### 1) Conflict queue (default sort)
```sql
select
  pil.id,
  pil.person_id,
  pil.identity_id,
  pil.status,
  pil.confidence,
  pil.created_at
from person_identity_links pil
where pil.org_id = $1
  and pil.valid_to is null
  and pil.status = 'conflict'
order by pil.confidence asc, pil.created_at desc
limit $2 offset $3;
```

Expected index: `person_identity_links_conflict_queue_idx`

### 2) Candidate lookup by source+email
```sql
select id, org_id, source, email, username, display_name
from identities
where org_id = $1
  and source = $2
  and email = $3;
```

Expected index: `identities_source_email_idx`

### 3) Candidate lookup by source+username
```sql
select id, org_id, source, email, username, display_name
from identities
where org_id = $1
  and source = $2
  and username = $3;
```

Expected index: `identities_source_username_idx`

## EXPLAIN checklist
Run on staging database after migration:

```sql
explain (analyze, buffers)
select ... -- each query above
```

Acceptance baseline:
- conflict queue query should not sequential-scan full `person_identity_links`
- email/username lookups should use source-specific btree indexes
- execution time should remain stable at high row counts

## Migration apply checks

When PostgreSQL is available:

1. Apply on clean DB
```bash
psql "$DATABASE_URL" -f backend/db/migrations/0001_identity_v2.sql
```

2. Re-apply (idempotency check)
```bash
psql "$DATABASE_URL" -f backend/db/migrations/0001_identity_v2.sql
```

3. Apply on existing schema snapshot (if available)
```bash
psql "$EXISTING_DB_URL" -f backend/db/migrations/0001_identity_v2.sql
```

4. Verify index creation
```sql
select indexname, indexdef
from pg_indexes
where schemaname = 'public'
  and tablename in ('identities', 'person_identity_links')
order by tablename, indexname;
```
