use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Per-runtime installation summary
#[derive(Debug, Clone)]
pub struct RuntimeInstallResult {
    pub name: String,
    pub agents_installed: usize,
    pub memory_setup: bool,
    pub sops_installed: usize,
}

impl RuntimeInstallResult {
    pub fn is_healthy(&self) -> bool {
        self.agents_installed > 0 && self.memory_setup
    }
}

/// AIT42 installation result (aggregated across runtimes)
#[derive(Debug, Clone)]
pub struct InstallResult {
    pub success: bool,
    pub agents_installed: usize,
    pub memory_setup: bool,
    pub sops_installed: usize,
    pub runtime_summaries: Vec<RuntimeInstallResult>,
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
        if !self.source_ait42_path.exists() {
            return Err(format!(
                "Source AIT42 directory not found: {}",
                self.source_ait42_path.display()
            ));
        }

        let mut errors = Vec::new();
        let mut runtime_summaries = Vec::new();
        let runtimes = ["claude", "codex", "gemini"];

        for runtime in &runtimes {
            match self.install_runtime(runtime, workspace_path) {
                Ok(summary) => runtime_summaries.push(summary),
                Err(e) => {
                    tracing::warn!("⚠️ Failed to install {} runtime: {}", runtime, e);
                    errors.push(e);
                }
            }
        }

        let mut result = InstallResult {
            success: false,
            agents_installed: 0,
            memory_setup: false,
            sops_installed: 0,
            runtime_summaries,
            errors,
        };

        if let Some(claude_summary) = result
            .runtime_summaries
            .iter()
            .find(|summary| summary.name == "claude")
        {
            result.agents_installed = claude_summary.agents_installed;
            result.memory_setup = claude_summary.memory_setup;
            result.sops_installed = claude_summary.sops_installed;
        }

        result.success = result.errors.is_empty()
            && !result.runtime_summaries.is_empty()
            && result
                .runtime_summaries
                .iter()
                .all(|summary| summary.is_healthy());

