#!/bin/bash
set -e

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
else
    echo "❌ .env file not found!"
    exit 1
fi

echo "🚀 Starting Health Bridge in development mode..."
echo "📡 Hot reload enabled - changes to src/ will trigger rebuild"
echo "🌐 Server will be available at http://localhost:8080"
echo ""
echo "Press Ctrl+C to stop"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Start with cargo-watch for hot reload
cargo watch \
    --why \
    --watch src \
    --ignore "target/*" \
    --ignore "*.swp" \
    -x 'run'