---
id: T-1305
name: "T-1304 follow-up: fabric card enrichment + minor polish"
description: >
  T-1304 follow-up: fabric card enrichment + minor polish

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-27T11:19:27Z
last_update: 2026-04-27T11:20:21Z
date_finished: 2026-04-27T11:20:21Z
---

# T-1305: T-1304 follow-up: fabric card enrichment + minor polish

## Context

T-1304 introduced `crates/termlink-hub/src/rpc_audit.rs`. `fw fabric register` created a stub card; this task enriches it (purpose, tags, edges) so PL-086 (recurring fabric drift) doesn't fire on this file.

## Acceptance Criteria

### Agent
- [x] `.fabric/components/crates-termlink-hub-src-rpc_audit.yaml` enriched with: real `purpose` line describing T-1304 audit-log behaviour, tags including `T-1304/T-1166/telemetry/observability`, `depended_by` edge to `server.rs`
- [x] No regression: `cargo build -p termlink-hub` and `cargo test -p termlink-hub rpc_audit` still green

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

grep -q "T-1304" .fabric/components/crates-termlink-hub-src-rpc_audit.yaml
grep -q "telemetry" .fabric/components/crates-termlink-hub-src-rpc_audit.yaml
cargo build -p termlink-hub 2>&1 | tail -3 | grep -qE "Finished"

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

### 2026-04-27T11:19:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1305-t-1304-follow-up-fabric-card-enrichment-.md
- **Context:** Initial task creation

### 2026-04-27T11:20:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
