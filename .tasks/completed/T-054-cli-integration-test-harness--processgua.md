---
id: T-054
name: "CLI integration test harness — ProcessGuard, wait_for_socket, and end-to-end test scenarios"
description: >
  CLI integration test harness — ProcessGuard, wait_for_socket, and end-to-end test scenarios

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-09T08:36:00Z
last_update: 2026-03-09T08:50:23Z
date_finished: 2026-03-09T08:50:23Z
---

# T-054: CLI integration test harness — ProcessGuard, wait_for_socket, and end-to-end test scenarios

## Context

Automated end-to-end CLI integration testing to replace manual multi-terminal verification. See docs/reports/T-053-integration-test-harness.md for inception research.

## Acceptance Criteria

### Agent
- [x] ProcessGuard RAII type kills child process on drop (even on panic)
- [x] wait_for_socket utility polls for socket readiness with timeout
- [x] Isolated TERMLINK_RUNTIME_DIR per test prevents cross-test interference
- [x] Library-level integration tests: event emit/poll, topics, topic filter, KV CRUD, KV not-found (5 tests)
- [x] CLI binary integration tests: register/list, ping, status, exec, emit/events, topics, wait+emit, wait timeout, KV CRUD, KV JSON, info, clean, multi-session (13 tests)
- [x] All 156 tests pass (138 existing + 18 new)
- [x] Wait command off-by-one bug fixed (Option<u64> cursor for empty bus edge case)
- [x] Fabric card registered for cli_integration.rs

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink-session --test integration 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo test -p termlink --test cli_integration 2>&1 | tail -1
/Users/dimidev32/.cargo/bin/cargo test 2>&1 | grep -E "^test result:" | grep -v "0 passed" | head -6

## Decisions

### 2026-03-09 — Two-layer test architecture
- **Chose:** Library-level tests (extend integration.rs) + CLI binary tests (new cli_integration.rs with ProcessGuard)
- **Why:** Library tests are fast/reliable for RPC logic; binary tests verify actual CLI UX (flags, output, exit codes)
- **Rejected:** Pure binary-only (too slow, timing-fragile), mock-based (misses real IPC issues), assert_cmd-only (can't handle background processes)

## Updates

### 2026-03-09T08:36:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-054-cli-integration-test-harness--processgua.md
- **Context:** Initial task creation

### 2026-03-09T08:50:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
