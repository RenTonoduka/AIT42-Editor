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
use ait42_ait42::{AgentRegistry, AgentExecutor, Coordinator, config::AIT42Config, ExecutionMode};
use tracing::{info, warn, error};

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

/// Initialize AIT42 agent registry if not already initialized
async fn ensure_registry_initialized(state: &State<'_, AppState>) -> Result<(), String> {
    let mut registry_guard = state.agent_registry.lock()
        .map_err(|e| format!("Failed to lock agent registry: {}", e))?;

    if registry_guard.is_some() {
        return Ok(());
    }

    // Initialize registry
    let config = match AIT42Config::load() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load AIT42 config: {}. Using default.", e);
            // Try to detect AIT42 root from common locations
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let ait42_root = std::path::PathBuf::from(&home)
                .join("Programming/AI/02_Workspace/05_Client/03_Sun/AIT42");
            AIT42Config::new(ait42_root)
        }
    };

    let registry = AgentRegistry::load_from_directory(&config.agents_dir())
        .map_err(|e| format!("Failed to load agent registry: {}", e))?;

    *registry_guard = Some(registry);
    Ok(())
}

/// Get registry (must be initialized first)
fn get_registry<'a>(state: &'a State<'a, AppState>) -> Result<std::sync::MutexGuard<'a, Option<AgentRegistry>>, String> {
    // Note: This is a synchronous function, but we need async initialization
    // For now, we'll initialize synchronously if needed
    let mut registry_guard = state.agent_registry.lock()
        .map_err(|e| format!("Failed to lock agent registry: {}", e))?;

    if registry_guard.is_none() {
        // Try to initialize synchronously
        let config = match AIT42Config::load() {
            Ok(config) => config,
            Err(e) => {
                warn!("Failed to load AIT42 config: {}. Using default.", e);
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                let ait42_root = std::path::PathBuf::from(&home)
                    .join("Programming/AI/02_Workspace/05_Client/03_Sun/AIT42");
                AIT42Config::new(ait42_root)
            }
        };

        let registry = AgentRegistry::load_from_directory(&config.agents_dir())
            .map_err(|e| format!("Failed to load agent registry: {}", e))?;
        *registry_guard = Some(registry);
    }

    Ok(registry_guard)
}

/// Initialize AIT42 coordinator if not already initialized
async fn ensure_coordinator_initialized(state: &State<'_, AppState>) -> Result<(), String> {
    let mut coordinator_guard = state.coordinator.lock().await;

    if coordinator_guard.is_some() {
        return Ok(());
    }

    // Initialize coordinator
    let config = match AIT42Config::load() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load AIT42 config: {}. Using default.", e);
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let ait42_root = std::path::PathBuf::from(&home)
                .join("Programming/AI/02_Workspace/05_Client/03_Sun/AIT42");
            AIT42Config::new(ait42_root)
        }
    };

    let coordinator = Coordinator::new(config)
        .map_err(|e| format!("Failed to initialize coordinator: {}", e))?;

    *coordinator_guard = Some(coordinator);
    Ok(())
}

/// Get coordinator (must be initialized first)
async fn get_coordinator<'a>(state: &'a State<'a, AppState>) -> Result<tokio::sync::MutexGuard<'a, Option<Coordinator>>, String> {
    ensure_coordinator_initialized(state).await?;
    Ok(state.coordinator.lock().await)
}

/**
 * List all available agents
 */
#[tauri::command]
pub async fn list_agents(state: State<'_, AppState>) -> Result<Vec<AgentInfo>, String> {
    ensure_registry_initialized(&state).await?;
    let registry_guard = get_registry(&state)?;
    let registry = registry_guard.as_ref()
        .ok_or_else(|| "Agent registry not initialized".to_string())?;

    let agents = registry.list();
    let mut agent_infos = Vec::new();

    for agent in agents {
        agent_infos.push(AgentInfo {
            name: agent.name.clone(),
            description: agent.description.clone(),
            category: format!("{:?}", agent.category),
            tools: agent.tools.clone(),
        });
    }

    info!("Listed {} agents", agent_infos.len());
    Ok(agent_infos)
}

/**
 * Get information about a specific agent
 */
#[tauri::command]
pub async fn get_agent_info(
    state: State<'_, AppState>,
    agent_name: String,
) -> Result<AgentInfo, String> {
    ensure_registry_initialized(&state).await?;
    let registry_guard = get_registry(&state)?;
    let registry = registry_guard.as_ref()
        .ok_or_else(|| "Agent registry not initialized".to_string())?;

    let agent = registry.get(&agent_name)
        .ok_or_else(|| format!("Agent not found: {}", agent_name))?;

    Ok(AgentInfo {
        name: agent.name.clone(),
        description: agent.description.clone(),
        category: format!("{:?}", agent.category),
        tools: agent.tools.clone(),
    })
}

