/// CRUD (Create, Read, Update, Delete) operation tests
///
/// Tests in this file verify:
/// - Session creation
/// - Session retrieval (by ID, all sessions)
/// - Session updates
/// - Session deletion
/// - Error handling for non-existent sessions
/// - Edge cases (null values, empty strings, etc.)

mod common;

use chrono::Utc;
use common::{create_test_pool, factories, setup_test_db, count_sessions, count_instances, count_messages};

#[tokio::test]
async fn test_create_session_minimal_fields() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    let session = factories::WorktreeSession::default();

    // ============ ACT ============
    let result = sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&session.id)
    .bind(&session.workspace_hash)
    .bind(&session.session_type)
    .bind(&session.task)
    .bind(&session.status)
    .bind(&session.created_at)
    .bind(&session.updated_at)
    .execute(&pool)
    .await;

    // ============ ASSERT ============
    assert!(result.is_ok(), "Session creation should succeed");
    let count = count_sessions(&pool).await;
    assert_eq!(count, 1, "Should have exactly 1 session");
}

#[tokio::test]
async fn test_create_session_all_fields() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    let session = factories::WorktreeSession::completed();
    let runtime_mix = serde_json::json!(["claude", "gpt4"]).to_string();

    // ============ ACT ============
    let result = sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at, completed_at, model,
            timeout_seconds, preserve_worktrees, winner_id,
            runtime_mix, total_duration, total_files_changed,
            total_lines_added, total_lines_deleted,
            integration_phase, integration_instance_id
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&session.id)
    .bind(&session.workspace_hash)
    .bind(&session.session_type)
    .bind(&session.task)
    .bind(&session.status)
    .bind(&session.created_at)
    .bind(&session.updated_at)
    .bind(&session.completed_at)
    .bind("claude-3-5-sonnet-20241022")
    .bind(3600)
    .bind(true)
    .bind(0)
    .bind(&runtime_mix)
    .bind(&session.total_duration)
    .bind(&session.total_files_changed)
    .bind(&session.total_lines_added)
    .bind(&session.total_lines_deleted)
    .bind("completed")
    .bind(0)
    .execute(&pool)
    .await;

    // ============ ASSERT ============
    assert!(result.is_ok(), "Session creation with all fields should succeed");
    let count = count_sessions(&pool).await;
    assert_eq!(count, 1, "Should have exactly 1 session");
}

#[tokio::test]
async fn test_get_session_by_id() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    let session = factories::WorktreeSession::with_id("test_session_123");

    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&session.id)
    .bind(&session.workspace_hash)
    .bind(&session.session_type)
    .bind(&session.task)
    .bind(&session.status)
    .bind(&session.created_at)
    .bind(&session.updated_at)
    .execute(&pool)
    .await
    .expect("Failed to insert test session");

    // ============ ACT ============
    let retrieved: Option<(String, String, String)> = sqlx::query_as(
        "SELECT id, task, status FROM sessions WHERE id = ?"
    )
    .bind("test_session_123")
    .fetch_optional(&pool)
    .await
    .expect("Failed to fetch session");

    // ============ ASSERT ============
    assert!(retrieved.is_some(), "Session should be found");
    let (id, task, status) = retrieved.unwrap();
    assert_eq!(id, "test_session_123", "Session ID should match");
    assert_eq!(task, "Test task", "Task should match");
    assert_eq!(status, "running", "Status should match");
}

#[tokio::test]
async fn test_get_session_not_found() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;

    // ============ ACT ============
    let retrieved: Option<(String,)> = sqlx::query_as(
        "SELECT id FROM sessions WHERE id = ?"
    )
    .bind("non_existent_session")
    .fetch_optional(&pool)
    .await
    .expect("Query should succeed");

    // ============ ASSERT ============
    assert!(retrieved.is_none(), "Non-existent session should return None");
}

#[tokio::test]
async fn test_get_all_sessions_empty() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // ============ ACT ============
    let sessions: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM sessions WHERE workspace_hash = ?"
    )
    .bind("workspace_1")
    .fetch_all(&pool)
    .await
    .expect("Query should succeed");

    // ============ ASSERT ============
    assert_eq!(sessions.len(), 0, "Should return empty list when no sessions");
}

