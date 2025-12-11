#!/bin/bash
set -e

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
else
    echo "❌ .env file not found!"
    exit 1
fi

echo "⚠️  This will DROP and recreate the database!"
read -p "Are you sure? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "Aborted."
    exit 0
fi

echo "🗑️  Dropping database..."
PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$DATABASE_HOST" -U "$POSTGRES_USER" -d postgres -c "DROP DATABASE IF EXISTS $POSTGRES_DB;"

echo "📦 Creating database..."
PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$DATABASE_HOST" -U "$POSTGRES_USER" -d postgres -c "CREATE DATABASE $POSTGRES_DB OWNER $POSTGRES_USER;"

echo "🔧 Granting privileges..."
PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$DATABASE_HOST" -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "GRANT ALL ON SCHEMA public TO $POSTGRES_USER; GRANT CREATE ON SCHEMA public TO $POSTGRES_USER;"

echo "📦 Running migrations..."
./scripts/migrate.sh

echo "✅ Database reset complete!"