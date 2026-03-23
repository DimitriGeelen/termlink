---
id: T-251
name: "Bypass eligibility — mutation flag to prevent read-write commands from promotion"
description: >
  Purely mechanical promotion based on success count cannot distinguish read-only from mutating commands. session.cleanup would be incorrectly promoted to Tier 3. Add mutating flag to orchestrator.route params or command denylist. See docs/reports/T-247-scenarios-framework-maintenance.md Scenario 3.

status: captured
workflow_type: build
owner: agent
horizon: now
tags: [T-247, T-238, orchestration, bypass]
components: []
related_tasks: [T-247, T-238, T-233]
created: 2026-03-23T16:54:24Z
last_update: 2026-03-23T16:54:24Z
date_finished: null
---

# T-251: Bypass eligibility — mutation flag to prevent read-write commands from promotion

## Context

Mutating commands like `session.cleanup` would be incorrectly promoted to bypass tier by the purely mechanical success-count promotion logic. See `docs/reports/T-247-scenarios-framework-maintenance.md` Scenario 3. Modified files: `crates/termlink-hub/src/bypass.rs`, `crates/termlink-hub/src/router.rs`.

## Acceptance Criteria

### Agent
- [ ] `orchestrator.route` accepts optional `mutating: true` param
- [ ] When `mutating=true`, bypass check is skipped and orchestrated runs are not tracked for promotion
- [ ] Test: mutating command not tracked in candidates after 10 successful runs
- [ ] Test: non-mutating command still promotes normally
- [ ] All hub tests pass

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

### 2026-03-23T16:54:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-251-bypass-eligibility--mutation-flag-to-pre.md
- **Context:** Initial task creation