#[tokio::test]
async fn test_get_all_sessions_multiple() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // Insert 5 sessions
    for i in 0..5 {
        let session = factories::WorktreeSession::with_id(&format!("session_{}", i));
        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, workspace_hash, session_type, task, status,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&session.id)
        .bind("workspace_1")
        .bind(&session.session_type)
        .bind(format!("Task {}", i))
        .bind(&session.status)
        .bind(&session.created_at)
        .bind(&session.updated_at)
        .execute(&pool)
        .await
        .expect("Failed to insert session");
    }

    // ============ ACT ============
    let sessions: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM sessions WHERE workspace_hash = ? ORDER BY created_at ASC"
    )
    .bind("workspace_1")
    .fetch_all(&pool)
    .await
    .expect("Query should succeed");

    // ============ ASSERT ============
    assert_eq!(sessions.len(), 5, "Should return all 5 sessions");
}

#[tokio::test]
async fn test_update_session_status() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    let session = factories::WorktreeSession::with_id("session_update");
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&session.id)
    .bind(&session.workspace_hash)
    .bind(&session.session_type)
    .bind(&session.task)
    .bind(&session.status)
    .bind(&session.created_at)
    .bind(&session.updated_at)
    .execute(&pool)
    .await
    .expect("Failed to insert session");

    // ============ ACT ============
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE sessions SET status = ?, updated_at = ?, completed_at = ? WHERE id = ?"
    )
    .bind("completed")
    .bind(&now)
    .bind(&now)
    .bind("session_update")
    .execute(&pool)
    .await
    .expect("Failed to update session");

    // ============ ASSERT ============
    let updated: (String, Option<String>) = sqlx::query_as(
        "SELECT status, completed_at FROM sessions WHERE id = ?"
    )
    .bind("session_update")
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch updated session");

    assert_eq!(updated.0, "completed", "Status should be updated");
    assert!(updated.1.is_some(), "completed_at should be set");
}

#[tokio::test]
async fn test_update_session_task() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    let session = factories::WorktreeSession::default();
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&session.id)
    .bind(&session.workspace_hash)
    .bind(&session.session_type)
    .bind("Original task")
    .bind(&session.status)
    .bind(&session.created_at)
    .bind(&session.updated_at)
    .execute(&pool)
    .await
    .expect("Failed to insert session");

    // ============ ACT ============
    sqlx::query("UPDATE sessions SET task = ? WHERE id = ?")
        .bind("Updated task description")
        .bind(&session.id)
        .execute(&pool)
        .await
        .expect("Failed to update task");

    // ============ ASSERT ============
    let task: String = sqlx::query_scalar("SELECT task FROM sessions WHERE id = ?")
        .bind(&session.id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch task");

    assert_eq!(task, "Updated task description", "Task should be updated");
}

#[tokio::test]
async fn test_update_session_statistics() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    let session = factories::WorktreeSession::default();
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&session.id)
    .bind(&session.workspace_hash)
    .bind(&session.session_type)
    .bind(&session.task)
    .bind(&session.status)
    .bind(&session.created_at)
    .bind(&session.updated_at)
    .execute(&pool)
    .await
    .expect("Failed to insert session");

    // ============ ACT ============
    sqlx::query(
        r#"
        UPDATE sessions SET
            total_duration = ?,
            total_files_changed = ?,
            total_lines_added = ?,
            total_lines_deleted = ?
        WHERE id = ?
        "#
    )
    .bind(3600)
    .bind(15)
    .bind(250)
    .bind(80)
    .bind(&session.id)
    .execute(&pool)
    .await
    .expect("Failed to update statistics");

    // ============ ASSERT ============
    let stats: (i64, i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT total_duration, total_files_changed, total_lines_added, total_lines_deleted
        FROM sessions WHERE id = ?
        "#
    )
    .bind(&session.id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch statistics");

    assert_eq!(stats.0, 3600, "Duration should be updated");
    assert_eq!(stats.1, 15, "Files changed should be updated");
    assert_eq!(stats.2, 250, "Lines added should be updated");
    assert_eq!(stats.3, 80, "Lines deleted should be updated");
}

#[tokio::test]
async fn test_delete_session() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    let session = factories::WorktreeSession::with_id("session_to_delete");
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&session.id)
    .bind(&session.workspace_hash)
    .bind(&session.session_type)
    .bind(&session.task)
    .bind(&session.status)
    .bind(&session.created_at)
    .bind(&session.updated_at)
    .execute(&pool)
    .await
    .expect("Failed to insert session");

    // Verify session exists
    let count_before = count_sessions(&pool).await;
    assert_eq!(count_before, 1, "Session should exist before deletion");

    // ============ ACT ============
    let result = sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind("session_to_delete")
        .execute(&pool)
        .await
        .expect("Failed to delete session");

    // ============ ASSERT ============
    assert_eq!(result.rows_affected(), 1, "Should delete exactly 1 row");

    let count_after = count_sessions(&pool).await;
    assert_eq!(count_after, 0, "Session should not exist after deletion");
}

