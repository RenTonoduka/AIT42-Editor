use sqlx::SqlitePool;
use std::collections::HashMap;
use tracing::debug;

use crate::error::{Result, SessionError};
use crate::models::{
    compute_workspace_hash, ChatMessage, ChatMessageRow, InstanceRow, SessionRow,
    WorktreeInstance, WorktreeSession,
};

// ===================================
// Workspace Queries
// ===================================

/// Ensure workspace exists in database (upsert)
pub async fn upsert_workspace(pool: &SqlitePool, workspace_path: &str) -> Result<String> {
    let workspace_hash = compute_workspace_hash(workspace_path);

    sqlx::query(
        r#"
        INSERT INTO workspaces (hash, path, last_accessed)
        VALUES (?1, ?2, datetime('now'))
        ON CONFLICT(hash) DO UPDATE SET last_accessed = datetime('now')
        "#,
    )
    .bind(&workspace_hash)
    .bind(workspace_path)
    .execute(pool)
    .await?;

    debug!("Upserted workspace: hash={}, path={}", workspace_hash, workspace_path);

    Ok(workspace_hash)
}

// ===================================
// Session Queries
// ===================================

/// Create a new session
pub async fn create_session(pool: &SqlitePool, session: WorktreeSession) -> Result<WorktreeSession> {
    let mut tx = pool.begin().await?;

    // Ensure workspace_hash is set
    let workspace_hash = session.workspace_hash.as_ref()
        .ok_or_else(|| SessionError::Validation("workspace_hash is required".to_string()))?
        .clone();

    // Ensure workspace exists
    sqlx::query(
        r#"
        INSERT INTO workspaces (hash, path, last_accessed)
        VALUES (?1, ?2, datetime('now'))
        ON CONFLICT(hash) DO UPDATE SET last_accessed = datetime('now')
        "#,
    )
    .bind(&workspace_hash)
    .bind(&workspace_hash) // Use hash as path for now
    .execute(&mut *tx)
    .await?;

    // Serialize runtime_mix to JSON
    let runtime_mix_json = session.runtime_mix.as_ref()
        .map(|v| serde_json::to_string(v))
        .transpose()?;

    // Insert session
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at, completed_at, model,
            timeout_seconds, preserve_worktrees, winner_id,
            runtime_mix, total_duration, total_files_changed,
            total_lines_added, total_lines_deleted
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
        "#,
    )
    .bind(&session.id)
    .bind(&workspace_hash)
    .bind(&session.r#type)
    .bind(&session.task)
    .bind(&session.status)
    .bind(&session.created_at)
    .bind(&session.updated_at)
    .bind(&session.completed_at)
    .bind(&session.model)
    .bind(session.timeout_seconds.map(|v| v as i64))
    .bind(session.preserve_worktrees.map(|v| if v { 1i64 } else { 0i64 }))
    .bind(session.winner_id.map(|v| v as i64))
    .bind(runtime_mix_json)
    .bind(session.total_duration.map(|v| v as i64))
    .bind(session.total_files_changed.map(|v| v as i64))
    .bind(session.total_lines_added.map(|v| v as i64))
    .bind(session.total_lines_deleted.map(|v| v as i64))
    .execute(&mut *tx)
    .await?;

    // Insert instances
    for instance in &session.instances {
        insert_instance(&mut tx, &session.id, instance).await?;
    }

    // Insert chat messages
    for message in &session.chat_history {
        insert_chat_message(&mut tx, &session.id, message).await?;
    }

    tx.commit().await?;

    debug!("Created session: id={}, type={}", session.id, session.r#type);

    Ok(session)
}

/// Update existing session
pub async fn update_session(pool: &SqlitePool, session: &WorktreeSession) -> Result<()> {
    let mut tx = pool.begin().await?;

    let runtime_mix_json = session.runtime_mix.as_ref()
        .map(|v| serde_json::to_string(v))
        .transpose()?;

    // Update session
    let rows_affected = sqlx::query(
        r#"
        UPDATE sessions SET
            session_type = ?2, task = ?3, status = ?4,
            updated_at = ?5, completed_at = ?6, model = ?7,
            timeout_seconds = ?8, preserve_worktrees = ?9,
            winner_id = ?10, runtime_mix = ?11,
            total_duration = ?12, total_files_changed = ?13,
            total_lines_added = ?14, total_lines_deleted = ?15
        WHERE id = ?1
        "#,
    )
    .bind(&session.id)
    .bind(&session.r#type)
    .bind(&session.task)
    .bind(&session.status)
    .bind(&session.updated_at)
    .bind(&session.completed_at)
    .bind(&session.model)
    .bind(session.timeout_seconds.map(|v| v as i64))
    .bind(session.preserve_worktrees.map(|v| if v { 1i64 } else { 0i64 }))
    .bind(session.winner_id.map(|v| v as i64))
    .bind(runtime_mix_json)
    .bind(session.total_duration.map(|v| v as i64))
    .bind(session.total_files_changed.map(|v| v as i64))
    .bind(session.total_lines_added.map(|v| v as i64))
    .bind(session.total_lines_deleted.map(|v| v as i64))
    .execute(&mut *tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(SessionError::NotFound(session.id.clone()));
    }

    // Delete and re-insert instances
    delete_instances(&mut tx, &session.id).await?;
    for instance in &session.instances {
        insert_instance(&mut tx, &session.id, instance).await?;
    }

    // Delete and re-insert chat messages
    delete_chat_messages(&mut tx, &session.id).await?;
    for message in &session.chat_history {
        insert_chat_message(&mut tx, &session.id, message).await?;
    }

    tx.commit().await?;

    debug!("Updated session: id={}", session.id);

    Ok(())
}

/// Get session by ID
pub async fn get_session(
    pool: &SqlitePool,
    workspace_hash: &str,
    session_id: &str,
) -> Result<WorktreeSession> {
    // Fetch session
    let row = sqlx::query_as::<_, SessionRow>(
        r#"
        SELECT
            id, workspace_hash, session_type, task, status,
            created_at, updated_at, completed_at, model,
            timeout_seconds, preserve_worktrees, winner_id,
            runtime_mix, total_duration, total_files_changed,
            total_lines_added, total_lines_deleted
        FROM sessions
        WHERE workspace_hash = ?1 AND id = ?2
        "#,
    )
    .bind(workspace_hash)
    .bind(session_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| SessionError::NotFound(session_id.to_string()))?;

    let mut session: WorktreeSession = row.into();

    // Load instances
    session.instances = load_instances(pool, session_id).await?;

    // Load chat messages
    session.chat_history = load_chat_messages(pool, session_id).await?;

    Ok(session)
}

/// Get all sessions for a workspace
/// ISSUE #4 FIX: Use batch loading with HashMap to avoid N+1 queries
pub async fn get_all_sessions(pool: &SqlitePool, workspace_hash: &str) -> Result<Vec<WorktreeSession>> {
    // Step 1: Load all sessions
    let rows = sqlx::query_as::<_, SessionRow>(
        r#"
        SELECT
            id, workspace_hash, session_type, task, status,
            created_at, updated_at, completed_at, model,
            timeout_seconds, preserve_worktrees, winner_id,
            runtime_mix, total_duration, total_files_changed,
            total_lines_added, total_lines_deleted
        FROM sessions
        WHERE workspace_hash = ?1
        ORDER BY updated_at DESC
        "#,
    )
    .bind(workspace_hash)
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    // Convert rows to sessions
    let mut sessions: Vec<WorktreeSession> = rows.into_iter()
        .map(|row| row.into())
        .collect();

    // Collect all session IDs
    let session_ids: Vec<&str> = sessions.iter()
        .map(|s| s.id.as_str())
        .collect();

    // Step 2: Batch load all instances for these sessions (single query)
    let all_instances = batch_load_instances(pool, &session_ids).await?;

    // Step 3: Batch load all chat messages for these sessions (single query)
    let all_messages = batch_load_chat_messages(pool, &session_ids).await?;

    // Step 4: Assign instances and messages to sessions
    for session in &mut sessions {
        session.instances = all_instances
            .get(session.id.as_str())
            .cloned()
            .unwrap_or_default();

        session.chat_history = all_messages
            .get(session.id.as_str())
            .cloned()
            .unwrap_or_default();
    }

    debug!("Fetched {} sessions for workspace {} (optimized batch loading)",
           sessions.len(), workspace_hash);

    Ok(sessions)
}

/// Delete session
pub async fn delete_session(pool: &SqlitePool, workspace_hash: &str, session_id: &str) -> Result<()> {
    let rows_affected = sqlx::query(
        "DELETE FROM sessions WHERE workspace_hash = ?1 AND id = ?2"
    )
    .bind(workspace_hash)
    .bind(session_id)
    .execute(pool)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(SessionError::NotFound(session_id.to_string()));
    }

    debug!("Deleted session: id={}", session_id);

    Ok(())
}

// ===================================
// Instance Queries
// ===================================

async fn insert_instance(
    tx: &mut sqlx::SqliteConnection,
    session_id: &str,
    instance: &WorktreeInstance,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO instances (
            session_id, instance_id, worktree_path, branch,
            agent_name, status, tmux_session_id, output,
            start_time, end_time, files_changed, lines_added,
            lines_deleted, runtime, model, runtime_label
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
        "#,
    )
    .bind(session_id)
    .bind(instance.instance_id as i64)
    .bind(&instance.worktree_path)
    .bind(&instance.branch)
    .bind(&instance.agent_name)
    .bind(&instance.status)
    .bind(&instance.tmux_session_id)
    .bind(&instance.output)
    .bind(&instance.start_time)
    .bind(&instance.end_time)
    .bind(instance.files_changed.map(|v| v as i64))
    .bind(instance.lines_added.map(|v| v as i64))
    .bind(instance.lines_deleted.map(|v| v as i64))
    .bind(&instance.runtime)
    .bind(&instance.model)
    .bind(&instance.runtime_label)
    .execute(tx)
    .await?;

    Ok(())
}

async fn delete_instances(tx: &mut sqlx::SqliteConnection, session_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM instances WHERE session_id = ?")
        .bind(session_id)
        .execute(tx)
        .await?;

    Ok(())
}

async fn load_instances(pool: &SqlitePool, session_id: &str) -> Result<Vec<WorktreeInstance>> {
    let rows = sqlx::query_as::<_, InstanceRow>(
        r#"
        SELECT
            id, session_id, instance_id, worktree_path, branch,
            agent_name, status, tmux_session_id, output,
            start_time, end_time, files_changed, lines_added,
            lines_deleted, runtime, model, runtime_label
        FROM instances
        WHERE session_id = ?
        ORDER BY instance_id ASC
        "#,
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.into()).collect())
}

