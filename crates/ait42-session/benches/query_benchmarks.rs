/// Performance benchmarks for SQLite operations
///
/// Benchmarks in this file measure:
/// - Query performance with various dataset sizes
/// - Index effectiveness
/// - JOIN performance
/// - Complex query optimization
/// - Concurrent access performance

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Setup test database with schema
async fn setup_test_db(pool: &SqlitePool) {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS workspaces (
            hash TEXT PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            last_accessed TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create workspaces table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            workspace_hash TEXT NOT NULL,
            session_type TEXT NOT NULL CHECK(session_type IN ('competition', 'ensemble', 'debate')),
            task TEXT NOT NULL,
            status TEXT NOT NULL CHECK(status IN ('running', 'completed', 'failed', 'paused')),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            completed_at TEXT,
            model TEXT,
            timeout_seconds INTEGER,
            preserve_worktrees INTEGER,
            winner_id INTEGER,
            runtime_mix TEXT,
            total_duration INTEGER,
            total_files_changed INTEGER,
            total_lines_added INTEGER,
            total_lines_deleted INTEGER,
            integration_phase TEXT CHECK(integration_phase IN ('pending', 'in_progress', 'completed')),
            integration_instance_id INTEGER,
            FOREIGN KEY (workspace_hash) REFERENCES workspaces(hash) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create sessions table");

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_workspace ON sessions(workspace_hash)")
        .execute(pool)
        .await
        .expect("Failed to create workspace index");

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status)")
        .execute(pool)
        .await
        .expect("Failed to create status index");

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_type ON sessions(session_type)")
        .execute(pool)
        .await
        .expect("Failed to create type index");

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_created ON sessions(created_at DESC)")
        .execute(pool)
        .await
        .expect("Failed to create created_at index");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS instances (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL,
            instance_id INTEGER NOT NULL,
            worktree_path TEXT NOT NULL,
            branch TEXT NOT NULL,
            agent_name TEXT NOT NULL,
            status TEXT NOT NULL CHECK(status IN ('idle', 'running', 'completed', 'failed', 'paused', 'archived')),
            tmux_session_id TEXT NOT NULL,
            output TEXT,
            start_time TEXT,
            end_time TEXT,
            files_changed INTEGER,
            lines_added INTEGER,
            lines_deleted INTEGER,
            runtime TEXT,
            model TEXT,
            runtime_label TEXT,
            FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
            UNIQUE(session_id, instance_id)
        )
        "#,
    )
    .execute(pool)
    .await
    .expect("Failed to create instances table");

    // Insert workspace
    sqlx::query(
        "INSERT OR IGNORE INTO workspaces (hash, path, last_accessed) VALUES (?, ?, ?)"
    )
    .bind("benchmark_workspace")
    .bind("/benchmark/path")
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await
    .expect("Failed to insert workspace");
}

/// Insert N test sessions
async fn insert_test_sessions(pool: &SqlitePool, count: usize) {
    for i in 0..count {
        let session_type = match i % 3 {
            0 => "competition",
            1 => "ensemble",
            _ => "debate",
        };
        let status = match i % 4 {
            0 => "running",
            1 => "completed",
            2 => "failed",
            _ => "paused",
        };

        sqlx::query(
            r#"
            INSERT INTO sessions (
                id, workspace_hash, session_type, task, status,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(format!("benchmark_session_{:06}", i))
        .bind("benchmark_workspace")
        .bind(session_type)
        .bind(format!("Benchmark task {}", i))
        .bind(status)
        .bind(Utc::now().to_rfc3339())
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await
        .expect("Failed to insert benchmark session");
    }
}

/// Insert instances for sessions
async fn insert_test_instances(pool: &SqlitePool, session_count: usize, instances_per_session: usize) {
    for i in 0..session_count {
        let session_id = format!("benchmark_session_{:06}", i);
        for j in 0..instances_per_session {
            sqlx::query(
                r#"
                INSERT INTO instances (
                    session_id, instance_id, worktree_path, branch,
                    agent_name, status, tmux_session_id
                ) VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&session_id)
            .bind(j as i64)
            .bind(format!("/tmp/worktree_{}_{}", i, j))
            .bind(format!("branch_{}_{}", i, j))
            .bind(format!("agent_{}", j))
            .bind("running")
            .bind(format!("tmux_{}_{}", i, j))
            .execute(pool)
            .await
            .expect("Failed to insert benchmark instance");
        }
    }
}

fn bench_get_all_sessions(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(async {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .expect("Failed to create pool")
    });

    rt.block_on(setup_test_db(&pool));

    let mut group = c.benchmark_group("get_all_sessions");

    for size in [10, 100, 500, 1000].iter() {
        rt.block_on(insert_test_sessions(&pool, *size));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.to_async(&rt).iter(|| async {
                let _sessions: Vec<(String, String, String)> = sqlx::query_as(
                    "SELECT id, session_type, status FROM sessions WHERE workspace_hash = ?"
                )
                .bind("benchmark_workspace")
                .fetch_all(black_box(&pool))
                .await
                .expect("Query failed");
            });
        });

        // Clean up for next size
        rt.block_on(async {
            sqlx::query("DELETE FROM sessions")
                .execute(&pool)
                .await
                .expect("Failed to clean up");
        });
    }

    group.finish();
}

