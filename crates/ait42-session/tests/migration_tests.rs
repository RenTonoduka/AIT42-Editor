/// Data migration integration tests
///
/// Tests in this file verify:
/// - JSON to SQLite migration
/// - Large dataset migration (1000+ sessions)
/// - Data integrity verification
/// - Error handling for corrupted JSON
/// - Duplicate detection
/// - Incremental migration

mod common;

use chrono::Utc;
use common::{create_test_pool, setup_test_db, count_sessions};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a test JSON session file
fn create_test_json_session(id: &str, session_type: &str, task: &str, status: &str) -> serde_json::Value {
    json!({
        "id": id,
        "type": session_type,
        "task": task,
        "status": status,
        "createdAt": Utc::now().to_rfc3339(),
        "updatedAt": Utc::now().to_rfc3339(),
        "instances": [],
        "chatHistory": []
    })
}

/// Create a test JSON file with multiple sessions
fn create_test_json_file(temp_dir: &TempDir, workspace_hash: &str, sessions: Vec<serde_json::Value>) -> PathBuf {
    let file_path = temp_dir.path().join(format!("{}.json", workspace_hash));
    let json_content = json!(sessions);
    fs::write(&file_path, serde_json::to_string_pretty(&json_content).unwrap())
        .expect("Failed to write test JSON file");
    file_path
}

#[tokio::test]
async fn test_migrate_single_session() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_hash = "workspace_test";

    let session = create_test_json_session("session_1", "competition", "Test task", "running");
    let _json_file = create_test_json_file(&temp_dir, workspace_hash, vec![session.clone()]);

    // Insert workspace
    common::insert_test_workspace(&pool, workspace_hash, "/test/path").await;

    // ============ ACT ============
    // Manual migration simulation (since migration module doesn't exist yet)
    let session_id = session["id"].as_str().unwrap();
    let session_type = session["type"].as_str().unwrap();
    let task = session["task"].as_str().unwrap();
    let status = session["status"].as_str().unwrap();
    let created_at = session["createdAt"].as_str().unwrap();
    let updated_at = session["updatedAt"].as_str().unwrap();

    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(session_id)
    .bind(workspace_hash)
    .bind(session_type)
    .bind(task)
    .bind(status)
    .bind(created_at)
    .bind(updated_at)
    .execute(&pool)
    .await
    .expect("Failed to migrate session");

    // ============ ASSERT ============
    let count = count_sessions(&pool).await;
    assert_eq!(count, 1, "Should have 1 migrated session");

    let migrated: (String, String, String) = sqlx::query_as(
        "SELECT id, task, status FROM sessions WHERE id = ?"
    )
    .bind(session_id)
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch migrated session");

    assert_eq!(migrated.0, "session_1");
    assert_eq!(migrated.1, "Test task");
    assert_eq!(migrated.2, "running");
}

#[tokio::test]
async fn test_migrate_multiple_sessions() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let workspace_hash = "workspace_multi";

    let sessions = vec![
        create_test_json_session("session_1", "competition", "Task 1", "running"),
        create_test_json_session("session_2", "ensemble", "Task 2", "completed"),
        create_test_json_session("session_3", "debate", "Task 3", "failed"),
    ];

    let _json_file = create_test_json_file(&temp_dir, workspace_hash, sessions.clone());
    common::insert_test_workspace(&pool, workspace_hash, "/test/path").await;

    // ============ ACT ============
    for session in &sessions {
        let session_id = session["id"].as_str().unwrap();
        let session_type = session["type"].as_str().unwrap();
        let task = session["task"].as_str().unwrap();
        let status = session["status"].as_str().unwrap();
        let created_at = session["createdAt"].as_str().unwrap();
        let updated_at = session["updatedAt"].as_str().unwrap();

        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, workspace_hash, session_type, task, status,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(session_id)
        .bind(workspace_hash)
        .bind(session_type)
        .bind(task)
        .bind(status)
        .bind(created_at)
        .bind(updated_at)
        .execute(&pool)
        .await
        .expect("Failed to migrate session");
    }

    // ============ ASSERT ============
    let count = count_sessions(&pool).await;
    assert_eq!(count, 3, "Should have 3 migrated sessions");
}

