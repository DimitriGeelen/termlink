---
id: T-1342
name: "doc updates — T-1335..T-1341 agent-conversation feature wave"
description: >
  doc updates — T-1335..T-1341 agent-conversation feature wave

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-04-27T19:20:03Z
last_update: '2026-08-18T18:58:48Z'
date_finished: 2026-04-27T19:22:01Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:55Z'
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
  - ts: '2026-08-18T18:58:48Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-1342: doc updates — T-1335..T-1341 agent-conversation feature wave

## Context

Document the 7-feature wave shipped this session (T-1335 stats, T-1336 search, T-1337 ack --since, T-1338 dm inbox, T-1339 mentions inbox, T-1340 ancestors, T-1341 members) in `docs/operations/agent-conversations.md`. Update the e2e step count from 10 → 19. Add Related task pointers.

## Acceptance Criteria

### Agent
- [x] Doc has new sections: "Topic stats and search", "Inbox views", "Receipt anchoring", "Thread navigation"
- [x] e2e step count corrected to 19
- [x] Related-tasks list extended with T-1335..T-1341
- [x] No broken markdown — file still parses (20 H2 sections, header `# Agent conversations on the channel bus`)

## Verification

head -1 docs/operations/agent-conversations.md
grep -c "^## " docs/operations/agent-conversations.md

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

### 2026-04-27T19:20:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1342-doc-updates--t-1335t-1341-agent-conversa.md
- **Context:** Initial task creation

### 2026-04-27T19:22:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
