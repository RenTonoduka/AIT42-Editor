# SQLite Migration - Detailed Migration Plan

**Version**: 1.0.0
**Date**: 2025-01-13
**Status**: Planning Phase

---

## Executive Summary

This document outlines the detailed execution plan for migrating from JSON-based session storage to SQLite database.

**Timeline**: 5 weeks (35 days)
**Risk Level**: Medium
**Rollback Strategy**: Parallel operation with JSON fallback

---

## Phase 0: Pre-Migration Checklist

**Duration**: 3 days
**Owner**: Backend Developer + DevOps Engineer

### Tasks

- [ ] **Backup Strategy**
  - [ ] Document current JSON file locations (`~/.ait42/sessions/*.json`)
  - [ ] Create backup script for JSON files
  - [ ] Test backup restoration procedure
  - [ ] Set up automated daily backups

- [ ] **Environment Setup**
  - [ ] Install SQLx CLI on all development machines
  - [ ] Configure DATABASE_URL in `.env`
  - [ ] Verify SQLite version (require 3.40+)
  - [ ] Set up monitoring for database file size

- [ ] **Documentation Review**
  - [ ] Review API_DESIGN.md with team
  - [ ] Review IMPLEMENTATION_SETUP.md with backend developers
  - [ ] Create runbook for emergency rollback
  - [ ] Document success criteria

- [ ] **Stakeholder Communication**
  - [ ] Notify users of upcoming migration (if applicable)
  - [ ] Schedule maintenance window (if needed)
  - [ ] Prepare rollback communication plan

### Success Criteria

- All JSON backups created and verified
- All developers have SQLx CLI installed
- Documentation reviewed and approved by tech lead

### Rollback Trigger

- Critical bug in backup/restore process
- SQLx CLI installation fails on >30% of machines

---

## Phase 1: Implementation (Week 1)

**Duration**: 7 days
**Owner**: Backend Developer

### Day 1-2: Database Module

**Tasks**:
- [ ] Create `src-tauri/src/db/` directory structure
- [ ] Implement `db/connection.rs` (connection pool)
- [ ] Implement `db/error.rs` (error types)
- [ ] Implement `db/models.rs` (type re-exports)
- [ ] Add SQLx dependencies to `Cargo.toml`

**Deliverables**:
```rust
// db/connection.rs - working connection pool
let db = Database::new().await.unwrap();
assert!(db.pool().acquire().await.is_ok());
```

**Testing**:
- Unit test: Connection pool creation
- Unit test: Migration execution
- Unit test: Error handling for missing home directory

### Day 3-4: Query Implementation

**Tasks**:
- [ ] Implement workspace queries (`upsert_workspace`, `get_workspace_id`)
- [ ] Implement session CRUD queries
- [ ] Implement instance queries
- [ ] Implement chat message queries
- [ ] Implement runtime mix queries
- [ ] Add comprehensive query tests

**Deliverables**:
```rust
// db/queries.rs - complete query module with 20+ functions
pub async fn insert_session(conn: &mut SqliteConnection, workspace_id: i64, session: &WorktreeSession) -> Result<(), String>;
pub async fn get_session_by_id(pool: &SqlitePool, workspace_path: &str, session_id: &str) -> Option<WorktreeSession>;
// ... (all queries from API_DESIGN.md)
```

**Testing**:
- Unit test: Each query function individually
- Integration test: Full session lifecycle (create → read → update → delete)
- Test: Transaction rollback on error
- Test: Concurrent access (10 parallel reads)

### Day 5-6: Tauri Commands

**Tasks**:
- [ ] Create `commands/session_history_sqlite.rs`
- [ ] Implement all 7 Tauri commands
- [ ] Update `main.rs` to integrate database
- [ ] Update `state.rs` with Database field
- [ ] Add command tests

**Deliverables**:
```rust
// commands/session_history_sqlite.rs
#[tauri::command]
pub async fn create_session(...) -> Result<WorktreeSession, String>;
#[tauri::command]
pub async fn update_session(...) -> Result<WorktreeSession, String>;
// ... (all 7 commands)
```

