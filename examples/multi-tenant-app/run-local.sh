#!/bin/bash
# Script to run the multi-tenant app locally with Minifly

set -e

echo "🚀 Starting Multi-Tenant App with Minifly"
echo "========================================="

# Check if Minifly API is running
if ! curl -s http://localhost:4280/v1/health > /dev/null; then
    echo "❌ Minifly API is not running!"
    echo "Please start it with: MINIFLY_DATABASE_URL='sqlite::memory:' cargo run --bin minifly-api"
    exit 1
fi

echo "✅ Minifly API is running"

# Build the Docker image
echo "🔨 Building Docker image..."
docker build -t multi-tenant-app:latest .

# Deploy using Minifly
echo "🚀 Deploying to Minifly..."
cd ../..  # Go back to minifly root
cargo run --bin minifly -- deploy examples/multi-tenant-app/fly.toml

echo ""
echo "✅ Application deployed!"
echo "🔗 Access the app at: http://localhost"
echo ""
echo "Try these commands:"
echo "  curl http://localhost/api/tenants"
echo "  curl -H 'X-Tenant: acme' http://localhost/api/items"