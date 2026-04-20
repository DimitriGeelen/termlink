---
id: T-1164
name: "T-1155/7 Migrate file.send/receive → channel.post {type: artifact}"
description: >
  ~10 sites in file.rs, remote.rs, main.rs, tools.rs. Artifact becomes typed channel.post; chunked transfer becomes bus implementation detail. Replaces T-1017/T-1018 fix path (silent drop, stale chunks) with channel semantics.

status: captured
workflow_type: refactor
owner: agent
horizon: later
tags: [T-1155, bus, migration]
components: []
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:15Z
last_update: 2026-04-20T14:12:15Z
date_finished: null
---

# T-1164: T-1155/7 Migrate file.send/receive → channel.post {type: artifact}

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

### 2026-04-20T14:12:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1164-t-11557-migrate-filesendreceive--channel.md
- **Context:** Initial task creation
