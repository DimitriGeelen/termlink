---
id: T-1423
name: "fleet-deploy-binary.sh: add --probe pre-swap dry-run (PL-100 mitigation)"
description: >
  PL-100 incident on .122 (T-1422) showed the script will happily kill+swap a binary that the target host can't actually execute (glibc/lib mismatch between build host and target). Add --probe: before kill, run NEW_BIN --version with a short timeout on the remote; abort if non-zero. Cheap pre-flight that would have caught this with zero downtime. Optional follow-on: --rollback-script flag that pre-stages a rollback shell on the remote so recovery doesn't depend on termlink remote-exec being available.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-30T20:05:11Z
last_update: 2026-04-30T20:06:55Z
date_finished: null
---

# T-1423: fleet-deploy-binary.sh: add --probe pre-swap dry-run (PL-100 mitigation)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `scripts/fleet-deploy-binary.sh --help` documents `--probe`
- [x] When `--probe` is set (and `--swap-restart` is on), the script runs `<NEW> --version` on the remote AFTER assembly and BEFORE kill
- [x] On non-zero exit from the probe, the script aborts with exit 5 and leaves the staged binary in place (no kill, no swap)
- [x] The probe runs even without `--swap-restart` (bare staging) so a one-shot deploy can verify exec-ability
- [x] Probe output (first 5 lines of stderr) is shown when probe fails
- [x] `bash -n scripts/fleet-deploy-binary.sh` passes

## Verification

bash -n scripts/fleet-deploy-binary.sh
grep -q -- "--probe" scripts/fleet-deploy-binary.sh
grep -q "exit 5" scripts/fleet-deploy-binary.sh
test -x scripts/fleet-deploy-binary.sh

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

### 2026-06-01T — closure-ready: --probe shipped, validated live in T-1903 [agent autonomous]

All 6 Agent ACs ticked, no Human ACs defined. `--probe` flag is in production use:

- T-1903 deployed 0.11.473 to 3 remote hubs (laptop-141, ring20-management, ring20-dashboard) using `scripts/fleet-deploy-binary.sh --probe --swap-restart`. Probe passed on each before swap; zero rollbacks.
- Smaller follow-up T-1438 staged musl 0.9.1657 to .122 (operator-ready, not swapped) using `--probe` standalone — confirmed exec-ability without committing to swap.
- `bash -n scripts/fleet-deploy-binary.sh` clean (this session).

The PL-100 mitigation contract is honored: probe runs after assembly, before kill, abort with exit 5 + first-5-lines-of-stderr surfacing on non-zero. T-1903 commit 8698f25e is the canonical proof of utility.

**Operator-actionable:** ready for `fw task update T-1423 --status work-completed`.

### 2026-04-30T20:05:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1423-fleet-deploy-binarysh-add---probe-pre-sw.md
- **Context:** Initial task creation

### 2026-04-30T20:05:18Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
