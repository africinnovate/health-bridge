#!/bin/bash
set -e

echo "Waiting for postgres..."
timeout=30
counter=0
while ! pg_isready -h "$DATABASE_HOST" -U "$POSTGRES_USER" > /dev/null 2>&1; do
  if [ $counter -ge $timeout ]; then
    echo "Timeout waiting for PostgreSQL"
    exit 1
  fi
  echo "Postgres is unavailable - sleeping (${counter}/${timeout})"
  sleep 1
  counter=$((counter + 1))
done

echo "PostgreSQL is ready!"

# Run migrations using psql
echo "Running database migrations..."
for migration in /app/migrations/*/up.sql; do
  if [ -f "$migration" ]; then
    echo "Running migration: $migration"
    PGPASSWORD=$POSTGRES_PASSWORD psql -h "$DATABASE_HOST" -U "$POSTGRES_USER" -d "$POSTGRES_DB" -f "$migration" 2>&1 || echo "Migration may have already been applied"
  fi
done

echo "Migrations complete!"
echo "Starting application with hot-reload on 0.0.0.0:8080..."
echo "Watching for changes in src/..."

# Use cargo watch with proper options
exec cargo watch \
  --why \
  --watch src \
  --ignore "target/*" \
  --ignore "*.swp" \
  -x 'run'