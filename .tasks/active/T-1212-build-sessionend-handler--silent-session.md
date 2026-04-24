---
id: T-1212
name: "Build SessionEnd handler + silent-session cron (T-1208 follow-up)"
description: >
  Implement per T-1208 GO: (S1) no-op SessionEnd logger for reason-field baseline; (S2) handover-trigger with idempotency guard (session_id match); (S3) 15-min silent-session cron scanning .claude/sessions/*.jsonl for sessions idle >30min with no handover, generating recovery handover marked [recovered, no agent context]. S3 is the antifragility piece — do not ship S2 without S3. See docs/reports/T-1208-sessionend-hook-inception.md.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: [hook, handover, framework-bridge, antifragility]
components: []
related_tasks: [T-1208, T-174]
created: 2026-04-24T10:05:10Z
last_update: 2026-04-24T10:05:10Z
date_finished: null
---

# T-1212: Build SessionEnd handler + silent-session cron (T-1208 follow-up)

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

### 2026-04-24T10:05:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1212-build-sessionend-handler--silent-session.md
- **Context:** Initial task creation
