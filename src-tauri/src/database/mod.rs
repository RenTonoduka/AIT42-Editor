/**
 * Database Module - SQLite connection and management
 *
 * Provides connection pooling, migrations, and health checks for session database.
 */
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::path::PathBuf;

pub mod queries;
pub mod migration;

/// Initialize SQLite connection pool with migrations
pub async fn create_connection_pool() -> Result<SqlitePool, sqlx::Error> {
    let db_path = get_database_path();

    // Ensure directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| sqlx::Error::Io(e))?;
    }

    let database_url = format!("sqlite:{}?mode=rwc", db_path.display());

    tracing::info!("Connecting to database: {}", database_url);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .idle_timeout(std::time::Duration::from_secs(60))
        .connect(&database_url)
        .await?;

    // Enable PRAGMA settings for each connection
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;

    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(&pool)
        .await?;

    sqlx::query("PRAGMA synchronous = NORMAL")
        .execute(&pool)
        .await?;

    // Run migrations
    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    tracing::info!("Database initialized successfully");

    Ok(pool)
}

/// Get path to SQLite database file (~/.ait42/sessions.db)
pub fn get_database_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".ait42")
        .join("sessions.db")
}

/// Check database health and integrity
pub async fn check_database_health(pool: &SqlitePool) -> Result<DatabaseHealth, sqlx::Error> {
    // Check connectivity and count sessions
    let session_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sessions")
        .fetch_one(pool)
        .await?;

    // Check database size
    let db_path = get_database_path();
    let db_size = std::fs::metadata(&db_path)
        .map(|m| m.len())
        .unwrap_or(0);

    // Run integrity check
    let integrity_result: (String,) = sqlx::query_as("PRAGMA integrity_check")
        .fetch_one(pool)
        .await?;

    Ok(DatabaseHealth {
        session_count: session_count as usize,
        db_size_bytes: db_size,
        integrity_ok: integrity_result.0 == "ok",
    })
}

/// Database health report
#[derive(Debug, Clone)]
pub struct DatabaseHealth {
    pub session_count: usize,
    pub db_size_bytes: u64,
    pub integrity_ok: bool,
}

impl std::fmt::Display for DatabaseHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Database Health ===")?;
        writeln!(f, "Sessions:       {}", self.session_count)?;
        writeln!(f, "Database size:  {} MB", self.db_size_bytes / 1024 / 1024)?;
        writeln!(f, "Integrity:      {}", if self.integrity_ok { "OK" } else { "FAILED" })?;
        Ok(())
    }
}
