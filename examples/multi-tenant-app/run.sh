#!/bin/bash
set -e

echo "🚀 Starting Multi-Tenant Application Example"
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo is not installed or not in PATH"
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Create data directory if it doesn't exist
mkdir -p data

# Set environment variables
export DATABASE_PATH="./data"
export PORT="8080"
export RUST_LOG="info,multi_tenant_app=debug"

echo "📂 Database path: $DATABASE_PATH"
echo "🌐 Server will start on: http://localhost:$PORT"
echo ""

# Run the application
echo "🔄 Building and starting application..."
cargo run

echo ""
echo "✅ Application stopped"