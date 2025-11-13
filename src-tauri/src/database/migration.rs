/**
 * Migration Module - JSON to SQLite data migration
 *
 * Safely migrates session data from JSON files to SQLite database.
 */
use super::queries::{SessionRow, InstanceRow, ChatMessageRow};
use super::queries::{insert_session, insert_instance, insert_chat_message};
use sqlx::SqlitePool;
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use glob::glob;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// WorktreeSession from JSON (existing format)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeSession {
    pub id: String,
    pub r#type: String,
    pub task: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub instances: Vec<WorktreeInstance>,
    pub chat_history: Vec<ChatMessage>,
    pub model: Option<String>,
    pub timeout_seconds: Option<u32>,
    pub preserve_worktrees: Option<bool>,
    pub winner_id: Option<u32>,
    pub runtime_mix: Option<Vec<String>>,
    pub total_duration: Option<u64>,
    pub total_files_changed: Option<u32>,
    pub total_lines_added: Option<u32>,
    pub total_lines_deleted: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeInstance {
    pub instance_id: u32,
    pub worktree_path: String,
    pub branch: String,
    pub agent_name: String,
    pub status: String,
    pub tmux_session_id: String,
    pub output: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub files_changed: Option<u32>,
    pub lines_added: Option<u32>,
    pub lines_deleted: Option<u32>,
    pub runtime: Option<String>,
    pub model: Option<String>,
    pub runtime_label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
    pub instance_id: Option<u32>,
}

/// Migration statistics
#[derive(Debug, Default)]
pub struct MigrationStats {
    pub files_processed: usize,
    pub sessions_migrated: usize,
    pub instances_migrated: usize,
    pub messages_migrated: usize,
    pub errors: Vec<String>,
}

impl std::fmt::Display for MigrationStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Migration Results ===")?;
        writeln!(f, "Files processed:     {}", self.files_processed)?;
        writeln!(f, "Sessions migrated:   {}", self.sessions_migrated)?;
        writeln!(f, "Instances migrated:  {}", self.instances_migrated)?;
        writeln!(f, "Messages migrated:   {}", self.messages_migrated)?;
        writeln!(f, "Errors:              {}", self.errors.len())?;
        Ok(())
    }
}

/// Migrate all JSON files to SQLite
pub async fn migrate_json_to_sqlite(pool: &SqlitePool, dry_run: bool) -> Result<MigrationStats, Box<dyn std::error::Error>> {
    let sessions_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".ait42")
        .join("sessions");

    if !sessions_dir.exists() {
        tracing::warn!("Sessions directory does not exist: {:?}", sessions_dir);
        return Ok(MigrationStats::default());
    }

    let mut stats = MigrationStats::default();

    // Create workspace mapping file
    let mapping_path = sessions_dir.parent().unwrap().join("workspace_mapping.json");
    let mut workspace_mapping = load_workspace_mapping(&mapping_path)?;

    // Find all JSON files
    let pattern = format!("{}/*.json", sessions_dir.display());
    tracing::info!("Searching for JSON files: {}", pattern);

    let json_files: Vec<_> = glob(&pattern)?
        .filter_map(Result::ok)
        .collect();

    println!("\nFound {} JSON files to process", json_files.len());

    if dry_run {
        println!("DRY RUN MODE - No changes will be made to the database\n");
    }

    for (idx, path) in json_files.iter().enumerate() {
        println!("[{}/{}] Processing: {:?}", idx + 1, json_files.len(), path.file_name().unwrap());

        // Read JSON file
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                let error_msg = format!("Failed to read {:?}: {}", path, e);
                eprintln!("  ERROR: {}", error_msg);
                stats.errors.push(error_msg);
                continue;
            }
        };

        if content.trim().is_empty() {
            println!("  SKIP: Empty file");
            continue;
        }

        // Parse JSON
        let sessions: Vec<WorktreeSession> = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(e) => {
                let error_msg = format!("Failed to parse {:?}: {}", path, e);
                eprintln!("  ERROR: {}", error_msg);
                stats.errors.push(error_msg);
                continue;
            }
        };

        // Extract workspace_path from filename hash
        let filename = path.file_stem().unwrap().to_str().unwrap();
        let workspace_path = get_or_prompt_workspace_path(
            filename,
            &mut workspace_mapping,
            &mapping_path,
        )?;

        println!("  Workspace: {}", workspace_path);
        println!("  Sessions: {}", sessions.len());

        // Migrate each session
        for session in sessions {
            if !dry_run {
                match migrate_session(pool, &session, &workspace_path).await {
                    Ok(session_stats) => {
                        stats.sessions_migrated += 1;
                        stats.instances_migrated += session_stats.instances_migrated;
                        stats.messages_migrated += session_stats.messages_migrated;
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to migrate session {}: {}", session.id, e);
                        eprintln!("  ERROR: {}", error_msg);
                        stats.errors.push(error_msg);
                    }
                }
            } else {
                // Dry run - just count
                stats.sessions_migrated += 1;
                stats.instances_migrated += session.instances.len();
                stats.messages_migrated += session.chat_history.len();
            }
        }

        stats.files_processed += 1;
    }

    println!();
    tracing::info!("Migration complete: {:?}", stats);
    Ok(stats)
}

