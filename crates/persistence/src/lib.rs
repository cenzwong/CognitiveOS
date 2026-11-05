pub mod errors;
pub mod repository;
pub mod sqlite;

pub use errors::PersistenceError;
pub use repository::{ProcessRepository, SqliteProcessRepository};
pub use sqlite::{create_sqlite_pool, run_migrations};

#[cfg(test)]
mod tests {
    use super::*;
    use cognitive_core::events::{EventType, ProcessEvent};
    use cognitive_core::ids::{PrincipalId, ProcessId, WorkspaceId};
    use cognitive_core::process::{Process, ProcessSpecification};
    use cognitive_core::state::ProcessState;

    async fn setup_test_repo() -> SqliteProcessRepository {
        let pool = create_sqlite_pool("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        SqliteProcessRepository::new(pool)
    }

    #[tokio::test]
    async fn test_create_and_get_process() {
        let repo = setup_test_repo().await;

        let proc_id = ProcessId::new_v7();
        let spec = ProcessSpecification {
            objective: "Fix bug in tests".to_string(),
            workspace_id: WorkspaceId::from("ws-test"),
            owner_principal_id: PrincipalId::from("user-1"),
            acceptance_criteria: vec![],
            constraints: vec![],
            program_id: cognitive_core::ids::ProgramId::from("default_agent"),
            model_profile_id: cognitive_core::ids::ModelProfileId::from("default_model"),
            parent_process_id: None,
            metadata: serde_json::json!({}),
        };

        let process = Process::new(proc_id.clone(), spec);
        let event = ProcessEvent::new(
            proc_id.clone(),
            1,
            EventType::ProcessCreated,
            serde_json::json!({"objective": &process.objective}),
        );

        repo.create_process(&process, &event).await.unwrap();

        let fetched = repo
            .get_process(&proc_id)
            .await
            .unwrap()
            .expect("process exists");
        assert_eq!(fetched.process_id, proc_id);
        assert_eq!(fetched.state, ProcessState::New);
        assert_eq!(fetched.state_version, 0);

        let events = repo.get_events(&proc_id, None).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sequence, 1);
        assert_eq!(events[0].event_type, EventType::ProcessCreated);
    }

    #[tokio::test]
    async fn test_version_conflict_optimistic_locking() {
        let repo = setup_test_repo().await;

        let proc_id = ProcessId::new_v7();
        let spec = ProcessSpecification {
            objective: "Concurrency test".to_string(),
            workspace_id: WorkspaceId::from("ws-test"),
            owner_principal_id: PrincipalId::from("user-1"),
            acceptance_criteria: vec![],
            constraints: vec![],
            program_id: cognitive_core::ids::ProgramId::from("default_agent"),
            model_profile_id: cognitive_core::ids::ModelProfileId::from("default_model"),
            parent_process_id: None,
            metadata: serde_json::json!({}),
        };

        let process = Process::new(proc_id.clone(), spec);
        let event = ProcessEvent::new(
            proc_id.clone(),
            1,
            EventType::ProcessCreated,
            serde_json::json!({}),
        );
        repo.create_process(&process, &event).await.unwrap();

        // Successful transition 0 -> 1 (New -> Ready)
        let transition_event = ProcessEvent::state_transition(
            proc_id.clone(),
            2,
            ProcessState::New,
            ProcessState::Ready,
            None,
        );
        let updated = repo
            .update_process_state_transactional(&proc_id, 0, ProcessState::Ready, &transition_event)
            .await
            .unwrap();
        assert_eq!(updated.state, ProcessState::Ready);
        assert_eq!(updated.state_version, 1);

        // Conflict: attempt with stale expected_version 0
        let stale_event = ProcessEvent::state_transition(
            proc_id.clone(),
            3,
            ProcessState::Ready,
            ProcessState::Paused,
            None,
        );
        let res = repo
            .update_process_state_transactional(&proc_id, 0, ProcessState::Paused, &stale_event)
            .await;
        assert!(matches!(res, Err(PersistenceError::VersionConflict { .. })));

        // Event was not inserted because transaction rolled back
        let events = repo.get_events(&proc_id, None).await.unwrap();
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn test_persistence_across_restart() {
        let temp_dir = std::env::temp_dir().join(format!("cog_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");
        let db_url = format!("sqlite://{}", db_path.display());

        let proc_id = ProcessId::new_v7();

        // Scope 1: create process and transition
        {
            let pool = create_sqlite_pool(&db_url).await.unwrap();
            run_migrations(&pool).await.unwrap();
            let repo = SqliteProcessRepository::new(pool.clone());

            let spec = ProcessSpecification {
                objective: "Restart test".to_string(),
                workspace_id: WorkspaceId::from("ws-restart"),
                owner_principal_id: PrincipalId::from("user-1"),
                acceptance_criteria: vec![],
                constraints: vec!["No crash".to_string()],
                program_id: cognitive_core::ids::ProgramId::from("default_agent"),
                model_profile_id: cognitive_core::ids::ModelProfileId::from("default_model"),
                parent_process_id: None,
                metadata: serde_json::json!({}),
            };
            let process = Process::new(proc_id.clone(), spec);
            let event = ProcessEvent::new(
                proc_id.clone(),
                1,
                EventType::ProcessCreated,
                serde_json::json!({}),
            );
            repo.create_process(&process, &event).await.unwrap();

            let transition_event = ProcessEvent::state_transition(
                proc_id.clone(),
                2,
                ProcessState::New,
                ProcessState::Ready,
                Some("Initial admission".to_string()),
            );
            repo.update_process_state_transactional(
                &proc_id,
                0,
                ProcessState::Ready,
                &transition_event,
            )
            .await
            .unwrap();

            pool.close().await;
        }

        // Scope 2: simulate restart by reopening the database
        {
            let pool = create_sqlite_pool(&db_url).await.unwrap();
            let repo = SqliteProcessRepository::new(pool.clone());

            let process = repo
                .get_process(&proc_id)
                .await
                .unwrap()
                .expect("Process preserved");
            assert_eq!(process.state, ProcessState::Ready);
            assert_eq!(process.state_version, 1);
            assert_eq!(process.specification.constraints, vec!["No crash"]);

            let events = repo.get_events(&proc_id, None).await.unwrap();
            assert_eq!(events.len(), 2);
            assert_eq!(events[1].event_type, EventType::StateTransition);

            pool.close().await;
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
