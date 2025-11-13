# SQLite Migration - Impact Analysis

## Overview

This document provides a detailed analysis of files affected by the SQLite migration and the specific changes required for each file.

---

## Summary Statistics

| Category | Count | Impact Level |
|----------|-------|--------------|
| **New Files** | 15+ | N/A |
| **Modified Files (Rust)** | 4 | Medium |
| **Modified Files (TypeScript)** | 3 | Low |
| **Configuration Files** | 2 | Low |
| **Total Affected Files** | 24+ | Low-Medium |

**Overall Risk Assessment**: 🟡 Medium (Mitigated by gradual migration)

---

## Affected Files by Category

### 1. New Files (To Be Created)

#### Crate Structure
```
crates/ait42-session/
├── Cargo.toml                          # NEW - Crate configuration
├── .env                                # NEW - SQLx configuration
├── src/
│   ├── lib.rs                          # NEW - Public API
│   ├── db/
│   │   ├── mod.rs                      # NEW - Database module
│   │   ├── connection.rs               # NEW - Connection pool
│   │   └── queries.rs                  # NEW - Compiled queries (optional)
│   ├── models/
│   │   ├── mod.rs                      # NEW - Model exports
│   │   ├── session.rs                  # NEW - WorktreeSession model
│   │   ├── instance.rs                 # NEW - WorktreeInstance model
│   │   └── message.rs                  # NEW - ChatMessage model
│   ├── repository/
│   │   ├── mod.rs                      # NEW - Repository traits
│   │   └── sqlite.rs                   # NEW - SQLite implementation
│   ├── migration/
│   │   ├── mod.rs                      # NEW - Migration module
│   │   ├── json_importer.rs            # NEW - JSON → SQLite importer
│   │   └── validator.rs                # NEW - Data validation
│   └── error.rs                        # NEW - Custom error types
├── migrations/
│   ├── 20250113_001_initial_schema.sql # NEW - Initial schema
│   ├── 20250113_002_add_indexes.sql    # NEW - Performance indexes
│   └── 20250113_003_add_fts.sql        # NEW - Full-text search (optional)
├── tests/
│   ├── integration_tests.rs            # NEW - Integration tests
│   └── fixtures/                       # NEW - Test data
│       └── sample_sessions.json
└── sqlx-data.json                      # NEW - Offline query data (generated)
```

**Estimated Lines of Code**: ~2,000 lines

---

### 2. Modified Files (Backend - Rust)

#### 2.1 `Cargo.toml` (Workspace Root)

**File**: `/home/user/AIT42-Editor/Cargo.toml`

**Impact**: Low (Add new crate to workspace)

**Changes**:
```diff
 [workspace]
 members = [
     "ait42-bin",
     "src-tauri",
     "crates/ait42-core",
     "crates/ait42-tui",
     "crates/ait42-lsp",
     "crates/ait42-ait42",
     "crates/ait42-fs",
     "crates/ait42-config",
     "crates/omega-theory",
     "crates/llm-estimator",
+    "crates/ait42-session",  # NEW CRATE
 ]
```

**Testing**: `cargo build --workspace` should succeed

---

#### 2.2 `src-tauri/Cargo.toml`

**File**: `/home/user/AIT42-Editor/src-tauri/Cargo.toml`

**Impact**: Low (Add dependency)

**Changes**:
```diff
 [dependencies]
 # Tauri framework
 tauri = { version = "1.5", features = [ "dialog-message", "dialog-open", "shell-open", "fs-all"] }
 serde = { version = "1.0", features = ["derive"] }
 serde_json = "1.0"

 # Async runtime
 tokio = { version = "1.35", features = ["full"] }
 async-trait = "0.1"
 futures = "0.3"

 # AIT42 core crates
 ait42-core = { path = "../crates/ait42-core" }
 ait42-tui = { path = "../crates/ait42-tui", optional = true }
 ait42-lsp = { path = "../crates/ait42-lsp" }
 ait42-fs = { path = "../crates/ait42-fs" }
 ait42-config = { path = "../crates/ait42-config" }
 ait42-ait42 = { path = "../crates/ait42-ait42" }
+ait42-session = { path = "../crates/ait42-session" }  # NEW DEPENDENCY
```

