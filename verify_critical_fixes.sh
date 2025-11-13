#!/usr/bin/env bash
# Verification script for SQLite Migration Phase 1 Critical Fixes
# This script verifies all 5 critical issues have been resolved

set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SQLite Migration Phase 1 - Critical Fixes Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASS=0
FAIL=0

# Helper functions
pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASS++))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    ((FAIL++))
}

info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

# ============================================================================
# Issue 1: Schema-Code Inconsistency (workspace_path vs workspace_hash)
# ============================================================================
echo "Issue 1: Schema-Code Inconsistency"
echo "-----------------------------------"

if grep -q "workspace_hash TEXT NOT NULL" migrations/20250113_000000_initial_schema.sql; then
    pass "Schema uses workspace_hash (not workspace_path)"
else
    fail "Schema still uses workspace_path instead of workspace_hash"
fi

if grep -q "workspace_hash TEXT NOT NULL" migrations/20250113_000000_initial_schema.sql && \
   grep -q "FOREIGN KEY (workspace_hash)" migrations/20250113_000000_initial_schema.sql; then
    pass "Foreign key references workspace_hash correctly"
else
    fail "Foreign key reference incorrect"
fi

echo ""

# ============================================================================
# Issue 2: Missing workspaces Table
# ============================================================================
echo "Issue 2: Missing workspaces Table"
echo "----------------------------------"

if grep -q "CREATE TABLE IF NOT EXISTS workspaces" migrations/20250113_000000_initial_schema.sql; then
    pass "workspaces table definition exists"
else
    fail "workspaces table definition missing"
fi

if grep -q "hash TEXT PRIMARY KEY NOT NULL" migrations/20250113_000000_initial_schema.sql && \
   grep -q "path TEXT NOT NULL" migrations/20250113_000000_initial_schema.sql; then
    pass "workspaces table has required columns (hash, path)"
else
    fail "workspaces table missing required columns"
fi

if grep -q "idx_workspaces_path" migrations/20250113_000000_initial_schema.sql; then
    pass "workspaces table has proper indexes"
else
    fail "workspaces table missing indexes"
fi

echo ""

# ============================================================================
# Issue 3: Missing dirs Dependency
# ============================================================================
echo "Issue 3: Missing dirs Dependency"
echo "---------------------------------"

if grep -q 'dirs = "5.0"' crates/ait42-session/Cargo.toml; then
    pass "dirs dependency added to Cargo.toml"
else
    fail "dirs dependency missing from Cargo.toml"
fi

if grep -q "dirs::home_dir()" crates/ait42-session/src/db/connection.rs; then
    pass "dirs::home_dir() usage confirmed in connection.rs"
else
    info "dirs::home_dir() usage not found (may have been refactored)"
fi

echo ""

# ============================================================================
# Issue 4: N+1 Query Problem
# ============================================================================
echo "Issue 4: N+1 Query Problem"
echo "--------------------------"

if grep -q "batch_load_instances" crates/ait42-session/src/db/queries.rs; then
    pass "batch_load_instances() function exists"
else
    fail "batch_load_instances() function missing"
fi

if grep -q "batch_load_chat_messages" crates/ait42-session/src/db/queries.rs; then
    pass "batch_load_chat_messages() function exists"
else
    fail "batch_load_chat_messages() function missing"
fi

if grep -q "use std::collections::HashMap" crates/ait42-session/src/db/queries.rs; then
    pass "HashMap imported for batch grouping"
else
    fail "HashMap import missing"
fi

# Check that old N+1 pattern is gone from get_all_sessions
if grep -A 10 "pub async fn get_all_sessions" crates/ait42-session/src/db/queries.rs | \
   grep -q "batch_load_instances"; then
    pass "get_all_sessions() uses batch loading (N+1 fixed)"
else
    fail "get_all_sessions() may still have N+1 query problem"
fi

echo ""

# ============================================================================
# Issue 5: Non-Idempotent Migrations
# ============================================================================
echo "Issue 5: Non-Idempotent Migrations"
echo "-----------------------------------"

if grep -q "DROP TRIGGER IF EXISTS update_instance_count_insert" migrations/20250113_000001_add_denormalized_counts.sql; then
    pass "Trigger drops use IF EXISTS (idempotent)"
else
    fail "Triggers not idempotent (missing DROP IF EXISTS)"
fi

TRIGGER_COUNT=$(grep -c "DROP TRIGGER IF EXISTS" migrations/20250113_000001_add_denormalized_counts.sql || true)
if [ "$TRIGGER_COUNT" -eq 4 ]; then
    pass "All 4 triggers have DROP IF EXISTS ($TRIGGER_COUNT/4)"
else
    fail "Not all triggers have DROP IF EXISTS ($TRIGGER_COUNT/4)"
fi

if grep -q "COALESCE(instance_count, 0)" migrations/20250113_000001_add_denormalized_counts.sql; then
    pass "Triggers use COALESCE for NULL safety"
else
    fail "Triggers missing COALESCE NULL handling"
fi

echo ""

# ============================================================================
# Additional Verification: Schema Consistency
# ============================================================================
echo "Additional Checks"
echo "-----------------"

if grep -q "session_type TEXT NOT NULL" migrations/20250113_000000_initial_schema.sql; then
    pass "Column renamed to session_type (not just 'type')"
else
    fail "Column name may be inconsistent (should be session_type)"
fi

if grep -q "instance_id INTEGER" migrations/20250113_000000_initial_schema.sql; then
    pass "instances table has instance_id column"
else
    fail "instances table missing instance_id"
fi

if grep -q "CREATE INDEX IF NOT EXISTS" migrations/20250113_000000_initial_schema.sql; then
    pass "Indexes use IF NOT EXISTS (idempotent)"
else
    info "Indexes may not be fully idempotent"
fi

echo ""

# ============================================================================
# Summary
# ============================================================================
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Verification Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "Passed: ${GREEN}$PASS${NC}"
echo -e "Failed: ${RED}$FAIL${NC}"
echo ""

if [ "$FAIL" -eq 0 ]; then
    echo -e "${GREEN}✓ All critical issues verified as FIXED!${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Run: cargo check -p ait42-session"
    echo "  2. Run: cargo test -p ait42-session"
    echo "  3. Test migrations with: ./test_migrations.sh"
    echo ""
    exit 0
else
    echo -e "${RED}✗ Some issues remain. Please review failed checks.${NC}"
    echo ""
    exit 1
fi
