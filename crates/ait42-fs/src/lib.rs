//! AIT42 File System Operations
//!
//! Provides file system operations with async support and file watching.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;

#[derive(Error, Debug)]
pub enum FsError {
    #[error("File not found: {0}")]
    NotFound(PathBuf),

    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, FsError>;

/// File tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_hidden: bool,
    pub size: u64,
    pub children: Option<Vec<FileNode>>,
}

/// Read directory contents
pub async fn read_dir(path: impl AsRef<Path>) -> Result<Vec<FileNode>> {
    let path = path.as_ref();
    let mut entries = fs::read_dir(path).await?;
    let mut nodes = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let metadata = entry.metadata().await?;
        let name = entry.file_name().to_string_lossy().to_string();
        let is_hidden = name.starts_with('.');

        nodes.push(FileNode {
            name,
            path: entry.path(),
            is_dir: metadata.is_dir(),
            is_hidden,
            size: metadata.len(),
            children: None,
        });
    }

    Ok(nodes)
}

/// Read file contents
pub async fn read_file(path: impl AsRef<Path>) -> Result<String> {
    fs::read_to_string(path).await.map_err(Into::into)
}

/// Write file contents
pub async fn write_file(path: impl AsRef<Path>, content: impl AsRef<str>) -> Result<()> {
    fs::write(path, content.as_ref()).await.map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_current_dir() {
        let result = read_dir(".").await;
        assert!(result.is_ok());
    }
}
