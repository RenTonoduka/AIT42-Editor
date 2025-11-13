/// Transaction and concurrency tests
///
/// Tests in this file verify:
/// - Transaction BEGIN/COMMIT
/// - Transaction ROLLBACK on error
/// - Concurrent write handling
/// - Isolation levels
/// - Deadlock prevention
/// - Atomic multi-table operations

mod common;

use common::{create_test_pool, factories, setup_test_db, count_sessions, count_instances};

#[tokio::test]
async fn test_transaction_commit() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // ============ ACT ============
    let mut tx = pool.begin().await.expect("Failed to begin transaction");

    // Insert session within transaction
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
    .execute(&mut *tx)
    .await
    .expect("Failed to insert session");

    // Commit transaction
    tx.commit().await.expect("Failed to commit transaction");

    // ============ ASSERT ============
    let count = count_sessions(&pool).await;
    assert_eq!(count, 1, "Session should exist after commit");
}

#[tokio::test]
async fn test_transaction_rollback_on_error() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // ============ ACT ============
    let mut tx = pool.begin().await.expect("Failed to begin transaction");

    // Insert valid session
    let session = factories::WorktreeSession::with_id("session_1");
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
    .execute(&mut *tx)
    .await
    .expect("Failed to insert session");

    // Attempt to insert duplicate (should fail)
    let result = sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind("session_1") // Duplicate ID
    .bind(&session.workspace_hash)
    .bind(&session.session_type)
    .bind("Different task")
    .bind(&session.status)
    .bind(&session.created_at)
    .bind(&session.updated_at)
    .execute(&mut *tx)
    .await;

    assert!(result.is_err(), "Duplicate insert should fail");

    // Rollback transaction
    tx.rollback().await.expect("Failed to rollback transaction");

    // ============ ASSERT ============
    let count = count_sessions(&pool).await;
    assert_eq!(count, 0, "No sessions should exist after rollback");
}

#[tokio::test]
async fn test_transaction_explicit_rollback() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // ============ ACT ============
    let mut tx = pool.begin().await.expect("Failed to begin transaction");

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
    .execute(&mut *tx)
    .await
    .expect("Failed to insert session");

    // Explicitly rollback
    tx.rollback().await.expect("Failed to rollback");

    // ============ ASSERT ============
    let count = count_sessions(&pool).await;
    assert_eq!(count, 0, "Session should not exist after explicit rollback");
}

#[tokio::test]
async fn test_transaction_multi_table_insert() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // ============ ACT ============
    let mut tx = pool.begin().await.expect("Failed to begin transaction");

    // Insert session
    let session = factories::WorktreeSession::with_id("session_multi");
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
    .execute(&mut *tx)
    .await
    .expect("Failed to insert session");

    // Insert instances
    for i in 0..3 {
        let instance = factories::WorktreeInstance::default("session_multi", i);
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
        .execute(&mut *tx)
        .await
        .expect("Failed to insert instance");
    }

    // Insert chat messages
    for i in 0..2 {
        let message = factories::ChatMessage::default("session_multi");
        sqlx::query(
            r#"
            INSERT INTO chat_messages (id, session_id, role, content, timestamp)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&message.id)
        .bind(&message.session_id)
        .bind(&message.role)
        .bind(format!("Message {}", i))
        .bind(&message.timestamp)
        .execute(&mut *tx)
        .await
        .expect("Failed to insert message");
    }

    // Commit all changes
    tx.commit().await.expect("Failed to commit");

    // ============ ASSERT ============
    let session_count = count_sessions(&pool).await;
    assert_eq!(session_count, 1, "Should have 1 session");

    let instance_count = count_instances(&pool, "session_multi").await;
    assert_eq!(instance_count, 3, "Should have 3 instances");

    let message_count = common::count_messages(&pool, "session_multi").await;
    assert_eq!(message_count, 2, "Should have 2 messages");
}

#[tokio::test]
async fn test_transaction_multi_table_rollback() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // ============ ACT ============
    let mut tx = pool.begin().await.expect("Failed to begin transaction");

    // Insert session
    let session = factories::WorktreeSession::with_id("session_rollback");
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
    .execute(&mut *tx)
    .await
    .expect("Failed to insert session");

    // Insert instances
    for i in 0..2 {
        let instance = factories::WorktreeInstance::default("session_rollback", i);
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
        .execute(&mut *tx)
        .await
        .expect("Failed to insert instance");
    }

    // Rollback
    tx.rollback().await.expect("Failed to rollback");

    // ============ ASSERT ============
    let session_count = count_sessions(&pool).await;
    assert_eq!(session_count, 0, "Session should not exist after rollback");

    let instance_count = count_instances(&pool, "session_rollback").await;
    assert_eq!(instance_count, 0, "Instances should not exist after rollback");
}

#[tokio::test]
async fn test_concurrent_read_operations() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // Insert test session
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
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let pool_clone = pool.clone();
            tokio::spawn(async move {
                sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions")
                    .fetch_one(&pool_clone)
                    .await
            })
        })
        .collect();

    let results = futures::future::join_all(handles).await;

    // ============ ASSERT ============
    for result in results {
        let count = result.expect("Task should succeed").expect("Query should succeed");
        assert_eq!(count, 1, "All concurrent reads should see 1 session");
    }
}