**Testing**:
- Integration test: Each Tauri command
- Test: Error handling (empty workspace path, invalid session ID)
- Test: Transaction atomicity

### Day 7: Review and Documentation

**Tasks**:
- [ ] Code review with tech lead
- [ ] Update API documentation
- [ ] Run `cargo sqlx prepare`
- [ ] Create PR for Phase 1

**Deliverables**:
- PR with 2000+ lines of new code
- All tests passing (≥95% coverage)
- `sqlx-data.json` committed

**Success Criteria**:
- All unit tests pass
- Integration tests pass
- Code review approved
- No compilation errors/warnings

**Rollback Trigger**:
- Test coverage <90%
- Critical bugs in transaction handling
- Performance <50% of JSON baseline

---

## Phase 2: Parallel Operation (Week 2-3)

**Duration**: 14 days
**Owner**: Backend Developer + Integration Tester

### Week 2: Hybrid Implementation

**Day 8-10: Hybrid Commands**

**Tasks**:
- [ ] Create `commands/session_history_hybrid.rs`
- [ ] Implement dual-write pattern (SQLite + JSON)
- [ ] Add comparison logic to validate consistency
- [ ] Add feature flags for storage backend selection

**Implementation Pattern**:
```rust
#[tauri::command]
pub async fn create_session(
    state: State<'_, AppState>,
    workspace_path: String,
    session: WorktreeSession,
) -> Result<WorktreeSession, String> {
    // 1. Write to SQLite (primary)
    let sqlite_result = session_history_sqlite::create_session(
        state.clone(),
        workspace_path.clone(),
        session.clone()
    ).await;

    // 2. Write to JSON (backup)
    let json_result = session_history_json::create_session(
        state.clone(),
        workspace_path.clone(),
        session.clone()
    ).await;

    // 3. Compare and log discrepancies
    match (&sqlite_result, &json_result) {
        (Ok(sqlite_session), Ok(json_session)) => {
            if !sessions_equal(sqlite_session, json_session) {
                tracing::warn!("Data inconsistency detected: SQLite vs JSON");
            }
        }
        (Ok(session), Err(e)) => {
            tracing::warn!("JSON write failed (non-critical): {}", e);
            return Ok(session.clone());
        }
        (Err(e), Ok(_)) => {
            tracing::error!("SQLite write failed (critical): {}", e);
            return Err(e.clone());
        }
        (Err(e), Err(_)) => {
            return Err(e.clone());
        }
    }

    sqlite_result
}
```

**Deliverables**:
- Hybrid commands for all 7 operations
- Comparison logic with detailed logging
- Feature flag configuration

**Testing**:
- Integration test: Verify dual writes
- Test: Data consistency validation
- Test: Graceful degradation when JSON fails

**Day 11-14: Deployment and Monitoring**

**Tasks**:
- [ ] Deploy hybrid version to development environment
- [ ] Enable detailed logging for all operations
- [ ] Set up data consistency monitoring
- [ ] Create dashboard for migration metrics

**Metrics to Monitor**:
- SQLite write success rate (target: ≥99.9%)
- JSON write success rate (baseline only)
- Data consistency rate (target: 100%)
- Average operation latency (target: ≤2x JSON baseline)
- Database file size growth rate

**Monitoring Setup**:
```rust
// Add metrics collection
pub struct MigrationMetrics {
    pub sqlite_writes: AtomicU64,
    pub json_writes: AtomicU64,
    pub sqlite_errors: AtomicU64,
    pub json_errors: AtomicU64,
    pub consistency_checks: AtomicU64,
    pub consistency_failures: AtomicU64,
}

// Log metrics every hour
#[tauri::command]
pub async fn get_migration_metrics(state: State<'_, AppState>) -> MigrationMetrics {
    // Return current metrics
}
```

