---
id: T-537
name: "Fix vendor gitignore — handle missing .gitignore file"
description: >
  Fix vendor gitignore — handle missing .gitignore file

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-27T19:24:45Z
last_update: 2026-03-27T19:24:45Z
date_finished: null
---

# T-537: Fix vendor gitignore — handle missing .gitignore file

## Context

`termlink vendor` warns when .gitignore doesn't have `.termlink/bin/` — but only if .gitignore exists. If there's no .gitignore at all, the warning is silently skipped. Fix: auto-append `.termlink/bin/` to .gitignore (creating it if needed).

## Acceptance Criteria

### Agent
- [x] `termlink vendor` creates .gitignore with `.termlink/` entry if no .gitignore exists
- [x] `termlink vendor` appends `.termlink/` to existing .gitignore if entry missing
- [x] Existing .gitignore content is preserved (no clobbering)
- [x] `cargo build` and `cargo test` pass

## Verification

cargo build 2>&1
# Test: vendor into fresh dir creates .gitignore
rm -rf /tmp/tl-gi-test && mkdir -p /tmp/tl-gi-test && cd /tmp/tl-gi-test && git init && /opt/termlink/target/debug/termlink vendor && grep -q '.termlink/bin' .gitignore

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

### 2026-03-27T19:24:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-537-fix-vendor-gitignore--handle-missing-git.md
- **Context:** Initial task creation
