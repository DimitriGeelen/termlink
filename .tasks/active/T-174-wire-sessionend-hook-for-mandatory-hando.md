---
id: T-174
name: "Wire SessionEnd hook for mandatory handover"
description: >
  Claude Code SessionEnd hook fires on session termination. Wire it to auto-trigger fw handover on every session exit. Known bugs: doesnt fire on /exit (#17885) or API 500 (#20197) — needs fallback.

status: captured
workflow_type: build
owner: human
horizon: next
tags: [framework, hooks, handover]
components: []
related_tasks: []
created: 2026-03-18T21:39:12Z
last_update: 2026-04-22T04:52:50Z
date_finished: null
---

# T-174: Wire SessionEnd hook for mandatory handover

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] [First criterion]
- [ ] [Second criterion]

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
