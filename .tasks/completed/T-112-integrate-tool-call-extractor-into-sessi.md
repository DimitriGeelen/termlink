---
id: T-112
name: "Integrate tool call extractor into session lifecycle"
description: >
  Hook extract-tool-calls.py into PreCompact/session-end. Append to .context/telemetry/tool-calls.jsonl.
  Align retention with T-110.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: [observability, telemetry, hooks, lifecycle]
components: []
related_tasks: [T-104, T-111, T-110]
created: 2026-03-12T06:31:11Z
last_update: '2026-08-18T18:58:44Z'
date_finished: 2026-03-12T06:39:54Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:46Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 2
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=2 (body:telemetry-or-audit-entry); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:44Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-112: Integrate tool call extractor into session lifecycle

## Context

Hook T-111's `extract-tool-calls.py` into the session lifecycle so tool call data
is automatically captured. Design from T-104 inception: batch extraction at PreCompact.

## Acceptance Criteria

### Agent
- [x] PreCompact hook script created (`capture-on-compact.sh`) — appends to `.context/telemetry/tool-calls.jsonl`
- [x] `.context/telemetry/` directory created if missing (mkdir -p in script)
- [x] Extraction runs silently (stderr to /dev/null, non-blocking — exits 0 on failure)
- [x] Manual extraction: `python3 agents/telemetry/extract-tool-calls.py --include-sidechains >> .context/telemetry/tool-calls.jsonl`
- [x] Integration tested: 1,008 valid JSONL records produced from current session
- [x] `.gitignore` updated to exclude `.context/telemetry/` (raw data, not committed)

## Verification

test -d .context/telemetry
python3 agents/telemetry/extract-tool-calls.py --help

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

### 2026-03-12T06:31:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-112-integrate-tool-call-extractor-into-sessi.md
- **Context:** Initial task creation

### 2026-03-12T06:39:54Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
