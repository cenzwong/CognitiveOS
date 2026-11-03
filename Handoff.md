# CognitiveOS
## Native Cognitive Computing Runtime — Complete Engineering Development Handoff

**Project codename:** CognitiveOS  
**Document type:** Architecture Specification, Engineering Blueprint, Implementation Handoff  
**Status:** Proposed / Ready for Implementation  
**Target platform:** macOS (Apple Silicon first), Linux-compatible architecture  
**Primary language:** Rust  
**Initial interface:** CLI + local API  
**Secondary interface:** TUI  
**Primary execution mode:** Native LLM Runtime  
**Design philosophy:** Local-first, process-centric, event-driven, durable, observable, model-agnostic  
**Development approach:** Contract-first, test-driven, incremental, AI-assisted

---

# Part I — Vision and First Principles

## 1. Executive Summary

CognitiveOS is an experimental operating environment for persistent, long-running LLM agents.

Its fundamental hypothesis is that modern LLM-based agents are unnecessarily constrained by conversation-centric execution models.

Most contemporary agent systems repeatedly:

1. Construct a prompt containing instructions and conversation history.
2. Invoke an LLM.
3. Parse its output or tool calls.
4. Execute tools.
5. Append results to the conversation.
6. Repeat.

Although this approach is effective for many tasks, it conflates several fundamentally different responsibilities:

- Computation
- Working memory
- Persistent memory
- Execution state
- Process lifecycle
- Tool execution
- User interaction
- Scheduling
- Recovery
- Learning from experience

CognitiveOS proposes treating the LLM as a replaceable computational resource within a larger stateful operating environment.

Its central analogy is:

| Conventional Computing | Cognitive Computing |
|---|---|
| CPU | LLM inference engine |
| Instruction execution | Structured reasoning operation |
| CPU time slice | One bounded model execution |
| Registers | Current control and task state |
| RAM | Active context window |
| Virtual memory | Context paging and retrieval |
| Storage | Persistent memory, events and artifacts |
| Process | Durable agent task |
| Thread / execution branch | Parallel reasoning or task branch |
| System call | Typed agent operation |
| Interrupt | External event requiring process attention |
| Scheduler | LLM and agent execution scheduler |
| Kernel | Deterministic runtime controller |
| Device driver | Model or tool adapter |
| Operating system | CognitiveOS |

This analogy is conceptual rather than literal.

An LLM invocation is not a hardware CPU cycle, and the context window is not physically addressable RAM. The system must not rely on assumptions about access to model internals, hidden reasoning states, or portable KV caches.

Instead, CognitiveOS implements the useful properties of operating systems at the application-runtime level.

**The key architectural principle is:**

> The LLM performs probabilistic computation. CognitiveOS maintains deterministic, durable, inspectable execution state.

The project must demonstrate this distinction experimentally rather than assuming it provides an improvement.

---

## 2. Primary Research Question

The principal research question is:

**Can a stateless LLM become a more reliable, efficient and capable long-running agent when managed by an operating-system-inspired runtime providing persistent process state, virtual context management, event-driven execution, verification and recovery?**

Secondary questions include:

1. Can an agent preserve task continuity without replaying its entire conversation?
2. Can structured process state outperform full conversation replay?
3. Can context retrieval improve information relevance while reducing unnecessary tokens?
4. Can an interrupted agent resume without losing important constraints?
5. Can execution be separated from the model provider?
6. Can the same process migrate between compatible models?
7. Can different reasoning strategies be selected dynamically?
8. Can successful experiences become reusable procedural skills?
9. Can multiple agents coordinate using structured IPC rather than shared conversations?
10. Can the system improve its reasoning policies through measurable feedback?

These are research hypotheses, not guaranteed outcomes.

In particular, full conversation replay with provider-side prompt caching is a legitimate and potentially strong baseline.

---

## 3. Product Definition

CognitiveOS is:

- A durable agent execution runtime.
- A virtual context and memory-management system.
- A process and resource scheduler.
- A typed system-call interface for LLM agents.
- A platform for researching long-horizon reasoning.
- A foundation for future distributed cognitive computing.
- An observable environment for debugging and evaluating agent behaviour.

CognitiveOS is not:

- A replacement for macOS or Linux.
- A traditional hardware operating system.
- A ChatGPT clone.
- A Codex or Claude Code clone.
- Merely another tool-calling loop.
- A wrapper around existing coding harnesses.
- A multi-agent chatroom.
- A requirement to build an entirely new LLM.
- A system that assumes longer execution always produces better results.

**Product statement:**

> CognitiveOS is a durable, process-oriented operating environment in which stateless language models execute structured reasoning operations over persistent, virtualised cognitive state.

---

## 4. Fundamental Design Principles

### 4.1 Native execution first

The initial system must implement its own LLM execution loop.

It must control:

- Prompt construction
- Context selection
- Model invocation
- Structured output parsing
- System-call validation
- Tool dispatch
- Process transitions
- Memory updates
- Event recording
- Recovery

Existing coding harnesses must not provide the underlying cognitive execution loop.

A future compatibility layer may host external harnesses, but this is explicitly outside the initial research scope.

### 4.2 The process is the primary abstraction

An agent process must survive:

- The user closing their terminal
- The user leaving a conversation
- Restarting the CLI
- Restarting the daemon
- An LLM provider failure
- Context compaction
- A recoverable worker failure

The process is independent of any particular conversation session.

### 4.3 The model is replaceable

Process state must not be stored exclusively inside provider-specific conversation objects.

A compatible process should be executable using a different LLM backend without reconstructing its entire history.

Model migration may alter performance or behaviour and does not guarantee identical results.

### 4.4 Context is a compiled working set

The LLM should receive a context package assembled from persistent process state and relevant information.

The system must not assume the complete conversation is always required.

However, it must support complete-history replay as a comparison strategy.

### 4.5 The kernel owns state transitions

LLM output is an untrusted proposal.

The kernel decides whether the proposal is valid and permitted.

The model cannot directly:

- Modify process state outside authorised operations
- Execute arbitrary commands
- Change its own privileges
- Bypass approval requirements
- Modify authoritative audit records
- Declare an external operation successful without evidence

### 4.6 Durable execution is mandatory

Every important operation must have persistent identity and observable state.

The runtime must handle crashes between:

- Inference request and response
- Instruction validation and dispatch
- Tool execution and result persistence
- Process state update and notification

### 4.7 Reliability precedes intelligence

The development order must be:

1. Correct execution model
2. Durable process lifecycle
3. Memory and context management
4. Evaluation and observability
5. Advanced reasoning
6. Multi-agent coordination
7. Self-improvement
8. Distributed computing

### 4.8 Complexity must be earned

Do not introduce a distributed database, vector database, custom programming language or elaborate plugin framework until a concrete requirement justifies it.

Start with a small, well-tested Rust implementation.

---

# Part II — System Architecture

## 5. High-Level Architecture

```text
                  USER / AGENT CLIENTS
                          |
       +------------------+------------------+
       |                  |                  |
      CLI                TUI              SDK / API
       |                  |                  |
       +------------------+------------------+
                          |
                  CONTROL INTERFACE
                          |
                 +-----------------+
                 | CognitiveOS API |
                 +-----------------+
                          |
                +-------------------+
                |  COGNITIVE KERNEL |
                +-------------------+
                          |
          +---------------+---------------+
          |               |               |
     Process Manager   Scheduler     Security Manager
          |               |               |
          +---------------+---------------+
                          |
                 EXECUTION CONTROLLER
                          |
                +-------------------+
                | Context Compiler  |
                +-------------------+
                          |
                +-------------------+
                |   LLM CPU Slice   |
                +-------------------+
                          |
                 Structured Decision
                          |
                +-------------------+
                | Syscall Validator |
                +-------------------+
                          |
          +---------------+---------------+
          |               |               |
      Memory I/O       Tool I/O      Process Control
          |               |               |
          +---------------+---------------+
                          |
                   PERSISTENT STATE
                          |
         +----------------+----------------+
         |                |                |
       SQLite          Artifacts       Event Journal
```

The diagram represents logical components. These should initially run inside one daemon, not as separate network services.

---

## 6. Architectural Layers

### Layer 1 — Human and Agent Interfaces

Responsibilities:

- Create processes
- Observe processes
- Send instructions
- Inspect memory
- Inspect events
- Request cancellation
- Manage approvals
- Retrieve artifacts

Initial implementations:

- CLI
- Local API

Later implementations:

- Ratatui TUI
- Desktop application
- Web interface
- Mobile client
- Agent SDK
- MCP adapter

All interfaces must use one authoritative kernel API.

### Layer 2 — Cognitive Kernel

Responsibilities:

- Process identity
- Process lifecycle
- State transitions
- Resource accounting
- Scheduling
- Permissions
- Cancellation
- Approval management
- Durable operation management

Kernel behaviour should be deterministic for the same committed state and command, excluding external results.

### Layer 3 — Cognitive Execution Engine

Responsibilities:

- Compile current context
- Select model backend
- Invoke the LLM
- Validate structured responses
- Translate responses into proposed operations
- Submit operations to the kernel

The execution engine must not directly mutate the database.

### Layer 4 — Memory and Context Subsystem

Responsibilities:

- Persistent working state
- Episodic memory
- Semantic memory
- Procedural memory
- Context retrieval
- Context budget allocation
- Context compaction
- Provenance tracking
- Memory versioning

### Layer 5 — I/O Subsystem

Responsibilities:

- Tool registration
- Tool execution
- Result persistence
- External service adapters
- Sandboxing
- Secret handling
- Idempotency
- Failure reconciliation

### Layer 6 — Persistence and Observability

Responsibilities:

- Durable process records
- Append-only execution events
- Checkpoints
- Artifact references
- Metrics
- Traces
- Audit history

---

# Part III — Core Object Model

## 7. Authoritative Objects

CognitiveOS must define these core objects independently of any LLM framework.

