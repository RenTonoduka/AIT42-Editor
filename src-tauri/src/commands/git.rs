/**
 * Git Commands
 *
 * Tauri commands for Git operations using git2-rs
 */

use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

use crate::state::AppState;

/**
 * Git file status
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFileStatus {
    pub path: String,
    pub status: String, // "modified", "added", "deleted", "untracked", "renamed"
}

/**
 * Git status response
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub files: Vec<GitFileStatus>,
}

/**
 * Git commit info
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub sha: String,
    pub author: String,
    pub email: String,
    pub message: String,
    pub timestamp: i64,
}

/**
 * Get Git status for current repository
 */
#[tauri::command]
pub async fn git_status(_state: State<'_, AppState>) -> Result<GitStatus, String> {
    // TODO: Implement actual git2-rs integration
    // For now, return a placeholder
    Ok(GitStatus {
        branch: String::from("main"),
        ahead: 0,
        behind: 0,
        files: vec![],
    })
}

/**
 * Stage files for commit
 */
#[tauri::command]
pub async fn git_add(
    files: Vec<String>,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // TODO: Implement git add
    println!("[Git] Adding files: {:?}", files);
    Ok(())
}

/**
 * Unstage files
 */
#[tauri::command]
pub async fn git_reset(
    files: Vec<String>,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // TODO: Implement git reset
    println!("[Git] Resetting files: {:?}", files);
    Ok(())
}

/**
 * Create a commit
 */
#[tauri::command]
pub async fn git_commit(
    message: String,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    // TODO: Implement git commit
    println!("[Git] Committing with message: {}", message);
    Ok(String::from("abc1234")) // Return commit SHA
}

/**
 * Push to remote
 */
#[tauri::command]
pub async fn git_push(
    remote: Option<String>,
    branch: Option<String>,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    let remote = remote.unwrap_or_else(|| String::from("origin"));
    let branch = branch.unwrap_or_else(|| String::from("main"));

    // TODO: Implement git push
    println!("[Git] Pushing to {}/{}", remote, branch);
    Ok(())
}

/**
 * Pull from remote
 */
#[tauri::command]
pub async fn git_pull(
    remote: Option<String>,
    branch: Option<String>,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    let remote = remote.unwrap_or_else(|| String::from("origin"));
    let branch = branch.unwrap_or_else(|| String::from("main"));

    // TODO: Implement git pull
    println!("[Git] Pulling from {}/{}", remote, branch);
    Ok(())
}

/**
 * Get commit history
 */
#[tauri::command]
pub async fn git_log(
    limit: Option<usize>,
    _state: State<'_, AppState>,
) -> Result<Vec<GitCommit>, String> {
    let _limit = limit.unwrap_or(50);

    // TODO: Implement git log
    println!("[Git] Getting commit history");
    Ok(vec![])
}

/**
 * Get list of branches
 */
#[tauri::command]
pub async fn git_branches(_state: State<'_, AppState>) -> Result<Vec<String>, String> {
    // TODO: Implement git branch list
    println!("[Git] Getting branches");
    Ok(vec![String::from("main")])
}

/**
 * Checkout branch
 */
#[tauri::command]
pub async fn git_checkout(
    branch: String,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // TODO: Implement git checkout
    println!("[Git] Checking out branch: {}", branch);
    Ok(())
}

/**
 * Create new branch
 */
#[tauri::command]
pub async fn git_create_branch(
    name: String,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // TODO: Implement git branch creation
    println!("[Git] Creating branch: {}", name);
    Ok(())
}

//
// ============================================================
// Git Worktree Management
// ============================================================
//

/// Worktree information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: String,
    pub commit: String,
    pub is_bare: bool,
    pub is_detached: bool,
}

