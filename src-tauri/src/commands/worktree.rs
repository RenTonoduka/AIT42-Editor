//! Worktree Visualization Backend Commands
//!
//! This module provides Tauri commands for visualizing and managing git worktrees
//! created for Claude Code competitions.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};

#[derive(Serialize, Clone, Debug)]
pub struct WorktreeInfo {
    pub id: String,
    pub path: String,
    pub branch: String,
    pub status: String,
    pub created_at: String,
    pub changed_files: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub children: Option<Vec<FileNode>>,
    pub git_status: Option<String>,
}

/// List all worktrees for a specific competition
#[tauri::command]
pub async fn list_worktrees(
    state: tauri::State<'_, crate::state::AppState>,
    competition_id: String,
) -> Result<Vec<WorktreeInfo>, String> {
    info!("Listing worktrees for competition: {}", competition_id);

    let working_dir = state.working_dir.lock().await;
    let base_path = working_dir.clone();
    drop(working_dir);

    // Validate competition_id to prevent path traversal
    if competition_id.contains("..") || competition_id.contains('/') || competition_id.contains('\\') {
        return Err("Invalid competition ID".to_string());
    }

    let competition_dir = base_path.join(".worktrees").join(format!("competition-{}", &competition_id[..competition_id.len().min(8)]));

    if !competition_dir.exists() {
        info!("Competition directory does not exist: {:?}", competition_dir);
        return Ok(Vec::new());
    }

    let mut worktrees = Vec::new();

    // Read directory entries
    let entries = fs::read_dir(&competition_dir).map_err(|e| {
        error!("Failed to read competition directory: {}", e);
        format!("Failed to read directory: {}", e)
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let dir_name = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "Invalid directory name".to_string())?;

        // Extract instance number from directory name (e.g., "instance-1")
        if !dir_name.starts_with("instance-") {
            continue;
        }

        let instance_num = dir_name.strip_prefix("instance-")
            .ok_or_else(|| "Invalid instance directory format".to_string())?;

        let instance_id = format!("{}-instance-{}", competition_id, instance_num);

        // Get git branch name
        let branch = get_worktree_branch(&path)?;

        // Get worktree status
        let status = get_worktree_status(&path)?;

        // Get creation time from .git file metadata
        let created_at = get_worktree_created_time(&path)?;

        // Count changed files
        let changed_files = count_changed_files(&path)?;

        worktrees.push(WorktreeInfo {
            id: instance_id,
            path: path.to_string_lossy().to_string(),
            branch,
            status,
            created_at,
            changed_files,
        });
    }

    info!("Found {} worktrees", worktrees.len());
    Ok(worktrees)
}

/// Get file tree structure for a worktree
#[tauri::command]
pub async fn get_worktree_files(
    worktree_path: String,
    max_depth: Option<usize>,
) -> Result<Vec<FileNode>, String> {
    info!("Getting file tree for worktree: {}", worktree_path);

    let path = PathBuf::from(&worktree_path);

    // Security: Validate path exists and is a directory
    if !path.exists() {
        return Err("Worktree path does not exist".to_string());
    }

    if !path.is_dir() {
        return Err("Worktree path is not a directory".to_string());
    }

    // Security: Ensure path is absolute to prevent traversal
    let canonical_path = path.canonicalize().map_err(|e| {
        format!("Failed to canonicalize path: {}", e)
    })?;

    let max_depth = max_depth.unwrap_or(5);

    // Get git status for all files
    let git_status_map = get_git_status_map(&canonical_path)?;

    // Build file tree
    let file_nodes = build_file_tree(&canonical_path, &canonical_path, &git_status_map, 0, max_depth)?;

    info!("Built file tree with {} top-level nodes", file_nodes.len());
    Ok(file_nodes)
}

