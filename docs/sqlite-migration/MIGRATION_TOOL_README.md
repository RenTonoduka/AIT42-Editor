# SQLite Migration Tool - User Guide

## Overview

This migration tool safely migrates AIT42 Editor session data from JSON files to SQLite database.

**Features**:
- Zero-downtime migration with transaction safety
- Dry-run mode for testing
- Automatic validation
- Workspace mapping management
- Backup and rollback support

---

## Installation

### Prerequisites

```bash
# Install SQLx CLI (for database management)
cargo install sqlx-cli --no-default-features --features sqlite

# Verify installation
sqlx --version
```

### Build Migration Tool

```bash
cd src-tauri
cargo build --release --bin migrate
```

Binary location: `src-tauri/target/release/migrate`

---

## Usage

### 1. Dry Run (Recommended First Step)

Test migration without making any changes:

```bash
cd src-tauri
cargo run --bin migrate -- --dry-run
```

**Output Example**:
```
=== AIT42 Session Migration Tool ===

Connecting to database...
Running database migrations...
Database initialized successfully

Migrating JSON files to SQLite...
DRY RUN MODE - No changes will be made

Found 3 JSON files to process
[1/3] Processing: "abc123def456.json"
  Workspace: /Users/user/projects/my-app
  Sessions: 5

=== Migration Results ===
Files processed:     3
Sessions migrated:   15
Instances migrated:  45
Messages migrated:   230
Errors:              0

Dry run complete - no changes were made
```

### 2. Full Migration

Run actual migration (creates database and inserts data):

```bash
cd src-tauri
cargo run --bin migrate
```

**Workflow**:
1. Creates `~/.ait42/sessions.db` if not exists
2. Runs database migrations (creates tables, indices, triggers)
3. Scans `~/.ait42/sessions/*.json` for JSON files
4. For each JSON file:
   - Prompts for workspace path (if hash not in mapping)
   - Parses sessions
   - Inserts into SQLite (in transactions)
5. Validates integrity
6. Offers to create backup

**Output Example**:
```
=== AIT42 Session Migration Tool ===

Connecting to database...
Running database migrations...
Database initialized successfully

Migrating JSON files to SQLite...

Found 3 JSON files to process
[1/3] Processing: "abc123def456.json"

=================================================
Unknown workspace hash: abc123def456
Please enter the workspace path:
=================================================
/Users/user/projects/my-app

  Workspace: /Users/user/projects/my-app
  Sessions: 5

[2/3] Processing: "def789ghi012.json"
  Workspace: /Users/user/projects/other-app
  Sessions: 8

=== Migration Results ===
Files processed:     2
Sessions migrated:   13
Instances migrated:  39
Messages migrated:   195
Errors:              0

Validating migration...

=== Migration Validation Report ===
Sessions:           13
Instances:          39
Messages:           195
Orphaned instances: 0
Orphaned messages:  0
Invalid statuses:   0
Database size:      2 MB
Integrity check:    OK
Overall:            VALID ✓

Migration successful!

Create backup of database? (y/n): y
Creating backup: ~/.ait42/sessions_backup_20250113_143022.db
Backup created successfully!
Backup created: ~/.ait42/sessions_backup_20250113_143022.db
```

### 3. Validate Existing Database

Check integrity of existing database:

```bash
cd src-tauri
cargo run --bin migrate -- --validate
```

**Output Example**:
```
=== AIT42 Session Migration Tool ===

Connecting to database...
Database initialized successfully

Validating database...

=== Migration Validation Report ===
Sessions:           13
Instances:          39
Messages:           195
Orphaned instances: 0
Orphaned messages:  0
Invalid statuses:   0
Database size:      2 MB
Integrity check:    OK
Overall:            VALID ✓

Database validation passed!
```

### 4. Backup Database

Create backup of existing database:

```bash
cd src-tauri
cargo run --bin migrate -- --backup
```

**Output Example**:
```
=== AIT42 Session Migration Tool ===

Creating backup: ~/.ait42/sessions_backup_20250113_143500.db
Backup created successfully!

Backup created: ~/.ait42/sessions_backup_20250113_143500.db
```

---

## Workspace Mapping