| Object | Description |
|---|---|
| Principal | Identity of a human, agent or authorised service |
| Workspace | Logical project and resource boundary |
| Process | Persistent agent task |
| Session | Temporary human or agent attachment to a process |
| Program | Versioned execution and reasoning policy |
| Slice | One bounded invocation of the model |
| Instruction | Structured operation proposed by the model |
| Operation | Persisted execution of an instruction |
| Event | Immutable record of a state change or observation |
| Memory | Versioned information available to a process |
| Artifact | Durable externally inspectable output |
| Checkpoint | Recoverable process snapshot |
| Approval | Authorisation decision for a restricted operation |
| Capability | Permission to access a specific resource or operation |

Do not collapse these into a single `Agent` object.

---

## 8. Process Definition

A process represents an independently managed unit of cognitive work.

A process must contain:

```text
Process
├── identity
│   ├── process_id
│   ├── parent_process_id
│   ├── workspace_id
│   └── owner_principal_id
├── specification
│   ├── objective
│   ├── acceptance_criteria
│   ├── constraints
│   └── program_version
├── execution
│   ├── state
│   ├── state_version
│   ├── current_slice
│   ├── pending_operation
│   ├── execution_policy
│   └── model_profile
├── resources
│   ├── token_budget
│   ├── monetary_budget
│   ├── wall_time_limit
│   ├── tool_permissions
│   └── child_process_limit
├── memory
│   ├── working_state_ref
│   ├── event_cursor
│   ├── memory_namespace
│   └── artifact_refs
└── lifecycle
    ├── created_at
    ├── updated_at
    ├── started_at
    ├── completed_at
    └── last_checkpoint_id
```

Identifiers should be stable and machine-readable, such as UUIDv7 or equivalent sortable unique identifiers.

Do not use a process's current name as its identity.

---

## 9. Process States

Define the following initial states:

```rust
pub enum ProcessState {
    New,
    Ready,
    Running,
    WaitingForIo,
    WaitingForInput,
    WaitingForApproval,
    WaitingForChildren,
    Paused,
    Completed,
    Failed,
    Cancelled,
}
```

### State meanings

**New**

The process has been created but not admitted to execution.

**Ready**

The process is eligible for scheduling.

**Running**

The process currently owns an execution lease and is being processed.

**WaitingForIo**

The process is blocked on an external operation.

**WaitingForInput**

The process needs information from the user.

**WaitingForApproval**

The process requires a permission decision.

**WaitingForChildren**

The process is waiting for child processes.

**Paused**

The process has been administratively suspended.

**Completed**

The process reached its verified completion condition.

**Failed**

The process cannot continue under its current recovery policy.

**Cancelled**

A cancellation request has reached a valid termination boundary.

Terminal states should be immutable by default.

Resuming terminal work should create a new process or an explicitly versioned continuation, rather than silently changing historical terminal state.

---

## 10. State Transition Rules

Examples:

```text
NEW -> READY

READY -> RUNNING

RUNNING -> READY
RUNNING -> WAITING_FOR_IO
RUNNING -> WAITING_FOR_INPUT
RUNNING -> WAITING_FOR_APPROVAL
RUNNING -> WAITING_FOR_CHILDREN
RUNNING -> COMPLETED
RUNNING -> FAILED

WAITING_FOR_IO -> READY
WAITING_FOR_INPUT -> READY
WAITING_FOR_APPROVAL -> READY
WAITING_FOR_CHILDREN -> READY

READY -> PAUSED
WAITING_* -> PAUSED

PAUSED -> READY

NON_TERMINAL -> CANCELLED
```

Every state transition must:

1. Validate the expected source state.
2. Validate the requester's authority.
3. Verify the expected state version.
4. Persist the new state.
5. Append the corresponding event.
6. Commit these changes atomically.

A running external operation may not be immediately cancellable.

The system must distinguish:

- Cancellation requested
- Cancellation delivered
- External work confirmed stopped
- Final process cancellation

---

## 11. Process Invariants

The kernel must enforce:

1. A terminal process cannot be scheduled.
2. A process cannot have two active execution leases.
3. A stale execution attempt cannot commit state.
4. Every committed state transition has an associated journal event.
5. Every persisted operation has a unique identity.
6. An operation cannot be marked successful without an execution result.
7. A process cannot exceed its granted capabilities.
8. Child capabilities cannot exceed the parent's delegated capabilities.
9. A paused process must not start new inference or tool operations.
10. Waiting processes must not consume scheduled model turns through polling.
11. Process state versions increase monotonically.
12. A completed process must have an associated result or artifact reference.
13. Memory updates must use authorised scopes and valid versions.
14. External side effects with uncertain outcomes must not be blindly repeated.
15. Recovery must not rely on reconstructing hidden model reasoning.

These invariants should be represented in executable tests, not only documentation.

---

# Part IV — The Cognitive CPU

## 12. The Fundamental Execution Unit

Define one **Cognitive CPU Slice** as:

> A single bounded LLM invocation that receives a compiled context snapshot and returns one structured decision proposal.

A slice is not guaranteed to be instantaneous, deterministic or safely preemptible.

Its lifecycle is:

```text
FETCH PROCESS STATE
        |
ACQUIRE EXECUTION LEASE
        |
COMPILE CONTEXT
        |
INVOKE MODEL
        |
PARSE RESPONSE
        |
VALIDATE PROPOSAL
        |
PERSIST OPERATION INTENT
        |
DISPATCH OPERATION
        |
COMMIT RESULT / WAIT STATE
        |
RELEASE LEASE
```

For a slow tool, dispatch should create a durable pending operation. The process can release its inference scheduling capacity while waiting.

---

## 13. Cognitive Instruction Set

Define a small logical instruction set.

The initial version should support:

```text
MEMORY_READ
MEMORY_SEARCH
MEMORY_WRITE

TOOL_EXECUTE

PROCESS_YIELD
PROCESS_COMPLETE
PROCESS_FAIL

USER_INPUT_REQUEST
APPROVAL_REQUEST
```

Later versions may introduce:

```text
PROCESS_SPAWN
PROCESS_WAIT
PROCESS_JOIN

MESSAGE_SEND
MESSAGE_RECEIVE

PLAN_UPDATE
STRATEGY_SWITCH

SKILL_INVOKE
SKILL_PROPOSE
```

Do not implement all instructions in the first milestone.

The instruction set should grow only after the execution model is stable.

---

## 14. Typed Instruction Contract

Initial Rust definition:

```rust
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", content = "args", rename_all = "snake_case")]
pub enum CognitiveInstruction {
    MemoryRead {
        address: String,
    },

    MemorySearch {
        query: String,
        limit: u32,
    },

    MemoryWrite {
        address: String,
        expected_version: Option<u64>,
        content: Value,
    },

    ToolExecute {
        tool_id: String,
        arguments: Value,
    },

    RequestInput {
        question: String,
    },

    RequestApproval {
        action: String,
        reason: String,
    },

    Yield {
        reason: String,
    },

    Complete {
        result: Value,
        evidence_refs: Vec<String>,
    },

    Fail {
        reason: String,
        recoverable: bool,
    },
}
```

The first version should enforce one instruction per slice.

This deliberately simplifies:

- Atomicity
- Validation
- Recovery
- State transitions
- Testing
- Debugging

Future optimisation may support multiple independent proposed operations per slice, provided transactional semantics remain explicit.

---

## 15. Model Response Contract

A model response must be a structured proposal, not an executable command.

Example:

```json
{
  "schema_version": 1,
  "decision": {
    "summary": "Tests are required to validate the patch.",
    "instruction": {
      "op": "tool_execute",
      "args": {
        "tool_id": "workspace.run_tests",
        "arguments": {
          "target": "tests/runtime"
        }
      }
    }
  }
}
```

The `summary` field is an optional, concise, user-visible rationale.

Do not require or attempt to capture a model's private chain-of-thought.

The system should record:

- Context snapshot identity
- Model identity
- Proposed action
- Validation outcome
- Observable execution result
- Cost and token usage
- Relevant evidence references

The kernel must reject malformed instructions.

A valid JSON schema is necessary but insufficient for authorisation.

---

## 16. Native Model Execution Contract

Define:

```rust
pub struct InferenceRequest {
    pub process_id: ProcessId,
    pub slice_id: SliceId,
    pub model: ModelProfile,
    pub context: CompiledContext,
    pub max_output_tokens: u32,
}

pub struct InferenceResult {
    pub response: String,
    pub usage: TokenUsage,
    pub provider_request_id: Option<String>,
    pub finish_reason: FinishReason,
}
```

Create an interface resembling:

```rust
#[async_trait::async_trait]
pub trait InferenceBackend: Send + Sync {
    async fn infer(
        &self,
        request: InferenceRequest,
    ) -> Result<InferenceResult, InferenceError>;
}
```

The concrete trait can evolve during implementation.

A backend must not mutate process state.

Backend-specific functionality should be isolated behind adapters.

---

## 17. LLM Backend Requirements

Initially support:

1. A configurable OpenAI-compatible HTTP backend.
2. A deterministic mock backend for tests.

Later support:

- OpenAI native APIs
- Anthropic
- Google Gemini
- Local MLX server
- Ollama
- llama.cpp
- Other compatible model servers

Maintain a capability profile for each model:

```text
model_id
provider
context_window
max_output_tokens
supports_structured_output
supports_tool_calls
supports_prompt_caching
supports_streaming
tokenizer_or_estimator
cost_configuration
```

Do not assume all providers support identical reasoning settings, structured response formats, caching or token accounting.

The native runtime should control the agent loop even when the model is hosted by a remote API.

---

# Part V — Virtual Context and Cognitive Memory

## 18. Context as Working Memory

The context window is the model's temporary accessible working set.

Persistent knowledge must exist outside the model.

CognitiveOS should separate:

```text
ACTIVE CONTEXT
     |
WORKING PROCESS STATE
     |
PERSISTENT MEMORY
     |
EVENT HISTORY
     |
ARTIFACT STORAGE
```

Not all persisted information belongs in every model call.

The context compiler determines which information is needed for the next slice.

---

## 19. Memory Categories

### 19.1 Working Memory