/// Batch load instances for multiple sessions (Issue #4 fix)
/// Returns HashMap of session_id -> Vec<WorktreeInstance>
async fn batch_load_instances(
    pool: &SqlitePool,
    session_ids: &[&str],
) -> Result<HashMap<String, Vec<WorktreeInstance>>> {
    if session_ids.is_empty() {
        return Ok(HashMap::new());
    }

    // Build IN clause with placeholders
    let placeholders = session_ids.iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect::<Vec<_>>()
        .join(", ");

    let query_str = format!(
        r#"
        SELECT
            id, session_id, instance_id, worktree_path, branch,
            agent_name, status, tmux_session_id, output,
            start_time, end_time, files_changed, lines_added,
            lines_deleted, runtime, model, runtime_label
        FROM instances
        WHERE session_id IN ({})
        ORDER BY session_id, instance_id ASC
        "#,
        placeholders
    );

    let mut query = sqlx::query_as::<_, InstanceRow>(&query_str);
    for session_id in session_ids {
        query = query.bind(*session_id);
    }

    let rows = query.fetch_all(pool).await?;

    // Group instances by session_id
    let mut grouped: HashMap<String, Vec<WorktreeInstance>> = HashMap::new();
    for row in rows {
        let session_id = row.session_id.clone();
        let instance: WorktreeInstance = row.into();
        grouped.entry(session_id).or_insert_with(Vec::new).push(instance);
    }

    Ok(grouped)
}

