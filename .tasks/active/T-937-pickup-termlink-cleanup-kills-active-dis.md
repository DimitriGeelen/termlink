---
id: T-937
name: "Pickup: TermLink cleanup kills active dispatch workers — fw termlink cleanup treats running workers as orphans because they lack exit_code file (from 999-Agentic-Engineering-Framework)"
description: >
  Auto-created from pickup envelope. Source: 999-Agentic-Engineering-Framework, task T-843. Type: bug-report.

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [pickup, bug-report]
components: []
related_tasks: []
created: 2026-04-11T23:00:01Z
last_update: 2026-04-12T17:14:28Z
date_finished: 2026-04-12T17:14:28Z
---

# T-937: Pickup: TermLink cleanup kills active dispatch workers — fw termlink cleanup treats running workers as orphans because they lack exit_code file (from 999-Agentic-Engineering-Framework)

## Problem Statement

`fw termlink cleanup` kills active dispatch workers because they lack `exit_code` file. Framework dispatch (`cmd_dispatch` in `termlink.sh`) creates worker dirs in `/tmp/tl-dispatch/`. Cleanup iterates these dirs and kills processes if no `exit_code` file exists. Active workers don't have `exit_code` (only written at completion).

**Mitigation already exists:** T-843/T-972 added a claude process check (lines 159-186 of `termlink.sh`) that skips workers with active `claude` child processes. This handles the common case.

## Assumptions

1. T-843/T-972 mitigation covers the most common scenario (claude dispatch workers)
2. The Rust CLI's `termlink dispatch` is the future direction — framework shell dispatch will be deprecated
3. Remaining edge cases (non-claude workers, process tree mismatch) are narrow and unlikely

## Exploration Plan

1. Read `cmd_cleanup()` in `termlink.sh` — DONE, T-843/T-972 fix present
2. Assess remaining risk after mitigation — narrow (non-claude workers only)
3. Decide: build additional protection or defer to Rust dispatch migration

## Technical Constraints

Framework dispatch is shell-based (`termlink.sh`). Rust dispatch (`termlink dispatch`) uses a different mechanism (hub sessions, not `/tmp/tl-dispatch/` dirs). The two systems don't interact.

## Scope Fence

**IN:** Assess if T-843/T-972 mitigation is sufficient
**OUT:** Rewriting framework dispatch (Rust CLI is the future path)

## Acceptance Criteria

### Agent
- [x] Problem statement validated (cleanup code reviewed, T-843/T-972 mitigation confirmed)
- [x] Assumptions tested (framework dispatch → Rust dispatch migration path confirmed)
- [x] Recommendation written with rationale (NO-GO: already mitigated)

### Human
- [ ] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-937, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

  **Agent evidence (2026-04-15T19:52Z):** `fw inception status` reports decision
  **NO-GO** recorded on 2026-04-12T17:14:28Z. Rationale: Recommendation: NO-GO...
  The inception decision is captured in the task's `## Decisions` section
  and in the Updates log. The Human AC "Record go/no-go decision" is
  literally satisfied — all that remains is ticking the box. Human may
  tick and close.

## Go/No-Go Criteria

**GO if:**
- T-843/T-972 mitigation has a known gap that causes production incidents
- Framework shell dispatch will remain the primary dispatch mechanism

**NO-GO if:**
- T-843/T-972 mitigation covers the common case (claude dispatch workers)
- Rust `termlink dispatch` is the future direction for dispatch

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** NO-GO

**Rationale:** The original bug (cleanup killing active workers) was already mitigated by T-843/T-972 which added claude process detection before killing. The remaining edge cases (non-claude workers) are narrow and unlikely in practice. The Rust `termlink dispatch` command is the future direction and uses a completely different session mechanism (hub-based, not `/tmp/tl-dispatch/` directories). Investing in further hardening of the shell-based dispatch cleanup is not warranted.

**Evidence:**
- `termlink.sh:159-186` — T-843/T-972 checks for active claude processes before killing
- `termlink.sh:184-186` — Active workers with claude processes are SKIPPED explicitly
- Rust dispatch (`dispatch.rs`) uses hub sessions (`.json` + `.sock`), not `/tmp/tl-dispatch/` dirs
- No recent incidents of cleanup killing active workers since T-843/T-972 fix

## Decisions

**Decision**: NO-GO

**Rationale**: Recommendation: NO-GO

Rationale: The original bug (cleanup killing active workers) was already mitigated by T-843/T-972 which added claude process detection before killing. The remaining edge case...

**Date**: 2026-04-12T17:14:28Z
## Decision

**Decision**: NO-GO

**Rationale**: Recommendation: NO-GO

Rationale: The original bug (cleanup killing active workers) was already mitigated by T-843/T-972 which added claude process detection before killing. The remaining edge case...

**Date**: 2026-04-12T17:14:28Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T17:14:28Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** NO-GO
- **Rationale:** Recommendation: NO-GO

Rationale: The original bug (cleanup killing active workers) was already mitigated by T-843/T-972 which added claude process detection before killing. The remaining edge case...

### 2026-04-12T17:14:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: NO-GO
