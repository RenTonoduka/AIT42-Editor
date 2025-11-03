//! Test helper utilities

use ait42_core::buffer::Buffer;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Create a temporary test environment
pub struct TestEnv {
    pub temp_dir: TempDir,
}

impl TestEnv {
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().unwrap(),
        }
    }

    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn create_file(&self, name: &str, content: &str) -> PathBuf {
        let path = self.temp_dir.path().join(name);
        fs::write(&path, content).unwrap();
        path
    }

    pub fn create_buffer_from_file(&self, name: &str, content: &str) -> Buffer {
        let path = self.create_file(name, content);
        Buffer::from_file(&path).unwrap()
    }
}

/// Generate test text of specified size
pub fn generate_text(size: usize) -> String {
    "Lorem ipsum dolor sit amet ".repeat(size / 27 + 1)[..size].to_string()
}

/// Generate multiline test text
pub fn generate_lines(count: usize) -> String {
    (0..count)
        .map(|i| format!("Line {} content", i))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Assert buffer contains expected text
#[macro_export]
macro_rules! assert_buffer_content {
    ($buffer:expr, $expected:expr) => {
        assert_eq!(
            $buffer.to_string(),
            $expected,
            "Buffer content mismatch"
        );
    };
}

/// Assert buffer is dirty
#[macro_export]
macro_rules! assert_dirty {
    ($buffer:expr) => {
        assert!($buffer.is_dirty(), "Buffer should be dirty");
    };
}

/// Assert buffer is clean
#[macro_export]
macro_rules! assert_clean {
    ($buffer:expr) => {
        assert!(!$buffer.is_dirty(), "Buffer should be clean");
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_creation() {
        let env = TestEnv::new();
        assert!(env.path().exists());
    }

    #[test]
    fn test_create_file() {
        let env = TestEnv::new();
        let path = env.create_file("test.txt", "content");
        assert!(path.exists());
        assert_eq!(fs::read_to_string(path).unwrap(), "content");
    }

    #[test]
    fn test_generate_text() {
        let text = generate_text(100);
        assert_eq!(text.len(), 100);
    }

    #[test]
    fn test_generate_lines() {
        let lines = generate_lines(5);
        assert_eq!(lines.lines().count(), 5);
    }
}
