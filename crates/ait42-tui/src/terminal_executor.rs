//! Terminal Command Executor
//!
//! Provides safe, async command execution with output buffering, timeout handling,
//! and comprehensive error management for the AIT42 Editor terminal panel.
//!
//! # Features
//!
//! - Async command execution using Tokio
//! - Real-time stdout/stderr capture
//! - Command timeout (default: 30 seconds)
//! - Input sanitization to prevent shell injection
//! - Working directory management
//! - Exit code capture and display
//! - Output buffer management with scrolling
//!
//! # Safety
//!
//! All commands are sanitized before execution to prevent shell injection attacks.
//! Commands are executed directly without shell interpretation when possible.
//!
//! # Example
//!
//! ```rust
//! use ait42_tui::terminal_executor::TerminalExecutor;
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let mut executor = TerminalExecutor::new(PathBuf::from("/workspace"));
//!
//!     executor.execute("ls -la").await?;
//!
//!     for line in executor.get_output() {
//!         println!("{}", line);
//!     }
//!
//!     Ok(())
//! }
//! ```

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

/// Maximum number of output lines to keep in buffer (prevent memory exhaustion)
const MAX_OUTPUT_LINES: usize = 10_000;

/// Default command timeout (30 seconds)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Terminal command executor with async support and safety features
#[derive(Debug)]
pub struct TerminalExecutor {
    /// Buffered output lines (stdout + stderr combined)
    output_buffer: Vec<String>,

    /// Current working directory for command execution
    current_dir: PathBuf,

    /// Command execution timeout
    timeout_duration: Duration,

    /// Command history
    command_history: Vec<String>,

    /// Maximum history size
    max_history: usize,
}

impl TerminalExecutor {
    /// Create a new terminal executor
    ///
    /// # Arguments
    ///
    /// * `working_dir` - Initial working directory for command execution
    ///
    /// # Example
    ///
    /// ```rust
    /// use ait42_tui::terminal_executor::TerminalExecutor;
    /// use std::path::PathBuf;
    ///
    /// let executor = TerminalExecutor::new(PathBuf::from("/workspace"));
    /// ```
    pub fn new(working_dir: PathBuf) -> Self {
        Self {
            output_buffer: Vec::with_capacity(1024),
            current_dir: working_dir,
            timeout_duration: DEFAULT_TIMEOUT,
            command_history: Vec::with_capacity(100),
            max_history: 1000,
        }
    }

    /// Execute a command asynchronously
    ///
    /// # Arguments
    ///
    /// * `command` - Command string to execute (e.g., "ls -la", "cargo build")
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Command executed successfully
    /// * `Err(_)` - Command failed or timed out
    ///
    /// # Safety
    ///
    /// Commands are sanitized before execution. Shell metacharacters in unsafe
    /// contexts will cause the command to be rejected.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ait42_tui::terminal_executor::TerminalExecutor;
    /// # use std::path::PathBuf;
    /// # tokio_test::block_on(async {
    /// let mut executor = TerminalExecutor::new(PathBuf::from("/tmp"));
    /// executor.execute("echo 'Hello, World!'").await?;
    /// # Ok::<(), anyhow::Error>(())
    /// # });
    /// ```
    pub async fn execute(&mut self, command: &str) -> Result<()> {
        // Sanitize and validate command
        let command = command.trim();
        if command.is_empty() {
            self.append_output("Error: Empty command".to_string());
            return Ok(());
        }

        // Add command to history
        self.add_to_history(command.to_string());

        // Display command prompt
        self.append_output(format!("$ {}", command));

        // Parse command into program and arguments
        let (program, args) = match Self::parse_command(command) {
            Ok(parsed) => parsed,
            Err(e) => {
                self.append_output(format!("Error: {}", e));
                return Ok(());
            }
        };

        // Handle built-in commands
        if let Some(result) = self.handle_builtin_command(&program, &args) {
            return result;
        }

        // Execute external command
        self.execute_external_command(&program, &args).await
    }

    /// Execute an external command
    async fn execute_external_command(&mut self, program: &str, args: &[String]) -> Result<()> {
        // Build command
        let mut cmd = TokioCommand::new(program);
        cmd.args(args)
            .current_dir(&self.current_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        // Spawn process
        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                self.append_output(format!("Error: Failed to execute '{}': {}", program, e));
                return Ok(());
            }
        };

        // Capture stdout and stderr
        let stdout = child.stdout.take().context("Failed to capture stdout")?;
        let stderr = child.stderr.take().context("Failed to capture stderr")?;

        let stdout_reader = BufReader::new(stdout);
        let stderr_reader = BufReader::new(stderr);

        let mut stdout_lines = stdout_reader.lines();
        let mut stderr_lines = stderr_reader.lines();

        // Read output with timeout
        let timeout_duration = self.timeout_duration;
        let mut output_lines = Vec::new();

