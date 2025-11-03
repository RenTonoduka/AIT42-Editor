#!/bin/bash
# AIT42-Editor Implementation Verification Script

set -e

echo "=== AIT42-Editor Implementation Verification ==="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
PASS=0
FAIL=0

check() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
        ((PASS++))
    else
        echo -e "${RED}✗${NC} $2"
        ((FAIL++))
    fi
}

echo "1. Checking project structure..."
echo "--------------------------------"

# Check main crates
[ -d "crates/ait42-ait42" ]; check $? "ait42-ait42 crate exists"
[ -d "crates/ait42-fs" ]; check $? "ait42-fs crate exists"
[ -d "crates/ait42-lsp" ]; check $? "ait42-lsp crate exists"
[ -d "crates/ait42-config" ]; check $? "ait42-config crate exists"

echo ""
echo "2. Checking ait42-ait42 files..."
echo "--------------------------------"

# Check implementation files
[ -f "crates/ait42-ait42/src/lib.rs" ]; check $? "lib.rs"
[ -f "crates/ait42-ait42/src/error.rs" ]; check $? "error.rs"
[ -f "crates/ait42-ait42/src/registry.rs" ]; check $? "registry.rs"
[ -f "crates/ait42-ait42/src/tmux.rs" ]; check $? "tmux.rs"
[ -f "crates/ait42-ait42/src/config.rs" ]; check $? "config.rs"
[ -f "crates/ait42-ait42/src/coordinator.rs" ]; check $? "coordinator.rs"
[ -f "crates/ait42-ait42/src/executor.rs" ]; check $? "executor.rs"
[ -f "crates/ait42-ait42/src/stream.rs" ]; check $? "stream.rs"
[ -f "crates/ait42-ait42/src/commands.rs" ]; check $? "commands.rs"
[ -f "crates/ait42-ait42/src/editor_integration.rs" ]; check $? "editor_integration.rs"

echo ""
echo "3. Checking tests..."
echo "--------------------"

[ -f "crates/ait42-ait42/tests/integration_tests.rs" ]; check $? "integration_tests.rs"

echo ""
echo "4. Checking examples..."
echo "-----------------------"

[ -f "crates/ait42-ait42/examples/basic_usage.rs" ]; check $? "basic_usage.rs"
[ -f "crates/ait42-ait42/examples/advanced_usage.rs" ]; check $? "advanced_usage.rs"

echo ""
echo "5. Checking documentation..."
echo "----------------------------"

[ -f "crates/ait42-ait42/README.md" ]; check $? "crate README.md"
[ -f "AIT42_INTEGRATION_REPORT.md" ]; check $? "AIT42_INTEGRATION_REPORT.md"
[ -f "IMPLEMENTATION_SUMMARY.md" ]; check $? "IMPLEMENTATION_SUMMARY.md"

echo ""
echo "6. Checking line counts..."
echo "--------------------------"

# Count lines
SRC_LINES=$(find crates/ait42-ait42/src -name "*.rs" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')
TEST_LINES=$(find crates/ait42-ait42/tests -name "*.rs" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')
EXAMPLE_LINES=$(find crates/ait42-ait42/examples -name "*.rs" -exec wc -l {} + 2>/dev/null | tail -1 | awk '{print $1}')

echo "Source files: ${SRC_LINES} lines"
echo "Test files: ${TEST_LINES} lines"
echo "Example files: ${EXAMPLE_LINES} lines"
echo "Total: $((SRC_LINES + TEST_LINES + EXAMPLE_LINES)) lines"

[ $SRC_LINES -gt 1900 ]; check $? "Source lines > 1900"
[ $TEST_LINES -gt 300 ]; check $? "Test lines > 300"

echo ""
echo "7. Checking Git status..."
echo "-------------------------"

if git status | grep -q "nothing to commit, working tree clean"; then
    check 0 "Working tree is clean"
else
    check 1 "Working tree has uncommitted changes"
fi

# Check commits
RECENT_COMMITS=$(git log --oneline -3)
echo ""
echo "Recent commits:"
echo "$RECENT_COMMITS"

echo ""
echo "8. External dependencies check..."
echo "---------------------------------"

# Check for required tools
command -v tmux >/dev/null 2>&1; check $? "tmux is installed"

# Check for AIT42 system
if [ -n "$AIT42_ROOT" ]; then
    [ -d "$AIT42_ROOT" ]; check $? "AIT42_ROOT directory exists"
    [ -d "$AIT42_ROOT/.claude/agents" ]; check $? "AIT42 agents directory exists"
    [ -d "$AIT42_ROOT/scripts" ]; check $? "AIT42 scripts directory exists"

    # Count agents
    AGENT_COUNT=$(find "$AIT42_ROOT/.claude/agents" -name "*.md" 2>/dev/null | wc -l | tr -d ' ')
    echo "Found ${AGENT_COUNT} agent files"
    [ $AGENT_COUNT -ge 40 ]; check $? "At least 40 agent files present"
else
    echo -e "${YELLOW}⚠${NC} AIT42_ROOT not set (set it to test with actual AIT42 system)"
fi

echo ""
echo "=== Verification Summary ==="
echo "PASSED: ${GREEN}${PASS}${NC}"
echo "FAILED: ${RED}${FAIL}${NC}"
echo ""

if [ $FAIL -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Run: cargo build -p ait42-ait42"
    echo "2. Run: cargo test -p ait42-ait42"
    echo "3. Set AIT42_ROOT environment variable"
    echo "4. Test with actual AIT42 system"
    exit 0
else
    echo -e "${RED}✗ Some checks failed${NC}"
    exit 1
fi
