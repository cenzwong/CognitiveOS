use cognitive_core::errors::CoreError;
use cognitive_persistence::errors::PersistenceError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KernelError {
    #[error("Core domain error: {0}")]
    Core(#[from] CoreError),

    #[error("Persistence error: {0}")]
    Persistence(#[from] PersistenceError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Process {0} not found")]
    ProcessNotFound(String),

    #[error("Illegal operation: {0}")]
    IllegalOperation(String),
}
