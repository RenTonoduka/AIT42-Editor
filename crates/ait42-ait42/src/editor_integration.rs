//! Integration bridge between editor and agent system

use crate::error::Result;
use crate::executor::AgentExecutor;

/// Simple buffer representation (minimal - would use actual ait42-core types)
#[derive(Debug, Clone)]
pub struct Buffer {
    pub content: String,
    pub file_path: Option<String>,
    pub language: Option<String>,
}

/// Simple selection representation
#[derive(Debug, Clone)]
pub struct Selection {
    pub start_line: usize,
    pub end_line: usize,
}

/// Bridge between editor and agent system
pub struct EditorAgentBridge {
    executor: AgentExecutor,
}

impl EditorAgentBridge {
    /// Create a new editor-agent bridge
    pub fn new(executor: AgentExecutor) -> Self {
        Self { executor }
    }

    /// Execute agent on current buffer content
    pub async fn run_on_buffer(&mut self, agent: &str, buffer: &Buffer) -> Result<String> {
        let context = self.build_buffer_context(buffer);
        let task = format!(
            "Analyze and process the following code:\n\n{}",
            context
        );

        let results = self
            .executor
            .execute_single(agent, &task)
            .await?;

        Ok(results.output)
    }

    /// Execute agent on selected text
    pub async fn run_on_selection(
        &mut self,
        agent: &str,
        selection: &Selection,
        buffer: &Buffer,
    ) -> Result<String> {
        let selected_text = self.extract_selection(buffer, selection);
        let context = format!(
            "File: {}\nLanguage: {}\nSelected lines {}-{}:\n\n{}",
            buffer.file_path.as_deref().unwrap_or("untitled"),
            buffer.language.as_deref().unwrap_or("text"),
            selection.start_line,
            selection.end_line,
            selected_text
        );

        let task = format!(
            "Analyze and process the following code selection:\n\n{}",
            context
        );

        let results = self.executor.execute_single(agent, &task).await?;

        Ok(results.output)
    }

    /// Run code review on buffer
    pub async fn review_buffer(&mut self, buffer: &Buffer) -> Result<String> {
        self.run_on_buffer("code-reviewer", buffer).await
    }

    /// Generate tests for buffer
    pub async fn generate_tests(&mut self, buffer: &Buffer) -> Result<String> {
        self.run_on_buffer("test-generator", buffer).await
    }

    /// Refactor selection
    pub async fn refactor_selection(
        &mut self,
        selection: &Selection,
        buffer: &Buffer,
    ) -> Result<String> {
        self.run_on_selection("refactor-specialist", selection, buffer)
            .await
    }

    /// Security scan buffer
    pub async fn security_scan(&mut self, buffer: &Buffer) -> Result<String> {
        self.run_on_buffer("security-scanner", buffer).await
    }

    /// Document code
    pub async fn document_code(&mut self, buffer: &Buffer) -> Result<String> {
        self.run_on_buffer("tech-writer", buffer).await
    }

    /// Build context string from buffer
    fn build_buffer_context(&self, buffer: &Buffer) -> String {
        let mut context = String::new();

        if let Some(path) = &buffer.file_path {
            context.push_str(&format!("File: {}\n", path));
        }

        if let Some(lang) = &buffer.language {
            context.push_str(&format!("Language: {}\n", lang));
        }

        context.push_str("\nContent:\n");
        context.push_str("```\n");
        context.push_str(&buffer.content);
        context.push_str("\n```\n");

        context
    }

    /// Extract selected text from buffer
    fn extract_selection(&self, buffer: &Buffer, selection: &Selection) -> String {
        let lines: Vec<&str> = buffer.content.lines().collect();

        lines
            .iter()
            .skip(selection.start_line.saturating_sub(1))
            .take(selection.end_line - selection.start_line + 1)
            .map(|&s| s)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get executor reference
    pub fn executor(&self) -> &AgentExecutor {
        &self.executor
    }

    /// Get mutable executor reference
    pub fn executor_mut(&mut self) -> &mut AgentExecutor {
        &mut self.executor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_env() -> Result<()> {
        let root = std::path::PathBuf::from("/tmp/ait42");

        // Create test directory structure
        fs::create_dir_all(root.join(".claude/agents"))?;
        fs::create_dir_all(root.join("scripts"))?;

        Ok(())
    }

    fn create_test_bridge() -> EditorAgentBridge {
        setup_test_env().expect("Failed to setup test environment");

        let config = crate::config::AIT42Config::default();
        let coordinator = crate::coordinator::Coordinator::new(config)
            .expect("Failed to create coordinator");
        let executor = AgentExecutor::new(coordinator);
        EditorAgentBridge::new(executor)
    }

    #[test]
    fn test_build_buffer_context() {
        let bridge = create_test_bridge();

        let buffer = Buffer {
            content: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
            file_path: Some("main.rs".to_string()),
            language: Some("rust".to_string()),
        };

        let context = bridge.build_buffer_context(&buffer);
        assert!(context.contains("File: main.rs"));
        assert!(context.contains("Language: rust"));
        assert!(context.contains("fn main()"));
    }

    #[test]
    fn test_extract_selection() {
        let bridge = create_test_bridge();

        let buffer = Buffer {
            content: "line 1\nline 2\nline 3\nline 4".to_string(),
            file_path: None,
            language: None,
        };

        let selection = Selection {
            start_line: 2,
            end_line: 3,
        };

        let selected = bridge.extract_selection(&buffer, &selection);
        assert_eq!(selected, "line 2\nline 3");
    }
}