#[tokio::test]
async fn test_delete_session_not_found() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;

    // ============ ACT ============
    let result = sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind("non_existent_session")
        .execute(&pool)
        .await
        .expect("Query should succeed");

    // ============ ASSERT ============
    assert_eq!(
        result.rows_affected(),
        0,
        "Deleting non-existent session should affect 0 rows"
    );
}

#[tokio::test]
async fn test_delete_session_cascades_to_instances() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // Create session
    let session = factories::WorktreeSession::with_id("session_cascade");
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&session.id)
    .bind(&session.workspace_hash)
    .bind(&session.session_type)
    .bind(&session.task)
    .bind(&session.status)
    .bind(&session.created_at)
    .bind(&session.updated_at)
    .execute(&pool)
    .await
    .expect("Failed to insert session");

    // Create 3 instances
    for i in 0..3 {
        let instance = factories::WorktreeInstance::default("session_cascade", i);
        sqlx::query(
            r#"
            INSERT INTO instances (
                session_id, instance_id, worktree_path, branch,
                agent_name, status, tmux_session_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&instance.session_id)
        .bind(instance.instance_id)
        .bind(&instance.worktree_path)
        .bind(&instance.branch)
        .bind(&instance.agent_name)
        .bind(&instance.status)
        .bind(&instance.tmux_session_id)
        .execute(&pool)
        .await
        .expect("Failed to insert instance");
    }

    let instances_before = count_instances(&pool, "session_cascade").await;
    assert_eq!(instances_before, 3, "Should have 3 instances before deletion");

    // ============ ACT ============
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind("session_cascade")
        .execute(&pool)
        .await
        .expect("Failed to delete session");

    // ============ ASSERT ============
    let instances_after = count_instances(&pool, "session_cascade").await;
    assert_eq!(
        instances_after, 0,
        "All instances should be deleted via CASCADE"
    );
}

#[tokio::test]
async fn test_delete_session_cascades_to_messages() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // Create session
    let session = factories::WorktreeSession::with_id("session_messages");
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&session.id)
    .bind(&session.workspace_hash)
    .bind(&session.session_type)
    .bind(&session.task)
    .bind(&session.status)
    .bind(&session.created_at)
    .bind(&session.updated_at)
    .execute(&pool)
    .await
    .expect("Failed to insert session");

    // Create 5 messages
    let messages = factories::ChatMessage::create_many("session_messages", 5);
    for msg in messages {
        sqlx::query(
            r#"
            INSERT INTO chat_messages (id, session_id, role, content, timestamp)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&msg.id)
        .bind(&msg.session_id)
        .bind(&msg.role)
        .bind(&msg.content)
        .bind(&msg.timestamp)
        .execute(&pool)
        .await
        .expect("Failed to insert message");
    }

    let messages_before = count_messages(&pool, "session_messages").await;
    assert_eq!(messages_before, 5, "Should have 5 messages before deletion");

    // ============ ACT ============
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind("session_messages")
        .execute(&pool)
        .await
        .expect("Failed to delete session");

    // ============ ASSERT ============
    let messages_after = count_messages(&pool, "session_messages").await;
    assert_eq!(
        messages_after, 0,
        "All messages should be deleted via CASCADE"
    );
}

#[tokio::test]
async fn test_create_session_with_null_optionals() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    let session = factories::WorktreeSession::default();

    // ============ ACT ============
    let result = sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at, completed_at, model,
            timeout_seconds, preserve_worktrees, winner_id,
            runtime_mix, total_duration, total_files_changed,
            total_lines_added, total_lines_deleted,
            integration_phase, integration_instance_id
        ) VALUES (?, ?, ?, ?, ?, ?, ?, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL)
        "#,
    )
    .bind(&session.id)
    .bind(&session.workspace_hash)
    .bind(&session.session_type)
    .bind(&session.task)
    .bind(&session.status)
    .bind(&session.created_at)
    .bind(&session.updated_at)
    .execute(&pool)
    .await;

    // ============ ASSERT ============
    assert!(result.is_ok(), "Session with NULL optionals should succeed");
}

#[tokio::test]
async fn test_update_nonexistent_session() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;

    // ============ ACT ============
    let result = sqlx::query("UPDATE sessions SET status = ? WHERE id = ?")
        .bind("completed")
        .bind("nonexistent_session")
        .execute(&pool)
        .await
        .expect("Query should succeed");

    // ============ ASSERT ============
    assert_eq!(
        result.rows_affected(),
        0,
        "Updating non-existent session should affect 0 rows"
    );
}
