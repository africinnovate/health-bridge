#!/bin/bash
set -e

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

echo "🧪 Running tests..."
cargo test --all-features

echo ""
echo "✅ All tests passed!"