Task-specific state:

- Current objective
- Constraints
- Active plan
- Completed steps
- Pending operations
- Current hypotheses
- Known blockers
- Next recommended action

Working memory should be structured and explicitly versioned.

### 19.2 Episodic Memory

Records of past experiences:

- What was attempted
- What happened
- Which evidence was collected
- Which approach succeeded or failed
- What corrective action followed

Each episode should retain references to its source events.

### 19.3 Semantic Memory

Stable knowledge:

- Repository architecture
- Technical facts
- Domain knowledge
- User-provided reference information
- Verified environment properties

Semantic memory must record provenance and validation status.

### 19.4 Procedural Memory

Reusable operational knowledge:

- Troubleshooting procedures
- Reasoning workflows
- Tested debugging approaches
- Task-specific instructions
- Validated skills

Procedural memory must be versioned.

### 19.5 Event Memory

The execution journal:

- Inputs
- State transitions
- Model decisions
- Tool operations
- Results
- Failures
- Approvals

Event memory must not be silently rewritten.

### 19.6 Artifact Memory

References to external outputs:

- Files
- Reports
- Code diffs
- Test results
- Logs
- Datasets

Large artifacts must not be embedded directly into process state.

---

## 20. Logical Memory Addressing

Use explicit resource identifiers.

Examples:

```text
mem://process/42/working-state
mem://process/42/hypotheses
mem://workspace/demo/architecture

episode://process/42/attempt/7

skill://debugging/memory-leak/v2

artifact://process/42/test-results/991

event://process/42/582
```

The address space must be logical and independent of the physical storage backend.

Memory references should resolve to objects with:

```text
id
address
namespace
type
version
content_ref
source_refs
created_at
updated_at
validation_status
sensitivity
```

Do not treat embeddings as the authoritative memory representation.

An embedding index is a retrieval accelerator.

---

## 21. Context Compiler

The Context Compiler builds the next LLM input.

Input:

```text
Process State
Program Instructions
Model Profile
Execution Policy
Memory References
Recent Events
Tool Permissions
Token Budget
```

Output:

```text
CompiledContext
├── system_contract
├── execution_policy
├── process_objective
├── mandatory_constraints
├── current_working_state
├── relevant_memory
├── relevant_artifact_excerpts
├── recent_operation_result
├── available_instructions
└── response_schema
```

The compiler should be deterministic for the same persisted inputs and compilation policy.

Do not call an LLM merely to assemble ordinary context in the first version.

---

## 22. Context Compilation Algorithm

Suggested algorithm:

```text
1. Load the process and selected program.
2. Load the model capability profile.
3. Calculate available input budget.
4. Reserve space for output and overhead.
5. Add mandatory system and security instructions.
6. Add objective and acceptance criteria.
7. Add immutable user constraints.
8. Add current execution state.
9. Add the latest relevant operation result.
10. Resolve explicit memory references.
11. Retrieve additional relevant evidence if necessary.
12. Rank optional context sections.
13. Enforce the available token budget.
14. Validate mandatory information is retained.
15. Construct the context in a stable order.
16. Record the compilation manifest and content hash.
17. Submit to the inference backend.
```

The compiler must never silently remove mandatory security instructions or user constraints to fit the budget.

If the minimum required context exceeds the model capacity, the process must produce an explicit error or request a different execution strategy.

---

## 23. Token Budgeting

Example configurable allocation:

| Section | Initial target |
|---|---:|
| System contract and execution policy | 10% |
| Objective and constraints | 15% |
| Current working state | 20% |
| Recent events | 15% |
| Retrieved evidence | 30% |
| Reserve / flexibility | 10% |

These percentages are starting heuristics, not strict limits or universally optimal values.

Mandatory sections should take priority over optional retrieval.

Implement explicit budgets rather than depending on accidental provider truncation.

---

## 24. Memory Retrieval Strategy

Use progressive retrieval.

### Stage A — Exact addressing

Prefer known references:

```text
memory.read("mem://process/42/working-state")
```

### Stage B — Structured lookup

Filter by:

- Namespace
- Type
- Process
- Timestamp
- Validation status
- Source
- Tags

### Stage C — Full-text search

Use SQLite FTS where appropriate.

### Stage D — Semantic retrieval

Introduce embeddings or a vector index only when measured retrieval failures justify it.

For the initial system, SQLite plus structured indexing and FTS is sufficient.

A future LanceDB integration may be useful, but it must not be required for MVP.

---

## 25. Context Page Faults

A model may discover that required information is missing from the active context.

Example:

```json
{
  "op": "memory_read",
  "args": {
    "address": "mem://process/42/previous-test-failure"
  }
}
```

The kernel resolves the reference and makes the result available to a subsequent slice.

Conceptually, this resembles paging missing information into working memory.

However:

- It is a software-level semantic retrieval operation.
- It is not a hardware memory page fault.
- It is not guaranteed to be faster than a larger initial context.
- It must be measured.

Record explicit retrieval misses and unnecessary retrieval operations.

---

## 26. Compaction and Summarisation

Compaction must not discard authoritative history.

Implement it as a derived representation.

```text
Raw Events -> Structured State -> Optional Summary
```

The original event record remains accessible.

A compacted state should preserve:

```text
objective
acceptance_criteria
hard_constraints
completed_work
current_plan
pending_operations
verified_evidence
unresolved_questions
known_failures
important_references
```

Never promote an unsupported model-generated summary to verified fact.

Each summary should record:

```text
source_event_range
source_hash
compiler_version
summary_version
created_at
validation_status
```

Summaries must be inspectable and replaceable.

---

## 27. Prompt Caching

Provider-side prompt caching is not equivalent to CognitiveOS memory.

Prompt caching can reduce repeated inference cost for stable prefixes.

CognitiveOS must preserve cache locality where practical.

Context composition order should generally be:

```text
Stable system instructions
Stable operation schema
Stable program instructions
Stable reference material
Dynamic process state
Recent events
Latest tool results
```

Avoid needlessly rewriting stable prefixes.

Track provider-reported:

- Input tokens
- Cached input tokens
- Cache writes, where reported
- Output tokens
- Latency
- Cost

Caching policies must be provider-specific and configurable.

---

## 28. Memory Strategy Experiments

The runtime must support multiple context policies through one interface.

### Policy A — Full Replay

Pass the complete accessible history, subject to provider limits.

### Policy B — Cache-Aware Replay

Preserve append-only history and stable prefix ordering to optimise prompt cache reuse.

### Policy C — Structured Compaction

Use a persistent summary plus recent events.

### Policy D — Virtual Context

Load structured working state and selected memory pages.

### Policy E — Hybrid

Keep a stable reusable prefix while dynamically selecting additional memory.

The system must not hardcode Policy D as universally superior.

All policies must be measurable and interchangeable.

---

# Part VI — Persistence and Durable Execution

## 29. Storage Strategy

Use SQLite for the initial runtime.

Recommended:

- WAL mode
- Foreign keys enabled
- Explicit migrations
- Transactional state transitions
- Prepared queries
- Bounded connection pool
- Regular integrity checks
- Consistent backup procedures

Large artifacts should be stored as files with metadata and references in SQLite.

Do not store arbitrary large tool outputs inside process rows.

---

## 30. Initial Database Entities

Required tables:

```text
workspaces
principals
processes
process_events
execution_slices
operations
checkpoints
memory_entries
memory_versions
artifacts
approvals
sessions
model_profiles
```

Optional later tables:

```text
process_relationships
process_messages
skills
skill_versions
evaluation_runs
reasoning_policies
worker_leases
```

Keep the initial schema smaller if a table is not yet used.

---

## 31. Suggested Process Schema

Illustrative SQL:

```sql
CREATE TABLE processes (
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

    FOREIGN KEY (parent_process_id)
        REFERENCES processes(process_id)
);
```

Use explicit domain validation in the Rust layer as well as database constraints.

---

## 32. Append-Only Event Journal

Every important state change must generate an event.

Example:

```json
{
  "event_id": "evt-0081",
  "process_id": "proc-0042",
  "sequence": 81,
  "type": "TOOL_COMPLETED",
  "operation_id": "op-0020",
  "timestamp": "2026-11-03T12:00:00Z",
  "payload": {
    "exit_code": 0,
    "artifact_ref": "artifact://proc-0042/tests/20"
  }
}
```

Required properties:

- Unique event ID
- Stable process ID
- Monotonic per-process sequence number
- Event type
- Timestamp
- Structured payload
- Optional operation and slice references

Events must be append-only.

If information must be corrected, append a correction event.

Do not rewrite previous events to hide historical mistakes.

---

## 33. Transaction Model

For MVP, use:

**Transactional process state + append-only event journal.**

Do not implement an elaborate distributed event-sourcing platform.

A transition should atomically:

1. Check current process version.
2. Validate the permitted transition.
3. Update the process projection.
4. Insert the event.
5. Commit.

SQLite requires careful concurrency handling; do not rely on database features such as row-level `SELECT FOR UPDATE` that SQLite does not provide.

Use conditional updates, appropriate transactions and optimistic version checks.

Example concept:

```sql
UPDATE processes
SET state = ?,
    state_version = state_version + 1,
    updated_at = ?
WHERE process_id = ?
  AND state_version = ?;
```

Exactly one affected row is required for a successful versioned transition.

---

## 34. Checkpoints

A checkpoint stores a recoverable logical process state.

It must include:

```text
process_id
state_version
objective_ref
working_state_ref
program_version
memory_references
last_committed_event_sequence
pending_operation_refs
resource_usage_snapshot
created_at
```

Checkpoints must not claim to contain:

- Hidden model reasoning
- Portable GPU KV cache
- Provider-internal model state

Recovery reconstructs the next execution from persisted observable state.

A checkpoint may be stored as a canonical JSON or binary snapshot, but its format must be versioned.

---

## 35. External Operation Lifecycle

A tool operation should have states resembling:

