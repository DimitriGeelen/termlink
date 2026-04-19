---
id: T-1131
name: "Wire protocol_version enforcement at hub — structured error instead of opaque serde parse failure (from T-1071 GO)"
description: >
  From T-1071 inception GO. Hub records each registered session's declared protocol_version (Capabilities.protocol_version: u8, already on wire at control.rs:79 but zero enforcement). On RPC call from a session whose declared version < hub's DATA_PLANE_VERSION for that method, return structured error PROTOCOL_VERSION_TOO_OLD with min required version, instead of letting serde fail with opaque parse error. Backwards-compatible: missing field defaults to 1. This converts the KeyEntry-style silent failures into actionable 'upgrade your client' messages. Load-bearing fix of the three T-1071 follow-ups.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [protocol, termlink, version-skew, T-1071]
components: []
related_tasks: []
created: 2026-04-18T22:59:37Z
last_update: 2026-04-19T14:02:30Z
date_finished: null
---

# T-1131: Wire protocol_version enforcement at hub — structured error instead of opaque serde parse failure (from T-1071 GO)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] New error code `PROTOCOL_VERSION_TOO_OLD = -32011` defined in `termlink-protocol::control::error_code`
- [x] `Capabilities` (control.rs) gains `#[serde(default = "default_protocol_version")]` so missing fields default to 1 (backward compatible)
- [x] New helper `check_protocol_version(id, declared, required, method) -> Option<ErrorResponse>` returns `Some(PROTOCOL_VERSION_TOO_OLD)` structured error with `{declared, required, method}` data when declared < required, else `None`
- [x] `RemoteEntry` (remote_store.rs) records `protocol_version: u8` captured at registration time (default 1)
- [x] `handle_register_remote` parses optional `protocol_version` from params and forwards it to the store
- [x] Unit test: helper returns structured error with correct shape when `declared < required`
- [x] Unit test: helper returns None when `declared >= required` (two variants: equal and greater)
- [x] Unit test: `Capabilities` deserializes with missing `protocol_version` field (backward-compat guarantee)
- [x] `cargo test -p termlink-protocol -p termlink-hub` passes (96 + 198 tests)

### Scope Fence
**IN:** scaffold (error code, helper, storage). **OUT:** per-method enforcement wiring on every RPC path — deferred to a follow-up task so each method group can add its min-version requirement with its own test; DATA_PLANE_VERSION is currently 1 so no method rejects today, but the rails are laid for the next bump.

### Human
- [ ] [RUBBER-STAMP] Confirm the scope fence is acceptable — follow-up task can plumb the check into each Tier-B handler
  **Steps:** review the commit, read the scope fence
  **Expected:** foundation looks right; follow-up is the correct next unit
  **If not:** call out which method you want plumbed now vs deferred

## Verification

grep -q "PROTOCOL_VERSION_TOO_OLD" /opt/termlink/crates/termlink-protocol/src/control.rs
grep -q "check_protocol_version" /opt/termlink/crates/termlink-protocol/src/control.rs
grep -q "protocol_version" /opt/termlink/crates/termlink-hub/src/remote_store.rs
bash -c 'cd /opt/termlink && cargo test -p termlink-protocol --lib 2>&1 | tail -3 | grep -qE "test result: ok"'

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

### 2026-04-18T22:59:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1131-wire-protocolversion-enforcement-at-hub-.md
- **Context:** Initial task creation

### 2026-04-19T14:02:30Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