/// Migrate a single session
async fn migrate_session(
    pool: &SqlitePool,
    session: &WorktreeSession,
    workspace_path: &str,
) -> Result<SessionMigrationStats, Box<dyn std::error::Error>> {
    let mut stats = SessionMigrationStats::default();

    // Begin transaction
    let mut tx = pool.begin().await?;

    // Insert session
    let session_row = SessionRow {
        id: session.id.clone(),
        workspace_path: workspace_path.to_string(),
        r#type: session.r#type.clone(),
        task: session.task.clone(),
        status: session.status.clone(),
        created_at: session.created_at.clone(),
        updated_at: session.updated_at.clone(),
        completed_at: session.completed_at.clone(),
        model: session.model.clone(),
        timeout_seconds: session.timeout_seconds.map(|v| v as i32),
        preserve_worktrees: session.preserve_worktrees.unwrap_or(false) as i32,
        winner_id: session.winner_id.map(|v| v as i32),
        runtime_mix: session.runtime_mix.as_ref().and_then(|v| serde_json::to_string(v).ok()),
        total_duration: session.total_duration.map(|v| v as i64),
        total_files_changed: session.total_files_changed.map(|v| v as i32),
        total_lines_added: session.total_lines_added.map(|v| v as i32),
        total_lines_deleted: session.total_lines_deleted.map(|v| v as i32),
        instance_count: Some(session.instances.len() as i32),
        message_count: Some(session.chat_history.len() as i32),
    };

    insert_session(&mut tx, &session_row).await?;

    // Insert instances
    for instance in &session.instances {
        let instance_row = InstanceRow {
            id: 0,  // Auto-increment
            session_id: session.id.clone(),
            instance_id: instance.instance_id as i32,
            worktree_path: instance.worktree_path.clone(),
            branch: instance.branch.clone(),
            agent_name: instance.agent_name.clone(),
            status: instance.status.clone(),
            tmux_session_id: instance.tmux_session_id.clone(),
            output: instance.output.clone(),
            start_time: instance.start_time.clone(),
            end_time: instance.end_time.clone(),
            files_changed: instance.files_changed.map(|v| v as i32),
            lines_added: instance.lines_added.map(|v| v as i32),
            lines_deleted: instance.lines_deleted.map(|v| v as i32),
            runtime: instance.runtime.clone(),
            model: instance.model.clone(),
            runtime_label: instance.runtime_label.clone(),
        };

        insert_instance(&mut tx, &instance_row).await?;
        stats.instances_migrated += 1;
    }

    // Insert chat messages
    for message in &session.chat_history {
        // Resolve instance_id_ref
        let instance_id_ref = if let Some(inst_id) = message.instance_id {
            sqlx::query_scalar!(
                "SELECT id FROM instances WHERE session_id = ? AND instance_id = ?",
                session.id,
                inst_id as i32
            )
            .fetch_optional(&mut *tx)
            .await?
        } else {
            None
        };

        let message_row = ChatMessageRow {
            id: message.id.clone(),
            session_id: session.id.clone(),
            instance_id_ref,
            role: message.role.clone(),
            content: message.content.clone(),
            timestamp: message.timestamp.clone(),
        };

        insert_chat_message(&mut tx, &message_row).await?;
        stats.messages_migrated += 1;
    }

    // Commit transaction
    tx.commit().await?;

    Ok(stats)
}

#[derive(Debug, Default)]
struct SessionMigrationStats {
    instances_migrated: usize,
    messages_migrated: usize,
}

/// Load workspace mapping from file
fn load_workspace_mapping(path: &PathBuf) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    if path.exists() {
        let content = fs::read_to_string(path)?;
        let mapping = serde_json::from_str(&content)?;
        Ok(mapping)
    } else {
        Ok(HashMap::new())
    }
}

/// Save workspace mapping to file
fn save_workspace_mapping(
    path: &PathBuf,
    mapping: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = serde_json::to_string_pretty(mapping)?;
    fs::write(path, content)?;
    Ok(())
}