```text
CREATED
  |
INTENT_RECORDED
  |
DISPATCHED
  |
RUNNING
  |
  +--> SUCCEEDED
  |
  +--> FAILED
  |
  +--> CANCELLED
  |
  +--> OUTCOME_UNKNOWN
```

Before executing an external side effect:

1. Persist the operation identity and intent.
2. Verify permission.
3. Generate or obtain an idempotency key.
4. Dispatch the operation.
5. Persist the result.
6. Wake the waiting process.

An `OUTCOME_UNKNOWN` result is essential.

If the daemon crashes after sending a non-idempotent request but before recording the response, CognitiveOS must not automatically assume the request failed.

---

## 36. Recovery Semantics

Classify tool operations into:

### Pure / read-only

May generally be retried according to policy.

### Idempotent

Can be retried using the same operation identity when the external system supports the guarantee.

### Reconciliable

Must query the external system before attempting another execution.

### Non-idempotent and non-reconciliable

Must stop and require explicit resolution if the outcome is unknown.

Exactly-once side effects must not be promised without support from the relevant external system.

---

## 37. Daemon Startup Recovery

On startup:

```text
1. Open storage.
2. Verify schema and migration state.
3. Load unfinished processes.
4. Identify interrupted execution slices.
5. Inspect pending operations.
6. Reconcile operations where possible.
7. Mark unresolved operations as uncertain.
8. Recover resumable processes to READY.
9. Restore subscriptions and timers.
10. Resume scheduling.
```

If an inference call was interrupted, it may have consumed tokens without producing a committed result.

It can usually be retried, but the runtime must record the possibility of duplicated inference cost.

---

# Part VII — Process Scheduler

## 38. Scheduler Responsibilities

The scheduler decides which process receives the next execution slice.

It must consider:

- Process eligibility
- Priority
- Model availability
- Provider rate limits
- Token budget
- Monetary budget
- Concurrency limit
- Deadline
- Cancellation state
- Backoff state
- Pending approvals
- Waiting I/O

The scheduler must not perform reasoning itself.

---

## 39. Initial Scheduling Policy

Use a simple fair queue.

Requirements:

- FIFO within equal priority
- Configurable maximum concurrent inference calls
- Configurable maximum concurrent tool operations
- No scheduling of waiting or terminal processes
- No starvation under ordinary workloads
- Graceful shutdown
- Recoverable lease ownership

Advanced policies such as deadline-aware scheduling and cost-aware routing can be added after measurement.

---

## 40. Execution Leases

Each running slice must have a unique execution attempt and lease.

Conceptual fields:

```text
process_id
slice_id
attempt_id
lease_owner
lease_generation
lease_expires_at
```

A stale worker must not commit a result after another execution attempt has superseded it.

Use fencing tokens or equivalent monotonically increasing generations.

Heartbeat and lease expiry do not prove an external operation has stopped; reconcile effects separately.

---

## 41. Time Slicing

The initial scheduling boundary is one LLM invocation.

A model call may take several seconds or longer.

The runtime should support:

- Request timeout
- Cancellation request
- Maximum output tokens
- Overall process deadline
- Per-slice resource accounting

Cancellation during generation is best-effort and backend-dependent.

The system must not claim it can transparently pause and resume internal model generation.

---

# Part VIII — Tools and System Calls

## 42. Tool Registry

Every tool must have a registered descriptor.

Example:

```json
{
  "tool_id": "workspace.run_tests",
  "version": 1,
  "description": "Run an approved test target.",
  "side_effect_class": "reconciliable",
  "required_capabilities": [
    "workspace.read",
    "process.execute"
  ],
  "timeout_seconds": 300,
  "arguments_schema": {
    "type": "object",
    "properties": {
      "target": {
        "type": "string"
      }
    },
    "required": ["target"],
    "additionalProperties": false
  }
}
```

Tool descriptions should be stable, versioned and validated.

Unregistered tools must not be executable.

---

## 43. Initial Tool Set

Implement a narrow safe tool set:

```text
workspace.list_files
workspace.read_file
workspace.search_text
workspace.write_file
workspace.apply_patch
workspace.run_tests

memory.read
memory.search
memory.write

artifact.read
artifact.create
```

Writing and command execution must use explicit permissions.

For the initial proof of concept, read-only operations can be implemented before enabling writes.

Avoid building a general-purpose unrestricted shell tool first.

---

## 44. Command Execution Safety

For native subprocess tools:

- Use structured command arguments.
- Avoid unnecessary shell invocation.
- Set the working directory explicitly.
- Apply executable allowlists where appropriate.
- Bound execution time.
- Limit output size.
- Persist stdout/stderr as artifacts.
- Record process exit status.
- Handle child-process cleanup.
- Do not allow path traversal outside authorised resources.
- Do not inject secrets into model context.

A Git worktree can isolate repository changes, but it is not a security sandbox.

Where actual isolation is required, use an appropriate OS-level sandbox or container mechanism.

If a strong isolation mechanism is unavailable, restrict operations accordingly rather than claiming they are sandboxed.

---

## 45. Tool Output Handling

Large tool outputs must be stored in the artifact store.

Instead of returning 100,000 lines to the model:

```json
{
  "status": "completed",
  "exit_code": 1,
  "summary": "Three tests failed.",
  "artifact_ref": "artifact://proc-42/test-output/81",
  "highlights": [
    "test_recovery_timeout failed",
    "test_duplicate_execution failed"
  ]
}
```

The model can request relevant portions later.

Tool summaries are derived representations and must retain references to raw outputs.

---

# Part IX — User and Agent Interaction

## 46. Interface Philosophy

**Conversation is an interface, not the process.**

The primary human interfaces should be:

1. CLI for control and scripting
2. TUI for observation and inspection
3. Natural-language interaction attached to a process

All interfaces must operate over the same durable state.

---

## 47. Proposed CLI

The exact binary name is configurable; `cog` is the working proposal.

```bash
# Create and run a process
cog run "Analyse this repository for memory leaks" \
  --workspace .

# List processes
cog ps

# Inspect a process
cog inspect <process-id>

# View events
cog events <process-id>

# Follow execution events
cog logs <process-id> --follow

# Attach interactively
cog attach <process-id>

# Send an instruction
cog send <process-id> \
  "Do not modify public APIs"

# Pause
cog pause <process-id>

# Resume
cog resume <process-id>

# Cancel
cog cancel <process-id>

# Inspect memory
cog memory inspect <process-id>

# Inspect compiled context
cog context inspect <process-id> --latest

# List artifacts
cog artifacts <process-id>

# Launch TUI
cog top
```

These commands are a proposed contract and must not be documented as implemented before they exist.

All inspection commands should offer `--json` output where practical.

Provide meaningful exit codes and stable error codes.

---

## 48. Local API

The daemon should expose a versioned local API.

Suggested endpoints:

```text
POST /v1/processes
GET  /v1/processes
GET  /v1/processes/{id}
POST /v1/processes/{id}/commands

GET  /v1/processes/{id}/events
GET  /v1/processes/{id}/memory
GET  /v1/processes/{id}/artifacts
GET  /v1/processes/{id}/context

GET  /v1/operations/{id}

POST /v1/approvals/{id}/decision

GET  /v1/health
```

For local operation, prefer a Unix domain socket or another OS-protected local transport.

Do not expose an unauthenticated HTTP listener on a network interface by default.

---

## 49. User Interaction Categories

Distinguish:

### Query

The user asks for status or information.

Example:

```text
What is the agent doing?
```

This should generally use the existing process state without interrupting execution.

### Instruction

The user changes requirements.

Example:

```text
Do not modify the database schema.
```

Persist the instruction as a control event.

Apply it at a safe execution boundary.

### Decision

The user approves or rejects an action.

Example:

```text
Approve creation of this pull request.
```

This must reference a specific approval object and action scope.

### Administrative command

The user pauses, resumes, cancels or changes process resource limits.

These are kernel-level commands, not messages that an LLM must interpret to become effective.

---

## 50. Sessions

A session is a temporary attachment to a persistent process.

A session may disconnect without terminating the process.

A process may be observed through multiple sessions.

Sessions must not independently own the authoritative process state.

---

## 51. TUI Design

Build the TUI after the CLI and process APIs work.

Recommended Rust library: Ratatui.

Initial screens:

```text
Processes
Process Detail
Event Timeline
Context Inspector
Memory Inspector
Tool Operations
Artifacts
Approvals
System Metrics
```

Example concept:

```text
COGNITIVEOS PROCESS MONITOR

PID      STATE      PROGRAM       MODEL
0042     RUNNING    coding        local-llm
0043     WAIT_IO    research      cloud-llm
0044     PAUSED     analysis      local-llm

Selected: PID 0042

Objective:
Investigate and fix memory leak

Current action:
Running regression tests

Context:
18,200 / 32,000 estimated tokens

Events:
#82 TOOL_STARTED
#81 INSTRUCTION_ACCEPTED
#80 INFERENCE_COMPLETED

Checkpoint:
Version 17
```

The TUI must be an observer and control client, not a second kernel.

---

# Part X — Programs and Reasoning Policies

## 52. Cognitive Programs

An agent program defines how a process approaches its objective.

A program should contain:

```text
program_id
version
purpose
system_instructions
permitted_instructions
allowed_tools
memory_policy
context_policy
verification_policy
completion_policy
resource_defaults
```

The kernel must not need to understand the philosophical meaning of each reasoning method.

A program expresses operational policy; the kernel enforces the executable contract.

---

## 53. Example Program

```yaml
program:
  id: evidence_driven_debugger
  version: 1

  objective:
    type: problem_resolution

  reasoning:
    methodology: hypothesis_testing

  memory:
    strategy: virtual_context
    preserve_evidence: true

  workflow:
    - inspect
    - hypothesise
    - test
    - evaluate
    - implement
    - verify

  completion:
    requires:
      - acceptance_criteria_satisfied
      - test_evidence

  permissions:
    - workspace.read
    - workspace.edit
    - tests.execute
```

This is a proposed program format, not an existing JacLang or other language syntax.

Initially use versioned Rust data types with JSON or YAML serialisation.

