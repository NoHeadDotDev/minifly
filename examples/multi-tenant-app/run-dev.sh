#!/bin/bash
# Script to run the multi-tenant app in development mode

set -e

echo "🚀 Starting Multi-Tenant App in Development Mode"
echo "==============================================="

# Create data directory
mkdir -p data

# Copy .env file if it doesn't exist
if [ ! -f .env ]; then
    cp .env.example .env
    echo "✅ Created .env file from .env.example"
fi

# Load environment variables
export $(cat .env | grep -v '^#' | xargs)

# Build and run
echo "🔨 Building application..."
cargo build --release

echo "🚀 Starting application..."
cargo run --release