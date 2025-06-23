#!/bin/bash

# Production Config Compatibility Demo Script
# This script demonstrates all the new production config features

set -e

echo "🚀 Production Config Compatibility Demo"
echo "======================================"
echo

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

cd "$(dirname "$0")"

echo -e "${BLUE}1. Setting up secrets...${NC}"
echo "   (This demonstrates the secrets management system)"

# Create secrets using the CLI
minifly secrets set DATABASE_URL=sqlite:///data/production.db
minifly secrets set DATABASE_PATH=/data
minifly secrets set SECRET_KEY=dev-secret-key-12345
minifly secrets set API_TOKEN=dev-api-token-67890
minifly secrets set STRIPE_SECRET_KEY=sk_test_development_key
minifly secrets set LOG_LEVEL=debug

echo -e "${GREEN}✓ Secrets configured${NC}"
echo

echo -e "${BLUE}2. Listing configured secrets...${NC}"
minifly secrets list
echo

echo -e "${BLUE}3. Starting Minifly with production config...${NC}"
echo "   (This will adapt production fly.toml and litefs.yml automatically)"
echo

# Build the application first
echo "Building application..."
cargo build --release

echo
echo -e "${YELLOW}Starting minifly serve --dev...${NC}"
echo "This will:"
echo "- ✅ Load production fly.toml without modifications"
echo "- ✅ Adapt production litefs.yml for local development"
echo "- ✅ Inject secrets as environment variables"
echo "- ✅ Set up .internal DNS resolution"
echo "- ✅ Enable hot reloading"
echo

echo "Press Ctrl+C to stop the demo when you're done testing."
echo

# Start minifly in development mode
exec minifly serve --dev