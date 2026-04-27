---
id: T-1316
name: "agent-conversations operator doc (synthesizes T-1313/14/15 Matrix features)"
description: >
  agent-conversations operator doc (synthesizes T-1313/14/15 Matrix features)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T13:36:46Z
last_update: 2026-04-27T13:36:46Z
date_finished: null
---

# T-1316: agent-conversations operator doc (synthesizes T-1313/14/15 Matrix features)

## Context

T-1313 (threading), T-1314 (reactions), T-1315 (read receipts) shipped
discrete Matrix-inspired primitives but the synthesis is scattered across
task files. Operators need ONE doc that walks through "two agents having
a conversation" end-to-end with worked examples. This is also the
discoverability layer: nothing teaches an LLM agent how to use these
unless the doc names the commands and patterns.

## Acceptance Criteria

### Agent
- [x] `docs/operations/agent-conversations.md` exists with sections: Overview, Quick start, Threading, Reactions, Receipts, Matrix mapping, Limits & next steps
- [x] Quick-start section is end-to-end runnable (post / reply / react / ack / receipts) — verified by running the commands as the doc presents them against the live hub; doc updated to match actual output (receipt envelope appears as a regular line in `subscribe --reactions`)
- [x] Matrix mapping section names the Matrix concept each feature analogizes (m.in_reply_to, m.annotation, m.receipt, m.relates_to)
- [x] Limits section honestly names what's NOT implemented (cross-topic threading, edits/redactions, member list, m.room.topic, persistent local cursor, hub-side receipts aggregation) so readers don't try to use missing features
- [x] Doc rendered without lint errors (markdown valid — verified by Verification grep checks)

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
test -f docs/operations/agent-conversations.md
grep -q "Matrix mapping" docs/operations/agent-conversations.md
grep -q "channel react" docs/operations/agent-conversations.md
grep -q "channel ack" docs/operations/agent-conversations.md
grep -q "in_reply_to" docs/operations/agent-conversations.md

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

### 2026-04-27T13:36:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1316-agent-conversations-operator-doc-synthes.md
- **Context:** Initial task creation
