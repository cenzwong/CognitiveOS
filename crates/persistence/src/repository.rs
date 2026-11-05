use crate::errors::PersistenceError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use cognitive_core::events::ProcessEvent;
use cognitive_core::ids::{EventId, OperationId, ProcessId, SliceId};
use cognitive_core::process::{Process, ProcessSpecification};
use cognitive_core::state::ProcessState;
use sqlx::{Row, SqlitePool};

#[async_trait]
pub trait ProcessRepository: Send + Sync {
    async fn create_process(
        &self,
        process: &Process,
        initial_event: &ProcessEvent,
    ) -> Result<(), PersistenceError>;

    async fn get_process(
        &self,
        process_id: &ProcessId,
    ) -> Result<Option<Process>, PersistenceError>;

    async fn list_processes(&self) -> Result<Vec<Process>, PersistenceError>;

    async fn update_process_state_transactional(
        &self,
        process_id: &ProcessId,
        expected_version: u64,
        target_state: ProcessState,
        event: &ProcessEvent,
    ) -> Result<Process, PersistenceError>;

    async fn get_events(
        &self,
        process_id: &ProcessId,
        from_sequence: Option<u64>,
    ) -> Result<Vec<ProcessEvent>, PersistenceError>;

    async fn get_latest_sequence(&self, process_id: &ProcessId) -> Result<u64, PersistenceError>;
}

#[derive(Clone)]
pub struct SqliteProcessRepository {
    pool: SqlitePool,
}

impl SqliteProcessRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>, PersistenceError> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| {
            PersistenceError::Core(cognitive_core::CoreError::ValidationError(e.to_string()))
        })
}