/// Get workspace path from hash or prompt user
fn get_or_prompt_workspace_path(
    hash: &str,
    mapping: &mut HashMap<String, String>,
    mapping_path: &PathBuf,
) -> Result<String, Box<dyn std::error::Error>> {
    // Check if already mapped
    if let Some(path) = mapping.get(hash) {
        return Ok(path.clone());
    }

    // Prompt user (CLI mode)
    println!("\n=================================================");
    println!("Unknown workspace hash: {}", hash);
    println!("Please enter the workspace path:");
    println!("=================================================");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let workspace_path = input.trim().to_string();

    // Verify hash matches
    let computed_hash = workspace_hash(&workspace_path);
    if computed_hash != hash {
        println!("WARNING: Hash mismatch!");
        println!("  Expected: {}", hash);
        println!("  Computed: {}", computed_hash);
        println!("Proceeding anyway...");
    }

    // Save mapping
    mapping.insert(hash.to_string(), workspace_path.clone());
    save_workspace_mapping(mapping_path, mapping)?;

    Ok(workspace_path)
}

/// Generate workspace hash (same algorithm as session_history.rs)
fn workspace_hash(workspace_path: &str) -> String {
    use std::path::Path;

    let normalized_path = match fs::canonicalize(Path::new(workspace_path)) {
        Ok(canonical) => canonical.to_string_lossy().to_string(),
        Err(_) => workspace_path.trim_end_matches('/').to_string(),
    };

    let mut hasher = Sha256::new();
    hasher.update(normalized_path.as_bytes());
    format!("{:x}", hasher.finalize())[..16].to_string()
}

/// Validate migration integrity
pub async fn validate_migration(pool: &SqlitePool) -> Result<ValidationReport, Box<dyn std::error::Error>> {
    let mut report = ValidationReport::default();

    // Count rows
    report.session_count = sqlx::query_scalar!("SELECT COUNT(*) FROM sessions")
        .fetch_one(pool)
        .await? as usize;

    report.instance_count = sqlx::query_scalar!("SELECT COUNT(*) FROM instances")
        .fetch_one(pool)
        .await? as usize;

    report.message_count = sqlx::query_scalar!("SELECT COUNT(*) FROM chat_messages")
        .fetch_one(pool)
        .await? as usize;

    // Check referential integrity
    report.orphaned_instances = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM instances WHERE session_id NOT IN (SELECT id FROM sessions)"
    )
    .fetch_one(pool)
    .await? as usize;

    report.orphaned_messages = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM chat_messages WHERE session_id NOT IN (SELECT id FROM sessions)"
    )
    .fetch_one(pool)
    .await? as usize;

    // Check data integrity
    report.invalid_statuses = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM sessions WHERE status NOT IN ('running', 'completed', 'failed', 'paused')"
    )
    .fetch_one(pool)
    .await? as usize;

    // Database size
    let db_path = super::get_database_path();
    report.db_size_bytes = fs::metadata(&db_path)?.len();

    // Integrity check
    let integrity: (String,) = sqlx::query_as("PRAGMA integrity_check")
        .fetch_one(pool)
        .await?;
    report.integrity_ok = integrity.0 == "ok";

    // Overall validation
    report.is_valid = report.orphaned_instances == 0
        && report.orphaned_messages == 0
        && report.invalid_statuses == 0
        && report.integrity_ok;

    Ok(report)
}

#[derive(Debug, Default)]
pub struct ValidationReport {
    pub session_count: usize,
    pub instance_count: usize,
    pub message_count: usize,
    pub orphaned_instances: usize,
    pub orphaned_messages: usize,
    pub invalid_statuses: usize,
    pub db_size_bytes: u64,
    pub integrity_ok: bool,
    pub is_valid: bool,
}

impl std::fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Migration Validation Report ===")?;
        writeln!(f, "Sessions:           {}", self.session_count)?;
        writeln!(f, "Instances:          {}", self.instance_count)?;
        writeln!(f, "Messages:           {}", self.message_count)?;
        writeln!(f, "Orphaned instances: {}", self.orphaned_instances)?;
        writeln!(f, "Orphaned messages:  {}", self.orphaned_messages)?;
        writeln!(f, "Invalid statuses:   {}", self.invalid_statuses)?;
        writeln!(f, "Database size:      {} MB", self.db_size_bytes / 1024 / 1024)?;
        writeln!(f, "Integrity check:    {}", if self.integrity_ok { "OK" } else { "FAILED" })?;
        writeln!(f, "Overall:            {}", if self.is_valid { "VALID ✓" } else { "INVALID ✗" })?;
        Ok(())
    }
}
