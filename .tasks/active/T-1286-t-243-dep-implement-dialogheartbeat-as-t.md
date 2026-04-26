---
id: T-1286
name: "T-243 dep: Implement dialog.heartbeat as typed RPC (or hub-tracked invariant)"
description: >
  The single must-be-infrastructure piece per T-243 inception (Agent B reframing + Agent C's own conversion trigger). Heartbeat: every ~5s during processing, responding agent emits lightweight signal on conversation channel. Two jobs at once: (a) resets caller's timeout clock — prevents 30s-timeout death of long LLM turns; (b) typing indicator emerges as side effect. Hub tracks last-heartbeat per (conversation_id, agent_id), evicts stale agents, resets request timeouts. Depends on channel-audit child task being clean.

status: captured
workflow_type: build
owner: agent
horizon: now
tags: [T-243, heartbeat, reliability]
components: []
related_tasks: []
created: 2026-04-26T09:31:58Z
last_update: 2026-04-26T09:31:58Z
date_finished: null
---

# T-1286: T-243 dep: Implement dialog.heartbeat as typed RPC (or hub-tracked invariant)

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

### 2026-04-26T09:31:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1286-t-243-dep-implement-dialogheartbeat-as-t.md
- **Context:** Initial task creation
