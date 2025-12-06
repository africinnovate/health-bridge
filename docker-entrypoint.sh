#!/bin/bash
set -e

echo "Waiting for postgres..."
while ! pg_isready -h "$DATABASE_HOST" -U "$POSTGRES_USER" > /dev/null 2>&1; do
  echo "Postgres is unavailable - sleeping"
  sleep 1
done

echo "PostgreSQL started"

# Run migrations using psql
echo "Running database migrations..."
for migration in /app/migrations/*/up.sql; do
  if [ -f "$migration" ]; then
    echo "Running migration: $migration"
    PGPASSWORD=$POSTGRES_PASSWORD psql -h "$DATABASE_HOST" -U "$POSTGRES_USER" -d "$POSTGRES_DB" -f "$migration"
  fi
done

echo "Starting application..."
exec /app/emr-api