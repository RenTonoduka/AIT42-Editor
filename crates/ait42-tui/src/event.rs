//! Event Loop and Input Handling
//!
//! Provides async event handling for terminal input (keyboard, mouse, resize).

use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, MouseEvent};
use futures::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error};

/// Editor events from terminal and timers
#[derive(Debug, Clone, PartialEq)]
pub enum EditorEvent {
    /// Keyboard input
    Key(KeyEvent),
    /// Mouse input
    Mouse(MouseEvent),
    /// Terminal window resized
    Resize(u16, u16),
    /// Paste text
    Paste(String),
    /// Timer tick for UI refresh
    Tick,
    /// Request to quit
    Quit,
}

/// Asynchronous event loop for handling terminal events
pub struct EventLoop {
    rx: mpsc::Receiver<EditorEvent>,
    tick_rate: Duration,
}

impl EventLoop {
    /// Create a new event loop with the specified tick rate
    ///
    /// # Arguments
    /// * `tick_rate` - Duration between tick events (default: 250ms)
    ///
    /// # Returns
    /// New EventLoop instance
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel(100);

        // Spawn input handler task
        Self::spawn_input_handler(tx.clone());

        // Spawn tick handler
        Self::spawn_tick_handler(tx, tick_rate);

        Self { rx, tick_rate }
    }

    /// Get the next event from the queue
    ///
    /// # Returns
    /// Next event or None if channel closed
    pub async fn next(&mut self) -> Option<EditorEvent> {
        self.rx.recv().await
    }

    /// Spawn task to handle terminal input events
    fn spawn_input_handler(tx: mpsc::Sender<EditorEvent>) {
        tokio::spawn(async move {
            let mut reader = EventStream::new();

            loop {
                match reader.next().await {
                    Some(Ok(event)) => {
                        let editor_event = match event {
                            CrosstermEvent::Key(key) => EditorEvent::Key(key),
                            CrosstermEvent::Mouse(mouse) => EditorEvent::Mouse(mouse),
                            CrosstermEvent::Resize(w, h) => EditorEvent::Resize(w, h),
                            CrosstermEvent::Paste(text) => EditorEvent::Paste(text),
                            _ => continue,
                        };

                        if tx.send(editor_event).await.is_err() {
                            debug!("Event receiver dropped");
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        error!("Error reading terminal event: {}", e);
                    }
                    None => {
                        debug!("Event stream ended");
                        break;
                    }
                }
            }
        });
    }

    /// Spawn task to generate periodic tick events
    fn spawn_tick_handler(tx: mpsc::Sender<EditorEvent>, tick_rate: Duration) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tick_rate);

            loop {
                interval.tick().await;

                if tx.send(EditorEvent::Tick).await.is_err() {
                    debug!("Tick receiver dropped");
                    break;
                }
            }
        });
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new(Duration::from_millis(250))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_event_loop_tick() {
        let mut event_loop = EventLoop::new(Duration::from_millis(100));

        // Should receive a tick within 200ms
        let result = timeout(Duration::from_millis(200), event_loop.next()).await;
        assert!(result.is_ok());

        if let Some(event) = result.unwrap() {
            assert_eq!(event, EditorEvent::Tick);
        }
    }

    #[tokio::test]
    async fn test_event_loop_multiple_ticks() {
        let mut event_loop = EventLoop::new(Duration::from_millis(50));

        // Collect multiple ticks
        let mut tick_count = 0;
        for _ in 0..3 {
            if let Some(EditorEvent::Tick) = timeout(Duration::from_millis(100), event_loop.next())
                .await
                .ok()
                .flatten()
            {
                tick_count += 1;
            }
        }

        assert!(tick_count >= 2);
    }
}