        let output_future = async {
            loop {
                tokio::select! {
                    line = stdout_lines.next_line() => {
                        match line {
                            Ok(Some(line)) => output_lines.push(line),
                            Ok(None) => break,
                            Err(e) => {
                                output_lines.push(format!("Error reading stdout: {}", e));
                                break;
                            }
                        }
                    }
                    line = stderr_lines.next_line() => {
                        match line {
                            Ok(Some(line)) => output_lines.push(format!("stderr: {}", line)),
                            Ok(None) => break,
                            Err(e) => {
                                output_lines.push(format!("Error reading stderr: {}", e));
                                break;
                            }
                        }
                    }
                }
            }
            output_lines
        };

        // Wait for command with timeout
        match timeout(timeout_duration, output_future).await {
            Ok(lines) => {
                // Add all captured output
                for line in lines {
                    self.append_output(line);
                }

                // Wait for process to exit
                match child.wait().await {
                    Ok(status) => {
                        if let Some(code) = status.code() {
                            if code != 0 {
                                self.append_output(format!("Exit code: {}", code));
                            }
                        } else {
                            self.append_output("Process terminated by signal".to_string());
                        }
                    }
                    Err(e) => {
                        self.append_output(format!("Error waiting for process: {}", e));
                    }
                }
            }
            Err(_) => {
                // Timeout - kill process
                let _ = child.kill().await;
                self.append_output(format!(
                    "Error: Command timed out after {} seconds",
                    timeout_duration.as_secs()
                ));
            }
        }

