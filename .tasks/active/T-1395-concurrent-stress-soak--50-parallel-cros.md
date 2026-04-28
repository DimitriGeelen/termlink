---
id: T-1395
name: "Concurrent stress soak — 50 parallel cross-hub posts; verify offset linearization + zero loss"
description: >
  Concurrent stress soak — 50 parallel cross-hub posts; verify offset linearization + zero loss

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T21:42:52Z
last_update: 2026-04-28T21:42:52Z
date_finished: null
---

# T-1395: Concurrent stress soak — 50 parallel cross-hub posts; verify offset linearization + zero loss

## Context

T-1390..T-1394 validated correctness under 6-12 parallel posts. The arc claims to handle realistic concurrent load. This task adds `tests/e2e/cross-hub-stress-soak.sh` with N=50 parallel posters (40 local on .107 + 10 cross-hub from .122) hitting one topic, plus a fanout phase (5 topics × 10 senders each in parallel). Proves:
- offset linearization (every post gets a unique monotonic offset)
- zero message loss (count of posts == count of envelopes in canonical state)
- cross-hub TCP holds under concurrent load
- arc-suite remains stable after stress (re-run confirms no leftover state breakage)

## Acceptance Criteria

### Agent
- [x] `tests/e2e/cross-hub-stress-soak.sh` exists, executable
- [x] Single-topic phase: 50 parallel posts (40 .107 local + 10 cross-hub from .122) — all land
- [x] Verify: count of envelopes == 50 (zero loss)
- [x] Verify: all 50 offsets are 0..49 contiguous (no gaps, no duplicates)
- [x] Verify: each cross-hub post is attributed correctly
- [x] Fanout phase: 5 topics × 10 senders/topic in parallel — all land
- [x] Verify: each of the 5 topics has exactly 10 envelopes
- [x] Soak completes in < 30s wall-clock (actual: 10s)
- [x] After stress, the arc-suite (T-1393) re-runs green
- [x] Script exits 0 with `STRESS-SOAK E2E PASSED` marker
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

test -x tests/e2e/cross-hub-stress-soak.sh
out=$(BIN=./target/release/termlink ./tests/e2e/cross-hub-stress-soak.sh 2>&1) && echo "$out" | grep -q "STRESS-SOAK E2E PASSED"

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

### 2026-04-28T21:42:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1395-concurrent-stress-soak--50-parallel-cros.md
- **Context:** Initial task creation
