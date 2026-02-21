#!/usr/bin/env bash
set -euo pipefail

# ---------------------------------------------------------------------------
# Ovia — PostgreSQL restore script
#
# Restores a pg_dump custom-format backup into the target database.
# Optionally drops and recreates the database before restoring (--drop).
# Verifies restore by counting rows in key tables.
# ---------------------------------------------------------------------------

# --- Configuration (from environment) --------------------------------------
POSTGRES_HOST="${POSTGRES_HOST:-localhost}"
POSTGRES_PORT="${POSTGRES_PORT:-5432}"
POSTGRES_DB="${POSTGRES_DB:-ovia}"
POSTGRES_USER="${POSTGRES_USER:-ovia}"
export PGPASSWORD="${PGPASSWORD:?PGPASSWORD must be set}"

# --- Helpers ----------------------------------------------------------------
log() {
    echo "[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] $*"
}

die() {
    log "ERROR: $*" >&2
    exit 1
}

usage() {
    echo "Usage: $0 [--drop] <backup-file>"
    echo ""
    echo "Options:"
    echo "  --drop    Drop and recreate the database before restoring"
    echo ""
    echo "Arguments:"
    echo "  backup-file   Path to a pg_dump custom-format (.dump) file"
    exit 1
}

# --- Argument parsing -------------------------------------------------------
DROP_DB=false
BACKUP_FILE=""

while [ $# -gt 0 ]; do
    case "$1" in
        --drop)
            DROP_DB=true
            shift
            ;;
        --help|-h)
            usage
            ;;
        -*)
            die "Unknown option: $1"
            ;;
        *)
            if [ -n "${BACKUP_FILE}" ]; then
                die "Unexpected argument: $1 (backup file already set to ${BACKUP_FILE})"
            fi
            BACKUP_FILE="$1"
            shift
            ;;
    esac
done

if [ -z "${BACKUP_FILE}" ]; then
    usage
fi

# --- Pre-flight checks ------------------------------------------------------
command -v pg_restore >/dev/null 2>&1 || die "pg_restore not found in PATH"
command -v psql >/dev/null 2>&1       || die "psql not found in PATH"

if [ ! -f "${BACKUP_FILE}" ]; then
    die "Backup file not found: ${BACKUP_FILE}"
fi

if [ ! -s "${BACKUP_FILE}" ]; then
    die "Backup file is empty: ${BACKUP_FILE}"
fi

FILE_SIZE="$(stat -c '%s' "${BACKUP_FILE}" 2>/dev/null || stat -f '%z' "${BACKUP_FILE}" 2>/dev/null)"
log "Backup file: ${BACKUP_FILE} (${FILE_SIZE} bytes)"

# --- Drop / recreate (optional) ---------------------------------------------
PSQL_CONN="-h ${POSTGRES_HOST} -p ${POSTGRES_PORT} -U ${POSTGRES_USER}"

if [ "${DROP_DB}" = "true" ]; then
    log "Dropping database '${POSTGRES_DB}'..."
    # Connect to the default 'postgres' database to issue DROP/CREATE
    psql ${PSQL_CONN} -d postgres --no-password -c \
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '${POSTGRES_DB}' AND pid <> pg_backend_pid();" \
        >/dev/null 2>&1 || true

    psql ${PSQL_CONN} -d postgres --no-password -c "DROP DATABASE IF EXISTS \"${POSTGRES_DB}\";"
    log "Creating database '${POSTGRES_DB}'..."
    psql ${PSQL_CONN} -d postgres --no-password -c "CREATE DATABASE \"${POSTGRES_DB}\" OWNER \"${POSTGRES_USER}\";"
    log "Database recreated."
fi

# --- Restore ----------------------------------------------------------------
log "Restoring from ${BACKUP_FILE} into ${POSTGRES_DB}..."

pg_restore \
    -h "${POSTGRES_HOST}" \
    -p "${POSTGRES_PORT}" \
    -U "${POSTGRES_USER}" \
    -d "${POSTGRES_DB}" \
    --no-password \
    --no-owner \
    --no-privileges \
    --if-exists \
    --clean \
    "${BACKUP_FILE}"

log "pg_restore completed."

# --- Verification -----------------------------------------------------------
log "Verifying restore — counting rows in key tables..."

KEY_TABLES="people identities person_identity_links"

echo ""
echo "=== Restore Verification ==="
echo ""

TOTAL_TABLES=0
for TABLE in ${KEY_TABLES}; do
    ROW_COUNT="$(psql ${PSQL_CONN} -d "${POSTGRES_DB}" --no-password -t -A -c \
        "SELECT count(*) FROM ${TABLE};" 2>/dev/null || echo "N/A")"
    printf "  %-30s %s rows\n" "${TABLE}" "${ROW_COUNT}"
    TOTAL_TABLES=$((TOTAL_TABLES + 1))
done

# Also check optional tables (may not exist yet)
OPTIONAL_TABLES="sync_watermarks identity_events kpi_snapshots"
for TABLE in ${OPTIONAL_TABLES}; do
    ROW_COUNT="$(psql ${PSQL_CONN} -d "${POSTGRES_DB}" --no-password -t -A -c \
        "SELECT count(*) FROM ${TABLE};" 2>/dev/null || echo "N/A (table not present)")"
    printf "  %-30s %s rows\n" "${TABLE}" "${ROW_COUNT}"
done

echo ""
echo "=== Summary ==="
echo "  Core tables checked: ${TOTAL_TABLES}"
echo "  Database: ${POSTGRES_DB}"
echo "  Host: ${POSTGRES_HOST}:${POSTGRES_PORT}"
echo ""

log "Restore verification complete."
exit 0
