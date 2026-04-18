---
id: T-1130
name: "Register fabric card for test_env_lock.rs and pickup framework globstar bug"
description: >
  Register fabric card for test_env_lock.rs and pickup framework globstar bug

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-18T21:49:13Z
last_update: 2026-04-18T21:58:10Z
date_finished: 2026-04-18T21:58:10Z
---

# T-1130: Register fabric card for test_env_lock.rs and pickup framework globstar bug

## Context

`fw audit` reports "Fabric drift: 1 source file(s) have no fabric card" (trend 5×). `fw fabric drift` and `fw fabric scan` both report 0 unregistered and 0 created. Root cause: drift.sh + register.sh (scan) use bash `for file in $glob_pattern` without `shopt -s globstar`, so `crates/*/src/**/*.rs` only expands one level. audit.sh uses Python `glob.glob(..., recursive=True)` which works correctly. The file the audit sees (and scan/drift miss) is `crates/termlink-cli/src/test_env_lock.rs` (T-unknown, added for parallel-test HOME/CWD serialization).

Fix locally: register the one missing fabric card. Fix upstream: pickup P-NNN to framework with the bash globstar patch.

## Acceptance Criteria

### Agent
- [x] Root cause confirmed — bash glob without globstar expands `**` as one level (14 matches vs 68 with globstar)
- [x] Fabric card registered for `crates/termlink-cli/src/test_env_lock.rs` with real purpose/subsystem
- [x] `fw audit` no longer warns "Fabric drift: 1 source file(s) have no fabric card" (passes: "All watched source files registered (88 cards)")
- [x] Pickup envelope sent to framework describing the bug + patch — P-037 in /opt/999-Agentic-Engineering-Framework/.context/pickup/inbox/

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

test -f .fabric/components/crates-termlink-cli-src-test_env_lock.yaml
grep -q "purpose:" .fabric/components/crates-termlink-cli-src-test_env_lock.yaml
! grep -q 'purpose: "TODO' .fabric/components/crates-termlink-cli-src-test_env_lock.yaml
ls /opt/999-Agentic-Engineering-Framework/.context/pickup/inbox/P-*-bug-report.yaml 2>/dev/null | head -1 | xargs -I{} grep -l "globstar" {} >/dev/null

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

### 2026-04-18T21:49:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1130-register-fabric-card-for-testenvlockrs-a.md
- **Context:** Initial task creation

### 2026-04-18T21:58:10Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
