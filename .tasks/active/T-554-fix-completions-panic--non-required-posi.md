---
id: T-554
name: "Fix completions panic — non-required positional before required"
description: >
  Fix completions panic — non-required positional before required

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-28T09:57:09Z
last_update: 2026-03-28T09:57:09Z
date_finished: null
---

# T-554: Fix completions panic — non-required positional before required

## Context

`termlink completions bash` panics: "Found non-required positional argument with a lower index than a required positional argument: target". A hidden backward-compat command has an optional `target` before a required arg.

## Acceptance Criteria

### Agent
- [x] Root cause identified — Interact, Inject (hidden), PtyCommand::Inject had Optional target before required command/text
- [x] Fix applied — made target required (String) in all 3 variants
- [x] `termlink completions bash` runs without panic
- [x] `termlink completions zsh` and `termlink completions fish` also work
- [x] All tests pass

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

### 2026-03-28T09:57:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-554-fix-completions-panic--non-required-posi.md
- **Context:** Initial task creation
