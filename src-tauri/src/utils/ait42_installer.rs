use std::fs;
use std::path::{Path, PathBuf};
use std::io;

/// AIT42 installation result
#[derive(Debug, Clone)]
pub struct InstallResult {
    pub success: bool,
    pub agents_installed: usize,
    pub memory_setup: bool,
    pub sops_installed: usize,
    pub errors: Vec<String>,
}

/// AIT42 installer utility
pub struct AIT42Installer {
    source_ait42_path: PathBuf,
}

impl AIT42Installer {
    /// Create a new installer with the source AIT42 directory
    pub fn new(source_ait42_path: impl Into<PathBuf>) -> Self {
        Self {
            source_ait42_path: source_ait42_path.into(),
        }
    }

    /// Install complete AIT42 system to target workspace
    pub fn install_to_workspace(&self, workspace_path: &Path) -> Result<InstallResult, String> {
        let mut result = InstallResult {
            success: false,
            agents_installed: 0,
            memory_setup: false,
            sops_installed: 0,
            errors: Vec::new(),
        };

        // Verify source AIT42 exists
        if !self.source_ait42_path.exists() {
            return Err(format!(
                "Source AIT42 directory not found: {}",
                self.source_ait42_path.display()
            ));
        }

        let source_agents = self.source_ait42_path.join(".claude/agents");
        if !source_agents.exists() {
            return Err(format!(
                "Source agents directory not found: {}",
                source_agents.display()
            ));
        }

        // Create target directories
        let target_claude = workspace_path.join(".claude");
        let target_agents = target_claude.join("agents");
        let target_memory = target_claude.join("memory");

        fs::create_dir_all(&target_agents)
            .map_err(|e| format!("Failed to create .claude/agents: {}", e))?;

        // Install agents
        tracing::info!("📦 Installing AIT42 agents to {}", workspace_path.display());
        match self.install_agents(&source_agents, &target_agents) {
            Ok(count) => {
                result.agents_installed = count;
                tracing::info!("✅ Installed {} agents", count);
            }
            Err(e) => {
                let err_msg = format!("Agent installation failed: {}", e);
                tracing::error!("{}", err_msg);
                result.errors.push(err_msg);
            }
        }

        // Setup memory system
        let source_memory = self.source_ait42_path.join(".claude/memory");
        if source_memory.exists() {
            match self.setup_memory_system(&source_memory, &target_memory) {
                Ok(_) => {
                    result.memory_setup = true;
                    tracing::info!("✅ Memory system configured");
                }
                Err(e) => {
                    let err_msg = format!("Memory setup failed: {}", e);
                    tracing::error!("{}", err_msg);
                    result.errors.push(err_msg);
                }
            }
        }

        // Install SOPs
        let source_sops = source_memory.join("sop-templates");
        let target_sops = target_memory.join("sop-templates");
        if source_sops.exists() {
            match self.install_sops(&source_sops, &target_sops) {
                Ok(count) => {
                    result.sops_installed = count;
                    tracing::info!("✅ Installed {} SOPs", count);
                }
                Err(e) => {
                    let err_msg = format!("SOP installation failed: {}", e);
                    tracing::error!("{}", err_msg);
                    result.errors.push(err_msg);
                }
            }
        }

        result.success = result.agents_installed > 0 && result.errors.is_empty();
        Ok(result)
    }

    /// Install all agent files
    fn install_agents(&self, source: &Path, target: &Path) -> Result<usize, String> {
        let entries = fs::read_dir(source)
            .map_err(|e| format!("Failed to read source agents: {}", e))?;

        let mut count = 0;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                let file_name = path.file_name().unwrap();
                let target_file = target.join(file_name);

                fs::copy(&path, &target_file)
                    .map_err(|e| format!("Failed to copy {}: {}", file_name.to_string_lossy(), e))?;

                count += 1;
            }
        }

        Ok(count)
    }

    /// Setup memory system structure
    fn setup_memory_system(&self, source: &Path, target: &Path) -> Result<(), String> {
        fs::create_dir_all(target)
            .map_err(|e| format!("Failed to create memory directory: {}", e))?;

        // Create subdirectories
        let subdirs = ["tasks", "agents"];
        for subdir in &subdirs {
            fs::create_dir_all(target.join(subdir))
                .map_err(|e| format!("Failed to create {} directory: {}", subdir, e))?;
        }

        // Copy config.yaml if exists
        let source_config = source.join("config.yaml");
        let target_config = target.join("config.yaml");
        if source_config.exists() {
            fs::copy(&source_config, &target_config)
                .map_err(|e| format!("Failed to copy config.yaml: {}", e))?;
        }

        Ok(())
    }

    /// Install SOP templates
    fn install_sops(&self, source: &Path, target: &Path) -> Result<usize, String> {
        fs::create_dir_all(target)
            .map_err(|e| format!("Failed to create sop-templates directory: {}", e))?;

        let entries = fs::read_dir(source)
            .map_err(|e| format!("Failed to read source SOPs: {}", e))?;

        let mut count = 0;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                let file_name = path.file_name().unwrap();
                let target_file = target.join(file_name);

                fs::copy(&path, &target_file)
                    .map_err(|e| format!("Failed to copy {}: {}", file_name.to_string_lossy(), e))?;

                count += 1;
            }
        }

        Ok(count)
    }

    /// Check if AIT42 is already installed in workspace
    pub fn is_installed(&self, workspace_path: &Path) -> bool {
        let agents_dir = workspace_path.join(".claude/agents");
        let coordinator = agents_dir.join("00-ait42-coordinator.md");

        agents_dir.exists() && coordinator.exists()
    }

    /// Verify installation completeness
    pub fn verify_installation(&self, workspace_path: &Path) -> Result<InstallResult, String> {
        let agents_dir = workspace_path.join(".claude/agents");
        let memory_dir = workspace_path.join(".claude/memory");

        let mut result = InstallResult {
            success: false,
            agents_installed: 0,
            memory_setup: false,
            sops_installed: 0,
            errors: Vec::new(),
        };

        // Count agents
        if agents_dir.exists() {
            match fs::read_dir(&agents_dir) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        if entry.path().extension().map_or(false, |ext| ext == "md") {
                            result.agents_installed += 1;
                        }
                    }
                }
                Err(e) => {
                    result.errors.push(format!("Failed to read agents: {}", e));
                }
            }
        }

        // Check memory system
        result.memory_setup = memory_dir.exists()
            && memory_dir.join("tasks").exists()
            && memory_dir.join("agents").exists();

        // Count SOPs
        let sops_dir = memory_dir.join("sop-templates");
        if sops_dir.exists() {
            match fs::read_dir(&sops_dir) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        if entry.path().extension().map_or(false, |ext| ext == "md") {
                            result.sops_installed += 1;
                        }
                    }
                }
                Err(e) => {
                    result.errors.push(format!("Failed to read SOPs: {}", e));
                }
            }
        }

        result.success = result.agents_installed >= 40 && result.memory_setup;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_installer_creation() {
        let source = PathBuf::from("/path/to/ait42");
        let installer = AIT42Installer::new(source.clone());
        assert_eq!(installer.source_ait42_path, source);
    }

    #[test]
    fn test_is_installed_false_when_not_exists() {
        let temp = tempdir().unwrap();
        let installer = AIT42Installer::new("/path/to/ait42");
        assert!(!installer.is_installed(temp.path()));
    }
}