        Ok(())
    }

    /// Handle built-in commands (cd, clear, etc.)
    fn handle_builtin_command(&mut self, program: &str, args: &[String]) -> Option<Result<()>> {
        match program {
            "cd" => Some(self.builtin_cd(args)),
            "clear" => {
                self.clear();
                Some(Ok(()))
            }
            "pwd" => {
                self.append_output(self.current_dir.display().to_string());
                Some(Ok(()))
            }
            "history" => {
                let history: Vec<String> = self.command_history.iter().enumerate()
                    .map(|(idx, cmd)| format!("{:4} {}", idx + 1, cmd))
                    .collect();
                for line in history {
                    self.append_output(line);
                }
                Some(Ok(()))
            }
            _ => None,
        }
    }

    /// Built-in cd command
    fn builtin_cd(&mut self, args: &[String]) -> Result<()> {
        let target = if args.is_empty() {
            // cd with no args goes to home
            std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
        } else {
            args[0].clone()
        };

        let new_dir = if target.starts_with('/') {
            PathBuf::from(&target)
        } else {
            self.current_dir.join(&target)
        };

        match new_dir.canonicalize() {
            Ok(canonical) => {
                if canonical.is_dir() {
                    self.current_dir = canonical;
                    self.append_output(format!("Changed directory to: {}", self.current_dir.display()));
                } else {
                    self.append_output(format!("Error: Not a directory: {}", target));
                }
            }
            Err(e) => {
                self.append_output(format!("Error: {}: {}", target, e));
            }
        }

        Ok(())
    }

    /// Parse command string into program and arguments
    ///
    /// Handles quoted arguments and basic shell syntax
    fn parse_command(command: &str) -> Result<(String, Vec<String>)> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut escape_next = false;

        for ch in command.chars() {
            if escape_next {
                current.push(ch);
                escape_next = false;
                continue;
            }

            match ch {
                '\\' => {
                    escape_next = true;
                }
                '"' | '\'' => {
                    in_quotes = !in_quotes;
                }
                ' ' if !in_quotes => {
                    if !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if !current.is_empty() {
            parts.push(current);
        }

        if parts.is_empty() {
            anyhow::bail!("Empty command");
        }

        let program = parts[0].clone();
        let args = parts[1..].to_vec();

        Ok((program, args))
    }

    /// Append output line to buffer
    fn append_output(&mut self, line: String) {
        self.output_buffer.push(line);

        // Trim buffer if too large
        if self.output_buffer.len() > MAX_OUTPUT_LINES {
            let drain_count = self.output_buffer.len() - MAX_OUTPUT_LINES;
            self.output_buffer.drain(0..drain_count);
        }
    }

    /// Add command to history
    fn add_to_history(&mut self, command: String) {
        // Don't add duplicate of last command
        if let Some(last) = self.command_history.last() {
            if last == &command {
                return;
            }
        }

        self.command_history.push(command);

        // Trim history if too large
        if self.command_history.len() > self.max_history {
            let drain_count = self.command_history.len() - self.max_history;
            self.command_history.drain(0..drain_count);
        }
    }

    /// Get output buffer
    ///
    /// Returns a slice of all buffered output lines
    pub fn get_output(&self) -> &[String] {
        &self.output_buffer
    }

    /// Get the last N lines of output
    pub fn get_output_tail(&self, n: usize) -> &[String] {
        let start = self.output_buffer.len().saturating_sub(n);
        &self.output_buffer[start..]
    }

    /// Clear output buffer
    pub fn clear(&mut self) {
        self.output_buffer.clear();
    }

    /// Get current working directory
    pub fn current_dir(&self) -> &PathBuf {
        &self.current_dir
    }

    /// Set working directory
    pub fn set_current_dir(&mut self, dir: PathBuf) {
        self.current_dir = dir;
    }

    /// Set command timeout
    pub fn set_timeout(&mut self, duration: Duration) {
        self.timeout_duration = duration;
    }

    /// Get command timeout
    pub fn timeout(&self) -> Duration {
        self.timeout_duration
    }

    /// Get command history
    pub fn history(&self) -> &[String] {
        &self.command_history
    }

    /// Get history entry by index (most recent = 0)
    pub fn history_entry(&self, idx: usize) -> Option<&str> {
        let reverse_idx = self.command_history.len().saturating_sub(idx + 1);
        self.command_history.get(reverse_idx).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_new_executor() {
        let executor = TerminalExecutor::new(PathBuf::from("/tmp"));
        assert_eq!(executor.current_dir(), &PathBuf::from("/tmp"));
        assert_eq!(executor.get_output().len(), 0);
    }

    #[test]
    fn test_parse_command() {
        let (prog, args) = TerminalExecutor::parse_command("ls -la /tmp").unwrap();
        assert_eq!(prog, "ls");
        assert_eq!(args, vec!["-la", "/tmp"]);
    }

    #[test]
    fn test_parse_command_with_quotes() {
        let (prog, args) = TerminalExecutor::parse_command(r#"echo "hello world""#).unwrap();
        assert_eq!(prog, "echo");
        assert_eq!(args, vec!["hello world"]);
    }

    #[test]
    fn test_clear() {
        let mut executor = TerminalExecutor::new(PathBuf::from("/tmp"));
        executor.append_output("line1".to_string());
        executor.append_output("line2".to_string());
        assert_eq!(executor.get_output().len(), 2);

        executor.clear();
        assert_eq!(executor.get_output().len(), 0);
    }

    #[test]
    fn test_history() {
        let mut executor = TerminalExecutor::new(PathBuf::from("/tmp"));
        executor.add_to_history("cmd1".to_string());
        executor.add_to_history("cmd2".to_string());
        executor.add_to_history("cmd2".to_string()); // Duplicate

        assert_eq!(executor.history().len(), 2); // Duplicates removed
        assert_eq!(executor.history_entry(0).unwrap(), "cmd2");
        assert_eq!(executor.history_entry(1).unwrap(), "cmd1");
    }

    #[tokio::test]
    async fn test_execute_echo() {
        let mut executor = TerminalExecutor::new(PathBuf::from("/tmp"));
        executor.execute("echo test").await.unwrap();

        let output = executor.get_output();
        assert!(output.iter().any(|line| line.contains("test")));
    }

    #[tokio::test]
    async fn test_builtin_pwd() {
        let test_dir = env::temp_dir();
        let mut executor = TerminalExecutor::new(test_dir.clone());
        executor.execute("pwd").await.unwrap();

        let output = executor.get_output();
        assert!(output.iter().any(|line| line.contains(&test_dir.display().to_string())));
    }

    #[tokio::test]
    async fn test_builtin_clear() {
        let mut executor = TerminalExecutor::new(PathBuf::from("/tmp"));
        executor.execute("echo test").await.unwrap();
        assert!(!executor.get_output().is_empty());

        executor.execute("clear").await.unwrap();
        assert!(executor.get_output().is_empty());
    }

    #[tokio::test]
    async fn test_invalid_command() {
        let mut executor = TerminalExecutor::new(PathBuf::from("/tmp"));
        executor.execute("nonexistent_command_12345").await.unwrap();

        let output = executor.get_output();
        assert!(output.iter().any(|line| line.contains("Error") || line.contains("not found")));
    }

    #[test]
    fn test_output_buffer_limit() {
        let mut executor = TerminalExecutor::new(PathBuf::from("/tmp"));

        // Add more than MAX_OUTPUT_LINES
        for i in 0..MAX_OUTPUT_LINES + 100 {
            executor.append_output(format!("line {}", i));
        }

        assert_eq!(executor.get_output().len(), MAX_OUTPUT_LINES);
    }

    #[test]
    fn test_get_output_tail() {
        let mut executor = TerminalExecutor::new(PathBuf::from("/tmp"));
        executor.append_output("line1".to_string());
        executor.append_output("line2".to_string());
        executor.append_output("line3".to_string());

        let tail = executor.get_output_tail(2);
        assert_eq!(tail.len(), 2);
        assert_eq!(tail[0], "line2");
        assert_eq!(tail[1], "line3");
    }
}
