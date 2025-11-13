# SQLite Migration Documentation

This directory contains comprehensive documentation for migrating AIT42 Editor's session history from JSON files to SQLite database.

## 📚 Documentation Index

### 📘 Core Architecture Documents (RECOMMENDED START HERE)

#### 1. [ARCHITECTURE.md](./ARCHITECTURE.md) - Complete System Architecture ⭐
**Primary audience**: Architects, Technical Leads, Stakeholders

**THE MOST COMPREHENSIVE DOCUMENT** covering all aspects of the migration:
- ✅ Executive Summary (benefits: 25x performance, timeline: 8-10 weeks)
- ✅ Current State Analysis (data model, storage, access patterns)
- ✅ Architecture Decision Records (ADRs)
  - ADR-001: SQLite + SQLx selection rationale
  - ADR-002: ait42-session crate design  - ADR-003: 4-phase migration strategy
- ✅ System Architecture (C4 diagrams: Context, Container, Component)
- ✅ Technology Stack (SQLite 3.45+, SQLx 0.7.3, configuration)
- ✅ Complete 4-Phase Migration Strategy
  - Phase 1: Foundation (Weeks 1-2)
  - Phase 2: Dual Write (Weeks 3-5)
  - Phase 3: SQLite Primary (Weeks 6-8)
  - Phase 4: Complete Migration (Weeks 9-10)
- ✅ Quality Attributes (Performance, Reliability, Security, Maintainability)
- ✅ Risk Assessment & Mitigation (detailed risk matrix)
- ✅ Implementation Roadmap (week-by-week breakdown)
- ✅ Stakeholder Approval Process

**Read this FIRST** for complete understanding of the migration architecture.

---

#### 2. [IMPLEMENTATION_GUIDE.md](./IMPLEMENTATION_GUIDE.md) - Hands-On Implementation ⭐
**Primary audience**: Backend Developers, Database Engineers

**PRACTICAL CODE EXAMPLES** for Phase 1 implementation:
- ✅ Quick start guide (prerequisites, setup commands)
- ✅ Complete Phase 1 code examples (~2000 lines)
  - Crate structure (`crates/ait42-session/`)
  - Cargo.toml configuration
  - Database schema SQL (with indexes, constraints)
  - Domain models (WorktreeSession, WorktreeInstance, ChatMessage)
  - Repository pattern (trait + SQLite implementation)
  - SQLite connection pool (with optimization pragmas)
  - Complete unit tests (integration_tests.rs)
- ✅ Development workflow
  - Running tests (`cargo test -p ait42-session`)
  - Running migrations (`sqlx migrate run`)
  - SQLx offline mode for CI/CD
- ✅ Troubleshooting common issues (database locked, SQLx errors)

**Read this SECOND** to start implementing the migration.

---

#### 3. [IMPACT_ANALYSIS.md](./IMPACT_ANALYSIS.md) - Change Impact & Testing ⭐
**Primary audience**: All Developers, QA Engineers, Project Managers

**COMPREHENSIVE CHANGE ANALYSIS** covering all affected files:
- ✅ Summary statistics (24+ files affected, breakdown by category)
- ✅ Complete list of new files
  - `crates/ait42-session/` (15+ new files, ~2000 lines)
  - Migration files, unit tests, documentation
- ✅ Modified files with line-by-line analysis
  - Backend: 4 files (Medium impact)
  - Frontend: 3 files (Low impact - minimal changes!)
  - Configuration: 2 files
- ✅ Dependency graph changes (before/after visualization)
- ✅ Data migration path (JSON → SQLite)
- ✅ Complete testing strategy
  - Unit tests (30-40 tests in ait42-session)
  - Integration tests (15-20 tests in src-tauri)
  - Performance tests (benchmark suite with Criterion)
  - E2E tests (user workflows)
- ✅ Rollback plans for each phase (detailed procedures)
- ✅ Success metrics (technical + user satisfaction)
- ✅ FAQ section (20+ common questions answered)

**Read this THIRD** to understand scope, testing needs, and rollback procedures.

---

### 📖 Detailed Technical Documents (Supplementary)

#### 4. [SCHEMA.md](./SCHEMA.md) - Database Schema Design
**Database Design Document** (Senior Database Architect level)

