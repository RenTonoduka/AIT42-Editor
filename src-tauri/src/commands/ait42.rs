/**
 * AIT42 Agent Commands
 *
 * Tauri commands for executing AIT42 AI agents with Tmux support
 */

use serde::{Deserialize, Serialize};
use tauri::{Manager, State};
use std::process::Command;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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

/// Monitor tmux session output and emit events to frontend
async fn monitor_tmux_session(
    app: tauri::AppHandle,
    session_id: String,
    instance_number: usize,
    log_file_path: String,
) {
    tracing::info!("🔍 Starting monitoring for session {} (instance {})", session_id, instance_number);

    let mut last_output = String::new();
    let mut last_line_count = 0;
    let mut last_log_size = 0;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Check if session still exists
        let check_output = Command::new("tmux")
            .arg("has-session")
            .arg("-t")
            .arg(&session_id)
            .output();

        match check_output {
            Ok(output) if output.status.success() => {
                // Session exists, try to read from log file first (more reliable for Claude output)
                if let Ok(log_contents) = tokio::fs::read_to_string(&log_file_path).await {
                    let log_size = log_contents.len();

                    if log_size > last_log_size {
                        // New content in log file
                        let new_content = &log_contents[last_log_size..];

                        if !new_content.trim().is_empty() {
                            let payload = serde_json::json!({
                                "instance": instance_number,
                                "output": new_content,
                                "status": "running"
                            });

                            let _ = app.emit_all("competition-output", payload);
                        }

                        last_log_size = log_size;
                    }
                } else {
                    // Fallback to tmux capture-pane if log file not available yet
                    let capture_output = Command::new("tmux")
                        .arg("capture-pane")
                        .arg("-p")
                        .arg("-t")
                        .arg(&session_id)
                        .arg("-S")
                        .arg("-")
                        .output();

                    if let Ok(capture) = capture_output {
                        if capture.status.success() {
                            let current_output = String::from_utf8_lossy(&capture.stdout).to_string();
                            let current_lines: Vec<&str> = current_output.lines().collect();

                            // Only send new lines
                            if current_lines.len() > last_line_count {
                                let new_lines = &current_lines[last_line_count..];
                                let new_content = new_lines.join("\n");

                                if !new_content.trim().is_empty() {
                                    let payload = serde_json::json!({
                                        "instance": instance_number,
                                        "output": new_content + "\n",
                                        "status": "running"
                                    });

                                    let _ = app.emit_all("competition-output", payload);
                                }

                                last_line_count = current_lines.len();
                            }

                            last_output = current_output;
                        }
                    }
                }
            }
            _ => {
                // Session no longer exists - completed or failed
                tracing::info!("Tmux session {} has ended", session_id);

                // Send final output from log file
                if let Ok(final_output) = tokio::fs::read_to_string(&log_file_path).await {
                    if final_output.len() > last_log_size {
                        let final_content = &final_output[last_log_size..];
                        if !final_content.trim().is_empty() {
                            let payload = serde_json::json!({
                                "instance": instance_number,
                                "output": final_content,
                                "status": "completed"
                            });
                            let _ = app.emit_all("competition-output", payload);
                        }
                    }
                }

                let payload = serde_json::json!({
                    "instance": instance_number,
                    "output": "",
                    "status": "completed"
                });
                let _ = app.emit_all("competition-output", payload);
                break;
            }
        }
    }
}

/// Execute Claude Code Competition
///
/// Creates multiple git worktrees and launches Claude Code instances in parallel
#[tauri::command]
pub async fn execute_claude_code_competition(
    app_handle: tauri::AppHandle,
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
        let output_log_path = format!("{}/.claude-output.log", worktree_path);

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

        // Enable pipe-pane to capture all output to a log file
        // This ensures Claude CLI output is captured even if tmux capture-pane misses it
        let pipe_output = Command::new("tmux")
            .arg("pipe-pane")
            .arg("-t")
            .arg(&session_id)
            .arg("-o")
            .arg(format!("cat >> {}", output_log_path))
            .output()
            .map_err(|e| format!("Failed to enable pipe-pane: {}", e))?;

        if !pipe_output.status.success() {
            let error = String::from_utf8_lossy(&pipe_output.stderr);
            tracing::warn!("Failed to enable pipe-pane for instance {}: {}", i, error);
        }

        // Send Claude Code command to tmux session
        // Use --print flag for non-interactive mode to avoid Ink raw mode errors
        // Note: Use echo | claude because it forces non-interactive mode in tmux
        let claude_cmd = format!(
            "echo '{}' | claude --model {} --print",
            request.task.replace("'", "'\\''"),
            request.model
        );

        let send_output = Command::new("tmux")
            .arg("send-keys")
            .arg("-t")
            .arg(&session_id)
            .arg(format!("{} && exit", claude_cmd))
            .arg("C-m")
            .output()
            .map_err(|e| format!("Failed to send command: {}", e))?;

        if !send_output.status.success() {
            let error = String::from_utf8_lossy(&send_output.stderr);
            return Err(format!("Failed to send command to instance {}: {}", i, error));
        }

        tracing::info!("Launched Claude Code instance {} in session {}", i, session_id);

        // Start monitoring task for this instance
        let app = app_handle.clone();
        let monitor_session_id = session_id.clone();
        let monitor_instance_number = i;
        let monitor_log_path = output_log_path.clone();

        tauri::async_runtime::spawn(async move {
            monitor_tmux_session(app, monitor_session_id, monitor_instance_number, monitor_log_path).await;
        });

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

//
// ============================================================
// Claude Code Debate Mode (Multi-Agent Debate System)
// ============================================================
//

/// Role definition for debate participants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleDefinition {
    pub id: String,
    pub name: String,
    pub system_prompt: String,
}

