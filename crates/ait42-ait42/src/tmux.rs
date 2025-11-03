//! Tmux session manager for running AI agents in isolated sessions

use crate::error::{AIT42Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::process::Command;
use tracing::{debug, error, info, warn};

/// Tmux session status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    Running,
    Completed,
    Failed(String),
}

/// Tmux session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxSession {
    pub id: String,
    pub agent_name: String,
    pub task: String,
    pub status: SessionStatus,
    pub start_time: SystemTime,
    pub output: Vec<String>,
}

/// Tmux session manager
#[derive(Debug)]
pub struct TmuxManager {
    script_path: PathBuf,
    _parallel_script_path: PathBuf, // Reserved for future parallel execution support
    ait42_root: PathBuf,
}

impl TmuxManager {
    /// Create a new tmux manager
    pub fn new(ait42_root: &Path) -> Self {
        let script_path = ait42_root.join("scripts/tmux-single-agent.sh");
        let parallel_script_path = ait42_root.join("scripts/tmux-parallel-agents.sh");

        Self {
            script_path,
            _parallel_script_path: parallel_script_path,
            ait42_root: ait42_root.to_path_buf(),
        }
    }

    /// Check if tmux is available
    pub async fn is_available() -> bool {
        Command::new("tmux")
            .arg("-V")
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Start a single agent in tmux session
    pub async fn start_agent(&self, agent: &str, task: &str) -> Result<String> {
        info!("Starting agent {} in tmux session", agent);

        if !self.script_path.exists() {
            return Err(AIT42Error::ConfigError(format!(
                "Tmux script not found: {}",
                self.script_path.display()
            )));
        }

        let output = Command::new(&self.script_path)
            .arg(agent)
            .arg(task)
            .current_dir(&self.ait42_root)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to start agent: {}", stderr);
            return Err(AIT42Error::TmuxError(stderr.to_string()));
        }

        // Extract session ID from output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let session_id = self.extract_session_id(&stdout)?;

        info!("Agent {} started in session: {}", agent, session_id);
        Ok(session_id)
    }

    /// Start multiple agents in parallel
    pub async fn start_parallel(&self, agents: Vec<(&str, &str)>) -> Result<Vec<String>> {
        info!("Starting {} agents in parallel", agents.len());

        let mut session_ids = Vec::new();

        for (agent, task) in agents {
            match self.start_agent(agent, task).await {
                Ok(session_id) => session_ids.push(session_id),
                Err(e) => {
                    error!("Failed to start agent {}: {}", agent, e);
                    // Continue with other agents
                }
            }
        }

        if session_ids.is_empty() {
            return Err(AIT42Error::ExecutionFailed("Failed to start any agents".to_string()));
        }

        Ok(session_ids)
    }

