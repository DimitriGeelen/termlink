---
id: T-1131
name: "Wire protocol_version enforcement at hub — structured error instead of opaque serde parse failure (from T-1071 GO)"
description: >
  From T-1071 inception GO. Hub records each registered session's declared protocol_version (Capabilities.protocol_version: u8, already on wire at control.rs:79 but zero enforcement). On RPC call from a session whose declared version < hub's DATA_PLANE_VERSION for that method, return structured error PROTOCOL_VERSION_TOO_OLD with min required version, instead of letting serde fail with opaque parse error. Backwards-compatible: missing field defaults to 1. This converts the KeyEntry-style silent failures into actionable 'upgrade your client' messages. Load-bearing fix of the three T-1071 follow-ups.

status: captured
workflow_type: build
owner: agent
horizon: later
tags: [protocol, termlink, version-skew, T-1071]
components: []
related_tasks: []
created: 2026-04-18T22:59:37Z
last_update: 2026-04-18T22:59:37Z
date_finished: null
---

# T-1131: Wire protocol_version enforcement at hub — structured error instead of opaque serde parse failure (from T-1071 GO)

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

### 2026-04-18T22:59:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1131-wire-protocolversion-enforcement-at-hub-.md
- **Context:** Initial task creation
