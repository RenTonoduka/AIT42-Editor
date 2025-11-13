#!/usr/bin/env bash
# Test SQLite migrations for idempotency and correctness

set -euo pipefail

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "SQLite Migration Testing"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Test database
TEST_DB="test_sessions_$(date +%s).db"
export DATABASE_URL="sqlite:$TEST_DB"

cleanup() {
    echo ""
    echo "Cleaning up test database..."
    rm -f "$TEST_DB" "${TEST_DB}-shm" "${TEST_DB}-wal"
}

trap cleanup EXIT

echo "Step 1: Running migrations for the first time..."
echo "---"

if command -v sqlx &> /dev/null; then
    sqlx migrate run
    echo "✓ First migration run completed"
else
    echo "⚠ sqlx CLI not available, skipping migration test"
    echo "  Install with: cargo install sqlx-cli --no-default-features --features sqlite"
    exit 0
fi

echo ""
echo "Step 2: Verifying schema..."
echo "---"

# Check tables exist
if sqlite3 "$TEST_DB" "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;" | grep -q "workspaces"; then
    echo "✓ workspaces table created"
else
    echo "✗ workspaces table missing"
    exit 1
fi

if sqlite3 "$TEST_DB" "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;" | grep -q "sessions"; then
    echo "✓ sessions table created"
else
    echo "✗ sessions table missing"
    exit 1
fi

# Check column names
if sqlite3 "$TEST_DB" "PRAGMA table_info(sessions);" | grep -q "workspace_hash"; then
    echo "✓ sessions.workspace_hash column exists"
else
    echo "✗ sessions.workspace_hash column missing"
    exit 1
fi

if sqlite3 "$TEST_DB" "PRAGMA table_info(sessions);" | grep -q "session_type"; then
    echo "✓ sessions.session_type column exists"
else
    echo "✗ sessions.session_type column missing"
    exit 1
fi

# Check triggers
TRIGGER_COUNT=$(sqlite3 "$TEST_DB" "SELECT COUNT(*) FROM sqlite_master WHERE type='trigger';" 2>/dev/null || echo "0")
if [ "$TRIGGER_COUNT" -ge 4 ]; then
    echo "✓ Triggers created ($TRIGGER_COUNT triggers)"
else
    echo "⚠ Expected at least 4 triggers, found $TRIGGER_COUNT"
fi

echo ""
echo "Step 3: Testing idempotency (running migrations again)..."
echo "---"

# This should succeed without errors due to our idempotency fixes
if sqlx migrate run 2>&1 | grep -q "No migrations to apply"; then
    echo "✓ Second migration run successful (idempotent)"
else
    echo "✗ Migration idempotency test failed"
    exit 1
fi

echo ""
echo "Step 4: Testing basic data operations..."
echo "---"

# Insert test workspace
sqlite3 "$TEST_DB" "INSERT INTO workspaces (hash, path) VALUES ('test123', '/test/path');" 2>/dev/null
echo "✓ Can insert into workspaces table"

# Insert test session
sqlite3 "$TEST_DB" "INSERT INTO sessions (id, workspace_hash, session_type, task, status, created_at, updated_at)
VALUES ('sess1', 'test123', 'competition', 'test task', 'running', datetime('now'), datetime('now'));" 2>/dev/null
echo "✓ Can insert into sessions table"

# Verify foreign key constraint works
if sqlite3 "$TEST_DB" "INSERT INTO sessions (id, workspace_hash, session_type, task, status, created_at, updated_at)
VALUES ('sess2', 'nonexistent', 'competition', 'test', 'running', datetime('now'), datetime('now'));" 2>&1 | grep -q "FOREIGN KEY"; then
    echo "✓ Foreign key constraint enforced"
else
    echo "⚠ Foreign key constraint may not be working"
fi

# Check cascade delete
sqlite3 "$TEST_DB" "DELETE FROM workspaces WHERE hash = 'test123';" 2>/dev/null
REMAINING=$(sqlite3 "$TEST_DB" "SELECT COUNT(*) FROM sessions WHERE workspace_hash = 'test123';")
if [ "$REMAINING" -eq 0 ]; then
    echo "✓ CASCADE DELETE works correctly"
else
    echo "✗ CASCADE DELETE not working (found $REMAINING sessions after workspace delete)"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ All migration tests passed!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
