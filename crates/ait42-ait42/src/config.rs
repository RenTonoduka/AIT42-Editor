//! Configuration for AIT42 agent integration

use crate::error::{AIT42Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// AIT42 integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIT42Config {
    /// Root directory of AIT42 system
    pub ait42_root: PathBuf,

    /// Maximum number of parallel agents
    #[serde(default = "default_max_parallel")]
    pub max_parallel_agents: usize,

    /// Session timeout in seconds
    #[serde(default = "default_timeout")]
    pub session_timeout_secs: u64,

    /// Auto-cleanup old sessions
    #[serde(default = "default_auto_cleanup")]
    pub auto_cleanup: bool,

    /// Max age for cleanup (seconds)
    #[serde(default = "default_cleanup_age")]
    pub cleanup_max_age_secs: u64,

    /// Claude API key (optional)
    pub claude_api_key: Option<String>,

    /// Enable debug logging
    #[serde(default)]
    pub debug: bool,
}

fn default_max_parallel() -> usize {
    3
}

fn default_timeout() -> u64 {
    600 // 10 minutes
}

fn default_auto_cleanup() -> bool {
    true
}

fn default_cleanup_age() -> u64 {
    3600 // 1 hour
}

impl AIT42Config {
    /// Create a new configuration
    pub fn new(ait42_root: PathBuf) -> Self {
        Self {
            ait42_root,
            max_parallel_agents: default_max_parallel(),
            session_timeout_secs: default_timeout(),
            auto_cleanup: default_auto_cleanup(),
            cleanup_max_age_secs: default_cleanup_age(),
            claude_api_key: None,
            debug: false,
        }
    }

    /// Load configuration from environment and defaults
    pub fn load() -> Result<Self> {
        let ait42_root = std::env::var("AIT42_ROOT")
            .ok()
            .map(PathBuf::from)
            .or_else(|| Self::detect_ait42_root())
            .ok_or_else(|| {
                AIT42Error::ConfigError(
                    "AIT42_ROOT not set and could not detect AIT42 installation".to_string(),
                )
            })?;

        let mut config = Self::new(ait42_root);

        // Load from environment variables
        if let Ok(max_parallel) = std::env::var("AIT42_MAX_PARALLEL") {
            if let Ok(value) = max_parallel.parse() {
                config.max_parallel_agents = value;
            }
        }

        if let Ok(timeout) = std::env::var("AIT42_TIMEOUT") {
            if let Ok(value) = timeout.parse() {
                config.session_timeout_secs = value;
            }
        }

        if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
            config.claude_api_key = Some(api_key);
        }

        if std::env::var("AIT42_DEBUG").is_ok() {
            config.debug = true;
        }

        config.validate()?;
        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if !self.ait42_root.exists() {
            return Err(AIT42Error::ConfigError(format!(
                "AIT42 root directory does not exist: {}",
                self.ait42_root.display()
            )));
        }

        let agents_dir = self.ait42_root.join(".claude/agents");
        if !agents_dir.exists() {
            return Err(AIT42Error::ConfigError(format!(
                "Agents directory not found: {}",
                agents_dir.display()
            )));
        }

        let scripts_dir = self.ait42_root.join("scripts");
        if !scripts_dir.exists() {
            return Err(AIT42Error::ConfigError(format!(
                "Scripts directory not found: {}",
                scripts_dir.display()
            )));
        }

        if self.max_parallel_agents == 0 {
            return Err(AIT42Error::ConfigError(
                "max_parallel_agents must be greater than 0".to_string(),
            ));
        }

        if self.max_parallel_agents > 10 {
            return Err(AIT42Error::ConfigError(
                "max_parallel_agents should not exceed 10".to_string(),
            ));
        }

        Ok(())
    }

    /// Detect AIT42 root directory
    fn detect_ait42_root() -> Option<PathBuf> {
        // Try common locations
        let home = std::env::var("HOME").ok()?;
        let candidates = vec![
            PathBuf::from(&home).join("Programming/AI/02_Workspace/05_Client/03_Sun/AIT42"),
            PathBuf::from(&home).join("AIT42"),
            PathBuf::from(&home).join(".ait42"),
            PathBuf::from("/opt/ait42"),
        ];

        for candidate in candidates {
            if candidate.join(".claude/agents").exists() {
                return Some(candidate);
            }
        }

        None
    }

    /// Get timeout as Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.session_timeout_secs)
    }

    /// Get cleanup age as Duration
    pub fn cleanup_age(&self) -> Duration {
        Duration::from_secs(self.cleanup_max_age_secs)
    }

    /// Get agents directory path
    pub fn agents_dir(&self) -> PathBuf {
        self.ait42_root.join(".claude/agents")
    }

    /// Get scripts directory path
    pub fn scripts_dir(&self) -> PathBuf {
        self.ait42_root.join("scripts")
    }
}

impl Default for AIT42Config {
    fn default() -> Self {
        Self::new(PathBuf::from("/tmp/ait42"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AIT42Config::new(PathBuf::from("/tmp/ait42"));
        assert_eq!(config.max_parallel_agents, 3);
        assert_eq!(config.session_timeout_secs, 600);
        assert!(config.auto_cleanup);
    }

    #[test]
    fn test_timeout_duration() {
        let config = AIT42Config::new(PathBuf::from("/tmp/ait42"));
        assert_eq!(config.timeout(), Duration::from_secs(600));
    }

    #[test]
    fn test_paths() {
        let config = AIT42Config::new(PathBuf::from("/tmp/ait42"));
        assert_eq!(
            config.agents_dir(),
            PathBuf::from("/tmp/ait42/.claude/agents")
        );
        assert_eq!(config.scripts_dir(), PathBuf::from("/tmp/ait42/scripts"));
    }
}
