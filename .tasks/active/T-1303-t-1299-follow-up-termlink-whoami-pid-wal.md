---
id: T-1303
name: "T-1299 follow-up: termlink whoami PID-walk fallback when env var absent"
description: >
  T-1299 follow-up item (b). When `termlink whoami` has no flag and no `$TERMLINK_SESSION_ID`, walk /proc/self/stat ancestors and check each PID against `manager::list_sessions()` — return the closest match. CLI-side only: avoids plumbing peer_pid through JSON-RPC dispatch (the hub-side path matters only for cross-host `termlink remote call`, which is out of scope). Reversible: pure additive fallback in metadata.rs::cmd_whoami; existing flag/env paths unchanged. Estimate: ~½ dev-day.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [termlink, routing, whoami, T-1299-followup, cli]
components: []
related_tasks: [T-1299, T-1297, T-1302]
created: 2026-04-26T22:13:07Z
last_update: 2026-04-26T22:13:07Z
date_finished: null
---

# T-1303: T-1299 follow-up: termlink whoami PID-walk fallback when env var absent

## Context

T-1302 closed item (a) — `register --shell` now seeds `TERMLINK_SESSION_ID`. But operators sometimes run `whoami` from a process that never inherited that env var (sub-shell that re-exec'd, container exec that lost env, debugging session opened via `nsenter`). For those cases, walk the process tree on Linux: read `/proc/<pid>/stat` field 4 (ppid) up to PID 1, check each ancestor against the local session registry, return the closest match.

Why CLI-side and not hub-side: the CLI already does local-file lookup via `manager::list_sessions()` (no hub round-trip), so adding the walk in `cmd_whoami` is ~50 lines. Hub-side would need `peer_pid` plumbed through `RpcRequest` → `route()` → handler — invasive. Hub-side path only matters for `termlink remote call <profile> session.whoami` against a remote hub, where the remote hub's `peer_pid` would be the local SSH/relay process anyway, not the actual caller. Out of scope.

Linux-only: `/proc` parsing. Other platforms get the existing ambiguous-list fallback. macOS would need `libproc` or `sysctl(KERN_PROC)`; deferred until someone needs it.

## Acceptance Criteria

### Agent
- [x] New helper `walk_ancestor_pids(start: u32) -> Vec<u32>` in `crates/termlink-cli/src/commands/metadata.rs` (or a sibling util module) that returns the ancestor chain from `start` up to PID 1, by parsing `/proc/<pid>/stat` field 4. Stops on missing /proc or read error (non-Linux platforms returns just `[start]`).
- [x] `cmd_whoami` inserts a step between env-var check and ambiguous-list fallback: build ancestor list of `std::process::id()`, check each against `manager::list_sessions()`, return the FIRST session whose `pid` matches an ancestor (closest wins).
- [x] Successful PID-walk hit prints a one-line note in human mode (e.g. `(matched via PID-walk: ancestor pid=NNN)`) so the operator knows why the lookup succeeded without an explicit hint. JSON mode adds `"resolved_via": "pid_walk"`.
- [x] Five new unit tests in `commands::metadata::tests` covering: parse_ppid_from_stat (simple, comm-with-paren, malformed); walk_ancestor_pids (self chain, unknown pid). All green.
- [x] Workspace `cargo test --workspace --lib` clean — 230 termlink unit + 172 integration + 4 ignored
- [x] Workspace `cargo clippy --all-targets -- -D warnings` clean

### Human
- [ ] [REVIEW] Smoke test: from a session NOT spawned by `register --shell` (so no env var inherited), run `termlink whoami` — should still resolve via PID-walk
  **Steps:**
  1. Start a session: `termlink register --shell --name test-1303` (in shell A)
  2. From shell A's spawned shell, run a sub-shell: `bash -c 'unset TERMLINK_SESSION_ID; termlink whoami'`
  3. Or from a process started outside termlink that's a descendant of an existing registered session
  **Expected:** `whoami` resolves to the test-1303 session via PID-walk; output includes the `matched via PID-walk` note.
  **If not:** Capture `cat /proc/self/stat` and `termlink list --json` to compare PIDs.

## Verification

cargo test -p termlink --lib commands::metadata::tests::walk_ancestor_pids --quiet 2>&1 | tail -3 | grep -qE "test result: ok"
cargo build --workspace 2>&1 | tail -3 | grep -qE "Finished|Compiling"
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -3 | grep -qE "Finished|Checking"

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

### 2026-04-26T22:13:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1303-t-1299-follow-up-termlink-whoami-pid-wal.md
- **Context:** Initial task creation

### 2026-04-26T22:25Z — PID-walk fallback shipped [agent autonomous pass]
- **Helpers added:** `walk_ancestor_pids(u32) -> Vec<u32>` (1024-iter cap; cycle/dup detection); `read_ppid_from_proc(u32)` (reads /proc/<pid>/stat); `parse_ppid_from_stat(&str)` (rfind ')' to handle comm-with-paren, ppid is field 1 of post-')' split).
- **Wiring:** Inserted between env-var check and ambiguous-list. Builds chain from `std::process::id()`, scans `manager::list_sessions(false)` for any session whose pid is in the chain (closest wins).
- **Output annotation:** `print_whoami_card` extracted as helper; takes `Option<u32>` for the matched ancestor pid → adds `(matched via PID-walk: ancestor pid=NNN)` line in human mode and `"resolved_via": "pid_walk"` + `"pid_walk_match"` fields in JSON.
- **Tests:** 5 new in `commands::metadata::tests`. parse_ppid_from_stat: simple, comm-with-paren (verifies `rfind(')')` correctness), malformed-returns-none. walk_ancestor_pids: self-chain (no dups, starts with self), unknown-pid (returns just `[start]`).
- **Verification:** workspace tests 230+172+4-ignored/0 fail; clippy clean; live whoami still falls through to ambiguous list correctly when caller's process tree includes no registered session (verified against agent's own claude process — no ancestors are session pids, so ambiguous list shown as before).
- **Operator AC:** smoke test from a sub-shell of `register --shell` confirms PID-walk hits.
- **Item (b) of T-1299 deferred items closed.** Item (c) (cross-host forward-compat) remains; lower priority since CLI uses local-file lookup, not RPC, in the dominant flow.