fn bench_get_session_by_id(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(async {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .expect("Failed to create pool")
    });

    rt.block_on(setup_test_db(&pool));
    rt.block_on(insert_test_sessions(&pool, 1000));

    c.bench_function("get_session_by_id", |b| {
        b.to_async(&rt).iter(|| async {
            let _session: Option<(String, String)> = sqlx::query_as(
                "SELECT id, task FROM sessions WHERE id = ?"
            )
            .bind("benchmark_session_000500")
            .fetch_optional(black_box(&pool))
            .await
            .expect("Query failed");
        });
    });
}

fn bench_filter_by_type(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(async {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .expect("Failed to create pool")
    });

    rt.block_on(setup_test_db(&pool));
    rt.block_on(insert_test_sessions(&pool, 1000));

    c.bench_function("filter_by_type_competition", |b| {
        b.to_async(&rt).iter(|| async {
            let _sessions: Vec<(String,)> = sqlx::query_as(
                "SELECT id FROM sessions WHERE session_type = ?"
            )
            .bind("competition")
            .fetch_all(black_box(&pool))
            .await
            .expect("Query failed");
        });
    });
}

fn bench_filter_by_status(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(async {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .expect("Failed to create pool")
    });

    rt.block_on(setup_test_db(&pool));
    rt.block_on(insert_test_sessions(&pool, 1000));

    c.bench_function("filter_by_status_running", |b| {
        b.to_async(&rt).iter(|| async {
            let _sessions: Vec<(String,)> = sqlx::query_as(
                "SELECT id FROM sessions WHERE status = ?"
            )
            .bind("running")
            .fetch_all(black_box(&pool))
            .await
            .expect("Query failed");
        });
    });
}

fn bench_complex_filter(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(async {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .expect("Failed to create pool")
    });

    rt.block_on(setup_test_db(&pool));
    rt.block_on(insert_test_sessions(&pool, 1000));

    c.bench_function("complex_filter_type_and_status", |b| {
        b.to_async(&rt).iter(|| async {
            let _sessions: Vec<(String,)> = sqlx::query_as(
                r#"
                SELECT id FROM sessions
                WHERE session_type = ? AND status IN (?, ?)
                ORDER BY created_at DESC
                LIMIT 50
                "#
            )
            .bind("competition")
            .bind("running")
            .bind("completed")
            .fetch_all(black_box(&pool))
            .await
            .expect("Query failed");
        });
    });
}

