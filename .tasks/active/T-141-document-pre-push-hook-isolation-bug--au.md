---
id: T-141
name: "Document pre-push hook isolation bug — audit checks framework instead of project"
description: >
  Document pre-push hook isolation bug — audit checks framework instead of project

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-14T22:20:01Z
last_update: 2026-03-14T22:42:27Z
date_finished: 2026-03-14T22:26:30Z
---

# T-141: Document pre-push hook isolation bug — audit checks framework instead of project

## Context

Pre-push hook in consumer projects (termlink) runs audit.sh without passing PROJECT_ROOT.
audit.sh → paths.sh falls back to `git rev-parse` from framework dir → resolves to `~/.agentic-framework`.
Result: termlink pushes blocked by framework repo issues (T-473 inception decision).

## Root Cause

`~/.agentic-framework/agents/git/lib/hooks.sh` line 328: `"$AUDIT_SCRIPT"` — missing `PROJECT_ROOT="$PROJECT_ROOT"` prefix.
`~/.agentic-framework/lib/paths.sh` lines 33-34: fallback resolves to framework repo when PROJECT_ROOT not set.

## Fix (in framework repo)

Line 328 of `hooks.sh`: change `"$AUDIT_SCRIPT"` → `PROJECT_ROOT="$PROJECT_ROOT" "$AUDIT_SCRIPT"`
Then reinstall hooks in consumer projects: `fw git install-hooks`

## Acceptance Criteria

### Agent
- [x] Bug documented with root cause, affected files, and fix
- [x] Feedback memory saved: never edit framework repo from consumer projects

### Human
- [ ] [REVIEW] Apply the one-line fix in the framework repo
  **Steps:**
  1. Open `~/.agentic-framework/agents/git/lib/hooks.sh`
  2. Find line 328: `"$AUDIT_SCRIPT"`
  3. Change to: `PROJECT_ROOT="$PROJECT_ROOT" "$AUDIT_SCRIPT"`
  4. In termlink: `fw git install-hooks` to regenerate the hook
  5. `git push` to verify it audits termlink, not framework
  **Expected:** Audit header shows `Project: /Users/dimidev32/001-projects/010-termlink`
  **If not:** Check that paths.sh respects the passed PROJECT_ROOT

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

### 2026-03-14T22:20:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-141-document-pre-push-hook-isolation-bug--au.md
- **Context:** Initial task creation

### 2026-03-14T22:26:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
