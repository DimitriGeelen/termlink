---
id: T-925
name: "termlink ping --target — first --target rollout"
description: >
  termlink ping --target — first --target rollout

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-11T21:13:35Z
last_update: 2026-04-11T21:13:35Z
date_finished: null
---

# T-925: termlink ping --target — first --target rollout

## Context

First per-command rollout from T-921 Spike 4 decomposition. T-923 proved the
hub forwarder path end-to-end; T-924 shipped `TargetOpts` + `call_session` in
`crates/termlink-cli/src/target.rs`. T-925 wires `termlink ping` through the
helper so `termlink ping my-sess --target host:4112 --secret-file …` works.

While wiring the first consumer, a naming collision surfaces: `Ping { target }`
historically means **session**, but `TargetOpts.target` means **hub**. Rename
`TargetOpts.target → TargetOpts.hub` (Rust field only — CLI flag stays
`--target` per T-921 decision, via `#[arg(long = "target")]`). This unblocks
every downstream T-926..T-935 consumer.

No breaking UX change: the existing `termlink ping <session>` positional stays.
Four new flags (`--target`, `--secret-file`, `--secret`, `--scope`) appear on
ping; when `--target` is set the helper routes via the hub forwarder,
otherwise the existing `manager::find_session + client::rpc_call` local path
is preserved unchanged.

## Acceptance Criteria

### Agent
- [x] `TargetOpts` in `crates/termlink-cli/src/target.rs` renames the Rust
      field `target` → `hub` while the clap long flag stays `--target`
      (via `#[arg(long = "target")]`). All call sites and unit tests updated.
- [x] `Ping` variant in `cli.rs` gains four cross-host routing fields
      (`hub` / `--target`, `secret_file`, `secret`, `scope`) alongside the
      existing positional session argument.
- [x] `cmd_ping` in `commands/session.rs` routes through `call_session`,
      which handles both the cross-host (hub) and local (unix-socket) paths.
- [x] `cmd_ping` reports latency for both paths using the same output format
      (text + JSON) so scripts keep working.
- [x] `cargo build --workspace` clean, no new warnings.
- [x] `cargo test -p termlink --bin termlink -- target::` still passes
      (rename regression guard): 20/20.
- [x] `termlink ping --help` shows the four new flags and still shows the
      positional session.
- [x] Existing ping integration tests pass (cli_ping_session, cli_ping_json_output,
      cli_ping_with_timeout — 3/3).
- [x] Smoke-tested against a real local session: `termlink ping t1109-l006-sweep`
      returns `PONG from tl-vvfixptj (t1109-l006-sweep) — state: ready, latency: 18ms`.

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

cargo build --workspace
cargo test -p termlink --bin termlink -- target::
./target/debug/termlink ping --help 2>&1 | grep -q -- --target
./target/debug/termlink ping --help 2>&1 | grep -q -- --secret-file

## Decisions

### 2026-04-11 — Rename TargetOpts.target → TargetOpts.hub; CLI flag stays --target

- **Chose:** Rename the Rust field only. The CLI surface stays `--target
  HOST:PORT` per the T-921 inception decision. Clap routes via
  `#[arg(long = "target")]` so the flag name is decoupled from the field.
- **Why:** Every session-scoped command that will roll out `--target` in
  T-926..T-935 already has a `target` field meaning "which session to act on"
  (Ping, Status, Kv, Inject, …). Two `target` fields in one clap variant is
  a Rust error. Renaming TargetOpts's field to `hub` resolves the collision
  without breaking the T-921 CLI promise or touching the existing positional
  session args on any command.
- **Rejected:**
  1. Rename the CLI flag to `--hub HOST:PORT`. Simpler Rust, but it backs
     out a T-921 inception decision that was already reviewed and GO'd by the
     human. Would also mean every consumer task has to learn a new name
     than what the inception report said.
  2. Rename every command's existing `target` positional to `session`. Huge
     blast radius, breaks every script using positional invocation, and
     surfaces a UX change that had nothing to do with cross-host routing.

### 2026-04-11 — Route every ping through call_session (local path included)

- **Chose:** `cmd_ping` always calls `call_session(opts, "termlink.ping", …)`.
  The local vs. cross-host branch lives inside `call_session`, not inside
  `cmd_ping`. Ping only owns the presentation layer (latency measurement,
  text/JSON formatting, timeout handling).
- **Why:** Keeps the two code paths unified so a future bug fix or
  instrumentation change lands in one place. T-923 already proved the
  forwarder is transparent at the protocol level, so the local path is
  *exactly* what the cross-host path becomes once it reaches the session —
  using `call_session` for both is the right abstraction level.
- **Rejected:** Inlining two separate paths inside `cmd_ping` (one for local,
  one for cross-host). Doubles the test surface, invites divergence, and
  makes the next T-926 rollout copy a more complex template.

### 2026-04-11 — `--target` requires explicit session (no remote picker)

- **Chose:** When `--target HOST:PORT` is set on `ping`, the positional
  session argument becomes mandatory — error out if omitted.
- **Why:** The local-only interactive picker walks the local FS session
  registry; over TCP we would need to do a remote `session.discover` before
  the ping, which turns a single RPC into two, doubles connection cost, and
  adds an interactive prompt to a flag-driven path. Not worth the
  complexity for the first rollout — users who want discovery can run
  `termlink remote list --hub HOST:PORT` first.
- **Rejected:** Running `session.discover` on the remote hub and prompting
  the user. Defer to a later enhancement if real usage demands it.

## Updates

### 2026-04-11T21:13:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-925-termlink-ping---target--first---target-r.md
- **Context:** Initial task creation
