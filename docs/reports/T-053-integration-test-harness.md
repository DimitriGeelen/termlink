# T-053: End-to-End CLI Integration Test Harness

## Problem Statement

TermLink's CLI has 27 commands and 19 RPC methods. The existing test suite (138 tests) covers unit-level RPC handler logic, but there are **zero end-to-end tests** that exercise the CLI binary against real sessions. Human ACs across 5+ tasks require manual multi-terminal verification — a workflow that doesn't scale and can't be repeated reliably.

**Core challenge:** TermLink is inherently multi-process. A `register` command blocks as a listener, while other commands connect to it. Testing requires orchestrating concurrent processes with timing coordination.

## Goals

1. Automated integration tests runnable via `cargo test`
2. Test the actual CLI binary end-to-end (not just library internals)
3. Cover multi-process scenarios: register + emit/wait, register + kv operations
4. Reliable timing: wait for socket readiness, handle async coordination
5. Clean process cleanup on test success AND failure
6. Subsume all existing human ACs that say "verify across terminals"

## Technical Constraints

- Rust ecosystem (must integrate with `cargo test`)
- Unix sockets for IPC (macOS + Linux)
- Background processes that must be killed on cleanup
- Timing-sensitive flows (wait must start before emit)
- `TERMLINK_RUNTIME_DIR` env var for test isolation (existing pattern)
- Hub tests already use `ENV_LOCK` mutex for env var isolation

## Exploration Plan

### Spike 1: Rust Integration Test Frameworks
- `assert_cmd` + `assert_fs` crates (CLI testing standard)
- `std::process::Command` (manual approach)
- `tokio::process::Command` (async process management)
- `duct` crate (process orchestration)
- Trade-offs: simplicity vs async control vs cleanup guarantees

### Spike 2: Process Lifecycle Management
- How to start `termlink register` as background process
- How to detect socket readiness (poll for file existence vs. connect attempt)
- How to guarantee cleanup (Drop guards, panic hooks, signal handlers)
- Tempdir isolation for runtime directory

### Spike 3: Existing Test Patterns
- What patterns do the 138 existing tests use?
- How does `ENV_LOCK` work and should integration tests use the same pattern?
- Can we reuse `SessionContext` directly instead of spawning CLI processes?
- Library-level vs binary-level testing trade-offs

### Spike 4: Test Scenarios
- Map all human ACs to integration test cases
- Identify ordering/timing requirements per scenario
- Group tests by feature area (events, kv, attach, stream)

### Spike 5: CI/CD Considerations
- Parallel test execution safety
- Timeout handling for stuck processes
- Platform differences (macOS vs Linux socket paths)

## Go/No-Go Criteria

**GO if:**
- At least one framework/approach can reliably orchestrate background CLI processes
- Socket readiness detection is solvable without flaky sleeps
- Cleanup guarantees exist even on test panic
- Can run in parallel with existing test suite without interference

**NO-GO if:**
- All approaches require unreliable timing hacks
- Process cleanup is fundamentally unreliable in the test harness
- The complexity cost exceeds the manual testing cost for the foreseeable future

## Research Findings

<!-- Updated incrementally as investigation proceeds -->

## Dialogue Log

<!-- Record questions, answers, course corrections -->

## Decision

<!-- Filled after go/no-go -->
