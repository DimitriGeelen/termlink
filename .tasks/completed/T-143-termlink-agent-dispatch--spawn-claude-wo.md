---
id: T-143
name: "TermLink agent dispatch — spawn claude workers in real terminals"
description: >
  TermLink agent dispatch — spawn claude workers in real terminals

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-15T21:22:05Z
last_update: 2026-03-19T17:52:31Z
date_finished: 2026-03-15T22:50:33Z
---

# T-143: TermLink agent dispatch — spawn claude workers in real terminals

## Context

Build a dispatch script (`scripts/tl-dispatch.sh`) that spawns `claude -p` workers
in real TermLink terminal sessions. Each worker gets its own terminal, full context
window, isolated environment. Results collected via files + TermLink events.
See T-142 inception for full rationale.

## Acceptance Criteria

### Agent
- [x] `scripts/tl-dispatch.sh` exists and is executable
- [x] Spawn: `--name <worker> --prompt "..."` spawns a TermLink session and runs `claude -p`
- [x] Spawn: `--prompt-file` reads prompt from file (avoids shell escaping)
- [x] Status: `status` subcommand lists active workers with state
- [x] Wait: `wait --name <worker>` blocks until worker completes
- [x] Wait: `wait --all` blocks until all workers complete
- [x] Result: `result --name <worker>` prints worker output
- [x] Cleanup: `cleanup` kills all sessions and removes temp files
- [x] Completion signaled via TermLink event (topic: `worker.done`)

### Human
- [x] [REVIEW] Spawn a worker and verify it runs claude in a visible Terminal window
  **Steps:**
  1. Run `scripts/tl-dispatch.sh --name test-1 --prompt "What is 2+2? Reply with just the number."`
  2. Observe Terminal.app — a new window should appear with a running claude instance
  3. Wait ~15s, then run `scripts/tl-dispatch.sh result --name test-1`
  4. Run `scripts/tl-dispatch.sh cleanup`
  **Expected:** Result file contains "4", cleanup removes session
  **If not:** Check `/tmp/tl-dispatch/test-1/stderr.log` for errors

## Verification

test -x scripts/tl-dispatch.sh
grep -q "cmd_spawn" scripts/tl-dispatch.sh
grep -q "worker.done" scripts/tl-dispatch.sh

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

### 2026-03-15T21:22:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-143-termlink-agent-dispatch--spawn-claude-wo.md
- **Context:** Initial task creation

### 2026-03-15T22:50:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
