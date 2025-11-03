//! AIT42 Agent Integration
//!
//! Integrates with the AIT42 agent system for AI-powered development tasks.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tokio::process::Command;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Agent not found: {0}")]
    NotFound(String),

    #[error("Agent execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Tmux session error: {0}")]
    TmuxError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AgentError>;

/// Represents an AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub description: String,
    pub category: String,
    pub file_path: PathBuf,
}

/// Manages AIT42 agents
pub struct AgentManager {
    agents_dir: PathBuf,
    agents: Vec<Agent>,
}

impl AgentManager {
    /// Create a new agent manager
    pub fn new(agents_dir: PathBuf) -> Self {
        Self {
            agents_dir,
            agents: Vec::new(),
        }
    }

    /// Load all available agents
    pub async fn load_agents(&mut self) -> Result<()> {
        // TODO: Scan agents directory and parse agent files
        Ok(())
    }

    /// Get list of all agents
    pub fn list_agents(&self) -> &[Agent] {
        &self.agents
    }

    /// Run an agent with a task
    pub async fn run_agent(&self, agent_name: &str, task: &str) -> Result<TmuxSession> {
        let script_path = self.agents_dir
            .parent()
            .unwrap()
            .join("scripts/tmux-single-agent.sh");

        let output = Command::new(&script_path)
            .arg(agent_name)
            .arg(task)
            .output()
            .await?;

        if !output.status.success() {
            return Err(AgentError::ExecutionFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Parse session name from output
        let session_name = String::from_utf8_lossy(&output.stdout)
            .lines()
            .find(|line| line.contains("ait42-"))
            .unwrap_or("unknown")
            .to_string();

        Ok(TmuxSession {
            name: session_name,
            agent: agent_name.to_string(),
            status: SessionStatus::Running,
        })
    }
}

/// Tmux session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxSession {
    pub name: String,
    pub agent: String,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Running,
    Completed,
    Failed,
}

/// Tmux session manager
pub struct TmuxManager;

impl TmuxManager {
    /// List all AIT42 tmux sessions
    pub async fn list_sessions() -> Result<Vec<TmuxSession>> {
        let output = Command::new("tmux")
            .args(["list-sessions", "-F", "#{session_name}"])
            .output()
            .await?;

        let sessions: Vec<TmuxSession> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| line.contains("ait42-"))
            .map(|name| TmuxSession {
                name: name.to_string(),
                agent: "unknown".to_string(),
                status: SessionStatus::Running,
            })
            .collect();

        Ok(sessions)
    }

    /// Capture session output
    pub async fn capture_output(session_name: &str) -> Result<String> {
        let output = Command::new("tmux")
            .args(["capture-pane", "-t", session_name, "-p"])
            .output()
            .await?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Kill a session
    pub async fn kill_session(session_name: &str) -> Result<()> {
        Command::new("tmux")
            .args(["kill-session", "-t", session_name])
            .output()
            .await?;

        Ok(())
    }
}
