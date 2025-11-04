/**
 * AIT42 Agent Commands
 *
 * Tauri commands for executing AIT42 AI agents with Tmux support
 */

use serde::{Deserialize, Serialize};
use tauri::State;
use std::process::Command;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::state::AppState;

/**
 * Agent information
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub description: String,
    pub category: String,
    pub tools: Vec<String>,
}

/**
 * Agent execution request
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionRequest {
    pub agent_name: String,
    pub task: String,
    pub context: Option<String>,
}

/**
 * Agent execution response
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionResponse {
    pub execution_id: String,
    pub agent_name: String,
    pub status: String, // "started", "running", "completed", "failed"
    pub output: Option<String>,
    pub error: Option<String>,
}

/**
 * Parallel execution request
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelExecutionRequest {
    pub agents: Vec<String>,
    pub task: String,
    pub context: Option<String>,
}

/**
 * Claude Code Competition Request
 *
 * Executes multiple Claude Code instances in parallel worktrees
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeCompetitionRequest {
    pub task: String,
    pub instance_count: usize,  // 2-10 instances
    pub model: String,  // "sonnet", "haiku", "opus"
    pub timeout_seconds: u64,  // Default: 300
    pub preserve_worktrees: bool,  // Keep worktrees after completion
}

/**
 * Claude Code Competition Result
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeCompetitionResult {
    pub competition_id: String,
    pub instances: Vec<ClaudeCodeInstanceResult>,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub status: String,  // "running", "completed", "failed"
}

/**
 * Individual Claude Code Instance Result
 */
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeInstanceResult {
    pub instance_id: String,
    pub instance_number: usize,
    pub worktree_path: String,
    pub tmux_session_id: String,
    pub status: String,  // "starting", "running", "completed", "failed", "timeout"
    pub output: String,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub started_at: String,
    pub completed_at: Option<String>,
}

/**
 * List all available agents
 */
#[tauri::command]
pub async fn list_agents(_state: State<'_, AppState>) -> Result<Vec<AgentInfo>, String> {
    // TODO: Integrate with ait42-ait42::registry
    // For now, return a sample list of core agents
    Ok(vec![
        AgentInfo {
            name: "code-reviewer".to_string(),
            description: "Automated code review specialist".to_string(),
            category: "quality".to_string(),
            tools: vec!["Read".to_string(), "Grep".to_string(), "Glob".to_string()],
        },
        AgentInfo {
            name: "test-generator".to_string(),
            description: "Automated test generation specialist".to_string(),
            category: "testing".to_string(),
            tools: vec!["Read".to_string(), "Write".to_string(), "Edit".to_string(), "Bash".to_string()],
        },
        AgentInfo {
            name: "refactor-specialist".to_string(),
            description: "Code refactoring and technical debt reduction specialist".to_string(),
            category: "quality".to_string(),
            tools: vec!["Read".to_string(), "Write".to_string(), "Edit".to_string(), "Grep".to_string()],
        },
        AgentInfo {
            name: "security-scanner".to_string(),
            description: "Security scanning and vulnerability management specialist".to_string(),
            category: "security".to_string(),
            tools: vec!["Read".to_string(), "Grep".to_string(), "Glob".to_string(), "Bash".to_string()],
        },
        AgentInfo {
            name: "bug-fixer".to_string(),
            description: "Automated bug fixing specialist".to_string(),
            category: "quality".to_string(),
            tools: vec!["Read".to_string(), "Write".to_string(), "Edit".to_string(), "Bash".to_string()],
        },
    ])
}

/**
 * Get information about a specific agent
 */
#[tauri::command]
pub async fn get_agent_info(
    _state: State<'_, AppState>,
    agent_name: String,
) -> Result<AgentInfo, String> {
    // TODO: Integrate with ait42-ait42::registry
    // For now, return sample data
    match agent_name.as_str() {
        "code-reviewer" => Ok(AgentInfo {
            name: "code-reviewer".to_string(),
            description: "Automated code review specialist. Proactively reviews code for security, quality, best practices, and generates quality scores (0-100).".to_string(),
            category: "quality".to_string(),
            tools: vec!["Read".to_string(), "Grep".to_string(), "Glob".to_string()],
        }),
        "test-generator" => Ok(AgentInfo {
            name: "test-generator".to_string(),
            description: "Automated test generation specialist. Invoked for unit tests, integration tests, E2E tests, and test fixture creation.".to_string(),
            category: "testing".to_string(),
            tools: vec!["Read".to_string(), "Write".to_string(), "Edit".to_string(), "Bash".to_string()],
        }),
        _ => Err(format!("Agent not found: {}", agent_name)),
    }
}

