---
id: T-1068
name: "Enhance handover partial-complete section with tag breakdown + age sort (follow-up
  to T-1066)"
description: >
  Enhance handover partial-complete section with tag breakdown + age sort (follow-up
  to T-1066)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-15T19:03:16Z
last_update: '2026-08-18T18:58:43Z'
date_finished: 2026-04-15T19:17:23Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:44Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 1
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=1 (body:episodic-only); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:43Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 3
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=3 
      (no-signal)
    rubric_sha: missing
---

# T-1068: Enhance handover partial-complete section with tag breakdown + age sort (follow-up to T-1066)

## Context

Follow-up to T-1066 (`fw task review-queue`). The handover agent has a partial-complete section at handover.sh:523–566 but lists tasks unsorted and untagged. Enhance: sort by date_finished ASC (oldest first), tag each entry RUBBER-STAMP / REVIEW / MIXED / UNTAGGED, add a summary line with counts by tag. Markdown-ready (no ANSI codes). Framework is vendored; local patch + docs/patches/ upstream record (same pattern as T-1066).

## Acceptance Criteria

### Agent
- [x] `.agentic-framework/agents/handover/handover.sh` partial-complete block enhanced: sorts by date_finished ASC, tags each task, adds summary
- [x] Test handover run renders the enhanced section
- [x] `docs/patches/T-1068-handover-partial-complete-tags.md` records the upstream patch

## Verification

grep -q 'RUBBER-STAMP\|tag_counts' .agentic-framework/agents/handover/handover.sh
test -f docs/patches/T-1068-handover-partial-complete-tags.md

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

### 2026-04-15T19:03:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1068-enhance-handover-partial-complete-sectio.md
- **Context:** Initial task creation

### 2026-04-15T19:17:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
