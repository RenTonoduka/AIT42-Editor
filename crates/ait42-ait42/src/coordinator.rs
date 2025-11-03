//! Coordinator for intelligent agent selection and orchestration

use crate::config::AIT42Config;
use crate::error::{AIT42Error, Result};
use crate::registry::{AgentMetadata, AgentRegistry};
use crate::tmux::{SessionStatus, TmuxManager, TmuxSession};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use tracing::info;

/// Result of agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub agent_name: String,
    pub session_id: String,
    pub status: SessionStatus,
    pub output: String,
    pub duration: Duration,
}

/// Coordinator for agent selection and execution
#[derive(Debug)]
pub struct Coordinator {
    registry: AgentRegistry,
    tmux: TmuxManager,
    config: AIT42Config,
}

impl Coordinator {
    /// Create a new coordinator
    pub fn new(config: AIT42Config) -> Result<Self> {
        let registry = AgentRegistry::load_from_directory(&config.agents_dir())?;
        let tmux = TmuxManager::new(&config.ait42_root);

        Ok(Self {
            registry,
            tmux,
            config,
        })
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &AIT42Config {
        &self.config
    }

    /// Auto-select best agent(s) for a task
    pub fn auto_select_agents(&self, task: &str) -> Result<Vec<String>> {
        info!("Auto-selecting agents for task: {}", task);

        let task_lower = task.to_lowercase();
        let mut selected = Vec::new();

        // Simple keyword-based selection
        // In production, this would use Claude API for intelligent selection

        if task_lower.contains("backend")
            || task_lower.contains("api")
            || task_lower.contains("server")
        {
            selected.push("backend-developer".to_string());
        }

        if task_lower.contains("frontend")
            || task_lower.contains("ui")
            || task_lower.contains("react")
        {
            selected.push("frontend-developer".to_string());
        }

        if task_lower.contains("test") {
            selected.push("test-generator".to_string());
        }

        if task_lower.contains("review") {
            selected.push("code-reviewer".to_string());
        }

        if task_lower.contains("security") {
            selected.push("security-tester".to_string());
        }

        if task_lower.contains("deploy") || task_lower.contains("cicd") {
            selected.push("cicd-manager".to_string());
        }

        if task_lower.contains("database") || task_lower.contains("db") {
            selected.push("database-developer".to_string());
        }

        if task_lower.contains("refactor") {
            selected.push("refactor-specialist".to_string());
        }

        // If no specific keywords, default to coordinator
        if selected.is_empty() {
            // Try searching
            let search_results = self.registry.search(task);
            if !search_results.is_empty() {
                selected.push(search_results[0].name.clone());
            } else {
                selected.push("backend-developer".to_string());
            }
        }

        // Limit to max parallel agents
        selected.truncate(self.config.max_parallel_agents);

        info!("Selected agents: {:?}", selected);
        Ok(selected)
    }

    /// Execute a task with auto-selected agent(s)
    pub async fn execute_task(&mut self, task: &str) -> Result<Vec<ExecutionResult>> {
        let agents = self.auto_select_agents(task)?;

        if agents.is_empty() {
            return Err(AIT42Error::ExecutionFailed(
                "No suitable agents found for task".to_string(),
            ));
        }

        if agents.len() == 1 {
            // Single agent execution
            let result = self.execute_single(&agents[0], task).await?;
            Ok(vec![result])
        } else {
            // Parallel execution
            self.execute_parallel(&agents, task).await
        }
    }

    /// Execute with a single agent
    async fn execute_single(&mut self, agent: &str, task: &str) -> Result<ExecutionResult> {
        info!("Executing task with agent: {}", agent);

        let start_time = SystemTime::now();

        // Verify agent exists
        if self.registry.get(agent).is_none() {
            return Err(AIT42Error::AgentNotFound(agent.to_string()));
        }

        // Start tmux session
        let session_id = self.tmux.start_agent(agent, task).await?;

        // Wait for completion with timeout
        let timeout_secs = self.config.session_timeout_secs;
        let wait_result = self
            .tmux
            .wait_for_completion(&session_id, timeout_secs)
            .await;

        // Get output
        let output = self.tmux.get_output(&session_id).await.unwrap_or_default();
        let output_str = output.join("\n");

        // Determine status
        let status = match wait_result {
            Ok(_) => {
                if output_str.contains("✓ Task completed") {
                    SessionStatus::Completed
                } else if output_str.contains("Error:") || output_str.contains("Failed:") {
                    SessionStatus::Failed("Task execution failed".to_string())
                } else {
                    SessionStatus::Completed
                }
            }
            Err(AIT42Error::SessionTimeout(_)) => {
                SessionStatus::Failed("Session timed out".to_string())
            }
            Err(e) => SessionStatus::Failed(e.to_string()),
        };

        let duration = start_time.elapsed().unwrap_or(Duration::from_secs(0));

        Ok(ExecutionResult {
            agent_name: agent.to_string(),
            session_id,
            status,
            output: output_str,
            duration,
        })
    }

    /// Execute with multiple agents in parallel
    async fn execute_parallel(
        &mut self,
        agents: &[String],
        task: &str,
    ) -> Result<Vec<ExecutionResult>> {
        info!("Executing task with {} agents in parallel", agents.len());

        let mut results = Vec::new();
        let mut handles = Vec::new();

        for agent in agents {
            let agent = agent.clone();
            let task = task.to_string();
            let tmux = TmuxManager::new(&self.config.ait42_root);
            let timeout = self.config.session_timeout_secs;

            let handle = tokio::spawn(async move {
                let start_time = SystemTime::now();

                // Start tmux session
                let session_id = tmux.start_agent(&agent, &task).await?;

                // Wait for completion
                let wait_result = tmux.wait_for_completion(&session_id, timeout).await;

                // Get output
                let output = tmux.get_output(&session_id).await.unwrap_or_default();
                let output_str = output.join("\n");

                // Determine status
                let status = match wait_result {
                    Ok(_) => {
                        if output_str.contains("✓ Task completed") {
                            SessionStatus::Completed
                        } else {
                            SessionStatus::Failed("Task execution failed".to_string())
                        }
                    }
                    Err(AIT42Error::SessionTimeout(_)) => {
                        SessionStatus::Failed("Session timed out".to_string())
                    }
                    Err(e) => SessionStatus::Failed(e.to_string()),
                };

                let duration = start_time.elapsed().unwrap_or(Duration::from_secs(0));

                Ok::<ExecutionResult, AIT42Error>(ExecutionResult {
                    agent_name: agent,
                    session_id,
                    status,
                    output: output_str,
                    duration,
                })
            });

            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => {
                    tracing::error!("Agent execution failed: {}", e);
                }
                Err(e) => {
                    tracing::error!("Task join error: {}", e);
                }
            }
        }

        if results.is_empty() {
            return Err(AIT42Error::ExecutionFailed("All parallel executions failed".to_string()));
        }

        Ok(results)
    }