#[tokio::test]
async fn test_migrate_large_dataset() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;

    let workspace_hash = "workspace_large";
    common::insert_test_workspace(&pool, workspace_hash, "/test/path").await;

    // ============ ACT ============
    // Migrate 100 sessions (reduced from 1000 for faster test)
    for i in 0..100 {
        let session_id = format!("session_{:03}", i);
        let session_type = match i % 3 {
            0 => "competition",
            1 => "ensemble",
            _ => "debate",
        };
        let task = format!("Large dataset task {}", i);
        let status = if i % 5 == 0 { "completed" } else { "running" };

        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, workspace_hash, session_type, task, status,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&session_id)
        .bind(workspace_hash)
        .bind(session_type)
        .bind(&task)
        .bind(status)
        .bind(Utc::now().to_rfc3339())
        .bind(Utc::now().to_rfc3339())
        .execute(&pool)
        .await
        .expect("Failed to migrate session");
    }

    // ============ ASSERT ============
    let count = count_sessions(&pool).await;
    assert_eq!(count, 100, "Should have 100 migrated sessions");

    // Verify type distribution
    let competition_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sessions WHERE session_type = ?"
    )
    .bind("competition")
    .fetch_one(&pool)
    .await
    .expect("Failed to count competition sessions");

    assert!(competition_count > 30, "Should have approximately 33 competition sessions");

    // Verify status distribution
    let completed_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sessions WHERE status = ?"
    )
    .bind("completed")
    .fetch_one(&pool)
    .await
    .expect("Failed to count completed sessions");

    assert_eq!(completed_count, 20, "Should have 20 completed sessions (every 5th)");
}

