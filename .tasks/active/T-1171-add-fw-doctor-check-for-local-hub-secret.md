---
id: T-1171
name: "Add fw doctor check for local hub secret cache drift (G-011)"
description: >
  Add fw doctor check: compare mtime of each ~/.termlink/secrets/*.hex against the corresponding hub's authoritative secret file (where locally resolvable). Warn if cache is older. Also audit chmod 600 on all .hex files (proxmox4.hex currently 644 — security smell). Deliverable: one new check in agents/doctor/, test coverage, CLAUDE.md §Hub Auth Rotation Protocol updated with the 'read-live, not cache' rule.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-20T20:08:57Z
last_update: 2026-04-20T20:17:37Z
date_finished: null
---

# T-1171: Add fw doctor check for local hub secret cache drift (G-011)

## Context

G-011 root cause: `~/.termlink/secrets/<host>.hex` is a client-side cache of a
shared hub secret. When the hub restarts with a new secret (or migrates runtime
dir), that cache silently diverges — auth-mismatch errors surface on the client
but the giver's cache looks "valid" by eyeball. Mirror-image of T-1051
(receiving-side rotation drift).

PL-041 captured the rule: read the live authoritative runtime file when
sharing a secret, never the IP/host-keyed cache. This task adds structural
detection so doctor warns before the operator hits the auth wall.

Two checks, both additive in `cmd_doctor` at
`crates/termlink-cli/src/commands/infrastructure.rs`:

1. **perms audit** — every `~/.termlink/secrets/*.hex` must be mode 0600
   (not the `.bak` siblings). Surfaced during G-011 registration when
   `proxmox4.hex` was world-readable (644).
2. **freshness heuristic** — when a local hub is running, compare mtime of
   `<runtime_dir>/hub.secret` against each `.hex` cache file. Warn per
   cache file older than the local hub secret — the operator confirms the
   mapping.

## Acceptance Criteria

### Agent
- [ ] `cmd_doctor` in `crates/termlink-cli/src/commands/infrastructure.rs` gains a `secret_cache` check block
- [ ] Perms audit: scans `~/.termlink/secrets/*.hex` (excluding `*.bak`). For any file whose mode != 0o600, emits `warn` listing the path and actual mode
- [ ] Freshness heuristic: when `resolve_hub_paths().parent()/hub.secret` exists and is readable, compares mtime against each `.hex` cache file. For any cache older than the hub secret, emits `warn` listing the stale file(s) with both mtimes
- [ ] Missing `~/.termlink/secrets/` directory is a `pass` (no cache, nothing to drift) — not a warn
- [ ] JSON output (`--json`) includes the `secret_cache` check entries in the `checks` array with the same `{check, status, message}` shape as the existing checks
- [ ] Unit test: one test writes a tmp `.hex` with mode 0o644 and asserts the perms branch is taken (use existing test pattern in the crate; if none, table-test the helper function)
- [ ] `cargo build -p termlink-cli` succeeds
- [ ] `cargo test -p termlink-cli` succeeds (all existing tests + new test)
- [ ] `cargo clippy -p termlink-cli -- -D warnings` succeeds

### Human
- [ ] [RUBBER-STAMP] Run `termlink doctor` on this box and confirm the `secret_cache` check appears in the output
  **Steps:**
  1. `cd /opt/termlink && cargo build -p termlink-cli --release`
  2. `./target/release/termlink doctor`
  3. Look for a `secret_cache` line in the check list
  **Expected:** One `pass` or `warn` line prefixed with `secret_cache:`
  **If not:** Check `cmd_doctor` fallthrough — the check block may have silently skipped

## Verification

cargo build -p termlink-cli
cargo test -p termlink-cli
cargo clippy -p termlink-cli -- -D warnings
grep -q "secret_cache" crates/termlink-cli/src/commands/infrastructure.rs

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

### 2026-04-20T20:08:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1171-add-fw-doctor-check-for-local-hub-secret.md
- **Context:** Initial task creation

### 2026-04-20T20:17:37Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-04-20T20:17:37Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