The migration tool needs to map JSON filename hashes to workspace paths.

### Mapping File Location

`~/.ait42/workspace_mapping.json`

### Format

```json
{
  "abc123def456": "/Users/user/projects/my-app",
  "def789ghi012": "/Users/user/projects/other-app"
}
```

### Manual Editing

You can pre-populate this file to avoid prompts during migration:

```bash
cat > ~/.ait42/workspace_mapping.json << 'EOF'
{
  "abc123def456": "/Users/user/projects/my-app",
  "def789ghi012": "/Users/user/projects/other-app"
}
EOF
```

### Finding Hash for Workspace

Use the same hashing algorithm as the tool:

```rust
use sha2::{Digest, Sha256};

fn workspace_hash(workspace_path: &str) -> String {
    let normalized_path = match std::fs::canonicalize(workspace_path) {
        Ok(canonical) => canonical.to_string_lossy().to_string(),
        Err(_) => workspace_path.trim_end_matches('/').to_string(),
    };

    let mut hasher = Sha256::new();
    hasher.update(normalized_path.as_bytes());
    format!("{:x}", hasher.finalize())[..16].to_string()
}
```

Or find existing JSON files:

```bash
ls -la ~/.ait42/sessions/
# Look for *.json files - the filename (without .json) is the hash
```

---

## Database Location

**Default**: `~/.ait42/sessions.db`

**Related Files**:
- `~/.ait42/sessions.db-wal` - Write-Ahead Log (WAL mode)
- `~/.ait42/sessions.db-shm` - Shared memory file (WAL mode)

---

## Backup and Rollback

### Manual Backup

Before migration:

```bash
# Backup JSON files
tar -czf ~/.ait42/sessions_backup_$(date +%Y%m%d).tar.gz ~/.ait42/sessions/

# Backup existing database (if exists)
cp ~/.ait42/sessions.db ~/.ait42/sessions_backup_$(date +%Y%m%d).db
```

### Rollback to JSON

If migration fails or you need to revert:

1. **Delete SQLite database**:
   ```bash
   rm ~/.ait42/sessions.db
   rm ~/.ait42/sessions.db-wal
   rm ~/.ait42/sessions.db-shm
   ```

2. **Restore JSON files** (if archived):
   ```bash
   cd ~/.ait42
   tar -xzf sessions_backup_20250113.tar.gz
   ```

3. **Revert application code** (if needed):
   - Ensure Tauri commands use JSON-based session_history.rs
   - Remove SQLite dependencies

---

## Troubleshooting

### Issue: Migration tool can't find JSON files

**Symptoms**:
```
Found 0 JSON files to process
```

**Solution**:
```bash
# Check directory exists
ls -la ~/.ait42/sessions/

# Check for JSON files
ls ~/.ait42/sessions/*.json

# Check permissions
chmod 755 ~/.ait42/sessions/
chmod 644 ~/.ait42/sessions/*.json
```

---

### Issue: Workspace hash mismatch

**Symptoms**:
```
WARNING: Hash mismatch!
  Expected: abc123def456
  Computed: xyz789abc123
```

**Cause**: Path normalization differences (symlinks, trailing slashes)

**Solution**:
1. Use the computed hash instead
2. Manually edit `~/.ait42/workspace_mapping.json`
3. Update mapping with correct path:
   ```json
   {
     "abc123def456": "/correct/absolute/path"
   }
   ```

---

### Issue: Database locked error

**Symptoms**:
```
Error: database is locked
```

**Cause**: Another process (e.g., Tauri app) is using the database

**Solution**:
```bash
# Close AIT42 Editor application
# Kill any running processes
pkill -f ait42-editor

# Check for locks
lsof ~/.ait42/sessions.db

# If no processes, remove lock files
rm ~/.ait42/sessions.db-shm
rm ~/.ait42/sessions.db-wal

# Retry migration
```

---

### Issue: Validation fails

**Symptoms**:
```
Orphaned instances: 5
Orphaned messages:  10
Overall:            INVALID ✗
```

**Cause**: Referential integrity violations (sessions deleted but instances remain)

