---
id: T-967
name: "Persistent agent sessions — mark, protect from cleanup, verify on resume, enable cross-agent discovery"
description: >
  Inception: Persistent agent sessions — mark, protect from cleanup, verify on resume, enable cross-agent discovery

status: captured
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T09:15:06Z
last_update: 2026-04-12T09:15:13Z
date_finished: null
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
- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Recommendation written with rationale

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

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

<!-- REQUIRED before fw inception decide. Write your recommendation here (T-974).
     Watchtower reads this section — if it's empty, the human sees nothing.
     Format:
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence from exploration)
     **Evidence:**
     - Finding 1
     - Finding 2
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T09:15:13Z — status-update [task-update-agent]
- **Change:** horizon: now → now
