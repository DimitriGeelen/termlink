---
id: T-1247
name: "T-1247 fw-upgrade governance restoration: keep project-specific fw path guidance"
description: >
  T-1247 fw-upgrade governance restoration: keep project-specific fw path guidance

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T11:04:48Z
last_update: 2026-04-25T11:04:48Z
date_finished: null
---

# T-1247: T-1247 fw-upgrade governance restoration: keep project-specific fw path guidance

## Context

`fw upgrade` (run during T-1218 prep) replaced project-specific CLAUDE.md
governance text with framework-template defaults. One regression: the
"Copy-pasteable Commands" rule lost the explicit guidance that consumer
projects must use `.agentic-framework/bin/fw`, not `bin/fw`. Per saved
memory `feedback_fw_path_consumer.md`, this is a load-bearing project
rule. Restore.

## Acceptance Criteria

### Agent
- [x] CLAUDE.md line 715 restored to project-specific version (mentions `.agentic-framework/bin/fw` explicitly)
- [x] CLAUDE.md.bak removed (no longer needed once restoration committed)
- [x] Commit message references the regression source (fw upgrade)

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [x] [REVIEW] Dashboard renders correctly
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

### 2026-04-25T11:04:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1247-t-1247-fw-upgrade-governance-restoration.md
- **Context:** Initial task creation
