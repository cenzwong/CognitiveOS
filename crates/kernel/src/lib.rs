pub mod errors;
pub mod service;

pub use errors::KernelError;
pub use service::KernelService;

#[cfg(test)]
mod tests {
    use super::*;
    use cognitive_core::ids::ProcessId;
    use cognitive_core::state::ProcessState;
    use cognitive_persistence::sqlite::{create_sqlite_pool, run_migrations};
    use cognitive_persistence::SqliteProcessRepository;
    use std::sync::Arc;

    async fn setup_kernel() -> KernelService<SqliteProcessRepository> {
        let pool = create_sqlite_pool("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        let repo = Arc::new(SqliteProcessRepository::new(pool));
        KernelService::new(repo)
    }

    #[tokio::test]
    async fn test_run_process_admitted_to_ready() {
        let kernel = setup_kernel().await;

        let process = kernel
            .run_process("Solve bug".to_string(), "/tmp/ws".to_string())
            .await
            .unwrap();

        assert_eq!(process.state, ProcessState::Ready);
        assert_eq!(process.state_version, 1);
        assert_eq!(process.objective, "Solve bug");

        let events = kernel
            .get_process_events(&process.process_id, None)
            .await
            .unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0].event_type,
            cognitive_core::EventType::ProcessCreated
        );
        assert_eq!(
            events[1].event_type,
            cognitive_core::EventType::StateTransition
        );
    }

    #[tokio::test]
    async fn test_pause_resume_cancel_flow() {
        let kernel = setup_kernel().await;

        let process = kernel
            .run_process("Multi-step task".to_string(), ".".to_string())
            .await
            .unwrap();

        let proc_id = &process.process_id;

        let paused = kernel.pause_process(proc_id, None).await.unwrap();
        assert_eq!(paused.state, ProcessState::Paused);
        assert_eq!(paused.state_version, 2);

        let resumed = kernel.resume_process(proc_id, None).await.unwrap();
        assert_eq!(resumed.state, ProcessState::Ready);
        assert_eq!(resumed.state_version, 3);

        let cancelled = kernel.cancel_process(proc_id, None).await.unwrap();
        assert_eq!(cancelled.state, ProcessState::Cancelled);
        assert_eq!(cancelled.state_version, 4);

        assert!(kernel.resume_process(proc_id, None).await.is_err());
        assert!(kernel.pause_process(proc_id, None).await.is_err());
        assert!(kernel.cancel_process(proc_id, None).await.is_err());
    }

    #[tokio::test]
    async fn test_invalid_arguments() {
        let kernel = setup_kernel().await;
        assert!(kernel
            .run_process("".to_string(), ".".to_string())
            .await
            .is_err());
        assert!(kernel
            .run_process("valid".to_string(), " ".to_string())
            .await
            .is_err());
        assert!(kernel
            .get_process(&ProcessId::from("nonexistent"))
            .await
            .is_err());
    }
}
