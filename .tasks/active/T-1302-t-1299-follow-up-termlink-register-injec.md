---
id: T-1302
name: "T-1299 follow-up: termlink register injects TERMLINK_SESSION_ID into spawned shell"
description: >
  Per T-1299 final unchecked AC item (a). When `termlink register --shell` spawns a child shell, inject `TERMLINK_SESSION_ID=<id>` into the child's environment between `fork()` and `execvp()`. Today the operator copies the id manually from `termlink list --json`. With this fix, the spawned shell can run `termlink whoami` immediately and have it auto-resolve. Reversible: additive env-var, default behavior unchanged for non-PTY callers (tests pass empty slice).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [termlink, routing, whoami, T-1299-followup, pty]
components: []
related_tasks: [T-1299, T-1297]
created: 2026-04-26T22:08:31Z
last_update: 2026-04-26T22:08:31Z
date_finished: null
---

# T-1302: T-1299 follow-up: termlink register injects TERMLINK_SESSION_ID into spawned shell

## Context

T-1299 (whoami MVP) shipped with `$TERMLINK_SESSION_ID` as the primary disambiguator, but the operator must still set it manually after each `termlink register --shell`. This closes the loop: registering with `--shell` causes the spawned child shell to inherit the env var automatically, so `termlink whoami` Just Works inside it. Item (a) of the three deferred follow-ups documented in T-1299's final AC.

`PtySession::spawn` uses raw `libc::fork()` + `libc::execvp()`. Env injection happens in the child branch by calling `libc::setenv("KEY", "VALUE", 1)` between fork and execvp. Existing call sites (tests, data_server) need no env and pass an empty slice.

## Acceptance Criteria

### Agent
- [x] `PtySession::spawn` accepts an `env: &[(String, String)]` parameter (or equivalent shape) and sets each pair in the child via `libc::setenv` after fork, before `execvp` — added `spawn_with_env(shell, scrollback, env)`; `spawn(shell, scrollback)` now delegates with empty env (back-compat)
- [x] `cmd_register --shell` passes `[("TERMLINK_SESSION_ID", session_id)]` as the env on spawn
- [x] All 7 existing `PtySession::spawn` call sites (cli + data_server tests + pty unit tests + integration test) pass an empty slice; no behavior change for them — kept the 2-arg `spawn` shim, all callers untouched
- [x] New unit test in `crates/termlink-session/src/pty.rs::tests`: spawn `/bin/sh` with env `[("TL_TEST_VAR", "hello")]`, write `echo "$TL_TEST_VAR"\n`, assert `hello` appears in scrollback within 500ms — `spawn_passes_env_to_child` (passes)
- [x] Workspace `cargo test --workspace --lib` clean — 311 termlink-session + 100 termlink-protocol + 5 hub tests, all green
- [x] Workspace `cargo clippy --all-targets -- -D warnings` clean

### Human
- [ ] [REVIEW] Smoke test: in a fresh shell, run `termlink register --shell --name test-1302` then in the spawned shell run `termlink whoami` — should auto-resolve without `--session`/`--name` flags
  **Steps:**
  1. Build current binary: `cd /opt/termlink && cargo build --release -p termlink`
  2. From a clean shell (no `TERMLINK_SESSION_ID` set): `target/release/termlink register --shell --name test-1302`
  3. Inside the spawned shell, run: `target/release/termlink whoami`
  4. Inside the spawned shell, also run: `echo "$TERMLINK_SESSION_ID"`
  **Expected:** `whoami` returns the test-1302 identity card without flags. `echo` prints a `tl-…` id.
  **If not:** Capture the env (`env | grep TERMLINK`) and the `whoami` output. Likely either fork-time env injection failed or the parent's env leaked through.

## Verification

cargo test -p termlink-session --lib pty::tests::spawn_passes_env_to_child --quiet 2>&1 | tail -3 | grep -qE "test result: ok"
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

### 2026-04-26T22:08:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1302-t-1299-follow-up-termlink-register-injec.md
- **Context:** Initial task creation

### 2026-04-26T22:15Z — env injection shipped [agent autonomous pass]
- **Signature:** `PtySession::spawn(shell, scrollback)` becomes a 2-arg back-compat shim that delegates to `spawn_with_env(shell, scrollback, &[])`. New 3-arg form pre-encodes pairs as `CString` in the parent, then in the child branch (between fork and execvp, alongside the existing `setsid`/`dup2` block) loops calling `libc::setenv(k, v, 1)`. Signal-safe: only `setenv` and `_exit`/`execvp` after fork.
- **Wiring:** `cmd_register` (PTY branch) now passes `[("TERMLINK_SESSION_ID", session.id().as_str().to_string())]`. Six other call sites (data_server tests, pty unit tests, integration test) untouched.
- **Test:** `pty::tests::spawn_passes_env_to_child` — spawns sh with `TL_TEST_VAR=hello-1302`, writes `echo VAR_IS=$TL_TEST_VAR`, asserts `VAR_IS=hello-1302` in scrollback. Green.
- **Verification:** workspace `cargo test --lib` 311+100+5/0 fail; workspace clippy clean.
- **Operator AC:** smoke test `register --shell` then `whoami` inside spawned shell — should auto-resolve without flags.
