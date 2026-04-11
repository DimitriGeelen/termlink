---
id: T-924
name: "Shared TargetOpts + call_session CLI helper for cross-host routing"
description: >
  T-921 Spike 3/4 prereq 2 of 2: create cli/src/target.rs with TargetOpts (derived via clap::Args so it composes into any command), a call_session(opts, method, params) async helper that routes through connect_remote_hub + session.forward (when --target set) or client::rpc_call + manager::find_session (when not), and secret-file lookup from ~/.termlink/secrets/<host>.hex. Depends on T-923. Tests: unit tests for validation paths (mirror T-919's pattern for connect_remote_hub).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-11T20:34:01Z
last_update: 2026-04-11T20:51:03Z
date_finished: null
---

# T-924: Shared TargetOpts + call_session CLI helper for cross-host routing

## Context

T-921 Spike 3 picked routing option γ (hub-as-forwarder). T-923 discovered the hub router already transparently forwards any non-hub-local method via `forward_to_target` at `crates/termlink-hub/src/router.rs:1171`, so no new hub RPC method is needed. T-924 is the CLI side: one shared module that every session-scoped command in T-925..T-935 will call instead of hand-rolling `client::rpc_call(reg.socket_path(), ...)`.

Shape:
- New file `crates/termlink-cli/src/target.rs`
- `TargetOpts` struct derived via `clap::Args` so commands can flatten it into their existing arg structs
- Fields: `target: Option<String>` (HOST:PORT), `secret_file: Option<PathBuf>`, `secret: Option<String>`, `scope: Option<String>` (defaults per method), `session: String` (target session ID or display name)
- `call_session(opts, method, params) -> Result<Value>` async helper:
  - If `opts.target.is_some()`: connect to the hub via `connect_remote_hub` (re-exporting from `commands::remote`) and `rpc.call(method, {target: opts.session, ...params})`
  - Else: `let reg = manager::find_session(&opts.session)?; client::rpc_call(reg.socket_path(), method, params).await`
- Implicit secret lookup from `~/.termlink/secrets/<host>.hex` when neither `--secret-file` nor `--secret` is given
- Unit tests mirroring T-919's pattern on `connect_remote_hub`: pure validation paths (no I/O) for secret resolution, missing args, default scope fallbacks

Depends on: T-923 (already complete at the mechanism level — forwarder exists).
Unblocks: T-925..T-935 (per-command rollouts of `--target`).

## Acceptance Criteria

### Agent
- [ ] New file `crates/termlink-cli/src/target.rs` exists, contains `TargetOpts` struct derived via `clap::Args` with the five fields above, and is wired into `cli.rs` via `mod target;` or via the existing module layout
- [ ] `TargetOpts::resolve_secret()` helper returns `Ok(Vec<u8>)` with 32 bytes on success, walking the precedence order: explicit `--secret` hex > `--secret-file` path > `~/.termlink/secrets/<host>.hex` (only when `--target` set) > error if required
- [ ] `call_session(opts, method, params)` routes cross-host when `opts.target.is_some()` and local when not, reusing `commands::remote::connect_remote_hub` for cross-host
- [ ] Unit tests cover: (1) explicit secret hex precedence; (2) secret-file precedence when no explicit; (3) default file lookup when only `--target` set; (4) missing secret error when required; (5) invalid hex rejected; (6) invalid host:port rejected; (7) unknown scope rejected; (8) session resolution happens only on the local branch (not when `--target` set). Minimum 8 tests, all pass in < 50ms
- [ ] `cargo test -p termlink -- target::` passes (exact package name and filter TBD — verify via `cargo test -p termlink --lib -- target` and adjust)
- [ ] `cargo build --workspace` clean, no new warnings from target.rs
- [ ] No behavioural change to any existing command (this task only adds the helper; T-925+ wires it in)

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

### 2026-04-11T20:34:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-924-shared-targetopts--callsession-cli-helpe.md
- **Context:** Initial task creation

### 2026-04-11T20:51:03Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
