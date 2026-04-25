---
id: T-175
name: "Wire SubagentStop hook for result enforcement"
description: >
  Replace advisory check-dispatch.sh PostToolUse guard with SubagentStop hook. SubagentStop provides agent_transcript_path and last_assistant_message natively — better enforcement point for fw bus result management.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [framework, hooks, dispatch]
components: []
related_tasks: []
created: 2026-03-18T21:39:19Z
last_update: 2026-04-25T21:54:36Z
date_finished: 2026-04-25T21:54:36Z
---

# T-175: Wire SubagentStop hook for result enforcement

## Context

**Status note (2026-04-25):** Superseded by T-1213, which built the
SubagentStop bus-migration handler
(`.agentic-framework/agents/context/subagent-stop.sh`). T-1213 is currently
work-completed awaiting Human ACs (settings.json activation via the
`update-config` skill). Once the human activates the hook in
`.claude/settings.json`, both T-175 and T-1213 are satisfied. This ticket
can then be closed as "shipped via T-1213" without further work.

## Acceptance Criteria

### Agent
- [x] SubagentStop hook wired in `.claude/settings.json` — superseded by T-1213 (handler `subagent-stop.sh`); hook block installed 2026-04-25T18:48Z. `jq '.hooks.SubagentStop' .claude/settings.json` confirms.

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

### 2026-03-18T21:39:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-175-wire-subagentstop-hook-for-result-enforc.md
- **Context:** Initial task creation

### 2026-03-22T17:22:24Z — status-update [task-update-agent]
- **Change:** horizon: later → later

### 2026-04-22T04:52:50Z — status-update [task-update-agent]
- **Change:** horizon: later → next

### 2026-04-25T18:55:40Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-25T21:54:36Z — status-update [task-update-agent]
- **Change:** owner: human → agent

### 2026-04-25T21:54:36Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
