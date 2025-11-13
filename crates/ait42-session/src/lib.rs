//! AIT42 Session Management
//!
//! This crate provides session data persistence using SQLite.
//!
//! # Features
//! - SQLite-based storage with SQLx
//! - Async/await support
//! - Transaction safety
//! - Workspace-scoped sessions
//! - Compatible with existing JSON-based format
//!
//! # Example
//! ```no_run
//! use ait42_session::{SqliteSessionRepository, WorktreeSession, SessionRepository};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create repository with default database path
//!     let repo = SqliteSessionRepository::new_default().await?;
//!
//!     // Create a new session
//!     let mut session = WorktreeSession::new(
//!         "session-123".to_string(),
//!         "workspace-hash-abc".to_string(),
//!         "competition".to_string(),
//!         "Implement feature X".to_string(),
//!     );
//!     session.workspace_hash = Some("workspace-hash-abc".to_string());
//!
//!     // Save to database
//!     let created = repo.create_session(session).await?;
//!     println!("Created session: {}", created.id);
//!
//!     Ok(())
//! }
//! ```

pub mod db;
pub mod error;
pub mod models;
pub mod repository;

// Re-export commonly used types
pub use db::DbPool;
pub use error::{Result, SessionError};
pub use models::{
    compute_workspace_hash, ChatMessage, WorktreeInstance, WorktreeSession,
};
pub use repository::{SessionRepository, SqliteSessionRepository};

// Re-export external dependencies for convenience
pub use chrono;
pub use sqlx;
pub use uuid;
