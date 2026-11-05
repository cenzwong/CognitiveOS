use crate::errors::PersistenceError;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::ConnectOptions;
use std::str::FromStr;
use tracing::log::LevelFilter;

pub async fn create_sqlite_pool(database_url: &str) -> Result<SqlitePool, PersistenceError> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(true)
        .busy_timeout(std::time::Duration::from_secs(5))
        .log_statements(LevelFilter::Debug);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), PersistenceError> {
    sqlx::migrate!("../../migrations").run(pool).await?;
    Ok(())
}
