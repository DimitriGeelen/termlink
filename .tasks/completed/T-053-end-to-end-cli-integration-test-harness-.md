---
id: T-053
name: "End-to-end CLI integration test harness — automated multi-process terminal testing"
description: >
  Inception: End-to-end CLI integration test harness — automated multi-process terminal testing

status: work-completed
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-09T06:34:01Z
last_update: 2026-03-09T08:35:50Z
date_finished: 2026-03-09T08:35:50Z
---

# T-053: End-to-end CLI integration test harness — automated multi-process terminal testing

## Problem Statement

TermLink CLI has 27 commands and 19 RPC methods. 138 unit tests cover handler logic, but zero tests exercise the CLI binary end-to-end. 5 tasks have human ACs requiring manual multi-terminal verification — inefficient, unrepeatable, and blocks development velocity.

## Assumptions

- A1: Existing integration test pattern (Session::register_in + run_accept_loop) can be extended for new scenarios — VALIDATED (12 tests already work this way)
- A2: CLI binary can be tested via tokio::process::Command with ProcessGuard cleanup — VALIDATED (pattern well-established in Rust ecosystem)
- A3: TERMLINK_RUNTIME_DIR + ENV_LOCK provides sufficient test isolation — VALIDATED (existing hub tests use this)

## Exploration Plan

- Spike 1: Rust CLI test frameworks (assert_cmd, tokio::process, duct, expectrl) — DONE
- Spike 2-3: Existing test patterns and session lifecycle — DONE
- Spike 4: Map human ACs to test scenarios — DONE

## Technical Constraints

- Unix sockets for IPC (macOS + Linux)
- TERMLINK_RUNTIME_DIR for test isolation
- ENV_LOCK mutex for parallel test safety
- tokio runtime required for async tests

## Scope Fence

**IN:** Library-level integration tests, CLI binary integration tests, ProcessGuard helper, wait_for_socket helper, test scenarios for T-027/T-031/T-038/T-046/T-052 human ACs
**OUT:** CI/CD pipeline setup, performance benchmarks, fuzz testing

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions tested
- [x] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- At least one approach reliably orchestrates background CLI processes — YES (tokio::process + ProcessGuard)
- Socket readiness detection solvable without flaky sleeps — YES (poll for socket file existence)
- Cleanup guarantees on panic — YES (RAII Drop guard)

**NO-GO if:**
- All approaches require unreliable timing hacks — NOT THE CASE

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
-->

## Decisions

### 2026-03-09 — Architecture: Two-layer integration test approach
- **Chose:** Layer 1 (library-level, extend integration.rs) + Layer 2 (CLI binary, new cli_integration.rs with ProcessGuard)
- **Why:** Library tests are fast/reliable for RPC logic; binary tests verify actual CLI UX (flags, output format, exit codes)
- **Rejected:** Pure binary-only testing (too slow, timing-fragile), mock-based testing (doesn't catch real IPC issues)

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-09T08:34:40Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Library-level integration tests extend existing pattern (12 tests in integration.rs). CLI binary tests use ProcessGuard + wait_for_socket + TERMLINK_RUNTIME_DIR isolation. Clear architecture, strong existing foundation.

### 2026-03-09T08:34:56Z — status-update [task-update-agent]
- **Change:** owner: human → agent

### 2026-03-09T08:35:01Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Library-level integration tests extend existing pattern (12 tests in integration.rs). CLI binary tests use ProcessGuard + wait_for_socket + TERMLINK_RUNTIME_DIR isolation. Clear architecture, strong existing foundation.

### 2026-03-09T08:35:50Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
