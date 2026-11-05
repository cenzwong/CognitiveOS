use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessState {
    New,
    Ready,
    Running,
    WaitingForIo,
    WaitingForInput,
    WaitingForApproval,
    WaitingForChildren,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl ProcessState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ProcessState::Completed | ProcessState::Failed | ProcessState::Cancelled
        )
    }

    pub fn is_waiting(&self) -> bool {
        matches!(
            self,
            ProcessState::WaitingForIo
                | ProcessState::WaitingForInput
                | ProcessState::WaitingForApproval
                | ProcessState::WaitingForChildren
        )
    }

    pub fn can_transition_to(&self, target: ProcessState) -> bool {
        if self == &target {
            return false;
        }

        if self.is_terminal() {
            return false;
        }

        if target == ProcessState::Cancelled {
            return true;
        }

        matches!(
            (self, target),
            (ProcessState::New, ProcessState::Ready)
                | (ProcessState::Ready, ProcessState::Running)
                | (ProcessState::Ready, ProcessState::Paused)
                | (ProcessState::Running, ProcessState::Ready)
                | (ProcessState::Running, ProcessState::WaitingForIo)
                | (ProcessState::Running, ProcessState::WaitingForInput)
                | (ProcessState::Running, ProcessState::WaitingForApproval)
                | (ProcessState::Running, ProcessState::WaitingForChildren)
                | (ProcessState::Running, ProcessState::Completed)
                | (ProcessState::Running, ProcessState::Failed)
                | (ProcessState::WaitingForIo, ProcessState::Ready)
                | (ProcessState::WaitingForInput, ProcessState::Ready)
                | (ProcessState::WaitingForApproval, ProcessState::Ready)
                | (ProcessState::WaitingForChildren, ProcessState::Ready)
                | (ProcessState::WaitingForIo, ProcessState::Paused)
                | (ProcessState::WaitingForInput, ProcessState::Paused)
                | (ProcessState::WaitingForApproval, ProcessState::Paused)
                | (ProcessState::WaitingForChildren, ProcessState::Paused)
                | (ProcessState::Paused, ProcessState::Ready)
        )
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessState::New => "new",
            ProcessState::Ready => "ready",
            ProcessState::Running => "running",
            ProcessState::WaitingForIo => "waiting_for_io",
            ProcessState::WaitingForInput => "waiting_for_input",
            ProcessState::WaitingForApproval => "waiting_for_approval",
            ProcessState::WaitingForChildren => "waiting_for_children",
            ProcessState::Paused => "paused",
            ProcessState::Completed => "completed",
            ProcessState::Failed => "failed",
            ProcessState::Cancelled => "cancelled",
        }
    }
}

impl fmt::Display for ProcessState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ProcessState {
    type Err = crate::errors::CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "new" => Ok(ProcessState::New),
            "ready" => Ok(ProcessState::Ready),
            "running" => Ok(ProcessState::Running),
            "waiting_for_io" => Ok(ProcessState::WaitingForIo),
            "waiting_for_input" => Ok(ProcessState::WaitingForInput),
            "waiting_for_approval" => Ok(ProcessState::WaitingForApproval),
            "waiting_for_children" => Ok(ProcessState::WaitingForChildren),
            "paused" => Ok(ProcessState::Paused),
            "completed" => Ok(ProcessState::Completed),
            "failed" => Ok(ProcessState::Failed),
            "cancelled" => Ok(ProcessState::Cancelled),
            _ => Err(crate::errors::CoreError::InvalidProcessState(s.to_string())),
        }
    }
}
