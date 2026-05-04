---
id: T-1452
name: "revisit-due-scan.sh cron + handover banner integration (T-1449 Phase-1 #2)"
description: >
  T-1449 Phase-1 deliverable #2: daily 07:00 cron scans .tasks/active/*.md for revisit_at <= today, writes ripe revisits to .context/working/.revisits-due.txt. Handover banner reads the file. Watchtower /home page surfaces it. Prerequisite: T-1451 (revisit_at field). ~50 LOC. Channel-1 mirror to upstream framework needed.

status: captured
workflow_type: build
owner: human
horizon: now
tags: [framework, governance, T-1449, phase-1, channel-1-mirror, cron]
components: []
related_tasks: [T-1449, T-1451]
created: 2026-05-02T22:21:38Z
last_update: 2026-05-04T05:12:34Z
date_finished: null
---

# T-1452: revisit-due-scan.sh cron + handover banner integration (T-1449 Phase-1 #2)

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

### 2026-05-02T22:21:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1452-revisit-due-scansh-cron--handover-banner.md
- **Context:** Initial task creation