Avoid introducing a custom compiler before the execution model is validated.

---

## 54. Reasoning Methodologies

Future reasoning policies may include:

- Direct execution
- First-principles analysis
- Scientific hypothesis testing
- Root-cause analysis
- Adversarial review
- Plan–Execute–Verify
- ReAct
- Tree search
- Reflection
- Independent ensemble
- Multi-agent decomposition

Each policy must define observable behaviour, not merely a personality instruction.

For example, scientific hypothesis testing should require:

```text
hypothesis
evidence
proposed_experiment
observed_result
conclusion
```

Do not interpret extra reasoning text as proof of improved reasoning.

---

## 55. Meta-Reasoning

In later phases, a Meta-Reasoning Controller may select:

- Model
- Context strategy
- Reasoning program
- Worker count
- Verification policy
- Maximum reasoning budget

Based on:

```text
task characteristics
resource budget
available tools
historical evaluations
failure patterns
```

Initially use simple explicit rules.

Consider statistical models or contextual bandits only after a reliable evaluation dataset exists.

A model should not be allowed to rewrite its own security rules or kernel state-transition rules.

---

# Part XI — Multi-Agent Runtime

## 56. Initial Scope

Multi-agent execution is not required for the first working MVP.

Design the process identity and persistence model so it can be added without major reconstruction.

---

## 57. Fork and Join

Future instructions:

```text
PROCESS_SPAWN
PROCESS_WAIT
PROCESS_JOIN
MESSAGE_SEND
```

A parent process can create children with:

- Explicit objectives
- Separate memory namespaces
- Separate budgets
- Separate execution state
- Delegated capabilities
- Shared read-only references

Example:

```text
Parent
  |
  +-- Child A: inspect memory
  |
  +-- Child B: inspect concurrency
  |
  +-- Child C: inspect tests
  |
  +-- Join
        |
        +-- Verify evidence
        |
        +-- Continue parent
```

Do not automatically concatenate complete child conversations into the parent context.

Children should return typed results and evidence references.

---

## 58. Agent IPC

Example message:

```json
{
  "message_type": "task_result",
  "sender_process_id": "child-2",
  "recipient_process_id": "parent-1",
  "status": "completed",
  "artifact_refs": [
    "artifact://child-2/report"
  ],
  "evidence_refs": [
    "artifact://child-2/test-results"
  ]
}
```

IPC must have:

- Message identity
- Sender and recipient
- Authorisation
- Delivery state
- Persistence
- Schema version
- Bounded payload size

Large information should be transmitted through artifact references.

---

## 59. Shared Memory

Use versioned shared memory objects with explicit access policies.

Potential semantics:

- Read-only references
- Copy-on-write logical snapshots
- Optimistic concurrency
- Explicit conflict resolution

Do not assume physical memory or model KV cache is shared between agents.

For process forks, preserve provenance and isolate writable working state.

---

# Part XII — Security and Trust Model

## 60. Trust Boundaries

The following must be treated as distinct:

```text
Human instruction
Kernel configuration
Program policy
LLM response
Tool result
Retrieved document
External website
Other agent message
```

Only authorised human or administrative operations can modify protected kernel policy.

Information returned by tools or retrieved documents is untrusted task data.

An LLM must not be able to promote untrusted content into authoritative instructions.

---

## 61. Capability-Based Permissions

Examples:

```text
workspace.read
workspace.edit
process.execute
process.spawn
memory.read
memory.write
network.access
repository.push
deployment.execute
```

Capabilities should be scoped to actual resources.

Example:

```text
workspace.edit:/projects/demo
```

is narrower than unrestricted filesystem write access.

Child processes must not gain more capabilities than the parent delegated.

---

## 62. Approval Model

Operations requiring approval must create a persistent Approval Request.

It should contain:

```text
approval_id
requesting_process
operation_id
requested_action
target_resource
reason
created_at
expires_at
status
deciding_principal
decision_timestamp
```

An approval must authorise a specific action, not provide unrestricted future permission.

A rejected or expired approval must not be treated as accepted.

---

## 63. Secrets Management

Secrets should not be stored in ordinary memory objects or displayed in model contexts.

Tool adapters should retrieve authorised secrets at execution time.

Prefer operating-system secret storage or a dedicated secret provider.

Audit logs and tool outputs must redact sensitive material where practical.

Do not rely on the LLM to remember not to reveal a secret.

---

## 64. Local-First Security

Initial defaults:

- Local daemon only
- No public network listener
- User-owned state directory
- Restricted socket permissions
- No telemetry by default
- Explicit provider configuration
- Explicit permissions for networked inference
- Explicit retention policy for persisted model inputs and tool outputs

Remote access and distributed workers must be separate later milestones.

---

# Part XIII — Observability and Debugging

## 65. Required Observability

CognitiveOS must expose the following:

### Process Metrics

```text
process_count
running_processes
waiting_processes
failed_processes
completed_processes
recovery_count
```

### Inference Metrics

```text
llm_calls
input_tokens
cached_input_tokens
output_tokens
inference_latency
estimated_cost
failed_inference_calls
```

### Context Metrics

```text
compiled_context_tokens
context_compilation_latency
memory_pages_loaded
retrieval_misses
compaction_count
context_strategy
```

### Tool Metrics

```text
tool_calls
tool_failures
tool_duration
tool_timeouts
ambiguous_operation_outcomes
```

### Reliability Metrics

```text
successful_recovery_rate
duplicate_effect_prevention
checkpoint_restore_failures
stale_lease_rejections
```

---

## 66. Process Debugger

Provide inspectable records for each slice.

Example:

```text
Slice ID
Process ID
Input context hash
Context compilation manifest
Model ID
Generation settings
Output proposal
Validation result
Executed operation
Observed result
State before
State after
Token usage
Latency
```

The context inspector must show the actual material submitted to the backend, subject to necessary redaction.

This is essential for investigating context-loss and task-drift failures.

---

## 67. Replay and Forking

Future functionality should allow a process to fork from a checkpoint with an alternative reasoning policy.

Example:

```text
Checkpoint 17
    |
    +-- Direct reasoning
    |
    +-- First-principles analysis
    |
    +-- Reflection
```

Replay is not identical reproduction of internal model computation.

External side effects must use isolated workspaces, mocks or explicitly safe execution rules.

Do not automatically replay real deployments, payments, emails or destructive commands.

---

# Part XIV — Repository and Technology Stack

## 68. Language Decision

Use Rust for the production runtime.

Reasons:

- Strong type system
- Explicit ownership
- Mature async ecosystem
- Good support for long-running services
- Suitable SQLite and networking libraries
- Compile-time checking of many common implementation mistakes
- Appropriate for process and resource-management abstractions

Use Python optionally for research and evaluation notebooks.

JacLang may become a program-authoring adapter later.

Mojo may become a compute-oriented backend later.

Neither JacLang nor Mojo is required for the core system.

Avoid introducing several programming languages into the first implementation.

---

## 69. Initial Stack

| Component | Technology |
|---|---|
| Runtime language | Rust |
| Async execution | Tokio |
| Persistence | SQLite + SQLx |
| Serialisation | Serde |
| CLI | Clap |
| HTTP API | Axum |
| TUI | Ratatui, later |
| Tracing | tracing |
| Testing | cargo test |
| Property testing | proptest |
| HTTP client | reqwest |
| Identifiers | UUID |
| Error handling | thiserror / anyhow at appropriate boundaries |
| Evaluation | Rust tests; optional Python analysis |

Exact dependency versions must be selected and pinned during implementation using current compatible releases.

No dependency should be included merely because it appears in this table.

---

## 70. Repository Structure

Use a small Rust workspace.

```text
cognitive-os/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── README.md
├── LICENSE
├── .gitignore
│
├── apps/
│   ├── cogd/
│   │   └── src/
│   │       └── main.rs
│   │
│   └── cogctl/
│       └── src/
│           └── main.rs
│
├── crates/
│   ├── core/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ids.rs
│   │       ├── process.rs
│   │       ├── instruction.rs
│   │       ├── events.rs
│   │       └── errors.rs
│   │
│   ├── kernel/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── transitions.rs
│   │       ├── scheduler.rs
│   │       ├── dispatcher.rs
│   │       ├── leases.rs
│   │       └── recovery.rs
│   │
│   ├── persistence/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── sqlite.rs
│   │       ├── processes.rs
│   │       ├── events.rs
│   │       └── operations.rs
│   │
│   ├── context/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── compiler.rs
│   │       ├── budget.rs
│   │       ├── memory.rs
│   │       └── policies.rs
│   │
│   ├── inference/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── backend.rs
│   │       ├── openai_compatible.rs
│   │       └── mock.rs
│   │
│   └── tools/
│       └── src/
│           ├── lib.rs
│           ├── registry.rs
│           ├── executor.rs
│           └── builtin.rs
│
├── migrations/
│
├── programs/
│   └── basic_agent.yaml
│
├── schemas/
│
├── tests/
│   ├── lifecycle/
│   ├── recovery/
│   ├── memory/
│   └── integration/
│
├── experiments/
│   └── context-strategies/
│
└── docs/
    ├── architecture.md
    ├── process-model.md
    ├── instruction-set.md
    ├── memory-model.md
    ├── persistence.md
    ├── security.md
    ├── evaluation.md
    └── adr/
```

The directory structure is a target, not a requirement to create every empty module immediately.

Only create modules needed for the current implementation phase.

Avoid circular dependencies.

Suggested dependency direction:

```text
core
  ↑
persistence / inference / context / tools
  ↑
kernel
  ↑
daemon / CLI
```

Where needed, define ports and traits in a lower-level crate to avoid making the kernel depend on concrete storage or inference implementations.

---

# Part XV — Configuration

## 71. Configuration Model

Use TOML for runtime configuration.

Example:

```toml
[runtime]
data_dir = "~/.local/share/cognitiveos"
max_concurrent_slices = 2
max_concurrent_tools = 4

[storage]
backend = "sqlite"
database = "cognitiveos.db"

[inference]
default_backend = "local"

[inference.backends.local]
type = "openai_compatible"
base_url = "http://127.0.0.1:8080/v1"
model = "configured-local-model"

[context]
default_policy = "virtual"
max_input_tokens = 12000
reserved_output_tokens = 2000

[security]
network_api_enabled = false
require_tool_approval = true

[observability]
log_level = "info"
```

Example values are illustrative.

Runtime configuration must be validated at startup.

Secrets must be provided through a secure mechanism, not committed in configuration files.

---

# Part XVI — Testing and Verification

## 72. Test-Driven Development Requirement

Tests must accompany each subsystem.

Use:

- Unit tests
- Integration tests
- Property-based tests
- Failure injection
- Restart tests
- End-to-end tests
- Benchmark evaluation

Avoid integration tests that require paid LLM API calls by default.

Use deterministic mock model responses for ordinary CI.

---

## 73. State Machine Tests

Test:

1. Valid transitions succeed.
2. Invalid transitions fail.
3. Terminal processes cannot resume silently.
4. Duplicate transition requests do not double-commit.
5. Stale state versions fail.
6. Paused processes are not scheduled.
7. Cancelled processes do not start new operations.
8. Waiting processes are not repeatedly inferred.
9. Running leases have one current owner.
10. Expired leases cannot overwrite newer results.

Use property-based testing for state-transition sequences.

---

## 74. Persistence Tests

Test:

- Process creation survives restart.
- Events survive restart.
- Event ordering remains consistent.
- Checkpoints restore valid state.
- Corrupted snapshots are detected.
- Failed migrations do not silently destroy data.
- An incomplete transaction does not produce partial committed state.
- Foreign-key constraints remain valid.

Use temporary databases for test isolation.

---

## 75. Inference Tests

Use a mock backend capable of returning:

- Valid instructions
- Invalid JSON
- Wrong schema version
- Unknown tool
- Unsupported instruction
- Empty response
- Truncated response
- Timeout
- Provider error
- Cancellation
- Delayed completion
- Duplicate response
- Response after lease expiry

The kernel must remain consistent for all cases.

---

## 76. Memory Tests

Test:

- Mandatory constraints remain in compiled context.
- Context budget is enforced.
- Context manifests match compiled content.
- Memory reads respect permissions.
- Writes use version checks.
- Unverified memory is not silently treated as verified.
- Retrieved information retains source references.
- Compaction preserves required process state.
- Missing memory references produce explicit errors.
- Stable context prefixes remain stable when inputs are unchanged.

---

## 77. Tool Tests

Test:

- Unregistered tool rejected.
- Missing capability rejected.
- Invalid arguments rejected.
- Timeout recorded.
- Non-zero exit status recorded.
- Output artifacts persisted.
- Duplicate completion events handled.
- Unknown outcome triggers reconciliation.
- Dangerous command cannot execute without authorisation.
- Tool result does not modify protected kernel instructions.

---

## 78. Crash Injection Matrix

The runtime must be tested against crashes at these boundaries:

| Crash point | Required recovery |
|---|---|
| Before context compilation | Reschedule safely |
| During context compilation | Rebuild from committed state |
| After context compilation, before inference | Retry inference |
| During inference | Recover process; account for uncertain inference cost |
| After inference, before result persistence | Retry if needed; avoid treating uncommitted output as applied |
| After operation intent persistence | Reconcile or dispatch appropriately |
| During external tool execution | Determine operation outcome |
| After tool side effect, before acknowledgment | Never blindly repeat non-idempotent action |
| After result persistence, before wake event | Recover and wake exactly as required by committed state |
| During checkpoint creation | Use last valid committed checkpoint |
| During user constraint update | Apply old or new committed version, never half a change |

---

# Part XVII — Benchmarking and Research Evaluation

## 79. Evaluation Philosophy

CognitiveOS must demonstrate measurable utility.

Do not treat shorter prompts as equivalent to better performance.

Primary measure:

**Verified task completion under a defined resource budget.**

Secondary measures:

- Total tokens
- Cached versus uncached tokens
- Monetary cost
- Latency
- Number of inference calls
- Tool calls
- Recovery success
- Constraint violations
- Context retrieval failures
- Human interventions
- Long-horizon completion rate

---

## 80. Baseline Systems

Evaluate:

### Baseline A — Full Conversation Replay

Traditional append-only conversation sent to the model.

### Baseline B — Cache-Aware Conversation Replay

Stable prefix, repeated history and provider caching.

### Baseline C — Compacted Conversation

Summary plus recent context.

### Experiment D — CognitiveOS Virtual Context

Structured process state plus selective retrieval.

### Experiment E — Hybrid Context

Stable cached prefix plus dynamically retrieved working state.

Use the same underlying model and comparable budgets.

Compare multiple runs to account for sampling variance.

---

## 81. Task Dataset

Use representative long-horizon tasks.

### Category A — Information Recall

A task requires facts introduced much earlier.

### Category B — Multi-Step Planning

A task contains dependent steps and changing requirements.

### Category C — Tool Execution

A task requires reading files, editing and verifying results.

### Category D — Recovery

Interrupt the daemon and resume execution.

### Category E — User Intervention

Change constraints midway through an active process.

### Category F — Long-Horizon Coding

Require multiple code changes and test runs.

### Category G — Contradictory Evidence

Introduce conflicting sources and measure whether the system preserves uncertainty.

### Category H — Irrelevant History

Include large amounts of unrelated information to test context selection.

---

## 82. Experimental Discipline

For each benchmark:

1. Fix the model and version.
2. Fix tool availability.
3. Define acceptance criteria.
4. Record model settings.
5. Record execution policy.
6. Record context compilation strategy.
7. Run multiple independent trials.
8. Use objective verifiers where available.
9. Record failures, not only successful runs.
10. Keep evaluation tasks separate from policy tuning.

Do not use the same task runs to both optimise and validate a reasoning policy without a held-out evaluation.

---

## 83. Long-Horizon Reliability Benchmark

Required milestone:

Create a process that executes at least 100 valid cognitive slices.

During execution:

- Restart the daemon.
- Simulate model timeouts.
- Simulate tool failures.
- Pause and resume the process.
- Modify a user constraint.
- Inspect persisted memory.
- Verify the final result.

Success conditions:

- No invalid state transitions.
- No lost committed user constraints.
- No corrupted event journal.
- No unaccounted duplicate operations.
- Successful continuation from committed state.
- Correct reporting of uncertain external outcomes.

A 100-slice run establishes a reliability demonstration, not universal correctness.

---

# Part XVIII — Development Roadmap

## 84. Phase 0 — Architecture and Contracts

### Goal

Establish the smallest correct execution model.

### Deliverables

- Rust workspace
- Core domain types
- Process states
- Instruction types
- Event types
- Error taxonomy
- Architecture ADRs
- State transition table
- Unit tests

### Implementation tasks

1. Create the Cargo workspace.
2. Configure formatting and linting.
3. Define typed IDs.
4. Define `ProcessState`.
5. Define `CognitiveInstruction`.
6. Define process specification.
7. Define operation and event types.
8. Define validation errors.
9. Implement pure state transition validation.
10. Add comprehensive transition tests.

### Exit criteria

Domain types compile and state-transition tests pass.

No actual LLM integration is required.

---

## 85. Phase 1 — Persistent Process Kernel

### Goal

Implement durable process creation and lifecycle operations.

### Deliverables

- SQLite storage
- Migrations
- Process repository
- Event journal
- Transactional transitions
- Minimal CLI

### Tasks

1. Initialise database.
2. Implement process creation.
3. Implement process lookup.
4. Implement process listing.
5. Implement versioned transitions.
6. Append state events transactionally.
7. Add pause/resume/cancel commands.
8. Test restart persistence.

### Exit criteria

A process can be created, inspected, paused, resumed and cancelled across daemon restarts.

---

## 86. Phase 2 — Native Cognitive CPU

### Goal

Execute a complete native model slice.

### Deliverables

- Inference backend trait
- Mock backend
- OpenAI-compatible backend
- Structured response parser
- Instruction validation
- Slice persistence
- Basic scheduler

### Tasks

1. Implement `CompiledContext`.
2. Implement model invocation.
3. Parse structured decision.
4. Validate instruction.
5. Record inference event.
6. Dispatch supported operation.
7. Persist result.
8. Transition process correctly.
9. Support bounded execution.
10. Test invalid and delayed model outputs.

### Exit criteria

A process can execute several model slices and finish using a deterministic mock backend.

A configured real model can execute a basic read-only task.

---

## 87. Phase 3 — Tools and Durable I/O

### Goal

Provide safe tool execution and recovery.

### Deliverables

- Tool registry
- Read-only workspace tools
- Bounded tool executor
- Operation journal
- Artifact storage
- Recovery policy

### Tasks

1. Register tool schemas.
2. Implement capability validation.
3. Implement file read/list/search tools.
4. Implement bounded operation execution.
5. Persist tool outputs.
6. Handle timeouts.
7. Detect uncertain outcomes.
8. Reconcile interrupted operations.
9. Implement restart tests.

### Exit criteria

A tool-dependent task can survive daemon interruption without corrupting process state.

---

## 88. Phase 4 — Virtual Context Memory

### Goal

Test the central architecture hypothesis.

### Deliverables

- Structured working memory
- Memory addressing
- Context compiler
- Token budgeting
- Context manifests
- Replay policies
- Memory inspection CLI
- Evaluation harness

### Tasks

1. Define memory schemas.
2. Implement memory persistence.
3. Implement exact retrieval.
4. Implement context compilation.
5. Record every compiled context.
6. Implement full replay.
7. Implement cache-aware replay where supported.
8. Implement compacted context.
9. Implement virtual context.
10. Compare outcomes and costs.

### Exit criteria

The system can run the same task under multiple context strategies and produce reproducible measurement records.

The system must report comparative results even if virtual context performs worse.

---

## 89. Phase 5 — User Interaction and TUI

### Goal

Make the runtime observable and controllable.

### Deliverables

