---
id: T-157
name: "claude-fw --termlink flag"
description: >
  Add opt-in --termlink flag to claude-fw wrapper so every session auto-registers
  as a TermLink session, enabling remote observation and input injection.

status: work-completed
workflow_type: build
owner: human
horizon:
tags: [remote-access, claude-fw, framework]
components: []
related_tasks: [T-155, T-156, T-158]
created: 2026-03-17T09:45:45Z
last_update: '2026-08-18T18:58:51Z'
date_finished: 2026-03-17T11:37:19Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:03Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (body:wrap-phrase-without-substrate)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:51Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-157: claude-fw --termlink flag

## Context

T-155 validated Claude Code works in TermLink PTY. T-156 created `tl-claude.sh`.
T-158 added persistent mode. This task creates a pickup prompt for the framework
project to integrate `--termlink` into `claude-fw`. We don't edit framework files
from the TermLink repo.

## Acceptance Criteria

### Agent
- [x] Pickup prompt written at `docs/specs/T-157-claude-fw-termlink-pickup.md`
- [x] Prompt includes exact code changes for claude-fw
- [x] Prompt includes CLAUDE.md section addition
- [x] Prompt includes testing steps

### Human
- [ ] [REVIEW] Paste prompt into framework Claude Code session and verify integration
  **Steps:**
  1. Open a Claude Code session in the framework project
  2. Paste the pickup prompt from `docs/specs/T-157-claude-fw-termlink-pickup.md`
  3. Verify claude-fw gets `--termlink` flag
  4. Test: `claude-fw --termlink` registers a TermLink session
  **Expected:** Claude Code starts wrapped in TermLink session
  **If not:** Note what the framework agent reports as issues

## Verification

test -f docs/specs/T-157-claude-fw-termlink-pickup.md
grep -q "termlink" docs/specs/T-157-claude-fw-termlink-pickup.md

## Updates

### 2026-03-17T09:45:45Z — task-created
### 2026-03-17T11:36:00Z — started-work

### 2026-03-17T11:37:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