/// Debate execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebateRequest {
    pub task: String,
    pub roles: Vec<RoleDefinition>,  // 3 roles (Architect, Pragmatist, Innovator)
    pub model: String,  // "sonnet", "haiku", "opus"
    pub timeout_seconds: u64,  // Per-round timeout (default: 800s = 13.3 min)
    pub preserve_worktrees: bool,  // Keep worktrees after completion
}

/// Debate execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebateResult {
    pub debate_id: String,
    pub status: String,  // "started", "round_1", "round_2", "round_3", "completed", "failed"
    pub message: String,
}

/// Round output (result from one agent in one round)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundOutput {
    pub round: u8,
    pub role_id: String,
    pub role_name: String,
    pub output: String,
    pub status: String,  // "running", "completed", "failed"
    pub started_at: String,
    pub completed_at: Option<String>,
    pub execution_time_ms: u64,
}

/// Debate status (complete state)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebateStatus {
    pub debate_id: String,
    pub current_round: u8,  // 1, 2, or 3
    pub total_rounds: u8,  // Always 3
    pub status: String,  // "started", "round_1", "round_2", "round_3", "completed", "failed"
    pub round_outputs: Vec<RoundOutput>,
    pub worktree_path: String,
    pub context_files: Vec<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

/// Execute Claude Code Debate
///
/// Creates a single git worktree and executes 3 rounds of debate sequentially
#[tauri::command]
pub async fn execute_debate(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    request: DebateRequest,
) -> Result<DebateResult, String> {
    // Validation
    if request.roles.len() != 3 {
        return Err("Debate mode requires exactly 3 roles".to_string());
    }

    if request.task.trim().is_empty() {
        return Err("Task cannot be empty".to_string());
    }

    let valid_models = ["sonnet", "haiku", "opus"];
    if !valid_models.contains(&request.model.as_str()) {
        return Err(format!("Invalid model. Must be one of: {:?}", valid_models));
    }

    let debate_id = uuid::Uuid::new_v4().to_string();
    let started_at = chrono::Utc::now();

    tracing::info!(
        "Starting debate: {} with 3 roles, model: {}, task: {}",
        debate_id,
        request.model,
        request.task.chars().take(50).collect::<String>()
    );

    let working_dir = state.working_dir.lock().await;
    let base_path = working_dir.clone();
    drop(working_dir);

    // Create debate directory
    let debate_dir = format!("{}/.worktrees/debate-{}", base_path.display(), &debate_id[..8]);
    std::fs::create_dir_all(&debate_dir).map_err(|e| e.to_string())?;

    // Create context directory for shared files
    let context_dir = format!("{}/context", debate_dir);
    std::fs::create_dir_all(&context_dir).map_err(|e| e.to_string())?;

    // Create single worktree for debate
    let worktree_path = format!("{}/debate-workspace", debate_dir);
    let branch_name = format!("debate-{}", &debate_id[..8]);

    tracing::info!("Creating debate worktree at {}", worktree_path);

    let mut cmd = Command::new("git");
    cmd.arg("worktree")
        .arg("add")
        .arg("-b")
        .arg(&branch_name)
        .arg(&worktree_path)
        .current_dir(&base_path);

    let output = cmd.output().map_err(|e| {
        format!("Failed to create worktree: {}", e)
    })?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        tracing::error!("Worktree creation failed: {}", error);
        return Err(format!("Failed to create worktree: {}", error));
    }

    tracing::info!("Debate worktree created successfully");

    // Initialize debate status in state
    let initial_status = DebateStatus {
        debate_id: debate_id.clone(),
        current_round: 0,
        total_rounds: 3,
        status: "started".to_string(),
        round_outputs: Vec::new(),
        worktree_path: worktree_path.clone(),
        context_files: Vec::new(),
        started_at: started_at.to_rfc3339(),
        completed_at: None,
    };

    {
        let mut debates = state.debates.lock()
            .map_err(|e| format!("Failed to lock debates: {}", e))?;
        debates.insert(debate_id.clone(), initial_status);
    }

    // Start debate execution in background
    let app = app_handle.clone();
    let debate_id_clone = debate_id.clone();
    let request_clone = request.clone();
    let worktree_path_clone = worktree_path.clone();
    let context_dir_clone = context_dir.clone();
    let debates_clone = Arc::clone(&state.debates);

    tauri::async_runtime::spawn(async move {
        if let Err(e) = execute_debate_rounds(
            app,
            debates_clone,
            debate_id_clone,
            request_clone,
            worktree_path_clone,
            context_dir_clone,
        ).await {
            tracing::error!("Debate execution failed: {}", e);
        }
    });

    Ok(DebateResult {
        debate_id,
        status: "started".to_string(),
        message: "Debate started successfully".to_string(),
    })
}

