---
id: T-268
name: "CLI integration tests for emit-to, discover, spawn, register --self"
description: >
  CLI integration tests for emit-to, discover, spawn, register --self

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: [testing, cli]
components: []
related_tasks: []
created: 2026-03-24T21:38:02Z
last_update: 2026-03-24T21:38:02Z
date_finished: null
---

# T-268: CLI integration tests for emit-to, discover, spawn, register --self

## Context

Add CLI integration tests for features added in T-256, T-263 that lacked E2E coverage.

## Acceptance Criteria

### Agent
- [x] 3 discover tests: by role, by name, JSON output
- [x] 2 register --self tests: endpoint creation + event support
- [x] All 20 CLI integration tests pass (15 existing + 5 new)

## Verification

/Users/dimidev32/.cargo/bin/cargo test -p termlink --test cli_integration -- --test-threads=1 2>&1 | grep -q "20 passed"

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

### 2026-03-24T21:38:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-268-cli-integration-tests-for-emit-to-discov.md
- **Context:** Initial task creation