/**
 * Execute a single agent
 */
#[tauri::command]
pub async fn execute_agent(
    _state: State<'_, AppState>,
    request: AgentExecutionRequest,
) -> Result<AgentExecutionResponse, String> {
    // TODO: Integrate with ait42-ait42::executor
    // For now, return a mock response
    let execution_id = uuid::Uuid::new_v4().to_string();

    Ok(AgentExecutionResponse {
        execution_id,
        agent_name: request.agent_name,
        status: "started".to_string(),
        output: Some("Agent execution started...".to_string()),
        error: None,
    })
}

/**
 * Execute multiple agents in parallel
 */
#[tauri::command]
pub async fn execute_parallel(
    _state: State<'_, AppState>,
    request: ParallelExecutionRequest,
) -> Result<Vec<AgentExecutionResponse>, String> {
    // TODO: Integrate with ait42-ait42::executor for parallel execution
    // For now, return mock responses
    let responses: Vec<AgentExecutionResponse> = request
        .agents
        .iter()
        .map(|agent_name| {
            let execution_id = uuid::Uuid::new_v4().to_string();
            AgentExecutionResponse {
                execution_id,
                agent_name: agent_name.clone(),
                status: "started".to_string(),
                output: Some(format!("Parallel execution started for {}", agent_name)),
                error: None,
            }
        })
        .collect();

    Ok(responses)
}

/**
 * Get output from a running or completed agent execution
 */
#[tauri::command]
pub async fn get_agent_output(
    _state: State<'_, AppState>,
    execution_id: String,
) -> Result<AgentExecutionResponse, String> {
    // TODO: Integrate with ait42-ait42::stream for real-time output
    // For now, return a mock response
    Ok(AgentExecutionResponse {
        execution_id,
        agent_name: "code-reviewer".to_string(),
        status: "completed".to_string(),
        output: Some("Code review completed successfully.".to_string()),
        error: None,
    })
}

/**
 * Cancel a running agent execution
 */
#[tauri::command]
pub async fn cancel_agent_execution(
    _state: State<'_, AppState>,
    execution_id: String,
) -> Result<(), String> {
    // TODO: Integrate with ait42-ait42::executor for cancellation
    // For now, just return success
    tracing::info!("Cancelled agent execution: {}", execution_id);
    Ok(())
}

//
// ============================================================
// Tmux Session Management
// ============================================================
//

/// Tmux session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxSession {
    pub session_id: String,
    pub agent_name: String,
    pub status: String, // "running", "completed", "failed"
    pub created_at: String,
}

/// Tmux execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxExecutionRequest {
    pub agent_name: String,
    pub task: String,
    pub context: Option<String>,
}