/// Execute all 3 debate rounds sequentially
async fn execute_debate_rounds(
    app: tauri::AppHandle,
    debates: Arc<Mutex<HashMap<String, DebateStatus>>>,
    debate_id: String,
    request: DebateRequest,
    worktree_path: String,
    context_dir: String,
) -> Result<(), String> {
    tracing::info!("Starting 3-round debate execution for {}", debate_id);

    // Helper function to update debate status
    let update_status = |round: u8, status: &str| {
        if let Ok(mut debates_lock) = debates.lock() {
            if let Some(debate) = debates_lock.get_mut(&debate_id) {
                debate.current_round = round;
                debate.status = status.to_string();
            }
        }
    };

    // Round 1: Independent proposals
    update_status(1, "round_1");
    emit_debate_status(&app, &debate_id, 1, "round_1");
    execute_round(
        app.clone(),
        debate_id.clone(),
        1,
        request.clone(),
        worktree_path.clone(),
        context_dir.clone(),
        None, // No previous context
    ).await?;

    // Round 2: Critical analysis with Round 1 context
    update_status(2, "round_2");
    emit_debate_status(&app, &debate_id, 2, "round_2");
    let round1_context = load_round_context(&context_dir, 1)?;
    execute_round(
        app.clone(),
        debate_id.clone(),
        2,
        request.clone(),
        worktree_path.clone(),
        context_dir.clone(),
        Some(round1_context),
    ).await?;

    // Round 3: Consensus formation with Round 1+2 context
    update_status(3, "round_3");
    emit_debate_status(&app, &debate_id, 3, "round_3");
    let round2_context = load_round_context(&context_dir, 2)?;
    let combined_context = format!("{}\n\n--- Round 2 ---\n\n{}",
        load_round_context(&context_dir, 1)?,
        round2_context
    );
    execute_round(
        app.clone(),
        debate_id.clone(),
        3,
        request.clone(),
        worktree_path.clone(),
        context_dir.clone(),
        Some(combined_context),
    ).await?;

    // Debate completed - update final status
    if let Ok(mut debates_lock) = debates.lock() {
        if let Some(debate) = debates_lock.get_mut(&debate_id) {
            debate.status = "completed".to_string();
            debate.completed_at = Some(chrono::Utc::now().to_rfc3339());
        }
    }
    emit_debate_status(&app, &debate_id, 3, "completed");
    tracing::info!("Debate {} completed successfully", debate_id);

    Ok(())
}

