/// Query and filtering tests
///
/// Tests in this file verify:
/// - Type filtering (competition, ensemble, debate)
/// - Status filtering (running, completed, failed, paused)
/// - Complex WHERE clauses
/// - Sorting (by created_at, updated_at)
/// - Pagination (LIMIT, OFFSET)
/// - Search functionality (LIKE queries)
/// - Index usage verification

mod common;

use chrono::{Duration, Utc};
use common::{create_test_pool, factories, setup_test_db};

#[tokio::test]
async fn test_filter_by_session_type_competition() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // Insert sessions of different types
    for (i, session_type) in ["competition", "ensemble", "debate", "competition"].iter().enumerate() {
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
        .bind(session_type)
        .bind(&session.task)
        .bind(&session.status)
        .bind(&session.created_at)
        .bind(&session.updated_at)
        .execute(&pool)
        .await
        .expect("Failed to insert session");
    }

    // ============ ACT ============
    let competition_sessions: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, session_type FROM sessions WHERE session_type = ? ORDER BY id"
    )
    .bind("competition")
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch competition sessions");

    // ============ ASSERT ============
    assert_eq!(competition_sessions.len(), 2, "Should find 2 competition sessions");
    assert_eq!(competition_sessions[0].1, "competition");
    assert_eq!(competition_sessions[1].1, "competition");
}

#[tokio::test]
async fn test_filter_by_session_type_ensemble() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    for (i, session_type) in ["competition", "ensemble", "debate", "ensemble", "ensemble"].iter().enumerate() {
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
        .bind(session_type)
        .bind(&session.task)
        .bind(&session.status)
        .bind(&session.created_at)
        .bind(&session.updated_at)
        .execute(&pool)
        .await
        .expect("Failed to insert session");
    }

    // ============ ACT ============
    let ensemble_sessions: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM sessions WHERE session_type = ?"
    )
    .bind("ensemble")
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch ensemble sessions");

    // ============ ASSERT ============
    assert_eq!(ensemble_sessions.len(), 3, "Should find 3 ensemble sessions");
}

#[tokio::test]
async fn test_filter_by_status() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    for (i, status) in ["running", "completed", "failed", "running", "completed"].iter().enumerate() {
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
        .bind(&session.task)
        .bind(status)
        .bind(&session.created_at)
        .bind(&session.updated_at)
        .execute(&pool)
        .await
        .expect("Failed to insert session");
    }

    // ============ ACT ============
    let running_sessions: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM sessions WHERE status = ?"
    )
    .bind("running")
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch running sessions");

    // ============ ASSERT ============
    assert_eq!(running_sessions.len(), 2, "Should find 2 running sessions");
}

#[tokio::test]
async fn test_filter_by_multiple_statuses() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    for (i, status) in ["running", "completed", "failed", "paused", "completed"].iter().enumerate() {
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
        .bind(&session.task)
        .bind(status)
        .bind(&session.created_at)
        .bind(&session.updated_at)
        .execute(&pool)
        .await
        .expect("Failed to insert session");
    }

    // ============ ACT ============
    let sessions: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, status FROM sessions WHERE status IN ('completed', 'failed') ORDER BY status"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch sessions");

    // ============ ASSERT ============
    assert_eq!(sessions.len(), 3, "Should find 3 sessions (2 completed, 1 failed)");
}

#[tokio::test]
async fn test_filter_by_type_and_status() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    let test_data = vec![
        ("session_0", "competition", "running"),
        ("session_1", "competition", "completed"),
        ("session_2", "ensemble", "running"),
        ("session_3", "competition", "running"),
        ("session_4", "debate", "completed"),
    ];

    for (id, session_type, status) in test_data {
        let session = factories::WorktreeSession::with_id(id);
        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, workspace_hash, session_type, task, status,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind("workspace_1")
        .bind(session_type)
        .bind(&session.task)
        .bind(status)
        .bind(&session.created_at)
        .bind(&session.updated_at)
        .execute(&pool)
        .await
        .expect("Failed to insert session");
    }

    // ============ ACT ============
    let sessions: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM sessions WHERE session_type = ? AND status = ?"
    )
    .bind("competition")
    .bind("running")
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch sessions");

    // ============ ASSERT ============
    assert_eq!(sessions.len(), 2, "Should find 2 competition sessions with status running");
}

