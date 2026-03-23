---
id: T-252
name: "Distinguish infrastructure vs command failure in bypass tracking"
description: >
  Dead specialist (infra failure) should not count against a command's promotion stats. Currently record_orchestrated_run takes a boolean — needs a third state or caller decides. See docs/reports/T-247-scenarios-code-review.md Scenario 1, T-247-scenarios-infrastructure.md Scenario 2.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [T-247, T-238, orchestration, bypass]
components: []
related_tasks: [T-247, T-238, T-233, T-250]
created: 2026-03-23T16:54:27Z
last_update: 2026-03-23T21:00:36Z
date_finished: 2026-03-23T21:00:36Z
---

# T-252: Distinguish infrastructure vs command failure in bypass tracking

## Context

Dead specialist (infra failure) should not count against a command's promotion stats, but currently `record_orchestrated_run` takes a boolean with no way to distinguish infrastructure failures from command failures. Depends on T-250 (transport failure tracking). See `docs/reports/T-247-scenarios-code-review.md` Scenario 1, `T-247-scenarios-infrastructure.md` Scenario 2. Modified files: `crates/termlink-hub/src/bypass.rs`, `crates/termlink-hub/src/router.rs`.

## Acceptance Criteria

### Agent
- [x] `record_orchestrated_run` accepts a third variant: `infra_failure` (does not count against promotion)
- [x] Router uses `infra_failure` for connection errors and timeouts, `command_failure` for RPC errors
- [x] Test: 4 infra failures + 5 successes still promotes (infra failures are invisible to promotion)
- [x] Test: 1 command failure + 5 successes does NOT promote (`fail_count > 0`)
- [x] All hub tests pass

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

### 2026-03-23T16:54:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-252-distinguish-infrastructure-vs-command-fa.md
- **Context:** Initial task creation

### 2026-03-23T20:55:17Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-23T21:00:36Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
