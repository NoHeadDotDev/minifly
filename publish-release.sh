#!/bin/bash

# Minifly v0.1.3 Release Publishing Script
# This script publishes all Minifly crates in the correct order

set -e

echo "🚀 Minifly v0.1.3 Release Publishing Script"
echo "==========================================="
echo

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Check if user is logged in to crates.io
echo -e "${BLUE}Checking crates.io authentication...${NC}"
if ! cargo login --quiet 2>/dev/null; then
    echo -e "${RED}❌ You need to login to crates.io first!${NC}"
    echo
    echo "Steps to login:"
    echo "1. Go to https://crates.io/me"
    echo "2. Copy your API token"
    echo "3. Run: cargo login <your-token>"
    exit 1
fi
echo -e "${GREEN}✓ Authenticated with crates.io${NC}"
echo

# Function to publish a crate
publish_crate() {
    local crate_name=$1
    local crate_dir=$2
    
    echo -e "${BLUE}Publishing $crate_name...${NC}"
    cd "$crate_dir"
    
    # Try to publish
    if cargo publish 2>&1 | tee /tmp/publish_output.txt; then
        echo -e "${GREEN}✓ Successfully published $crate_name${NC}"
    else
        # Check if already published
        if grep -q "already uploaded" /tmp/publish_output.txt; then
            echo -e "${YELLOW}⚠️  $crate_name v0.1.3 already published, skipping...${NC}"
        else
            echo -e "${RED}❌ Failed to publish $crate_name${NC}"
            exit 1
        fi
    fi
    
    # Return to root directory
    cd - > /dev/null
    echo
}

# Confirm before proceeding
echo -e "${YELLOW}This will publish the following crates to crates.io:${NC}"
echo "1. minifly-core v0.1.3"
echo "2. minifly-logging v0.1.3"
echo "3. minifly-network v0.1.3"
echo "4. minifly-litefs v0.1.3"
echo "5. minifly-cli v0.1.3 (the main package)"
echo
read -p "Continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Publishing cancelled."
    exit 1
fi

# Start publishing in dependency order
echo
echo -e "${BLUE}Starting publish process...${NC}"
echo

# 1. Core packages (no dependencies)
publish_crate "minifly-core" "minifly-core"
sleep 10  # Wait for crates.io index to update

publish_crate "minifly-logging" "minifly-logging"
sleep 10

publish_crate "minifly-network" "minifly-network"
sleep 10

# 2. LiteFS (depends on core)
publish_crate "minifly-litefs" "minifly-litefs"
sleep 10

# 3. CLI (the main package users install)
publish_crate "minifly-cli" "minifly-cli"

echo
echo -e "${GREEN}🎉 Successfully published Minifly v0.1.3!${NC}"
echo
echo "Users can now install with:"
echo -e "${BLUE}cargo install minifly-cli${NC}"
echo
echo "Release notes: https://github.com/NoHeadDotDev/minifly/releases/tag/v0.1.3"
echo
echo "Next steps:"
echo "1. Create a GitHub release with the changelog"
echo "2. Update the documentation site"
echo "3. Announce on social media/forums"
echo
echo -e "${GREEN}Congratulations on the release! 🚀${NC}"

# Clean up
rm -f /tmp/publish_output.txt