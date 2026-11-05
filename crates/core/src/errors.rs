use crate::state::ProcessState;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("Invalid process state string: {0}")]
    InvalidProcessState(String),

    #[error("Invalid state transition from {from} to {to}: {reason}")]
    InvalidStateTransition {
        from: ProcessState,
        to: ProcessState,
        reason: String,
    },

    #[error("Process is in terminal state {0} and cannot transition")]
    TerminalStateImmutable(ProcessState),

    #[error("Optimistic locking conflict on process {process_id}: expected version {expected_version}, found {actual_version}")]
    VersionConflict {
        process_id: String,
        expected_version: u64,
        actual_version: u64,
    },

    #[error("Process not found: {0}")]
    ProcessNotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