**Solution**:
```bash
# Connect to database
sqlite3 ~/.ait42/sessions.db

# Check orphaned records
SELECT * FROM instances WHERE session_id NOT IN (SELECT id FROM sessions);
SELECT * FROM chat_messages WHERE session_id NOT IN (SELECT id FROM sessions);

# Delete orphaned records
DELETE FROM instances WHERE session_id NOT IN (SELECT id FROM sessions);
DELETE FROM chat_messages WHERE session_id NOT IN (SELECT id FROM sessions);

# Exit SQLite
.exit

# Re-validate
cargo run --bin migrate -- --validate
```

---

### Issue: Out of memory during migration

**Symptoms**:
```
Error: Cannot allocate memory
```

**Cause**: Large JSON files or too many sessions

**Solution**:
1. **Split migration** by moving some JSON files temporarily:
   ```bash
   mkdir ~/.ait42/sessions_temp
   mv ~/.ait42/sessions/*.json ~/.ait42/sessions_temp/

   # Move back files one by one
   mv ~/.ait42/sessions_temp/file1.json ~/.ait42/sessions/
   cargo run --bin migrate

   mv ~/.ait42/sessions_temp/file2.json ~/.ait42/sessions/
   cargo run --bin migrate
   ```

2. **Increase system limits**:
   ```bash
   ulimit -v unlimited
   ulimit -m unlimited
   ```

---

## Performance Benchmarks

Expected migration time (reference: M1 MacBook Pro):

| JSON Files | Sessions | Instances | Messages | Time   |
|------------|----------|-----------|----------|--------|
| 1          | 10       | 30        | 100      | <1s    |
| 10         | 100      | 300       | 1,000    | ~3s    |
| 50         | 500      | 1,500     | 5,000    | ~15s   |
| 100        | 1,000    | 3,000     | 10,000   | ~30s   |

**Factors affecting performance**:
- Disk I/O speed (SSD vs HDD)
- JSON file size
- Number of instances per session
- Chat message count

---

## Advanced Usage

### Re-run Migration (Idempotent)

Migration uses UPSERT (INSERT ... ON CONFLICT DO UPDATE), so you can safely re-run:

```bash
# First run
cargo run --bin migrate

# Later, migrate new sessions
cargo run --bin migrate  # Only inserts new data
```

### Manual Database Management

```bash
# Connect to database
sqlite3 ~/.ait42/sessions.db

# List tables
.tables

# Show schema
.schema sessions

# Count sessions
SELECT COUNT(*) FROM sessions;

# Check database size
SELECT page_count * page_size / 1024 / 1024 AS size_mb
FROM pragma_page_count(), pragma_page_size();

# Vacuum database (reclaim space)
VACUUM;

# Exit
.exit
```

### Full-Text Search

After migration, you can search sessions:

```sql
-- Search task descriptions
SELECT * FROM sessions
WHERE id IN (
    SELECT rowid FROM sessions_fts
    WHERE sessions_fts MATCH 'database OR backend'
)
ORDER BY created_at DESC;
```

---

## Next Steps

After successful migration:

1. **Test Application**: Ensure Tauri app works with SQLite backend
2. **Archive JSON Files**: Keep for 30 days, then delete
3. **Monitor Performance**: Check query performance in application
4. **Set Up Backups**: Automate database backups (e.g., cron job)

Example backup cron job:

```bash
# Add to crontab (crontab -e)
0 2 * * * cp ~/.ait42/sessions.db ~/.ait42/sessions_backup_$(date +\%Y\%m\%d).db

# Cleanup old backups (keep last 7 days)
0 3 * * * find ~/.ait42 -name "sessions_backup_*.db" -mtime +7 -delete
```

---

## Support

If you encounter issues not covered in this guide:

1. Check logs: `tracing` output in terminal
2. Validate database: `cargo run --bin migrate -- --validate`
3. Create backup: `cargo run --bin migrate -- --backup`
4. Open GitHub issue with:
   - Migration output
   - Validation report
   - Database size
   - Number of sessions/instances/messages

---

## Implementation Details

See technical documentation:
- `/docs/sqlite-migration/SCHEMA.md` - Database schema design
- `/docs/sqlite-migration/MIGRATION_SCRIPT.md` - Implementation guide
- `/migrations/*.sql` - SQLx migration files
