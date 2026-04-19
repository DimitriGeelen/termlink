---
id: T-1138
name: "Enrich fabric cards — fix manifest.rs and inbox.rs metadata + re-run enricher"
description: >
  Enrich fabric cards — fix manifest.rs and inbox.rs metadata + re-run enricher

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-19T12:00:47Z
last_update: 2026-04-19T12:00:47Z
date_finished: null
---

# T-1138: Enrich fabric cards — fix manifest.rs and inbox.rs metadata + re-run enricher

## Context

`fw fabric enrich` left 5 cards without edges. Two of them (`manifest.rs`, `inbox.rs`) are skeleton cards with `type: script` and `subsystem: unknown` placeholders — the enricher skips them because it can't resolve their subsystem. Fixing their metadata and filling real purpose + reverse-edges brings the fabric's enriched-coverage from 83/88 to 85/88. The remaining 3 (`test_env_lock.rs`, `scripts/watchdog.sh`, `transcripts-agent`) are intentionally standalone — leaf nodes with no imports, documented as such.

## Acceptance Criteria

### Agent
- [x] `manifest.rs` card updated: type=module, subsystem=cli, real purpose, depended_by edges from dispatch.rs/infrastructure.rs/main.rs
- [x] `inbox.rs` card updated: type=module, subsystem=hub, real purpose, depended_by edges from router.rs/supervisor.rs/lib.rs
- [x] `fw fabric enrich` re-run (no regressions — card count stays 88, edge count grows)
- [x] Python YAML load passes on both edited cards

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

### 2026-04-19T12:00:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1138-enrich-fabric-cards--fix-manifestrs-and-.md
- **Context:** Initial task creation