Comprehensive schema design including:
- ✅ Entity-Relationship Diagrams
- ✅ Table definitions with constraints and indexes
- ✅ Normalization analysis (3NF)
- ✅ Query optimization examples
- ✅ Performance benchmarks
- ✅ Migration strategy (zero-downtime)
- ✅ SQLx migration files
- ✅ Data validation procedures

**Key Sections**:
- Executive Summary
- Schema Design (sessions, instances, chat_messages)
- Indexing Strategy
- Query Optimization (N+1 prevention, pagination, aggregation)
- Migration Strategy (dual-write, rollback plan)
- Performance Metrics (P50/P95/P99 latency targets)
- ADRs (Architecture Decision Records)

---

### 2. [MIGRATION_SCRIPT.md](./MIGRATION_SCRIPT.md) - Implementation Guide
**Data Migration Script** (Implementation level)

Step-by-step implementation guide:
- ✅ SQLx setup and configuration
- ✅ Database connection pool
- ✅ Query module (CRUD operations)
- ✅ Migration script (JSON → SQLite)
- ✅ CLI tool for running migration
- ✅ Unit tests
- ✅ Validation procedures
- ✅ Rollback plan
- ✅ Troubleshooting guide

**Key Sections**:
- Phase 1: Preparation (dependencies, setup)
- Phase 2: Query Module (Rust implementation)
- Phase 3: Migration Script (JSON parser, transaction handling)
- Phase 4: CLI Tool (standalone migration tool)
- Phase 5: Testing (unit tests, integration tests)
- Phase 6: Execution (running migration)
- Phase 7: Rollback Plan (recovery procedures)

---

### 3. [API_DESIGN.md](./API_DESIGN.md) - Data Access API Design (NEW)
**API Architecture Document** (Senior API Architect level)

Complete API design specification for SQLite migration:
- ✅ Existing API analysis (JSON-based patterns)
- ✅ SQLite schema design with STRICT mode
- ✅ Tauri command design (maintains existing signatures)
- ✅ Rust implementation patterns with SQLx
- ✅ SQLx query examples with compile-time checking
- ✅ Error handling strategy (DatabaseError types)
- ✅ Frontend integration (zero changes required)
- ✅ Testing strategy (unit, integration, performance)
- ✅ Performance considerations (connection pooling, WAL mode)

**Key Sections**:
- Existing API Analysis (current JSON implementation)
- SQLite Schema Design (workspaces, sessions, instances, chat_messages)
- Tauri Command Design (7 commands with SQLite backend)
- SQLx Query Implementation (compile-time checked queries)
- Error Handling Strategy (transaction rollback, retry logic)
- Frontend Integration (TypeScript types, Zustand store)
- Migration Strategy (4-phase gradual rollout)
- Testing Strategy (unit, integration, performance benchmarks)
- Performance Considerations (2-5x improvement targets)
- Rollback Plan (emergency procedures)

**Target Audience**: API Architects, Backend Developers, Frontend Developers

---

### 4. [IMPLEMENTATION_SETUP.md](./IMPLEMENTATION_SETUP.md) - Developer Setup Guide (NEW)
**Setup and Configuration Document** (Implementation level)

Step-by-step setup guide for developers:
- ✅ Cargo dependencies configuration
- ✅ SQLx CLI installation and usage
- ✅ Database migration files creation
- ✅ Compile-time query checking setup
- ✅ Directory structure setup
- ✅ Module templates and examples
- ✅ Testing infrastructure setup
- ✅ Environment variables configuration
- ✅ Build commands (dev and release)
- ✅ Troubleshooting guide
- ✅ Performance tuning options

**Key Sections**:
- Cargo Dependencies (SQLx, thiserror)
- SQLx CLI Setup (install, create database, run migrations)
- Migration Files Setup (initial schema SQL)
- Compile-Time Query Checking (prepare, CI/CD)
- Directory Structure Setup (db/ module)
- Module File Templates (connection, queries, error)
- SQLx Query Examples (basic, transaction, dynamic)
- Testing Setup (in-memory database, fixtures)
- Environment Variables (DATABASE_URL, RUST_LOG)
- Build Commands (features, offline mode)
- Troubleshooting (common issues, solutions)
- Performance Tuning (PRAGMA, connection pool)

**Target Audience**: Backend Developers, DevOps Engineers

---

### 5. [MIGRATION_PLAN.md](./MIGRATION_PLAN.md) - Phased Migration Plan (NEW)
**Project Management Document** (Tech Lead level)

