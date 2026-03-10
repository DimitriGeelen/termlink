---
id: T-066
name: "Hub as persistent daemon — extract from CLI, supervision, pidfile"
description: >
  Hub is a CLI subcommand with no persistence. Inception: should it be a daemon with pidfile, graceful shutdown, session supervision, auto-recovery?

status: work-completed
workflow_type: inception
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-03-10T08:44:28Z
last_update: 2026-03-10T22:10:08Z
date_finished: 2026-03-10T22:10:08Z
---

# T-066: Hub as persistent daemon — extract from CLI, supervision, pidfile

## Problem Statement

Enhancement opportunity identified by reflection fleet enhance agent. Hub is a CLI subcommand with no persistence — single point of failure for multi-agent coordination. See [docs/reports/reflection-result-enhance.md].

## Assumptions

- A1: A persistent daemon model (pidfile, graceful shutdown) is more reliable than a CLI subcommand that dies when the terminal closes
- A2: Session supervision (heartbeat + auto-deregister for dead sessions) requires a persistent process
- A3: The hub can be extracted from `termlink-cli` into a standalone binary or long-running mode without protocol changes
- A4: launchd (macOS) / systemd (Linux) integration is feasible and valuable for auto-start

## Exploration Plan

1. **Spike 1 (1h):** Prototype pidfile management — write PID, check liveness, handle stale pidfiles
2. **Spike 2 (1h):** Test graceful shutdown — SIGTERM handler, drain active connections, deregister sessions
3. **Research (30m):** launchd plist vs. systemd unit file for auto-start. What do similar tools (Docker daemon, tmux server) do?
4. **Design (1h):** Draft daemon lifecycle: start, pidfile, health check, session supervision loop, shutdown

## Technical Constraints

- macOS uses launchd (plist), Linux uses systemd (unit files) — need both or neither
- Hub currently holds state in-memory (session registry, event stores) — daemon crash loses all state
- CLI `termlink hub` command must remain for manual/development use alongside daemon mode

## Scope Fence

**IN scope:** Daemon extraction, pidfile, graceful shutdown, session liveness supervision, auto-restart on crash.
**OUT of scope:** Hub clustering/federation (T-011), persistent event storage (WAL), web dashboard.

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions tested
- [x] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- Daemon extraction is clean (no protocol changes, no session API changes)
- Pidfile + SIGTERM shutdown works reliably on both macOS and Linux
- Session supervision adds real value (catches dead sessions within seconds, not minutes)

**NO-GO if:**
- Hub state is too complex for in-memory-only (requires persistence first, which is a different task)
- The complexity of daemon management exceeds the benefit for local-only deployment

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
-->

## Decisions

**Decision**: GO

**Rationale**: Hub is stateless routing service — daemon extraction is ~100 lines, no protocol changes. Pidfile+SIGTERM+supervision all validated. Addresses G-004.

**Date**: 2026-03-10T22:10:08Z
## Decision

**Decision**: GO

**Rationale**: Hub is stateless routing service — daemon extraction is ~100 lines, no protocol changes. Pidfile+SIGTERM+supervision all validated. Addresses G-004.

**Date**: 2026-03-10T22:10:08Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-10T22:07:18Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-10T22:09:51Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Hub is stateless routing service — daemon extraction is ~100 lines, no protocol changes. Pidfile+SIGTERM+supervision all validated. Addresses G-004.

### 2026-03-10T22:10:08Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Hub is stateless routing service — daemon extraction is ~100 lines, no protocol changes. Pidfile+SIGTERM+supervision all validated. Addresses G-004.

### 2026-03-10T22:10:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