/**
 * Execute a single agent
 */
#[tauri::command]
pub async fn execute_agent(
    state: State<'_, AppState>,
    request: AgentExecutionRequest,
) -> Result<AgentExecutionResponse, String> {
    // Verify agent exists
    {
        ensure_registry_initialized(&state).await?;
        let registry_guard = get_registry(&state)?;
        let registry = registry_guard.as_ref()
            .ok_or_else(|| "Agent registry not initialized".to_string())?;
        if registry.get(&request.agent_name).is_none() {
            return Err(format!("Agent not found: {}", request.agent_name));
        }
    }

    let execution_id = uuid::Uuid::new_v4().to_string();

    info!("Executing agent: {} with task: {}", request.agent_name, request.task);

    // Build task with context if provided
    let task = if let Some(ref context) = request.context {
        format!("{}\n\nContext:\n{}", request.task, context)
    } else {
        request.task.clone()
    };

    // Create coordinator and executor for this execution
    let config = match AIT42Config::load() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load AIT42 config: {}. Using default.", e);
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let ait42_root = std::path::PathBuf::from(&home)
                .join("Programming/AI/02_Workspace/05_Client/03_Sun/AIT42");
            AIT42Config::new(ait42_root)
        }
    };

    let coordinator = Coordinator::new(config)
        .map_err(|e| format!("Failed to initialize coordinator: {}", e))?;
    let mut executor = AgentExecutor::new(coordinator);
    let mode = ExecutionMode::Single(request.agent_name.clone());

    match executor.execute(mode, &task).await {
        Ok(results) => {
            if results.is_empty() {
                return Err("No execution results returned".to_string());
            }

            let result = &results[0];
            Ok(AgentExecutionResponse {
                execution_id,
                agent_name: result.agent_name.clone(),
                status: format!("{:?}", result.status),
                output: Some(result.output.clone()),
                error: None,
            })
        }
        Err(e) => {
            error!("Agent execution failed: {}", e);
            Ok(AgentExecutionResponse {
                execution_id,
                agent_name: request.agent_name,
                status: "failed".to_string(),
                output: None,
                error: Some(e.to_string()),
            })
        }
    }
}

/**
 * Execute multiple agents in parallel
 */
