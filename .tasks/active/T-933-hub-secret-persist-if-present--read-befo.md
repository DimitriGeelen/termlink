---
id: T-933
name: "Hub secret persist-if-present — read before generate"
description: >
  crates/termlink-hub/src/server.rs:46 generate_and_write_hub_secret() unconditionally generates a fresh secret on every start, and server.rs:154/210 deletes the file on clean shutdown. Rotation is incidental, not deliberate (no comment asserts it as a security property — T-930 Spike 3). Fix: read existing hex if present and valid (64 chars, mode 0600, parses), otherwise generate. Remove remove_file(hub_secret_path()) from both cleanup paths. Add integration test that starts hub twice and asserts the same secret is used. Zero network security delta. From T-930 decomposition.

status: captured
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-930, T-931]
created: 2026-04-11T22:29:26Z
last_update: 2026-04-11T22:29:26Z
date_finished: null
---

# T-933: Hub secret persist-if-present — read before generate

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

### 2026-04-11T22:29:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-933-hub-secret-persist-if-present--read-befo.md
- **Context:** Initial task creation
