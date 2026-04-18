---
id: T-1136
name: "Update README one-liner install — replace 'cargo install --git' hint with install.sh curl-pipe (from T-1070 GO)"
description: >
  From T-1070 inception GO. README currently points consumers to 'brew install termlink' (macOS-centric) or 'cargo install --git' (requires toolchain — the failure mode for LXCs). After T-1070-install-sh lands, update README Install section to lead with the curl-pipe one-liner (cross-platform, no toolchain). Keep brew as the macOS preferred path, cargo as the 'from source' path, but de-emphasize. Small, text-only change that consolidates the install UX behind the bootstrap script.

status: captured
workflow_type: build
owner: agent
horizon: later
tags: [readme, docs, ux, T-1070]
components: []
related_tasks: []
created: 2026-04-18T23:02:47Z
last_update: 2026-04-18T23:02:47Z
date_finished: null
---

# T-1136: Update README one-liner install — replace 'cargo install --git' hint with install.sh curl-pipe (from T-1070 GO)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] [First criterion]
- [ ] [Second criterion]

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

### 2026-04-18T23:02:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1136-update-readme-one-liner-install--replace.md
- **Context:** Initial task creation
