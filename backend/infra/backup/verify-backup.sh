#!/usr/bin/env bash
set -euo pipefail

# ---------------------------------------------------------------------------
# Ovia — Backup verification script
#
# Validates a pg_dump custom-format archive without restoring it.
# Checks that expected tables are present and reports file metadata.
# ---------------------------------------------------------------------------

# --- Helpers ----------------------------------------------------------------
log() {
    echo "[$(date -u '+%Y-%m-%dT%H:%M:%SZ')] $*"
}

die() {
    log "ERROR: $*" >&2
    exit 1
}

usage() {
    echo "Usage: $0 <backup-file>"
    echo ""
    echo "Verifies a pg_dump custom-format (.dump) archive is readable"
    echo "and contains the expected Ovia database tables."
    exit 1
}

# --- Argument parsing -------------------------------------------------------
if [ $# -lt 1 ] || [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    usage
fi

BACKUP_FILE="$1"

# --- Pre-flight checks ------------------------------------------------------
command -v pg_restore >/dev/null 2>&1 || die "pg_restore not found in PATH"

if [ ! -f "${BACKUP_FILE}" ]; then
    die "Backup file not found: ${BACKUP_FILE}"
fi

if [ ! -s "${BACKUP_FILE}" ]; then
    die "Backup file is empty: ${BACKUP_FILE}"
fi

# --- File metadata ----------------------------------------------------------
FILE_SIZE="$(stat -c '%s' "${BACKUP_FILE}" 2>/dev/null || stat -f '%z' "${BACKUP_FILE}" 2>/dev/null)"
FILE_DATE="$(stat -c '%y' "${BACKUP_FILE}" 2>/dev/null || stat -f '%Sm' "${BACKUP_FILE}" 2>/dev/null)"

echo ""
echo "=== Backup File Info ==="
echo "  File:     ${BACKUP_FILE}"
echo "  Size:     ${FILE_SIZE} bytes ($(awk "BEGIN {printf \"%.2f\", ${FILE_SIZE}/1048576}") MB)"
echo "  Modified: ${FILE_DATE}"
echo ""

# --- Archive listing --------------------------------------------------------
log "Reading archive table of contents..."

TOC="$(pg_restore --list "${BACKUP_FILE}" 2>&1)" || die "pg_restore --list failed — archive may be corrupt"

TABLE_LINES="$(echo "${TOC}" | grep -c '^[0-9].*;.*TABLE ' || true)"

echo "=== Archive Contents ==="
echo "  Total TOC entries: $(echo "${TOC}" | wc -l | tr -d ' ')"
echo "  Table entries:     ${TABLE_LINES}"
echo ""

# --- Expected tables --------------------------------------------------------
EXPECTED_TABLES="people identities person_identity_links sync_watermarks identity_events"
# kpi_snapshots is a future table (Epic 4) -- check but don't fail on absence
OPTIONAL_TABLES="kpi_snapshots"

MISSING=0
FOUND=0

echo "=== Expected Tables ==="
for TABLE in ${EXPECTED_TABLES}; do
    if echo "${TOC}" | grep -q "TABLE.*${TABLE}"; then
        printf "  [OK]      %s\n" "${TABLE}"
        FOUND=$((FOUND + 1))
    else
        printf "  [MISSING] %s\n" "${TABLE}"
        MISSING=$((MISSING + 1))
    fi
done

for TABLE in ${OPTIONAL_TABLES}; do
    if echo "${TOC}" | grep -q "TABLE.*${TABLE}"; then
        printf "  [OK]      %s (optional)\n" "${TABLE}"
    else
        printf "  [--]      %s (optional — not yet created)\n" "${TABLE}"
    fi
done

echo ""

# --- Index and sequence counts ----------------------------------------------
INDEX_COUNT="$(echo "${TOC}" | grep -c 'INDEX' || true)"
SEQUENCE_COUNT="$(echo "${TOC}" | grep -c 'SEQUENCE' || true)"
CONSTRAINT_COUNT="$(echo "${TOC}" | grep -c 'CONSTRAINT\|FK CONSTRAINT' || true)"

echo "=== Additional Objects ==="
echo "  Indexes:     ${INDEX_COUNT}"
echo "  Sequences:   ${SEQUENCE_COUNT}"
echo "  Constraints: ${CONSTRAINT_COUNT}"
echo ""

# --- Verdict ----------------------------------------------------------------
echo "=== Verdict ==="
if [ "${MISSING}" -gt 0 ]; then
    echo "  WARN: ${MISSING} expected table(s) missing from backup"
    echo "  Found ${FOUND} of $((FOUND + MISSING)) expected tables"
    echo ""
    exit 1
else
    echo "  OK: All ${FOUND} expected tables present"
    echo "  Archive is readable and structurally valid"
    echo ""
    exit 0
fi
