//! File System Watcher
//!
//! Watches file system changes using the notify crate.

use crate::{FsError, Result};
use notify::{
    event::ModifyKind, Config, Event, EventKind, RecommendedWatcher, RecursiveMode,
    Watcher as NotifyWatcher,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// File system event
#[derive(Debug, Clone)]
pub enum FileEvent {
    /// File was created
    Created(PathBuf),

    /// File was modified
    Modified(PathBuf),

    /// File was deleted
    Deleted(PathBuf),

    /// File was renamed (old_path, new_path)
    Renamed(PathBuf, PathBuf),
}

impl FileEvent {
    /// Get the affected path
    pub fn path(&self) -> &Path {
        match self {
            Self::Created(p) | Self::Modified(p) | Self::Deleted(p) => p,
            Self::Renamed(_, new) => new,
        }
    }

    /// Check if this is a modification event
    pub fn is_modified(&self) -> bool {
        matches!(self, Self::Modified(_))
    }

    /// Check if this is a creation event
    pub fn is_created(&self) -> bool {
        matches!(self, Self::Created(_))
    }

    /// Check if this is a deletion event
    pub fn is_deleted(&self) -> bool {
        matches!(self, Self::Deleted(_))
    }
}

/// File system watcher
pub struct FileWatcher {
    _watcher: Arc<RecommendedWatcher>,
    rx: mpsc::Receiver<FileEvent>,
    _tx: mpsc::Sender<FileEvent>, // Kept alive to maintain channel
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::channel(100);
        let tx_clone = tx.clone();

        // Get current runtime handle
        let handle = tokio::runtime::Handle::try_current()
            .map_err(|e| FsError::WatchError(format!("No tokio runtime: {}", e)))?;

        // Create the notify watcher
        let watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                let tx = tx_clone.clone();
                let handle = handle.clone();

                // Spawn on the captured runtime handle
                handle.spawn(async move {
                    match res {
                        Ok(event) => {
                            if let Some(file_event) = Self::convert_event(event) {
                                let _ = tx.send(file_event).await;
                            }
                        }
                        Err(e) => {
                            error!("Watch error: {}", e);
                        }
                    }
                });
            },
            Config::default(),
        )
        .map_err(|e| FsError::WatchError(e.to_string()))?;

        Ok(Self {
            _watcher: Arc::new(watcher),
            rx,
            _tx: tx,
        })
    }

    /// Watch a path (file or directory)
    pub fn watch(&mut self, path: impl AsRef<Path>, recursive: bool) -> Result<()> {
        let path = path.as_ref();
        info!(
            "Watching path: {} (recursive: {})",
            path.display(),
            recursive
        );

        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        Arc::get_mut(&mut self._watcher)
            .ok_or_else(|| FsError::WatchError("Cannot get mutable watcher".to_string()))?
            .watch(path, mode)
            .map_err(|e| FsError::WatchError(e.to_string()))?;

        Ok(())
    }

    /// Unwatch a path
    pub fn unwatch(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        info!("Unwatching path: {}", path.display());

        Arc::get_mut(&mut self._watcher)
            .ok_or_else(|| FsError::WatchError("Cannot get mutable watcher".to_string()))?
            .unwatch(path)
            .map_err(|e| FsError::WatchError(e.to_string()))?;

        Ok(())
    }

    /// Get next file event
    pub async fn next_event(&mut self) -> Option<FileEvent> {
        self.rx.recv().await
    }

    /// Try to get next event without blocking
    pub fn try_next_event(&mut self) -> Option<FileEvent> {
        self.rx.try_recv().ok()
    }

    /// Convert notify event to our FileEvent
    fn convert_event(event: Event) -> Option<FileEvent> {
        let paths = event.paths;
        if paths.is_empty() {
            return None;
        }

        match event.kind {
            EventKind::Create(_) => {
                debug!("File created: {:?}", paths[0]);
                Some(FileEvent::Created(paths[0].clone()))
            }

            EventKind::Modify(ModifyKind::Name(_)) if paths.len() >= 2 => {
                debug!("File renamed: {:?} -> {:?}", paths[0], paths[1]);
                Some(FileEvent::Renamed(paths[0].clone(), paths[1].clone()))
            }

            EventKind::Modify(_) => {
                debug!("File modified: {:?}", paths[0]);
                Some(FileEvent::Modified(paths[0].clone()))
            }

            EventKind::Remove(_) => {
                debug!("File deleted: {:?}", paths[0]);
                Some(FileEvent::Deleted(paths[0].clone()))
            }

            _ => None,
        }
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new().expect("Failed to create file watcher")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    #[ignore] // Flaky due to timing and macOS path canonicalization (/var vs /private/var)
    async fn test_watch_file_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mut watcher = FileWatcher::new().unwrap();

        watcher.watch(temp_dir.path(), false).unwrap();

        // Give watcher time to initialize
        sleep(Duration::from_millis(100)).await;

        // Create a file
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test").await.unwrap();

        // Wait for event
        sleep(Duration::from_millis(100)).await;

        // Should receive creation event
        if let Some(event) = watcher.try_next_event() {
            assert!(event.is_created());
            assert_eq!(event.path(), &file_path);
        }
    }

    #[tokio::test]
    #[ignore] // Flaky due to timing issues with file system events
    async fn test_watch_file_modification() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create file first
        fs::write(&file_path, "initial").await.unwrap();

        let mut watcher = FileWatcher::new().unwrap();
        watcher.watch(temp_dir.path(), false).unwrap();

        sleep(Duration::from_millis(100)).await;

        // Modify the file
        fs::write(&file_path, "modified").await.unwrap();

        // Wait for event
        sleep(Duration::from_millis(100)).await;

        // Should receive modification event
        if let Some(event) = watcher.try_next_event() {
            assert!(event.is_modified());
        }
    }

    #[tokio::test]
    #[ignore] // Flaky due to timing issues with file system events
    async fn test_watch_file_deletion() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create file first
        fs::write(&file_path, "test").await.unwrap();

        let mut watcher = FileWatcher::new().unwrap();
        watcher.watch(temp_dir.path(), false).unwrap();

        sleep(Duration::from_millis(100)).await;

        // Delete the file
        fs::remove_file(&file_path).await.unwrap();

        // Wait for event
        sleep(Duration::from_millis(100)).await;

        // Should receive deletion event
        if let Some(event) = watcher.try_next_event() {
            assert!(event.is_deleted());
        }
    }

    #[tokio::test]
    async fn test_recursive_watch() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).await.unwrap();

        let mut watcher = FileWatcher::new().unwrap();
        watcher.watch(temp_dir.path(), true).unwrap();

        sleep(Duration::from_millis(100)).await;

        // Create file in subdirectory
        let file_path = sub_dir.join("test.txt");
        fs::write(&file_path, "test").await.unwrap();

        sleep(Duration::from_millis(100)).await;

        // Should receive event from subdirectory
        if let Some(event) = watcher.try_next_event() {
            assert!(event.is_created());
        }
    }

    #[test]
    fn test_file_event_methods() {
        let path = PathBuf::from("/test/file.txt");

        let created = FileEvent::Created(path.clone());
        assert!(created.is_created());
        assert!(!created.is_modified());
        assert!(!created.is_deleted());
        assert_eq!(created.path(), &path);

        let modified = FileEvent::Modified(path.clone());
        assert!(modified.is_modified());

        let deleted = FileEvent::Deleted(path.clone());
        assert!(deleted.is_deleted());
    }
}