#[async_trait]
impl ProcessRepository for SqliteProcessRepository {
    async fn create_process(
        &self,
        process: &Process,
        initial_event: &ProcessEvent,
    ) -> Result<(), PersistenceError> {
        let mut tx = self.pool.begin().await?;

        let spec_json = serde_json::to_string(&process.specification)?;
        let created_at_str = process.created_at.to_rfc3339();
        let updated_at_str = process.updated_at.to_rfc3339();
        let parent_id_str = process.parent_process_id.as_ref().map(|id| id.as_str());
        let current_slice_id_str = process.current_slice_id.as_ref().map(|id| id.as_str());
        let pending_op_id_str = process.pending_operation_id.as_ref().map(|id| id.as_str());

        sqlx::query(
            r#"
            INSERT INTO processes (
                process_id,
                workspace_id,
                parent_process_id,
                owner_principal_id,
                objective,
                specification_json,
                state,
                state_version,
                program_id,
                model_profile_id,
                current_slice_id,
                pending_operation_id,
                created_at,
                updated_at,
                completed_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL)
            "#,
        )
        .bind(process.process_id.as_str())
        .bind(process.workspace_id.as_str())
        .bind(parent_id_str)
        .bind(process.owner_principal_id.as_str())
        .bind(&process.objective)
        .bind(spec_json)
        .bind(process.state.as_str())
        .bind(process.state_version as i64)
        .bind(process.program_id.as_str())
        .bind(process.model_profile_id.as_str())
        .bind(current_slice_id_str)
        .bind(pending_op_id_str)
        .bind(created_at_str)
        .bind(updated_at_str)
        .execute(&mut *tx)
        .await?;

        // Insert initial event
        let payload_json = serde_json::to_string(&initial_event.payload)?;
        let event_timestamp = initial_event.timestamp.to_rfc3339();
        let event_type_str = initial_event.event_type.as_str();
        let slice_str = initial_event.slice_id.as_ref().map(|s| s.as_str());
        let op_str = initial_event.operation_id.as_ref().map(|o| o.as_str());

        sqlx::query(
            r#"
            INSERT INTO process_events (
                event_id,
                process_id,
                sequence,
                event_type,
                timestamp,
                payload_json,
                slice_id,
                operation_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(initial_event.event_id.as_str())
        .bind(initial_event.process_id.as_str())
        .bind(initial_event.sequence as i64)
        .bind(event_type_str)
        .bind(event_timestamp)
        .bind(payload_json)
        .bind(slice_str)
        .bind(op_str)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn get_process(
        &self,
        process_id: &ProcessId,
    ) -> Result<Option<Process>, PersistenceError> {
        let row = sqlx::query(
            r#"
            SELECT
                process_id,
                workspace_id,
                parent_process_id,
                owner_principal_id,
                objective,
                specification_json,
                state,
                state_version,
                program_id,
                model_profile_id,
                current_slice_id,
                pending_operation_id,
                created_at,
                updated_at,
                completed_at
            FROM processes
            WHERE process_id = ?
            "#,
        )
        .bind(process_id.as_str())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            let spec_json: String = r.get("specification_json");
            let spec: ProcessSpecification = serde_json::from_str(&spec_json)?;
            let state_str: String = r.get("state");
            let state: ProcessState = state_str.parse()?;

            let created_at_str: String = r.get("created_at");
            let updated_at_str: String = r.get("updated_at");
            let completed_at_str: Option<String> = r.get("completed_at");

            let created_at = parse_datetime(&created_at_str)?;
            let updated_at = parse_datetime(&updated_at_str)?;
            let completed_at = completed_at_str
                .as_deref()
                .map(parse_datetime)
                .transpose()?;

            let parent_id_str: Option<String> = r.get("parent_process_id");
            let slice_id_str: Option<String> = r.get("current_slice_id");
            let op_id_str: Option<String> = r.get("pending_operation_id");
            let state_version_i64: i64 = r.get("state_version");

            Ok(Some(Process {
                process_id: ProcessId::from(r.get::<String, _>("process_id")),
                workspace_id: spec.workspace_id.clone(),
                parent_process_id: parent_id_str.map(ProcessId::from),
                owner_principal_id: spec.owner_principal_id.clone(),
                objective: r.get("objective"),
                specification: spec,
                state,
                state_version: state_version_i64 as u64,
                program_id: cognitive_core::ids::ProgramId::from(r.get::<String, _>("program_id")),
                model_profile_id: cognitive_core::ids::ModelProfileId::from(
                    r.get::<String, _>("model_profile_id"),
                ),
                current_slice_id: slice_id_str.map(SliceId::from),
                pending_operation_id: op_id_str.map(OperationId::from),
                created_at,
                updated_at,
                completed_at,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_processes(&self) -> Result<Vec<Process>, PersistenceError> {
        let rows = sqlx::query(
            r#"
            SELECT
                process_id,
                workspace_id,
                parent_process_id,
                owner_principal_id,
                objective,
                specification_json,
                state,
                state_version,
                program_id,
                model_profile_id,
                current_slice_id,
                pending_operation_id,
                created_at,
                updated_at,
                completed_at
            FROM processes
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut list = Vec::new();
        for r in rows {
            let spec_json: String = r.get("specification_json");
            let spec: ProcessSpecification = serde_json::from_str(&spec_json)?;
            let state_str: String = r.get("state");
            let state: ProcessState = state_str.parse()?;

            let created_at_str: String = r.get("created_at");
            let updated_at_str: String = r.get("updated_at");
            let completed_at_str: Option<String> = r.get("completed_at");

            let created_at = parse_datetime(&created_at_str)?;
            let updated_at = parse_datetime(&updated_at_str)?;
            let completed_at = completed_at_str
                .as_deref()
                .map(parse_datetime)
                .transpose()?;

            let parent_id_str: Option<String> = r.get("parent_process_id");
            let slice_id_str: Option<String> = r.get("current_slice_id");
            let op_id_str: Option<String> = r.get("pending_operation_id");
            let state_version_i64: i64 = r.get("state_version");

            list.push(Process {
                process_id: ProcessId::from(r.get::<String, _>("process_id")),
                workspace_id: spec.workspace_id.clone(),
                parent_process_id: parent_id_str.map(ProcessId::from),
                owner_principal_id: spec.owner_principal_id.clone(),
                objective: r.get("objective"),
                specification: spec,
                state,
                state_version: state_version_i64 as u64,
                program_id: cognitive_core::ids::ProgramId::from(r.get::<String, _>("program_id")),
                model_profile_id: cognitive_core::ids::ModelProfileId::from(
                    r.get::<String, _>("model_profile_id"),
                ),
                current_slice_id: slice_id_str.map(SliceId::from),
                pending_operation_id: op_id_str.map(OperationId::from),
                created_at,
                updated_at,
                completed_at,
            });
        }

        Ok(list)
    }

    async fn update_process_state_transactional(
        &self,
        process_id: &ProcessId,
        expected_version: u64,
        target_state: ProcessState,
        event: &ProcessEvent,
    ) -> Result<Process, PersistenceError> {
        let mut tx = self.pool.begin().await?;

        let now = Utc::now();
        let updated_at_str = now.to_rfc3339();
        let target_state_str = target_state.as_str();
        let new_version = expected_version + 1;
        let completed_at_str = if target_state.is_terminal() {
            Some(now.to_rfc3339())
        } else {
            None
        };

        let update_result = sqlx::query(
            r#"
            UPDATE processes
            SET state = ?,
                state_version = ?,
                updated_at = ?,
                completed_at = COALESCE(?, completed_at)
            WHERE process_id = ?
              AND state_version = ?
            "#,
        )
        .bind(target_state_str)
        .bind(new_version as i64)
        .bind(updated_at_str)
        .bind(completed_at_str)
        .bind(process_id.as_str())
        .bind(expected_version as i64)
        .execute(&mut *tx)
        .await?;

        if update_result.rows_affected() == 0 {
            let exists = sqlx::query("SELECT state_version FROM processes WHERE process_id = ?")
                .bind(process_id.as_str())
                .fetch_optional(&mut *tx)
                .await?;

            if exists.is_none() {
                return Err(PersistenceError::ProcessNotFound {
                    process_id: process_id.to_string(),
                });
            } else {
                return Err(PersistenceError::VersionConflict {
                    process_id: process_id.to_string(),
                    expected_version,
                });
            }
        }

        // Insert event
        let payload_json = serde_json::to_string(&event.payload)?;
        let event_timestamp = event.timestamp.to_rfc3339();
        let event_type_str = event.event_type.as_str();
        let slice_str = event.slice_id.as_ref().map(|s| s.as_str());
        let op_str = event.operation_id.as_ref().map(|o| o.as_str());

        sqlx::query(
            r#"
            INSERT INTO process_events (
                event_id,
                process_id,
                sequence,
                event_type,
                timestamp,
                payload_json,
                slice_id,
                operation_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(event.event_id.as_str())
        .bind(event.process_id.as_str())
        .bind(event.sequence as i64)
        .bind(event_type_str)
        .bind(event_timestamp)
        .bind(payload_json)
        .bind(slice_str)
        .bind(op_str)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        let updated_process = self.get_process(process_id).await?.ok_or_else(|| {
            PersistenceError::ProcessNotFound {
                process_id: process_id.to_string(),
            }
        })?;

        Ok(updated_process)
    }

    async fn get_events(
        &self,
        process_id: &ProcessId,
        from_sequence: Option<u64>,
    ) -> Result<Vec<ProcessEvent>, PersistenceError> {
        let seq_filter = from_sequence.unwrap_or(0) as i64;
        let rows = sqlx::query(
            r#"
            SELECT
                event_id,
                process_id,
                sequence,
                event_type,
                timestamp,
                payload_json,
                slice_id,
                operation_id
            FROM process_events
            WHERE process_id = ?
              AND sequence >= ?
            ORDER BY sequence ASC
            "#,
        )
        .bind(process_id.as_str())
        .bind(seq_filter)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::new();
        for r in rows {
            let timestamp_str: String = r.get("timestamp");
            let timestamp = parse_datetime(&timestamp_str)?;
            let payload_json: String = r.get("payload_json");
            let payload: serde_json::Value = serde_json::from_str(&payload_json)?;
            let event_type_str: String = r.get("event_type");
            let event_type: cognitive_core::EventType =
                serde_json::from_value(serde_json::Value::String(event_type_str))?;

            let slice_id_str: Option<String> = r.get("slice_id");
            let op_id_str: Option<String> = r.get("operation_id");
            let sequence_i64: i64 = r.get("sequence");

            events.push(ProcessEvent {
                event_id: EventId::from(r.get::<String, _>("event_id")),
                process_id: ProcessId::from(r.get::<String, _>("process_id")),
                sequence: sequence_i64 as u64,
                event_type,
                timestamp,
                payload,
                slice_id: slice_id_str.map(SliceId::from),
                operation_id: op_id_str.map(OperationId::from),
            });
        }

        Ok(events)
    }

    async fn get_latest_sequence(&self, process_id: &ProcessId) -> Result<u64, PersistenceError> {
        let row = sqlx::query(
            r#"
            SELECT COALESCE(MAX(sequence), 0) as max_seq
            FROM process_events
            WHERE process_id = ?
            "#,
        )
        .bind(process_id.as_str())
        .fetch_one(&self.pool)
        .await?;

        let max_seq: i64 = row.get("max_seq");
        Ok(max_seq as u64)
    }
}
