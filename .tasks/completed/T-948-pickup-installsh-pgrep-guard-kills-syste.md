---
id: T-948
name: "Pickup: install.sh pgrep guard kills systemd-managed hub on every idempotent
  run (from termlink)"
description: >
  Auto-created from pickup envelope. Source: termlink, task T-940. Type: bug-report.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [pickup, bug-report, systemd]
components: []
related_tasks: []
created: 2026-04-12T08:11:31Z
last_update: '2026-08-18T18:59:23Z'
date_finished: 2026-04-12T13:03:03Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:15Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 1
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=1 (body:fix-without-learning); D2=0 (no-signal); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:23Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 2
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=2 
      (no-signal)
    rubric_sha: missing
---

# T-948: Pickup: install.sh pgrep guard kills systemd-managed hub on every idempotent run (from termlink)

## Context

Bug: `install.sh` line 130 uses `pgrep -f "termlink hub start"` to find manually-launched hubs, but this also matches the systemd-managed hub process (same command line). Every idempotent install run kills the hub then systemd restarts it — causing a brief outage. Fix: add `! systemctl is-active --quiet termlink-hub.service` guard so pgrep-based kill only fires when the hub is NOT systemd-managed.

## Acceptance Criteria

### Agent
- [x] pgrep guard skips kill when `termlink-hub.service` is active (systemd-managed)
- [x] pgrep guard still kills manually-launched hub when systemd is not managing it

## Verification

# Shell commands that MUST pass before work-completed. One per line.
grep -q 'systemctl is-active.*termlink-hub' /opt/termlink/.context/systemd/install.sh

## Decisions

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T13:02:12Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now

### 2026-04-12T13:03:03Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