**Testing**: `cargo build -p ait42-editor` should succeed

---

#### 2.3 `src-tauri/src/commands/session_history.rs`

**File**: `/home/user/AIT42-Editor/src-tauri/src/commands/session_history.rs`

**Impact**: High (Major refactoring)

**Current Implementation**: 350 lines of JSON file-based storage

**Migration Strategy**:

**Phase 1**: No changes (create new crate alongside)

**Phase 2**: Wrap with compatibility layer
```rust
// src-tauri/src/commands/session_history_compat.rs (NEW FILE)
use ait42_session::{SessionRepository, SqliteSessionRepository};
use std::sync::Arc;

pub struct DualWriteAdapter {
    json_backend: JsonBackend,  // Original implementation
    sqlite_backend: Arc<dyn SessionRepository>,
}

impl DualWriteAdapter {
    pub async fn create_session(&self, session: WorktreeSession) -> Result<WorktreeSession> {
        // Write to JSON (primary)
        let json_result = self.json_backend.create_session(session.clone())?;

        // Async write to SQLite
        let sqlite = self.sqlite_backend.clone();
        let session_clone = session.clone();
        tokio::spawn(async move {
            let _ = sqlite.create_session(session_clone).await;
        });

        Ok(json_result)
    }

    // Similar wrappers for other methods...
}
```

**Phase 3**: Switch to SQLite primary
```rust
// Update existing commands to use SQLite with JSON fallback
#[tauri::command]
pub async fn get_all_sessions(
    state: State<'_, AppState>,
    workspace_path: String,
) -> Result<Vec<WorktreeSession>, String> {
    let workspace_hash = workspace_hash(&workspace_path);

    // Read from SQLite
    match state.session_repo.get_all_sessions(&workspace_hash).await {
        Ok(sessions) => Ok(sessions),
        Err(e) => {
            tracing::error!("SQLite read failed: {}, falling back to JSON", e);
            // Fallback to JSON
            load_sessions(&state, &workspace_path)
        }
    }
}
```

**Phase 4**: Remove JSON code entirely
```rust
// Direct SQLite usage
#[tauri::command]
pub async fn get_all_sessions(
    state: State<'_, AppState>,
    workspace_path: String,
) -> Result<Vec<WorktreeSession>, String> {
    let workspace_hash = workspace_hash(&workspace_path);
    state.session_repo
        .get_all_sessions(&workspace_hash)
        .await
        .map_err(|e| e.to_string())
}
```

**Lines Changed**:
- Phase 1: 0 lines
- Phase 2: ~200 lines (add compatibility layer)
- Phase 3: ~100 lines (switch reads)
- Phase 4: -150 lines (remove JSON code)

**Testing Requirements**:
- [ ] All existing Tauri commands still work
- [ ] Data integrity validated
- [ ] Performance benchmarks pass

---

#### 2.4 `src-tauri/src/state.rs`

**File**: `/home/user/AIT42-Editor/src-tauri/src/state.rs`

**Impact**: Medium (Add repository to AppState)

**Current State** (estimated):
```rust
pub struct AppState {
    // Existing fields...
}
```

**Phase 2 Changes**:
```diff
+use ait42_session::SqliteSessionRepository;
+use std::sync::Arc;

 pub struct AppState {
+    pub session_repo: Arc<SqliteSessionRepository>,  # NEW FIELD
     // Existing fields...
 }
```

**Initialization** (in `main.rs`):
```rust
#[tokio::main]
async fn main() {
    let db_path = dirs::home_dir()
        .unwrap()
        .join(".ait42")
        .join("sessions.db");

    let session_repo = SqliteSessionRepository::new(
        &format!("sqlite://{}", db_path.display())
    )
    .await
    .expect("Failed to initialize session repository");

    tauri::Builder::default()
        .manage(AppState {
            session_repo: Arc::new(session_repo),
            // ... existing fields
        })
        // ... rest of Tauri setup
}
```