/// Create a tmux session for agent execution
#[tauri::command]
pub async fn create_tmux_session(
    _state: State<'_, AppState>,
    request: TmuxExecutionRequest,
) -> Result<TmuxSession, String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    let session_id = format!("ait42-{}-{}", request.agent_name, timestamp);

    // Check if tmux is available
    let tmux_check = Command::new("tmux")
        .arg("-V")
        .output();

    match tmux_check {
        Err(_) => return Err("Tmux is not installed or not in PATH".to_string()),
        Ok(output) if !output.status.success() => {
            return Err("Tmux is not available".to_string())
        }
        _ => {}
    }

    // Create new tmux session
    let mut cmd = Command::new("tmux");
    cmd.arg("new-session")
        .arg("-d") // Detached
        .arg("-s")
        .arg(&session_id)
        .arg("-c")
        .arg(std::env::current_dir().map_err(|e| e.to_string())?)
        .arg("echo")
        .arg(format!("🚀 Starting agent: {} for task: {}", request.agent_name, request.task));

    let output = cmd.output().map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(format!(
            "Failed to create tmux session: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    tracing::info!("Created tmux session: {}", session_id);

    Ok(TmuxSession {
        session_id,
        agent_name: request.agent_name,
        status: "running".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    })
}

/// List all AIT42 tmux sessions
#[tauri::command]
pub async fn list_tmux_sessions(_state: State<'_, AppState>) -> Result<Vec<TmuxSession>, String> {
    let output = Command::new("tmux")
        .arg("list-sessions")
        .arg("-F")
        .arg("#{session_name}")
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let sessions_output = String::from_utf8_lossy(&output.stdout);
    let sessions: Vec<TmuxSession> = sessions_output
        .lines()
        .filter(|line| line.starts_with("ait42-"))
        .map(|session_name| {
            let parts: Vec<&str> = session_name.split('-').collect();
            let agent_name = if parts.len() >= 3 {
                parts[1..parts.len()-1].join("-")
            } else {
                "unknown".to_string()
            };

            TmuxSession {
                session_id: session_name.to_string(),
                agent_name,
                status: "running".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            }
        })
        .collect();

    Ok(sessions)
}

/// Capture output from a tmux session
#[tauri::command]
pub async fn capture_tmux_output(
    _state: State<'_, AppState>,
    session_id: String,
) -> Result<String, String> {
    let output = Command::new("tmux")
        .arg("capture-pane")
        .arg("-p")
        .arg("-t")
        .arg(&session_id)
        .arg("-S")
        .arg("-")
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(format!(
            "Failed to capture tmux output: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Send keys/command to a tmux session
#[tauri::command]
pub async fn send_tmux_keys(
    _state: State<'_, AppState>,
    session_id: String,
    keys: String,
) -> Result<(), String> {
    let output = Command::new("tmux")
        .arg("send-keys")
        .arg("-t")
        .arg(&session_id)
        .arg(&keys)
        .arg("C-m") // Enter key
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(format!(
            "Failed to send keys: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
}

/// Kill a tmux session
#[tauri::command]
pub async fn kill_tmux_session(
    _state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    let output = Command::new("tmux")
        .arg("kill-session")
        .arg("-t")
        .arg(&session_id)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(format!(
            "Failed to kill session: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    tracing::info!("Killed tmux session: {}", session_id);
    Ok(())
}

//
// ============================================================
// Claude Code Competition Mode
// ============================================================
//

/// Execute Claude Code Competition
///
/// Creates multiple git worktrees and launches Claude Code instances in parallel
#[tauri::command]
pub async fn execute_claude_code_competition(
    state: State<'_, AppState>,
    request: ClaudeCodeCompetitionRequest,
) -> Result<ClaudeCodeCompetitionResult, String> {
    // Validation
    if request.instance_count < 2 || request.instance_count > 10 {
        return Err("Instance count must be between 2 and 10".to_string());
    }

    if request.task.trim().is_empty() {
        return Err("Task cannot be empty".to_string());
    }

    let valid_models = ["sonnet", "haiku", "opus"];
    if !valid_models.contains(&request.model.as_str()) {
        return Err(format!("Invalid model. Must be one of: {:?}", valid_models));
    }

    let competition_id = uuid::Uuid::new_v4().to_string();
    let started_at = chrono::Utc::now();

    tracing::info!(
        "Starting Claude Code competition: {} instances, model: {}, task: {}",
        request.instance_count,
        request.model,
        request.task.chars().take(50).collect::<String>()
    );

    let working_dir = state.working_dir.lock().await;
    let base_path = working_dir.clone();
    drop(working_dir); // Release lock

    // Create competition directory
    let competition_dir = format!("{}/.worktrees/competition-{}", base_path.display(), &competition_id[..8]);
    std::fs::create_dir_all(&competition_dir).map_err(|e| e.to_string())?;

    let mut instances = Vec::new();

    // Create worktrees and launch instances
    for i in 1..=request.instance_count {
        let instance_id = format!("{}-instance-{}", competition_id, i);
        let worktree_path = format!("{}/instance-{}", competition_dir, i);
        let branch_name = format!("competition-{}-{}", &competition_id[..8], i);

        // Create git worktree
        tracing::info!("Creating worktree {} at {}", i, worktree_path);

        let mut cmd = Command::new("git");
        cmd.arg("worktree")
            .arg("add")
            .arg("-b")
            .arg(&branch_name)
            .arg(&worktree_path)
            .current_dir(&base_path);

        let output = cmd.output().map_err(|e| {
            format!("Failed to create worktree {}: {}", i, e)
        })?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            tracing::error!("Worktree creation failed: {}", error);
            return Err(format!("Failed to create worktree {}: {}", i, error));
        }

        // Create tmux session for this instance
        let session_id = format!("claude-code-comp-{}-{}", &competition_id[..8], i);

        let tmux_output = Command::new("tmux")
            .arg("new-session")
            .arg("-d")
            .arg("-s")
            .arg(&session_id)
            .arg("-c")
            .arg(&worktree_path)
            .output()
            .map_err(|e| format!("Failed to create tmux session: {}", e))?;

        if !tmux_output.status.success() {
            let error = String::from_utf8_lossy(&tmux_output.stderr);
            return Err(format!("Failed to create tmux session {}: {}", i, error));
        }

        // Send Claude Code command to tmux session
        let claude_cmd = format!(
            "echo '{}' | claude --model {} code",
            request.task.replace("'", "'\\''"),
            request.model
        );

        let send_output = Command::new("tmux")
            .arg("send-keys")
            .arg("-t")
            .arg(&session_id)
            .arg(&claude_cmd)
            .arg("C-m")
            .output()
            .map_err(|e| format!("Failed to send command: {}", e))?;

        if !send_output.status.success() {
            let error = String::from_utf8_lossy(&send_output.stderr);
            return Err(format!("Failed to send command to instance {}: {}", i, error));
        }

        tracing::info!("Launched Claude Code instance {} in session {}", i, session_id);

        instances.push(ClaudeCodeInstanceResult {
            instance_id,
            instance_number: i,
            worktree_path,
            tmux_session_id: session_id,
            status: "running".to_string(),
            output: String::new(),
            error: None,
            execution_time_ms: 0,
            started_at: started_at.to_rfc3339(),
            completed_at: None,
        });
    }

    Ok(ClaudeCodeCompetitionResult {
        competition_id,
        instances,
        started_at: started_at.to_rfc3339(),
        completed_at: None,
        status: "running".to_string(),
    })
}

/// Get competition status and results
#[tauri::command]
pub async fn get_competition_status(
    _state: State<'_, AppState>,
    competition_id: String,
) -> Result<ClaudeCodeCompetitionResult, String> {
    // TODO: Implement status tracking
    // For now, return a placeholder
    Err("Status tracking not yet implemented".to_string())
}

/// Cancel a running competition
#[tauri::command]
pub async fn cancel_competition(
    state: State<'_, AppState>,
    competition_id: String,
    cleanup_worktrees: bool,
) -> Result<(), String> {
    tracing::info!("Cancelling competition: {}", competition_id);

    // Kill all tmux sessions for this competition
    let session_pattern = format!("claude-code-comp-{}", &competition_id[..8]);

    let list_output = Command::new("tmux")
        .arg("list-sessions")
        .arg("-F")
        .arg("#{session_name}")
        .output()
        .map_err(|e| e.to_string())?;

    if list_output.status.success() {
        let sessions = String::from_utf8_lossy(&list_output.stdout);
        for session in sessions.lines() {
            if session.contains(&session_pattern) {
                let _ = Command::new("tmux")
                    .arg("kill-session")
                    .arg("-t")
                    .arg(session)
                    .output();

                tracing::info!("Killed session: {}", session);
            }
        }
    }

    // Cleanup worktrees if requested
    if cleanup_worktrees {
        let working_dir = state.working_dir.lock().await;
        let competition_dir = format!("{}/.worktrees/competition-{}", working_dir.display(), &competition_id[..8]);
        drop(working_dir);

        // Remove all worktrees
        if let Ok(entries) = std::fs::read_dir(&competition_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let path_str = path.to_string_lossy().to_string();

                    // Remove git worktree
                    let _ = Command::new("git")
                        .arg("worktree")
                        .arg("remove")
                        .arg("--force")
                        .arg(&path_str)
                        .output();

                    tracing::info!("Removed worktree: {}", path_str);
                }
            }
        }

        // Remove competition directory
        let _ = std::fs::remove_dir_all(&competition_dir);
        tracing::info!("Cleaned up competition directory: {}", competition_dir);
    }

    Ok(())
}
