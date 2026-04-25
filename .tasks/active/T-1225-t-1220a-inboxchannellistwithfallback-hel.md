---
id: T-1225
name: "T-1220a: inbox_channel::list_with_fallback helper (T-1220 wedge a)"
description: >
  termlink-session helper that wraps capabilities probe + channel.subscribe(topic=inbox:<target>) + legacy inbox.list fallback + dedup-merge. Foundation for T-1220b/c/d migrations. ~100 LOC + tests. Per T-1220 GO inception: in-memory cursor (Q1 D), per-session-per-target cap cache (Q2 B), warn-once + flag-legacy fallback (Q3 B+C), dual-read transition (Q5 A).

status: captured
workflow_type: build
owner: agent
horizon: now
tags: [T-1155, bus, migration, T-1220, wedge-a]
components: []
related_tasks: [T-1220, T-1215, T-1163]
created: 2026-04-25T07:00:04Z
last_update: 2026-04-25T07:00:04Z
date_finished: null
---

# T-1225: T-1220a: inbox_channel::list_with_fallback helper (T-1220 wedge a)

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

### 2026-04-25T07:00:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1225-t-1220a-inboxchannellistwithfallback-hel.md
- **Context:** Initial task creation
