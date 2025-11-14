use ait42_session::*;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_create_and_get_session() {
    let db_file = NamedTempFile::new().unwrap();
    let repo = SqliteSessionRepository::with_path(db_file.path())
        .await
        .unwrap();

    let mut session = WorktreeSession::new(
        uuid::Uuid::new_v4().to_string(),
        "test_workspace_hash".to_string(),
        "competition".to_string(),
        "Test task".to_string(),
    );
    session.workspace_hash = Some("test_workspace_hash".to_string());

    // Create session
    let created = repo.create_session(session.clone()).await.unwrap();
    assert_eq!(created.id, session.id);
    assert_eq!(created.task, "Test task");

    // Get session
    let fetched = repo
        .get_session("test_workspace_hash", &session.id)
        .await
        .unwrap();
    assert_eq!(fetched.id, session.id);
    assert_eq!(fetched.task, "Test task");
}

#[tokio::test]
async fn test_get_all_sessions() {
    let db_file = NamedTempFile::new().unwrap();
    let repo = SqliteSessionRepository::with_path(db_file.path())
        .await
        .unwrap();

    // Create multiple sessions
    for i in 0..5 {
        let mut session = WorktreeSession::new(
            uuid::Uuid::new_v4().to_string(),
            "test_workspace_hash".to_string(),
            "competition".to_string(),
            format!("Task {}", i),
        );
        session.workspace_hash = Some("test_workspace_hash".to_string());
        repo.create_session(session).await.unwrap();
    }

    // Get all sessions
    let sessions = repo.get_all_sessions("test_workspace_hash").await.unwrap();
    assert_eq!(sessions.len(), 5);
}

#[tokio::test]
async fn test_update_session() {
    let db_file = NamedTempFile::new().unwrap();
    let repo = SqliteSessionRepository::with_path(db_file.path())
        .await
        .unwrap();

    let mut session = WorktreeSession::new(
        uuid::Uuid::new_v4().to_string(),
        "test_workspace_hash".to_string(),
        "competition".to_string(),
        "Original task".to_string(),
    );
    session.workspace_hash = Some("test_workspace_hash".to_string());

    // Create session
    let created = repo.create_session(session.clone()).await.unwrap();

    // Update session
    let mut updated = created.clone();
    updated.task = "Updated task".to_string();
    updated.status = "completed".to_string();
    updated.updated_at = chrono::Utc::now().to_rfc3339();

    let result = repo.update_session(updated.clone()).await.unwrap();
    assert_eq!(result.task, "Updated task");
    assert_eq!(result.status, "completed");
}

#[tokio::test]
async fn test_delete_session() {
    let db_file = NamedTempFile::new().unwrap();
    let repo = SqliteSessionRepository::with_path(db_file.path())
        .await
        .unwrap();

    let mut session = WorktreeSession::new(
        uuid::Uuid::new_v4().to_string(),
        "test_workspace_hash".to_string(),
        "competition".to_string(),
        "Test task".to_string(),
    );
    session.workspace_hash = Some("test_workspace_hash".to_string());

    // Create session
    repo.create_session(session.clone()).await.unwrap();

    // Delete session
    repo.delete_session("test_workspace_hash", &session.id)
        .await
        .unwrap();

    // Verify deletion
    let result = repo.get_session("test_workspace_hash", &session.id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_session_with_instances() {
    let db_file = NamedTempFile::new().unwrap();
    let repo = SqliteSessionRepository::with_path(db_file.path())
        .await
        .unwrap();

    let mut session = WorktreeSession::new(
        uuid::Uuid::new_v4().to_string(),
        "test_workspace_hash".to_string(),
        "competition".to_string(),
        "Test task".to_string(),
    );
    session.workspace_hash = Some("test_workspace_hash".to_string());

    // Add instances
    session.instances = vec![
        WorktreeInstance {
            instance_id: 0,
            session_id: None,
            worktree_path: "/tmp/worktree-0".to_string(),
            branch: "main".to_string(),
            agent_name: "Agent 0".to_string(),
            status: "running".to_string(),
            tmux_session_id: "tmux-0".to_string(),
            output: None,
            start_time: Some(chrono::Utc::now().to_rfc3339()),
            end_time: None,
            files_changed: None,
            lines_added: None,
            lines_deleted: None,
            runtime: Some("claude-code".to_string()),
            model: Some("claude-3.5-sonnet".to_string()),
            runtime_label: None,
        },
        WorktreeInstance {
            instance_id: 1,
            session_id: None,
            worktree_path: "/tmp/worktree-1".to_string(),
            branch: "main".to_string(),
            agent_name: "Agent 1".to_string(),
            status: "running".to_string(),
            tmux_session_id: "tmux-1".to_string(),
            output: None,
            start_time: Some(chrono::Utc::now().to_rfc3339()),
            end_time: None,
            files_changed: None,
            lines_added: None,
            lines_deleted: None,
            runtime: Some("claude-code".to_string()),
            model: Some("claude-3.5-sonnet".to_string()),
            runtime_label: None,
        },
    ];

    // Create session
    let created = repo.create_session(session.clone()).await.unwrap();
    assert_eq!(created.instances.len(), 2);

    // Get session and verify instances
    let fetched = repo
        .get_session("test_workspace_hash", &session.id)
        .await
        .unwrap();
    assert_eq!(fetched.instances.len(), 2);
    assert_eq!(fetched.instances[0].agent_name, "Agent 0");
    assert_eq!(fetched.instances[1].agent_name, "Agent 1");
}

#[tokio::test]
async fn test_session_with_chat_messages() {
    let db_file = NamedTempFile::new().unwrap();
    let repo = SqliteSessionRepository::with_path(db_file.path())
        .await
        .unwrap();

    let mut session = WorktreeSession::new(
        uuid::Uuid::new_v4().to_string(),
        "test_workspace_hash".to_string(),
        "competition".to_string(),
        "Test task".to_string(),
    );
    session.workspace_hash = Some("test_workspace_hash".to_string());

    // Add chat messages
    session.chat_history = vec![
        ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: None,
            role: "user".to_string(),
            content: "Hello".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            instance_id: None,
        },
        ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: None,
            role: "assistant".to_string(),
            content: "Hi there!".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            instance_id: None,
        },
    ];

    // Create session
    let created = repo.create_session(session.clone()).await.unwrap();
    assert_eq!(created.chat_history.len(), 2);

    // Get session and verify messages
    let fetched = repo
        .get_session("test_workspace_hash", &session.id)
        .await
        .unwrap();
    assert_eq!(fetched.chat_history.len(), 2);
    assert_eq!(fetched.chat_history[0].content, "Hello");
    assert_eq!(fetched.chat_history[1].content, "Hi there!");
}