/// Update instance status
pub async fn update_instance_status(
    pool: &SqlitePool,
    session_id: &str,
    instance_id: u32,
    new_status: &str,
) -> Result<()> {
    let mut tx = pool.begin().await?;

    let rows_affected = sqlx::query(
        "UPDATE instances SET status = ?1 WHERE session_id = ?2 AND instance_id = ?3"
    )
    .bind(new_status)
    .bind(session_id)
    .bind(instance_id as i64)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(SessionError::NotFound(format!(
            "Instance {} in session {}",
            instance_id, session_id
        )));
    }

    // Update session timestamp
    sqlx::query("UPDATE sessions SET updated_at = datetime('now') WHERE id = ?")
        .bind(session_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    debug!("Updated instance status: session={}, instance={}, status={}",
           session_id, instance_id, new_status);

    Ok(())
}

// ===================================
// Chat Message Queries
// ===================================

async fn insert_chat_message(
    tx: &mut sqlx::SqliteConnection,
    session_id: &str,
    message: &ChatMessage,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO chat_messages (id, session_id, role, content, timestamp, instance_id)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
    )
    .bind(&message.id)
    .bind(session_id)
    .bind(&message.role)
    .bind(&message.content)
    .bind(&message.timestamp)
    .bind(message.instance_id.map(|v| v as i64))
    .execute(tx)
    .await?;

    Ok(())
}