        Ok(result)
    }

    fn install_runtime(
        &self,
        runtime: &str,
        workspace_path: &Path,
    ) -> Result<RuntimeInstallResult, String> {
        let runtime_dir_name = format!(".{}", runtime);
        let source_runtime = self.source_ait42_path.join(&runtime_dir_name);
        if !source_runtime.exists() {
            return Err(format!(
                "Source runtime directory not found: {}",
                source_runtime.display()
            ));
        }

        let target_runtime = workspace_path.join(&runtime_dir_name);
        fs::create_dir_all(&target_runtime)
            .map_err(|e| format!("Failed to create {} directory: {}", runtime_dir_name, e))?;

        let source_agents = source_runtime.join("agents");
        let target_agents = target_runtime.join("agents");
        if source_agents.exists() {
            fs::create_dir_all(&target_agents)
                .map_err(|e| format!("Failed to create agents directory for {}: {}", runtime, e))?;
            self.install_agents(&source_agents, &target_agents)?;
        }

        let source_memory = source_runtime.join("memory");
        let target_memory = target_runtime.join("memory");
        if source_memory.exists() {
            self.setup_memory_system(&source_memory, &target_memory)?;
        } else {
            fs::create_dir_all(&target_memory)
                .map_err(|e| format!("Failed to create memory directory for {}: {}", runtime, e))?;
        }

        let source_sops = source_memory.join("sop-templates");
        let target_sops = target_memory.join("sop-templates");
        if source_sops.exists() {
            self.install_sops(&source_sops, &target_sops)?;
        }

        Ok(self.collect_runtime_stats(workspace_path, runtime))
    }

    fn collect_runtime_stats(&self, workspace_path: &Path, runtime: &str) -> RuntimeInstallResult {
        let runtime_dir = workspace_path.join(format!(".{}", runtime));
        let agents_dir = runtime_dir.join("agents");
        let memory_dir = runtime_dir.join("memory");
        let sops_dir = memory_dir.join("sop-templates");

        RuntimeInstallResult {
            name: runtime.to_string(),
            agents_installed: count_markdown_files(&agents_dir),
            memory_setup: memory_dir.exists()
                && memory_dir.join("tasks").exists()
                && memory_dir.join("agents").exists(),
            sops_installed: count_markdown_files(&sops_dir),
        }
    }

    /// Install all agent files
    fn install_agents(&self, source: &Path, target: &Path) -> Result<usize, String> {
        let entries =
            fs::read_dir(source).map_err(|e| format!("Failed to read source agents: {}", e))?;

        let mut count = 0;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                let file_name = path.file_name().unwrap();
                let target_file = target.join(file_name);

                fs::copy(&path, &target_file).map_err(|e| {
                    format!("Failed to copy {}: {}", file_name.to_string_lossy(), e)
                })?;

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

        self.copy_memory_extras(source, target)?;

        Ok(())
    }

    fn copy_memory_extras(&self, source: &Path, target: &Path) -> Result<(), String> {
        if !source.exists() {
            return Ok(());
        }

        let entries =
            fs::read_dir(source).map_err(|e| format!("Failed to read memory directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if name_str == "tasks"
                || name_str == "agents"
                || name_str == "config.yaml"
                || name_str == "sop-templates"
            {
                continue;
            }

            let destination = target.join(&name);
            let file_type = entry
                .file_type()
                .map_err(|e| format!("Failed to inspect entry type: {}", e))?;

            if file_type.is_dir() {
                copy_dir_recursive(&entry.path(), &destination)
                    .map_err(|e| format!("Failed to copy directory {:?}: {}", entry.path(), e))?;
            } else if file_type.is_file() {
                fs::copy(&entry.path(), &destination)
                    .map_err(|e| format!("Failed to copy file {:?}: {}", entry.path(), e))?;
            }
        }

        Ok(())
    }

    /// Install SOP templates
    fn install_sops(&self, source: &Path, target: &Path) -> Result<usize, String> {
        fs::create_dir_all(target)
            .map_err(|e| format!("Failed to create sop-templates directory: {}", e))?;

        let entries =
            fs::read_dir(source).map_err(|e| format!("Failed to read source SOPs: {}", e))?;

        let mut count = 0;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                let file_name = path.file_name().unwrap();
                let target_file = target.join(file_name);

                fs::copy(&path, &target_file).map_err(|e| {
                    format!("Failed to copy {}: {}", file_name.to_string_lossy(), e)
                })?;

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
        let runtime_summaries = ["claude", "codex", "gemini"]
            .iter()
            .map(|runtime| self.collect_runtime_stats(workspace_path, runtime))
            .collect::<Vec<_>>();

        let mut result = InstallResult {
            success: false,
            agents_installed: 0,
            memory_setup: false,
            sops_installed: 0,
            runtime_summaries,
            errors: Vec::new(),
        };

        if let Some(claude_summary) = result
            .runtime_summaries
            .iter()
            .find(|summary| summary.name == "claude")
        {
            result.agents_installed = claude_summary.agents_installed;
            result.memory_setup = claude_summary.memory_setup;
            result.sops_installed = claude_summary.sops_installed;
        }

        result.success = result
            .runtime_summaries
            .iter()
            .all(|summary| summary.is_healthy());

        Ok(result)
    }
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> io::Result<()> {
    if !source.exists() {
        return Ok(());
    }

    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_path = entry.path();
        let dest_path = destination.join(entry.file_name());

        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry_path, &dest_path)?;
        } else {
            fs::copy(&entry_path, &dest_path)?;
        }
    }

    Ok(())
}

fn count_markdown_files(path: &Path) -> usize {
    if !path.exists() {
        return 0;
    }

    match fs::read_dir(path) {
        Ok(entries) => entries
            .flatten()
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "md"))
            .count(),
        Err(_) => 0,
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
