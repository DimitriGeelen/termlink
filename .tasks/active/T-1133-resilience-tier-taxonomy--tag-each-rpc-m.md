---
id: T-1133
name: "Resilience-tier taxonomy — tag each RPC method Tier-A (opaque) or Tier-B (typed) in protocol doc comments (from T-1071 GO)"
description: >
  From T-1071 inception GO. Tag every RPC method as Tier-A (opaque payload, drift-tolerant — event.broadcast, event.emit, kv.set strings) or Tier-B (typed struct, drift-fragile — command.inject, command.exec, session.update). Document in crates/termlink-protocol/src/control.rs as doc comments on each method constant. fleet doctor can then flag fleets where Tier-B methods would fail across the observed version diversity (extends T-1132). This is the 'codify the event.broadcast resilience property' deliverable — promotes a happy accident into a documented design tier.

status: captured
workflow_type: build
owner: agent
horizon: later
tags: [termlink, protocol, taxonomy, documentation, T-1071]
components: []
related_tasks: []
created: 2026-04-18T23:00:36Z
last_update: 2026-04-18T23:00:36Z
date_finished: null
---

# T-1133: Resilience-tier taxonomy — tag each RPC method Tier-A (opaque) or Tier-B (typed) in protocol doc comments (from T-1071 GO)

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

### 2026-04-18T23:00:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1133-resilience-tier-taxonomy--tag-each-rpc-m.md
- **Context:** Initial task creation
