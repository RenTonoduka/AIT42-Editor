//! Command palette integration for agent operations

use crate::coordinator::ExecutionResult;
use crate::error::Result;
use crate::executor::{AgentExecutor, ExecutionMode};
use serde::{Deserialize, Serialize};

/// Result of command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandResult {
    /// Execution results
    Executed(Vec<ExecutionResult>),
    /// Agent list
    AgentList(Vec<AgentInfo>),
    /// Session list
    SessionList(Vec<SessionInfo>),
    /// Session output
    SessionOutput(String),
    /// Success message
    Success(String),
}

/// Agent information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub description: String,
    pub category: String,
}

/// Session information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub agent_name: String,
    pub status: String,
    pub age_secs: u64,
}

/// Agent command types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentCommand {
    /// Run a specific agent with a task
    RunAgent { agent: String, task: String },

    /// Run coordinator (auto-select agent)
    RunCoordinator { task: String },

    /// Run multiple agents in parallel
    RunParallel { agents: Vec<String>, task: String },

    /// Run multiple agents sequentially
    RunSequential { agents: Vec<String>, task: String },

    /// List all available agents
    ListAgents,

    /// Search for agents
    SearchAgents { query: String },

    /// List active tmux sessions
    ListSessions,

    /// View session output
    ViewSessionOutput { session_id: String },

    /// Kill a session
    KillSession { session_id: String },

    /// Attach to a session
    AttachSession { session_id: String },

    /// Cleanup old sessions
    CleanupSessions,

    /// Get agent details
    GetAgentDetails { agent: String },
}

impl AgentCommand {
    /// Execute the command
    pub async fn execute(&self, executor: &mut AgentExecutor) -> Result<CommandResult> {
        match self {
            AgentCommand::RunAgent { agent, task } => {
                let results = executor
                    .execute(ExecutionMode::Single(agent.clone()), task)
                    .await?;
                Ok(CommandResult::Executed(results))
            }

            AgentCommand::RunCoordinator { task } => {
                let results = executor.execute(ExecutionMode::Coordinated, task).await?;
                Ok(CommandResult::Executed(results))
            }

            AgentCommand::RunParallel { agents, task } => {
                let results = executor
                    .execute(ExecutionMode::Parallel(agents.clone()), task)
                    .await?;
                Ok(CommandResult::Executed(results))
            }

            AgentCommand::RunSequential { agents, task } => {
                let results = executor
                    .execute(ExecutionMode::Sequential(agents.clone()), task)
                    .await?;
                Ok(CommandResult::Executed(results))
            }

            AgentCommand::ListAgents => {
                let agents = executor
                    .coordinator()
                    .list_agents()
                    .into_iter()
                    .map(|agent| AgentInfo {
                        name: agent.name.clone(),
                        description: agent.description.clone(),
                        category: format!("{:?}", agent.category),
                    })
                    .collect();
                Ok(CommandResult::AgentList(agents))
            }

            AgentCommand::SearchAgents { query } => {
                let agents = executor
                    .coordinator()
                    .search_agents(query)
                    .into_iter()
                    .map(|agent| AgentInfo {
                        name: agent.name.clone(),
                        description: agent.description.clone(),
                        category: format!("{:?}", agent.category),
                    })
                    .collect();
                Ok(CommandResult::AgentList(agents))
            }

            AgentCommand::ListSessions => {
                let sessions = executor.coordinator().list_sessions().await?;
                let session_info = sessions
                    .into_iter()
                    .map(|session| {
                        let age = session
                            .start_time
                            .elapsed()
                            .unwrap_or_default()
                            .as_secs();
                        SessionInfo {
                            id: session.id,
                            agent_name: session.agent_name,
                            status: format!("{:?}", session.status),
                            age_secs: age,
                        }
                    })
                    .collect();
                Ok(CommandResult::SessionList(session_info))
            }

            AgentCommand::ViewSessionOutput { session_id } => {
                let output = executor
                    .coordinator()
                    .get_session_output(session_id)
                    .await?;
                Ok(CommandResult::SessionOutput(output.join("\n")))
            }

            AgentCommand::KillSession { session_id } => {
                executor.coordinator().kill_session(session_id).await?;
                Ok(CommandResult::Success(format!(
                    "Session {} killed",
                    session_id
                )))
            }

            AgentCommand::AttachSession { session_id } => {
                // This command should be handled specially as it requires TTY
                Ok(CommandResult::Success(format!(
                    "To attach, run: tmux attach -t {}",
                    session_id
                )))
            }

            AgentCommand::CleanupSessions => {
                let count = executor.coordinator().cleanup_sessions().await?;
                Ok(CommandResult::Success(format!(
                    "Cleaned up {} old sessions",
                    count
                )))
            }

            AgentCommand::GetAgentDetails { agent } => {
                let metadata = executor
                    .coordinator()
                    .get_agent(agent)
                    .ok_or_else(|| crate::error::AIT42Error::AgentNotFound(agent.clone()))?;

                let info = AgentInfo {
                    name: metadata.name.clone(),
                    description: metadata.description.clone(),
                    category: format!("{:?}", metadata.category),
                };

                Ok(CommandResult::AgentList(vec![info]))
            }
        }
    }

    /// Get command description
    pub fn description(&self) -> String {
        match self {
            AgentCommand::RunAgent { agent, .. } => format!("Run agent: {}", agent),
            AgentCommand::RunCoordinator { .. } => "Run with auto-selected agent".to_string(),
            AgentCommand::RunParallel { agents, .. } => {
                format!("Run {} agents in parallel", agents.len())
            }
            AgentCommand::RunSequential { agents, .. } => {
                format!("Run {} agents sequentially", agents.len())
            }
            AgentCommand::ListAgents => "List all agents".to_string(),
            AgentCommand::SearchAgents { query } => format!("Search agents: {}", query),
            AgentCommand::ListSessions => "List active sessions".to_string(),
            AgentCommand::ViewSessionOutput { session_id } => {
                format!("View output: {}", session_id)
            }
            AgentCommand::KillSession { session_id } => format!("Kill session: {}", session_id),
            AgentCommand::AttachSession { session_id } => {
                format!("Attach to session: {}", session_id)
            }
            AgentCommand::CleanupSessions => "Cleanup old sessions".to_string(),
            AgentCommand::GetAgentDetails { agent } => format!("Get details: {}", agent),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_description() {
        let cmd = AgentCommand::RunAgent {
            agent: "backend-developer".to_string(),
            task: "test".to_string(),
        };
        assert_eq!(cmd.description(), "Run agent: backend-developer");

        let cmd = AgentCommand::ListAgents;
        assert_eq!(cmd.description(), "List all agents");

        let cmd = AgentCommand::RunParallel {
            agents: vec!["agent1".to_string(), "agent2".to_string()],
            task: "test".to_string(),
        };
        assert_eq!(cmd.description(), "Run 2 agents in parallel");
    }

    #[test]
    fn test_command_serialization() {
        let cmd = AgentCommand::RunAgent {
            agent: "test-agent".to_string(),
            task: "test task".to_string(),
        };

        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: AgentCommand = serde_json::from_str(&json).unwrap();

        match deserialized {
            AgentCommand::RunAgent { agent, task } => {
                assert_eq!(agent, "test-agent");
                assert_eq!(task, "test task");
            }
            _ => panic!("Wrong variant"),
        }
    }
}