**Success Criteria**:
- SQLite write success rate ≥99%
- Data consistency rate ≥99.9%
- No critical bugs reported
- Performance within acceptable range (≤3x JSON latency)

**Rollback Trigger**:
- SQLite write success rate <95%
- Data consistency rate <95%
- Critical data loss bug
- Performance degradation >5x JSON baseline

### Week 3: Validation and Optimization

**Day 15-18: Consistency Validation**

**Tasks**:
- [ ] Run automated consistency checker daily
- [ ] Investigate all inconsistencies
- [ ] Fix bugs causing inconsistencies
- [ ] Optimize slow queries

**Consistency Checker**:
```rust
#[tauri::command]
pub async fn validate_data_consistency(
    state: State<'_, AppState>,
) -> Result<ConsistencyReport, String> {
    let mut report = ConsistencyReport {
        total_sessions: 0,
        consistent: 0,
        inconsistent: 0,
        sqlite_only: 0,
        json_only: 0,
        discrepancies: Vec::new(),
    };

    // Get all workspaces from JSON
    let json_workspaces = list_json_workspaces().await?;

    for workspace_path in json_workspaces {
        // Load JSON sessions
        let json_sessions = session_history_json::get_all_sessions(
            state.clone(),
            workspace_path.clone()
        ).await?;

        // Load SQLite sessions
        let sqlite_sessions = session_history_sqlite::get_all_sessions(
            state.clone(),
            workspace_path.clone()
        ).await?;

        // Compare
        report.total_sessions += json_sessions.len();

        for json_session in &json_sessions {
            if let Some(sqlite_session) = sqlite_sessions.iter().find(|s| s.id == json_session.id) {
                if sessions_equal(json_session, sqlite_session) {
                    report.consistent += 1;
                } else {
                    report.inconsistent += 1;
                    report.discrepancies.push(Discrepancy {
                        session_id: json_session.id.clone(),
                        workspace_path: workspace_path.clone(),
                        field: detect_difference(json_session, sqlite_session),
                    });
                }
            } else {
                report.json_only += 1;
            }
        }

        // Check for SQLite-only sessions
        for sqlite_session in &sqlite_sessions {
            if !json_sessions.iter().any(|s| s.id == sqlite_session.id) {
                report.sqlite_only += 1;
            }
        }
    }

    Ok(report)
}
```

**Day 19-21: Performance Optimization**

**Tasks**:
- [ ] Run performance benchmarks
- [ ] Identify slow queries (using `EXPLAIN QUERY PLAN`)
- [ ] Add missing indexes if needed
- [ ] Optimize connection pool settings
- [ ] Test under load (1000 concurrent operations)

**Benchmark Suite**:
```rust
#[tokio::test]
async fn benchmark_operations() {
    let state = setup_test_app_state().await;

    // Create 1000 sessions
    let start = Instant::now();
    for i in 0..1000 {
        create_session(state.clone(), "/test/workspace".to_string(), create_test_session(&format!("bench-{}", i))).await.unwrap();
    }
    let create_time = start.elapsed();

    // Read all sessions
    let start = Instant::now();
    let sessions = get_all_sessions(state.clone(), "/test/workspace".to_string()).await.unwrap();
    let read_time = start.elapsed();

    // Update all sessions
    let start = Instant::now();
    for session in sessions {
        let mut updated = session.clone();
        updated.status = "completed".to_string();
        update_session(state.clone(), "/test/workspace".to_string(), updated).await.unwrap();
    }
    let update_time = start.elapsed();

    println!("Create 1000: {:?} ({:.2} ms/op)", create_time, create_time.as_millis() as f64 / 1000.0);
    println!("Read 1000: {:?}", read_time);
    println!("Update 1000: {:?} ({:.2} ms/op)", update_time, update_time.as_millis() as f64 / 1000.0);

    // Performance targets
    assert!(create_time.as_millis() < 10_000, "Create too slow"); // <10ms per operation
    assert!(read_time.as_millis() < 500, "Read too slow");
    assert!(update_time.as_millis() < 10_000, "Update too slow");
}
```

