pub mod errors;
pub mod events;
pub mod ids;
pub mod instruction;
pub mod process;
pub mod state;

pub use errors::CoreError;
pub use events::{EventType, ProcessEvent};
pub use ids::*;
pub use instruction::{CognitiveInstruction, ModelDecision, ModelResponseContract};
pub use process::{Process, ProcessSpecification};
pub use state::ProcessState;

pub fn validate_transition(from: ProcessState, to: ProcessState) -> Result<(), CoreError> {
    if from.is_terminal() {
        return Err(CoreError::TerminalStateImmutable(from));
    }

    if !from.can_transition_to(to) {
        return Err(CoreError::InvalidStateTransition {
            from,
            to,
            reason: format!("Transition from {} to {} is not allowed", from, to),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        assert!(validate_transition(ProcessState::New, ProcessState::Ready).is_ok());
        assert!(validate_transition(ProcessState::Ready, ProcessState::Running).is_ok());
        assert!(validate_transition(ProcessState::Ready, ProcessState::Paused).is_ok());
        assert!(validate_transition(ProcessState::Ready, ProcessState::Cancelled).is_ok());
        assert!(validate_transition(ProcessState::Paused, ProcessState::Ready).is_ok());
        assert!(validate_transition(ProcessState::Paused, ProcessState::Cancelled).is_ok());
        assert!(validate_transition(ProcessState::Running, ProcessState::WaitingForIo).is_ok());
        assert!(validate_transition(ProcessState::WaitingForIo, ProcessState::Ready).is_ok());
        assert!(validate_transition(ProcessState::Running, ProcessState::Completed).is_ok());
        assert!(validate_transition(ProcessState::Running, ProcessState::Failed).is_ok());
    }

    #[test]
    fn test_invalid_transitions() {
        assert!(validate_transition(ProcessState::New, ProcessState::Running).is_err());
        assert!(validate_transition(ProcessState::New, ProcessState::Paused).is_err());
        assert!(validate_transition(ProcessState::New, ProcessState::Completed).is_err());

        assert!(validate_transition(ProcessState::Ready, ProcessState::Ready).is_err());

        assert!(validate_transition(ProcessState::Completed, ProcessState::Ready).is_err());
        assert!(validate_transition(ProcessState::Completed, ProcessState::Cancelled).is_err());
        assert!(validate_transition(ProcessState::Failed, ProcessState::Ready).is_err());
        assert!(validate_transition(ProcessState::Cancelled, ProcessState::Ready).is_err());
    }

    #[test]
    fn test_instruction_serialization() {
        let instr = CognitiveInstruction::ToolExecute {
            tool_id: "workspace.read_file".to_string(),
            arguments: serde_json::json!({"path": "README.md"}),
        };

        let serialized = serde_json::to_string(&instr).unwrap();
        assert!(serialized.contains(r#""op":"tool_execute""#));
        assert!(serialized.contains("workspace.read_file"));

        let deserialized: CognitiveInstruction = serde_json::from_str(&serialized).unwrap();
        assert_eq!(instr, deserialized);
    }

    #[test]
    fn test_process_state_string_roundtrip() {
        let states = [
            ProcessState::New,
            ProcessState::Ready,
            ProcessState::Running,
            ProcessState::WaitingForIo,
            ProcessState::WaitingForInput,
            ProcessState::WaitingForApproval,
            ProcessState::WaitingForChildren,
            ProcessState::Paused,
            ProcessState::Completed,
            ProcessState::Failed,
            ProcessState::Cancelled,
        ];

        for state in states {
            let s = state.as_str();
            let parsed: ProcessState = s.parse().unwrap();
            assert_eq!(state, parsed);
        }
    }

    #[test]
    fn test_process_spec_and_creation() {
        let spec = ProcessSpecification {
            objective: "Investigate issue".to_string(),
            workspace_id: WorkspaceId::from("ws-123"),
            owner_principal_id: PrincipalId::from("prn-user"),
            acceptance_criteria: vec!["criterion 1".to_string()],
            constraints: vec!["constraint 1".to_string()],
            program_id: ProgramId::from("default_agent"),
            model_profile_id: ModelProfileId::from("default_model"),
            parent_process_id: None,
            metadata: serde_json::json!({}),
        };

        let proc_id = ProcessId::new_v7();
        let process = Process::new(proc_id.clone(), spec);

        assert_eq!(process.process_id, proc_id);
        assert_eq!(process.state, ProcessState::New);
        assert_eq!(process.state_version, 0);
    }
}
