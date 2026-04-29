---
id: T-1397
name: "Cross-hub mention + subscribe streaming flow — proves notifications + live tail"
description: >
  Cross-hub mention + subscribe streaming flow — proves notifications + live tail

status: work-completed
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T21:48:34Z
last_update: 2026-04-28T21:50:26Z
date_finished: 2026-04-28T21:50:26Z
---

# T-1397: Cross-hub mention + subscribe streaming flow — proves notifications + live tail

## Context

Closes the last two primitive surfaces in the arc: `--mention` (Matrix `m.mention`, T-1325) and `channel subscribe --follow` streaming (live tail). Validates that:
1. mentions cross-hub: bob (.122) mentions alice in a post; alice's subscribe filter on `metadata.mentions=alice` finds it
2. streaming cross-hub: a `subscribe --follow` from .107 receives posts emitted from .122 within 5s wall-clock

## Acceptance Criteria

### Agent
- [x] `tests/e2e/cross-hub-mention-stream-flow.sh` exists, executable
- [x] Topic created on .107; bob (.122) emits a post with `--mention alice`
- [x] Alice subscribes from .107 and finds bob's mention envelope by walking metadata.mentions
- [x] `subscribe --follow` started in background BEFORE posts; receives at least 3 posts pushed during the follow window
- [x] Streaming receives at least 1 cross-hub post (from .122)
- [x] Follow process is killed cleanly after capture (no zombie)
- [x] Script exits 0 with `MENTION-STREAM E2E PASSED` marker
- [x] Suite still green with the new script wired in
- [x] Work committed with task reference

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

test -x tests/e2e/cross-hub-mention-stream-flow.sh
out=$(BIN=./target/release/termlink ./tests/e2e/cross-hub-mention-stream-flow.sh 2>&1) && echo "$out" | grep -q "MENTION-STREAM E2E PASSED"
grep -q "cross-hub-mention-stream-flow.sh" tests/e2e/arc-suite.sh

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

### 2026-04-28T21:48:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1397-cross-hub-mention--subscribe-streaming-f.md
- **Context:** Initial task creation

### 2026-04-28T21:50:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
