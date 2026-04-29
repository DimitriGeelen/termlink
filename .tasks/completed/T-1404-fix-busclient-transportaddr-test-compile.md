---
id: T-1404
name: "Fix bus_client TransportAddr test compile breaks (T-1385 fallout)"
description: >
  Fix bus_client TransportAddr test compile breaks (T-1385 fallout)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-session/src/bus_client.rs, crates/termlink-session/tests/bus_client_integration.rs]
related_tasks: []
created: 2026-04-29T18:33:08Z
last_update: 2026-04-29T18:36:11Z
date_finished: 2026-04-29T18:36:11Z
---

# T-1404: Fix bus_client TransportAddr test compile breaks (T-1385 fallout)

## Context

`cargo test -p termlink-session --no-run` fails at 4 call sites that pass
`PathBuf` to `BusClient::connect_with_interval` after T-1385 changed its
signature to take `TransportAddr`. Surfaced during T-1166 workspace test
sanity check on 2026-04-29. Pre-existing; not introduced by T-1401/T-1403.

Sites:
- `crates/termlink-session/src/bus_client.rs` lines 274, 294, 320 (lib tests)
- `crates/termlink-session/tests/bus_client_integration.rs` line 119

Mechanical fix: wrap each `socket` arg in `TransportAddr::unix(socket)`.

## Acceptance Criteria

### Agent
- [x] `cargo test -p termlink-session --no-run` builds without compile errors
- [x] `cargo test -p termlink-session` passes (or shows the same pass count it did before T-1385 broke compilation) — 336 passed (314 unit + 1 bus_client_integration + 20 integration + 1 doctest)
- [x] No production code changed — only test wrapping; the change is purely TransportAddr boilerplate

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

grep -q "TransportAddr::unix(nonexistent_socket)" crates/termlink-session/src/bus_client.rs
grep -q "TransportAddr::unix(socket)" crates/termlink-session/src/bus_client.rs
grep -q "TransportAddr::unix(socket.clone())" crates/termlink-session/tests/bus_client_integration.rs
grep -q "use termlink_protocol::transport::TransportAddr" crates/termlink-session/tests/bus_client_integration.rs

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

### 2026-04-29T18:33:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1404-fix-busclient-transportaddr-test-compile.md
- **Context:** Initial task creation

### 2026-04-29T18:36:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
