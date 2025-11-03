//! Directory Operations
//!
//! Operations for listing and searching directories.

use crate::{FileNode, FsError, Result};
use ignore::WalkBuilder;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use tokio::fs;
use tracing::debug;
use walkdir::WalkDir;

/// Directory listing result
#[derive(Debug, Clone)]
pub struct DirectoryListing {
    pub files: Vec<PathBuf>,
    pub directories: Vec<PathBuf>,
}

impl DirectoryListing {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            directories: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty() && self.directories.is_empty()
    }

    pub fn total_count(&self) -> usize {
        self.files.len() + self.directories.len()
    }
}

impl Default for DirectoryListing {
    fn default() -> Self {
        Self::new()
    }
}

/// List directory contents
pub async fn list_directory(path: &Path) -> Result<DirectoryListing> {
    debug!("Listing directory: {}", path.display());

    let mut listing = DirectoryListing::new();
    let mut entries = fs::read_dir(path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let metadata = entry.metadata().await?;
        let path = entry.path();

        if metadata.is_dir() {
            listing.directories.push(path);
        } else {
            listing.files.push(path);
        }
    }

    // Sort for consistent ordering
    listing.files.sort();
    listing.directories.sort();

    Ok(listing)
}

/// Find files matching a glob pattern
///
/// Uses the `ignore` crate which respects .gitignore files.
///
/// # Arguments
/// * `root` - Root directory to search from
/// * `pattern` - Glob pattern (e.g., "*.rs", "**/*.txt")
///
/// # Example
/// ```no_run
/// # use ait42_fs::directory::find_files;
/// # use std::path::Path;
/// # tokio_test::block_on(async {
/// let files = find_files(Path::new("."), "*.rs").await.unwrap();
/// # });
/// ```
pub async fn find_files(root: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    debug!("Finding files: {} in {}", pattern, root.display());

    let root = root.to_path_buf();
    let pattern = pattern.to_string();

    // Run in blocking task since ignore crate is synchronous
    let result = tokio::task::spawn_blocking(move || {
        let mut matches = Vec::new();
        let glob_pattern = glob::Pattern::new(&pattern)
            .map_err(|e| FsError::InvalidPath(format!("Invalid pattern: {}", e)))?;

        for entry in WalkBuilder::new(&root)
            .hidden(false) // Include hidden files
            .git_ignore(true) // Respect .gitignore
            .build()
        {
            let entry = entry.map_err(|e| FsError::InvalidPath(format!("Walk error: {}", e)))?;
            let path = entry.path();

            if path.is_file() {
                // Match against filename or relative path
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if glob_pattern.matches(name) {
                        matches.push(path.to_path_buf());
                    }
                } else if let Ok(rel_path) = path.strip_prefix(&root) {
                    if let Some(rel_str) = rel_path.to_str() {
                        if glob_pattern.matches(rel_str) {
                            matches.push(path.to_path_buf());
                        }
                    }
                }
            }
        }

        Ok::<Vec<PathBuf>, FsError>(matches)
    })
    .await
    .map_err(|e| FsError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))??;

    Ok(result)
}

/// Build a file tree recursively
pub async fn build_tree(root: &Path, max_depth: usize) -> Result<FileNode> {
    let root = root.to_path_buf();
    let root_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let metadata = fs::metadata(&root).await?;

    if !metadata.is_dir() {
        return Ok(FileNode {
            name: root_name.clone(),
            path: root.clone(),
            is_dir: false,
            is_hidden: root_name.starts_with('.'),
            size: metadata.len(),
            children: None,
        });
    }

    let children = if max_depth > 0 {
        Some(build_tree_recursive(&root, max_depth, 0).await?)
    } else {
        None
    };

    Ok(FileNode {
        name: root_name,
        path: root.clone(),
        is_dir: true,
        is_hidden: root
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.'))
            .unwrap_or(false),
        size: 0,
        children,
    })
}

