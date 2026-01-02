#!/bin/bash
set -e

# Check for environment argument
ENV_FILE="${1:-.env}"

if [ ! -f "$ENV_FILE" ]; then
    echo "❌ Environment file not found: $ENV_FILE"
    exit 1
fi

# Load environment variables
export $(cat "$ENV_FILE" | grep -v '^#' | xargs)

echo "🚀 Starting Health Bridge in development mode..."
echo "📊 Using database: $DATABASE_HOST"
echo "📡 Hot reload enabled - changes to src/ will trigger rebuild"
echo "🌐 Server will be available at http://localhost:8080"
echo ""
echo "Press Ctrl+C to stop"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cargo watch \
    --why \
    --watch src \
    --ignore "target/*" \
    --ignore "*.swp" \
    -x 'run --bin health-bridge'