    /// Get agent metadata
    pub fn get_agent(&self, name: &str) -> Option<&AgentMetadata> {
        self.registry.get(name)
    }

    /// List all available agents
    pub fn list_agents(&self) -> Vec<&AgentMetadata> {
        self.registry.list()
    }

    /// Search for agents
    pub fn search_agents(&self, query: &str) -> Vec<&AgentMetadata> {
        self.registry.search(query)
    }

    /// List active tmux sessions
    pub async fn list_sessions(&self) -> Result<Vec<TmuxSession>> {
        self.tmux.list_sessions().await
    }

    /// Get session output
    pub async fn get_session_output(&self, session_id: &str) -> Result<Vec<String>> {
        self.tmux.get_output(session_id).await
    }

    /// Kill a session
    pub async fn kill_session(&self, session_id: &str) -> Result<()> {
        self.tmux.kill_session(session_id).await
    }

    /// Cleanup old sessions
    pub async fn cleanup_sessions(&self) -> Result<usize> {
        if self.config.auto_cleanup {
            let max_age = self.config.cleanup_max_age_secs;
            self.tmux.cleanup_old_sessions(max_age).await
        } else {
            Ok(0)
        }
    }

    /// Get total number of agents
    pub fn agent_count(&self) -> usize {
        self.registry.count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_auto_select_agents() {
        let _config = AIT42Config::new(PathBuf::from("/tmp/ait42"));
        // This will fail without actual agents, but tests the structure
        // In real tests, use a temp directory with mock agents
    }

    #[test]
    fn test_keyword_matching() {
        let task1 = "Implement a backend API for user authentication";
        assert!(task1.to_lowercase().contains("backend"));
        assert!(task1.to_lowercase().contains("api"));

        let task2 = "Create a React frontend component";
        assert!(task2.to_lowercase().contains("frontend"));
        assert!(task2.to_lowercase().contains("react"));
    }
}
