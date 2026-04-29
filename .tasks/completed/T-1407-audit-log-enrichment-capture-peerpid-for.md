---
id: T-1407
name: "Audit log enrichment: capture peer_pid for Unix-socket dispatch (T-1166 forensics)"
description: >
  Audit log enrichment: capture peer_pid for Unix-socket dispatch (T-1166 forensics)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-29T20:36:28Z
last_update: 2026-04-29T20:44:47Z
date_finished: 2026-04-29T20:44:47Z
---

# T-1407: Audit log enrichment: capture peer_pid for Unix-socket dispatch (T-1166 forensics)

## Context

The T-1166 bake-window diagnosis (2026-04-29) hit the limit of the current
audit log: 3145 anonymous `inbox.status` calls were attributable only by
their 60s cadence + empty params. The hub already extracts peer
credentials at connect time (`PeerCredentials::from_tokio_stream` in
`server.rs::run`) for the same-UID check, then discards them. Threading
`peer_pid` into the per-line audit record + the legacy-method warn log
makes future incidents identifiable in one `ps -p <pid>` call.

Predecessor: T-1304 (audit-log surface), T-1309 (`from` field), T-1311
(legacy warn). Caveat from PL-088: prefer CLI-side introspection where it
covers the use case. Here it does NOT — non-TermLink callers (raw JSON-RPC
shells, third-party tools) are exactly what we cannot identify today, and
they're hub-side-only by definition.

Schema change is **additive** — `peer_pid` is an Optional new field on the
JSONL record. Existing readers (`fw metrics api-usage`) ignore it; no
migration needed. TCP/TLS connections have no peer_pid (they are
network-remote); audit lines for those omit the field, same as a missing
`from`.

## Acceptance Criteria

### Agent
- [x] `handle_connection` signature gains a `peer_pid: Option<u32>`
      parameter and threads it into the dispatch loop. All 3 call sites
      (Unix, TCP+TLS, TCP no-TLS) pass `Some(pid)` for Unix, `None` for
      TCP.
- [x] `rpc_audit::record(method, from, peer_pid)` signature updated;
      writes `"peer_pid": <u32>` to the JSONL line when provided, omits
      when None or 0. Output is valid JSON in both cases. Implemented
      via new `build_audit_line()` helper.
- [x] `rpc_audit::warn_if_legacy(method, from, peer_pid)` includes
      `peer_pid` in the structured tracing fields when present.
- [x] Existing rpc_audit unit tests updated to the new signatures and
      pass; 3 new tests assert peer_pid handling
      (`line_with_peer_pid_includes_field`,
      `line_with_from_and_peer_pid_includes_both`,
      `line_with_peer_pid_zero_omits_field`).
- [x] `cargo build -p termlink-hub` green; `cargo test -p termlink-hub`
      reports 284 passed + 3 integration passed.
- [x] **Live verification:** binary 0.9.1579 installed, hub restarted
      (PID 713361), `termlink event broadcast t1407.test --targets
      nonexistent-target` invoked. Audit log line:
      `{"ts":1777495453165,"method":"event.broadcast","from":"tl-t1407-test","peer_pid":723266}`.
      Hub journal warn:
      `WARN termlink_hub::rpc_audit: deprecated primitive called —
      T-1166: schedule retirement once legacy <1% over 60d
      method=event.broadcast from=tl-t1407-test peer_pid=Some(723266)`.

### Decisions
- **Hub-side capture, not CLI-side.** PL-088 advises preferring CLI-side
  introspection for caller PID. That advice covers cases where the
  caller-process info is needed for *the caller's own logic*. Here the
  hub is the only party that can identify *non-TermLink* callers (raw
  JSON-RPC shells, third-party tools) — exactly the blind spot diagnosed
  during the T-1166 bake. CLI-side introspection cannot help; hub-side
  is the only viable surface.
- **Additive schema, no migration.** Added `peer_pid` as an optional
  JSONL field. `fw metrics api-usage` ignores unknown keys, so no
  agent-side change is required. Existing rolling 60d data stays
  unenriched (history before this commit had no peer_pid); new entries
  carry it forward. Reading code that needs peer_pid can branch on its
  presence.
- **Pid 0 treated as absent.** Defensive: getsockopt sometimes returns 0
  for kernel-side connections. `build_audit_line` omits the field for
  `Some(0)` to keep "absent" semantics consistent with `None`.

## Verification

cargo build -p termlink-hub
cargo test -p termlink-hub --test no_legacy_callers --no-fail-fast
cargo test -p termlink-hub rpc_audit::tests --no-fail-fast 2>&1 | tail -5
grep -q "peer_pid" crates/termlink-hub/src/rpc_audit.rs
grep -q "peer_pid" crates/termlink-hub/src/server.rs

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

### 2026-04-29T20:36:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1407-audit-log-enrichment-capture-peerpid-for.md
- **Context:** Initial task creation

### 2026-04-29T20:44:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
