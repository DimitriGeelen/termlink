---
id: T-913
name: "fw doctor check — warn if .agentic-framework is a symlink"
description: >
  Follow-up from T-909. fw doctor should detect when a consumer project has .agentic-framework
  as a symlink (vs a real vendored directory) and emit a WARN with pointer to fw vendor.
  This would have caught G-001 immediately instead of letting it sit undetected for
  weeks. Also consider: fw upgrade pre-flight should refuse to upgrade a consumer
  project that still has a symlink (suggest vendoring first).

status: work-completed
workflow_type: build
owner: human
horizon:
tags: [infrastructure, doctor, symlink]
components: []
related_tasks: []
created: 2026-04-11T12:28:45Z
last_update: '2026-08-18T18:59:23Z'
date_finished: 2026-04-12T20:35:30Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:13Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 4
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=4 (body:fw-audit-or-doctor); D3=0
      (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:23Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-913: fw doctor check — warn if .agentic-framework is a symlink

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `fw doctor` emits WARN when `.agentic-framework` is a symlink (not a real directory)
- [x] WARN message includes pointer to `fw vendor` as the fix
- [x] `fw doctor` shows OK when `.agentic-framework` is a real directory (not symlink)
- [x] Check appears early in doctor output (after framework installation check)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
grep -q 'symlink' /opt/termlink/.agentic-framework/bin/fw
fw doctor 2>&1 | grep -qE 'OK.*Framework vendoring|SKIP.*symlink'

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

### 2026-04-11T12:28:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-913-fw-doctor-check--warn-if-agentic-framewo.md
- **Context:** Initial task creation

### 2026-04-12T11:19:14Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-12T20:35:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Human reviewed
