//! Agent registry for discovering and managing AIT42 agents

use crate::error::{AIT42Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::{debug, info, warn};

/// Agent category classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentCategory {
    Backend,
    Frontend,
    Testing,
    Documentation,
    Security,
    Infrastructure,
    Coordination,
    Planning,
    QualityAssurance,
    Operations,
    Meta,
}

impl FromStr for AgentCategory {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "backend" => Ok(Self::Backend),
            "frontend" => Ok(Self::Frontend),
            "testing" | "qa" => Ok(Self::Testing),
            "documentation" | "docs" => Ok(Self::Documentation),
            "security" => Ok(Self::Security),
            "infrastructure" | "infra" => Ok(Self::Infrastructure),
            "coordination" => Ok(Self::Coordination),
            "planning" | "design" => Ok(Self::Planning),
            "quality_assurance" | "qualityassurance" => Ok(Self::QualityAssurance),
            "operations" | "ops" => Ok(Self::Operations),
            "meta" => Ok(Self::Meta),
            _ => Err(format!("Unknown agent category: {}", s)),
        }
    }
}

/// Metadata for an AI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub name: String,
    pub description: String,
    pub category: AgentCategory,
    pub capabilities: Vec<String>,
    pub tools: Vec<String>,
    pub model: String,
    pub file_path: PathBuf,
}

/// Registry of available AI agents
#[derive(Debug)]
pub struct AgentRegistry {
    agents: HashMap<String, AgentMetadata>,
    agents_dir: PathBuf,
}

impl AgentRegistry {
    /// Create a new agent registry
    pub fn new(agents_dir: PathBuf) -> Self {
        Self {
            agents: HashMap::new(),
            agents_dir,
        }
    }

    /// Load agents from directory
    pub fn load_from_directory(agents_dir: &Path) -> Result<Self> {
        let mut registry = Self::new(agents_dir.to_path_buf());
        registry.load_agents()?;
        Ok(registry)
    }