/// List all worktrees
#[tauri::command]
pub async fn git_list_worktrees(state: State<'_, AppState>) -> Result<Vec<WorktreeInfo>, String> {
    let working_dir = state.working_dir.lock().await;

    let output = Command::new("git")
        .arg("worktree")
        .arg("list")
        .arg("--porcelain")
        .current_dir(&*working_dir)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = Vec::new();
    let mut current_worktree = None;
    let mut path = String::new();
    let mut commit = String::new();
    let mut branch = String::new();
    let mut is_bare = false;
    let mut is_detached = false;

    for line in stdout.lines() {
        if line.starts_with("worktree ") {
            // Save previous worktree if exists
            if !path.is_empty() {
                worktrees.push(WorktreeInfo {
                    path: path.clone(),
                    branch: branch.clone(),
                    commit: commit.clone(),
                    is_bare,
                    is_detached,
                });
            }

            // Start new worktree
            path = line.strip_prefix("worktree ").unwrap_or("").to_string();
            commit = String::new();
            branch = String::new();
            is_bare = false;
            is_detached = false;
        } else if line.starts_with("HEAD ") {
            commit = line.strip_prefix("HEAD ").unwrap_or("").to_string();
        } else if line.starts_with("branch ") {
            branch = line.strip_prefix("branch ").unwrap_or("").to_string();
            // Remove refs/heads/ prefix
            if branch.starts_with("refs/heads/") {
                branch = branch.strip_prefix("refs/heads/").unwrap_or(&branch).to_string();
            }
        } else if line == "bare" {
            is_bare = true;
        } else if line == "detached" {
            is_detached = true;
        }
    }

    // Save last worktree
    if !path.is_empty() {
        worktrees.push(WorktreeInfo {
            path,
            branch,
            commit,
            is_bare,
            is_detached,
        });
    }

    tracing::info!("Listed {} worktrees", worktrees.len());
    Ok(worktrees)
}

/// Create a new worktree
#[tauri::command]
pub async fn git_create_worktree(
    state: State<'_, AppState>,
    path: String,
    branch: String,
    create_branch: bool,
) -> Result<WorktreeInfo, String> {
    let working_dir = state.working_dir.lock().await;

    let mut cmd = Command::new("git");
    cmd.arg("worktree")
        .arg("add");

    if create_branch {
        cmd.arg("-b").arg(&branch);
    }

    cmd.arg(&path);

    if !create_branch {
        cmd.arg(&branch);
    }

    cmd.current_dir(&*working_dir);

    let output = cmd.output().map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    // Get commit hash of the new worktree
    let commit_output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    let commit = String::from_utf8_lossy(&commit_output.stdout)
        .trim()
        .to_string();

    tracing::info!("Created worktree at {} for branch {}", path, branch);

    Ok(WorktreeInfo {
        path,
        branch,
        commit,
        is_bare: false,
        is_detached: false,
    })
}

/// Remove a worktree
#[tauri::command]
pub async fn git_remove_worktree(
    state: State<'_, AppState>,
    path: String,
    force: bool,
) -> Result<(), String> {
    let working_dir = state.working_dir.lock().await;

    let mut cmd = Command::new("git");
    cmd.arg("worktree")
        .arg("remove");

    if force {
        cmd.arg("--force");
    }

    cmd.arg(&path)
        .current_dir(&*working_dir);

    let output = cmd.output().map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    tracing::info!("Removed worktree at {}", path);
    Ok(())
}

/// Prune stale worktree administrative data
#[tauri::command]
pub async fn git_prune_worktrees(state: State<'_, AppState>) -> Result<(), String> {
    let working_dir = state.working_dir.lock().await;

    let output = Command::new("git")
        .arg("worktree")
        .arg("prune")
        .current_dir(&*working_dir)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    tracing::info!("Pruned stale worktrees");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_file_status_creation() {
        let status = GitFileStatus {
            path: String::from("src/main.rs"),
            status: String::from("modified"),
        };

        assert_eq!(status.path, "src/main.rs");
        assert_eq!(status.status, "modified");
    }

    #[test]
    fn test_git_status_creation() {
        let status = GitStatus {
            branch: String::from("main"),
            ahead: 1,
            behind: 0,
            files: vec![],
        };

        assert_eq!(status.branch, "main");
        assert_eq!(status.ahead, 1);
        assert_eq!(status.behind, 0);
    }
}
