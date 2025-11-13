/**
 * Database Queries - CRUD operations for sessions, instances, and messages
 *
 * Provides type-safe query functions using SQLx.
 */
use sqlx::SqlitePool;

/// Session row from database
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct SessionRow {
    pub id: String,
    pub workspace_path: String,
    pub r#type: String,
    pub task: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
    pub model: Option<String>,
    pub timeout_seconds: Option<i32>,
    pub preserve_worktrees: i32,
    pub winner_id: Option<i32>,
    pub runtime_mix: Option<String>,
    pub total_duration: Option<i64>,
    pub total_files_changed: Option<i32>,
    pub total_lines_added: Option<i32>,
    pub total_lines_deleted: Option<i32>,
    pub instance_count: Option<i32>,
    pub message_count: Option<i32>,
}

/// Instance row from database
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct InstanceRow {
    pub id: i64,
    pub session_id: String,
    pub instance_id: i32,
    pub worktree_path: String,
    pub branch: String,
    pub agent_name: String,
    pub status: String,
    pub tmux_session_id: String,
    pub output: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub files_changed: Option<i32>,
    pub lines_added: Option<i32>,
    pub lines_deleted: Option<i32>,
    pub runtime: Option<String>,
    pub model: Option<String>,
    pub runtime_label: Option<String>,
}

/// Chat message row from database
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)]
pub struct ChatMessageRow {
    pub id: String,
    pub session_id: String,
    pub instance_id_ref: Option<i64>,
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

/// Insert session into database (with UPSERT)
pub async fn insert_session(
    pool: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    session: &SessionRow,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO sessions (
            id, workspace_path, type, task, status,
            created_at, updated_at, completed_at,
            model, timeout_seconds, preserve_worktrees,
            winner_id, runtime_mix,
            total_duration, total_files_changed,
            total_lines_added, total_lines_deleted
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            updated_at = excluded.updated_at,
            status = excluded.status,
            completed_at = excluded.completed_at,
            total_duration = excluded.total_duration,
            total_files_changed = excluded.total_files_changed,
            total_lines_added = excluded.total_lines_added,
            total_lines_deleted = excluded.total_lines_deleted
        "#,
        session.id,
        session.workspace_path,
        session.r#type,
        session.task,
        session.status,
        session.created_at,
        session.updated_at,
        session.completed_at,
        session.model,
        session.timeout_seconds,
        session.preserve_worktrees,
        session.winner_id,
        session.runtime_mix,
        session.total_duration,
        session.total_files_changed,
        session.total_lines_added,
        session.total_lines_deleted
    )
    .execute(&mut **pool)
    .await?;

    Ok(())
}

/// Insert instance into database (with UPSERT)
pub async fn insert_instance(
    pool: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    instance: &InstanceRow,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO instances (
            session_id, instance_id, worktree_path, branch,
            agent_name, status, tmux_session_id, output,
            start_time, end_time, files_changed,
            lines_added, lines_deleted, runtime, model, runtime_label
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(session_id, instance_id) DO UPDATE SET
            status = excluded.status,
            output = excluded.output,
            end_time = excluded.end_time,
            files_changed = excluded.files_changed,
            lines_added = excluded.lines_added,
            lines_deleted = excluded.lines_deleted
        "#,
        instance.session_id,
        instance.instance_id,
        instance.worktree_path,
        instance.branch,
        instance.agent_name,
        instance.status,
        instance.tmux_session_id,
        instance.output,
        instance.start_time,
        instance.end_time,
        instance.files_changed,
        instance.lines_added,
        instance.lines_deleted,
        instance.runtime,
        instance.model,
        instance.runtime_label
    )
    .execute(&mut **pool)
    .await?;

    Ok(())
}

/// Insert chat message into database (with UPSERT)
pub async fn insert_chat_message(
    pool: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    message: &ChatMessageRow,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO chat_messages (
            id, session_id, instance_id_ref, role, content, timestamp
        ) VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO NOTHING
        "#,
        message.id,
        message.session_id,
        message.instance_id_ref,
        message.role,
        message.content,
        message.timestamp
    )
    .execute(&mut **pool)
    .await?;

    Ok(())
}

/// Get all sessions for a workspace
#[allow(dead_code)]
pub async fn get_sessions_by_workspace(
    pool: &SqlitePool,
    workspace_path: &str,
) -> Result<Vec<SessionRow>, sqlx::Error> {
    let sessions = sqlx::query_as!(
        SessionRow,
        r#"
        SELECT
            id, workspace_path, type, task, status,
            created_at, updated_at, completed_at,
            model, timeout_seconds, preserve_worktrees,
            winner_id, runtime_mix,
            total_duration, total_files_changed,
            total_lines_added, total_lines_deleted,
            instance_count, message_count
        FROM sessions
        WHERE workspace_path = ?
        ORDER BY created_at DESC
        "#,
        workspace_path
    )
    .fetch_all(pool)
    .await?;

    Ok(sessions)
}

/// Get instances for a session
#[allow(dead_code)]
pub async fn get_instances_by_session(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Vec<InstanceRow>, sqlx::Error> {
    let instances = sqlx::query_as!(
        InstanceRow,
        r#"
        SELECT
            id, session_id, instance_id, worktree_path, branch,
            agent_name, status, tmux_session_id, output,
            start_time, end_time, files_changed,
            lines_added, lines_deleted, runtime, model, runtime_label
        FROM instances
        WHERE session_id = ?
        ORDER BY instance_id
        "#,
        session_id
    )
    .fetch_all(pool)
    .await?;

    Ok(instances)
}

/// Get chat messages for a session
#[allow(dead_code)]
pub async fn get_chat_messages_by_session(
    pool: &SqlitePool,
    session_id: &str,
) -> Result<Vec<ChatMessageRow>, sqlx::Error> {
    let messages = sqlx::query_as!(
        ChatMessageRow,
        r#"
        SELECT id, session_id, instance_id_ref, role, content, timestamp
        FROM chat_messages
        WHERE session_id = ?
        ORDER BY timestamp
        "#,
        session_id
    )
    .fetch_all(pool)
    .await?;

    Ok(messages)
}
