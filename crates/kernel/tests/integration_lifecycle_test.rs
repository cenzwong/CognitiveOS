use cognitive_core::ids::ProcessId;
use cognitive_core::state::ProcessState;
use cognitive_kernel::KernelService;
use cognitive_persistence::sqlite::{create_sqlite_pool, run_migrations};
use cognitive_persistence::{ProcessRepository, SqliteProcessRepository};
use std::sync::Arc;

#[tokio::test]
async fn test_full_process_lifecycle_and_restart() {
    let temp_dir = std::env::temp_dir().join(format!("cog_integration_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let db_path = temp_dir.join("cognitive.db");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    let proc_id: ProcessId;

    // Session 1: Run process (New -> Ready)
    {
        let pool = create_sqlite_pool(&db_url).await.unwrap();
        run_migrations(&pool).await.unwrap();
        let repo = Arc::new(SqliteProcessRepository::new(pool.clone()));
        let kernel = KernelService::new(repo);

        let process = kernel
            .run_process(
                "Test persistent agent execution lifecycle".to_string(),
                "/workspace/target".to_string(),
            )
            .await
            .unwrap();

        assert_eq!(process.state, ProcessState::Ready);
        assert_eq!(process.state_version, 1);
        proc_id = process.process_id;

        pool.close().await;
    }

    // Session 2: Pause process across simulated restart
    {
        let pool = create_sqlite_pool(&db_url).await.unwrap();
        let repo = Arc::new(SqliteProcessRepository::new(pool.clone()));
        let kernel = KernelService::new(repo);

        let process = kernel.get_process(&proc_id).await.unwrap();
        assert_eq!(process.state, ProcessState::Ready);
        assert_eq!(process.state_version, 1);

        let paused = kernel
            .pause_process(
                &proc_id,
                Some("Human intervention: pausing for review".to_string()),
            )
            .await
            .unwrap();
        assert_eq!(paused.state, ProcessState::Paused);
        assert_eq!(paused.state_version, 2);

        pool.close().await;
    }

    // Session 3: Resume and cancel
    {
        let pool = create_sqlite_pool(&db_url).await.unwrap();
        let repo = Arc::new(SqliteProcessRepository::new(pool.clone()));
        let kernel = KernelService::new(repo);

        let resumed = kernel.resume_process(&proc_id, None).await.unwrap();
        assert_eq!(resumed.state, ProcessState::Ready);
        assert_eq!(resumed.state_version, 3);

        let cancelled = kernel
            .cancel_process(&proc_id, Some("Task superseded".to_string()))
            .await
            .unwrap();
        assert_eq!(cancelled.state, ProcessState::Cancelled);
        assert_eq!(cancelled.state_version, 4);
        assert!(cancelled.completed_at.is_some());

        let illegal_resume = kernel.resume_process(&proc_id, None).await;
        assert!(illegal_resume.is_err());

        let events = kernel.get_process_events(&proc_id, None).await.unwrap();
        assert_eq!(events.len(), 5);
        for (i, ev) in events.iter().enumerate() {
            assert_eq!(ev.sequence, (i + 1) as u64);
        }

        assert_eq!(
            events[0].event_type,
            cognitive_core::EventType::ProcessCreated
        );
        assert_eq!(
            events[1].event_type,
            cognitive_core::EventType::StateTransition
        );
        assert_eq!(
            events[2].event_type,
            cognitive_core::EventType::ProcessPaused
        );
        assert_eq!(
            events[3].event_type,
            cognitive_core::EventType::ProcessResumed
        );
        assert_eq!(
            events[4].event_type,
            cognitive_core::EventType::ProcessCancelled
        );

        pool.close().await;
    }

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_concurrency_optimistic_lock_rejection() {
    let pool = create_sqlite_pool("sqlite::memory:").await.unwrap();
    run_migrations(&pool).await.unwrap();
    let repo = Arc::new(SqliteProcessRepository::new(pool.clone()));
    let kernel = KernelService::new(repo.clone());

    let process = kernel
        .run_process("Optimistic lock test".to_string(), ".".to_string())
        .await
        .unwrap();

    let proc_id = &process.process_id;

    let event1 = cognitive_core::ProcessEvent::state_transition(
        proc_id.clone(),
        3,
        ProcessState::Ready,
        ProcessState::Paused,
        None,
    );
    let event2 = cognitive_core::ProcessEvent::state_transition(
        proc_id.clone(),
        3,
        ProcessState::Ready,
        ProcessState::Cancelled,
        None,
    );

    let res1 = repo
        .update_process_state_transactional(proc_id, 1, ProcessState::Paused, &event1)
        .await;
    assert!(res1.is_ok());

    let res2 = repo
        .update_process_state_transactional(proc_id, 1, ProcessState::Cancelled, &event2)
        .await;
    assert!(matches!(
        res2,
        Err(cognitive_persistence::errors::PersistenceError::VersionConflict { .. })
    ));
}