fn bench_insert_session(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(async {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .expect("Failed to create pool")
    });

    rt.block_on(setup_test_db(&pool));

    let mut counter = 0;
    c.bench_function("insert_session", |b| {
        b.to_async(&rt).iter(|| async {
            counter += 1;
            sqlx::query(
                r#"
                INSERT INTO sessions (
                    id, workspace_hash, session_type, task, status,
                    created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(format!("insert_bench_{}", counter))
            .bind("benchmark_workspace")
            .bind("competition")
            .bind("Benchmark insert")
            .bind("running")
            .bind(Utc::now().to_rfc3339())
            .bind(Utc::now().to_rfc3339())
            .execute(black_box(&pool))
            .await
            .expect("Insert failed");
        });
    });
}

fn bench_update_session(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(async {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .expect("Failed to create pool")
    });

    rt.block_on(setup_test_db(&pool));
    rt.block_on(insert_test_sessions(&pool, 100));

    c.bench_function("update_session_status", |b| {
        let mut counter = 0;
        b.to_async(&rt).iter(|| async {
            let session_id = format!("benchmark_session_{:06}", counter % 100);
            counter += 1;
            sqlx::query(
                "UPDATE sessions SET status = ?, updated_at = ? WHERE id = ?"
            )
            .bind("completed")
            .bind(Utc::now().to_rfc3339())
            .bind(&session_id)
            .execute(black_box(&pool))
            .await
            .expect("Update failed");
        });
    });
}

fn bench_delete_session(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(async {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .expect("Failed to create pool")
    });

    rt.block_on(setup_test_db(&pool));

    let mut counter = 0;
    c.bench_function("delete_session", |b| {
        b.iter_batched(
            || {
                // Setup: Insert a session to delete
                let session_id = format!("delete_bench_{}", counter);
                counter += 1;
                rt.block_on(async {
                    sqlx::query(
                        r#"
                        INSERT INTO sessions (
                            id, workspace_hash, session_type, task, status,
                            created_at, updated_at
                        ) VALUES (?, ?, ?, ?, ?, ?, ?)
                        "#,
                    )
                    .bind(&session_id)
                    .bind("benchmark_workspace")
                    .bind("competition")
                    .bind("Delete benchmark")
                    .bind("running")
                    .bind(Utc::now().to_rfc3339())
                    .bind(Utc::now().to_rfc3339())
                    .execute(&pool)
                    .await
                    .expect("Insert failed");
                });
                session_id
            },
            |session_id| {
                // Measured: Delete the session
                rt.block_on(async {
                    sqlx::query("DELETE FROM sessions WHERE id = ?")
                        .bind(&session_id)
                        .execute(black_box(&pool))
                        .await
                        .expect("Delete failed");
                });
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_join_with_instances(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(async {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .expect("Failed to create pool")
    });

    rt.block_on(setup_test_db(&pool));
    rt.block_on(insert_test_sessions(&pool, 100));
    rt.block_on(insert_test_instances(&pool, 100, 5));

    c.bench_function("join_sessions_with_instances", |b| {
        b.to_async(&rt).iter(|| async {
            let _results: Vec<(String, String, i64)> = sqlx::query_as(
                r#"
                SELECT s.id, s.task, COUNT(i.id) as instance_count
                FROM sessions s
                LEFT JOIN instances i ON s.id = i.session_id
                WHERE s.workspace_hash = ?
                GROUP BY s.id
                "#
            )
            .bind("benchmark_workspace")
            .fetch_all(black_box(&pool))
            .await
            .expect("Query failed");
        });
    });
}

fn bench_search_like_query(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(async {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .expect("Failed to create pool")
    });

    rt.block_on(setup_test_db(&pool));
    rt.block_on(insert_test_sessions(&pool, 1000));

    c.bench_function("search_task_like", |b| {
        b.to_async(&rt).iter(|| async {
            let _sessions: Vec<(String, String)> = sqlx::query_as(
                "SELECT id, task FROM sessions WHERE task LIKE ?"
            )
            .bind("%task 5%")
            .fetch_all(black_box(&pool))
            .await
            .expect("Query failed");
        });
    });
}

criterion_group!(
    benches,
    bench_get_all_sessions,
    bench_get_session_by_id,
    bench_filter_by_type,
    bench_filter_by_status,
    bench_complex_filter,
    bench_insert_session,
    bench_update_session,
    bench_delete_session,
    bench_join_with_instances,
    bench_search_like_query
);
criterion_main!(benches);