**Success Criteria**:
- Consistency rate ≥99.9%
- All discrepancies investigated and resolved
- Performance benchmarks meet targets
- Load test passes (1000 concurrent ops)

**Rollback Trigger**:
- Consistency rate <98%
- Unresolvable data corruption
- Performance degradation after optimization

---

## Phase 3: SQLite Primary (Week 4)

**Duration**: 7 days
**Owner**: Backend Developer + DevOps Engineer

### Day 22-24: Cutover Preparation

**Tasks**:
- [ ] Create feature flag for primary storage selection
- [ ] Update read commands to prefer SQLite with JSON fallback
- [ ] Test fallback mechanism thoroughly
- [ ] Prepare rollback procedure

**Read Fallback Implementation**:
```rust
#[tauri::command]
pub async fn get_all_sessions(
    state: State<'_, AppState>,
    workspace_path: String,
) -> Result<Vec<WorktreeSession>, String> {
    // Attempt SQLite first
    match session_history_sqlite::get_all_sessions(state.clone(), workspace_path.clone()).await {
        Ok(sessions) => {
            tracing::info!("Successfully read {} sessions from SQLite", sessions.len());
            Ok(sessions)
        }
        Err(e) => {
            tracing::error!("SQLite read failed, falling back to JSON: {}", e);

            // Fallback to JSON
            match session_history_json::get_all_sessions(state.clone(), workspace_path.clone()).await {
                Ok(sessions) => {
                    tracing::warn!("Fallback to JSON successful: {} sessions", sessions.len());
                    Ok(sessions)
                }
                Err(json_err) => {
                    tracing::error!("JSON fallback also failed: {}", json_err);
                    Err(format!("Both SQLite and JSON failed: {} / {}", e, json_err))
                }
            }
        }
    }
}
```

**Deliverables**:
- Feature flag: `STORAGE_BACKEND=sqlite|json|hybrid`
- Fallback-enabled read commands
- Comprehensive fallback tests

### Day 25-26: Staged Rollout

**Rollout Plan**:

**Stage 1: Internal Developers (10% of users)**
- Enable SQLite primary for 2-3 developers
- Monitor for 48 hours
- Collect feedback

**Stage 2: Beta Users (30% of users)**
- Enable for beta test group
- Monitor for 48 hours
- Address issues

**Stage 3: Full Rollout (100% of users)**
- Enable for all users
- Monitor for 7 days
- Disable JSON writes after 7 days of stability

**Monitoring Checklist**:
- [ ] Database file size within expected range
- [ ] No increase in error rates
- [ ] Latency within acceptable bounds
- [ ] No user complaints about data loss
- [ ] Fallback rate <1%

### Day 27-28: Post-Cutover Validation

**Tasks**:
- [ ] Verify all users on SQLite primary
- [ ] Run full data consistency check
- [ ] Performance benchmarks vs JSON baseline
- [ ] User acceptance testing

**Success Criteria**:
- 100% of users on SQLite primary
- Zero data loss incidents
- Performance ≥ JSON baseline
- Error rate ≤ 0.1%
- Fallback rate ≤ 1%

**Rollback Trigger**:
- Data loss incident affecting >1% of users
- Error rate >5%
- Critical performance degradation (>10x latency)
- Unrecoverable database corruption

**Rollback Procedure**:
```bash
# 1. Disable SQLite feature flag
export STORAGE_BACKEND=json

# 2. Restart application
systemctl restart ait42-editor  # or equivalent

# 3. Verify JSON mode active
tail -f ~/.ait42/logs/app.log | grep "Storage backend: json"

# 4. Investigate SQLite issues offline
# 5. Fix and re-test before re-enabling
```

---

## Phase 4: JSON Deprecation (Week 5+)

**Duration**: 7+ days
**Owner**: Backend Developer

### Day 29-31: Remove JSON Writes

