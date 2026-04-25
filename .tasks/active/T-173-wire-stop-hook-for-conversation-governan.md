---
id: T-173
name: "Wire Stop hook for conversation governance (G-005)"
description: >
  Claude Code Stop hook fires after every response with last_assistant_message. Wire it into framework to enforce N-exchange guard and close G-005 (pure conversation sessions bypass enforcement).

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [framework, hooks, governance]
components: []
related_tasks: []
created: 2026-03-18T21:39:06Z
last_update: 2026-04-25T18:55:39Z
date_finished: null
---

# T-173: Wire Stop hook for conversation governance (G-005)

## Context

**Status note (2026-04-25):** Superseded by T-1211, which built the actual
Stop-hook nudge script (`.agentic-framework/agents/context/stop-guard.sh`).
T-1211 is currently work-completed awaiting Human ACs (settings.json
activation via the `update-config` skill). Once the human activates the
hook in `.claude/settings.json`, both T-173 and T-1211 are satisfied. This
ticket can then be closed as "shipped via T-1211" without further work.

## Acceptance Criteria

### Agent
- [x] Stop hook wired in `.claude/settings.json` — superseded by T-1211 (handler `stop-guard.sh`); hook block installed 2026-04-25T18:48Z. `jq '.hooks.Stop' .claude/settings.json` confirms.

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     Examples:
       python3 -c "import yaml; yaml.safe_load(open('path/to/file.yaml'))"
       curl -sf http://localhost:3000/page
       grep -q "expected_string" output_file.txt
-->

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

### 2026-03-18T21:39:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-173-wire-stop-hook-for-conversation-governan.md
- **Context:** Initial task creation

### 2026-03-18T21:55:34Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-18T22:25:12Z — status-update [task-update-agent]
- **Change:** status: started-work → captured

### 2026-03-22T17:22:24Z — status-update [task-update-agent]
- **Change:** horizon: later → later

### 2026-04-22T04:52:49Z — status-update [task-update-agent]
- **Change:** horizon: later → next

### 2026-04-25T18:55:39Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
