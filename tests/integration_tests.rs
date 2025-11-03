//! Integration Tests for AIT42 Editor
//!
//! These tests verify the integration between different crates.

use ait42_core::{Editor, EditorConfig};

#[tokio::test]
async fn test_editor_initialization() {
    let config = EditorConfig::default();
    let editor = Editor::new(config);
    assert!(editor.is_ok(), "Editor should initialize successfully");
}

#[tokio::test]
async fn test_config_loading() {
    let config = ait42_config::Config::load_default().await;
    assert!(config.is_ok(), "Default config should load");
}

#[tokio::test]
async fn test_file_system_read() {
    let result = ait42_fs::read_dir(".").await;
    assert!(result.is_ok(), "Should read current directory");
}

#[tokio::test]
async fn test_agent_manager_creation() {
    use std::path::PathBuf;
    let manager = ait42_ait42::AgentManager::new(PathBuf::from("../.claude/agents"));
    // Basic smoke test - just ensure creation doesn't panic
    assert!(true);
}
