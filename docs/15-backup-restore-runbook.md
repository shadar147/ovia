# Ovia -- Backup & Restore Runbook

## Overview

Ovia stores all operational data in a single PostgreSQL 16 database (`ovia`). This includes identity mappings, person records, sync watermarks, audit events, and (in future) KPI snapshots and ask sessions. Loss of this database would require full re-ingestion from all connected sources and loss of all manual identity confirmations.

| Item              | Value                                          |
|-------------------|-------------------------------------------------|
| Database engine   | PostgreSQL 16                                   |
| Backup format     | `pg_dump` custom format (`-Fc`)                 |
| Daily schedule    | Every 24 hours via Docker Swarm `backup` service |
| Weekly schedule   | Sundays (copy of daily backup)                   |
| Daily retention   | 7 backups                                        |
| Weekly retention  | 4 backups                                        |
| Storage location  | `backup_data` Docker volume (`/backups`)         |

## Automated Backups

The `backup` service in `docker-compose.swarm.yml` runs `backup.sh` every 24 hours inside a `postgres:16` container that has network access to the database.

### How it works

1. The container starts and runs `/usr/local/bin/backup.sh`.
2. `pg_dump` creates a daily custom-format dump in `/backups/daily/`.
3. On Sundays, the daily dump is also copied to `/backups/weekly/`.
4. Old backups beyond the retention window are deleted.
5. The script sleeps 86400 seconds (24 hours) and repeats.

### Checking backup logs

```bash
# Tail the backup service logs
docker service logs ovia_backup --tail 50

# Follow logs in real time
docker service logs ovia_backup --follow
```

### Verifying the latest backup

```bash
# Find the latest daily backup file
docker exec $(docker ps -q -f name=ovia_backup) \
  ls -lt /backups/daily/ | head -5

# Run the verification script against the latest backup
docker exec $(docker ps -q -f name=ovia_backup) \
  /usr/local/bin/verify-backup.sh /backups/daily/$(ls -t /backups/daily/ | head -1)
```

## Manual Backup

### Ad-hoc backup on the Swarm host

```bash
# Run a one-off backup using the same container image
docker run --rm \
  --network ovia_ovia-net \
  -e POSTGRES_HOST=postgres \
  -e POSTGRES_PORT=5432 \
  -e POSTGRES_DB=ovia \
  -e POSTGRES_USER=ovia \
  -e PGPASSWORD='<your-password>' \
  -e BACKUP_DIR=/backups \
  -v ovia_backup_data:/backups \
  -v $(pwd)/backend/infra/backup/backup.sh:/usr/local/bin/backup.sh:ro \
  postgres:16 \
  bash /usr/local/bin/backup.sh
```

### Copying a backup off-host

```bash
# Find the backup container
CONTAINER=$(docker ps -q -f name=ovia_backup)

# Copy latest daily backup to local machine
docker cp "${CONTAINER}:/backups/daily/$(docker exec ${CONTAINER} ls -t /backups/daily/ | head -1)" ./ovia-backup-latest.dump

# Or copy directly from the volume on the host
docker run --rm -v ovia_backup_data:/backups -v $(pwd):/out alpine \
  cp /backups/daily/$(ls -t /backups/daily/ 2>/dev/null | head -1) /out/
```

### Copying backup to remote storage

```bash
# SCP to a remote backup host
scp ./ovia-backup-latest.dump backup-user@backup-host:/backups/ovia/

# Or use rclone for S3-compatible storage (if configured)
rclone copy ./ovia-backup-latest.dump remote:ovia-backups/
```

## Restore Procedures

### Prerequisites for all scenarios

- Access to the backup `.dump` file
- `pg_restore` and `psql` available (included in `postgres:16` image)
- Database credentials (`POSTGRES_USER`, `PGPASSWORD`)
- The target PostgreSQL server is running and reachable

### Scenario 1: Restore to same host

Use this when the database is corrupted or data was accidentally deleted, but the Swarm infrastructure is intact.