Detailed phased migration execution plan:
- ✅ Phase 0: Pre-migration checklist (backups, environment)
- ✅ Phase 1: Implementation (Week 1 - database module, commands)
- ✅ Phase 2: Parallel operation (Week 2-3 - dual write, validation)
- ✅ Phase 3: SQLite primary (Week 4 - cutover, monitoring)
- ✅ Phase 4: JSON deprecation (Week 5+ - cleanup)
- ✅ Rollback strategy and procedures
- ✅ Risk assessment and mitigation
- ✅ Success metrics and KPIs
- ✅ Communication plan
- ✅ Emergency procedures

**Key Sections**:
- Phase 0: Pre-Migration Checklist (backup strategy, environment setup)
- Phase 1: Implementation (database module, queries, Tauri commands)
- Phase 2: Parallel Operation (hybrid implementation, consistency validation)
- Phase 3: SQLite Primary (cutover preparation, staged rollout)
- Phase 4: JSON Deprecation (remove JSON writes, migration tool)
- Rollback Strategy (trigger conditions, procedures)
- Risk Assessment (high/medium risk items, mitigation)
- Success Metrics (data integrity, performance, reliability)
- Communication Plan (stakeholder updates, user communication)
- Dependencies and Blockers (prerequisites, external dependencies)

**Target Audience**: Project Managers, Tech Leads, Backend Developers, DevOps Engineers

---

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install SQLx CLI
cargo install sqlx-cli --no-default-features --features sqlite

# Verify installation
sqlx --version
```

### Step 1: Review Schema Design

```bash
# Read schema documentation
cat docs/sqlite-migration/SCHEMA.md

# Review ER diagrams (Mermaid format)
# Use VS Code with Mermaid extension or online viewer
```

### Step 2: Run Migrations

```bash
# Navigate to Tauri directory
cd src-tauri

# Run SQLx migrations (creates database and schema)
sqlx migrate run --database-url sqlite://$HOME/.ait42/sessions.db

# Verify migrations
sqlx migrate info --database-url sqlite://$HOME/.ait42/sessions.db
```

### Step 3: Migrate Data

```bash
# Build migration tool
cargo build --release --bin migrate

# Run migration (interactive - will prompt for workspace paths)
./target/release/migrate

# Or use cargo run
cargo run --bin migrate
```

### Step 4: Validate

```bash
# Check database integrity
sqlite3 ~/.ait42/sessions.db "PRAGMA integrity_check;"

# Count migrated records
sqlite3 ~/.ait42/sessions.db "SELECT COUNT(*) FROM sessions;"
sqlite3 ~/.ait42/sessions.db "SELECT COUNT(*) FROM instances;"
sqlite3 ~/.ait42/sessions.db "SELECT COUNT(*) FROM chat_messages;"
```

---

## 📊 Migration Timeline

### Phase 1: Design & Review (Week 1)
- [x] Schema design document
- [x] ER diagram creation
- [x] ADRs (Architecture Decision Records)
- [ ] Stakeholder review
- [ ] Security review

### Phase 2: Implementation (Week 2)
- [ ] Database module implementation
- [ ] Query functions (CRUD)
- [ ] Migration script
- [ ] CLI tool

### Phase 3: Testing (Week 3)
- [ ] Unit tests
- [ ] Integration tests
- [ ] Load testing (1000+ sessions)
- [ ] Performance benchmarking

### Phase 4: Deployment (Week 4)
- [ ] Dual-write implementation (JSON + SQLite)
- [ ] Data migration (production)
- [ ] Monitoring setup
- [ ] Documentation updates

### Phase 5: Cleanup (Week 5+)
- [ ] Remove JSON code paths
- [ ] Archive JSON files
- [ ] Performance tuning
- [ ] Post-mortem review

---

## 🎯 Success Criteria

### Data Integrity
- ✅ Zero data loss during migration
- ✅ All foreign key constraints validated
- ✅ PRAGMA integrity_check passes
- ✅ Referential integrity (no orphaned records)

### Performance
- ✅ P50 query latency: <10ms
- ✅ P95 query latency: <50ms
- ✅ P99 query latency: <100ms
- ✅ Database size: <1GB (initial), <10GB (10x growth)

### Reliability
- ✅ Zero downtime during migration
- ✅ Rollback plan tested
- ✅ Backup strategy in place
- ✅ Monitoring dashboards configured

---

## 📁 File Structure

```
docs/sqlite-migration/
├── README.md                          # This file (documentation index)
│
├── **CORE ARCHITECTURE DOCUMENTS (START HERE)**
├── ARCHITECTURE.md                    # ⭐ Complete system architecture (MUST READ FIRST)
├── IMPLEMENTATION_GUIDE.md            # ⭐ Hands-on implementation with code examples
├── IMPACT_ANALYSIS.md                 # ⭐ Change impact, testing, and rollback plans
│
├── **DETAILED TECHNICAL DOCUMENTS**
├── SCHEMA.md                          # Database schema design (database architect)
├── MIGRATION_SCRIPT.md                # Legacy implementation guide (data migration)
├── API_DESIGN.md                      # Data access API design (API architect)
├── IMPLEMENTATION_SETUP.md            # Developer setup guide (implementation)
└── MIGRATION_PLAN.md                  # Phased migration plan (project management)