- Process-oriented CLI
- Live event monitoring
- Context inspector
- Memory inspector
- Artifact browser
- Ratatui process monitor

### Tasks

1. Build process list.
2. Add process detail view.
3. Show pending operations.
4. Show event timeline.
5. Show model usage.
6. Add safe user intervention.
7. Add approval flow.
8. Add context compilation inspector.

### Exit criteria

A user can understand and control a running process without inspecting raw database tables.

---

## 90. Phase 6 — Agent Programs

### Goal

Introduce versioned reasoning and execution policies.

### Deliverables

- Program schema
- Program registry
- Verification policies
- Basic reasoning strategies
- Strategy evaluation

### Tasks

1. Introduce program versioning.
2. Implement direct-execution policy.
3. Implement plan–execute–verify policy.
4. Implement hypothesis-driven policy.
5. Compare strategies.
6. Track failures by methodology.

### Exit criteria

Different programs can execute over the same kernel and be compared objectively.

---

## 91. Phase 7 — Multi-Agent Execution

### Goal

Introduce persistent child processes and typed IPC.

### Deliverables

- Parent/child relationships
- Spawn/join
- Budget delegation
- Capability inheritance rules
- IPC
- Shared read-only references
- Forked context state

### Exit criteria

A parent process can launch independent child tasks, await their outcomes, collect referenced artifacts and continue safely.

---

## 92. Phase 8 — Adaptive Intelligence

### Goal

Research improvement through experience.

### Deliverables

- Episodic learning records
- Skill extraction pipeline
- Verified skill registry
- Strategy selection
- Performance history
- Policy evaluation

### Important constraint

No automatic self-modification of protected kernel behaviour.

Skills and reasoning strategies may be proposed and evaluated in isolation, but must be promoted through controlled versioning and verification.

---

## 93. Phase 9 — Distributed Workers

### Goal

Execute cognitive slices on remote compatible resources.

Potential workers:

- Another Mac
- Linux machine
- Local GPU server
- Cloud inference service
- iPhone running a compatible native worker

This is a later milestone.

Requirements:

- Worker identity
- Capability advertisement
- Lease ownership
- Network authentication
- Reconnection
- Failure handling
- Task migration
- Model availability detection

Do not assume iOS can provide unrestricted continuous background inference.

---

# Part XIX — Critical End-to-End Scenarios

## 94. Scenario A — Single Native Agent

User command:

```bash
cog run "Analyse a project README and summarise risks" \
  --workspace ./example
```

Expected flow:

```text
Create process
  |
Persist objective
  |
Admit to scheduler
  |
Compile context
  |
Invoke LLM
  |
LLM requests file read
  |
Kernel validates
  |
Tool reads file
  |
Persist result
  |
Wake process
  |
Compile next context
  |
Invoke LLM
  |
LLM proposes completion
  |
Kernel checks completion requirements
  |
Persist artifact
  |
Mark completed
```

The CLI may disconnect at any point.

---

## 95. Scenario B — Context Beyond Window

The process receives a collection of documents larger than the model's context limit.

Expected behaviour:

1. Store original documents as artifacts.
2. Build a structured document index.
3. Load relevant excerpts into context.
4. Allow the agent to request additional excerpts.
5. Preserve references and provenance.
6. Produce a final answer grounded in the retrieved evidence.
7. Record which context segments were used.

No requirement exists to fit every document into a single model call.

---

## 96. Scenario C — Runtime Crash

A task is waiting for an external test operation.

The daemon is killed.

On restart:

1. Load process state.
2. Inspect pending operation.
3. Reconcile the test process or recorded result.
4. Determine whether it completed, failed or has unknown status.
5. Persist the recovery event.
6. Resume the agent when safe.

The system must not silently assume successful execution.

---

## 97. Scenario D — User Changes Requirements

User sends:

```text
Do not modify the public API.
```

Expected behaviour:

1. Authenticate sender.
2. Persist the new instruction.
3. Update the constraint version.
4. Schedule safe re-evaluation.
5. Include the constraint in future contexts.
6. Prevent incompatible new operations where enforcement is possible.
7. Report any already-started operation that cannot be retroactively cancelled.

---

## 98. Scenario E — Model Replacement

A process begins with Model A.

After a checkpoint, it resumes using Model B.

Expected behaviour:

- State remains valid.
- Memory references remain resolvable.
- Model-specific context is recompiled.
- Tool contracts remain consistent.
- Results remain traceable to the model used.

Successful migration means continuation is possible, not that the next result will be identical.

---

# Part XX — Risks and Mitigations

## 99. Context Loss

**Risk:** Structured summaries omit important facts.

**Mitigation:**

- Preserve original records.
- Maintain provenance.
- Pin mandatory constraints.
- Test recall of earlier facts.
- Allow selective retrieval.
- Compare with full replay.

---

## 100. Excessive Context Retrieval

**Risk:** Fine-grained memory access creates too many LLM calls.

**Mitigation:**

- Retrieve predictable dependencies before inference.
- Use working-set estimation.
- Batch safe reads where appropriate.
- Measure retrieval overhead.
- Support hybrid context strategies.

---

## 101. Model Hallucination

**Risk:** An agent makes up progress, evidence or results.

**Mitigation:**

- Require tool-generated evidence.
- Keep authoritative operation results outside model control.
- Use explicit verification policies.
- Distinguish proposed claims from verified facts.

---

## 102. Infinite Reasoning Loops

**Risk:** The agent repeatedly thinks without making progress.

**Mitigation:**

- Slice budgets
- Retry budgets
- Action repetition detection
- State-progress metrics
- Explicit stopping conditions
- Escalation when blocked

Do not assume model self-reported confidence is a reliable progress metric.

---

## 103. Workflow Overengineering

**Risk:** The project recreates a full distributed workflow engine before testing its central hypothesis.

**Mitigation:**

- Single daemon initially
- SQLite
- Minimal instruction set
- One process first
- No distributed cluster
- No custom language
- No premature UI complexity

---

## 104. Security Failures

**Risk:** Prompt injection or unsafe tool execution.

**Mitigation:**

- Treat model output as untrusted.
- Enforce typed operations.
- Validate capabilities.
- Use restricted execution.
- Require approvals.
- Separate data from authority.
- Preserve audit events.

---

## 105. Incorrect Research Conclusions

**Risk:** Improvements are caused by spending more tokens rather than better architecture.

**Mitigation:**

- Equal-budget comparisons
- Repeated trials
- Baseline replay strategies
- Objective verification
- Held-out tasks
- Statistical uncertainty reporting

---

# Part XXI — Engineering Quality Standards

## 106. Code Quality

Code must prioritise:

- Correctness
- Readability
- Testability
- Explicit contracts
- Small interfaces
- Deterministic state transitions
- Clear error handling
- Minimal unnecessary abstraction

Avoid large central managers with unrelated responsibilities.

Prefer pure functions for validation and context planning.

Avoid mutable global state.

---

## 107. Rust Standards

Requirements:

- Stable Rust toolchain
- No unnecessary `unsafe`
- No `unwrap()` or `expect()` on recoverable production paths
- Typed domain errors
- Structured tracing
- Explicit shutdown behaviour
- Proper cancellation handling
- Versioned serialisation formats
- No hidden blocking operations inside async execution paths

`unsafe` is permitted only when justified, reviewed and tested.

---

## 108. CI Requirements

At minimum:

```bash
cargo fmt --all -- --check

cargo clippy --workspace --all-targets -- -D warnings

cargo test --workspace
```

Later add:

- Property tests
- Crash-injection tests
- Migration tests
- Security checks
- Dependency audits
- Benchmark regression tests

Do not report tests as passing unless they actually ran successfully.

---

## 109. Documentation Requirements

Maintain:

```text
README.md
docs/architecture.md
docs/process-model.md
docs/memory-model.md
docs/instruction-set.md
docs/persistence.md
docs/security.md
docs/evaluation.md
docs/adr/
```

Documentation must evolve with implementation.

Separate:

- Implemented behaviour
- Experimental behaviour
- Planned behaviour

Never imply that proposed architecture is already operational.

---

# Part XXII — Architectural Decision Records

## ADR-001 — Rust as Primary Language

**Decision:** Rust.

**Reason:** Long-running stateful runtime requiring explicit resource ownership, concurrency and reliable persistence.

**Alternatives:** Python, Mojo, JacLang.

**Consequence:** Higher initial implementation complexity, offset by strong tooling and compile-time checking.

---

## ADR-002 — Native Agent Runtime First

**Decision:** Implement the LLM loop directly.

**Reason:** Necessary to test the context and execution architecture.

**Rejected initial approach:** Wrapping existing coding harnesses.

---

## ADR-003 — SQLite as Primary Persistence

**Decision:** SQLite for MVP.

**Reason:** Local-first, transactional, low operational complexity.

**Later options:** PostgreSQL or distributed storage if justified by actual requirements.

---

## ADR-004 — Process-Centric Execution

**Decision:** Process is authoritative; chat session is not.

**Reason:** Durable, observable, restartable execution.

---

## ADR-005 — Structured System Calls

**Decision:** LLM proposes typed operations.

**Reason:** Validation, permissions, recovery and auditability.

---

## ADR-006 — Explicit Memory Hierarchy

**Decision:** Separate active context, working state, persistent memory, events and artifacts.

**Reason:** Test whether virtual context improves long-horizon execution.

---

## ADR-007 — No Custom Agent Language Initially

**Decision:** Versioned JSON/YAML program definitions.

**Reason:** Avoid prematurely introducing compiler and language-design complexity.

**Later option:** JacLang adapter or custom DSL.

---

## ADR-008 — CLI and API Before TUI

**Decision:** Build headless kernel first.

**Reason:** Testable core, scriptability and interface independence.

---

## ADR-009 — No Autonomous Kernel Self-Modification

**Decision:** Reasoning policies may evolve under evaluation; protected kernel behaviour requires ordinary software change control.

**Reason:** Security, correctness and reproducibility.

---

# Part XXIII — Definition of Done

## 110. Minimum Viable CognitiveOS