#[tokio::test]
async fn test_add_chat_message() {
    let db_file = NamedTempFile::new().unwrap();
    let repo = SqliteSessionRepository::with_path(db_file.path())
        .await
        .unwrap();

    let mut session = WorktreeSession::new(
        uuid::Uuid::new_v4().to_string(),
        "test_workspace_hash".to_string(),
        "competition".to_string(),
        "Test task".to_string(),
    );
    session.workspace_hash = Some("test_workspace_hash".to_string());

    // Create session
    let created = repo.create_session(session.clone()).await.unwrap();

    // Add chat message
    let message = ChatMessage {
        id: uuid::Uuid::new_v4().to_string(),
        session_id: None,
        role: "user".to_string(),
        content: "New message".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        instance_id: None,
    };

    repo.add_chat_message(&created.id, message.clone())
        .await
        .unwrap();

    // Get session and verify message was added
    let fetched = repo
        .get_session("test_workspace_hash", &created.id)
        .await
        .unwrap();
    assert_eq!(fetched.chat_history.len(), 1);
    assert_eq!(fetched.chat_history[0].content, "New message");
}

#[tokio::test]
async fn test_update_instance_status() {
    let db_file = NamedTempFile::new().unwrap();
    let repo = SqliteSessionRepository::with_path(db_file.path())
        .await
        .unwrap();

    let mut session = WorktreeSession::new(
        uuid::Uuid::new_v4().to_string(),
        "test_workspace_hash".to_string(),
        "competition".to_string(),
        "Test task".to_string(),
    );
    session.workspace_hash = Some("test_workspace_hash".to_string());

    // Add instance
    session.instances = vec![WorktreeInstance {
        instance_id: 0,
        session_id: None,
        worktree_path: "/tmp/worktree-0".to_string(),
        branch: "main".to_string(),
        agent_name: "Agent 0".to_string(),
        status: "running".to_string(),
        tmux_session_id: "tmux-0".to_string(),
        output: None,
        start_time: Some(chrono::Utc::now().to_rfc3339()),
        end_time: None,
        files_changed: None,
        lines_added: None,
        lines_deleted: None,
        runtime: Some("claude-code".to_string()),
        model: Some("claude-3.5-sonnet".to_string()),
        runtime_label: None,
    }];

    // Create session
    let created = repo.create_session(session.clone()).await.unwrap();

    // Update instance status
    repo.update_instance_status(&created.id, 0, "completed")
        .await
        .unwrap();

    // Verify status was updated
    let fetched = repo
        .get_session("test_workspace_hash", &created.id)
        .await
        .unwrap();
    assert_eq!(fetched.instances[0].status, "completed");
}

#[tokio::test]
async fn test_database_integrity_check() {
    let db_file = NamedTempFile::new().unwrap();
    let repo = SqliteSessionRepository::with_path(db_file.path())
        .await
        .unwrap();

    let is_ok = repo.pool().verify_integrity().await.unwrap();
    assert!(is_ok);
}
