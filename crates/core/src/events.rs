use crate::ids::*;
use crate::state::ProcessState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    ProcessCreated,
    StateTransition,
    ProcessPaused,
    ProcessResumed,
    ProcessCancelled,
    ProcessCompleted,
    ProcessFailed,
    SliceStarted,
    SliceCompleted,
    ToolIntentRecorded,
    ToolDispatched,
    ToolCompleted,
    ToolFailed,
    MemoryWritten,
    UserInstructionReceived,
    ApprovalRequested,
    ApprovalDecided,
    RecoveryTriggered,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::ProcessCreated => "PROCESS_CREATED",
            EventType::StateTransition => "STATE_TRANSITION",
            EventType::ProcessPaused => "PROCESS_PAUSED",
            EventType::ProcessResumed => "PROCESS_RESUMED",
            EventType::ProcessCancelled => "PROCESS_CANCELLED",
            EventType::ProcessCompleted => "PROCESS_COMPLETED",
            EventType::ProcessFailed => "PROCESS_FAILED",
            EventType::SliceStarted => "SLICE_STARTED",
            EventType::SliceCompleted => "SLICE_COMPLETED",
            EventType::ToolIntentRecorded => "TOOL_INTENT_RECORDED",
            EventType::ToolDispatched => "TOOL_DISPATCHED",
            EventType::ToolCompleted => "TOOL_COMPLETED",
            EventType::ToolFailed => "TOOL_FAILED",
            EventType::MemoryWritten => "MEMORY_WRITTEN",
            EventType::UserInstructionReceived => "USER_INSTRUCTION_RECEIVED",
            EventType::ApprovalRequested => "APPROVAL_REQUESTED",
            EventType::ApprovalDecided => "APPROVAL_DECIDED",
            EventType::RecoveryTriggered => "RECOVERY_TRIGGERED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessEvent {
    pub event_id: EventId,
    pub process_id: ProcessId,
    pub sequence: u64,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub payload: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slice_id: Option<SliceId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<OperationId>,
}

impl ProcessEvent {
    pub fn new(
        process_id: ProcessId,
        sequence: u64,
        event_type: EventType,
        payload: Value,
    ) -> Self {
        Self {
            event_id: EventId::new_v7(),
            process_id,
            sequence,
            event_type,
            timestamp: Utc::now(),
            payload,
            slice_id: None,
            operation_id: None,
        }
    }

    pub fn state_transition(
        process_id: ProcessId,
        sequence: u64,
        from: ProcessState,
        to: ProcessState,
        reason: Option<String>,
    ) -> Self {
        let payload = serde_json::json!({
            "from": from.as_str(),
            "to": to.as_str(),
            "reason": reason,
        });

        let event_type = match to {
            ProcessState::Paused => EventType::ProcessPaused,
            ProcessState::Ready if from == ProcessState::Paused => EventType::ProcessResumed,
            ProcessState::Cancelled => EventType::ProcessCancelled,
            ProcessState::Completed => EventType::ProcessCompleted,
            ProcessState::Failed => EventType::ProcessFailed,
            _ => EventType::StateTransition,
        };

        Self::new(process_id, sequence, event_type, payload)
    }
}