**Lines Changed**: ~15 lines

**Testing**: Application should start without errors

---

#### 2.5 `src-tauri/src/commands/ait42.rs`

**File**: `/home/user/AIT42-Editor/src-tauri/src/commands/ait42.rs`

**Impact**: Low (Update session creation calls)

**Affected Lines**: Lines 1829, 1868

**Current Usage**:
```rust
let session = crate::commands::session_history::WorktreeSession {
    id: competition_id.clone(),
    // ... fields
};

// Later: Manual JSON serialization
let mut sessions: Vec<crate::commands::session_history::WorktreeSession> =
    serde_json::from_str(&content).unwrap_or_default();
```

**Phase 2 Changes**:
```diff
-use crate::commands::session_history::WorktreeSession;
+use ait42_session::WorktreeSession;

 // Session creation remains the same, but goes through adapter
-create_session(state, workspace_path, session).await?;
+state.session_adapter.create_session(session).await?;
```

**Lines Changed**: ~10 lines

**Testing**: Competition mode session creation should work

---

### 3. Modified Files (Frontend - TypeScript)

#### 3.1 `src/store/sessionHistoryStore.ts`

**File**: `/home/user/AIT42-Editor/src/store/sessionHistoryStore.ts`

**Impact**: Low (No breaking changes to store API)

**Current Implementation**: 392 lines of Zustand store

**Changes Required**: None for Phase 1-3

**Optional Optimization (Phase 4)**:
```typescript
// Add optimistic updates for better UX
createSession: async (session: WorktreeSession) => {
  const { workspacePath } = get();

  // Optimistic update
  set((state) => ({
    sessions: [session, ...state.sessions],
    activeSessionId: session.id,
  }));

  try {
    await tauriApi.createSession(workspacePath, session);
  } catch (error) {
    // Rollback on error
    set((state) => ({
      sessions: state.sessions.filter((s) => s.id !== session.id),
      error: error instanceof Error ? error.message : 'Failed to create session',
    }));
    throw error;
  }
},
```

**Lines Changed**: 0-50 lines (optional)

**Testing**: All frontend tests should pass

---

#### 3.2 `src/types/worktree.ts`

**File**: `/home/user/AIT42-Editor/src/types/worktree.ts`

**Impact**: None (Type definitions remain the same)

**Changes Required**: None

**Reason**: SQLite schema matches existing TypeScript types

**Verification**: Compare Rust `WorktreeSession` with TypeScript `WorktreeSession`

---

#### 3.3 `src/services/tauri.ts`

**File**: `/home/user/AIT42-Editor/src/services/tauri.ts`

**Impact**: None (Tauri command signatures unchanged)

**Changes Required**: None

**Reason**: Tauri commands maintain same interface (only backend implementation changes)

**Verification**: Type checking should pass (`tsc --noEmit`)

---

### 4. Configuration Files

#### 4.1 `.gitignore`

**File**: `/home/user/AIT42-Editor/.gitignore`

**Impact**: Low (Add database files to ignore)

**Changes**:
```diff
+# SQLite database files (user data, not committed)
+*.db
+*.db-shm
+*.db-wal
+
+# SQLx prepared query data (committed for offline builds)
+!sqlx-data.json
```

**Lines Changed**: ~5 lines

---

#### 4.2 CI/CD Configuration (GitHub Actions, etc.)

**File**: `.github/workflows/*.yml` (if exists)

**Impact**: Medium (Add SQLx offline mode)

**Changes**:
```yaml
- name: Build Rust
  env:
    SQLX_OFFLINE: true  # Use pre-generated query metadata
  run: cargo build --release
```

**Testing**: CI/CD pipeline should pass

---

## Dependency Graph Changes

