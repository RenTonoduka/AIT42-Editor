//! Agent executor with different execution modes

use crate::coordinator::{Coordinator, ExecutionResult};
use crate::error::{AIT42Error, Result};
use tracing::{info, warn};

/// Agent execution mode
#[derive(Debug, Clone)]
pub enum ExecutionMode {
    /// Execute a single agent
    Single(String),
    /// Execute multiple agents in parallel
    Parallel(Vec<String>),
    /// Execute multiple agents sequentially
    Sequential(Vec<String>),
    /// Let coordinator auto-select agents
    Coordinated,
}

/// Agent executor
pub struct AgentExecutor {
    coordinator: Coordinator,
}

impl AgentExecutor {
    /// Create a new executor
    pub fn new(coordinator: Coordinator) -> Self {
        Self { coordinator }
    }

    /// Execute task with specified mode
    pub async fn execute(
        &mut self,
        mode: ExecutionMode,
        task: &str,
    ) -> Result<Vec<ExecutionResult>> {
        match mode {
            ExecutionMode::Single(agent) => {
                let result = self.execute_single(&agent, task).await?;
                Ok(vec![result])
            }
            ExecutionMode::Parallel(agents) => self.execute_parallel(&agents, task).await,
            ExecutionMode::Sequential(agents) => self.execute_sequential(&agents, task).await,
            ExecutionMode::Coordinated => self.coordinator.execute_task(task).await,
        }
    }

    /// Execute with a single agent
    pub async fn execute_single(&mut self, agent: &str, task: &str) -> Result<ExecutionResult> {
        info!("Executing single agent: {}", agent);

        // Verify agent exists
        if self.coordinator.get_agent(agent).is_none() {
            return Err(AIT42Error::AgentNotFound(agent.to_string()));
        }

        // Use coordinator to execute
        let mut results = self.coordinator.execute_task(task).await?;

        if results.is_empty() {
            return Err(AIT42Error::ExecutionFailed("No execution results returned".to_string()));
        }

        Ok(results.remove(0))
    }

    /// Execute multiple agents in parallel
    pub async fn execute_parallel(
        &mut self,
        agents: &[String],
        task: &str,
    ) -> Result<Vec<ExecutionResult>> {
        info!("Executing {} agents in parallel", agents.len());

        // Verify all agents exist
        for agent in agents {
            if self.coordinator.get_agent(agent).is_none() {
                return Err(AIT42Error::AgentNotFound(agent.to_string()));
            }
        }

        // Create tasks for each agent
        let mut handles = Vec::new();

        for _agent in agents {
            let task = task.to_string();
            let mut coordinator = self.coordinator.clone_for_parallel()?;

            let handle = tokio::spawn(async move { coordinator.execute_task(&task).await });

            handles.push(handle);
        }

        // Collect results
        let mut all_results = Vec::new();
        let mut errors = Vec::new();

        for handle in handles {
            match handle.await {
                Ok(Ok(mut results)) => {
                    all_results.append(&mut results);
                }
                Ok(Err(e)) => {
                    warn!("Parallel execution failed: {}", e);
                    errors.push(e);
                }
                Err(e) => {
                    warn!("Task join error: {}", e);
                    errors.push(AIT42Error::ExecutionFailed(e.to_string()));
                }
            }
        }

        if all_results.is_empty() && !errors.is_empty() {
            return Err(AIT42Error::multiple(errors));
        }

        Ok(all_results)
    }

    /// Execute multiple agents sequentially
    pub async fn execute_sequential(
        &mut self,
        agents: &[String],
        task: &str,
    ) -> Result<Vec<ExecutionResult>> {
        info!("Executing {} agents sequentially", agents.len());

        let mut results = Vec::new();
        let mut context = task.to_string();

        for (idx, agent) in agents.iter().enumerate() {
            info!("Step {}/{}: Executing {}", idx + 1, agents.len(), agent);

            // Verify agent exists
            if self.coordinator.get_agent(agent).is_none() {
                return Err(AIT42Error::AgentNotFound(agent.to_string()));
            }

            // Execute agent
            match self.execute_single(agent, &context).await {
                Ok(result) => {
                    // Use output as context for next agent
                    if !result.output.is_empty() {
                        context = format!(
                            "{}\n\nPrevious step output from {}:\n{}",
                            task, agent, result.output
                        );
                    }
                    results.push(result);
                }
                Err(e) => {
                    warn!("Sequential execution failed at step {}: {}", idx + 1, e);
                    return Err(e);
                }
            }
        }

        Ok(results)
    }

    /// Get coordinator reference
    pub fn coordinator(&self) -> &Coordinator {
        &self.coordinator
    }

    /// Get mutable coordinator reference
    pub fn coordinator_mut(&mut self) -> &mut Coordinator {
        &mut self.coordinator
    }
}

impl Coordinator {
    /// Clone coordinator for parallel execution
    /// Note: This creates a new coordinator with same config
    fn clone_for_parallel(&self) -> Result<Self> {
        Coordinator::new(self.config().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_mode() {
        let single = ExecutionMode::Single("backend-developer".to_string());
        let parallel = ExecutionMode::Parallel(vec![
            "backend-developer".to_string(),
            "frontend-developer".to_string(),
        ]);
        let sequential = ExecutionMode::Sequential(vec![
            "api-designer".to_string(),
            "backend-developer".to_string(),
        ]);

        // Just ensure enum variants work
        match single {
            ExecutionMode::Single(_) => {}
            _ => panic!("Wrong variant"),
        }

        match parallel {
            ExecutionMode::Parallel(agents) => assert_eq!(agents.len(), 2),
            _ => panic!("Wrong variant"),
        }

        match sequential {
            ExecutionMode::Sequential(agents) => assert_eq!(agents.len(), 2),
            _ => panic!("Wrong variant"),
        }
    }
}
