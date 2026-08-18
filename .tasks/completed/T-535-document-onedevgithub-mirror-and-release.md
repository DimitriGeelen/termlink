---
id: T-535
name: "Document OneDev→GitHub mirror and release chain in CLAUDE.md"
description: >
  Document OneDev→GitHub mirror and release chain in CLAUDE.md

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-27T18:19:21Z
last_update: '2026-08-18T18:59:17Z'
date_finished: 2026-03-27T18:21:07Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:57:01Z'
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
  - ts: '2026-08-18T18:59:17Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-535: Document OneDev→GitHub mirror and release chain in CLAUDE.md

## Context

T-534 RCA found agent repeatedly suggests `git push github` because CLAUDE.md doesn't document the OneDev→GitHub auto-mirror chain. Fix: add CI/Release section to CLAUDE.md.

## Acceptance Criteria

### Agent
- [x] CLAUDE.md has a CI/Release section documenting the push→mirror→release chain
- [x] Section states OneDev is the only push target
- [x] Section explains auto-mirror via `.onedev-buildspec.yml`
- [x] Section explains GitHub Actions release workflow triggers automatically

## Verification

grep -q 'onedev-buildspec' CLAUDE.md
grep -q 'auto-mirror' CLAUDE.md
grep -qi 'NEVER push to github' CLAUDE.md

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

### 2026-03-27T18:19:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-535-document-onedevgithub-mirror-and-release.md
- **Context:** Initial task creation

### 2026-03-27T18:21:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
