---
id: T-1057
name: "build.rs: make binary version update on every commit (robust rerun-if-changed)"
description: >
  build.rs: make binary version update on every commit (robust rerun-if-changed)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-14T20:34:57Z
last_update: 2026-04-14T20:34:57Z
date_finished: null
---

# T-1057: build.rs: make binary version update on every commit (robust rerun-if-changed)

## Context

User observed: `termlink --version` reports `0.9.860`, but `git describe --tags`
says we are at `v0.9.0-865-g4c520d7a`. Expected version: `0.9.865`. Binary is
5 commits stale.

Root cause diagnosis (verified with `stat`):
`crates/termlink-cli/build.rs` declares:
```
println!("cargo:rerun-if-changed=../../.git/HEAD");
println!("cargo:rerun-if-changed=../../.git/refs/tags");
```
These paths never change on a normal commit:
- `.git/HEAD` contains `ref: refs/heads/main` — a symbolic ref. The file is
  only rewritten on `git switch`. Local mtime: 2026-03-16 (a month old).
- `.git/refs/tags/` is a directory, only bumped when tagging.

On every commit, git updates `.git/logs/HEAD` (reflog, always appended) and
`.git/refs/heads/<branch>` (the actual ref). Cargo sees no change in the
watched paths → never reruns build.rs → cached version persists forever.

The three version surfaces, for clarity:
1. `Cargo.toml workspace.version = "0.9.0"` — FLOOR value, only used if
   build.rs can't run (e.g., crates.io tarball build). Overridden by
   `CARGO_PKG_VERSION` env from build.rs when git is available.
2. `VERSION` file — stamped by the pre-push git hook
   (`.agentic-framework/agents/git/lib/hooks.sh:386`). Updates on push,
   not on local commits. Acceptable — audit scripts run on pushed state.
3. Binary version (`termlink --version`) — derived by build.rs from
   `git describe`. THIS is what's stuck, and what this task fixes.

The permanent fix watches `.git/logs/HEAD` (reliably appended on every
HEAD-moving operation) and `.git/refs/heads/` (directory, bumps when
any local branch commit lands), in addition to the existing paths.
`.git/packed-refs` is also watched for the rare case of a fresh gc.

## Acceptance Criteria

### Agent
- [x] `crates/termlink-cli/build.rs` adds `rerun-if-changed` for `.git/logs/HEAD`, `.git/refs/heads/`, and `.git/packed-refs`
- [x] Retains existing watches on `.git/HEAD` and `.git/refs/tags` (still meaningful for `git switch` and `git tag` triggers)
- [x] Comment block documents WHY each watch path matters (so the next agent to edit this file doesn't re-introduce the bug)
- [x] Mid-fix verification: clean rebuild → version jumped 0.9.860 → 0.9.865 (current state as of HEAD)
- [x] `cargo build -p termlink` clean, no clippy warnings
- [x] `cargo test -p termlink --bin termlink`: 189 tests pass
- [x] Graceful fallback preserved: `rerun-if-changed` on missing paths is tolerated by cargo; `git describe` failure path still falls back to Cargo.toml workspace version via the existing `if let Some(...)` guard
- [x] Round-trip verified post-commit: see VERIFICATION below

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

cargo build -p termlink 2>&1 | tail -3
cargo test -p termlink --bin termlink 2>&1 | grep -E "189 passed"
grep -q "cargo:rerun-if-changed=../../.git/logs/HEAD" crates/termlink-cli/build.rs

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

### 2026-04-14T20:34:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1057-buildrs-make-binary-version-update-on-ev.md
- **Context:** Initial task creation
