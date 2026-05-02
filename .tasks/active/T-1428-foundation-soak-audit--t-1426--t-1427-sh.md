---
id: T-1428
name: "Foundation soak audit — T-1426 + T-1427 ship status (2-week check)"
description: >
  Scheduled audit fire date: 2026-05-14. Check whether T-1426 (deprecation print on legacy primitives) and T-1427 (termlink whoami + identity binding) have shipped, and gather T-1166 cut-readiness signal from any deprecation telemetry the picks may have produced. This is a foundation-soak sentinel — created at the same time as T-1425 inception RFC + T-1426/T-1427 captures so the system has a structural reminder to re-check that the pre-cut foundation actually got built.

status: captured
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-30T21:21:07Z
last_update: 2026-04-30T21:21:07Z
date_finished: null
---

# T-1428: Foundation soak audit — T-1426 + T-1427 ship status (2-week check)

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

### 2026-04-30T21:21:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1428-foundation-soak-audit--t-1426--t-1427-sh.md
- **Context:** Initial task creation

## Updates

### 2026-05-02T06:46:00Z — Mid-soak data point (12 days before formal audit fire)

**T-1426 (deprecation print on legacy primitives):** Status unchanged from prior sessions — implementation reviewed in commit history but no telemetry yet on production usage frequency. Refer to T-1432 fleet doctor `--legacy-usage` output once cron data accumulates.

**T-1427 (whoami + identity binding):** Live at 0.9.1693+ on .107 + .122. Strict-reject `-32014` on sender_id/pubkey mismatch enforced. Verified during T-1438 chat-arc rollout — both `framework-agent` (.107) and `ring20-management-agent` (.122) carry `identity_fingerprint` in remote_list metadata.

**T-1438 chat-arc soak — early indicator (6 days post-bus-launch):**
- `agent-chat-arc` topic: 54 posts, 2 senders.
  - `d1993c2c3ec44c94` (.107 framework-agent): 46 posts.
  - `9219671e28054458` (.122 ring20-management-agent): 2 posts.
- Topic description carries full T-1429.5/T-1430 invariant block (5 invariants).
- 1 read receipt: .107 acked through offset 37.
- 123 dm:* topics on .107 hub (heavy fleet activity); 26 contain self-fp; 23/26 unread (typical async DM backlog).

**Cut-readiness signal:** All three blockers from T-1438 fields ("Field-readiness matrix") are operator-gated, not protocol gaps:
1. .143 auth heal (T-1418)
2. .141 binary swap to 0.9.1702
3. .141 PATH wiring + identity registration

The T-1166 cut depends on (1) — auth heal must land before legacy event.broadcast can be retired without orphaning .143's chat-arc signal.

**Recommendation for the formal 2026-05-14 audit:** Compare these numbers against the same fields. If sender count is still 2 of 4 hosts (unchanged), .141 + .143 are still cold and the cut should be deferred. If sender count climbs to 3-4, the protocol has soaked successfully and T-1166 cut is safe.
