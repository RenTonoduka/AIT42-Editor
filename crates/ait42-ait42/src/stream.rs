//! Real-time output streaming from tmux sessions

use crate::coordinator::ExecutionResult;
use crate::error::Result;
use crate::tmux::{SessionStatus, TmuxManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{debug, error};

/// Stream event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamEvent {
    /// New output line
    Output(String),
    /// Session status changed
    StatusChange(SessionStatus),
    /// Session completed
    Completed(ExecutionResult),
    /// Error occurred
    Error(String),
}

/// Output stream receiver
pub struct OutputStream {
    rx: mpsc::Receiver<StreamEvent>,
    session_id: String,
}

impl OutputStream {
    /// Get the next event from the stream
    pub async fn next(&mut self) -> Option<StreamEvent> {
        self.rx.recv().await
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

/// Stream manager for multiple concurrent streams
pub struct StreamManager {
    streams: HashMap<String, mpsc::Sender<StreamEvent>>,
    tmux: TmuxManager,
    poll_interval: Duration,
}

impl StreamManager {
    /// Create a new stream manager
    pub fn new(tmux: TmuxManager) -> Self {
        Self {
            streams: HashMap::new(),
            tmux,
            poll_interval: Duration::from_millis(500),
        }
    }

    /// Create a new output stream for a session
    pub fn create_stream(&mut self, session_id: String) -> OutputStream {
        let (tx, rx) = mpsc::channel(100);
        self.streams.insert(session_id.clone(), tx);

        OutputStream { rx, session_id }
    }

    /// Start polling outputs from tmux sessions
    pub async fn start_polling(&mut self) {
        let mut interval_timer = interval(self.poll_interval);
        let mut last_outputs: HashMap<String, Vec<String>> = HashMap::new();

        loop {
            interval_timer.tick().await;

            // Get list of session IDs to poll
            let session_ids: Vec<String> = self.streams.keys().cloned().collect();

            for session_id in session_ids {
                // Check if session still exists
                if !self.tmux.is_session_alive(&session_id).await {
                    self.send_event(&session_id, StreamEvent::StatusChange(SessionStatus::Completed))
                        .await;
                    continue;
                }

                // Get current output
                match self.tmux.get_output(&session_id).await {
                    Ok(output) => {
                        // Compare with last output
                        let last = last_outputs.get(&session_id);
                        let new_lines = self.get_new_lines(last, &output);

                        // Send new lines
                        for line in new_lines {
                            self.send_event(&session_id, StreamEvent::Output(line.clone()))
                                .await;
                        }

                        // Update last output
                        last_outputs.insert(session_id.clone(), output);
                    }
                    Err(e) => {
                        error!("Failed to get output for {}: {}", session_id, e);
                        self.send_event(&session_id, StreamEvent::Error(e.to_string()))
                            .await;
                    }
                }
            }

            // Remove completed streams
            self.cleanup_completed_streams().await;
        }
    }

    /// Poll a single session once
    pub async fn poll_session(&mut self, session_id: &str) -> Result<Vec<String>> {
        self.tmux.get_output(session_id).await
    }

    /// Send an event to a stream
    async fn send_event(&self, session_id: &str, event: StreamEvent) {
        if let Some(tx) = self.streams.get(session_id) {
            if let Err(e) = tx.send(event).await {
                debug!("Failed to send event to stream {}: {}", session_id, e);
            }
        }
    }

    /// Get new lines by comparing outputs
    fn get_new_lines(&self, last: Option<&Vec<String>>, current: &[String]) -> Vec<String> {
        match last {
            None => current.to_vec(),
            Some(last) => {
                if current.len() > last.len() {
                    current[last.len()..].to_vec()
                } else {
                    Vec::new()
                }
            }
        }
    }

    /// Remove streams for completed sessions
    async fn cleanup_completed_streams(&mut self) {
        let mut to_remove = Vec::new();

        for session_id in self.streams.keys() {
            if !self.tmux.is_session_alive(session_id).await {
                to_remove.push(session_id.clone());
            }
        }

        for session_id in to_remove {
            debug!("Removing completed stream: {}", session_id);
            self.streams.remove(&session_id);
        }
    }

    /// Close a stream
    pub fn close_stream(&mut self, session_id: &str) {
        self.streams.remove(session_id);
    }

    /// Get number of active streams
    pub fn active_count(&self) -> usize {
        self.streams.len()
    }
}

/// Simplified stream helper for single session
pub struct SessionStream {
    manager: StreamManager,
    stream: OutputStream,
}

impl SessionStream {
    /// Create a new session stream
    pub fn new(tmux: TmuxManager, session_id: String) -> Self {
        let mut manager = StreamManager::new(tmux);
        let stream = manager.create_stream(session_id);

        Self { manager, stream }
    }

    /// Get the next event
    pub async fn next(&mut self) -> Option<StreamEvent> {
        self.stream.next().await
    }

    /// Poll the session once
    pub async fn poll_once(&mut self) -> Result<Vec<String>> {
        self.manager.poll_session(self.stream.session_id()).await
    }

    /// Start background polling (spawns task)
    pub fn start_polling(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            self.manager.start_polling().await;
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_get_new_lines() {
        let manager = StreamManager::new(TmuxManager::new(Path::new("/tmp")));

        let last = vec!["line 1".to_string(), "line 2".to_string()];
        let current = vec![
            "line 1".to_string(),
            "line 2".to_string(),
            "line 3".to_string(),
        ];

        let new_lines = manager.get_new_lines(Some(&last), &current);
        assert_eq!(new_lines.len(), 1);
        assert_eq!(new_lines[0], "line 3");
    }

    #[test]
    fn test_get_new_lines_no_previous() {
        let manager = StreamManager::new(TmuxManager::new(Path::new("/tmp")));

        let current = vec!["line 1".to_string(), "line 2".to_string()];
        let new_lines = manager.get_new_lines(None, &current);
        assert_eq!(new_lines.len(), 2);
    }

    #[tokio::test]
    async fn test_stream_creation() {
        let tmux = TmuxManager::new(Path::new("/tmp"));
        let mut manager = StreamManager::new(tmux);

        let stream = manager.create_stream("test-session".to_string());
        assert_eq!(stream.session_id(), "test-session");
        assert_eq!(manager.active_count(), 1);
    }
}
