#!/usr/bin/env bash
set -euo pipefail

# ---------------------------------------------------------------------------
# Ovia — PostgreSQL backup script
#
# Creates daily backups (custom format) and weekly backups on Sundays.
# Retention: 7 daily, 4 weekly (configurable via env vars).
# Designed to run inside a Docker container alongside the postgres service.
# ---------------------------------------------------------------------------

# --- Configuration (from environment) --------------------------------------
POSTGRES_HOST="${POSTGRES_HOST:-localhost}"
POSTGRES_PORT="${POSTGRES_PORT:-5432}"
POSTGRES_DB="${POSTGRES_DB:-ovia}"
POSTGRES_USER="${POSTGRES_USER:-ovia}"
# PGPASSWORD is read directly by pg_dump — no wrapper needed
export PGPASSWORD="${PGPASSWORD:?PGPASSWORD must be set}"

BACKUP_DIR="${BACKUP_DIR:-/var/backups/ovia}"
BACKUP_RETENTION_DAILY="${BACKUP_RETENTION_DAILY:-7}"
BACKUP_RETENTION_WEEKLY="${BACKUP_RETENTION_WEEKLY:-4}"

# --- Helpers ----------------------------------------------------------------
log() {
    echo "[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] $*"
}

die() {
    log "ERROR: $*" >&2
    exit 1
}

# --- Pre-flight checks ------------------------------------------------------
command -v pg_dump >/dev/null 2>&1 || die "pg_dump not found in PATH"

mkdir -p "${BACKUP_DIR}/daily" "${BACKUP_DIR}/weekly"

NOW="$(date -u '+%Y-%m-%d_%H%M%S')"
TODAY="$(date -u '+%Y-%m-%d')"
DAY_OF_WEEK="$(date -u '+%u')"   # 1=Mon … 7=Sun

# --- Daily backup -----------------------------------------------------------
DAILY_FILE="${BACKUP_DIR}/daily/ovia_daily_${NOW}.dump"

log "Starting daily backup -> ${DAILY_FILE}"

pg_dump \
    -h "${POSTGRES_HOST}" \
    -p "${POSTGRES_PORT}" \
    -U "${POSTGRES_USER}" \
    -d "${POSTGRES_DB}" \
    -Fc \
    --no-password \
    -f "${DAILY_FILE}"

# Verify the dump was created and is non-empty
if [ ! -s "${DAILY_FILE}" ]; then
    die "Daily backup file is missing or empty: ${DAILY_FILE}"
fi

DAILY_SIZE="$(stat -c '%s' "${DAILY_FILE}" 2>/dev/null || stat -f '%z' "${DAILY_FILE}" 2>/dev/null)"
log "Daily backup complete — ${DAILY_SIZE} bytes"

# --- Weekly backup (Sunday = day 7) -----------------------------------------
if [ "${DAY_OF_WEEK}" = "7" ]; then
    WEEKLY_FILE="${BACKUP_DIR}/weekly/ovia_weekly_${TODAY}.dump"
    log "Sunday detected — creating weekly backup -> ${WEEKLY_FILE}"
    cp "${DAILY_FILE}" "${WEEKLY_FILE}"

    if [ ! -s "${WEEKLY_FILE}" ]; then
        die "Weekly backup file is missing or empty: ${WEEKLY_FILE}"
    fi

    WEEKLY_SIZE="$(stat -c '%s' "${WEEKLY_FILE}" 2>/dev/null || stat -f '%z' "${WEEKLY_FILE}" 2>/dev/null)"
    log "Weekly backup complete — ${WEEKLY_SIZE} bytes"
fi

# --- Retention cleanup ------------------------------------------------------
log "Enforcing retention: ${BACKUP_RETENTION_DAILY} daily, ${BACKUP_RETENTION_WEEKLY} weekly"

# Remove oldest daily backups beyond retention count
DAILY_COUNT="$(find "${BACKUP_DIR}/daily" -maxdepth 1 -name 'ovia_daily_*.dump' -type f | wc -l)"
if [ "${DAILY_COUNT}" -gt "${BACKUP_RETENTION_DAILY}" ]; then
    EXCESS=$((DAILY_COUNT - BACKUP_RETENTION_DAILY))
    log "Removing ${EXCESS} old daily backup(s)"
    find "${BACKUP_DIR}/daily" -maxdepth 1 -name 'ovia_daily_*.dump' -type f -print0 \
        | sort -z \
        | head -z -n "${EXCESS}" \
        | xargs -0 rm -f
fi

# Remove oldest weekly backups beyond retention count
WEEKLY_COUNT="$(find "${BACKUP_DIR}/weekly" -maxdepth 1 -name 'ovia_weekly_*.dump' -type f | wc -l)"
if [ "${WEEKLY_COUNT}" -gt "${BACKUP_RETENTION_WEEKLY}" ]; then
    EXCESS=$((WEEKLY_COUNT - BACKUP_RETENTION_WEEKLY))
    log "Removing ${EXCESS} old weekly backup(s)"
    find "${BACKUP_DIR}/weekly" -maxdepth 1 -name 'ovia_weekly_*.dump' -type f -print0 \
        | sort -z \
        | head -z -n "${EXCESS}" \
        | xargs -0 rm -f
fi

# --- Summary ----------------------------------------------------------------
log "Backup complete. Daily: $(find "${BACKUP_DIR}/daily" -maxdepth 1 -name 'ovia_daily_*.dump' -type f | wc -l) files, Weekly: $(find "${BACKUP_DIR}/weekly" -maxdepth 1 -name 'ovia_weekly_*.dump' -type f | wc -l) files"

exit 0
