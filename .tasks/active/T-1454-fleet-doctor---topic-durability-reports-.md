---
id: T-1454
name: "fleet doctor --topic-durability reports audit_unsupported on T-1446-bearing hubs"
description: >
  All 4 fleet hubs (.107 0.9.1701, .122/.141/.121 0.9.1702) report 'audit_unsupported (pre-T-1446 hub)' when running termlink fleet doctor --topic-durability — but hub-side handle_hub_bus_state exists in router.rs:920+ and T-1446 was committed at 204ad1d1 (in-tree before all hub binaries were built). Suspected bug: either client-side dispatch returning Err before reaching hub, or hub-side router not registering the method, or version-gate logic in remote.rs:1758-1772 misclassifying the response. Reproduce: termlink fleet doctor --topic-durability. Expected: each hub should return runtime_dir + audit_present + topic-list. Actual: all 4 hubs report audit_unsupported. Discovered 2026-05-03T10:15Z while completing G-051 mitigation.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-05-03T08:17:28Z
last_update: 2026-05-03T08:17:28Z
date_finished: null
---

# T-1454: fleet doctor --topic-durability reports audit_unsupported on T-1446-bearing hubs

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

### 2026-05-03T08:17:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1454-fleet-doctor---topic-durability-reports-.md
- **Context:** Initial task creation
