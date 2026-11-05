# CognitiveOS Persistence Model

## Storage Engine

CognitiveOS uses SQLite in WAL (Write-Ahead Logging) mode with foreign keys enabled.

## Schema

### `processes`
- `process_id` (TEXT PRIMARY KEY): Stable identifier (`proc-...`).
- `workspace_id` (TEXT NOT NULL): Workspace reference.
- `parent_process_id` (TEXT NULL): Parent process hierarchy reference.
- `owner_principal_id` (TEXT NOT NULL): Owner identity.
- `objective` (TEXT NOT NULL): High-level task objective.
- `specification_json` (TEXT NOT NULL): Full process specification.
- `state` (TEXT NOT NULL): Current process state.
- `state_version` (INTEGER NOT NULL DEFAULT 0): Monotonic version counter.
- `program_id` (TEXT NOT NULL): Configured program policy.
- `model_profile_id` (TEXT NOT NULL): Configured model profile.
- `current_slice_id` (TEXT NULL): Current cognitive slice lease.
- `pending_operation_id` (TEXT NULL): Active durable operation.
- `created_at` (TEXT NOT NULL): ISO8601 creation timestamp.
- `updated_at` (TEXT NOT NULL): ISO8601 update timestamp.
- `completed_at` (TEXT NULL): Terminal completion timestamp.

### `process_events`
- `event_id` (TEXT PRIMARY KEY): Stable identifier (`evt-...`).
- `process_id` (TEXT NOT NULL): Process foreign key.
- `sequence` (INTEGER NOT NULL): Monotonically increasing per-process sequence number.
- `event_type` (TEXT NOT NULL): Event classification (`PROCESS_CREATED`, `STATE_TRANSITION`, etc.).
- `timestamp` (TEXT NOT NULL): Event timestamp.
- `payload_json` (TEXT NOT NULL): Event structured data.
- `slice_id` (TEXT NULL): Associated slice reference.
- `operation_id` (TEXT NULL): Associated operation reference.
