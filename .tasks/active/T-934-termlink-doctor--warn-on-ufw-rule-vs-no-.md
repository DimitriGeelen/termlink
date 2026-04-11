---
id: T-934
name: "termlink doctor — warn on UFW-rule-vs-no-listener mismatch"
description: >
  Cheap doctor check that catches the exact state T-930 started from: UFW has an ALLOW rule for hub port 9100/tcp, but nothing is listening on 9100. Implementation: read ufw status output (sudo-free if possible), grep for 9100/tcp rule, then ss -tln to check listener. If rule present but no listener, emit a warn-level doctor check. Belt-and-braces for the systemd unit approach — catches manual kills, crashes-before-restart, and botched unit edits. From T-930 decomposition.

status: captured
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-930]
created: 2026-04-11T22:29:30Z
last_update: 2026-04-11T22:29:30Z
date_finished: null
---

# T-934: termlink doctor — warn on UFW-rule-vs-no-listener mismatch

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

### 2026-04-11T22:29:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-934-termlink-doctor--warn-on-ufw-rule-vs-no-.md
- **Context:** Initial task creation