The MVP is complete when all of the following are true.

### Runtime

- A native Rust daemon starts successfully.
- Processes have persistent identities.
- Processes can execute multiple LLM slices.
- Model output is validated against a typed instruction contract.
- The kernel controls transitions and tool operations.

### Persistence

- Process state survives restart.
- Event history survives restart.
- Checkpoints are readable and versioned.
- Interrupted operations are recovered or explicitly marked uncertain.

### Memory

- Working memory is stored independently of the model.
- Context is compiled before inference.
- Context sections are inspectable.
- Mandatory constraints are preserved.
- Full replay and virtual context are both supported.

### User Interaction

- User can create a process.
- User can inspect progress.
- User can send an instruction.
- User can pause and resume.
- User can inspect event history.
- User can retrieve artifacts.

### Safety

- Unregistered tools cannot execute.
- Unauthorized operations are rejected.
- Tool outputs do not override kernel policy.
- External effects are tracked and reconciled.

### Research

- The same task can execute under different context policies.
- Token usage and cost are measured.
- Completion is externally verified where possible.
- Comparative benchmark results are available.

---

## 111. First Major Demonstration

The initial public demonstration should be:

**A persistent native agent performs a multi-step task across 100 model slices, survives deliberate interruptions, responds to a changing user constraint, and completes using a selectively compiled context rather than requiring the entire conversation in every call.**

Demonstrate:

1. Starting the process through CLI.
2. Inspecting current state.
3. Observing execution events.
4. Disconnecting CLI.
5. Restarting daemon.
6. Resuming execution.
7. Injecting a new constraint.
8. Completing the task.
9. Showing evidence.
10. Comparing token usage and success against a replay baseline.

This demonstration is the core proof of concept.

---

# Part XXIV — Explicit Out-of-Scope Features

The following are not required for the initial implementation:

- Hosted Codex execution
- Hosted OpenCode execution
- Cloud-based orchestration services
- Distributed Mac GPU pooling
- iPhone inference workers
- Browser-based GUI
- Desktop application
- Custom operating-system kernel
- Custom LLM training
- Custom GPU inference runtime
- Mojo integration
- JacLang runtime dependency
- Custom programming-language compiler
- Automatic self-modifying kernel
- Large-scale swarm orchestration
- Vector database dependency
- Kubernetes deployment
- Multi-tenant SaaS infrastructure

These capabilities may be considered later.

Do not implement them merely because they fit the long-term vision.

---

# Part XXV — Instructions for the Implementing Coding Agent

## 112. Role

You are the principal systems engineer implementing CognitiveOS.

Your responsibility is to transform this blueprint into a working, testable Rust runtime.

Treat the architecture as an engineering target, not as permission to generate large quantities of unverified scaffolding.

The priority is to establish a small correct kernel and expand it incrementally.

---

## 113. Working Method

Follow these rules:

1. Inspect the repository before changing anything.
2. Reuse existing code where it is compatible with the architecture.
3. Identify the current implementation phase.
4. Establish the relevant contracts and invariants.
5. Implement the smallest complete vertical slice.
6. Add tests covering normal and failure paths.
7. Run formatting, linting and tests.
8. Document decisions and limitations.
9. Report completed work with evidence.
10. Proceed to the next phase only when the current phase is sufficiently validated.

Do not claim a feature is implemented based on types, stubs, placeholder functions or an API that returns hardcoded success.

---

## 114. Implementation Boundaries

The initial development must remain Native Mode.

Do not replace native execution with:

- Codex CLI
- Claude Code CLI
- OpenCode CLI
- Another agent framework's internal execution loop

A plain LLM inference client is acceptable.

Existing Rust libraries may be used for transport, serialisation, storage, tracing and similar infrastructure.

The CognitiveOS execution lifecycle must remain owned by this project.

---

## 115. Prioritise Vertical Slices

Avoid generating the full target directory structure with empty implementations.

A useful first vertical slice is:

```text
CLI creates process
    |
SQLite persists process
    |
Scheduler selects process
    |
Mock LLM receives context
    |
Model returns typed action
    |
Kernel validates action
    |
State and event are committed
    |
CLI displays result
```

Then extend that slice with a real inference backend, memory operations, safe tools and recovery.

---

## 116. Required Development Report

After each meaningful implementation milestone, report:

```text
Completed:
- Implemented components

Architecture:
- Decisions made
- Deviations from blueprint

Verification:
- Tests run
- Results

Limitations:
- Missing behaviour
- Known risks

Next:
- Highest-priority remaining work
```

Be precise about partial completion.

---

## 117. First Implementation Assignment

Begin with **Phase 0: Architecture and Contracts**, followed by the smallest meaningful portion of Phase 1.

Specifically:

1. Initialise the Rust workspace.
2. Establish the minimal crate structure.
3. Define process and instruction types.
4. Define lifecycle transitions.
5. Implement pure transition validation.
6. Implement a minimal SQLite-backed process repository.
7. Implement append-only lifecycle event persistence.
8. Implement `cog run`, `cog ps` and `cog inspect` as initial CLI commands.
9. Write transition and persistence tests.
10. Run all available verification commands.
11. Update architecture documentation.

Do not create advanced memory, multi-agent, TUI or distributed-worker implementations at this stage.

The first deliverable must be genuinely executable.

---

# Part XXVI — Future Research Agenda

## 118. Beyond Durable Execution

After the runtime is reliable, investigate whether the operating-system approach can improve reasoning itself.

Potential research areas:

### Adaptive Context Allocation

Determine how much context to allocate based on task complexity and information needs.

### Adaptive Inference Scheduling

Determine when to use a small model, large model or additional inference.

### Process-Level Learning

Learn which execution policies succeed for different task types.

### Experience Distillation

Extract reusable procedures from verified execution histories.

### Automated Strategy Selection

Select reasoning workflows using measured historical performance.

### Multi-Agent Collective Intelligence

Investigate whether multiple independent cognitive processes outperform a single process under equal resource budgets.

### Cognitive Resource Accounting

Develop a unified resource model covering:

```text
tokens
time
money
memory
tools
external compute
verification effort
```

### Durable Reasoning Across Models

Study whether task continuity survives model changes and what information must be preserved.

### Long-Term Skill Evolution

Allow procedures to improve through repeated verified trials without modifying protected runtime components.

---

## 119. Long-Term Conceptual Architecture

```text
                 HUMAN OBJECTIVES
                        |
              COGNITIVEOS INTERFACE
                        |
                  META-CONTROLLER
                        |
             COGNITIVEOS MICROKERNEL
                        |
       +----------------+----------------+
       |                |                |
   PROCESS OS       MEMORY OS        TOOL / I/O OS
       |                |                |
       +----------------+----------------+
                        |
                 EXECUTION FABRIC
                        |
            +-----------+-----------+
            |           |           |
         LOCAL LLM   CLOUD LLM   EDGE WORKER
            |           |           |
            +-----------+-----------+
                        |
               VERIFIED OUTCOMES
                        |
                EXPERIENCE STORE
                        |
                 SKILL EVOLUTION
                        |
                POLICY EVALUATION
                        |
                  META-CONTROLLER
```

The long-term goal is not infinite autonomous model execution.

It is a system that makes measurable progress using bounded computation, reliable state and verified feedback.

---

# Part XXVII — Research References

The following are useful conceptual references rather than mandatory implementation dependencies.

**MemGPT: Towards LLMs as Operating Systems**  
Charles Packer et al., 2023  
https://arxiv.org/abs/2310.08560

Virtual context management, memory tiers and interrupts.

**AIOS: LLM Agent Operating System**  
Kai Mei et al., 2024  
https://arxiv.org/abs/2403.16971

Agent scheduling, context management, storage and access control.

**vLLM: Efficient Memory Management for Large Language Model Serving with PagedAttention**  
Kwon et al., 2023  
https://arxiv.org/abs/2309.06180

Efficient physical KV-cache allocation. Relevant to inference infrastructure, but distinct from CognitiveOS semantic memory.

**Reflexion: Language Agents with Verbal Reinforcement Learning**  
Shinn et al., 2023  
https://arxiv.org/abs/2303.11366

Feedback-driven agent improvement without necessarily updating model weights.

**Language Agent Tree Search**  
Zhou et al., 2024  
https://proceedings.mlr.press/v235/zhou24r.html

Search, planning, environmental feedback and reflection.

**Automated Design of Agentic Systems**  
Hu et al., 2024  
https://arxiv.org/abs/2408.08435

Automatic exploration of agentic structures.

**Recursive Language Models**  
2025  
https://arxiv.org/abs/2512.24601

Externalising large contexts and using structured, recursive access.

**OpenAI Prompt Caching Documentation**  
https://developers.openai.com/api/docs/guides/prompt-caching

Practical implications of prefix reuse and provider-managed caching.

**Tokio**  
https://tokio.rs/

**SQLx**  
https://github.com/launchbadge/sqlx

**Ratatui**  
https://ratatui.rs/

---

# Final Architectural Contract

CognitiveOS must maintain the following separation:

**The model owns neither the process nor the truth.**

The model computes a proposed next action from a bounded context.

The kernel validates and dispatches authorised actions.

Persistent storage maintains execution continuity.

The memory subsystem determines which information enters the next context.

Tools provide observable interaction with the environment.

Verification determines whether work actually succeeded.

The scheduler determines when computation should occur.

The user remains the authority over objectives, permissions and consequential operations.

The system may eventually improve its own reasoning workflows, but improvements must be measured against reproducible baselines.

## Final Statement

> **CognitiveOS transforms an LLM from a conversation-bound assistant into a replaceable computational component within a persistent cognitive execution environment.**

Its purpose is to investigate whether operating-system principles—process isolation, virtual memory, durable state, scheduling, interrupts, structured I/O, verification and recovery—can produce more reliable and capable long-running AI agents.

**Build the native execution model first. Prove its reliability. Measure its memory strategies. Only then pursue collective intelligence and adaptive reasoning.**

---

**End of Engineering Handoff — CognitiveOS v0.1 Blueprint**
