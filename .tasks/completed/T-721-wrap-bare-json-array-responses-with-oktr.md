---
id: T-721
name: "Wrap bare JSON array responses with ok:true in list, discover, and remote list"
description: >
  Wrap bare JSON array responses with ok:true in list, discover, and remote list

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/metadata.rs, 
      crates/termlink-cli/src/commands/remote.rs, 
      crates/termlink-cli/src/commands/session.rs]
related_tasks: []
created: 2026-03-29T11:34:31Z
last_update: '2026-08-18T18:59:20Z'
date_finished: 2026-03-29T11:35:52Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:07Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 6
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-721: Wrap bare JSON array responses with ok:true in list, discover, and remote list

## Context

`list --json`, `discover --json`, and `remote list --json` return bare arrays. All JSON success responses should wrap with `{"ok": true, "sessions": [...]}` for consistency.

## Acceptance Criteria

### Agent
- [x] `cmd_list` JSON output wraps with `{"ok": true, "sessions": [...]}`
- [x] `cmd_list --names --json` wraps with `{"ok": true, "names": [...]}`
- [x] `cmd_list --ids --json` wraps with `{"ok": true, "ids": [...]}`
- [x] `cmd_discover` JSON output wraps with `{"ok": true, "sessions": [...]}`
- [x] `cmd_remote_list` JSON output wraps with `{"ok": true, "sessions": [...]}`
- [x] Project compiles with `cargo check`

## Verification

cargo check 2>&1 | grep -q 'Finished'

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

### 2026-03-29T11:34:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-721-wrap-bare-json-array-responses-with-oktr.md
- **Context:** Initial task creation

### 2026-03-29T11:35:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
