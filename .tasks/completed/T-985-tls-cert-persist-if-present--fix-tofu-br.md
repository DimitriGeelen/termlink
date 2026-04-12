---
id: T-985
name: "TLS cert persist-if-present — fix TOFU breakage on hub restart"
description: >
  TLS cert persist-if-present — fix TOFU breakage on hub restart

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-hub/src/server.rs, crates/termlink-hub/src/tls.rs]
related_tasks: []
created: 2026-04-12T17:18:38Z
last_update: 2026-04-12T17:21:37Z
date_finished: 2026-04-12T17:21:37Z
---

# T-985: TLS cert persist-if-present — fix TOFU breakage on hub restart

## Context

T-945 GO. Hub regenerates TLS cert on every restart, breaking client TOFU. Apply T-933 persist-if-present pattern to certs. See `docs/reports/T-984-fw-upgrade-local-patch-reversion.md` for T-945 inception.

## Acceptance Criteria

### Agent
- [x] `tls.rs`: `load_or_generate_cert()` loads existing cert+key from disk if valid, else generates new
- [x] `tls.rs`: Invalid existing certs trigger regeneration with warning log
- [x] `tls.rs`: `cleanup()` retained but no longer called on normal shutdown (doc updated)
- [x] `server.rs`: Both call sites use `load_or_generate_cert()` instead of `generate_and_write_cert()`
- [x] `server.rs`: `tls::cleanup()` call removed from shutdown path
- [x] `cargo build --workspace` succeeds
- [x] `cargo test --package termlink-hub` passes (179/179)
- [x] New tests: `load_existing_cert_persists` + `invalid_existing_cert_triggers_regeneration`

## Verification

# Shell commands that MUST pass before work-completed. One per line.
cargo build --workspace --quiet
cargo test --package termlink-hub -- --quiet
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

### 2026-04-12T17:18:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-985-tls-cert-persist-if-present--fix-tofu-br.md
- **Context:** Initial task creation

### 2026-04-12T17:21:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** 179/179 tests pass, all agent ACs verified