/// Execute a single round with all 3 roles
async fn execute_round(
    app: tauri::AppHandle,
    debate_id: String,
    round: u8,
    request: DebateRequest,
    worktree_path: String,
    context_dir: String,
    previous_context: Option<String>,
) -> Result<(), String> {
    tracing::info!("Executing round {} for debate {}", round, debate_id);

    let mut round_outputs = Vec::new();

    for (index, role) in request.roles.iter().enumerate() {
        let started_at = chrono::Utc::now();

        // Build prompt with context
        let prompt = if let Some(ref context) = previous_context {
            format!(
                "{}\n\n--- Previous Round Context ---\n{}\n\n--- Your Task ---\n{}",
                role.system_prompt,
                context,
                request.task
            )
        } else {
            format!("{}\n\n{}", role.system_prompt, request.task)
        };

        // Create tmux session
        let session_id = format!("claude-debate-{}-r{}-{}", &debate_id[..8], round, index + 1);
        let output_log_path = format!("{}/.claude-round{}-{}.log", worktree_path, round, role.id);

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
            return Err(format!("Failed to create tmux session: {}", error));
        }

        // Enable pipe-pane for output capture
        let _ = Command::new("tmux")
            .arg("pipe-pane")
            .arg("-t")
            .arg(&session_id)
            .arg("-o")
            .arg(format!("cat >> {}", output_log_path))
            .output();

        // Send Claude Code command
        let claude_cmd = format!(
            "echo '{}' | claude --model {} code --print",
            prompt.replace("'", "'\\''"),
            request.model
        );

        let send_output = Command::new("tmux")
            .arg("send-keys")
            .arg("-t")
            .arg(&session_id)
            .arg(format!("{} && exit", claude_cmd))
            .arg("C-m")
            .output()
            .map_err(|e| format!("Failed to send command: {}", e))?;

        if !send_output.status.success() {
            let error = String::from_utf8_lossy(&send_output.stderr);
            return Err(format!("Failed to send command: {}", error));
        }

        tracing::info!("Launched role {} ({}) in round {}", role.name, role.id, round);

        // Wait for completion (poll tmux session)
        let timeout = tokio::time::Duration::from_secs(request.timeout_seconds);
        let start_time = tokio::time::Instant::now();

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            // Check if session still exists
            let check_output = Command::new("tmux")
                .arg("has-session")
                .arg("-t")
                .arg(&session_id)
                .output();

            match check_output {
                Ok(output) if output.status.success() => {
                    // Session still running
                    if start_time.elapsed() > timeout {
                        // Timeout - kill session
                        let _ = Command::new("tmux")
                            .arg("kill-session")
                            .arg("-t")
                            .arg(&session_id)
                            .output();

                        return Err(format!("Round {} role {} timed out", round, role.name));
                    }
                }
                _ => {
                    // Session completed
                    break;
                }
            }
        }

        // Read output
        let output = tokio::fs::read_to_string(&output_log_path)
            .await
            .unwrap_or_else(|_| String::from("No output captured"));

        let completed_at = chrono::Utc::now();
        let execution_time_ms = (completed_at - started_at).num_milliseconds() as u64;

        round_outputs.push(output.clone());

        // Emit round output event
        let payload = serde_json::json!({
            "debateId": debate_id,
            "round": round,
            "roleId": role.id,
            "roleName": role.name,
            "output": output,
            "status": "completed",
            "executionTimeMs": execution_time_ms
        });
        let _ = app.emit_all("debate-round-output", payload);

        tracing::info!("Round {} role {} completed in {}ms", round, role.name, execution_time_ms);
    }

    // Save round context to file
    let context_file_path = format!("{}/round{}.txt", context_dir, round);
    let combined_output = round_outputs.join("\n\n--- Next Role ---\n\n");
    tokio::fs::write(&context_file_path, combined_output)
        .await
        .map_err(|e| format!("Failed to save context: {}", e))?;

    tracing::info!("Round {} completed and context saved", round);

    Ok(())
}

/// Load context from a specific round
fn load_round_context(context_dir: &str, round: u8) -> Result<String, String> {
    let context_file_path = format!("{}/round{}.txt", context_dir, round);
    std::fs::read_to_string(&context_file_path)
        .map_err(|e| format!("Failed to load round {} context: {}", round, e))
}

/// Emit debate status event
fn emit_debate_status(app: &tauri::AppHandle, debate_id: &str, current_round: u8, status: &str) {
    let payload = serde_json::json!({
        "debateId": debate_id,
        "currentRound": current_round,
        "status": status
    });
    let _ = app.emit_all("debate-status", payload);
}

/// Get debate status
#[tauri::command]
pub async fn get_debate_status(
    state: State<'_, AppState>,
    debate_id: String,
) -> Result<DebateStatus, String> {
    let debates = state.debates.lock()
        .map_err(|e| format!("Failed to lock debates: {}", e))?;

    debates.get(&debate_id)
        .cloned()
        .ok_or_else(|| format!("Debate {} not found", debate_id))
}

/// Cancel a running debate
#[tauri::command]
pub async fn cancel_debate(
    state: State<'_, AppState>,
    debate_id: String,
    cleanup_worktrees: bool,
) -> Result<(), String> {
    tracing::info!("Cancelling debate: {}", debate_id);

    // Kill all tmux sessions for this debate
    let session_pattern = format!("claude-debate-{}", &debate_id[..8]);

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
        let debate_dir = format!("{}/.worktrees/debate-{}", working_dir.display(), &debate_id[..8]);
        drop(working_dir);

        // Remove worktree
        let worktree_path = format!("{}/debate-workspace", debate_dir);
        let _ = Command::new("git")
            .arg("worktree")
            .arg("remove")
            .arg("--force")
            .arg(&worktree_path)
            .output();

        // Remove debate directory
        let _ = std::fs::remove_dir_all(&debate_dir);
        tracing::info!("Cleaned up debate directory: {}", debate_dir);
    }

    Ok(())
}
