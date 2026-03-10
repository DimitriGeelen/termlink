---
id: T-062
name: "Agent-to-agent e2e tests — Claude Code orchestrator and specialists"
description: >
  Progressive e2e tests: echo, file task, persistent agent, multi-specialist. Validates TermLink as Claude Code inter-agent communication layer.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-09T17:37:33Z
last_update: 2026-03-09T17:37:33Z
date_finished: null
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