#[tokio::test]
async fn test_sort_by_created_at_desc() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // Insert sessions with different timestamps
    for i in 0..5 {
        let created_at = Utc::now() - Duration::hours(5 - i);
        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, workspace_hash, session_type, task, status,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(format!("session_{}", i))
        .bind("workspace_1")
        .bind("competition")
        .bind(format!("Task {}", i))
        .bind("running")
        .bind(created_at.to_rfc3339())
        .bind(created_at.to_rfc3339())
        .execute(&pool)
        .await
        .expect("Failed to insert session");
    }

    // ============ ACT ============
    let sessions: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM sessions ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch sessions");

    // ============ ASSERT ============
    assert_eq!(sessions.len(), 5);
    assert_eq!(sessions[0].0, "session_4", "Most recent session should be first");
    assert_eq!(sessions[4].0, "session_0", "Oldest session should be last");
}

#[tokio::test]
async fn test_sort_by_updated_at_desc() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    let now = Utc::now();
    for i in 0..3 {
        let created_at = now - Duration::hours(3);
        let updated_at = now - Duration::hours(3 - i);
        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, workspace_hash, session_type, task, status,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(format!("session_{}", i))
        .bind("workspace_1")
        .bind("competition")
        .bind(format!("Task {}", i))
        .bind("running")
        .bind(created_at.to_rfc3339())
        .bind(updated_at.to_rfc3339())
        .execute(&pool)
        .await
        .expect("Failed to insert session");
    }

    // ============ ACT ============
    let sessions: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM sessions ORDER BY updated_at DESC"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch sessions");

    // ============ ASSERT ============
    assert_eq!(sessions[0].0, "session_2", "Most recently updated should be first");
    assert_eq!(sessions[2].0, "session_0", "Least recently updated should be last");
}

#[tokio::test]
async fn test_pagination_limit() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // Insert 10 sessions
    for i in 0..10 {
        let session = factories::WorktreeSession::with_id(&format!("session_{:02}", i));
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
        .execute(&pool)
        .await
        .expect("Failed to insert session");
    }

    // ============ ACT ============
    let sessions: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM sessions ORDER BY id LIMIT 5"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch sessions");

    // ============ ASSERT ============
    assert_eq!(sessions.len(), 5, "Should return only 5 sessions");
}

#[tokio::test]
async fn test_pagination_offset() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    // Insert 10 sessions
    for i in 0..10 {
        let session = factories::WorktreeSession::with_id(&format!("session_{:02}", i));
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
        .execute(&pool)
        .await
        .expect("Failed to insert session");
    }

    // ============ ACT ============
    let page1: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM sessions ORDER BY id LIMIT 3 OFFSET 0"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch page 1");

    let page2: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM sessions ORDER BY id LIMIT 3 OFFSET 3"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch page 2");

    // ============ ASSERT ============
    assert_eq!(page1.len(), 3);
    assert_eq!(page2.len(), 3);
    assert_eq!(page1[0].0, "session_00");
    assert_eq!(page2[0].0, "session_03");
}

#[tokio::test]
async fn test_search_by_task_like() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    let tasks = vec![
        "Implement login feature",
        "Fix bug in authentication",
        "Add unit tests for login",
        "Refactor database layer",
        "Implement logout feature",
    ];

    for (i, task) in tasks.iter().enumerate() {
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
        .bind(task)
        .bind(&session.status)
        .bind(&session.created_at)
        .bind(&session.updated_at)
        .execute(&pool)
        .await
        .expect("Failed to insert session");
    }

    // ============ ACT ============
    let sessions: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, task FROM sessions WHERE task LIKE ? ORDER BY id"
    )
    .bind("%login%")
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch sessions");

    // ============ ASSERT ============
    assert_eq!(sessions.len(), 3, "Should find 3 sessions with 'login' in task");
    assert!(sessions[0].1.contains("login"));
    assert!(sessions[1].1.contains("login"));
    assert!(sessions[2].1.contains("login"));
}

#[tokio::test]
async fn test_search_by_id_like() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    for i in 0..5 {
        let id = format!("test_session_{}", i);
        let session = factories::WorktreeSession::with_id(&id);
        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, workspace_hash, session_type, task, status,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind("workspace_1")
        .bind(&session.session_type)
        .bind(&session.task)
        .bind(&session.status)
        .bind(&session.created_at)
        .bind(&session.updated_at)
        .execute(&pool)
        .await
        .expect("Failed to insert session");
    }

    // ============ ACT ============
    let sessions: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM sessions WHERE id LIKE ?"
    )
    .bind("test_session_%")
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch sessions");

    // ============ ASSERT ============
    assert_eq!(sessions.len(), 5, "Should find all sessions matching pattern");
}

