//! Integration tests for AIT42 agent system

use ait42_ait42::prelude::*;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a temporary AIT42 environment for testing
fn setup_test_env() -> (TempDir, AIT42Config) {
    let temp_dir = TempDir::new().unwrap();
    let ait42_root = temp_dir.path().to_path_buf();

    // Create directory structure
    fs::create_dir_all(ait42_root.join(".claude/agents")).unwrap();
    fs::create_dir_all(ait42_root.join("scripts")).unwrap();

    // Create test agents
    create_test_agent(&ait42_root, "test-agent-1", "Backend", "A test backend agent");
    create_test_agent(&ait42_root, "test-agent-2", "Frontend", "A test frontend agent");
    create_test_agent(&ait42_root, "backend-developer", "Backend", "Backend development agent");
    create_test_agent(&ait42_root, "code-reviewer", "QA", "Code review agent");

    // Create mock tmux script
    create_mock_tmux_script(&ait42_root);

    let config = AIT42Config::new(ait42_root);

    (temp_dir, config)
}

fn create_test_agent(root: &PathBuf, name: &str, category: &str, description: &str) {
    let agent_path = root.join(format!(".claude/agents/{}.md", name));
    let content = format!(
        r#"---
name: {}
description: "{}"
tools: Read, Write
model: sonnet
---

<capabilities>
- Test capability 1
- Test capability 2
</capabilities>

<role>
Test agent for {}
</role>
"#,
        name, description, category
    );

    let mut file = fs::File::create(agent_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}

fn create_mock_tmux_script(root: &PathBuf) {
    let script_path = root.join("scripts/tmux-single-agent.sh");
    let content = r#"#!/bin/bash
echo "Session: ait42-$1-12345"
echo "Agent $1 started"
"#;

    let mut file = fs::File::create(&script_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }
}

#[test]
fn test_config_creation() {
    let (_temp, config) = setup_test_env();
    assert!(config.ait42_root.exists());
    assert_eq!(config.max_parallel_agents, 3);
}

#[test]
fn test_config_validation() {
    let (_temp, config) = setup_test_env();
    let result = config.validate();
    assert!(result.is_ok(), "Config validation failed: {:?}", result);
}

#[test]
fn test_agent_registry_loading() {
    let (_temp, config) = setup_test_env();
    let registry = ait42_ait42::registry::AgentRegistry::load_from_directory(&config.agents_dir());
    assert!(registry.is_ok(), "Failed to load registry: {:?}", registry);

    let registry = registry.unwrap();
    assert_eq!(registry.count(), 4, "Should have loaded 4 test agents");
}

#[test]
fn test_agent_registry_get() {
    let (_temp, config) = setup_test_env();
    let registry =
        ait42_ait42::registry::AgentRegistry::load_from_directory(&config.agents_dir()).unwrap();

    let agent = registry.get("test-agent-1");
    assert!(agent.is_some());
    assert_eq!(agent.unwrap().name, "test-agent-1");

    let missing = registry.get("nonexistent");
    assert!(missing.is_none());
}

#[test]
fn test_agent_registry_search() {
    let (_temp, config) = setup_test_env();
    let registry =
        ait42_ait42::registry::AgentRegistry::load_from_directory(&config.agents_dir()).unwrap();

    let results = registry.search("backend");
    assert!(!results.is_empty(), "Should find backend agents");
    assert!(results.iter().any(|a| a.name.contains("backend")));
}

#[test]
fn test_agent_category_classification() {
    let (_temp, config) = setup_test_env();
    let registry =
        ait42_ait42::registry::AgentRegistry::load_from_directory(&config.agents_dir()).unwrap();

    let backend_agents = registry.list_by_category(AgentCategory::Backend);
    assert!(!backend_agents.is_empty());
}

#[tokio::test]
async fn test_tmux_manager_creation() {
    let (_temp, config) = setup_test_env();
    let tmux = ait42_ait42::tmux::TmuxManager::new(&config.ait42_root);

    // Just ensure it's created without panic
    let _ = tmux;
}

#[test]
fn test_coordinator_creation() {
    let (_temp, config) = setup_test_env();
    let coordinator = Coordinator::new(config);
    assert!(coordinator.is_ok(), "Failed to create coordinator: {:?}", coordinator);
}

#[test]
fn test_coordinator_agent_count() {
    let (_temp, config) = setup_test_env();
    let coordinator = Coordinator::new(config).unwrap();
    assert_eq!(coordinator.agent_count(), 4);
}

#[test]
fn test_coordinator_auto_select() {
    let (_temp, config) = setup_test_env();
    let coordinator = Coordinator::new(config).unwrap();

    let agents = coordinator.auto_select_agents("implement backend API");
    assert!(agents.is_ok());

    let agents = agents.unwrap();
    assert!(!agents.is_empty(), "Should select at least one agent");
}

#[test]
fn test_executor_creation() {
    let (_temp, config) = setup_test_env();
    let coordinator = Coordinator::new(config).unwrap();
    let executor = AgentExecutor::new(coordinator);

    // Just ensure it's created
    let _ = executor;
}

#[test]
fn test_execution_mode_variants() {
    let single = ExecutionMode::Single("test-agent".to_string());
    let parallel = ExecutionMode::Parallel(vec!["agent1".to_string(), "agent2".to_string()]);
    let sequential = ExecutionMode::Sequential(vec!["agent1".to_string()]);
    let coordinated = ExecutionMode::Coordinated;

    // Ensure all variants work
    match single {
        ExecutionMode::Single(_) => {}
        _ => panic!("Wrong variant"),
    }

    match parallel {
        ExecutionMode::Parallel(_) => {}
        _ => panic!("Wrong variant"),
    }

    match sequential {
        ExecutionMode::Sequential(_) => {}
        _ => panic!("Wrong variant"),
    }

    match coordinated {
        ExecutionMode::Coordinated => {}
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_command_creation() {
    let cmd = ait42_ait42::commands::AgentCommand::RunAgent {
        agent: "test-agent".to_string(),
        task: "test task".to_string(),
    };

    assert_eq!(cmd.description(), "Run agent: test-agent");
}

#[test]
fn test_command_serialization() {
    let cmd = ait42_ait42::commands::AgentCommand::ListAgents;
    let json = serde_json::to_string(&cmd).unwrap();
    let deserialized: ait42_ait42::commands::AgentCommand = serde_json::from_str(&json).unwrap();

    match deserialized {
        ait42_ait42::commands::AgentCommand::ListAgents => {}
        _ => panic!("Wrong command after deserialization"),
    }
}

#[test]
fn test_editor_bridge_buffer_context() {
    let (_temp, config) = setup_test_env();
    let coordinator = Coordinator::new(config).unwrap();
    let executor = AgentExecutor::new(coordinator);
    let bridge = ait42_ait42::editor_integration::EditorAgentBridge::new(executor);

    let buffer = ait42_ait42::editor_integration::Buffer {
        content: "fn main() {}".to_string(),
        file_path: Some("test.rs".to_string()),
        language: Some("rust".to_string()),
    };

    // Private method, but we can test through public API
    // Just ensure bridge is created with buffer
    let _ = buffer;
    let _ = bridge;
}

#[test]
fn test_stream_manager_creation() {
    let (_temp, config) = setup_test_env();
    let tmux = ait42_ait42::tmux::TmuxManager::new(&config.ait42_root);
    let manager = ait42_ait42::stream::StreamManager::new(tmux);

    // Just ensure it's created
    let _ = manager;
}

#[test]
fn test_error_types() {
    let err1 = AIT42Error::AgentNotFound("test".to_string());
    assert_eq!(err1.to_string(), "Agent not found: test");

    let err2 = AIT42Error::TmuxError("tmux failed".to_string());
    assert_eq!(err2.to_string(), "Tmux error: tmux failed");

    let err3 = AIT42Error::ExecutionFailed("task failed".to_string());
    assert_eq!(err3.to_string(), "Execution failed: task failed");
}

#[test]
fn test_session_status() {
    let status1 = ait42_ait42::tmux::SessionStatus::Running;
    let status2 = ait42_ait42::tmux::SessionStatus::Completed;
    let status3 = ait42_ait42::tmux::SessionStatus::Failed("error".to_string());

    assert_eq!(status1, ait42_ait42::tmux::SessionStatus::Running);
    assert_eq!(status2, ait42_ait42::tmux::SessionStatus::Completed);

    match status3 {
        ait42_ait42::tmux::SessionStatus::Failed(msg) => assert_eq!(msg, "error"),
        _ => panic!("Wrong status"),
    }
}

#[test]
fn test_prelude_imports() {
    use ait42_ait42::prelude::*;

    // Ensure all prelude items are accessible
    let _ = std::any::type_name::<AIT42Config>();
    let _ = std::any::type_name::<Coordinator>();
    let _ = std::any::type_name::<AgentExecutor>();
    let _ = std::any::type_name::<AIT42Error>();
}
