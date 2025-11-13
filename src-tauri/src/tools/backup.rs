/**
 * Backup and Rollback Tools
 *
 * Provides database backup, restoration, and rollback capabilities.
 */
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Utc;

/// Backup database to timestamped file
pub fn backup_database(db_path: &Path) -> Result<PathBuf, std::io::Error> {
    if !db_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Database not found: {:?}", db_path),
        ));
    }

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let backup_filename = format!(
        "sessions_backup_{}.db",
        timestamp
    );

    let backup_path = db_path
        .parent()
        .unwrap()
        .join(backup_filename);

    println!("Creating backup: {:?}", backup_path);
    fs::copy(db_path, &backup_path)?;

    // Also backup WAL and SHM files if they exist
    let wal_path = db_path.with_extension("db-wal");
    if wal_path.exists() {
        let backup_wal = backup_path.with_extension("db-wal");
        fs::copy(&wal_path, &backup_wal)?;
    }

    let shm_path = db_path.with_extension("db-shm");
    if shm_path.exists() {
        let backup_shm = backup_path.with_extension("db-shm");
        fs::copy(&shm_path, &backup_shm)?;
    }

    println!("Backup created successfully!");
    Ok(backup_path)
}

/// Restore database from backup file
pub fn restore_database(backup_path: &Path, db_path: &Path) -> Result<(), std::io::Error> {
    if !backup_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Backup not found: {:?}", backup_path),
        ));
    }

    println!("Restoring database from: {:?}", backup_path);

    // Remove existing database files
    if db_path.exists() {
        fs::remove_file(db_path)?;
    }

    let wal_path = db_path.with_extension("db-wal");
    if wal_path.exists() {
        let _ = fs::remove_file(wal_path);
    }

    let shm_path = db_path.with_extension("db-shm");
    if shm_path.exists() {
        let _ = fs::remove_file(shm_path);
    }

    // Restore from backup
    fs::copy(backup_path, db_path)?;

    let backup_wal = backup_path.with_extension("db-wal");
    if backup_wal.exists() {
        fs::copy(&backup_wal, &wal_path)?;
    }

    let backup_shm = backup_path.with_extension("db-shm");
    if backup_shm.exists() {
        fs::copy(&backup_shm, &shm_path)?;
    }

    println!("Database restored successfully!");
    Ok(())
}

/// List available backups
pub fn list_backups(db_dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut backups = Vec::new();

    for entry in fs::read_dir(db_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy();
            if filename_str.starts_with("sessions_backup_") && filename_str.ends_with(".db") {
                backups.push(path);
            }
        }
    }

    backups.sort_by(|a, b| b.cmp(a)); // Most recent first
    Ok(backups)
}

/// Archive old JSON files
pub fn archive_json_files(sessions_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if !sessions_dir.exists() {
        return Err(format!("Sessions directory not found: {:?}", sessions_dir).into());
    }

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let archive_dir = sessions_dir.parent().unwrap().join("sessions_archive");

    // Create archive directory
    if !archive_dir.exists() {
        fs::create_dir_all(&archive_dir)?;
    }

    let archive_path = archive_dir.join(format!("sessions_archive_{}.tar.gz", timestamp));

    // Use tar command to create compressed archive
    println!("Archiving JSON files to: {:?}", archive_path);

    let status = std::process::Command::new("tar")
        .args(&[
            "-czf",
            archive_path.to_str().unwrap(),
            "-C",
            sessions_dir.to_str().unwrap(),
            ".",
        ])
        .status()?;

    if !status.success() {
        return Err("Failed to create archive".into());
    }

    println!("Archive created successfully!");
    Ok(archive_path)
}

/// Clean up old backups (keep only N most recent)
pub fn cleanup_old_backups(db_dir: &Path, keep_count: usize) -> Result<usize, std::io::Error> {
    let mut backups = list_backups(db_dir)?;

    if backups.len() <= keep_count {
        return Ok(0);
    }

    let to_remove = backups.split_off(keep_count);
    let removed_count = to_remove.len();

    println!("Removing {} old backups...", removed_count);
    for backup in to_remove {
        println!("  Removing: {:?}", backup.file_name().unwrap());
        fs::remove_file(&backup)?;

        // Also remove WAL and SHM if they exist
        let wal = backup.with_extension("db-wal");
        if wal.exists() {
            fs::remove_file(wal)?;
        }

        let shm = backup.with_extension("db-shm");
        if shm.exists() {
            fs::remove_file(shm)?;
        }
    }

    println!("Cleanup complete!");
    Ok(removed_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_backup_database() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create dummy database file
        fs::write(&db_path, b"test data").unwrap();

        let backup_path = backup_database(&db_path).unwrap();

        assert!(backup_path.exists());
        assert_eq!(fs::read(&backup_path).unwrap(), b"test data");
    }

    #[test]
    fn test_list_backups() {
        let temp_dir = TempDir::new().unwrap();

        // Create dummy backup files
        fs::write(temp_dir.path().join("sessions_backup_20250113_100000.db"), b"backup1").unwrap();
        fs::write(temp_dir.path().join("sessions_backup_20250113_110000.db"), b"backup2").unwrap();
        fs::write(temp_dir.path().join("other_file.txt"), b"other").unwrap();

        let backups = list_backups(temp_dir.path()).unwrap();

        assert_eq!(backups.len(), 2);
        // Should be sorted most recent first
        assert!(backups[0].to_string_lossy().contains("110000"));
    }

    #[test]
    fn test_cleanup_old_backups() {
        let temp_dir = TempDir::new().unwrap();

        // Create 5 backup files
        for i in 0..5 {
            let filename = format!("sessions_backup_2025011310{:02}00.db", i);
            fs::write(temp_dir.path().join(&filename), format!("backup{}", i)).unwrap();
        }

        let removed = cleanup_old_backups(temp_dir.path(), 2).unwrap();

        assert_eq!(removed, 3);
        assert_eq!(list_backups(temp_dir.path()).unwrap().len(), 2);
    }
}
