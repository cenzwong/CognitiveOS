# CognitiveOS Process Model & State Machine

## Process Lifecycle

A Process is the primary abstraction in CognitiveOS, representing an independently managed unit of cognitive work.

```text
                     +---------+
                     |   New   |
                     +----+----+
                          |
                          v
                     +----+----+ <---------------+
    +--------------> |  Ready  |                 |
    |                +----+----+                 |
    |                     |                      |
    |                     v                      |
    |                +----+----+                 |
    +---------------+ Running |                 |
    |                +----+----+                 |
    |                     |                      |
    |          +----------+----------+           |
    |          |          |          |           |
    |          v          v          v           |
    |      +-------+  +-------+  +-------+       |
    |      |WaitIo |  |WaitInp|  |WaitApp|       |
    |      +---+---+  +---+---+  +---+---+       |
    |          |          |          |           |
    +----------+----------+----------+           |
                          |                      |
                          v                      |
                     +----+----+                 |
                     | Paused  |-----------------+
                     +----+----+
                          |
        +-----------------+-----------------+
        |                 |                 |
        v                 v                 v
  +-----------+     +-----------+     +-----------+
  | Completed |     |  Failed   |     | Cancelled |
  +-----------+     +-----------+     +-----------+
    (Terminal)        (Terminal)        (Terminal)
```

## Invariants Enforced in Phase 0 & Phase 1

1. **Terminal State Immutability**: `Completed`, `Failed`, and `Cancelled` cannot transition to any other state.
2. **Cancellation Boundary**: Any non-terminal state may transition to `Cancelled`.
3. **Monotonic Versioning**: `state_version` increments by 1 on every committed transition. Transitions with mismatched versions fail atomically with `VersionConflict`.
4. **Event Atomicity**: Every state transition commits an entry to the append-only `process_events` journal within the same transaction.