### Before Migration
```
src-tauri
  ├── ait42-core
  ├── ait42-tui
  ├── ait42-lsp
  ├── ait42-fs
  ├── ait42-config
  └── ait42-ait42
```

### After Migration
```
src-tauri
  ├── ait42-core
  ├── ait42-tui
  ├── ait42-lsp
  ├── ait42-fs
  ├── ait42-config
  ├── ait42-ait42
  └── ait42-session  # NEW DEPENDENCY
      └── sqlx
```

**Binary Size Impact**: +1.4 MB (7% increase from 20MB baseline)

---

## Data Migration Path

### User Data Location

**Before Migration**:
```
~/.ait42/sessions/
├── a1b2c3d4e5f6g7h8.json  # Workspace 1 sessions
├── 9i0j1k2l3m4n5o6p.json  # Workspace 2 sessions
└── ...
```

**After Migration (Phase 2-3)**:
```
~/.ait42/
├── sessions/
│   ├── a1b2c3d4e5f6g7h8.json  # Backup (Phase 2-3)
│   └── ...
└── sessions.db                 # SQLite database (new)
```

**After Migration (Phase 4)**:
```
~/.ait42/
├── sessions/
│   ├── a1b2c3d4e5f6g7h8.json.archive  # Archived
│   └── ...
└── sessions.db                         # Primary storage
```

### Migration Tool

**Command**: Automatic on first launch (Phase 2)

**Manual Trigger** (for advanced users):
```bash
# CLI tool for manual migration
ait42-migrate --workspace /path/to/workspace

# Dry run (validation only)
ait42-migrate --workspace /path/to/workspace --dry-run

# Force re-import
ait42-migrate --workspace /path/to/workspace --force
```

**Implementation**: See `crates/ait42-session/src/migration/json_importer.rs`

---

## Testing Strategy

### Unit Tests

**New Tests** (in `ait42-session`):
- [ ] Database connection and initialization
- [ ] CRUD operations for sessions
- [ ] CRUD operations for instances
- [ ] CRUD operations for messages
- [ ] Transaction rollback on error
- [ ] Concurrent access (multiple readers/writers)
- [ ] Data validation (constraints, foreign keys)
- [ ] Search and filtering

**Estimated Test Count**: 30-40 tests

---

### Integration Tests

**Modified Tests** (in `src-tauri`):
- [ ] Session history commands (all existing tests)
- [ ] Competition mode session creation
- [ ] Ensemble mode session creation
- [ ] Debate mode session creation
- [ ] Data persistence across app restarts

**New Tests**:
- [ ] JSON → SQLite migration
- [ ] Dual-write consistency validation
- [ ] Fallback to JSON on SQLite error
- [ ] Database corruption recovery

**Estimated Test Count**: 15-20 tests

---

### Performance Tests

**Benchmarks** (using Criterion.rs):
- [ ] Get all sessions (100, 1000, 10000 sessions)
- [ ] Create session
- [ ] Update session
- [ ] Delete session
- [ ] Search by type
- [ ] Search by status
- [ ] Complex query (multiple filters)
- [ ] Full-text search

**Target**: 25x improvement over JSON baseline

---

### End-to-End Tests

**User Workflows**:
- [ ] Create competition session → Monitor → Complete → View history
- [ ] Create ensemble session → Start integration → Complete
- [ ] Create debate session → Execute rounds → Complete
- [ ] Search sessions by type and status
- [ ] Delete old sessions
- [ ] Migrate from old version (JSON) to new version (SQLite)

---

## Rollback Plan

### Phase 2 Rollback (Dual Write)

**Scenario**: Critical bugs in SQLite implementation

**Steps**:
1. Disable SQLite writes (feature flag)
2. Continue using JSON as primary
3. Roll back to previous version
4. No data loss (JSON still being written)

**Time to Rollback**: <1 hour

---

### Phase 3 Rollback (SQLite Primary)

**Scenario**: Performance regression or data integrity issues