async fn delete_chat_messages(tx: &mut sqlx::SqliteConnection, session_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM chat_messages WHERE session_id = ?")
        .bind(session_id)
        .execute(tx)
        .await?;

    Ok(())
}

async fn load_chat_messages(pool: &SqlitePool, session_id: &str) -> Result<Vec<ChatMessage>> {
    let rows = sqlx::query_as::<_, ChatMessageRow>(
        r#"
        SELECT id, session_id, role, content, timestamp, instance_id
        FROM chat_messages
        WHERE session_id = ?
        ORDER BY timestamp ASC
        "#,
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.into()).collect())
}

/// Batch load chat messages for multiple sessions (Issue #4 fix)
/// Returns HashMap of session_id -> Vec<ChatMessage>
async fn batch_load_chat_messages(
    pool: &SqlitePool,
    session_ids: &[&str],
) -> Result<HashMap<String, Vec<ChatMessage>>> {
    if session_ids.is_empty() {
        return Ok(HashMap::new());
    }

    // Build IN clause with placeholders
    let placeholders = session_ids.iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect::<Vec<_>>()
        .join(", ");

    let query_str = format!(
        r#"
        SELECT id, session_id, role, content, timestamp, instance_id
        FROM chat_messages
        WHERE session_id IN ({})
        ORDER BY session_id, timestamp ASC
        "#,
        placeholders
    );

    let mut query = sqlx::query_as::<_, ChatMessageRow>(&query_str);
    for session_id in session_ids {
        query = query.bind(*session_id);
    }

    let rows = query.fetch_all(pool).await?;

    // Group messages by session_id
    let mut grouped: HashMap<String, Vec<ChatMessage>> = HashMap::new();
    for row in rows {
        let session_id = row.session_id.clone();
        let message: ChatMessage = row.into();
        grouped.entry(session_id).or_insert_with(Vec::new).push(message);
    }

    Ok(grouped)
}

/// Add chat message to session
pub async fn add_chat_message(
    pool: &SqlitePool,
    session_id: &str,
    message: &ChatMessage,
) -> Result<()> {
    let mut tx = pool.begin().await?;

    insert_chat_message(&mut tx, session_id, message).await?;

    // Update session timestamp
    sqlx::query("UPDATE sessions SET updated_at = datetime('now') WHERE id = ?")
        .bind(session_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    debug!("Added chat message: session={}, message={}", session_id, message.id);

    Ok(())
}
