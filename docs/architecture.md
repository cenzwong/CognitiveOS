# CognitiveOS Architecture Specification

## Overview

CognitiveOS is an operating environment for persistent, long-running LLM agents.
It replaces conversation-centric loops with a process-centric operating runtime.

## Workspace Layout

```text
cognitive-os/
├── apps/
│   └── cogctl/             # CLI binary: cog
├── crates/
│   ├── core/               # Domain contracts, IDs, states, events, instructions
│   ├── persistence/        # SQLite migrations, repository, optimistic locking
│   └── kernel/             # Application services, state transitions, lifecycle
├── migrations/             # SQL migration files
└── tests/                  # Integration and verification tests
```

## Layering Contract

```text
       cog CLI
          |
          v
 cognitive-kernel (KernelService)
          |
          v
cognitive-persistence (ProcessRepository / SQLite)
          |
          v
    cognitive-core (Types, Invariants, Validations)
```

1. **CLI Layer (`cogctl`)**: Parses user arguments, coordinates output (standard text or JSON), and delegates all business logic to `KernelService`.
2. **Kernel Layer (`cognitive-kernel`)**: Coordinates lifecycle transitions, validates operations against domain invariants, and ensures state changes and events are dispatched together.
3. **Persistence Layer (`cognitive-persistence`)**: Manages the SQLite database connection pool, applies migrations, executes atomic transactions, and enforces optimistic locking.
4. **Core Layer (`cognitive-core`)**: Pure domain model without external runtime dependencies (except serde/uuid/chrono).
