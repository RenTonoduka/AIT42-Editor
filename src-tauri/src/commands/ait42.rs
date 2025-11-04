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