#[tauri::command]
pub async fn execute_parallel(
    state: State<'_, AppState>,
    request: ParallelExecutionRequest,
) -> Result<Vec<AgentExecutionResponse>, String> {
    // Verify all agents exist
    {
        ensure_registry_initialized(&state).await?;
        let registry_guard = get_registry(&state)?;
        let registry = registry_guard.as_ref()
            .ok_or_else(|| "Agent registry not initialized".to_string())?;
        for agent_name in &request.agents {
            if registry.get(agent_name).is_none() {
                return Err(format!("Agent not found: {}", agent_name));
            }
        }
    }

    info!("Executing {} agents in parallel", request.agents.len());

    // Build task with context if provided
    let task = if let Some(ref context) = request.context {
        format!("{}\n\nContext:\n{}", request.task, context)
    } else {
        request.task.clone()
    };

    // Create coordinator and executor for this execution
    let config = match AIT42Config::load() {
        Ok(config) => config,
        Err(e) => {
            warn!("Failed to load AIT42 config: {}. Using default.", e);
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let ait42_root = std::path::PathBuf::from(&home)
                .join("Programming/AI/02_Workspace/05_Client/03_Sun/AIT42");
            AIT42Config::new(ait42_root)
        }
    };

    let coordinator = Coordinator::new(config)
        .map_err(|e| format!("Failed to initialize coordinator: {}", e))?;
    let mut executor = AgentExecutor::new(coordinator);
    let mode = ExecutionMode::Parallel(request.agents.clone());

    match executor.execute(mode, &task).await {
        Ok(results) => {
            let responses: Vec<AgentExecutionResponse> = results
                .into_iter()
                .map(|result| {
                    let execution_id = uuid::Uuid::new_v4().to_string();
                    AgentExecutionResponse {
                        execution_id,
                        agent_name: result.agent_name.clone(),
                        status: format!("{:?}", result.status),
                        output: Some(result.output.clone()),
                        error: None,
                    }
                })
                .collect();

            Ok(responses)
        }
        Err(e) => {
            error!("Parallel execution failed: {}", e);
            Err(format!("Parallel execution failed: {}", e))
        }
    }
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

    // Helper function to strip ANSI codes using regex-like approach
    // Remove ANSI escape sequences: \x1b[...m, \x1b[?..., etc.
    let strip_ansi = |text: &str| -> String {
        let mut result = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\x1b' {
                // Skip ANSI escape sequence
                if let Some(&'[') = chars.peek() {
                    chars.next(); // consume '['
                    // Skip until we find a letter (end of escape sequence)
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_alphabetic() || next_ch == 'm' {
                            chars.next();
                            break;
                        }
                        chars.next();
                    }
                } else if let Some(&'%') = chars.peek() {
                    chars.next(); // consume '%'
                } else if let Some(&'(') = chars.peek() {
                    chars.next(); // consume '('
                } else if let Some(&')') = chars.peek() {
                    chars.next(); // consume ')'
                } else if let Some(&'#') = chars.peek() {
                    chars.next(); // consume '#'
                } else if let Some(&'\\') = chars.peek() {
                    chars.next(); // consume '\'
                } else {
                    // Not an ANSI sequence, keep the character
                    result.push(ch);
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    };

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
                            // Strip ANSI codes before sending
                            let cleaned_content = strip_ansi(new_content);

                            // First event emission timing log
                            if last_log_size == 0 {
                                tracing::info!(
                                    "🕐 First event emission for instance {} at {:?}",
                                    instance_number,
                                    std::time::SystemTime::now()
                                );
                            }

                            let payload = serde_json::json!({
                                "instance": instance_number,
                                "output": cleaned_content,
                                "status": "running"
                            });

                            // Log the exact payload being sent for debugging
                            tracing::info!(
                                "📤 Preparing to emit event 'competition-output' (incremental) with payload: instance={}, output_length={}, status=\"running\", preview=\"{}...\"",
                                instance_number,
                                cleaned_content.len(),
                                cleaned_content.chars().take(50).collect::<String>().replace('\n', "\\n")
                            );

                            match app.emit_all("competition-output", payload.clone()) {
                                Ok(_) => tracing::info!("✅ Sent {} bytes (incremental) for instance {}", cleaned_content.len(), instance_number),
                                Err(e) => tracing::error!("❌ Failed to emit incremental output for instance {}: {}", instance_number, e),
                            }
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
                                    // Strip ANSI codes before sending
                                    let cleaned_content = strip_ansi(&new_content);
                                    let output_with_newline = format!("{}\n", cleaned_content);
                                    let content_len = cleaned_content.len();

                                    let payload = serde_json::json!({
                                        "instance": instance_number,
                                        "output": output_with_newline,
                                        "status": "running"
                                    });

                                    match app.emit_all("competition-output", payload) {
                                        Ok(_) => tracing::debug!("📤 Sent {} bytes (tmux fallback) for instance {}", content_len, instance_number),
                                        Err(e) => tracing::warn!("⚠️ Failed to emit tmux output: {}", e),
                                    }
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

                // Send final output from log file (ALWAYS send full content on completion)
                match tokio::fs::read_to_string(&log_file_path).await {
                    Ok(final_output) => {
                        if !final_output.trim().is_empty() {
                            // Strip ANSI codes before sending
                            let cleaned_output = strip_ansi(&final_output);
                            
                            let payload = serde_json::json!({
                                "instance": instance_number,
                                "output": cleaned_output,
                                "status": "completed"
                            });

                            // Debug: Log payload details
                            tracing::info!("📤 Emitting event 'competition-output': instance={}, output_len={}, status=completed",
                                instance_number, cleaned_output.len());

                            match app.emit_all("competition-output", payload) {
                                Ok(_) => tracing::info!("✅ Sent final output for instance {} ({} bytes)", instance_number, cleaned_output.len()),
                                Err(e) => tracing::error!("❌ Failed to emit final output for instance {}: {}", instance_number, e),
                            }
                        } else {
                            tracing::warn!("⚠️ Log file for instance {} is empty", instance_number);

                            // Send completion event even if log is empty
                            let payload = serde_json::json!({
                                "instance": instance_number,
                                "output": "⚠️ No output captured",
                                "status": "completed"
                            });
                            let _ = app.emit_all("competition-output", payload);
                        }
                    }
                    Err(e) => {
                        tracing::error!("❌ Failed to read log file for instance {}: {}", instance_number, e);

                        // Send completion event even if file read failed
                        let payload = serde_json::json!({
                            "instance": instance_number,
                            "output": format!("❌ Failed to read output: {}", e),
                            "status": "error",
                            "error": e.to_string()
                        });
                        let _ = app.emit_all("competition-output", payload);
                    }
                }

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
    let project_root = working_dir.clone();
    drop(working_dir); // Release lock

    tracing::info!("📁 Project root for competition: {}", project_root.display());

    // Create worktrees in the project directory (not in home directory)
    // This keeps worktrees with their respective projects
    let ait42_worktrees = project_root.join(".ait42").join(".worktrees");

    tracing::info!("📁 Worktrees directory: {}", ait42_worktrees.display());

    // Create competition directory in project's .ait42 directory
    let competition_dir = format!("{}/competition-{}", ait42_worktrees.display(), &competition_id[..8]);

    tracing::info!("📁 Creating competition directory: {}", competition_dir);
    std::fs::create_dir_all(&competition_dir).map_err(|e| {
        let err_msg = format!("Failed to create competition directory at {}: {}", competition_dir, e);
        tracing::error!("{}", err_msg);
        err_msg
    })?;

    let mut instances = Vec::new();

    // ハンドシェイク準備: フロントエンドのリスナー登録を待つ仕組み
    let ready_signal_received = Arc::new(Mutex::new(false));
    let ready_clone = Arc::clone(&ready_signal_received);
    let competition_id_for_listener = competition_id.clone();

    tracing::info!("🕐 [HANDSHAKE] Registering global listener BEFORE creating worktrees for competition {} at {:?}",
        competition_id, std::time::SystemTime::now());

    // STEP 1: グローバルリスナーを先に登録（非同期タスクの外で）
    let listener_handle = app_handle.listen_global("competition-listener-ready", move |event| {
        tracing::info!("🔔 [HANDSHAKE] Received event on 'competition-listener-ready' at {:?}", std::time::SystemTime::now());
        if let Some(payload) = event.payload() {
            tracing::debug!("📦 [HANDSHAKE] Payload: {}", payload);
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(payload) {
                if let Some(received_id) = data.get("competitionId").and_then(|v| v.as_str()) {
                    tracing::info!("🔍 [HANDSHAKE] Checking received_id='{}' vs expected='{}'", received_id, competition_id_for_listener);
                    if received_id == competition_id_for_listener {
                        tracing::info!("✅ [HANDSHAKE] Frontend ready signal received for competition {}", competition_id_for_listener);
                        *ready_clone.lock().unwrap() = true;
                    } else {
                        tracing::warn!("⚠️ [HANDSHAKE] Competition ID mismatch: received '{}', expected '{}'", received_id, competition_id_for_listener);
                    }
                } else {
                    tracing::warn!("⚠️ [HANDSHAKE] No competitionId field in payload: {:?}", data);
                }
            } else {
                tracing::error!("❌ [HANDSHAKE] Failed to parse payload as JSON: {}", payload);
            }
        } else {
            tracing::error!("❌ [HANDSHAKE] Event payload is None");
        }
    });

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
            .current_dir(&project_root);

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
        // Use --permission-mode bypassPermissions to auto-approve changes and prevent interactive prompts
        // Note: Use echo | claude because it forces non-interactive mode in tmux
        // Build Claude Code command with explicit instructions
        let enhanced_task = format!(
            "{}

あなたはこのタスクを完遂する開発者です。以下の手順で実行してください：
1. タスクの要件を分析
2. 必要なファイルやコードを特定
3. 具体的な実装を提案・実行
4. テストと検証

質問はせず、直接実装してください。",
            request.task
        );

        // Escape newlines and quotes for proper shell handling
        let escaped_task = enhanced_task
            .replace('\\', "\\\\")      // Escape backslashes first
            .replace('\n', "\\n")       // Escape newlines
            .replace('\'', "'\\''");    // Escape single quotes

        let claude_cmd = format!(
            "echo -e '{}' | claude --model {} --print --permission-mode bypassPermissions",
            escaped_task,
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

        // モニタリングタスクを起動（ハンドシェイク待機済み前提）
        let app = app_handle.clone();
        let monitor_session_id = session_id.clone();
        let monitor_instance_number = i;
        let monitor_log_path = output_log_path.clone();
        let ready_signal_clone = Arc::clone(&ready_signal_received);

        tauri::async_runtime::spawn(async move {
            // STEP 2: フロントエンド準備完了を待機（最大5秒タイムアウト）
            let timeout_duration = std::time::Duration::from_secs(5);
            let start_time = std::time::Instant::now();

            tracing::info!("⏳ [HANDSHAKE] Instance {} waiting for frontend ready signal...", monitor_instance_number);

            while !*ready_signal_clone.lock().unwrap() {
                if start_time.elapsed() > timeout_duration {
                    tracing::warn!("⚠️ [HANDSHAKE] Frontend ready signal timeout for instance {}, proceeding anyway", monitor_instance_number);
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }

            if *ready_signal_clone.lock().unwrap() {
                tracing::info!("✅ [HANDSHAKE] Frontend ready confirmed for instance {}", monitor_instance_number);
            }

            // STEP 3: モニタリング開始
            tracing::info!("🚀 Starting monitoring for instance {} at {:?}",
                monitor_instance_number, std::time::SystemTime::now());

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

    // STEP 4: リスナークリーンアップは最初のインスタンス完了時に行う
    // （実際にはタイムアウト後に自動的にunlistenすべきだが、ここでは簡略化）
    let app_for_cleanup = app_handle.clone();
    let competition_id_for_cleanup = competition_id.clone();
    tauri::async_runtime::spawn(async move {
        // 10秒後にリスナーをクリーンアップ（ハンドシェイク期間終了）
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        app_for_cleanup.unlisten(listener_handle);
        tracing::info!("🧹 [HANDSHAKE] Cleaned up listener for competition {}", competition_id_for_cleanup);
    });

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
        // Get project root for worktree cleanup
        let working_dir = state.working_dir.lock().await;
        let project_root = working_dir.clone();
        drop(working_dir);

        // Use project directory for worktrees (not home directory)
        let ait42_worktrees = project_root.join(".ait42").join(".worktrees");
        let competition_dir = format!("{}/competition-{}", ait42_worktrees.display(), &competition_id[..8]);

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
    pub worktree_path: String,
    pub branch: String,
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
    let project_root = working_dir.clone();
    drop(working_dir);

    // Create worktrees in the project directory (not in home directory)
    // This keeps worktrees with their respective projects
    let ait42_worktrees = project_root.join(".ait42").join(".worktrees");

    // Create debate directory in project's .ait42 directory
    let debate_dir = format!("{}/debate-{}", ait42_worktrees.display(), &debate_id[..8]);
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
        .current_dir(&project_root);

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
    let working_dir_clone = Arc::clone(&state.working_dir);

    tauri::async_runtime::spawn(async move {
        if let Err(e) = execute_debate_rounds(
            app,
            debates_clone,
            working_dir_clone,
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
        worktree_path,
        branch: branch_name,
    })
}

/// Execute all 3 debate rounds sequentially
async fn execute_debate_rounds(
    app: tauri::AppHandle,
    debates: Arc<Mutex<HashMap<String, DebateStatus>>>,
    working_dir: Arc<tokio::sync::Mutex<std::path::PathBuf>>,
    debate_id: String,
    request: DebateRequest,
    worktree_path: String,
    context_dir: String,
) -> Result<(), String> {
    tracing::info!("Starting 3-round debate execution for {}", debate_id);

    // Helper function to update debate status and persist to session history
    let update_status = |round: u8, status: String| {
        let debate_id_clone = debate_id.clone();
        let working_dir_clone = working_dir.clone();
        let debates_clone = debates.clone();

        // Spawn async task for persistence
        tauri::async_runtime::spawn(async move {
            // Update in-memory state
            let (started_at, updated_at) = {
                let mut debates_lock = match debates_clone.lock() {
                    Ok(lock) => lock,
                    Err(e) => {
                        tracing::error!("Failed to lock debates: {}", e);
                        return;
                    }
                };

                if let Some(debate) = debates_lock.get_mut(&debate_id_clone) {
                    debate.current_round = round;
                    debate.status = status.clone();
                    if status == "completed" {
                        debate.completed_at = Some(chrono::Utc::now().to_rfc3339());
                    }
                    (debate.started_at.clone(), chrono::Utc::now().to_rfc3339())
                } else {
                    tracing::error!("Debate {} not found in state", debate_id_clone);
                    return;
                }
            };

            // Get workspace path
            let workspace_path = {
                let wd = working_dir_clone.lock().await;
                wd.to_string_lossy().to_string()
            };

            // Prepare session data for persistence
            let session = crate::commands::session_history::WorktreeSession {
                id: debate_id_clone.clone(),
                r#type: "debate".to_string(),
                task: "Debate in progress".to_string(),
                status: status.clone(),
                created_at: started_at,
                updated_at: updated_at.clone(),
                completed_at: if status == "completed" {
                    Some(updated_at)
                } else {
                    None
                },
                instances: vec![], // Instances will be updated separately
                chat_history: vec![],
                model: None,
                timeout_seconds: None,
                preserve_worktrees: None,
                winner_id: None,
                total_duration: None,
                total_files_changed: Some(0),
                total_lines_added: Some(0),
                total_lines_deleted: Some(0),
            };

            // Persist to session history using direct file operations
            use sha2::{Sha256, Digest};

            let mut hasher = Sha256::new();
            hasher.update(workspace_path.as_bytes());
            let hash = format!("{:x}", hasher.finalize())[..16].to_string();

            let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
            let sessions_file = home_dir
                .join(".ait42")
                .join("sessions")
                .join(format!("{}.json", hash));

            // Load existing sessions
            let mut sessions: Vec<crate::commands::session_history::WorktreeSession> = if sessions_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&sessions_file) {
                    serde_json::from_str(&content).unwrap_or_else(|_| Vec::new())
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            // Update or insert session
            if let Some(existing) = sessions.iter_mut().find(|s| s.id == session.id) {
                *existing = session;
            } else {
                sessions.push(session);
            }

            // Save back to file
            if let Ok(content) = serde_json::to_string_pretty(&sessions) {
                // Ensure directory exists
                if let Some(parent) = sessions_file.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Err(e) = std::fs::write(&sessions_file, content) {
                    tracing::error!("Failed to persist debate status: {}", e);
                } else {
                    tracing::info!("Persisted debate status update: {} - {}", debate_id_clone, status);
                }
            }
        });
    };

    // Set global timeout for entire debate (3 rounds * timeout_seconds)
    let total_timeout = tokio::time::Duration::from_secs(request.timeout_seconds * 3);
    let debate_start_time = tokio::time::Instant::now();

    // Round 1: Independent proposals
    update_status(1, "round_1".to_string());
    emit_debate_status(&app, &debate_id, 1, "round_1");

    if debate_start_time.elapsed() > total_timeout {
        tracing::error!("Debate {} timed out before Round 1", debate_id);
        update_status(1, "failed".to_string());
        return Err("Debate timed out".to_string());
    }

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
    update_status(2, "round_2".to_string());
    emit_debate_status(&app, &debate_id, 2, "round_2");

    if debate_start_time.elapsed() > total_timeout {
        tracing::error!("Debate {} timed out before Round 2", debate_id);
        update_status(2, "failed".to_string());
        return Err("Debate timed out".to_string());
    }

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
    update_status(3, "round_3".to_string());
    emit_debate_status(&app, &debate_id, 3, "round_3");

    if debate_start_time.elapsed() > total_timeout {
        tracing::error!("Debate {} timed out before Round 3", debate_id);
        update_status(3, "failed".to_string());
        return Err("Debate timed out".to_string());
    }

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
    update_status(3, "completed".to_string());
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

        // Send Claude Code command using echo -e for proper multiline handling
        // Use --permission-mode bypassPermissions to auto-approve changes and prevent interactive prompts
        let escaped_prompt = prompt
            .replace('\\', "\\\\")      // Escape backslashes first
            .replace('\n', "\\n")       // Escape newlines
            .replace('\'', "'\\''");    // Escape single quotes

        let claude_cmd = format!(
            "echo -e '{}' | claude --model {} code --print --permission-mode bypassPermissions",
            escaped_prompt,
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
        // Get project root for worktree cleanup
        let working_dir = state.working_dir.lock().await;
        let project_root = working_dir.clone();
        drop(working_dir);

        // Use project directory for worktrees (not home directory)
        let ait42_worktrees = project_root.join(".ait42").join(".worktrees");
        let debate_dir = format!("{}/debate-{}", ait42_worktrees.display(), &debate_id[..8]);

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

//
// ============================================================
// Claude Code Meta-Analysis (Ω-theory Self-Analysis)
// ============================================================
//

/// Task analysis request for Claude Code
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeAnalysisRequest {
    pub task: String,
    pub model: String,  // "sonnet", "haiku", "opus"
    pub timeout_seconds: u64,  // Default: 120s
}

/// Task analysis response from Claude Code
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeAnalysisResponse {
    pub analysis_id: String,
    pub complexity_class: String,  // "Logarithmic", "Linear", "Quadratic", "Exponential"
    pub recommended_subtasks: usize,
    pub recommended_instances: usize,
    pub confidence: f64,  // 0.0-1.0
    pub reasoning: String,
    pub raw_output: String,
    pub status: String,  // "completed", "failed", "timeout"
}

/// Analyze task using Claude Code itself (meta-analysis)
///
/// Instead of using Claude API directly, this launches Claude Code CLI
/// to analyze the task and provide decomposition recommendations.
#[tauri::command]
pub async fn analyze_task_with_claude_code(
    state: State<'_, AppState>,
    request: ClaudeCodeAnalysisRequest,
) -> Result<ClaudeCodeAnalysisResponse, String> {
    // Validation
    if request.task.trim().is_empty() {
        return Err("Task cannot be empty".to_string());
    }

    let valid_models = ["sonnet", "haiku", "opus"];
    if !valid_models.contains(&request.model.as_str()) {
        return Err(format!("Invalid model. Must be one of: {:?}", valid_models));
    }

    let analysis_id = uuid::Uuid::new_v4().to_string();

    tracing::info!(
        "Starting Claude Code meta-analysis: {} for task: {}",
        analysis_id,
        request.task.chars().take(50).collect::<String>()
    );

    let working_dir = state.working_dir.lock().await;
    let base_path = working_dir.clone();
    drop(working_dir);

    // Create analysis directory
    let analysis_dir = format!("{}/.worktrees/analysis-{}", base_path.display(), &analysis_id[..8]);
    std::fs::create_dir_all(&analysis_dir).map_err(|e| e.to_string())?;

    // Create tmux session
    let session_id = format!("claude-analysis-{}", &analysis_id[..8]);
    let output_log_path = format!("{}/.claude-analysis.log", analysis_dir);

    let tmux_output = Command::new("tmux")
        .arg("new-session")
        .arg("-d")
        .arg("-s")
        .arg(&session_id)
        .arg("-c")
        .arg(&base_path)
        .output()
        .map_err(|e| format!("Failed to create tmux session: {}", e))?;

    if !tmux_output.status.success() {
        let error = String::from_utf8_lossy(&tmux_output.stderr);
        return Err(format!("Failed to create tmux session: {}", error));
    }

    // Enable pipe-pane to capture output
    let _ = Command::new("tmux")
        .arg("pipe-pane")
        .arg("-t")
        .arg(&session_id)
        .arg("-o")
        .arg(format!("cat >> {}", output_log_path))
        .output();

    // Build analysis prompt
    let analysis_prompt = format!(
        r#"あなたはタスク分析の専門家です。以下のタスクを分析し、以下の形式で回答してください：

タスク: {}

以下の形式で回答してください：
COMPLEXITY_CLASS: [Logarithmic/Linear/Quadratic/Exponential]
SUBTASKS: [推奨サブタスク数（数値のみ）]
INSTANCES: [推奨並列実行インスタンス数（数値のみ）]
CONFIDENCE: [信頼度 0.0-1.0]
REASONING: [なぜこの複雑度クラスと分解数が適切か、詳細な理由を説明]

複雑度クラスの定義:
- Logarithmic (Ω(log n)): 単純なタスク（例: ランディングページ作成、単純なCRUD）
- Linear (Ω(n)): 標準的なタスク（例: REST API実装、認証システム）
- Quadratic (Ω(n²)): 複雑なタスク（例: ECサイト、データベース移行）
- Exponential (Ω(2ⁿ)): 非常に複雑なタスク（例: マイクロサービスアーキテクチャ）

サブタスク数の推奨範囲:
- Logarithmic: 2-3
- Linear: 3-5
- Quadratic: 5-8
- Exponential: 8-15

インスタンス数の推奨:
- Logarithmic: 2-3
- Linear: 2-5
- Quadratic: 3-8
- Exponential: 5-10"#,
        request.task
    );

    // Send Claude Code command using echo -e for proper multiline handling
    let escaped_prompt = analysis_prompt
        .replace('\\', "\\\\")      // Escape backslashes first
        .replace('\n', "\\n")       // Escape newlines
        .replace('\'', "'\\''");    // Escape single quotes

    let claude_cmd = format!(
        "echo -e '{}' | claude --model {} --print --permission-mode bypassPermissions",
        escaped_prompt,
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

    tracing::info!("Claude Code analysis started in session {}", session_id);

    // Wait for completion with timeout
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

                    return Err("Analysis timed out".to_string());
                }
            }
            _ => {
                // Session completed
                break;
            }
        }
    }

    // Read output
    let raw_output = tokio::fs::read_to_string(&output_log_path)
        .await
        .unwrap_or_else(|_| String::from("No output captured"));

    tracing::info!("Claude Code analysis completed, parsing output...");

    // Parse output
    let response = parse_analysis_output(&raw_output, &analysis_id)?;

    // Cleanup
    let _ = std::fs::remove_dir_all(&analysis_dir);

    tracing::info!(
        "Analysis complete: {} ({} subtasks, {} instances, confidence: {:.2})",
        response.complexity_class,
        response.recommended_subtasks,
        response.recommended_instances,
        response.confidence
    );

    Ok(response)
}

/// Parse Claude Code analysis output
fn parse_analysis_output(output: &str, analysis_id: &str) -> Result<ClaudeCodeAnalysisResponse, String> {
    let mut complexity_class = String::new();
    let mut subtasks = 0;
    let mut instances = 0;
    let mut confidence = 0.0;
    let mut reasoning = String::new();

    // Parse structured output
    for line in output.lines() {
        let line = line.trim();

        if line.starts_with("COMPLEXITY_CLASS:") {
            complexity_class = line
                .strip_prefix("COMPLEXITY_CLASS:")
                .unwrap_or("")
                .trim()
                .to_string();
        } else if line.starts_with("SUBTASKS:") {
            if let Ok(val) = line
                .strip_prefix("SUBTASKS:")
                .unwrap_or("0")
                .trim()
                .parse::<usize>()
            {
                subtasks = val;
            }
        } else if line.starts_with("INSTANCES:") {
            if let Ok(val) = line
                .strip_prefix("INSTANCES:")
                .unwrap_or("0")
                .trim()
                .parse::<usize>()
            {
                instances = val;
            }
        } else if line.starts_with("CONFIDENCE:") {
            if let Ok(val) = line
                .strip_prefix("CONFIDENCE:")
                .unwrap_or("0.0")
                .trim()
                .parse::<f64>()
            {
                confidence = val;
            }
        } else if line.starts_with("REASONING:") {
            reasoning = line
                .strip_prefix("REASONING:")
                .unwrap_or("")
                .trim()
                .to_string();

            // Collect multi-line reasoning
            let reasoning_start = output.find("REASONING:").unwrap_or(0);
            reasoning = output[reasoning_start..]
                .strip_prefix("REASONING:")
                .unwrap_or("")
                .trim()
                .to_string();
        }
    }

    // Validation
    let valid_classes = ["Logarithmic", "Linear", "Quadratic", "Exponential"];
    if !valid_classes.contains(&complexity_class.as_str()) {
        // Fallback: try to infer from output
        complexity_class = infer_complexity_class(output);
    }

    if subtasks == 0 {
        subtasks = infer_subtasks(output, &complexity_class);
    }

    if instances == 0 {
        instances = calculate_instances_from_complexity(&complexity_class, subtasks);
    }

    if confidence == 0.0 {
        confidence = 0.7; // Default confidence
    }

    if reasoning.is_empty() {
        reasoning = extract_reasoning(output);
    }

    Ok(ClaudeCodeAnalysisResponse {
        analysis_id: analysis_id.to_string(),
        complexity_class,
        recommended_subtasks: subtasks,
        recommended_instances: instances,
        confidence,
        reasoning,
        raw_output: output.to_string(),
        status: "completed".to_string(),
    })
}

/// Infer complexity class from unstructured output
fn infer_complexity_class(output: &str) -> String {
    let lower = output.to_lowercase();

    if lower.contains("exponential") || lower.contains("指数") {
        "Exponential".to_string()
    } else if lower.contains("quadratic") || lower.contains("二乗") || lower.contains("o(n²)") {
        "Quadratic".to_string()
    } else if lower.contains("linear") || lower.contains("線形") || lower.contains("o(n)") {
        "Linear".to_string()
    } else if lower.contains("logarithmic") || lower.contains("対数") || lower.contains("o(log") {
        "Logarithmic".to_string()
    } else {
        "Linear".to_string() // Default
    }
}

/// Infer subtasks from output
fn infer_subtasks(output: &str, complexity_class: &str) -> usize {
    // Try to find numbers in output
    let numbers: Vec<usize> = output
        .split_whitespace()
        .filter_map(|word| word.parse::<usize>().ok())
        .filter(|&n| n >= 2 && n <= 20)
        .collect();

    if let Some(&first) = numbers.first() {
        first
    } else {
        // Fallback based on complexity class
        match complexity_class {
            "Logarithmic" => 3,
            "Linear" => 4,
            "Quadratic" => 6,
            "Exponential" => 10,
            _ => 4,
        }
    }
}

/// Calculate instances from complexity class and subtasks
fn calculate_instances_from_complexity(complexity_class: &str, subtasks: usize) -> usize {
    match complexity_class {
        "Logarithmic" => std::cmp::max(2, std::cmp::min(3, subtasks)),
        "Linear" => std::cmp::max(2, std::cmp::min(5, subtasks / 3)),
        "Quadratic" => std::cmp::max(3, std::cmp::min(8, subtasks / 2)),
        "Exponential" => std::cmp::max(5, std::cmp::min(10, subtasks)),
        _ => 3,
    }
}

/// Extract reasoning from unstructured output
fn extract_reasoning(output: &str) -> String {
    // Take first meaningful paragraph
    let lines: Vec<&str> = output
        .lines()
        .filter(|line| !line.trim().is_empty() && line.len() > 20)
        .collect();

    if lines.is_empty() {
        "Claude Code分析による推奨".to_string()
    } else {
        lines.join(" ").chars().take(200).collect()
    }
}
