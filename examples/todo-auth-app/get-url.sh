#!/bin/bash
# Helper script to get the URL of the deployed todo-auth-app

PORT=$(docker ps --filter "name=minifly-todo-auth-app" --format "table {{.Ports}}" | grep -oE '0\.0\.0\.0:([0-9]+)' | cut -d: -f2 | head -1)

if [ -z "$PORT" ]; then
    echo "❌ Todo-auth-app container not found or not running"
    echo "Make sure you've deployed it with: minifly deploy"
    exit 1
fi

echo "✅ Todo-auth-app is running!"
echo "🔗 Access it at: http://localhost:$PORT"
echo ""
echo "You can also check the container with:"
echo "  docker ps | grep todo-auth-app"