migrations/
├── 20250113_000000_initial_schema.sql     # Create tables and indexes
├── 20250113_000001_add_denormalized_counts.sql  # Performance optimization
└── 20250113_000002_add_fts_search.sql     # Full-text search

src-tauri/src/db/                      # Database module (NEW structure)
├── mod.rs                             # Module exports
├── connection.rs                      # Connection pool management
├── queries.rs                         # SQLx query functions
├── models.rs                          # Database model types
├── error.rs                           # Database error types
└── migration.rs                       # JSON → SQLite migration logic

src-tauri/src/commands/
├── session_history.rs                 # Original JSON implementation (legacy)
├── session_history_sqlite.rs          # NEW: SQLite implementation
└── session_history_hybrid.rs          # NEW: Parallel operation (Phase 2)

src-tauri/src/bin/
└── migrate.rs                         # CLI migration tool

src-tauri/tests/
├── integration_test.rs                # Integration tests
└── performance_test.rs                # Performance benchmarks
```

---

## 🔍 Key Features

### 1. Zero-Downtime Migration
- **Dual-write period**: Write to both JSON and SQLite for 2 weeks
- **Gradual rollout**: Test on small datasets first
- **Rollback safety**: Keep JSON files for 30 days as backup

### 2. Performance Optimizations
- **Indexing**: 15 indexes covering all common queries
- **Denormalization**: Optional counters (instance_count, message_count)
- **Full-text search**: FTS5 virtual table for task search
- **WAL mode**: Better concurrency (multiple readers + 1 writer)

### 3. Data Integrity
- **Foreign keys**: ON DELETE CASCADE prevents orphaned records
- **CHECK constraints**: Validate status values, type values
- **UNIQUE constraints**: Prevent duplicate (session_id, instance_id) pairs
- **Transaction guarantees**: ACID compliance

### 4. Developer Experience
- **SQLx compile-time checking**: Type-safe queries
- **Migrations**: Version-controlled schema changes
- **CLI tool**: Standalone migration utility
- **Comprehensive tests**: Unit, integration, load tests

---

## 🛠️ Troubleshooting

### Common Issues

#### 1. Migration Tool Can't Find JSON Files
```bash
# Check directory exists
ls -la ~/.ait42/sessions/

# If not, create test session
mkdir -p ~/.ait42/sessions
echo '[]' > ~/.ait42/sessions/test.json
```

#### 2. Database Locked Error
```bash
# Close all connections
killall -9 AIT42-Editor

# Check for locks
lsof ~/.ait42/sessions.db

# If persistent, delete WAL files
rm ~/.ait42/sessions.db-wal
rm ~/.ait42/sessions.db-shm
```

#### 3. Workspace Hash Mismatch
```bash
# Manually create mapping file
cat > ~/.ait42/workspace_mapping.json << EOF
{
  "abc123def456": "/Users/user/my-project",
  "xyz789ghi012": "/Users/user/another-project"
}
EOF
```

#### 4. SQLx Offline Mode (No Internet)
```bash
# Prepare offline mode (with internet)
cargo sqlx prepare --database-url sqlite://$HOME/.ait42/sessions.db

