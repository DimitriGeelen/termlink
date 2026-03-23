---
id: T-238
name: "Bypass registry — Tier 3 operationalized for local execution"
description: >
  YAML registry of commands that have earned autonomous execution rights (Tier 3). Commands promoted via track record (5+ runs, 0 failures). Agents cannot self-promote. Failed bypass de-promotes. See T-233 research: Q2b-bypass-mechanism.md

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-233, orchestration, bypass]
components: []
related_tasks: [T-233]
created: 2026-03-23T13:27:24Z
last_update: 2026-03-23T16:41:58Z
date_finished: 2026-03-23T16:41:58Z
---

# T-238: Bypass registry — Tier 3 operationalized for local execution

## Context

Bypass registry per T-233 research (Q2b-bypass-mechanism). Commands that have earned autonomous execution rights (Tier 3) skip orchestration. See docs/reports/T-233-specialist-agent-orchestration.md.

## Acceptance Criteria

### Agent
- [x] `bypass.rs` module in termlink-hub with BypassRegistry struct
- [x] JSON registry file at `{runtime_dir}/bypass-registry.json` with entries: command, tier, run_count, fail_count, promoted_at, last_run
- [x] `load()` / `save()` / `check(command)` / `record_orchestrated_run(command, success)` / `record_bypass_run(command, success)` API
- [x] Promotion: command auto-added to registry after 5+ successful orchestrated runs with 0 failures
- [x] De-promotion: entry removed from registry when a bypass execution fails
- [x] `orchestrator.route` checks bypass registry before routing — returns `{ bypassed: true, command, tier, run_count }` for hits
- [x] `orchestrator.bypass_status` RPC to query registry contents (bypassed commands + promotion candidates)
- [x] 5 tests: load/save round-trip, check hit/miss, promotion threshold, promotion blocked by failure, de-promotion on failure
- [x] All 54 hub tests pass

## Verification

/Users/dimidev32/.cargo/bin/cargo test --package termlink-hub

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-03-23T13:27:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-238-bypass-registry--tier-3-operationalized-.md
- **Context:** Initial task creation

### 2026-03-23T16:36:42Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-23T16:41:58Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
