use cognitive_core::errors::CoreError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Core domain error: {0}")]
    Core(#[from] CoreError),

    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Process {process_id} not found")]
    ProcessNotFound { process_id: String },

    #[error(
        "Optimistic locking conflict on process {process_id}: expected version {expected_version}"
    )]
    VersionConflict {
        process_id: String,
        expected_version: u64,
    },
}
