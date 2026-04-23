---
id: T-967
name: "Persistent agent sessions — mark, protect from cleanup, verify on resume, enable cross-agent discovery"
description: >
  Inception: Persistent agent sessions — mark, protect from cleanup, verify on resume, enable cross-agent discovery

status: work-completed
workflow_type: inception
owner: human
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-04-12T09:15:06Z
last_update: 2026-04-23T19:30:30Z
date_finished: 2026-04-12T09:54:28Z
---

# T-967: Persistent agent sessions — mark, protect from cleanup, verify on resume, enable cross-agent discovery

## Problem Statement

Two operational needs conflict:

1. **Session cleanup cron** kills stale/orphaned termlink sessions to prevent resource leaks
2. **Persistent agent sessions** (framework agent, termlink agent, consumer project agents) must stay alive indefinitely so other agents can discover and contact them

Currently there's no way to distinguish a "stale orphan" from a "persistent agent session that's idle but listening." The cleanup cron kills both equally.

**For whom:** Any project that wants to be a persistent, discoverable participant in the agent network. Today that's the framework agent (.107) and termlink itself, but every consumer project benefits from having an always-reachable agent.

**Why now:** We just built cross-host parity (T-921), systemd-supervised hub (T-930..T-935), and push-based event delivery (T-690). The infrastructure for networked agent communication exists — but agents keep losing their persistent sessions to the cleanup cron, breaking the network.

## Assumptions

1. Persistent sessions can be marked via metadata (tag, KV, or registration flag) that cleanup cron checks before killing
2. The framework's session-start flow (/resume, fw context init) is the right place to verify persistent agent availability
3. Consumer projects want to maintain a discoverable agent session without manual intervention
4. The cleanup cron can distinguish "idle but listening" from "truly orphaned" without race conditions

## Exploration Plan

1. **Spike 1: Current cleanup cron analysis** (20min) — Read the cleanup cron script, understand what "stale" means, identify the kill criteria
2. **Spike 2: Marking mechanism** (30min) — Evaluate options: KV `persistent=true`, tag `persistent`, registration flag, PID file convention
3. **Spike 3: Session-start verification** (20min) — Design the check flow: on init/resume, verify the project's agent session exists, is registered with hub, responds to ping
4. **Spike 4: Auto-recovery** (20min) — What happens when the persistent session is dead? Auto-restart? Alert? Create new session?

## Technical Constraints

- Sessions are ephemeral by default (tied to a terminal/PTY process)
- Persistent sessions need a process that stays alive without a terminal (systemd unit or background process)
- Hub already supports named sessions and TCP connections
- Cleanup cron runs on crontab schedule, reads from session registry

## Scope Fence

**IN scope:**
- Marking sessions as persistent (protection from cleanup)
- Verifying persistent session health on framework start
- Design for auto-recovery on persistent session death
- Cross-project pattern (any consumer, not just termlink)

**OUT of scope:**
- Implementing the actual persistent agent process (separate build task)
- MCP tool exposure for persistent sessions (already covered by T-922)
- Multi-host persistent session coordination (future)

## Acceptance Criteria

### Agent
- [x] Problem statement validated
- [x] Assumptions tested
- [x] Recommendation written with rationale

### Human
- [x] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-967, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

  **Agent evidence (2026-04-15T19:52Z):** `fw inception status` reports decision
  **GO** recorded on 2026-04-12T10:26:25Z. Rationale: User approved: persistent agent sessions with KV persistent=true, tag role:receptionist, .framework.yaml config. Joint design with framework agent completed via PTY coordination....
  The inception decision is captured in the task's `## Decisions` section
  and in the Updates log. The Human AC "Record go/no-go decision" is
  literally satisfied — all that remains is ticking the box. Human may
  tick and close.

## Go/No-Go Criteria

**GO if:**
- A simple marking mechanism exists that cleanup cron can check (no complex protocol)
- Session-start verification can be done in <2s without blocking the agent
- At least 2 consumer projects would benefit from persistent sessions

**NO-GO if:**
- Persistent sessions require a fundamentally different session type (too much refactoring)
- The cleanup cron cannot safely distinguish persistent from orphaned (race conditions)
- The pattern is termlink-specific with no generalization path

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO

