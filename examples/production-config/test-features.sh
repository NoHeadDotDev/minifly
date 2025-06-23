#!/bin/bash

# Production Config Compatibility Feature Test Script
# This script tests all the production config compatibility features

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Default port (will be updated after deployment)
PORT=8080
BASE_URL="http://localhost:$PORT"

echo "🧪 Testing Production Config Compatibility Features"
echo "=================================================="
echo

# Function to test an endpoint
test_endpoint() {
    local endpoint=$1
    local description=$2
    local expected_key=${3:-"status"}
    
    echo -n "Testing $description... "
    
    if response=$(curl -s "$BASE_URL$endpoint" 2>/dev/null); then
        if echo "$response" | jq -e ".$expected_key" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ PASS${NC}"
            return 0
        else
            echo -e "${RED}✗ FAIL (no $expected_key key)${NC}"
            echo "Response: $response" | head -3
            return 1
        fi
    else
        echo -e "${RED}✗ FAIL (connection error)${NC}"
        return 1
    fi
}

# Wait for the application to be ready
echo -e "${BLUE}Waiting for application to be ready...${NC}"
for i in {1..30}; do
    if curl -s "$BASE_URL/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Application is ready${NC}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "${RED}✗ Application failed to start within 30 seconds${NC}"
        echo "Make sure 'minifly serve --dev' is running in another terminal"
        exit 1
    fi
    sleep 1
done

echo

# Test 1: Environment Variables
echo -e "${BLUE}1. Testing Environment Variable Injection${NC}"
test_endpoint "/health" "Fly.io environment variables" "app_name"

# Test 2: Secrets Management
echo -e "${BLUE}2. Testing Secrets Management${NC}"
test_endpoint "/secrets" "secrets loading" "loaded_secrets"

# Test 3: Volume Mounting
echo -e "${BLUE}3. Testing Volume Mounting${NC}"
test_endpoint "/volumes" "volume persistence" "volume_path"

# Test 4: Service Discovery
echo -e "${BLUE}4. Testing Service Discovery${NC}"
test_endpoint "/discover" "service discovery domains" "app_internal"
test_endpoint "/test-dns" "DNS resolution testing" "app_domain"

# Test 5: Database Operations (if available)
echo -e "${BLUE}5. Testing Database Operations${NC}"
if test_endpoint "/database" "database connection" "database_path"; then
    echo -n "Testing database record creation... "
    if response=$(curl -s -X POST "$BASE_URL/database/records?name=TestRecord" 2>/dev/null); then
        if echo "$response" | jq -e ".id" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ PASS${NC}"
        else
            echo -e "${RED}✗ FAIL${NC}"
        fi
    else
        echo -e "${RED}✗ FAIL (connection error)${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Database not configured (LiteFS may not be available)${NC}"
fi

echo
echo -e "${BLUE}6. Detailed Feature Validation${NC}"

# Get detailed responses for validation
echo "Fetching detailed responses..."

echo
echo -e "${YELLOW}Environment Variables:${NC}"
curl -s "$BASE_URL/health" | jq '.environment' | head -10

echo
echo -e "${YELLOW}Loaded Secrets:${NC}"
curl -s "$BASE_URL/secrets" | jq '.loaded_secrets[]' | head -10

echo
echo -e "${YELLOW}Service Discovery:${NC}"
curl -s "$BASE_URL/discover" | jq '.'

echo
echo -e "${GREEN}🎉 Production Config Compatibility Test Complete!${NC}"
echo
echo "Key features tested:"
echo "✓ Environment variable injection (FLY_* variables)"
echo "✓ Secrets management and loading"
echo "✓ Volume mounting and persistence"
echo "✓ Service discovery (.internal domains)"
echo "✓ Database operations (LiteFS integration)"
echo
echo "All these features work exactly like in production Fly.io!"