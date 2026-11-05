-- CognitiveOS Initial Schema: Processes and Process Events

CREATE TABLE IF NOT EXISTS processes (
    process_id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    parent_process_id TEXT,
    owner_principal_id TEXT NOT NULL,

    objective TEXT NOT NULL,
    specification_json TEXT NOT NULL,

    state TEXT NOT NULL,
    state_version INTEGER NOT NULL DEFAULT 0,

    program_id TEXT NOT NULL,
    model_profile_id TEXT NOT NULL,

    current_slice_id TEXT,
    pending_operation_id TEXT,

    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    completed_at TEXT,

    FOREIGN KEY (parent_process_id) REFERENCES processes(process_id)
);

CREATE INDEX IF NOT EXISTS idx_processes_workspace ON processes(workspace_id);
CREATE INDEX IF NOT EXISTS idx_processes_state ON processes(state);

CREATE TABLE IF NOT EXISTS process_events (
    event_id TEXT PRIMARY KEY,
    process_id TEXT NOT NULL,
    sequence INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    slice_id TEXT,
    operation_id TEXT,

    FOREIGN KEY (process_id) REFERENCES processes(process_id) ON DELETE CASCADE,
    UNIQUE(process_id, sequence)
);

CREATE INDEX IF NOT EXISTS idx_process_events_process_seq ON process_events(process_id, sequence);
CREATE INDEX IF NOT EXISTS idx_process_events_type ON process_events(event_type);
