use anyhow::{Context, Result};
use libsql::Builder;

pub fn resolve_db_path() -> String {
    if let Ok(p) = std::env::var("APP_DB_PATH") {
        return p;
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    format!("{home}/.local/share/{{project-name}}/data.db")
}

pub async fn open_database(path: &str) -> Result<libsql::Database> {
    if let Some(dir) = std::path::Path::new(path).parent() {
        tokio::fs::create_dir_all(dir).await?;
    }
    Builder::new_local(path)
        .build()
        .await
        .context("failed to open database")
}

pub async fn init_connection(conn: &libsql::Connection) -> Result<()> {
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .await?;
    Ok(())
}

pub async fn run_migrations(conn: &libsql::Connection) -> Result<()> {
    init_connection(conn).await?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name TEXT PRIMARY KEY,
            applied_at INTEGER NOT NULL
        );",
    )
    .await?;

    let migrations: &[(&str, &str)] = &[
        ("001_init", include_str!("../migrations/001_init.sql")),
    ];

    for (name, sql) in migrations {
        let applied: i64 = conn
            .query(
                "SELECT COUNT(*) FROM _migrations WHERE name = ?1",
                [*name],
            )
            .await?
            .next()
            .await?
            .map(|r| r.get::<i64>(0).unwrap_or(0))
            .unwrap_or(0);

        if applied == 0 {
            conn.execute_batch(sql).await?;
            conn.execute(
                "INSERT INTO _migrations (name, applied_at) VALUES (?1, unixepoch())",
                [*name],
            )
            .await?;
            tracing::info!("applied migration: {name}");
        }
    }
    Ok(())
}
