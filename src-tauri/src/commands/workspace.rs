/**
 * Workspace management commands
 * 
 * Allows users to select and manage their project workspace directory
 */

use tauri::{State, Manager};
use crate::state::AppState;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub path: String,
    pub is_git_repo: bool,
}

/// Open a folder selection dialog and set it as the working directory
#[tauri::command]
pub async fn select_workspace(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<WorkspaceInfo, String> {
    use tauri::api::dialog::blocking::FileDialogBuilder;

    // Show folder selection dialog
    let selected = FileDialogBuilder::new()
        .set_title("プロジェクトフォルダを選択")
        .pick_folder();

    if let Some(path) = selected {
        // Check if it's a git repository
        let is_git_repo = path.join(".git").exists();

        if !is_git_repo {
            return Err(format!(
                "選択されたフォルダはGitリポジトリではありません: {}",
                path.display()
            ));
        }

        // Update working directory
        let mut working_dir = state.working_dir.lock().await;
        *working_dir = path.clone();
        drop(working_dir);

        tracing::info!("📁 Workspace set to: {}", path.display());

        // Save to config
        save_workspace_config(&path)?;

        Ok(WorkspaceInfo {
            path: path.to_string_lossy().to_string(),
            is_git_repo,
        })
    } else {
        Err("フォルダが選択されませんでした".to_string())
    }
}

/// Get current workspace path
#[tauri::command]
pub async fn get_workspace(state: State<'_, AppState>) -> Result<WorkspaceInfo, String> {
    let working_dir = state.working_dir.lock().await;
    let path = working_dir.clone();
    drop(working_dir);

    let is_git_repo = path.join(".git").exists();

    Ok(WorkspaceInfo {
        path: path.to_string_lossy().to_string(),
        is_git_repo,
    })
}

/// Save workspace path to config file
fn save_workspace_config(path: &PathBuf) -> Result<(), String> {
    use std::fs;

    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?
        .join("ait42-editor");

    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let config_file = config_dir.join("workspace.json");

    let config = serde_json::json!({
        "workspace_path": path.to_string_lossy().to_string(),
    });

    fs::write(&config_file, serde_json::to_string_pretty(&config).unwrap())
        .map_err(|e| format!("Failed to save workspace config: {}", e))?;

    Ok(())
}

/// Load workspace path from config file
pub fn load_workspace_config() -> Option<PathBuf> {
    use std::fs;

    let config_dir = dirs::config_dir()?.join("ait42-editor");
    let config_file = config_dir.join("workspace.json");

    if !config_file.exists() {
        return None;
    }

    let contents = fs::read_to_string(&config_file).ok()?;
    let config: serde_json::Value = serde_json::from_str(&contents).ok()?;

    let path_str = config.get("workspace_path")?.as_str()?;
    let path = PathBuf::from(path_str);

    // Verify it still exists and is a git repo
    if path.exists() && path.join(".git").exists() {
        Some(path)
    } else {
        None
    }
}
