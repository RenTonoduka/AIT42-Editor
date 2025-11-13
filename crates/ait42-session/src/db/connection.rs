use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::error::{Result, SessionError};

/// SQLite database connection pool
pub struct DbPool {
    pool: SqlitePool,
    db_path: PathBuf,
}

impl DbPool {
    /// Create new database pool with given database path
    pub async fn new(database_path: impl AsRef<Path>) -> Result<Self> {
        let db_path = database_path.as_ref().to_path_buf();

        info!("Initializing database at {:?}", db_path);

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
                debug!("Created database directory: {:?}", parent);
            }
        }

        // Build database URL
        let database_url = format!("sqlite:{}", db_path.display());

        // Configure SQLite connection options
        let options = SqliteConnectOptions::from_str(&database_url)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .foreign_keys(true)
            .busy_timeout(Duration::from_secs(30))
            .disable_statement_logging();

        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(10))
            .connect_with(options)
            .await?;

        debug!("Database connection pool created");

        // Run migrations
        info!("Running database migrations");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| SessionError::Migration(e.to_string()))?;

        info!("Database migrations completed");

        // Optimize database pragmas
        Self::optimize_pragmas(&pool).await?;

        info!("Database initialized successfully");

        Ok(Self { pool, db_path })
    }

    /// Create database pool with default path (~/.ait42/sessions.db)
    pub async fn new_default() -> Result<Self> {
        let db_path = Self::default_db_path()?;
        Self::new(db_path).await
    }

    /// Get default database path
    pub fn default_db_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| SessionError::Internal("Home directory not found".to_string()))?;

        Ok(home_dir.join(".ait42").join("sessions.db"))
    }

    /// Get reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Get database file path
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Close the connection pool
    pub async fn close(self) {
        info!("Closing database connection pool");
        self.pool.close().await;
    }

    /// Optimize database (run PRAGMA optimize and incremental vacuum)
    pub async fn optimize(&self) -> Result<()> {
        info!("Optimizing database");

        sqlx::query("PRAGMA optimize")
            .execute(&self.pool)
            .await?;

        sqlx::query("PRAGMA incremental_vacuum(100)")
            .execute(&self.pool)
            .await?;

        debug!("Database optimization completed");
        Ok(())
    }

    /// Verify database integrity
    pub async fn verify_integrity(&self) -> Result<bool> {
        debug!("Checking database integrity");

        let result: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&self.pool)
            .await?;

        let is_ok = result == "ok";

        if is_ok {
            debug!("Database integrity check passed");
        } else {
            warn!("Database integrity check failed: {}", result);
        }

        Ok(is_ok)
    }

    /// Set optimized pragmas for performance
    async fn optimize_pragmas(pool: &SqlitePool) -> Result<()> {
        // Cache size: 8MB
        sqlx::query("PRAGMA cache_size = -8000")
            .execute(pool)
            .await?;

        // Temp store in memory
        sqlx::query("PRAGMA temp_store = MEMORY")
            .execute(pool)
            .await?;

        // Analysis limit for query optimizer
        sqlx::query("PRAGMA analysis_limit = 1000")
            .execute(pool)
            .await?;

        debug!("Database pragmas optimized");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_database_initialization() {
        let db_file = NamedTempFile::new().unwrap();
        let db_path = db_file.path();

        let db_pool = DbPool::new(db_path).await;
        assert!(db_pool.is_ok());

        let db_pool = db_pool.unwrap();
        assert_eq!(db_pool.db_path(), db_path);
    }

    #[tokio::test]
    async fn test_database_integrity_check() {
        let db_file = NamedTempFile::new().unwrap();
        let db_pool = DbPool::new(db_file.path()).await.unwrap();

        let is_ok = db_pool.verify_integrity().await.unwrap();
        assert!(is_ok);
    }

    #[tokio::test]
    async fn test_database_optimization() {
        let db_file = NamedTempFile::new().unwrap();
        let db_pool = DbPool::new(db_file.path()).await.unwrap();

        let result = db_pool.optimize().await;
        assert!(result.is_ok());
    }
}
