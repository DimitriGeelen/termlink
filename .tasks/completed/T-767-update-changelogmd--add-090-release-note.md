---
id: T-767
name: "Update CHANGELOG.md — add 0.9.0 release notes for features since 0.8.0"
description: >
  Update CHANGELOG.md — add 0.9.0 release notes for features since 0.8.0

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-29T23:05:03Z
last_update: '2026-08-18T18:59:20Z'
date_finished: 2026-03-29T23:06:48Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:09Z'
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
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-767: Update CHANGELOG.md — add 0.9.0 release notes for features since 0.8.0

## Context

CHANGELOG.md has entries for 0.8.0, 0.7.0, etc. but is missing a 0.9.0 section (307 commits, major features: vendor, push, file transfer, agent protocol, dispatch improvements, release optimization).

## Acceptance Criteria

### Agent
- [x] Add 0.9.0 section to CHANGELOG.md with Added/Changed/Fixed subsections
- [x] Cover major features: vendor, push, file transfer, agent protocol, dispatch, release optimization, Linux aarch64
- [x] Follow Keep a Changelog format consistent with existing entries
- [x] Include test count update

## Verification

grep -q "0.9.0" CHANGELOG.md

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

### 2026-03-29T23:05:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-767-update-changelogmd--add-090-release-note.md
- **Context:** Initial task creation

### 2026-03-29T23:06:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
