# CognitiveOS
Reasoning as Compute, Context as Working Memory, Experience as Persistent State

For humans: Talk to your work, not your chat history.
For agents: Execute through the OS, not through an endless conversation.

---

## Current Status: Phase 0 & Phase 1 Implemented

CognitiveOS is currently implemented through **Phase 0 (Architecture & Contracts)** and **Phase 1 (Persistent Process Kernel)**.

### Implemented Capabilities (Phase 0 & 1):
- **Core Domain & Contracts (`cognitive-core`)**:
  - Strongly typed identifiers (`ProcessId`, `WorkspaceId`, `EventId`, `SliceId`, `OperationId`, etc.).
  - Formal process state machine (`New`, `Ready`, `Running`, `WaitingForIo`, `WaitingForInput`, `WaitingForApproval`, `WaitingForChildren`, `Paused`, `Completed`, `Failed`, `Cancelled`).
  - Pure transition validation preventing illegal state transitions, self-transitions, and resumption of terminal states.
  - Typed instruction set (`CognitiveInstruction`) and model response contract specifications.
  - Append-only event models with monotonic sequences.
- **Durable Persistence (`cognitive-persistence`)**:
  - SQLite backend with WAL mode, foreign keys, and explicit migrations (`0001_initial_schema.sql`).
  - Atomic state transitions using optimistic locking (`state_version`).
  - Append-only `process_events` journal committed atomically with process state projections.
  - Complete recovery across CLI/runtime restarts.
- **Kernel Service (`cognitive-kernel`)**:
  - Application service providing clean APIs for process creation, lookup, listing, pausing, resuming, and cancelling.
  - Enforces lifecycle invariants and state version increments.
- **CLI (`cog` via `apps/cogctl`)**:
  - `cog run "<objective>" --workspace <path>`: Validates input, creates a persistent process in `New`, and admits it to `Ready`.
  - `cog ps`: Lists all processes with state, version, and objective.
  - `cog inspect <process_id>`: Shows full process details and monotonic event timeline.
  - `cog pause <process_id>`: Transitions eligible process to `Paused`.
  - `cog resume <process_id>`: Resumes paused process to `Ready`.
  - `cog cancel <process_id>`: Transitions non-terminal process to `Cancelled`.
  - Global `--db <path>` and `--json` support.

### Planned for Future Phases (Explicitly Not Yet Implemented):
- **Phase 2**: Native Cognitive CPU, LLM inference backend adapters, bounded execution loop, scheduler.
- **Phase 3**: Sandboxed tool registry and durable external I/O execution.
- **Phase 4**: Virtual context memory and token compilation policies.
- **Phase 5**: Ratatui-based TUI (`cog top`).
- **Phase 6**: Programmable reasoning policies.
- **Phase 7**: Multi-agent IPC and fork/join.

---

## Quickstart

### Build
```bash
cargo build --workspace
```

### Run Tests
```bash
cargo test --workspace
```

### Try the CLI
```bash
# Create and admit a process to READY
target/debug/cog run "Investigate memory leak" --workspace .

# List processes
target/debug/cog ps

# Inspect process state and event journal
target/debug/cog inspect <process_id>

# Pause, resume, and cancel
target/debug/cog pause <process_id>
target/debug/cog resume <process_id>
target/debug/cog cancel <process_id>
```