    /// List all AIT42 tmux sessions
    pub async fn list_sessions(&self) -> Result<Vec<TmuxSession>> {
        debug!("Listing tmux sessions");

        let output = Command::new("tmux")
            .args(["list-sessions", "-F", "#{session_name}:#{session_created}"])
            .output()
            .await?;

        if !output.status.success() {
            // No sessions might exist, which is fine
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut sessions = Vec::new();

        for line in stdout.lines() {
            if line.contains("ait42-") {
                if let Some(session) = self.parse_session_line(line).await {
                    sessions.push(session);
                }
            }
        }

        Ok(sessions)
    }

    /// Get session output
    pub async fn get_output(&self, session_id: &str) -> Result<Vec<String>> {
        debug!("Capturing output from session: {}", session_id);

        let output = Command::new("tmux")
            .args(["capture-pane", "-t", session_id, "-p", "-S", "-"])
            .output()
            .await?;

        if !output.status.success() {
            return Err(AIT42Error::SessionNotFound(session_id.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    /// Kill a session
    pub async fn kill_session(&self, session_id: &str) -> Result<()> {
        info!("Killing session: {}", session_id);

        let output = Command::new("tmux")
            .args(["kill-session", "-t", session_id])
            .output()
            .await?;

        if !output.status.success() {
            return Err(AIT42Error::SessionNotFound(session_id.to_string()));
        }

        Ok(())
    }

    /// Attach to a session
    pub async fn attach_session(&self, session_id: &str) -> Result<()> {
        info!("Attaching to session: {}", session_id);

        let status = Command::new("tmux")
            .args(["attach", "-t", session_id])
            .status()
            .await?;

        if !status.success() {
            return Err(AIT42Error::SessionNotFound(session_id.to_string()));
        }

        Ok(())
    }

    /// Check if session is still running
    pub async fn is_session_alive(&self, session_id: &str) -> bool {
        Command::new("tmux")
            .args(["has-session", "-t", session_id])
            .status()
            .await
            .map(|status| status.success())
            .unwrap_or(false)
    }

    /// Wait for session to complete
    pub async fn wait_for_completion(&self, session_id: &str, timeout_secs: u64) -> Result<()> {
        use tokio::time::{sleep, Duration};

        let start = SystemTime::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            if !self.is_session_alive(session_id).await {
                info!("Session {} completed", session_id);
                return Ok(());
            }

            if start.elapsed().unwrap_or(Duration::from_secs(0)) > timeout {
                warn!("Session {} timed out after {} seconds", session_id, timeout_secs);
                return Err(AIT42Error::SessionTimeout(session_id.to_string()));
            }

            sleep(Duration::from_secs(1)).await;
        }
    }

    /// Extract session ID from script output
    fn extract_session_id(&self, output: &str) -> Result<String> {
        for line in output.lines() {
            if line.contains("ait42-") {
                // Look for session name pattern: ait42-{agent}-{timestamp}
                if let Some(session) = line
                    .split_whitespace()
                    .find(|word| word.starts_with("ait42-"))
                {
                    return Ok(session.to_string());
                }
            }
        }

        // Try to extract from "Session: " line
        for line in output.lines() {
            if line.starts_with("Session:") {
                if let Some(session) = line.split(':').nth(1) {
                    return Ok(session.trim().to_string());
                }
            }
        }

        Err(AIT42Error::TmuxError("Could not extract session ID from output".to_string()))
    }

    /// Parse session line from tmux list-sessions
    async fn parse_session_line(&self, line: &str) -> Option<TmuxSession> {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 2 {
            return None;
        }

        let session_id = parts[0].to_string();

        // Parse agent name from session ID: ait42-{agent}-{timestamp}
        let agent_name = session_id
            .strip_prefix("ait42-")?
            .rsplit_once('-')?
            .0
            .to_string();

        // Parse timestamp
        let created_timestamp = parts[1].parse::<u64>().ok()?;
        let start_time = UNIX_EPOCH + std::time::Duration::from_secs(created_timestamp);

        // Get output to determine status
        let output = self.get_output(&session_id).await.ok()?;
        let status = self.determine_status(&output);

        Some(TmuxSession {
            id: session_id,
            agent_name,
            task: String::new(), // Task not stored in session metadata
            status,
            start_time,
            output,
        })
    }

    /// Determine session status from output
    fn determine_status(&self, output: &[String]) -> SessionStatus {
        // Look for completion markers
        for line in output.iter().rev().take(10) {
            if line.contains("✓ Task completed") || line.contains("Task completed") {
                return SessionStatus::Completed;
            }
            if line.contains("Error:") || line.contains("Failed:") {
                return SessionStatus::Failed(line.clone());
            }
        }

        SessionStatus::Running
    }

    /// Send keys to a session
    pub async fn send_keys(&self, session_id: &str, keys: &str) -> Result<()> {
        debug!("Sending keys to session {}: {}", session_id, keys);

        let output = Command::new("tmux")
            .args(["send-keys", "-t", session_id, keys, "C-m"])
            .output()
            .await?;

        if !output.status.success() {
            return Err(AIT42Error::TmuxError("Failed to send keys to session".to_string()));
        }

        Ok(())
    }

    /// Cleanup old completed sessions
    pub async fn cleanup_old_sessions(&self, max_age_secs: u64) -> Result<usize> {
        let sessions = self.list_sessions().await?;
        let mut cleaned = 0;

        let now = SystemTime::now();
        let max_age = std::time::Duration::from_secs(max_age_secs);

        for session in sessions {
            if session.status != SessionStatus::Running {
                if let Ok(age) = now.duration_since(session.start_time) {
                    if age > max_age && self.kill_session(&session.id).await.is_ok() {
                        cleaned += 1;
                    }
                }
            }
        }

        if cleaned > 0 {
            info!("Cleaned up {} old sessions", cleaned);
        }

        Ok(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_is_available() {
        // This test will fail if tmux is not installed
        let available = TmuxManager::is_available().await;
        // Just ensure it doesn't panic
        let _ = available;
    }

    #[tokio::test]
    async fn test_extract_session_id() {
        let manager = TmuxManager::new(Path::new("/tmp"));

        let output = r#"
Creating tmux session...
✅ Session created: ait42-backend-developer-1234567890
"#;

        let session_id = manager.extract_session_id(output).unwrap();
        assert_eq!(session_id, "ait42-backend-developer-1234567890");
    }

    #[test]
    fn test_determine_status() {
        let manager = TmuxManager::new(Path::new("/tmp"));

        let output = vec![
            "Starting task...".to_string(),
            "Processing...".to_string(),
            "✓ Task completed".to_string(),
        ];

        assert_eq!(manager.determine_status(&output), SessionStatus::Completed);

        let error_output = vec![
            "Starting task...".to_string(),
            "Error: Something went wrong".to_string(),
        ];

        match manager.determine_status(&error_output) {
            SessionStatus::Failed(_) => (),
            _ => panic!("Expected Failed status"),
        }
    }
}