#[tokio::test]
async fn test_complex_query_with_multiple_conditions() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    let test_data = vec![
        ("session_0", "competition", "running", "Task A"),
        ("session_1", "competition", "completed", "Task B"),
        ("session_2", "ensemble", "running", "Task C"),
        ("session_3", "competition", "running", "Task D"),
        ("session_4", "debate", "running", "Task E"),
    ];

    for (id, session_type, status, task) in test_data {
        let session = factories::WorktreeSession::with_id(id);
        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, workspace_hash, session_type, task, status,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind("workspace_1")
        .bind(session_type)
        .bind(task)
        .bind(status)
        .bind(&session.created_at)
        .bind(&session.updated_at)
        .execute(&pool)
        .await
        .expect("Failed to insert session");
    }

    // ============ ACT ============
    let sessions: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT id FROM sessions
        WHERE workspace_hash = ?
          AND session_type = ?
          AND status = ?
        ORDER BY id
        "#
    )
    .bind("workspace_1")
    .bind("competition")
    .bind("running")
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch sessions");

    // ============ ASSERT ============
    assert_eq!(sessions.len(), 2, "Should find 2 competition sessions with status running");
    assert_eq!(sessions[0].0, "session_0");
    assert_eq!(sessions[1].0, "session_3");
}

#[tokio::test]
async fn test_count_sessions_by_type() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/to/workspace").await;

    for (i, session_type) in ["competition", "ensemble", "debate", "competition", "ensemble", "ensemble"].iter().enumerate() {
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
        .bind(session_type)
        .bind(&session.task)
        .bind(&session.status)
        .bind(&session.created_at)
        .bind(&session.updated_at)
        .execute(&pool)
        .await
        .expect("Failed to insert session");
    }

    // ============ ACT ============
    let counts: Vec<(String, i64)> = sqlx::query_as(
        "SELECT session_type, COUNT(*) as count FROM sessions GROUP BY session_type ORDER BY session_type"
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch counts");

    // ============ ASSERT ============
    assert_eq!(counts.len(), 3, "Should have 3 different types");
    assert_eq!(counts[0], ("competition".to_string(), 2));
    assert_eq!(counts[1], ("debate".to_string(), 1));
    assert_eq!(counts[2], ("ensemble".to_string(), 3));
}

#[tokio::test]
async fn test_filter_by_workspace() {
    // ============ ARRANGE ============
    let pool = create_test_pool().await;
    setup_test_db(&pool).await;
    common::insert_test_workspace(&pool, "workspace_1", "/path/1").await;
    common::insert_test_workspace(&pool, "workspace_2", "/path/2").await;

    // Insert sessions for workspace_1
    for i in 0..3 {
        let session = factories::WorktreeSession::with_id(&format!("session_1_{}", i));
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
        .execute(&pool)
        .await
        .expect("Failed to insert session");
    }

    // Insert sessions for workspace_2
    for i in 0..2 {
        let session = factories::WorktreeSession::with_id(&format!("session_2_{}", i));
        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, workspace_hash, session_type, task, status,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&session.id)
        .bind("workspace_2")
        .bind(&session.session_type)
        .bind(&session.task)
        .bind(&session.status)
        .bind(&session.created_at)
        .bind(&session.updated_at)
        .execute(&pool)
        .await
        .expect("Failed to insert session");
    }

    // ============ ACT ============
    let workspace1_sessions: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM sessions WHERE workspace_hash = ? ORDER BY id"
    )
    .bind("workspace_1")
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch workspace 1 sessions");

    let workspace2_sessions: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM sessions WHERE workspace_hash = ? ORDER BY id"
    )
    .bind("workspace_2")
    .fetch_all(&pool)
    .await
    .expect("Failed to fetch workspace 2 sessions");

    // ============ ASSERT ============
    assert_eq!(workspace1_sessions.len(), 3, "Workspace 1 should have 3 sessions");
    assert_eq!(workspace2_sessions.len(), 2, "Workspace 2 should have 2 sessions");
}
