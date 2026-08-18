---
id: T-584
name: "Add --timeout to kv and event emit/emit-to/broadcast"
description: >
  Add --timeout to kv and event emit/emit-to/broadcast

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/cli.rs, 
      crates/termlink-cli/src/commands/events.rs, 
      crates/termlink-cli/src/commands/metadata.rs, 
      crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-28T15:49:53Z
last_update: '2026-08-18T18:59:18Z'
date_finished: 2026-03-28T15:53:35Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:03Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=0 (no-signal); 
      D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:18Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 5
      tier: 2
      effort: 7
    rationale: blast_radius=5 (no-signal); tier=2 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-584: Add --timeout to kv and event emit/emit-to/broadcast

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] --timeout added to Kv command in cli.rs (default 5s)
- [x] --timeout added to EventCommand::Emit, EmitTo, Broadcast and hidden aliases (default 5s)
- [x] cmd_kv wraps all RPC calls in tokio::time::timeout
- [x] cmd_emit, cmd_emit_to, cmd_broadcast wrap RPC calls in tokio::time::timeout
- [x] cargo build succeeds

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

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     Examples:
       python3 -c "import yaml; yaml.safe_load(open('path/to/file.yaml'))"
       curl -sf http://localhost:3000/page
       grep -q "expected_string" output_file.txt
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

## Updates

### 2026-03-28T15:49:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-584-add---timeout-to-kv-and-event-emitemit-t.md
- **Context:** Initial task creation

### 2026-03-28T15:53:35Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
