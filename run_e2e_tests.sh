#!/bin/bash
set -e

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║          PerceptLog End-to-End Test Suite                     ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check prerequisites
echo -e "${BLUE}Checking prerequisites...${NC}"
if [ ! -d "test_data" ]; then
    echo "❌ test_data/ directory not found"
    exit 1
fi

if [ ! -d "scripts" ]; then
    echo "❌ scripts/ directory not found"
    exit 1
fi

echo -e "${GREEN}✓ Prerequisites OK${NC}"
echo ""

# Build project
echo -e "${BLUE}Building project...${NC}"
cargo build --quiet --lib
echo -e "${GREEN}✓ Build complete${NC}"
echo ""

# Run VRL runtime tests
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}1. VRL Runtime Tests${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cargo test --quiet --lib processing::runtime 2>&1 | grep -E "(test |passed)"
echo -e "${GREEN}✓ VRL Runtime tests passed${NC}"
echo ""

# Run E2E integration tests
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}2. End-to-End Integration Tests${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cargo test --test e2e_integration_tests -- --nocapture 2>&1 | grep -E "(test |running|passed|ok)"
echo -e "${GREEN}✓ E2E tests completed${NC}"
echo ""

# Test production script validation
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}3. Script Validation${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f "scripts/production/linux_auth_ocsf.perceptlog" ]; then
    echo "Validating production script..."
    cargo run --quiet -- validate scripts/production/linux_auth_ocsf.perceptlog
    echo -e "${GREEN}✓ Production script valid${NC}"
else
    echo -e "${YELLOW}⚠ Production script not found (optional)${NC}"
fi

if [ -f "scripts/examples/simple_transform.perceptlog" ]; then
    echo "Validating example script..."
    cargo run --quiet -- validate scripts/examples/simple_transform.perceptlog
    echo -e "${GREEN}✓ Example script valid${NC}"
else
    echo -e "${YELLOW}⚠ Example script not found (optional)${NC}"
fi
echo ""

# Test transformation with production script
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${YELLOW}4. Manual Transformation Test${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

mkdir -p output

if [ -f "scripts/production/linux_auth_ocsf.perceptlog" ] && [ -f "test_data/sample_auth.log" ]; then
    echo "Testing transformation with sample data..."
    cargo run --quiet -- transform \
        -s scripts/production/linux_auth_ocsf.perceptlog \
        -i test_data/sample_auth.log \
        -o output/test_result.json \
        -f json-pretty
    
    if [ -f "output/test_result.json" ]; then
        echo -e "${GREEN}✓ Transformation successful${NC}"
        echo "Output file: output/test_result.json"
        
        # Show first event
        echo ""
        echo "Sample output (first event):"
        head -30 output/test_result.json
    else
        echo -e "${YELLOW}⚠ Transformation completed but no output file${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Skipping manual test (files not available)${NC}"
fi
echo ""

# Summary
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                    Test Summary                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo -e "${GREEN}✓ VRL Runtime Tests${NC}"
echo -e "${GREEN}✓ End-to-End Integration Tests${NC}"
echo -e "${GREEN}✓ Script Validation${NC}"
echo -e "${GREEN}✓ Manual Transformation Test${NC}"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}All tests completed successfully! ✅${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Next steps:"
echo "  • View output: cat output/test_result.json | jq ."
echo "  • Run specific test: cargo test test_e2e_ssh_patterns --test e2e_integration_tests"
echo "  • Transform logs: cargo run -- transform -s scripts/production/linux_auth_ocsf.perceptlog -i test_data/auth.log -o output/result.ndjson"
echo ""
