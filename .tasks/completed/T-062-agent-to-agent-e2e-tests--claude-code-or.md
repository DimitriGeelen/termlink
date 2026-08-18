---
id: T-062
name: "Agent-to-agent e2e tests — Claude Code orchestrator and specialists"
description: >
  Progressive e2e tests: echo, file task, persistent agent, multi-specialist. Validates
  TermLink as Claude Code inter-agent communication layer.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-09T17:37:33Z
last_update: '2026-08-18T18:58:41Z'
date_finished: 2026-03-10T08:07:29Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:40Z'
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
  - ts: '2026-08-18T18:58:41Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-062: Agent-to-agent e2e tests — Claude Code orchestrator and specialists

## Context

Progressive e2e tests validating TermLink as Claude Code inter-agent communication layer. Tests escalate from simple echo to 3 parallel specialists.

## Acceptance Criteria

### Agent
- [x] Level 1 echo test exists and passes — one-shot agent emits event back to orchestrator
- [x] Level 2 file task test exists and passes — specialist reads file, writes summary, emits task.completed
- [x] Level 3 persistent agent test exists and passes — same watcher handles 2 sequential tasks
- [x] Level 4 multi-specialist test exists and passes — 3 parallel specialists (reviewer, tester, documenter)
- [x] Reusable specialist-watcher.sh for persistent agent pattern
- [x] Agent delegation event schema convention documented

## Verification

# All test scripts exist and are executable
test -x tests/e2e/level1-echo.sh
test -x tests/e2e/level2-file-task.sh
test -x tests/e2e/level3-persistent-agent.sh
test -x tests/e2e/level4-multi-specialist.sh
test -x tests/e2e/specialist-watcher.sh
# Convention doc exists
test -f docs/conventions/agent-delegation-events.md

## Decisions

### 2026-03-09 — Prompt delivery mechanism
- **Chose:** Write prompt to file, use `claude -p "$(cat $PROMPT_FILE)"`
- **Why:** AppleScript quote mangling corrupts complex prompts passed inline
- **Rejected:** Inline prompts via AppleScript `do script` — quotes get corrupted

### 2026-03-09 — Persistent agent pattern
- **Chose:** Bash watcher loop polling events, dispatching to fresh `claude -p` per task
- **Why:** Each task gets fresh 200K context window; watcher stays lightweight
- **Rejected:** Single long-running Claude session — would exhaust context on multi-task workloads

## Updates

### 2026-03-09T17:37:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-062-agent-to-agent-e2e-tests--claude-code-or.md
- **Context:** Initial task creation

### 2026-03-10T08:07:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
