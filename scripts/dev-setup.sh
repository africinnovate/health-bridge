#!/bin/bash
set -e

echo "🔧 Setting up development environment..."

# Check prerequisites
command -v psql >/dev/null 2>&1 || { echo "❌ PostgreSQL not installed"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "❌ Rust/Cargo not installed"; exit 1; }
command -v cargo-watch >/dev/null 2>&1 || { echo "⚠️  cargo-watch not installed, installing..."; cargo install cargo-watch; }

# Load environment variables
ENV_FILE="${1:-.env}"

if [ -f "$ENV_FILE" ]; then
    echo "📄 Loading environment variables from $ENV_FILE"
    set -a
    source "$ENV_FILE"
    set +a
else
    echo "❌ Env file '$ENV_FILE' not found!"
    exit 1
fi


echo "✅ Prerequisites checked"

# Test database connection
echo "🔍 Testing database connection..."
if psql -h "$DATABASE_HOST" -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "SELECT 1;" >/dev/null 2>&1; then
    echo "✅ Database connection successful"
else
    echo "❌ Database connection failed"
    echo "Please ensure PostgreSQL is running and credentials are correct"
    exit 1
fi

# Run migrations
echo "📦 Running migrations..."
./scripts/migrations.sh

echo ""
echo "✅ Development environment ready!"
echo ""
echo "To start development server, run:"
echo "  ./scripts/dev-start.sh"
echo ""