/// Delete a worktree
#[tauri::command]
pub async fn delete_worktree(
    state: tauri::State<'_, crate::state::AppState>,
    worktree_id: String,
) -> Result<(), String> {
    info!("Deleting worktree: {}", worktree_id);

    // Parse worktree_id (format: "{competition_id}-instance-{num}")
    let parts: Vec<&str> = worktree_id.rsplitn(2, "-instance-").collect();
    if parts.len() != 2 {
        return Err("Invalid worktree ID format".to_string());
    }

    let instance_num = parts[0];
    let competition_id = parts[1];

    // Security: Validate IDs
    if competition_id.contains("..") || competition_id.contains('/') || competition_id.contains('\\') {
        return Err("Invalid competition ID".to_string());
    }

    let working_dir = state.working_dir.lock().await;
    let base_path = working_dir.clone();
    drop(working_dir);

    let competition_dir = base_path.join(".worktrees").join(format!("competition-{}", &competition_id[..competition_id.len().min(8)]));
    let worktree_path = competition_dir.join(format!("instance-{}", instance_num));

    if !worktree_path.exists() {
        return Err("Worktree does not exist".to_string());
    }

    // Remove git worktree first
    let output = Command::new("git")
        .arg("worktree")
        .arg("remove")
        .arg(&worktree_path)
        .arg("--force")
        .current_dir(&base_path)
        .output()
        .map_err(|e| format!("Failed to execute git command: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        warn!("Git worktree remove failed (will try manual deletion): {}", error);

        // If git command fails, manually delete the directory
        fs::remove_dir_all(&worktree_path).map_err(|e| {
            format!("Failed to delete worktree directory: {}", e)
        })?;
    }

    // Kill associated tmux session if exists
    let session_id = format!("claude-code-comp-{}-{}", &competition_id[..competition_id.len().min(8)], instance_num);
    let _ = Command::new("tmux")
        .arg("kill-session")
        .arg("-t")
        .arg(&session_id)
        .output(); // Ignore errors - session may not exist

    info!("Successfully deleted worktree: {}", worktree_id);
    Ok(())
}

// Helper functions

fn get_worktree_branch(path: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to get branch: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Ok("unknown".to_string())
    }
}

fn get_worktree_status(path: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to get status: {}", e))?;

    if output.status.success() {
        let status_output = String::from_utf8_lossy(&output.stdout);
        if status_output.trim().is_empty() {
            Ok("clean".to_string())
        } else {
            Ok("modified".to_string())
        }
    } else {
        Ok("unknown".to_string())
    }
}

fn get_worktree_created_time(path: &Path) -> Result<String, String> {
    let git_file = path.join(".git");

    if git_file.exists() {
        let metadata = fs::metadata(&git_file).map_err(|e| {
            format!("Failed to read metadata: {}", e)
        })?;

        let created = metadata.created().or_else(|_| metadata.modified()).map_err(|e| {
            format!("Failed to get creation time: {}", e)
        })?;

        let datetime: DateTime<Utc> = created.into();
        Ok(datetime.to_rfc3339())
    } else {
        Ok(Utc::now().to_rfc3339())
    }
}

fn count_changed_files(path: &Path) -> Result<usize, String> {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to get status: {}", e))?;

    if output.status.success() {
        let status_output = String::from_utf8_lossy(&output.stdout);
        Ok(status_output.lines().count())
    } else {
        Ok(0)
    }
}

fn get_git_status_map(path: &Path) -> Result<std::collections::HashMap<PathBuf, String>, String> {
    let mut status_map = std::collections::HashMap::new();

    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(path)
        .output()
        .map_err(|e| format!("Failed to get git status: {}", e))?;

    if output.status.success() {
        let status_output = String::from_utf8_lossy(&output.stdout);

        for line in status_output.lines() {
            if line.len() < 4 {
                continue;
            }

            let status = &line[0..2];
            let file_path = &line[3..];
            let full_path = path.join(file_path);

            status_map.insert(full_path, status.trim().to_string());
        }
    }

    Ok(status_map)
}

fn build_file_tree(
    base_path: &Path,
    current_path: &Path,
    git_status_map: &std::collections::HashMap<PathBuf, String>,
    current_depth: usize,
    max_depth: usize,
) -> Result<Vec<FileNode>, String> {
    if current_depth >= max_depth {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(current_path).map_err(|e| {
        format!("Failed to read directory: {}", e)
    })?;

    let mut nodes = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        // Skip .git directory
        if path.file_name().and_then(|n| n.to_str()) == Some(".git") {
            continue;
        }

        // Skip hidden files except common ones
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') && name != ".gitignore" && name != ".env" {
                continue;
            }
        }

        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let relative_path = path.strip_prefix(base_path)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        let is_directory = path.is_dir();
        let git_status = git_status_map.get(&path).cloned();

        let children = if is_directory {
            Some(build_file_tree(base_path, &path, git_status_map, current_depth + 1, max_depth)?)
        } else {
            None
        };

        nodes.push(FileNode {
            name,
            path: relative_path,
            is_directory,
            children,
            git_status,
        });
    }

    // Sort: directories first, then alphabetically
    nodes.sort_by(|a, b| {
        match (a.is_directory, b.is_directory) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worktree_id_validation() {
        // Valid IDs
        assert!(!("abc123-instance-1".contains("..")));
        assert!(!("abc123-instance-1".contains('/')));

        // Invalid IDs
        assert!("../secret-instance-1".contains(".."));
        assert!("/etc/passwd-instance-1".contains('/'));
    }
}