**Tasks**:
- [ ] Remove JSON write operations from hybrid commands
- [ ] Make JSON read-only
- [ ] Add deprecation warnings to JSON functions
- [ ] Update documentation

**Code Changes**:
```rust
// Mark JSON commands as deprecated
#[deprecated(since = "1.7.0", note = "Use SQLite storage instead")]
pub async fn create_session_json(...) -> Result<WorktreeSession, String> {
    Err("JSON storage is deprecated. Please migrate to SQLite.".to_string())
}
```

### Day 32-35: Migration Tool

**Tasks**:
- [ ] Create JSON import tool
- [ ] Add UI for manual migration trigger
- [ ] Test import with large datasets (1000+ sessions)
- [ ] Document migration procedure for users

**Import Tool**:
```rust
#[tauri::command]
pub async fn import_legacy_json_sessions(
    state: State<'_, AppState>,
) -> Result<MigrationReport, String> {
    let sessions_dir = dirs::home_dir()
        .ok_or("Home directory not found")?
        .join(".ait42")
        .join("sessions");

    let mut report = MigrationReport {
        total_workspaces: 0,
        total_sessions: 0,
        migrated_sessions: 0,
        failed_sessions: 0,
        duration_ms: 0,
        errors: Vec::new(),
    };

    let start_time = Instant::now();

    // Iterate JSON files
    for entry in std::fs::read_dir(&sessions_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        report.total_workspaces += 1;

        // Parse JSON
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {:?}: {}", path, e))?;

        let sessions: Vec<WorktreeSession> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse {:?}: {}", path, e))?;

        report.total_sessions += sessions.len();

        // Import each session
        for session in sessions {
            // Derive workspace_path from filename (requires mapping)
            let workspace_path = derive_workspace_path_from_hash(&path)?;

            match session_history_sqlite::create_session(
                state.clone(),
                workspace_path,
                session.clone()
            ).await {
                Ok(_) => report.migrated_sessions += 1,
                Err(e) => {
                    report.failed_sessions += 1;
                    report.errors.push(format!("Failed to import {}: {}", session.id, e));
                }
            }
        }

        // Rename JSON file to .json.migrated
        std::fs::rename(&path, path.with_extension("json.migrated"))
            .map_err(|e| format!("Failed to rename {:?}: {}", path, e))?;
    }

    report.duration_ms = start_time.elapsed().as_millis() as u64;
    Ok(report)
}
```

### Day 36+: Cleanup

**Tasks**:
- [ ] Remove `session_history_json.rs`
- [ ] Remove JSON dependencies from `Cargo.toml`
- [ ] Remove JSON-related tests
- [ ] Update all documentation
- [ ] Final code review

**Success Criteria**:
- All JSON code removed
- All tests passing
- Documentation updated
- User migration guide published

---

## Rollback Strategy

### Trigger Conditions

**Critical (Immediate Rollback)**:
- Data loss affecting >1% of users
- Database corruption causing >5% of operations to fail
- Security vulnerability discovered in SQLite implementation
- Performance degradation >10x baseline

**Major (Rollback within 24 hours)**:
- Consistency issues affecting >2% of sessions
- Error rate >5% sustained for >1 hour
- Fallback rate >10%

**Minor (Fix forward, no rollback)**:
- Performance degradation 2-5x baseline
- Consistency issues <1%
- Error rate 1-5%

### Rollback Procedure

**Step 1: Disable SQLite (5 minutes)**
```bash
# Set feature flag
export STORAGE_BACKEND=json

# Restart application
# (method depends on deployment)
```

**Step 2: Verify JSON Mode (10 minutes)**
```bash
# Check logs
tail -f ~/.ait42/logs/app.log | grep "Storage backend"

# Verify operations
# - Create test session
# - Retrieve test session
# - Update test session
# - Delete test session
```

**Step 3: Notify Stakeholders (15 minutes)**
- Post incident notification
- Explain rollback reason
- Provide ETA for fix

