---
id: T-210
name: "Fix aef references in installer, add post-install verification"
description: >
  Fix aef references in installer, add post-install verification

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-21T15:42:42Z
last_update: 2026-03-22T17:01:23Z
date_finished: 2026-03-22T17:01:23Z
---

# T-210: Fix aef references in installer, add post-install verification

## Context

Build task for T-207 inception (fix phantom "aef" binary name). Resolved upstream — install.sh already uses `fw` throughout with zero `aef` references and 3-step post-install verification.

## Acceptance Criteria

### Agent
- [x] Zero "aef" references in install.sh (verified upstream)
- [x] Post-install verification present in install.sh (3-step: binary, version, doctor)

## Verification

! grep -qi "aef" /opt/999-Agentic-Engineering-Framework/install.sh
grep -q "Post-install verification" /opt/999-Agentic-Engineering-Framework/install.sh

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

### 2026-03-21T15:42:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimitri/.termlink/.tasks/active/T-210-fix-aef-references-in-installer-add-post.md
- **Context:** Initial task creation

### 2026-03-22T17:01:23Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
