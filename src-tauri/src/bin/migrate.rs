/**
 * CLI Migration Tool - JSON to SQLite migration utility
 *
 * Usage:
 *   cargo run --bin migrate              # Run migration
 *   cargo run --bin migrate --dry-run    # Dry run (no changes)
 *   cargo run --bin migrate --validate   # Validate existing database
 *   cargo run --bin migrate --backup     # Backup database
 */
use ait42_editor::database::{create_connection_pool, migration, get_database_path};
use ait42_editor::tools::backup;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let dry_run = args.contains(&"--dry-run".to_string());
    let validate_only = args.contains(&"--validate".to_string());
    let backup_only = args.contains(&"--backup".to_string());

    println!("=== AIT42 Session Migration Tool ===\n");

    // Backup database if requested
    if backup_only {
        let db_path = get_database_path();
        if db_path.exists() {
            let backup_path = backup::backup_database(&db_path)?;
            println!("\nBackup created: {:?}", backup_path);
        } else {
            println!("No database found to backup: {:?}", db_path);
        }
        return Ok(());
    }

    // Create database connection
    println!("Connecting to database...");
    let pool = create_connection_pool().await?;

    // Validate database if requested
    if validate_only {
        println!("\nValidating database...");
        let report = migration::validate_migration(&pool).await?;
        println!("\n{}", report);

        if report.is_valid {
            println!("Database validation passed!");
            return Ok(());
        } else {
            println!("Database validation failed!");
            return Err("Validation failed".into());
        }
    }

    // Run migration
    println!("\nMigrating JSON files to SQLite...");
    if dry_run {
        println!("DRY RUN MODE - No changes will be made\n");
    }

    let stats = migration::migrate_json_to_sqlite(&pool, dry_run).await?;

    // Print results
    println!("\n{}", stats);

    if !stats.errors.is_empty() {
        println!("\n=== Errors ===");
        for (idx, error) in stats.errors.iter().enumerate() {
            println!("{}. {}", idx + 1, error);
        }
    }

    // Validate migration if not dry run
    if !dry_run {
        println!("\nValidating migration...");
        let report = migration::validate_migration(&pool).await?;
        println!("\n{}", report);

        if report.is_valid {
            println!("Migration successful!");

            // Ask user if they want to create backup
            println!("\nCreate backup of database? (y/n): ");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            if input.trim().eq_ignore_ascii_case("y") {
                let db_path = get_database_path();
                let backup_path = backup::backup_database(&db_path)?;
                println!("Backup created: {:?}", backup_path);
            }

            Ok(())
        } else {
            println!("Migration validation failed!");
            Err("Migration validation failed".into())
        }
    } else {
        println!("\nDry run complete - no changes were made");
        Ok(())
    }
}