**Step 4: Post-Mortem (24-48 hours after rollback)**
- Root cause analysis
- Corrective actions
- Prevention measures
- Re-migration plan

---

## Risk Assessment

### High Risk Items

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Data loss during migration | Low (10%) | Critical | Parallel operation phase, backups |
| Database corruption | Low (15%) | High | WAL mode, transaction safety, backups |
| Performance degradation | Medium (30%) | Medium | Benchmarking, optimization, fallback |
| Migration complexity | Medium (40%) | Medium | Phased approach, thorough testing |

### Medium Risk Items

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Consistency issues | Medium (25%) | Medium | Validation checks, monitoring |
| Rollback complexity | Low (20%) | Medium | Well-documented procedures |
| User confusion | Medium (30%) | Low | Clear communication, migration guide |

---

## Success Metrics

### Technical Metrics

- **Data Integrity**: 100% (zero data loss)
- **Consistency**: ≥99.9%
- **Performance**: ≥ JSON baseline (ideally 2-5x faster)
- **Error Rate**: ≤0.1%
- **Fallback Rate**: ≤1%
- **Test Coverage**: ≥95%

### Operational Metrics

- **Migration Downtime**: 0 hours (zero-downtime migration)
- **Rollback Events**: 0 (no rollbacks required)
- **User Complaints**: ≤5 (related to migration)
- **Developer Satisfaction**: ≥4/5

---

## Communication Plan

### Stakeholder Updates

**Weekly Updates**:
- Progress summary
- Metrics dashboard
- Issues encountered and resolved
- Next week's plan

**Phase Completion Reports**:
- Detailed phase summary
- Success criteria verification
- Lessons learned
- Go/No-Go decision for next phase

### User Communication

**Pre-Migration** (Week 0):
- Announcement of upcoming migration
- Benefits explanation (better performance, reliability)
- No action required from users

**During Migration** (Week 2-4):
- Progress updates
- Data safety assurances
- How to report issues

**Post-Migration** (Week 5+):
- Migration completion announcement
- Performance improvement statistics
- Thank users for patience

---

## Dependencies and Blockers

### Prerequisites

- SQLx CLI installed on all machines
- Database backup strategy implemented
- Monitoring infrastructure ready
- Test environment available

### External Dependencies

- SQLite 3.40+ availability on all platforms
- Sufficient disk space (~2x current JSON storage)
- Network access for CI/CD (SQLx query checking)

### Potential Blockers

- **Platform-specific issues**: SQLite behaves differently on different OSes
  - **Mitigation**: Test on all target platforms (macOS, Linux, Windows)

- **Concurrent access issues**: Multiple app instances accessing same database
  - **Mitigation**: WAL mode, proper locking, busy timeout

- **Large dataset migration**: Users with 1000+ sessions
  - **Mitigation**: Batch processing, progress reporting, background migration

---

## Post-Migration Activities

### Week 6: Stabilization

- Monitor for late-emerging issues
- Optimize based on production metrics
- Address user feedback
- Complete documentation

### Week 7-8: Enhancement

- Implement advanced features (full-text search, analytics)
- Add database maintenance tools (vacuum, optimize)
- Performance tuning based on usage patterns

### Week 9+: Long-term Maintenance

- Regular database backups (automated)
- Monitoring and alerting
- Capacity planning
- Schema evolution (future migrations)

---

## Appendix: Emergency Contacts

**Tech Lead**: [Name, Email, Phone]
**Backend Developer**: [Name, Email, Phone]
**DevOps Engineer**: [Name, Email, Phone]
**Integration Tester**: [Name, Email, Phone]

**Escalation Path**:
1. Backend Developer (0-2 hours)
2. Tech Lead (2-4 hours)
3. CTO/Engineering Manager (4+ hours)

---

**Document Version**: 1.0.0
**Last Updated**: 2025-01-13
**Next Review**: Weekly during migration phases

**Related Documents**:
- API_DESIGN.md
- IMPLEMENTATION_SETUP.md
- TESTING_GUIDE.md (to be created)