```bash
# 1. Stop application services to prevent writes during restore
docker service scale ovia_ovia-api=0 ovia_ovia-ingest=0 ovia_ovia-metrics=0

# 2. Identify the backup file to restore
docker exec $(docker ps -q -f name=ovia_backup) ls -lt /backups/daily/

# 3. Run the restore script (with --drop to start clean)
docker run --rm \
  --network ovia_ovia-net \
  -e POSTGRES_HOST=postgres \
  -e POSTGRES_PORT=5432 \
  -e POSTGRES_DB=ovia \
  -e POSTGRES_USER=ovia \
  -e PGPASSWORD='<your-password>' \
  -v ovia_backup_data:/backups:ro \
  -v $(pwd)/backend/infra/backup/restore.sh:/usr/local/bin/restore.sh:ro \
  postgres:16 \
  bash /usr/local/bin/restore.sh --drop /backups/daily/ovia_daily_2026-02-21_030000.dump

# 4. Verify row counts in the output summary

# 5. Re-run migrations to pick up any schema changes since the backup
docker service update --force ovia_migrate

# 6. Restart application services
docker service scale ovia_ovia-api=1 ovia_ovia-ingest=1 ovia_ovia-metrics=1

# 7. Verify the API is healthy
curl -sf http://localhost:8080/health
```

### Scenario 2: Restore to new host

Use this when provisioning a replacement server or migrating to new infrastructure.

```bash
# 1. On the new host: deploy the Swarm stack (without application services)
docker stack deploy -c docker-compose.swarm.yml ovia

# 2. Wait for PostgreSQL to become healthy
docker service logs ovia_postgres --follow
# Wait until you see "database system is ready to accept connections"

# 3. Copy the backup file to the new host
scp backup-user@old-host:/path/to/ovia_daily_2026-02-21_030000.dump /tmp/

# 4. Restore into the new database
docker run --rm \
  --network ovia_ovia-net \
  -e POSTGRES_HOST=postgres \
  -e POSTGRES_PORT=5432 \
  -e POSTGRES_DB=ovia \
  -e POSTGRES_USER=ovia \
  -e PGPASSWORD='<your-password>' \
  -v /tmp/ovia_daily_2026-02-21_030000.dump:/backup.dump:ro \
  -v $(pwd)/backend/infra/backup/restore.sh:/usr/local/bin/restore.sh:ro \
  postgres:16 \
  bash /usr/local/bin/restore.sh --drop /backup.dump

# 5. Verify the restore summary output

# 6. Run migrations to ensure schema is current
docker service update --force ovia_migrate

# 7. Scale up all application services
docker service scale ovia_ovia-api=1 ovia_ovia-ingest=1 ovia_ovia-metrics=1 ovia_ovia-rag=1 ovia_ovia-scheduler=1

# 8. Verify health
curl -sf http://localhost:8080/health
```

### Scenario 3: Point-in-time recovery

Point-in-time recovery (PITR) requires continuous WAL archiving, which is **not yet configured** in this deployment. The current backup strategy captures full snapshots only.

**Future enhancement:** To enable PITR, add the following to the PostgreSQL configuration:

```ini
archive_mode = on
archive_command = 'cp %p /backups/wal/%f'
restore_command = 'cp /backups/wal/%f %p'
```

This would allow restoring the database to any point between backups by replaying WAL segments. Until WAL archiving is configured, the recovery point objective (RPO) is bounded by the backup interval (24 hours worst case).

## Monitoring & Alerts

### Checking backup freshness

```bash
# Check the timestamp of the most recent daily backup
docker exec $(docker ps -q -f name=ovia_backup) \
  stat /backups/daily/$(ls -t /backups/daily/ | head -1)

# Quick freshness check: fail if no backup in the last 26 hours
docker exec $(docker ps -q -f name=ovia_backup) \
  find /backups/daily -name 'ovia_daily_*.dump' -mmin -1560 | grep -q . \
  && echo "OK: Recent backup exists" \
  || echo "ALERT: No backup in the last 26 hours"
```

### What to do if backup fails

1. Check the backup service logs: `docker service logs ovia_backup --tail 100`
2. Common causes:
   - PostgreSQL is unreachable (network or health issue)
   - Disk full on the backup volume
   - `PGPASSWORD` environment variable changed
3. After fixing the root cause, the backup loop will retry on its next cycle.
4. To force an immediate retry: `docker service update --force ovia_backup`

### Suggested external monitoring

