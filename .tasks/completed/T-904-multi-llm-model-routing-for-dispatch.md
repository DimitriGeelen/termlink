---
id: T-904
name: "Multi-LLM model routing for dispatch"
description: >
  Multi-LLM model routing for dispatch

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-08T06:55:49Z
last_update: 2026-04-11T14:37:32Z
date_finished: 2026-04-11T14:37:32Z
---

# T-904: Multi-LLM model routing for dispatch

## Context

Extends T-903 (task-type routing) with multi-LLM model selection. Dispatch system gains optional model parameter (opus/sonnet/haiku), model fallback chain via circuit breaker, and per-model success rate tracking in the route cache.

## Acceptance Criteria

### Agent
- [x] DispatchParams (MCP) has optional `model` field
- [x] DispatchOpts (CLI) has optional `model` field and `--model` flag
- [x] Model is passed as `TERMLINK_MODEL` env var to spawned workers
- [x] Route cache tracks per-model success rates via ModelStats
- [x] Circuit breaker handles model-level unavailability with fallback chain (opus→sonnet→haiku)
- [x] DispatchRecord includes model field in dispatch manifest
- [x] Default behavior (no model specified) is unchanged — backward compatible
- [x] Tests pass: model dispatch, model fallback, model success tracking
- [x] `cargo test` passes for all modified crates
- [x] Summary report written to docs/reports/T-904-multi-llm-routing.md

## Verification

test -f docs/reports/T-904-multi-llm-routing.md
cd /opt/termlink && cargo test -p termlink-mcp --lib -- dispatch_params 2>&1 | tail -1
cd /opt/termlink && cargo test -p termlink-hub --lib -- model 2>&1 | tail -1

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

### 2026-04-08T06:55:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-904-multi-llm-model-routing-for-dispatch.md
- **Context:** Initial task creation

### 2026-04-11T14:37:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
