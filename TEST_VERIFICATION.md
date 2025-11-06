# Test Verification Guide: Competition Mode Output Fix

## Pre-Test Checklist

- [x] Old binary killed (PID 7305)
- [x] Clean build completed (2.3GB removed)
- [x] New binary running (PID 23100, started 4:43PM)
- [x] Vite dev server running (port 5173)
- [x] Application initialized successfully

## Test Plan

### Test 1: Competition Mode Basic Execution

**Objective**: Verify that output logs are displayed in UI

**Steps**:
1. Open AIT42 Editor application
2. Navigate to Competition Mode panel
3. Create a new competition:
   - Task: "List files in current directory using ls"
   - Instances: 3
   - Click "Start Competition"

**Expected Results**:
- ✅ Backend logs show: `📤 Sent X bytes for instance N` (incremental output)
- ✅ Backend logs show: `✅ Sent final output for instance N (Y bytes)` (completion)
- ✅ UI displays output for each instance
- ✅ Instance status updates to "completed" (green checkmark icon)
- ✅ No "No output yet..." message persists after completion

**Log File Verification**:
```bash
# Check log files exist
ls -lh src-tauri/.worktrees/competition-*/instance-*/.claude-output.log

# View log contents
cat src-tauri/.worktrees/competition-*/instance-1/.claude-output.log
```

**Backend Log Verification**:
```bash
# Monitor tauri dev logs
tail -f /tmp/tauri-final-rebuild.log | grep -E "(✅|📤|❌|⚠️|competition|instance)"
```

### Test 2: Empty Output Handling

**Objective**: Verify graceful handling when no output is captured

**Steps**:
1. Create competition with task that produces minimal/no output
2. Wait for completion

**Expected Results**:
- ✅ Backend logs show: `⚠️ Log file for instance N is empty`
- ✅ UI displays: "⚠️ No output captured"
- ✅ Instance status: "completed" (not "error")

### Test 3: Error Case Handling

**Objective**: Verify error propagation to UI

**Steps**:
1. Create competition with invalid task (e.g., syntax error)
2. Wait for completion or error

**Expected Results**:
- ✅ Backend logs show: `❌ Failed to read log file` OR error event
- ✅ UI displays error message or status
- ✅ Instance status: "error" (red alert icon)

### Test 4: Multi-Instance Parallel Execution

**Objective**: Verify all instances receive events correctly

**Steps**:
1. Create competition with 4 instances
2. Task: "Count to 10 with 1 second delay between each number"
3. Monitor all instances simultaneously

**Expected Results**:
- ✅ All 4 instances show incremental output
- ✅ All 4 instances complete successfully
- ✅ Backend logs show 4x `✅ Sent final output` messages
- ✅ No instances stuck in "running" state

### Test 5: Tmux Session Management

**Objective**: Verify tmux sessions are properly monitored

**Steps**:
1. Start competition
2. During execution, check tmux sessions:
   ```bash
   tmux ls | grep claude-code-comp
   ```
3. Wait for completion
4. Verify sessions are cleaned up

**Expected Results**:
- ✅ Tmux sessions exist during execution
- ✅ Sessions terminate after completion
- ✅ Log files persist after session termination
- ✅ No orphaned tmux sessions

## Debugging Commands

### Check Running Processes
```bash
ps aux | grep "AIT42 Editor"
ps aux | grep "tauri dev"
ps aux | grep "vite"
```

### Monitor Backend Logs
```bash
# Real-time monitoring
tail -f /tmp/tauri-final-rebuild.log

# Filter for competition events
tail -f /tmp/tauri-final-rebuild.log | grep -E "(competition|instance|✅|❌|⚠️|📤)"

# Check for event emission
tail -f /tmp/tauri-final-rebuild.log | grep "competition-output"
```

### Check Log Files
```bash
# List all competition log files
find src-tauri/.worktrees -name ".claude-output.log" -ls

# View latest log
ls -t src-tauri/.worktrees/competition-*/instance-1/.claude-output.log | head -1 | xargs cat

# Check log sizes
find src-tauri/.worktrees -name ".claude-output.log" -exec ls -lh {} \;
```

### Verify Tmux Sessions
```bash
# List active sessions
tmux ls

# View session output
tmux capture-pane -pt claude-code-comp-{id}-{n} -S -

# Kill stuck sessions
tmux kill-session -t claude-code-comp-{id}-{n}
```

### Browser DevTools (Frontend Debugging)
```javascript
// Open DevTools (Cmd+Option+I on Mac)
// Console tab - Check for event listeners
console.log("Listening for: competition-output");

// Monitor Tauri events
window.__TAURI__.event.listen('competition-output', (event) => {
  console.log('Received event:', event.payload);
});
```

## Success Criteria

### ✅ Pass Conditions
1. All 5 tests pass without errors
2. Backend logs show consistent `✅ Sent final output` messages
3. UI displays output for all instances
4. No "No output yet..." messages after completion
5. Error cases show meaningful messages to user

### ❌ Fail Conditions
1. Backend logs missing `✅ Sent final output` messages
2. UI shows "No output yet..." after completion
3. Events emitted but not received by UI
4. Orphaned tmux sessions remain after completion
5. Application crashes or hangs

## Rollback Plan

If tests fail:

```bash
# 1. Stop application
pkill -9 -f "tauri dev"
killall -9 "AIT42 Editor"

# 2. Revert to last known good commit
cd /Users/tonodukaren/Programming/AI/02_Workspace/05_Client/03_Sun/AIT42-Editor
git log --oneline -10  # Find last good commit
git checkout <commit-hash>

# 3. Clean rebuild
cargo clean --manifest-path=src-tauri/Cargo.toml
cargo build --manifest-path=src-tauri/Cargo.toml

# 4. Restart
npm run tauri dev
```

## Known Issues & Limitations

### Current Warnings (Non-Critical)
- `unused variable: competition_id` (line 763)
- `value assigned to last_output is never read` (line 541)
- `struct Position is never constructed` (editor.rs:16)
- `fields buffer_manager and config are never read` (state.rs:28)

**Impact**: None - these are code quality warnings, not runtime errors.

### Platform-Specific Notes
- **macOS**: Tested on Darwin 25.0.0
- **Tmux**: Version 3.x+ required
- **Claude Code CLI**: Must be installed and in PATH

## Post-Test Actions

After successful testing:

1. **Document Results**:
   - Update BUG_FIX_REPORT.md with test outcomes
   - Add screenshots/recordings to `docs/evidence/`

2. **Commit Changes** (if tests pass):
   ```bash
   git add BUG_FIX_REPORT.md TEST_VERIFICATION.md
   git commit -m "fix: resolve Competition Mode output event emission

   - Fixed old binary issue via full clean rebuild
   - Added comprehensive logging for event emission
   - Verified all instances receive completion events

   Test Results: 5/5 passed

   🤖 Generated with [Claude Code](https://claude.ai/code)

   Co-Authored-By: Claude <noreply@anthropic.com>"
   ```

3. **Monitor Production**:
   - Watch for event emission logs in production
   - Collect user feedback on Competition Mode UX
   - Add telemetry for event delivery rates

---

**Created**: 2025-11-06
**Last Updated**: 2025-11-06 16:43 JST
**Status**: Ready for Testing