**Rationale:** Cross-agent coordination with framework agent (T-1135) confirms joint design. Both persistent sessions (`framework-agent`, `termlink-agent`) found dead with orphaned registrations — proving the problem. Implementation is trivial on termlink side (~15 lines for cleanup exemption, ~5 for `--persistent` flag). Framework handles protocol (config, init check), termlink handles mechanism (session management, cleanup).

**Evidence:**
- Both `framework-agent` and `termlink-agent` sessions dead, zero detection — proves cross-session blindness extends to persistent sessions
- `clean_stale_sessions()` in `manager.rs:313` has no tag/KV check — purely PID+socket based
- Framework agent agrees on: KV `persistent=true` for exemption, tag `role:receptionist` for discovery, `.framework.yaml` config for protocol
- Cost is minimal (~500KB per idle session, zero CPU/network when idle)
- Implementation split: termlink owns cleanup exemption + health command, framework owns init check + doctor report

**Joint Design (agreed with fw-agent T-1135):**
- KV: `persistent=true` (machine-readable, cleanup exemption)
- Tag: `role:receptionist` + `project:<name>` (discovery)
- `fw context init`: non-blocking health check, WARN if dead
- `fw doctor`: reports persistent session status
- Respawn: manual via `fw termlink respawn` (auto-respawn needs explicit opt-in)
- Config in `.framework.yaml` → `fw upgrade` propagates to consumers

## Decisions

**Decision**: GO

**Rationale**: User approved: persistent agent sessions with KV persistent=true, tag role:receptionist, .framework.yaml config. Joint design with framework agent completed via PTY coordination.

**Date**: 2026-04-12T10:26:25Z
## Decision

**Decision**: GO

**Rationale**: User approved: persistent agent sessions with KV persistent=true, tag role:receptionist, .framework.yaml config. Joint design with framework agent completed via PTY coordination.

**Date**: 2026-04-12T10:26:25Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T09:15:13Z — status-update [task-update-agent]
- **Change:** horizon: now → now

### 2026-04-12T09:20:50Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-12T09:54:28Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale: Cross-agent coordination with framework agent (T-1135) confirms joint design. Both persistent sessions (`framework-agent`, `termlink-agent`) found dead with orphaned registrations — proving the problem. Implementation is trivial on termlink side (~15 lines for cleanup exemption, ~5 for `--persistent` flag). Framework handles protocol (config, init check), termlink handles mechanism (session management, cleanup).

Evidence:
- Both `framework-agent` and `termlink-agent` sessions dead, zero detection — proves cross-session blindness extends to persistent sessions
- `clean_stale_sessions()` in `manager.rs:313` has no tag/KV check — purely PID+socket based
- Framework agent agrees on: KV `persistent=true` for exemption, tag `role:receptionist` for discovery, `.framework.yaml` config for protocol
- Cost is minimal (~500KB per idle session, zero CPU/network when idle)
- Implementation split: termlink owns cleanup exemption + health command, framework owns init check + doctor report

Joint Design (agreed with fw-agent T-1135):
- KV: `persistent=true` (machine-readable, cleanup exemption)
- Tag: `role:receptionist` + `project:<name>` (discovery)
- `fw context init`: non-blocking health check, WARN if dead
- `fw doctor`: reports persistent session status
- Respawn: manual via `fw termlink respawn` (auto-respawn needs explicit opt-in)
- Config in `.framework.yaml` → `fw upgrade` propagates to consumers

### 2026-04-12T09:54:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO

### 2026-04-12T10:26:25Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** User approved: persistent agent sessions with KV persistent=true, tag role:receptionist, .framework.yaml config. Joint design with framework agent completed via PTY coordination.

### 2026-04-16T05:39:44Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-16T21:04:50Z — programmatic-evidence [T-1090]
- **Evidence:** termlink discover shows 8 persistent sessions with tags and state=ready; cross-agent discovery working
- **Verified by:** automated command execution

### 2026-04-22T04:52:53Z — status-update [task-update-agent]
- **Change:** horizon: later → next

### 2026-04-22T10:20Z — human-ac-approved [T-1186 batch]
- **Action:** Human AC ticked by agent under user Tier 2 authorization (2026-04-22 batch-approve T-1186 (user Tier 2: 'batch approve them'))
- **Decision:** already recorded in Decision section prior to this approval
