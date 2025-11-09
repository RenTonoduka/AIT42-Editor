# Bug Fix: CompetitionDialog Timeout Error Handling

## Bug Summary

**Issue**: CompetitionDialog UI froze at "Claude Codeがタスクを分析中..." when analysis timed out
**Impact**: Users unable to start Competition, no error feedback
**Severity**: P1 (High) - Blocks core functionality
**Status**: FIXED

## Root Cause Analysis

### Problem Location

**File**: `src/components/AI/CompetitionDialog.tsx:85-95`

**Buggy Code**:
```typescript
useEffect(() => {
  if (!task.trim() || task.trim().length < 10) {
    return;
  }

  const debounceTimer = setTimeout(() => {
    analyze(task.trim());  // ❌ Promise not awaited, errors not caught
  }, 1500);

  return () => clearTimeout(debounceTimer);
}, [task, analyze]);
```

### Why It Failed

1. **Unhandled Promise**: `analyze()` returns a Promise, but was not awaited in setTimeout
2. **No Try-Catch**: Timeout errors from backend (120s) were not caught in UI layer
3. **State Stuck**: Error state was set in `useTaskOptimizer`, but UI didn't react properly
4. **Poor UX**: No visual feedback for errors, users assumed the app was broken

### Error Flow

```
User types "AIT42最適化したいです"
    ↓
CompetitionDialog triggers analyze() after 1.5s debounce
    ↓
useTaskOptimizer.analyze() calls tauriApi.analyzeTaskWithClaudeCode()
    ↓
Backend timeout after 120s (Claude Code CLI takes too long)
    ↓
useTaskOptimizer sets state.status = 'error'
    ↓
❌ CompetitionDialog useEffect doesn't catch the error
    ↓
❌ UI shows "分析中..." forever (isAnalyzing stuck at true)
    ↓
User is confused, cannot proceed
```

## Fix Implementation

### 1. Error Handling (CompetitionDialog.tsx)

**Before**:
```typescript
const debounceTimer = setTimeout(() => {
  analyze(task.trim());  // ❌ No error handling
}, 1500);
```

**After**:
```typescript
const debounceTimer = setTimeout(async () => {
  try {
    await analyze(task.trim());  // ✅ Awaited
  } catch (error) {
    // ✅ Caught and logged (state already updated in useTaskOptimizer)
    console.warn('[CompetitionDialog] Auto-analysis failed (non-critical):', error);
  }
}, 1500);
```

**Impact**:
- Errors are now caught and logged
- UI state properly reflects error condition
- Non-blocking: user can still proceed

### 2. Improved Error Messages (CompetitionDialog.tsx)

**Before**:
```tsx
{optimizerState.status === 'error' && (
  <div className="px-4 py-3 bg-red-900/20 border border-red-700/30 rounded-lg">
    <span className="text-sm text-red-300">
      分析エラー: {optimizerState.error}
    </span>
  </div>
)}
```

**After**:
```tsx
{optimizerState.status === 'error' && (
  <div className="px-4 py-3 bg-yellow-900/20 border border-yellow-700/30 rounded-lg">
    <div className="flex flex-col gap-2">
      <span className="text-sm font-semibold text-yellow-300">
        ⚠️ 自動分析失敗
      </span>
      <span className="text-xs text-yellow-400/80">
        {optimizerState.error}
      </span>
      <span className="text-xs text-yellow-500/70">
        手動でインスタンス数を設定してCompetitionを開始できます（推奨: 3インスタンス）
      </span>
    </div>
  </div>
)}
```

**Impact**:
- Clear error indication (⚠️ icon)
- Actionable guidance (fallback to manual settings)
- Less alarming color (yellow warning vs red error)

### 3. User Guidance During Analysis (CompetitionDialog.tsx)

**Before**:
```tsx
{isAnalyzing && (
  <div className="flex items-center gap-3 ...">
    <Loader2 className="animate-spin" />
    <span>Claude Codeがタスクを分析中...</span>
  </div>
)}
```

**After**:
```tsx
{isAnalyzing && (
  <div className="flex flex-col gap-2 ...">
    <div className="flex items-center gap-3">
      <Loader2 className="animate-spin" />
      <span>Claude Codeがタスクを分析中...</span>
    </div>
    <span className="text-xs text-purple-400/70">
      分析完了を待たずにCompetitionを開始することもできます
    </span>
  </div>
)}
```

**Impact**:
- Users know they can proceed immediately
- Analysis is optional, not blocking

## Regression Tests

### Test File: `src/components/AI/__tests__/CompetitionDialog.test.tsx`

**Test Cases**:

1. ✅ **Timeout Error Handling**
   - Simulates 120s timeout from backend
   - Verifies error message appears
   - Verifies fallback guidance is shown

2. ✅ **Competition Start After Error**
   - Analysis fails with timeout
   - User can still start Competition
   - Default 3 instances used

3. ✅ **Dialog Closable During Analysis**
   - Long-running analysis in progress
   - User can close dialog (cancel operation)

4. ✅ **Success Path**
   - Analysis completes successfully
   - Recommended instances applied
   - Success message shown

5. ✅ **Empty Error Message**
   - Backend returns error with no message
   - Fallback message displayed
   - No UI crash

### Running Tests

```bash
npm run test src/components/AI/__tests__/CompetitionDialog.test.tsx
```

## Prevention Strategy

### Code Review Checklist

For future async operations in React components:

- [ ] **Always await Promises** in useEffect/setTimeout
- [ ] **Add try-catch** around async calls
- [ ] **Provide user-friendly error messages**
- [ ] **Implement fallback behavior** (graceful degradation)
- [ ] **Never block UI** on optional operations
- [ ] **Test error paths** (not just happy path)

### Error Handling Pattern

```typescript
// ✅ Recommended pattern for optional async operations
useEffect(() => {
  if (!input.trim()) return;

  const timer = setTimeout(async () => {
    try {
      await optionalAsyncOperation(input);
    } catch (error) {
      console.warn('[Component] Optional operation failed (non-critical):', error);
      // UI should show error but allow user to proceed
    }
  }, DEBOUNCE_MS);

  return () => clearTimeout(timer);
}, [input]);
```

### UX Guidelines

1. **Distinguish Warning vs Error**
   - Warning (yellow): Operation failed but user can continue
   - Error (red): Critical failure, user cannot proceed

2. **Provide Actionable Guidance**
   - What went wrong?
   - What can user do about it?
   - What's the fallback option?

3. **Progressive Enhancement**
   - Core functionality works without enhancements
   - Analysis/optimization is a bonus, not requirement
   - Users can always override AI suggestions

## Performance Impact

### Before Fix
- Analysis timeout: 120s → UI frozen for 120s
- User cannot proceed until refresh

### After Fix
- Analysis timeout: 120s → Error shown after 120s
- User can start Competition at any time
- Analysis is background operation

## Related Files

- `src/components/AI/CompetitionDialog.tsx` - Main UI component
- `src/hooks/useTaskOptimizer.ts` - Analysis state management
- `src/services/tauri.ts` - Backend API interface
- `src-tauri/src/commands/claude_code.rs` - Backend implementation

## References

- Issue: #N/A (internal bug report)
- PR: #N/A (direct commit)
- Related: Ω-theory optimization feature (v1.6.0-omega)

---

**Fix Date**: 2025-11-09
**Author**: Claude Code (bug-fixer agent)
**Reviewed By**: User (tonodukaren)
