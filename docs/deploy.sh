#!/bin/bash
set -e

echo "🚀 Deploying Minifly documentation to Fly.io..."

# Build the documentation locally first to ensure it works
echo "📦 Building documentation..."
npm run build

# Create the app if it doesn't exist
if ! fly status --app minifly-docs >/dev/null 2>&1; then
    echo "🏗️ Creating Fly app..."
    fly apps create minifly-docs --org personal
fi

# Deploy to Fly.io
echo "🎯 Deploying to Fly.io..."
fly deploy --app minifly-docs

echo "✅ Documentation deployed successfully!"
echo "🌐 Visit: https://minifly-docs.fly.dev"