**Steps**:
1. Re-enable JSON as primary (feature flag)
2. Export SQLite data to JSON (backup tool)
3. Roll back to Phase 2 implementation
4. Investigate and fix issues

**Time to Rollback**: 1-2 hours

---

### Phase 4 Rollback (SQLite Only)

**Scenario**: Severe bugs discovered in production

**Steps**:
1. Use archived JSON files to restore data
2. Roll back to Phase 3 implementation
3. User notification and manual intervention
4. Data export tool for user recovery

**Time to Rollback**: 2-4 hours (manual intervention required)

---

## Communication Plan

### Developer Communication

**Announcement**: Week before Phase 1 kickoff

**Channels**:
- Team meeting presentation
- Slack/Discord announcement
- Email to all developers
- Update in project README

**Key Messages**:
- New `ait42-session` crate available
- API documentation published
- Migration timeline and phases
- Who to contact for questions

---

### User Communication

**Phase 2 Announcement**: "Improved session management coming soon"

**Phase 3 Announcement**: "Session loading is now 25x faster!"

**Phase 4 Announcement**: "Migration complete - thank you for your patience"

**Channels**:
- In-app notification
- Release notes
- User documentation update
- Blog post (optional)

---

## Success Metrics

### Technical Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Performance** | 25x improvement | Benchmark suite |
| **Reliability** | 99.99% uptime | Error logging |
| **Data Integrity** | 0 data loss | Validation tests |
| **Test Coverage** | >80% | `cargo tarpaulin` |
| **Build Time** | <5% increase | CI/CD logs |

### User Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **User Satisfaction** | >90% positive | Survey |
| **Bug Reports** | <5 critical bugs | Issue tracker |
| **Adoption Rate** | 100% migrated | Telemetry |
| **Support Tickets** | <10 migration issues | Support dashboard |

---

## Appendix: File Change Checklist

### Phase 1: Foundation

- [ ] Create `crates/ait42-session/` directory
- [ ] Add `Cargo.toml` for new crate
- [ ] Update workspace `Cargo.toml`
- [ ] Write SQLite schema migration files
- [ ] Implement repository layer
- [ ] Write unit tests
- [ ] Document API (rustdoc)

### Phase 2: Dual Write

- [ ] Update `src-tauri/Cargo.toml` (add dependency)
- [ ] Create `session_history_compat.rs` (compatibility layer)
- [ ] Update `state.rs` (add repository field)
- [ ] Update `main.rs` (initialize repository)
- [ ] Implement JSON importer
- [ ] Add migration UI
- [ ] Write integration tests

### Phase 3: SQLite Primary

- [ ] Switch read operations to SQLite
- [ ] Implement JSON fallback
- [ ] Add performance monitoring
- [ ] Run beta testing
- [ ] Fix critical bugs

### Phase 4: Complete Migration

- [ ] Remove JSON backend code
- [ ] Update all Tauri commands
- [ ] Add JSON archive tool
- [ ] Update documentation
- [ ] Prepare release notes
- [ ] Deploy and monitor

---

## Questions & Answers

**Q: Will users lose their existing sessions?**
A: No. Phase 2 automatically imports JSON data to SQLite. Users can roll back to JSON at any time during Phase 2-3.

**Q: What happens if SQLite database is corrupted?**
A: During Phase 2-3, JSON files serve as backup. Phase 4 includes integrity checks and automatic recovery.

**Q: How long does migration take?**
A: For 1000 sessions: <5 minutes. Migration runs in background on first launch.

**Q: Can I use the app during migration?**
A: Yes. Migration runs asynchronously. Session operations may be slightly slower during migration.

**Q: What if I need to downgrade to an older version?**
A: Phase 2-3: No action needed (JSON still exists). Phase 4: Use archive tool to restore JSON files.

---

## Contact

**Architecture Questions**: Backend Team Lead
**Implementation Questions**: Migration Team
**Testing Questions**: QA Team
**Documentation**: Technical Writer

---

**Document Version**: 1.0
**Last Updated**: 2025-01-13
**Status**: 🟡 Proposed
