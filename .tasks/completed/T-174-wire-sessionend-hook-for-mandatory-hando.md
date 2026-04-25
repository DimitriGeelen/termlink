---
id: T-174
name: "Wire SessionEnd hook for mandatory handover"
description: >
  Claude Code SessionEnd hook fires on session termination. Wire it to auto-trigger fw handover on every session exit. Known bugs: doesnt fire on /exit (#17885) or API 500 (#20197) — needs fallback.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [framework, hooks, handover]
components: []
related_tasks: []
created: 2026-03-18T21:39:12Z
last_update: 2026-04-25T21:54:34Z
date_finished: 2026-04-25T21:54:34Z
---

# T-174: Wire SessionEnd hook for mandatory handover

## Context

**Status note (2026-04-25):** Superseded by T-1212, which built the
SessionEnd handler (`.agentic-framework/agents/context/session-end.sh`)
plus the silent-session cron scanner. T-1212 is currently work-completed
awaiting Human ACs (settings.json activation via the `update-config`
skill). Once the human activates the hook in `.claude/settings.json`, both
T-174 and T-1212 are satisfied. This ticket can then be closed as
"shipped via T-1212" without further work.

## Acceptance Criteria

### Agent
- [x] SessionEnd hook wired in `.claude/settings.json` — superseded by T-1212 (handler `session-end.sh` + silent-session cron); hook block installed 2026-04-25T18:48Z, cron at `*/15`. `jq '.hooks.SessionEnd' .claude/settings.json` confirms.

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

### 2026-03-18T21:39:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-174-wire-sessionend-hook-for-mandatory-hando.md
- **Context:** Initial task creation

### 2026-03-22T17:22:24Z — status-update [task-update-agent]
- **Change:** horizon: later → later

### 2026-04-22T04:52:50Z — status-update [task-update-agent]
- **Change:** horizon: later → next

### 2026-04-25T18:55:40Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-25T21:54:34Z — status-update [task-update-agent]
- **Change:** owner: human → agent

### 2026-04-25T21:54:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
