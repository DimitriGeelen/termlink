---
id: T-1279
name: "Fix fw promote dedup: check promoted_from not derived_from (PL-083)"
description: >
  Fix fw promote dedup: check promoted_from not derived_from (PL-083)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-25T21:18:02Z
last_update: '2026-08-18T18:58:47Z'
date_finished: 2026-04-25T21:20:24Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:52Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=2 (body:learning-ref); D2=0 (no-signal); D3=0 (no-signal); 
      D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:47Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-1279: Fix fw promote dedup: check promoted_from not derived_from (PL-083)

## Context

PL-083 — fw promote dedup uses `derived_from` (which holds the directive D1/D2/...) instead of `promoted_from` (which holds the L-XXX origin). Practices from `fw promote` always set derived_from=Dx, so dedup never matches. Result: an already-promoted L-XXX with TBD application field gets re-suggested and re-promoted. Hit on T-1278 (L-007 → duplicate PP-009).

Fix: at lib/promote.sh:62-72, also collect `promoted_from` into the `promoted_ids` set. One-place change fixes both `suggest` (line 138-149) and the explicit-promote dedup gate (line 249-251).

## Acceptance Criteria

### Agent
- [x] /opt/999-Agentic-Engineering-Framework/lib/promote.sh patched (commit 84a77e66 on master)
- [x] Vendored /opt/termlink/.agentic-framework/lib/promote.sh updated to match
- [x] `fw promote suggest` no longer lists L-007 in output (verified)
- [x] `fw promote L-007 ...` exits with "Learning L-007 is already promoted to a practice"
- [x] Upstream commit pushed to onedev master (84a77e66)

## Verification

test ! -z "$(grep -E 'promoted_from' .agentic-framework/lib/promote.sh | grep -v '^\s*#')"
test -z "$(.agentic-framework/bin/fw promote suggest 2>&1 | grep -E '^\s*L-007 ')"
test -n "$(.agentic-framework/bin/fw promote L-007 --name x --directive D2 2>&1 | grep 'already promoted')"

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

### 2026-04-25T21:18:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1279-fix-fw-promote-dedup-check-promotedfrom-.md
- **Context:** Initial task creation

### 2026-04-25T21:20:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