#[tokio::test]
async fn test_concurrent_write_operations() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // ============ ACT ============
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let pool_clone = pool.clone();
            tokio::spawn(async move {
                let session = factories::WorktreeSession::with_id(&format!("concurrent_session_{}", i));
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
                .bind(&session.task)
                .bind(&session.status)
                .bind(&session.created_at)
                .bind(&session.updated_at)
                .execute(&pool_clone)
                .await
            })
        })
        .collect();

    let results = futures::future::join_all(handles).await;

    // ============ ASSERT ============
    let mut success_count = 0;
    for result in results {
        if result.expect("Task should complete").is_ok() {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 5, "All concurrent writes should succeed");

    let total_sessions = count_sessions(&pool).await;
    assert_eq!(total_sessions, 5, "Should have 5 sessions after concurrent writes");
}

#[tokio::test]
async fn test_transaction_isolation_read_uncommitted() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // ============ ACT ============
    // Start transaction 1
    let mut tx1 = pool.begin().await.expect("Failed to begin transaction 1");

    let session = factories::WorktreeSession::with_id("session_isolation");
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
    .execute(&mut *tx1)
    .await
    .expect("Failed to insert session");

    // Transaction 2 reads before tx1 commits
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions WHERE id = ?")
        .bind("session_isolation")
        .fetch_one(&pool)
        .await
        .expect("Failed to read session");

    // ============ ASSERT ============
    assert_eq!(count, 0, "Should not see uncommitted changes from other transaction");

    // Commit tx1
    tx1.commit().await.expect("Failed to commit");

    // Now should see the session
    let count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions WHERE id = ?")
        .bind("session_isolation")
        .fetch_one(&pool)
        .await
        .expect("Failed to read session");

    assert_eq!(count_after, 1, "Should see committed changes");
}

#[tokio::test]
async fn test_transaction_update_conflict() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // Insert initial session
    let session = factories::WorktreeSession::with_id("session_conflict");
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
    .bind("running")
    .bind(&session.created_at)
    .bind(&session.updated_at)
    .execute(&pool)
    .await
    .expect("Failed to insert session");

    // ============ ACT ============
    // Both transactions try to update the same session
    let pool1 = pool.clone();
    let pool2 = pool.clone();

    let handle1 = tokio::spawn(async move {
        let mut tx = pool1.begin().await.expect("Failed to begin tx1");
        sqlx::query("UPDATE sessions SET status = ? WHERE id = ?")
            .bind("completed")
            .bind("session_conflict")
            .execute(&mut *tx)
            .await
            .expect("Failed to update in tx1");
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        tx.commit().await.expect("Failed to commit tx1");
    });

    let handle2 = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        let mut tx = pool2.begin().await.expect("Failed to begin tx2");
        let result = sqlx::query("UPDATE sessions SET status = ? WHERE id = ?")
            .bind("failed")
            .bind("session_conflict")
            .execute(&mut *tx)
            .await;

        if result.is_ok() {
            let _ = tx.commit().await;
        }
        result
    });

    let _ = handle1.await;
    let result2 = handle2.await.expect("Task should complete");

    // ============ ASSERT ============
    // One of the transactions should succeed
    // In SQLite with WAL mode, both might succeed due to snapshot isolation
    // The last commit wins
    let final_status: String = sqlx::query_scalar("SELECT status FROM sessions WHERE id = ?")
        .bind("session_conflict")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch final status");

    assert!(
        final_status == "completed" || final_status == "failed",
        "Status should be either completed or failed"
    );
}

#[tokio::test]
async fn test_savepoint_rollback() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // ============ ACT ============
    let mut tx = pool.begin().await.expect("Failed to begin transaction");

    // Insert first session
    let session1 = factories::WorktreeSession::with_id("session_1");
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&session1.id)
    .bind(&session1.workspace_hash)
    .bind(&session1.session_type)
    .bind(&session1.task)
    .bind(&session1.status)
    .bind(&session1.created_at)
    .bind(&session1.updated_at)
    .execute(&mut *tx)
    .await
    .expect("Failed to insert session 1");

    // Create savepoint
    sqlx::query("SAVEPOINT sp1")
        .execute(&mut *tx)
        .await
        .expect("Failed to create savepoint");

    // Insert second session
    let session2 = factories::WorktreeSession::with_id("session_2");
    sqlx::query(
        r#"
        INSERT INTO sessions (
            id, workspace_hash, session_type, task, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&session2.id)
    .bind(&session2.workspace_hash)
    .bind(&session2.session_type)
    .bind(&session2.task)
    .bind(&session2.status)
    .bind(&session2.created_at)
    .bind(&session2.updated_at)
    .execute(&mut *tx)
    .await
    .expect("Failed to insert session 2");

    // Rollback to savepoint (session_2 should be discarded)
    sqlx::query("ROLLBACK TO SAVEPOINT sp1")
        .execute(&mut *tx)
        .await
        .expect("Failed to rollback to savepoint");

    // Commit transaction
    tx.commit().await.expect("Failed to commit");

    // ============ ASSERT ============
    let count = count_sessions(&pool).await;
    assert_eq!(count, 1, "Should have only session_1 after savepoint rollback");

    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM sessions WHERE id = ?)")
        .bind("session_1")
        .fetch_one(&pool)
        .await
        .expect("Failed to check session_1");
    assert!(exists, "session_1 should exist");

    let not_exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM sessions WHERE id = ?)")
        .bind("session_2")
        .fetch_one(&pool)
        .await
        .expect("Failed to check session_2");
    assert!(!not_exists, "session_2 should not exist");
}