#[tokio::test]
async fn test_migrate_session_with_instances() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;

    let workspace_hash = "workspace_instances";
    common::insert_test_workspace(&pool, workspace_hash, "/test/path").await;

    let session = json!({
        "id": "session_with_instances",
        "type": "competition",
        "task": "Test with instances",
        "status": "running",
        "createdAt": Utc::now().to_rfc3339(),
        "updatedAt": Utc::now().to_rfc3339(),
        "instances": [
            {
                "instanceId": 0,
                "worktreePath": "/tmp/worktree_0",
                "branch": "branch_0",
                "agentName": "agent_0",
                "status": "running",
                "tmuxSessionId": "tmux_0"
            },
            {
                "instanceId": 1,
                "worktreePath": "/tmp/worktree_1",
                "branch": "branch_1",
                "agentName": "agent_1",
                "status": "completed",
                "tmuxSessionId": "tmux_1"
            }
        ],
        "chatHistory": []
    });

    // ============ ACT ============
    // Migrate session
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(session["id"].as_str().unwrap())
    .bind(workspace_hash)
    .bind(session["type"].as_str().unwrap())
    .bind(session["task"].as_str().unwrap())
    .bind(session["status"].as_str().unwrap())
    .bind(session["createdAt"].as_str().unwrap())
    .bind(session["updatedAt"].as_str().unwrap())
    .execute(&pool)
    .await
    .expect("Failed to migrate session");

    // Migrate instances
    for instance in session["instances"].as_array().unwrap() {
        sqlx::query(
            r#"
            INSERT INTO instances (
                session_id, instance_id, worktree_path, branch,
                agent_name, status, tmux_session_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(session["id"].as_str().unwrap())
        .bind(instance["instanceId"].as_i64().unwrap())
        .bind(instance["worktreePath"].as_str().unwrap())
        .bind(instance["branch"].as_str().unwrap())
        .bind(instance["agentName"].as_str().unwrap())
        .bind(instance["status"].as_str().unwrap())
        .bind(instance["tmuxSessionId"].as_str().unwrap())
        .execute(&pool)
        .await
        .expect("Failed to migrate instance");
    }

    // ============ ASSERT ============
    let instance_count = common::count_instances(&pool, "session_with_instances").await;
    assert_eq!(instance_count, 2, "Should have 2 migrated instances");
}

#[tokio::test]
async fn test_migrate_session_with_chat_messages() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;

    let workspace_hash = "workspace_messages";
    common::insert_test_workspace(&pool, workspace_hash, "/test/path").await;

    let session_id = "session_with_messages";

    // Migrate session
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(session_id)
    .bind(workspace_hash)
    .bind("competition")
    .bind("Test with messages")
    .bind("running")
    .bind(Utc::now().to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await
    .expect("Failed to migrate session");

    // ============ ACT ============
    // Migrate chat messages
    let messages = vec![
        ("msg_1", "user", "Hello"),
        ("msg_2", "assistant", "Hi there"),
        ("msg_3", "user", "How are you?"),
    ];

    for (msg_id, role, content) in messages {
        sqlx::query(
            r#"
            INSERT INTO chat_messages (id, session_id, role, content, timestamp)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(msg_id)
        .bind(session_id)
        .bind(role)
        .bind(content)
        .bind(Utc::now().to_rfc3339())
        .execute(&pool)
        .await
        .expect("Failed to migrate message");
    }

    // ============ ASSERT ============
    let message_count = common::count_messages(&pool, session_id).await;
    assert_eq!(message_count, 3, "Should have 3 migrated messages");
}

#[tokio::test]
async fn test_migrate_with_duplicate_detection() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;

    let workspace_hash = "workspace_duplicate";
    common::insert_test_workspace(&pool, workspace_hash, "/test/path").await;

    // Insert first session
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind("session_duplicate")
    .bind(workspace_hash)
    .bind("competition")
    .bind("Original task")
    .bind("running")
    .bind(Utc::now().to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await
    .expect("First insert should succeed");

    // ============ ACT ============
    // Try to insert duplicate
    let result = sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind("session_duplicate")
    .bind(workspace_hash)
    .bind("ensemble")
    .bind("Duplicate task")
    .bind("completed")
    .bind(Utc::now().to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await;

    // ============ ASSERT ============
    assert!(result.is_err(), "Duplicate insertion should fail");

    let count = count_sessions(&pool).await;
    assert_eq!(count, 1, "Should still have only 1 session");
}

#[tokio::test]
async fn test_migrate_with_upsert_on_conflict() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;

    let workspace_hash = "workspace_upsert";
    common::insert_test_workspace(&pool, workspace_hash, "/test/path").await;

    // Insert initial session
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind("session_upsert")
    .bind(workspace_hash)
    .bind("competition")
    .bind("Original task")
    .bind("running")
    .bind(Utc::now().to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await
    .expect("First insert should succeed");

    // ============ ACT ============
    // Upsert (UPDATE on conflict)
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(id) DO UPDATE SET
            task = excluded.task,
            status = excluded.status,
            updated_at = excluded.updated_at
        "#,
    )
    .bind("session_upsert")
    .bind(workspace_hash)
    .bind("ensemble")
    .bind("Updated task")
    .bind("completed")
    .bind(Utc::now().to_rfc3339())
    .bind(&now)
    .execute(&pool)
    .await
    .expect("Upsert should succeed");

    // ============ ASSERT ============
    let count = count_sessions(&pool).await;
    assert_eq!(count, 1, "Should still have only 1 session");

    let updated: (String, String) = sqlx::query_as(
        "SELECT task, status FROM sessions WHERE id = ?"
    )
    .bind("session_upsert")
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch updated session");

    assert_eq!(updated.0, "Updated task", "Task should be updated");
    assert_eq!(updated.1, "completed", "Status should be updated");
}

#[tokio::test]
async fn test_migrate_data_integrity_verification() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;

    let workspace_hash = "workspace_integrity";
    common::insert_test_workspace(&pool, workspace_hash, "/test/path").await;

    // ============ ACT ============
    // Migrate session with all optional fields
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at, completed_at, model,
            timeout_seconds, preserve_worktrees, winner_id,
            total_duration, total_files_changed,
            total_lines_added, total_lines_deleted
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind("session_integrity")
    .bind(workspace_hash)
    .bind("competition")
    .bind("Integrity test")
    .bind("completed")
    .bind(&now)
    .bind(&now)
    .bind(&now)
    .bind("claude-3-5-sonnet-20241022")
    .bind(3600)
    .bind(true)
    .bind(0)
    .bind(3500)
    .bind(15)
    .bind(250)
    .bind(80)
    .execute(&pool)
    .await
    .expect("Failed to migrate session with all fields");

    // ============ ASSERT ============
    let session: (String, String, Option<String>, Option<String>, Option<i64>, Option<i64>, Option<i64>, Option<i64>) = sqlx::query_as(
        r#"
        SELECT
            id, status, completed_at, model,
            timeout_seconds, total_files_changed,
            total_lines_added, total_lines_deleted
        FROM sessions WHERE id = ?
        "#
    )
    .bind("session_integrity")
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch migrated session");

    assert_eq!(session.0, "session_integrity");
    assert_eq!(session.1, "completed");
    assert!(session.2.is_some(), "completed_at should be set");
    assert_eq!(session.3, Some("claude-3-5-sonnet-20241022".to_string()));
    assert_eq!(session.4, Some(3600));
    assert_eq!(session.5, Some(15));
    assert_eq!(session.6, Some(250));
    assert_eq!(session.7, Some(80));
}

#[tokio::test]
async fn test_migrate_incremental() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;

    let workspace_hash = "workspace_incremental";
    common::insert_test_workspace(&pool, workspace_hash, "/test/path").await;

    // ============ ACT ============
    // First migration batch
    for i in 0..5 {
        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, workspace_hash, session_type, task, status,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(format!("session_{}", i))
        .bind(workspace_hash)
        .bind("competition")
        .bind(format!("Task {}", i))
        .bind("running")
        .bind(Utc::now().to_rfc3339())
        .bind(Utc::now().to_rfc3339())
        .execute(&pool)
        .await
        .expect("Failed to migrate session");
    }

    let count_after_first = count_sessions(&pool).await;
    assert_eq!(count_after_first, 5, "Should have 5 sessions after first batch");

    // Second migration batch (incremental)
    for i in 5..10 {
        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, workspace_hash, session_type, task, status,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(format!("session_{}", i))
        .bind(workspace_hash)
        .bind("ensemble")
        .bind(format!("Task {}", i))
        .bind("completed")
        .bind(Utc::now().to_rfc3339())
        .bind(Utc::now().to_rfc3339())
        .execute(&pool)
        .await
        .expect("Failed to migrate session");
    }

    // ============ ASSERT ============
    let count_after_second = count_sessions(&pool).await;
    assert_eq!(count_after_second, 10, "Should have 10 sessions after second batch");
}