    /// Load all agent metadata files
    fn load_agents(&mut self) -> Result<()> {
        info!("Loading agents from: {}", self.agents_dir.display());

        let entries = fs::read_dir(&self.agents_dir).map_err(|e| {
            AIT42Error::ConfigError(format!(
                "Failed to read agents directory {}: {}",
                self.agents_dir.display(),
                e
            ))
        })?;

        let mut loaded = 0;
        let mut errors = Vec::new();

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                match self.parse_agent_file(&path) {
                    Ok(metadata) => {
                        debug!("Loaded agent: {}", metadata.name);
                        self.agents.insert(metadata.name.clone(), metadata);
                        loaded += 1;
                    }
                    Err(e) => {
                        warn!("Failed to parse {}: {}", path.display(), e);
                        errors.push(e);
                    }
                }
            }
        }

        info!("Loaded {} agents successfully", loaded);
        if !errors.is_empty() {
            warn!("Failed to load {} agents", errors.len());
        }

        Ok(())
    }

    /// Parse agent metadata from markdown file
    fn parse_agent_file(&self, path: &Path) -> Result<AgentMetadata> {
        let content = fs::read_to_string(path)?;

        // Parse YAML frontmatter
        let frontmatter = self.extract_frontmatter(&content)?;

        let name = frontmatter
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AIT42Error::InvalidMetadata("Missing 'name' field".to_string()))?
            .to_string();

        let description = frontmatter
            .get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AIT42Error::InvalidMetadata("Missing 'description' field".to_string()))?
            .to_string();

        // Infer category from agent name
        let category = self.infer_category(&name);

        // Extract capabilities from content
        let capabilities = self.extract_capabilities(&content);

        // Extract tools
        let tools = frontmatter
            .get("tools")
            .and_then(|v| v.as_str())
            .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
            .unwrap_or_default();

        let model = frontmatter
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("sonnet")
            .to_string();

        Ok(AgentMetadata {
            name,
            description,
            category,
            capabilities,
            tools,
            model,
            file_path: path.to_path_buf(),
        })
    }

    /// Extract YAML frontmatter from markdown
    fn extract_frontmatter(&self, content: &str) -> Result<HashMap<String, serde_json::Value>> {
        let lines: Vec<&str> = content.lines().collect();

        if lines.first() != Some(&"---") {
            return Err(AIT42Error::InvalidMetadata(
                "No frontmatter delimiter found".to_string(),
            ));
        }

        let end_idx = lines
            .iter()
            .skip(1)
            .position(|&line| line == "---")
            .ok_or_else(|| {
                AIT42Error::InvalidMetadata("No frontmatter end delimiter found".to_string())
            })?
            + 1;

        let yaml_content = lines[1..end_idx].join("\n");

        // Simple YAML parser (key: value format)
        let mut map = HashMap::new();
        for line in yaml_content.lines() {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim().to_string();
                let value = value.trim().trim_matches('"').to_string();
                map.insert(key, serde_json::Value::String(value));
            }
        }

        Ok(map)
    }

    /// Extract capabilities from agent content
    fn extract_capabilities(&self, content: &str) -> Vec<String> {
        let mut capabilities = Vec::new();

        // Look for <capabilities> section
        if let Some(cap_start) = content.find("<capabilities>") {
            if let Some(cap_end) = content[cap_start..].find("</capabilities>") {
                let cap_section = &content[cap_start + 14..cap_start + cap_end];
                for line in cap_section.lines() {
                    let line = line.trim();
                    if line.starts_with('-') {
                        let cap = line.trim_start_matches('-').trim().to_string();
                        if !cap.is_empty() {
                            capabilities.push(cap);
                        }
                    }
                }
            }
        }

        capabilities
    }

    /// Infer agent category from name
    fn infer_category(&self, name: &str) -> AgentCategory {
        match name {
            n if n.contains("backend") || n.contains("api") || n.contains("database") => {
                AgentCategory::Backend
            }
            n if n.contains("frontend") || n.contains("ui") => AgentCategory::Frontend,
            n if n.contains("test") || n.contains("qa") => AgentCategory::Testing,
            n if n.contains("doc") || n.contains("writer") => AgentCategory::Documentation,
            n if n.contains("security") => AgentCategory::Security,
            n if n.contains("devops")
                || n.contains("infrastructure")
                || n.contains("container")
                || n.contains("cloud") =>
            {
                AgentCategory::Infrastructure
            }
            n if n.contains("coordinator") || n.contains("workflow") => {
                AgentCategory::Coordination
            }
            n if n.contains("architect") || n.contains("designer") || n.contains("planner") => {
                AgentCategory::Planning
            }
            n if n.contains("reviewer")
                || n.contains("refactor")
                || n.contains("complexity")
                || n.contains("mutation") =>
            {
                AgentCategory::QualityAssurance
            }
            n if n.contains("cicd")
                || n.contains("monitoring")
                || n.contains("incident")
                || n.contains("release")
                || n.contains("backup") =>
            {
                AgentCategory::Operations
            }
            _ => AgentCategory::Meta,
        }
    }

    /// Get agent by name
    pub fn get(&self, name: &str) -> Option<&AgentMetadata> {
        self.agents.get(name)
    }

    /// List all agents
    pub fn list(&self) -> Vec<&AgentMetadata> {
        self.agents.values().collect()
    }

    /// List agents by category
    pub fn list_by_category(&self, category: AgentCategory) -> Vec<&AgentMetadata> {
        self.agents
            .values()
            .filter(|agent| agent.category == category)
            .collect()
    }

    /// Search agents by query (fuzzy match)
    pub fn search(&self, query: &str) -> Vec<&AgentMetadata> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<(&AgentMetadata, u32)> = self
            .agents
            .values()
            .filter_map(|agent| {
                let score = self.calculate_match_score(agent, &query_lower);
                if score > 0 {
                    Some((agent, score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (highest first)
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.into_iter().map(|(agent, _)| agent).collect()
    }

    /// Calculate match score for agent
    fn calculate_match_score(&self, agent: &AgentMetadata, query: &str) -> u32 {
        let mut score = 0u32;

        // Exact name match
        if agent.name.to_lowercase() == query {
            score += 100;
        } else if agent.name.to_lowercase().contains(query) {
            score += 50;
        }

        // Description match
        if agent.description.to_lowercase().contains(query) {
            score += 30;
        }

        // Capabilities match
        for cap in &agent.capabilities {
            if cap.to_lowercase().contains(query) {
                score += 10;
            }
        }

        score
    }

    /// Get total number of agents
    pub fn count(&self) -> usize {
        self.agents.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_agent_category_from_str() {
        assert_eq!(AgentCategory::from_str("backend"), Ok(AgentCategory::Backend));
        assert_eq!(AgentCategory::from_str("Backend"), Ok(AgentCategory::Backend));
        assert_eq!(AgentCategory::from_str("qa"), Ok(AgentCategory::Testing));
        assert!(AgentCategory::from_str("invalid").is_err());
    }

    #[test]
    fn test_infer_category() {
        let registry = AgentRegistry::new(PathBuf::from("/tmp"));
        assert_eq!(
            registry.infer_category("backend-developer"),
            AgentCategory::Backend
        );
        assert_eq!(
            registry.infer_category("frontend-developer"),
            AgentCategory::Frontend
        );
        assert_eq!(
            registry.infer_category("test-generator"),
            AgentCategory::Testing
        );
    }

    #[test]
    fn test_parse_agent_file() {
        let temp_dir = TempDir::new().unwrap();
        let agent_path = temp_dir.path().join("test-agent.md");

        let content = r#"---
name: test-agent
description: "A test agent"
tools: Read, Write
model: sonnet
---

<capabilities>
- Capability 1
- Capability 2
</capabilities>
"#;

        let mut file = fs::File::create(&agent_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let registry = AgentRegistry::new(temp_dir.path().to_path_buf());
        let metadata = registry.parse_agent_file(&agent_path).unwrap();

        assert_eq!(metadata.name, "test-agent");
        assert_eq!(metadata.description, "A test agent");
        assert_eq!(metadata.capabilities.len(), 2);
    }
}
