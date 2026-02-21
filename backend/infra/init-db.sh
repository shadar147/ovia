#!/bin/sh
set -e

echo "Waiting for postgres..."
until pg_isready -d "$DATABASE_URL" 2>/dev/null; do
  sleep 1
done

echo "Running migrations..."
for f in /migrations/sql/*.sql; do
  echo "Applying $f ..."
  psql "$DATABASE_URL" -f "$f"
done

echo "Migrations complete."
