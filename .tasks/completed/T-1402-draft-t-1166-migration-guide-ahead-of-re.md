---
id: T-1402
name: "Draft T-1166 migration guide ahead of retirement"
description: >
  Draft T-1166 migration guide ahead of retirement

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-29T08:49:34Z
last_update: '2026-08-18T18:58:49Z'
date_finished: 2026-04-29T08:51:42Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:57Z'
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
  - ts: '2026-08-18T18:58:49Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 5
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=5 
      (no-signal)
    rubric_sha: missing
---

# T-1402: Draft T-1166 migration guide ahead of retirement

## Context

T-1166 (legacy primitive retirement) requires a published migration guide at
`docs/migrations/T-1166-retire-legacy-primitives.md` for downstream consumers.
The guide can be drafted now (does not require T-1166's bake window to pass)
so it's ready when retirement actually starts. Pre-staging removes one piece
of work from the critical path.

Audience: downstream consumers (ring20-management, ring20-dashboard,
ntb-atc-plugin, skills-manager, framework-agent, etc.) who currently call any
of: `event.broadcast`, `inbox.list`, `inbox.status`, `inbox.clear`,
`file.send`, `file.receive`. Tells them what to do BEFORE the protocol bump
breaks their existing calls.

## Acceptance Criteria

### Agent
- [x] `docs/migrations/T-1166-retire-legacy-primitives.md` exists with these sections: Audience, What's Removed, Migration Recipe (per-method), Timeline, Capability Handshake change, Diagnostic, Roll-Forward Checklist
- [x] For each retired method, the doc shows the legacy call shape AND the replacement call shape side-by-side with a one-liner diff
- [x] Doc cross-links the corresponding ship-tasks (T-1162 mirror, T-1163 inbox, T-1164 file, T-1400 doctor migration, T-1401 broadcast migration) so consumers can read the full history
- [x] Doc states the timeline contract clearly: "After this version of the hub ships, calls to these legacy methods receive `method_not_found`. The capability handshake advertises `legacy_primitives = false`."

## Verification

test -f docs/migrations/T-1166-retire-legacy-primitives.md
grep -q "broadcast:global" docs/migrations/T-1166-retire-legacy-primitives.md
grep -q "channel.list" docs/migrations/T-1166-retire-legacy-primitives.md
grep -q "T-1162\|T-1163\|T-1164" docs/migrations/T-1166-retire-legacy-primitives.md
grep -q "legacy_primitives" docs/migrations/T-1166-retire-legacy-primitives.md

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

### 2026-04-29T08:49:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1402-draft-t-1166-migration-guide-ahead-of-re.md
- **Context:** Initial task creation

### 2026-04-29T08:51:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
