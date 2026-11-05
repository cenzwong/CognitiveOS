use crate::errors::KernelError;
use cognitive_core::events::{EventType, ProcessEvent};
use cognitive_core::ids::{PrincipalId, ProcessId, WorkspaceId};
use cognitive_core::process::{Process, ProcessSpecification};
use cognitive_core::state::ProcessState;
use cognitive_core::validate_transition;
use cognitive_persistence::repository::ProcessRepository;
use std::sync::Arc;
use tracing::info;

pub struct KernelService<R: ProcessRepository> {
    repository: Arc<R>,
}

impl<R: ProcessRepository> KernelService<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub fn repository(&self) -> &Arc<R> {
        &self.repository
    }

    /// Creates and admits a new process to `Ready`.
    ///
    /// 1. Validates objective and workspace
    /// 2. Creates Process in `New`
    /// 3. Commits creation and `PROCESS_CREATED` event
    /// 4. Validates transition `New` -> `Ready`
    /// 5. Commits transition and `STATE_TRANSITION` event
    ///
    /// Returns the admitted Process in `Ready` state.
    pub async fn run_process(
        &self,
        objective: String,
        workspace: String,
    ) -> Result<Process, KernelError> {
        let objective = objective.trim().to_string();
        if objective.is_empty() {
            return Err(KernelError::Validation(
                "Objective cannot be empty".to_string(),
            ));
        }

        let workspace = workspace.trim().to_string();
        if workspace.is_empty() {
            return Err(KernelError::Validation(
                "Workspace cannot be empty".to_string(),
            ));
        }

        let process_id = ProcessId::new_v7();
        let spec = ProcessSpecification {
            objective: objective.clone(),
            workspace_id: WorkspaceId::from(workspace),
            owner_principal_id: PrincipalId::from("system_user"),
            acceptance_criteria: Vec::new(),
            constraints: Vec::new(),
            program_id: cognitive_core::ids::ProgramId::from("default_agent"),
            model_profile_id: cognitive_core::ids::ModelProfileId::from("default_model"),
            parent_process_id: None,
            metadata: serde_json::json!({}),
        };

        let process = Process::new(process_id.clone(), spec);

        let initial_event = ProcessEvent::new(
            process_id.clone(),
            1,
            EventType::ProcessCreated,
            serde_json::json!({
                "objective": &objective,
                "workspace": process.workspace_id.as_str(),
            }),
        );

        self.repository
            .create_process(&process, &initial_event)
            .await?;
        info!(process_id = %process.process_id, "Process created in NEW state");

        validate_transition(ProcessState::New, ProcessState::Ready)?;

        let transition_event = ProcessEvent::state_transition(
            process_id.clone(),
            2,
            ProcessState::New,
            ProcessState::Ready,
            Some("Admitted to ready pool for future scheduling".to_string()),
        );

        let admitted_process = self
            .repository
            .update_process_state_transactional(
                &process_id,
                0,
                ProcessState::Ready,
                &transition_event,
            )
            .await?;

        info!(process_id = %admitted_process.process_id, "Process admitted to READY state");
        Ok(admitted_process)
    }

    pub async fn get_process(&self, process_id: &ProcessId) -> Result<Process, KernelError> {
        self.repository
            .get_process(process_id)
            .await?
            .ok_or_else(|| KernelError::ProcessNotFound(process_id.to_string()))
    }

    pub async fn list_processes(&self) -> Result<Vec<Process>, KernelError> {
        let processes = self.repository.list_processes().await?;
        Ok(processes)
    }

    pub async fn get_process_events(
        &self,
        process_id: &ProcessId,
        from_sequence: Option<u64>,
    ) -> Result<Vec<ProcessEvent>, KernelError> {
        self.get_process(process_id).await?;
        let events = self
            .repository
            .get_events(process_id, from_sequence)
            .await?;
        Ok(events)
    }

    pub async fn pause_process(
        &self,
        process_id: &ProcessId,
        reason: Option<String>,
    ) -> Result<Process, KernelError> {
        let current = self.get_process(process_id).await?;
        validate_transition(current.state, ProcessState::Paused)?;

        let next_seq = self.repository.get_latest_sequence(process_id).await? + 1;
        let event = ProcessEvent::state_transition(
            process_id.clone(),
            next_seq,
            current.state,
            ProcessState::Paused,
            reason.or_else(|| Some("Administratively paused".to_string())),
        );

        let updated = self
            .repository
            .update_process_state_transactional(
                process_id,
                current.state_version,
                ProcessState::Paused,
                &event,
            )
            .await?;

        info!(process_id = %process_id, "Process paused");
        Ok(updated)
    }

    pub async fn resume_process(
        &self,
        process_id: &ProcessId,
        reason: Option<String>,
    ) -> Result<Process, KernelError> {
        let current = self.get_process(process_id).await?;
        validate_transition(current.state, ProcessState::Ready)?;

        let next_seq = self.repository.get_latest_sequence(process_id).await? + 1;
        let event = ProcessEvent::state_transition(
            process_id.clone(),
            next_seq,
            current.state,
            ProcessState::Ready,
            reason.or_else(|| Some("Resumed from paused state".to_string())),
        );

        let updated = self
            .repository
            .update_process_state_transactional(
                process_id,
                current.state_version,
                ProcessState::Ready,
                &event,
            )
            .await?;

        info!(process_id = %process_id, "Process resumed to READY");
        Ok(updated)
    }

    pub async fn cancel_process(
        &self,
        process_id: &ProcessId,
        reason: Option<String>,
    ) -> Result<Process, KernelError> {
        let current = self.get_process(process_id).await?;
        validate_transition(current.state, ProcessState::Cancelled)?;

        let next_seq = self.repository.get_latest_sequence(process_id).await? + 1;
        let event = ProcessEvent::state_transition(
            process_id.clone(),
            next_seq,
            current.state,
            ProcessState::Cancelled,
            reason.or_else(|| Some("Administratively cancelled".to_string())),
        );

        let updated = self
            .repository
            .update_process_state_transactional(
                process_id,
                current.state_version,
                ProcessState::Cancelled,
                &event,
            )
            .await?;

        info!(process_id = %process_id, "Process cancelled");
        Ok(updated)
    }
}
