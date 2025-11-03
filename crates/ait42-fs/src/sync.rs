//! File Synchronization
//!
//! Synchronizes buffer content with file system, handling file watching and auto-save.

use crate::{FileEvent, FileHandle, FileWatcher, FsError, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::time::{interval, Duration, Interval};
use tracing::{debug, info};

/// File synchronizer
///
/// Manages synchronization between open files and the file system,
/// including watching for external changes and auto-save functionality.
pub struct FileSynchronizer {
    watcher: FileWatcher,
    open_files: HashMap<PathBuf, FileHandle>,
    auto_save_interval: Option<Interval>,
    auto_save_delay: Duration,
}

impl FileSynchronizer {
    /// Create a new file synchronizer
    ///
    /// # Arguments
    /// * `auto_save_delay` - Delay before auto-saving (in seconds), or None to disable
    pub fn new(auto_save_delay: Option<u64>) -> Result<Self> {
        let watcher = FileWatcher::new()?;

        let (auto_save_interval, auto_save_delay) = if let Some(delay) = auto_save_delay {
            let duration = Duration::from_secs(delay);
            (Some(interval(duration)), duration)
        } else {
            (None, Duration::from_secs(0))
        };

        Ok(Self {
            watcher,
            open_files: HashMap::new(),
            auto_save_interval,
            auto_save_delay,
        })
    }

    /// Open a file and start watching it
    pub async fn open_file(&mut self, path: &Path) -> Result<String> {
        info!("Opening file for sync: {}", path.display());

        // Open the file
        let handle = FileHandle::open(path).await?;
        let content = handle.read().await?;

        // Watch the file's parent directory
        if let Some(parent) = path.parent() {
            self.watcher.watch(parent, false)?;
        }

        // Store the handle
        self.open_files.insert(path.to_path_buf(), handle);

        Ok(content)
    }

    /// Save file content
    pub async fn save_file(&mut self, path: &Path, content: &str) -> Result<()> {
        debug!("Saving file: {}", path.display());

        let handle = self
            .open_files
            .get_mut(path)
            .ok_or_else(|| FsError::NotFound(path.to_path_buf()))?;

        handle.save_atomic(content).await?;

        info!("File saved: {}", path.display());
        Ok(())
    }

    /// Close a file and stop watching it
    pub async fn close_file(&mut self, path: &Path) -> Result<()> {
        info!("Closing file: {}", path.display());

        self.open_files.remove(path);

        // If no more files in this directory, stop watching
        if let Some(parent) = path.parent() {
            let has_files_in_dir = self
                .open_files
                .keys()
                .any(|p| p.parent() == Some(parent));

            if !has_files_in_dir {
                let _ = self.watcher.unwatch(parent);
            }
        }

        Ok(())
    }

    /// Check for file system changes
    ///
    /// Returns events for files that have changed externally.
    pub async fn poll_changes(&mut self) -> Option<(PathBuf, FileEvent)> {
        if let Some(event) = self.watcher.try_next_event() {
            let path = event.path().to_path_buf();

            // Only report events for files we're tracking
            if self.open_files.contains_key(&path) {
                debug!("External change detected: {:?}", event);
                return Some((path, event));
            }
        }

        None
    }

    /// Wait for next file change
    pub async fn next_change(&mut self) -> Option<(PathBuf, FileEvent)> {
        loop {
            if let Some(event) = self.watcher.next_event().await {
                let path = event.path().to_path_buf();

                if self.open_files.contains_key(&path) {
                    return Some((path, event));
                }
            } else {
                return None;
            }
        }
    }

    /// Check if file has been modified externally
    pub async fn has_external_changes(&self, path: &Path) -> Result<bool> {
        let handle = self
            .open_files
            .get(path)
            .ok_or_else(|| FsError::NotFound(path.to_path_buf()))?;

        handle.has_changed().await
    }

    /// Refresh file metadata
    pub async fn refresh_metadata(&mut self, path: &Path) -> Result<()> {
        let handle = self
            .open_files
            .get_mut(path)
            .ok_or_else(|| FsError::NotFound(path.to_path_buf()))?;

        handle.refresh_metadata().await
    }

    /// Get list of open files
    pub fn open_files(&self) -> Vec<&Path> {
        self.open_files.keys().map(|p| p.as_path()).collect()
    }

    /// Check if file is open
    pub fn is_open(&self, path: &Path) -> bool {
        self.open_files.contains_key(path)
    }

    /// Get file handle
    pub fn get_handle(&self, path: &Path) -> Option<&FileHandle> {
        self.open_files.get(path)
    }

    /// Tick auto-save timer
    ///
    /// Should be called periodically to check if auto-save should trigger.
    pub async fn tick_auto_save(&mut self) -> bool {
        if let Some(interval) = &mut self.auto_save_interval {
            interval.tick().await;
            true
        } else {
            false
        }
    }

    /// Enable auto-save
    pub fn enable_auto_save(&mut self, delay_seconds: u64) {
        let duration = Duration::from_secs(delay_seconds);
        self.auto_save_interval = Some(interval(duration));
        self.auto_save_delay = duration;
        info!("Auto-save enabled with {}s delay", delay_seconds);
    }

    /// Disable auto-save
    pub fn disable_auto_save(&mut self) {
        self.auto_save_interval = None;
        info!("Auto-save disabled");
    }

    /// Check if auto-save is enabled
    pub fn is_auto_save_enabled(&self) -> bool {
        self.auto_save_interval.is_some()
    }

    /// Get auto-save delay
    pub fn auto_save_delay(&self) -> Option<Duration> {
        if self.auto_save_interval.is_some() {
            Some(self.auto_save_delay)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_open_and_save() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create initial file
        fs::write(&file_path, "initial content").await.unwrap();

        let mut sync = FileSynchronizer::new(None).unwrap();

        // Open file
        let content = sync.open_file(&file_path).await.unwrap();
        assert_eq!(content, "initial content");
        assert!(sync.is_open(&file_path));

        // Save file
        sync.save_file(&file_path, "new content").await.unwrap();

        // Verify file was saved
        let saved_content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(saved_content, "new content");
    }

    #[tokio::test]
    async fn test_close_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "test").await.unwrap();

        let mut sync = FileSynchronizer::new(None).unwrap();

        sync.open_file(&file_path).await.unwrap();
        assert!(sync.is_open(&file_path));

        sync.close_file(&file_path).await.unwrap();
        assert!(!sync.is_open(&file_path));
    }

    #[tokio::test]
    async fn test_external_changes() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "initial").await.unwrap();

        let mut sync = FileSynchronizer::new(None).unwrap();
        sync.open_file(&file_path).await.unwrap();

        // Modify file externally
        sleep(Duration::from_millis(100)).await;
        fs::write(&file_path, "modified externally").await.unwrap();

        // Check for changes
        sleep(Duration::from_millis(200)).await;
        let has_changes = sync.has_external_changes(&file_path).await.unwrap();
        assert!(has_changes);
    }

    #[tokio::test]
    async fn test_poll_changes() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "initial").await.unwrap();

        let mut sync = FileSynchronizer::new(None).unwrap();
        sync.open_file(&file_path).await.unwrap();

        sleep(Duration::from_millis(100)).await;

        // Modify file externally
        fs::write(&file_path, "modified").await.unwrap();

        sleep(Duration::from_millis(200)).await;

        // Poll for changes
        if let Some((path, event)) = sync.poll_changes().await {
            assert_eq!(path, file_path);
            assert!(event.is_modified());
        }
    }

    #[tokio::test]
    async fn test_auto_save() {
        let mut sync = FileSynchronizer::new(Some(1)).unwrap();

        assert!(sync.is_auto_save_enabled());
        assert_eq!(sync.auto_save_delay(), Some(Duration::from_secs(1)));

        sync.disable_auto_save();
        assert!(!sync.is_auto_save_enabled());

        sync.enable_auto_save(2);
        assert!(sync.is_auto_save_enabled());
        assert_eq!(sync.auto_save_delay(), Some(Duration::from_secs(2)));
    }

    #[tokio::test]
    async fn test_multiple_files() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        fs::write(&file1, "content1").await.unwrap();
        fs::write(&file2, "content2").await.unwrap();

        let mut sync = FileSynchronizer::new(None).unwrap();

        sync.open_file(&file1).await.unwrap();
        sync.open_file(&file2).await.unwrap();

        assert_eq!(sync.open_files().len(), 2);
        assert!(sync.is_open(&file1));
        assert!(sync.is_open(&file2));

        sync.close_file(&file1).await.unwrap();
        assert_eq!(sync.open_files().len(), 1);
        assert!(!sync.is_open(&file1));
        assert!(sync.is_open(&file2));
    }

    #[tokio::test]
    async fn test_get_handle() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "test").await.unwrap();

        let mut sync = FileSynchronizer::new(None).unwrap();
        sync.open_file(&file_path).await.unwrap();

        let handle = sync.get_handle(&file_path);
        assert!(handle.is_some());
        assert_eq!(handle.unwrap().path(), file_path.as_path());
    }
}
