---
id: T-286
name: "Fix remote exec clap panic — same positional arg bug as T-284"
description: >
  Fix remote exec clap panic — same positional arg bug as T-284

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-03-25T20:25:29Z
last_update: 2026-03-25T20:28:27Z
date_finished: 2026-03-25T20:28:27Z
---

# T-286: Fix remote exec clap panic — same positional arg bug as T-284

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `termlink remote exec` no longer panics with `Option<String>` before required positional arg
- [x] Session parameter is now required (matches T-284 fix pattern for inject/send-file)
- [x] `termlink remote exec --help` works without panic

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

### 2026-03-25T20:25:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-286-fix-remote-exec-clap-panic--same-positio.md
- **Context:** Initial task creation

### 2026-03-25T20:28:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