# This creates sqlx-data.json for offline compilation
git add sqlx-data.json
```

---

## 📖 Additional Resources

### SQLite Documentation
- [SQLite Documentation](https://www.sqlite.org/docs.html)
- [SQLite Performance Tuning](https://www.sqlite.org/performance.html)
- [SQLite Full-Text Search (FTS5)](https://www.sqlite.org/fts5.html)
- [SQLite Write-Ahead Logging](https://www.sqlite.org/wal.html)

### SQLx Documentation
- [SQLx GitHub](https://github.com/launchbadge/sqlx)
- [SQLx Documentation](https://docs.rs/sqlx/latest/sqlx/)
- [SQLx Migrations](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli)

### Database Design
- [Database Normalization](https://en.wikipedia.org/wiki/Database_normalization)
- [SQL Anti-Patterns](https://pragprog.com/titles/bksqla/sql-antipatterns/)
- [High Performance MySQL](https://www.oreilly.com/library/view/high-performance-mysql/9781449332471/)

### AIT42 Editor Documentation
- [Session History System](../SESSION_HISTORY.md)
- [Tauri Integration](../TAURI_INTEGRATION_GUIDE.md)
- [Architecture Overview](../architecture/)

---

## 🤝 Contributing

### Adding New Migrations

1. Create migration file:
   ```bash
   sqlx migrate add <migration_name>
   ```

2. Write up migration (SQL):
   ```sql
   -- migrations/YYYYMMDD_HHMMSS_<migration_name>.sql
   CREATE TABLE new_table (...);
   ```

3. Test migration:
   ```bash
   sqlx migrate run --database-url sqlite://$HOME/.ait42/sessions_test.db
   ```

4. Document changes in SCHEMA.md

### Running Tests

```bash
# Unit tests
cargo test --package ait42-editor database::

# Integration tests
cargo test --package ait42-editor --test integration_tests

# Load tests (requires test data)
cargo test --package ait42-editor --release -- --ignored
```

### Code Review Checklist

- [ ] Schema changes documented in SCHEMA.md
- [ ] Migration script includes up and down paths
- [ ] Indexes added for new columns used in WHERE/ORDER BY
- [ ] Foreign key constraints defined
- [ ] Unit tests added for new queries
- [ ] Performance benchmarks run
- [ ] No SQL injection vulnerabilities (use parameterized queries)

---

## 📝 Changelog

### v1.0.0 (2025-01-13) - Initial Design
- Created comprehensive schema design document
- Designed 3NF normalized schema (sessions, instances, chat_messages)
- Defined 15 indexes for query optimization
- Wrote migration script with validation
- Created CLI tool for data migration
- Documented ADRs for key design decisions

### Planned
- v1.1.0: Add metrics table for analytics
- v1.2.0: Implement database backup/restore commands
- v1.3.0: Add query caching layer (Redis)
- v2.0.0: Migrate to PostgreSQL for multi-user support

---

## 🎓 Learning Resources

### For Database Developers
1. Read [SCHEMA.md](./SCHEMA.md) sections:
   - Table Definitions (understand schema)
   - Indexing Strategy (performance optimization)
   - Query Optimization Examples (best practices)

2. Read [MIGRATION_SCRIPT.md](./MIGRATION_SCRIPT.md) sections:
   - Phase 2: Query Module (Rust implementation)
   - Phase 3: Migration Script (data transformation)

### For Backend Developers
1. Read [MIGRATION_SCRIPT.md](./MIGRATION_SCRIPT.md) sections:
   - Phase 1: Preparation (SQLx setup)
   - Phase 2: Query Module (API examples)
   - Phase 6: Execution (running migration)

### For DevOps Engineers
1. Read [SCHEMA.md](./SCHEMA.md) sections:
   - Migration Strategy (zero-downtime deployment)
   - Performance Metrics (monitoring targets)
   - Risk Assessment (failure scenarios)

2. Read [MIGRATION_SCRIPT.md](./MIGRATION_SCRIPT.md) sections:
   - Phase 6: Execution (deployment process)
   - Phase 7: Rollback Plan (disaster recovery)

---

## 📧 Support

For questions or issues:
1. Check [Troubleshooting](#troubleshooting) section
2. Review [Additional Resources](#additional-resources)
3. Open GitHub issue with:
   - Error logs
   - Database schema version (run `sqlx migrate info`)
   - Steps to reproduce
   - Expected vs actual behavior

---

## 📄 License

This documentation is part of AIT42 Editor and follows the same license as the main project.

---

**Last Updated**: 2025-01-13
**Status**: Draft - Pending Stakeholder Review
**Next Review**: 2025-01-20
