---
id: T-1214
name: "Fleet termlink diagnosis (T-1210 S1) + converge-vs-federate decision"
description: >
  Execute T-1210 S1: probe every reachable peer for termlink binary lineage (version, subcommand list, mtime, source path if discoverable). Classify as same-lineage-older / same-lineage-newer / forked / stranger. Preliminary finding from T-1210 probe: .122 has 0.9.844 install with no channel subcommand and no /opt/termlink source → stranger lineage. After S1 complete, produce converge-vs-federate recommendation. Pilot S2 (unified install) or S3 (capability probe) depending on direction. See .tasks/completed/T-1210-fleet-termlink-version-divergence--unifi.md.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: [fleet, install, capability-probe]
components: []
related_tasks: [T-1210, T-1165, T-1168]
created: 2026-04-24T10:05:17Z
last_update: 2026-04-24T10:05:17Z
date_finished: null
---

# T-1214: Fleet termlink diagnosis (T-1210 S1) + converge-vs-federate decision

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

### 2026-04-24T10:05:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1214-fleet-termlink-diagnosis-t-1210-s1--conv.md
- **Context:** Initial task creation