fn build_tree_recursive(
    path: &Path,
    max_depth: usize,
    current_depth: usize,
) -> Pin<Box<dyn Future<Output = Result<Vec<FileNode>>> + Send>> {
    let path = path.to_path_buf();
    Box::pin(async move {
        if current_depth >= max_depth {
            return Ok(Vec::new());
        }

        let mut entries = fs::read_dir(&path).await?;
        let mut nodes = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let metadata = entry.metadata().await?;
            let entry_path = entry.path();
            let name = entry
                .file_name()
                .to_string_lossy()
                .to_string();

            let is_dir = metadata.is_dir();
            let is_hidden = name.starts_with('.');

            let children = if is_dir && current_depth + 1 < max_depth {
                Some(build_tree_recursive(&entry_path, max_depth, current_depth + 1).await?)
            } else {
                None
            };

            nodes.push(FileNode {
                name,
                path: entry_path,
                is_dir,
                is_hidden,
                size: metadata.len(),
                children,
            });
        }

        // Sort: directories first, then by name
        nodes.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });

        Ok(nodes)
    })
}

/// Find files by extension
pub async fn find_by_extension(root: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let pattern = format!("*.{}", extension);
    find_files(root, &pattern).await
}

/// Get directory size (recursively)
pub async fn directory_size(path: &Path) -> Result<u64> {
    let path = path.to_path_buf();

    tokio::task::spawn_blocking(move || {
        let mut total = 0u64;

        for entry in WalkDir::new(&path) {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total += metadata.len();
                    }
                }
            }
        }

        Ok(total)
    })
    .await
    .map_err(|e| FsError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_list_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create some files and directories
        fs::write(temp_dir.path().join("file1.txt"), "test")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "test")
            .await
            .unwrap();
        fs::create_dir(temp_dir.path().join("subdir"))
            .await
            .unwrap();

        let listing = list_directory(temp_dir.path()).await.unwrap();

        assert_eq!(listing.files.len(), 2);
        assert_eq!(listing.directories.len(), 1);
        assert_eq!(listing.total_count(), 3);
        assert!(!listing.is_empty());
    }

    #[tokio::test]
    async fn test_find_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        fs::write(temp_dir.path().join("test1.rs"), "code")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("test2.rs"), "code")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("readme.md"), "docs")
            .await
            .unwrap();

        let rs_files = find_files(temp_dir.path(), "*.rs").await.unwrap();
        assert_eq!(rs_files.len(), 2);

        let md_files = find_files(temp_dir.path(), "*.md").await.unwrap();
        assert_eq!(md_files.len(), 1);
    }

    #[tokio::test]
    async fn test_find_by_extension() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("test.rs"), "code")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("main.rs"), "code")
            .await
            .unwrap();

        let files = find_by_extension(temp_dir.path(), "rs")
            .await
            .unwrap();
        assert_eq!(files.len(), 2);
    }

    #[tokio::test]
    async fn test_build_tree() {
        let temp_dir = TempDir::new().unwrap();

        // Create directory structure
        fs::create_dir(temp_dir.path().join("subdir"))
            .await
            .unwrap();
        fs::write(temp_dir.path().join("file.txt"), "test")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("subdir").join("nested.txt"), "test")
            .await
            .unwrap();

        let tree = build_tree(temp_dir.path(), 2).await.unwrap();

        assert!(tree.is_dir);
        assert!(tree.children.is_some());

        let children = tree.children.unwrap();
        assert_eq!(children.len(), 2); // 1 dir + 1 file
    }

    #[tokio::test]
    async fn test_directory_size() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("file1.txt"), "1234")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "567890")
            .await
            .unwrap();

        let size = directory_size(temp_dir.path()).await.unwrap();
        assert_eq!(size, 10); // 4 + 6 bytes
    }

    #[test]
    fn test_directory_listing() {
        let mut listing = DirectoryListing::new();
        assert!(listing.is_empty());
        assert_eq!(listing.total_count(), 0);

        listing.files.push(PathBuf::from("test.txt"));
        assert!(!listing.is_empty());
        assert_eq!(listing.total_count(), 1);
    }
}