- Set up a cron job or monitoring agent on the Swarm host that checks backup freshness daily.
- Alert if no backup file newer than 26 hours exists in the backup volume.
- Monitor Docker volume disk usage to prevent the backup volume from filling up.

## Testing

### Monthly restore drill procedure

Perform this drill at least once per month to validate that backups are restorable.

1. **Select a backup file** -- use the most recent daily or weekly backup.

2. **Verify the archive** (non-destructive):
   ```bash
   docker run --rm \
     -v ovia_backup_data:/backups:ro \
     -v $(pwd)/backend/infra/backup/verify-backup.sh:/usr/local/bin/verify-backup.sh:ro \
     postgres:16 \
     bash /usr/local/bin/verify-backup.sh /backups/daily/<selected-file>.dump
   ```

3. **Restore to a test database** (does not affect production):
   ```bash
   # Create a temporary test database
   docker exec $(docker ps -q -f name=ovia_postgres) \
     psql -U ovia -d postgres -c "CREATE DATABASE ovia_restore_test OWNER ovia;"

   # Restore into it
   docker run --rm \
     --network ovia_ovia-net \
     -e POSTGRES_HOST=postgres \
     -e POSTGRES_PORT=5432 \
     -e POSTGRES_DB=ovia_restore_test \
     -e POSTGRES_USER=ovia \
     -e PGPASSWORD='<your-password>' \
     -v ovia_backup_data:/backups:ro \
     -v $(pwd)/backend/infra/backup/restore.sh:/usr/local/bin/restore.sh:ro \
     postgres:16 \
     bash /usr/local/bin/restore.sh /backups/daily/<selected-file>.dump

   # Clean up the test database
   docker exec $(docker ps -q -f name=ovia_postgres) \
     psql -U ovia -d postgres -c "DROP DATABASE ovia_restore_test;"
   ```

4. **Record the drill result** -- note the date, backup file used, row counts, and any issues in the team's operational log.

### Verification checklist

- [ ] `verify-backup.sh` reports all expected tables present
- [ ] `restore.sh` completes without errors
- [ ] Row counts for `people`, `identities`, `person_identity_links` are non-zero
- [ ] Row counts are consistent with expected data volume
- [ ] Application health check passes after restore (`/health` endpoint)

## Troubleshooting

### pg_dump fails with "connection refused"

**Cause:** PostgreSQL is not running or the backup container cannot reach it.

**Fix:**
```bash
# Check PostgreSQL service health
docker service ps ovia_postgres
docker service logs ovia_postgres --tail 20

# Verify network connectivity
docker exec $(docker ps -q -f name=ovia_backup) pg_isready -h postgres -p 5432
```

### pg_dump fails with "authentication failed"

**Cause:** `PGPASSWORD` does not match the PostgreSQL server password.

**Fix:** Ensure the `POSTGRES_PASSWORD` environment variable in `.env` matches across all services. Redeploy:
```bash
docker stack deploy -c docker-compose.swarm.yml ovia
```

### Backup volume is full

**Cause:** Retention cleanup is not removing files fast enough, or the volume is undersized.

**Fix:**
```bash
# Check volume usage
docker run --rm -v ovia_backup_data:/backups alpine df -h /backups

# Manually remove old backups
docker run --rm -v ovia_backup_data:/backups alpine \
  sh -c 'ls -lt /backups/daily/ && ls -lt /backups/weekly/'
# Then remove specific old files
```

### pg_restore fails with "invalid archive"

**Cause:** The backup file is corrupted or was created with a different format.

**Fix:**
```bash
# Verify the archive first
verify-backup.sh /path/to/backup.dump

# If corrupt, try an older backup
ls -lt /backups/daily/
```

### Restore completes but tables have 0 rows

**Cause:** The backup was taken from an empty database, or `--clean` dropped tables that `pg_restore` then failed to recreate (check for errors in output).

**Fix:** Re-run `restore.sh` with `--drop` to start from a clean database, which avoids conflicts with existing objects.

### Slow restore on large databases

**Cause:** Single-threaded restore on a large dump.

**Fix:** For databases over 1 GB, consider using `pg_restore --jobs=N` for parallel restore. Modify `restore.sh` or run manually:
```bash
pg_restore -h postgres -p 5432 -U ovia -d ovia --no-owner --jobs=4 /path/to/backup.dump
```
