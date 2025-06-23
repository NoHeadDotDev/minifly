#!/bin/bash
set -e

echo "🚀 Starting Todo Auth App Example"
echo "================================"

# Check if minifly is running
if ! curl -s http://localhost:4280/v1/health > /dev/null 2>&1; then
    echo "❌ Minifly platform is not running!"
    echo "Please start it first with: minifly serve --dev"
    exit 1
fi

echo "✅ Minifly platform is running"

# Deploy the app
echo ""
echo "📦 Deploying todo-auth-app..."
minifly deploy

echo ""
echo "✨ Deployment complete!"
echo ""
echo "📝 Next steps:"
echo "1. Check the URL above to access your app"
echo "2. Sign up with an email and select a region"
echo "3. Create some todos and upload images"
echo "4. Check 'docker ps' to see the tenant containers"
echo ""