#!/bin/bash
set -e

echo "-----------------------------------------------------"
echo "🟦 Starting dev-entry script"

# Resolve script directory as project root
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"
MIGRATIONS_DIR="$PROJECT_ROOT/migrations"

echo "Script directory: $SCRIPT_DIR"
echo "Project root: $PROJECT_ROOT"
echo "Migrations directory: $MIGRATIONS_DIR"
echo "-----------------------------------------------------"

# Validate environment variables
: "${DATABASE_HOST:?❌ DATABASE_HOST is not set}"
: "${POSTGRES_USER:?❌ POSTGRES_USER is not set}"
: "${POSTGRES_PASSWORD:?❌ POSTGRES_PASSWORD is not set}"
: "${POSTGRES_DB:?❌ POSTGRES_DB is not set}"

echo "Using database parameters:"
echo "  HOST: $DATABASE_HOST"
echo "  USER: $POSTGRES_USER"
echo "  DB:   $POSTGRES_DB"
echo "-----------------------------------------------------"

echo "⏳ Checking if PostgreSQL is ready..."

TIMEOUT=30
COUNTER=0

while ! pg_isready -h "$DATABASE_HOST" -U "$POSTGRES_USER" >/dev/null 2>&1; do
  if [[ $COUNTER -ge $TIMEOUT ]]; then
    echo "❌ Timeout waiting for PostgreSQL"
    exit 1
  fi

  echo "Postgres unavailable... retry (${COUNTER}/${TIMEOUT})"
  sleep 1
  COUNTER=$((COUNTER + 1))
done

echo "✅ PostgreSQL is ready!"
echo "-----------------------------------------------------"
echo "🔍 Searching for migrations in: $MIGRATIONS_DIR/*/up.sql"
echo "-----------------------------------------------------"

found_any=false

for migration in "$MIGRATIONS_DIR"/*/up.sql; do
  if [[ -f "$migration" ]]; then
    found_any=true
    echo "➡️  Applying migration: $migration"

    if ! PGPASSWORD="$POSTGRES_PASSWORD" \
      psql -h "$DATABASE_HOST" -U "$POSTGRES_USER" -d "$POSTGRES_DB" -f "$migration"
    then
      echo "⚠️  Migration failed OR already applied: $migration"
    fi
  fi
done

if [[ $found_any == false ]]; then
  echo "❌ No migrations found in: $MIGRATIONS_DIR/*/up.sql"
  echo "Check your folder structure."
fi

echo "-----------------------------------------------------"
echo "✅ Migrations complete!"
echo "-----------------------------------------------------"

echo "🚀 Starting application with hot-reload on 0.0.0.0:8080"
echo "📡 Watching for changes in src/..."

exec cargo watch \
  --why \
  --watch src \
  --ignore "target/*" \
  --ignore "*.swp" \
  -x 'run'
