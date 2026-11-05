use crate::ids::*;
use crate::state::ProcessState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessSpecification {
    pub objective: String,
    pub workspace_id: WorkspaceId,
    pub owner_principal_id: PrincipalId,
    #[serde(default)]
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default = "default_program_id")]
    pub program_id: ProgramId,
    #[serde(default = "default_model_profile_id")]
    pub model_profile_id: ModelProfileId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_process_id: Option<ProcessId>,
    #[serde(default)]
    pub metadata: Value,
}

fn default_program_id() -> ProgramId {
    ProgramId::from("default_agent")
}

fn default_model_profile_id() -> ModelProfileId {
    ModelProfileId::from("default_model")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Process {
    pub process_id: ProcessId,
    pub workspace_id: WorkspaceId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_process_id: Option<ProcessId>,
    pub owner_principal_id: PrincipalId,

    pub objective: String,
    pub specification: ProcessSpecification,

    pub state: ProcessState,
    pub state_version: u64,

    pub program_id: ProgramId,
    pub model_profile_id: ModelProfileId,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_slice_id: Option<SliceId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_operation_id: Option<OperationId>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

impl Process {
    pub fn new(process_id: ProcessId, spec: ProcessSpecification) -> Self {
        let now = Utc::now();
        Self {
            process_id,
            workspace_id: spec.workspace_id.clone(),
            parent_process_id: spec.parent_process_id.clone(),
            owner_principal_id: spec.owner_principal_id.clone(),
            objective: spec.objective.clone(),
            program_id: spec.program_id.clone(),
            model_profile_id: spec.model_profile_id.clone(),
            specification: spec,
            state: ProcessState::New,
            state_version: 0,
            current_slice_id: None,
            pending_operation_id: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }
}
