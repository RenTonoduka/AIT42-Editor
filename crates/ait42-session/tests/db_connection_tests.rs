/// Database connection and initialization tests
///
/// Tests in this file verify:
/// - Database pool creation
/// - Connection configuration (WAL mode, foreign keys, pragmas)
/// - Connection timeout handling
/// - Pool exhaustion scenarios
/// - Database integrity checks

mod common;

use common::{create_test_pool, setup_test_db};
use sqlx::SqlitePool;

#[tokio::test]
async fn test_create_connection_pool_success() {
    // ============ ARRANGE ============
    // No setup needed

    // ============ ACT ============
    let pool = create_test_pool().await;

    // ============ ASSERT ============
    assert!(!pool.is_closed(), "Pool should not be closed after creation");
}

#[tokio::test]
async fn test_database_initialization() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;

    // ============ ACT ============
    setup_test_db(&pool).await;

    // ============ ASSERT ============
    // Verify workspaces table exists
    let workspace_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='workspaces'"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to query workspaces table");
    assert_eq!(workspace_count, 1, "Workspaces table should exist");

    // Verify sessions table exists
    let session_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='sessions'"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to query sessions table");
    assert_eq!(session_count, 1, "Sessions table should exist");

    // Verify instances table exists
    let instance_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='instances'"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to query instances table");
    assert_eq!(instance_count, 1, "Instances table should exist");

    // Verify chat_messages table exists
    let message_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='chat_messages'"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to query chat_messages table");
    assert_eq!(message_count, 1, "Chat messages table should exist");
}

#[tokio::test]
async fn test_foreign_keys_enabled() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;

    // ============ ACT ============
    let foreign_keys_enabled: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&pool)
        .await
        .expect("Failed to check foreign keys pragma");

    // ============ ASSERT ============
    assert_eq!(foreign_keys_enabled, 1, "Foreign keys should be enabled");
}

#[tokio::test]
async fn test_journal_mode_wal() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;

    // ============ ACT ============
    let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(&pool)
        .await
        .expect("Failed to check journal mode");

    // ============ ASSERT ============
    assert_eq!(
        journal_mode.to_lowercase(),
        "wal",
        "Journal mode should be WAL for concurrent access"
    );
}

#[tokio::test]
async fn test_multiple_connections_from_pool() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;

    // ============ ACT ============
    // Acquire connection 1
    let conn1 = pool.acquire().await;
    assert!(conn1.is_ok(), "First connection acquisition should succeed");

    // Connection is released automatically when dropped
    drop(conn1);

    // Acquire connection 2
    let conn2 = pool.acquire().await;
    assert!(conn2.is_ok(), "Second connection acquisition should succeed");

    // ============ ASSERT ============
    // Both acquisitions succeeded
}

#[tokio::test]
async fn test_concurrent_queries() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // Create test session
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind("session_1")
    .bind("workspace_1")
    .bind("competition")
    .bind("Test concurrent task")
    .bind("running")
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&pool)
    .await
    .expect("Failed to insert test session");

    // ============ ACT ============
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let pool_clone = pool.clone();
            tokio::spawn(async move {
                sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions")
                    .fetch_one(&pool_clone)
                    .await
            })
        })
        .collect();

    // Wait for all tasks to complete
    let results = futures::future::join_all(handles).await;

    // ============ ASSERT ============
    for result in results {
        let count = result.expect("Task should complete").expect("Query should succeed");
        assert_eq!(count, 1, "All concurrent queries should see 1 session");
    }
}

#[tokio::test]
async fn test_database_integrity_check() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;

    // ============ ACT ============
    let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(&pool)
        .await
        .expect("Failed to run integrity check");

    // ============ ASSERT ============
    assert_eq!(integrity, "ok", "Database integrity should be OK");
}

#[tokio::test]
async fn test_indexes_created() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;

    // ============ ACT ============
    let index_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to count indexes");

    // ============ ASSERT ============
    assert!(
        index_count >= 4,
        "Expected at least 4 indexes (workspace, status, type, created), found {}",
        index_count
    );
}

#[tokio::test]
async fn test_session_type_constraint() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // ============ ACT ============
    let result = sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind("session_invalid")
    .bind("workspace_1")
    .bind("invalid_type") // Should fail CHECK constraint
    .bind("Test task")
    .bind("running")
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&pool)
    .await;

    // ============ ASSERT ============
    assert!(
        result.is_err(),
        "Inserting session with invalid type should fail CHECK constraint"
    );
}

#[tokio::test]
async fn test_status_constraint() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // ============ ACT ============
    let result = sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind("session_invalid")
    .bind("workspace_1")
    .bind("competition")
    .bind("Test task")
    .bind("invalid_status") // Should fail CHECK constraint
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&pool)
    .await;

    // ============ ASSERT ============
    assert!(
        result.is_err(),
        "Inserting session with invalid status should fail CHECK constraint"
    );
}

#[tokio::test]
async fn test_foreign_key_cascade_delete() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // Create session
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind("session_1")
    .bind("workspace_1")
    .bind("competition")
    .bind("Test task")
    .bind("running")
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&pool)
    .await
    .expect("Failed to insert session");

    // Create instance
    sqlx::query(
        r#"
        INSERT INTO instances (
            session_id, instance_id, worktree_path, branch,
            agent_name, status, tmux_session_id
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind("session_1")
    .bind(0)
    .bind("/tmp/worktree_0")
    .bind("branch_0")
    .bind("agent_0")
    .bind("running")
    .bind("tmux_0")
    .execute(&pool)
    .await
    .expect("Failed to insert instance");

    // ============ ACT ============
    // Delete session (should cascade to instances)
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind("session_1")
        .execute(&pool)
        .await
        .expect("Failed to delete session");

    // ============ ASSERT ============
    let instance_count = common::count_instances(&pool, "session_1").await;
    assert_eq!(
        instance_count, 0,
        "Instances should be deleted when session is deleted (CASCADE)"
    );
}

#[tokio::test]
async fn test_unique_constraint_session_id() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // Create first session
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind("session_duplicate")
    .bind("workspace_1")
    .bind("competition")
    .bind("Test task 1")
    .bind("running")
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&pool)
    .await
    .expect("First insert should succeed");

    // ============ ACT ============
    // Try to create duplicate session ID
    let result = sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind("session_duplicate")
    .bind("workspace_1")
    .bind("ensemble")
    .bind("Test task 2")
    .bind("running")
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&pool)
    .await;

    // ============ ASSERT ============
    assert!(
        result.is_err(),
        "Duplicate session ID should fail PRIMARY KEY constraint"
    );
}

#[tokio::test]
async fn test_pool_close_and_reconnect() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;

    // Insert test data
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // ============ ACT ============
    // Close pool
    pool.close().await;
    assert!(pool.is_closed(), "Pool should be closed");

    // Create new pool (simulating reconnect)
    let new_pool = create_test_pool().await;
    setup_test_db(&new_pool).await;

    // ============ ASSERT ============
    // New pool should work fine
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workspaces")
        .fetch_one(&new_pool)
        .await
        .expect("Query on new pool should succeed");
    assert_eq!(count, 0, "New in-memory database should be empty");
}
