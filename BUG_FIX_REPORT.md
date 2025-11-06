# Bug Fix Report: Handshake Protocol Failure

**Date**: 2025-11-06
**Issue**: Competition/Ensemble Mode "No output yet..." persists after rebuild
**Status**: FIXED (pending verification)

---

## Bug Analysis

### Issue
- **Type**: Event Handshake Protocol Timing Race Condition
- **Location**: `src-tauri/src/commands/ait42.rs:754-796` (before fix)
- **Symptom**: After Tauri rebuild, Ensemble Mode execution shows "No output yet..." indefinitely

### Root Cause

**Critical Timing Race Condition**: Backend registered listener **AFTER** spawning async tasks

**Timeline (BUGGY)**:
```
T+25ms:  Backend returns OK to frontend
T+30ms:  Frontend emits 'competition-listener-ready'
T+40ms:  Backend async task STARTS registering listener ❌ TOO LATE
```

**Fix**: Register listener **BEFORE** any async operations (Line 669)

**Timeline (FIXED)**:
```
T+5ms:   Backend registers listener 🎯 READY
T+35ms:  Frontend emits signal ✅ RECEIVED
T+36ms:  Backend fires → monitoring starts
```

---

## Verification Steps

### Backend Logs (Expected)
```
🕐 [HANDSHAKE] Registering global listener BEFORE creating worktrees
⏳ [HANDSHAKE] Instance 1 waiting for frontend ready signal...
🔔 [HANDSHAKE] Received event on 'competition-listener-ready'
✅ [HANDSHAKE] Frontend ready signal received
🚀 Starting monitoring for instance 1
📤 Preparing to emit event 'competition-output'
```

### Frontend Console (Expected)
```
[Frontend] Registering listener for competition <UUID>
[Frontend] Ready signal sent for competition <UUID>
[Frontend] Received competition-output event: instance=1
```

### Visual Test
1. Restart dev server: `npm run tauri dev`
2. Run Ensemble Mode (2 instances)
3. **Expected**: Output appears within 2-3 seconds

---

## Files Modified

- `/Users/tonodukaren/Programming/AI/02_Workspace/05_Client/03_Sun/AIT42-Editor/src-tauri/src/commands/ait42.rs` (Lines 658-836)

**Key Changes**:
- Moved listener registration to line 669 (before worktree loop)
- Added comprehensive `[HANDSHAKE]` logging
- Shared `Arc<Mutex<bool>>` for all async tasks to wait on
- Automatic cleanup after 10 seconds

---

## Testing

```bash
# Clean rebuild test
cd /Users/tonodukaren/Programming/AI/02_Workspace/05_Client/03_Sun/AIT42-Editor
npm run tauri dev

# Open browser: http://localhost:5173/
# Run AI Task → Ensemble Mode → 2 instances → "ファイル一覧を表示"
# Verify output appears within 3 seconds
```

**Success Criteria**:
- Backend logs show `✅ [HANDSHAKE] Frontend ready signal received`
- Frontend shows real-time output streaming
- No timeout warnings in logs
