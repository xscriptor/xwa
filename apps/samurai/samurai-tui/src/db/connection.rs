use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{PgPool, SqlitePool};
use std::env;

#[derive(Clone)]
pub enum DbPool {
    Postgres(PgPool),
    Sqlite(SqlitePool),
}

pub async fn create_pool() -> Result<DbPool, sqlx::Error> {
    let force_pg = env::var("SAMURAI_DB_BACKEND")
        .map(|v| v.to_lowercase() == "postgres")
        .unwrap_or(false);

    if force_pg {
        return create_pg_pool().await.map(DbPool::Postgres);
    }

    match create_pg_pool().await {
        Ok(pool) => {
            log::info!("PostgreSQL connected");
            Ok(DbPool::Postgres(pool))
        }
        Err(e) => {
            log::warn!("PostgreSQL unavailable ({}), using SQLite fallback", e);
            create_sqlite_pool().await.map(DbPool::Sqlite)
        }
    }
}

async fn create_pg_pool() -> Result<PgPool, sqlx::Error> {
    let user = env::var("DB_USER").unwrap_or_else(|_| "postgres".into());
    let pass = env::var("DB_PASS").unwrap_or_else(|_| "postgres".into());
    let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".into());
    let name = env::var("DB_NAME").unwrap_or_else(|_| "samurai".into());

    let url = format!("postgresql://{}:{}@{}/{}", user, pass, host, name);

    PgPoolOptions::new()
        .max_connections(3)
        .min_connections(1)
        .connect(&url)
        .await
}

async fn create_sqlite_pool() -> Result<SqlitePool, sqlx::Error> {
    let db_path = env::var("SAMURAI_SQLITE_PATH").unwrap_or_else(|_| "samurai.db".into());
    log::info!("SQLite database at {}", db_path);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite:{}?mode=rwc", db_path))
        .await?;

    sqlx::query("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .execute(&pool)
        .await?;

    Ok(pool)
}

pub async fn init_db(pool: &DbPool) -> Result<(), sqlx::Error> {
    match pool {
        DbPool::Postgres(p) => init_pg(p).await,
        DbPool::Sqlite(p) => init_sqlite(p).await,
    }
}

async fn init_pg(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS scans (
            id SERIAL PRIMARY KEY,
            domain_target VARCHAR NOT NULL,
            status VARCHAR DEFAULT 'RUNNING',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            scan_type VARCHAR DEFAULT 'port_scan'
        );
        CREATE INDEX IF NOT EXISTS idx_scans_target ON scans(domain_target);
        CREATE INDEX IF NOT EXISTS idx_scans_status ON scans(status);

        CREATE TABLE IF NOT EXISTS discovered_links (
            id SERIAL PRIMARY KEY,
            scan_id INTEGER REFERENCES scans(id) ON DELETE CASCADE,
            url VARCHAR NOT NULL,
            status_code INTEGER,
            content_type VARCHAR
        );
        CREATE INDEX IF NOT EXISTS idx_links_scan ON discovered_links(scan_id);

        CREATE TABLE IF NOT EXISTS findings (
            id SERIAL PRIMARY KEY,
            scan_id INTEGER REFERENCES scans(id) ON DELETE CASCADE,
            link_id INTEGER REFERENCES discovered_links(id) ON DELETE CASCADE,
            severity VARCHAR NOT NULL,
            finding_type VARCHAR NOT NULL,
            description VARCHAR NOT NULL,
            poc_payload TEXT,
            cvss_score VARCHAR
        );
        CREATE INDEX IF NOT EXISTS idx_findings_scan ON findings(scan_id);
        CREATE INDEX IF NOT EXISTS idx_findings_link ON findings(link_id);
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn init_sqlite(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS scans (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            domain_target TEXT NOT NULL,
            status TEXT DEFAULT 'RUNNING',
            created_at TEXT DEFAULT (datetime('now')),
            scan_type TEXT DEFAULT 'port_scan'
        );
        CREATE INDEX IF NOT EXISTS idx_scans_target ON scans(domain_target);
        CREATE INDEX IF NOT EXISTS idx_scans_status ON scans(status);

        CREATE TABLE IF NOT EXISTS discovered_links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_id INTEGER REFERENCES scans(id) ON DELETE CASCADE,
            url TEXT NOT NULL,
            status_code INTEGER,
            content_type TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_links_scan ON discovered_links(scan_id);

        CREATE TABLE IF NOT EXISTS findings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            scan_id INTEGER REFERENCES scans(id) ON DELETE CASCADE,
            link_id INTEGER REFERENCES discovered_links(id) ON DELETE CASCADE,
            severity TEXT NOT NULL,
            finding_type TEXT NOT NULL,
            description TEXT NOT NULL,
            poc_payload TEXT,
            cvss_score TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_findings_scan ON findings(scan_id);
        CREATE INDEX IF NOT EXISTS idx_findings_link ON findings(link_id);
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}
