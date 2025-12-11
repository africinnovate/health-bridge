#!/bin/bash
echo "🔒 Creating SSH tunnel to database..."
echo "PostgreSQL will be available at localhost:5432"
echo "Press Ctrl+C to close tunnel"

ssh -L 5432:localhost:5432 afric@192.168.1.138 -N
