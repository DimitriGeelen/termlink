---
id: T-237
name: "Hub orchestrator.route RPC — discover, delegate, relay in one call"
description: >
  Add orchestrator.route RPC method to TermLink hub. Combines session.discover + delegate + relay into a single call. Agent sends capability slug, hub finds matching specialist, forwards request, relays response. ~100 LOC Rust on existing hub primitives. See T-233 research: Q2b-termlink-mapping.md

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [T-233, orchestration, hub]
components: []
related_tasks: [T-233]
created: 2026-03-23T13:27:16Z
last_update: 2026-03-23T22:00:20Z
date_finished: 2026-03-23T16:21:04Z
---

# T-237: Hub orchestrator.route RPC — discover, delegate, relay in one call

## Context

Hub RPC method per T-233 research (Q2b-termlink-mapping). See docs/reports/T-233-specialist-agent-orchestration.md.

## Acceptance Criteria

### Agent
- [x] ORCHESTRATOR_ROUTE constant in termlink-protocol control.rs
- [x] handle_orchestrator_route handler in hub router.rs
- [x] Discovers sessions by selector (tags/roles/capabilities/name), local + remote
- [x] Forwards method+params to first matching candidate with failover
- [x] Returns routed_to metadata + specialist response
- [x] 3 tests: success routing, no-match error, missing method error
- [x] All 49 hub tests pass


## Verification

/Users/dimidev32/.cargo/bin/cargo test --package termlink-hub
<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     Examples:
       python3 -c "import yaml; yaml.safe_load(open('path/to/file.yaml'))"
       curl -sf http://localhost:3000/page
       grep -q "expected_string" output_file.txt
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

## Updates

### 2026-03-23T13:27:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-237-hub-orchestratorroute-rpc--discover-dele.md
- **Context:** Initial task creation

### 2026-03-23T16:14:26Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-23T16:21:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
