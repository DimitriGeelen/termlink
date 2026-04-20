---
id: T-1157
name: "Pickup to framework — G-006 pre-push hook stamps wrong VERSION into .agentic-framework/VERSION"
description: >
  Pickup to framework — G-006 pre-push hook stamps wrong VERSION into .agentic-framework/VERSION

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-20T13:54:50Z
last_update: 2026-04-20T13:54:50Z
date_finished: null
---

# T-1157: Pickup to framework — G-006 pre-push hook stamps wrong VERSION into .agentic-framework/VERSION

## Context

G-006 (registered 2026-04-15): framework's pre-push hook at `.agentic-framework/agents/git/lib/hooks.sh:399-401` writes the project's git-derived version into `$PROJECT_ROOT/.agentic-framework/VERSION`. Wrong target — the vendored framework's VERSION file should record which framework release was vendored (set by `fw vendor`/`fw upgrade`), not the consumer project's version.

Symptom: `.agentic-framework/VERSION` shows as modified in `git status` on every push (been seen repeatedly this session). T-1061 reverted it manually once; it keeps drifting.

Fix: delete the 3 lines writing to `.agentic-framework/VERSION`. The file is managed by `fw vendor`, not by per-push stamping. Upstream change only — patching the vendored copy locally is pointless because the next `fw upgrade` reverts it.

## Acceptance Criteria

### Agent
- [x] Pickup envelope drafted citing exact line range and proposed fix (delete 3 lines)
- [x] Pickup delivered to framework via termlink + direct inbox drop
- [x] Framework received pickup (present in inbox/ or processed/ — processor cron picks up within a minute)

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
test -f /opt/999-Agentic-Engineering-Framework/.context/pickup/inbox/P-T-1157-bug-report.yaml || test -f /opt/999-Agentic-Engineering-Framework/.context/pickup/processed/P-T-1157-bug-report.yaml

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

### 2026-04-20T13:54:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1157-pickup-to-framework--g-006-pre-push-hook.md
- **Context:** Initial task creation
