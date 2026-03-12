---
id: T-119
name: "Agent mesh task gate bypass — workers blocked by check-active-task.sh"
description: >
  Agent mesh workers (claude --print via agent-wrapper.sh) are blocked by the
  PreToolUse task gate (check-active-task.sh) because no task is focused in
  their session. Workers need an ungated write path or task-aware dispatch.
status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: [agent-mesh, enforcement, bug]
components: []
related_tasks: [T-114, T-116]
created: 2026-03-12T19:00:39Z
last_update: 2026-03-12T19:00:39Z
date_finished: null
---

# T-119: Agent mesh task gate bypass — workers blocked by check-active-task.sh

## Problem Statement

Agent mesh workers run `claude --print` via `agent-wrapper.sh`. The framework's
PreToolUse hook (`check-active-task.sh`) blocks Write/Edit unless a task is focused
in `.context/working/focus.yaml`. Workers have no task context — they're ephemeral
`--print` sessions. Result: 4/4 mesh workers were blocked from writing exploration
reports. 2/4 fell back to inline output; 2/4 returned nothing useful.

**Evidence:** 2026-03-12 dispatch of explore-T009, T010, T071, T073.

## Options

1. **Exempt `docs/reports/` from task gate** — add path whitelist to `check-active-task.sh`
2. **Dispatch sets focus before spawning** — `dispatch.sh` runs `fw context focus T-XXX` in worker env
3. **Workers write to ungated path** — `/tmp/` or `.context/bus/`, orchestrator copies in
4. **Tag-based gate bypass** — workers with `agent-mesh` tag get automatic exemption
5. **Prompt workaround** — instruct agents to return inline if write blocked (works but fragile)

## Technical Constraints

- `check-active-task.sh` is a framework hook — changes need framework PR
- Workers run in clean env (`unset CLAUDECODE`) — no inherited session state
- Workers are `--no-session-persistence` — no `.claude/` state directory
- `focus.yaml` is per-project, shared across sessions — concurrent workers would conflict

## Scope Fence

**IN:** Decide which bypass mechanism to use for mesh workers
**OUT:** Implementation (separate build task)

## Acceptance Criteria

### Agent
- [x] Problem statement validated (4/4 workers blocked)
- [x] Options enumerated with tradeoffs
- [ ] Go/No-Go decision made

### Human
- [ ] Approach reviewed and direction decided

## Go/No-Go Criteria

**GO if:**
- A clean bypass exists that doesn't weaken the task gate for non-mesh usage
- Implementation is bounded (< 1 session)

**NO-GO if:**
- All options create security/governance holes
- Workaround (option 5) is sufficient for current usage

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
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
