//! File Operations
//!
//! Handle file reading, writing, and metadata operations.

use crate::{FsError, Result};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::fs;
use tracing::{debug, info};

/// File metadata
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub modified: SystemTime,
    pub is_readonly: bool,
    pub is_hidden: bool,
}

/// Handle for file operations
#[derive(Debug, Clone)]
pub struct FileHandle {
    path: PathBuf,
    metadata: FileMetadata,
}

impl FileHandle {
    /// Open a file and read its metadata
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        debug!("Opening file: {}", path.display());

        let metadata = fs::metadata(path).await?;

        let file_metadata = FileMetadata {
            size: metadata.len(),
            modified: metadata.modified()?,
            is_readonly: metadata.permissions().readonly(),
            is_hidden: path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(false),
        };

        Ok(Self {
            path: path.to_path_buf(),
            metadata: file_metadata,
        })
    }

    /// Read file contents
    pub async fn read(&self) -> Result<String> {
        debug!("Reading file: {}", self.path.display());
        fs::read_to_string(&self.path).await.map_err(Into::into)
    }

    /// Read file contents as bytes
    pub async fn read_bytes(&self) -> Result<Vec<u8>> {
        debug!("Reading file bytes: {}", self.path.display());
        fs::read(&self.path).await.map_err(Into::into)
    }

    /// Write file contents
    pub async fn write(&mut self, content: &str) -> Result<()> {
        debug!("Writing file: {}", self.path.display());

        if self.metadata.is_readonly {
            return Err(FsError::PermissionDenied(self.path.clone()));
        }

        fs::write(&self.path, content).await?;

        // Update metadata
        self.refresh_metadata().await?;

        Ok(())
    }

    /// Atomic file save
    ///
    /// Writes to a temporary file first, then renames to the target.
    /// This ensures we don't lose data if the write fails.
    pub async fn save_atomic(&mut self, content: &str) -> Result<()> {
        debug!("Atomic save: {}", self.path.display());

        if self.metadata.is_readonly {
            return Err(FsError::PermissionDenied(self.path.clone()));
        }

        // Create temp file in same directory
        let temp_path = self.path.with_extension(".tmp");

        // Write to temp file
        fs::write(&temp_path, content).await?;

        // Atomic rename
        fs::rename(&temp_path, &self.path).await?;

        // Update metadata
        self.refresh_metadata().await?;

        info!("File saved atomically: {}", self.path.display());
        Ok(())
    }

    /// Refresh file metadata
    pub async fn refresh_metadata(&mut self) -> Result<()> {
        let metadata = fs::metadata(&self.path).await?;

        self.metadata = FileMetadata {
            size: metadata.len(),
            modified: metadata.modified()?,
            is_readonly: metadata.permissions().readonly(),
            is_hidden: self
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(false),
        };

        Ok(())
    }

    /// Check if file has been modified since last read
    pub async fn has_changed(&self) -> Result<bool> {
        let metadata = fs::metadata(&self.path).await?;
        let current_modified = metadata.modified()?;

        Ok(current_modified > self.metadata.modified)
    }

    /// Get file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get file metadata
    pub fn metadata(&self) -> &FileMetadata {
        &self.metadata
    }

    /// Check if file exists
    pub async fn exists(&self) -> bool {
        fs::metadata(&self.path).await.is_ok()
    }

    /// Delete file
    pub async fn delete(self) -> Result<()> {
        info!("Deleting file: {}", self.path.display());
        fs::remove_file(&self.path).await.map_err(Into::into)
    }

    /// Rename/move file
    pub async fn rename(&mut self, new_path: impl AsRef<Path>) -> Result<()> {
        let new_path = new_path.as_ref();
        info!("Renaming {} to {}", self.path.display(), new_path.display());

        fs::rename(&self.path, new_path).await?;
        self.path = new_path.to_path_buf();

        Ok(())
    }

    /// Copy file
    pub async fn copy(&self, dest: impl AsRef<Path>) -> Result<FileHandle> {
        let dest = dest.as_ref();
        info!("Copying {} to {}", self.path.display(), dest.display());

        fs::copy(&self.path, dest).await?;
        FileHandle::open(dest).await
    }
}

/// Create a new file
pub async fn create_file(path: impl AsRef<Path>, content: &str) -> Result<FileHandle> {
    let path = path.as_ref();
    debug!("Creating file: {}", path.display());

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    fs::write(path, content).await?;
    FileHandle::open(path).await
}

/// Check if file exists
pub async fn exists(path: impl AsRef<Path>) -> bool {
    fs::metadata(path).await.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_handle_read_write() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create file
        let mut handle = create_file(&file_path, "Hello, World!").await.unwrap();
        assert!(handle.exists().await);

        // Read file
        let content = handle.read().await.unwrap();
        assert_eq!(content, "Hello, World!");

        // Write file
        handle.write("New content").await.unwrap();
        let content = handle.read().await.unwrap();
        assert_eq!(content, "New content");
    }

    #[tokio::test]
    async fn test_atomic_save() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("atomic.txt");

        let mut handle = create_file(&file_path, "initial").await.unwrap();

        // Atomic save
        handle.save_atomic("atomic save").await.unwrap();

        let content = handle.read().await.unwrap();
        assert_eq!(content, "atomic save");

        // Temp file should be cleaned up
        let temp_path = file_path.with_extension(".tmp");
        assert!(!exists(&temp_path).await);
    }

    #[tokio::test]
    async fn test_file_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("meta.txt");

        let handle = create_file(&file_path, "test content").await.unwrap();

        let metadata = handle.metadata();
        assert!(metadata.size > 0);
        assert!(!metadata.is_readonly);
    }

    #[tokio::test]
    async fn test_has_changed() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("change.txt");

        let mut handle = create_file(&file_path, "initial").await.unwrap();

        // File should not have changed immediately
        assert!(!handle.has_changed().await.unwrap());

        // Write directly to file
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        fs::write(&file_path, "changed").await.unwrap();

        // Should detect change
        assert!(handle.has_changed().await.unwrap());

        // Refresh metadata
        handle.refresh_metadata().await.unwrap();
        assert!(!handle.has_changed().await.unwrap());
    }

    #[tokio::test]
    async fn test_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("ops.txt");

        // Create
        let mut handle = create_file(&file_path, "test").await.unwrap();
        assert!(handle.exists().await);

        // Copy
        let copy_path = temp_dir.path().join("copy.txt");
        let copy_handle = handle.copy(&copy_path).await.unwrap();
        assert!(copy_handle.exists().await);

        // Rename
        let new_path = temp_dir.path().join("renamed.txt");
        handle.rename(&new_path).await.unwrap();
        assert_eq!(handle.path(), new_path);

        // Delete
        handle.delete().await.unwrap();
        assert!(!exists(&new_path).await);
    }

    #[tokio::test]
    async fn test_hidden_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join(".hidden");

        let handle = create_file(&file_path, "hidden").await.unwrap();
        assert!(handle.metadata().is_hidden);
    }

    #[tokio::test]
    async fn test_read_bytes() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("bytes.bin");

        let data = vec![0u8, 1, 2, 3, 4, 255];
        fs::write(&file_path, &data).await.unwrap();

        let handle = FileHandle::open(&file_path).await.unwrap();
        let read_data = handle.read_bytes().await.unwrap();

        assert_eq!(read_data, data);
    }
}
