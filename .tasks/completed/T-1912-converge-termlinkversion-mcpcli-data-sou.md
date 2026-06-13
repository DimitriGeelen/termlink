---
id: T-1912
name: "Converge termlink_version MCP/CLI data source (T-1909 third-catch)"
description: >
  MCP returns crate Cargo.toml version (0.9.0/commit=unknown/target=unknown). CLI returns workspace build.rs git-derived (0.11.501/commit=8a1aafb0/target=x86_64-...). Both should report the same canonical version-source. Un-ignore parity_version when converged.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-1904, T-1909]
created: 2026-06-01T11:34:57Z
last_update: 2026-06-01T11:58:01Z
date_finished: 2026-06-01T12:43:36Z
---

# T-1912: Converge termlink_version MCP/CLI data source (T-1909 third-catch)

## Context

Operator-visible drift: an MCP client asking the termlink server "what
version am I talking to?" via the `termlink_version` tool gets back
`{"version":"0.9.0","commit":"unknown","target":"unknown"}` — the
`termlink-mcp` crate's own Cargo.toml (frozen at 0.9.0). The same query
via the CLI `termlink version --json` returns
`{"version":"0.11.501","commit":"8a1aafb0","target":"x86_64-unknown-linux-gnu"}`
— the workspace bin's `build.rs` git-derived metadata. Two different
ground-truths for "what version is running" from the same binary.

**Root cause:** `crates/termlink-cli/build.rs` injects
`CARGO_PKG_VERSION` (from `git describe --tags`), `GIT_COMMIT`, and
`BUILD_TARGET` as compile-time env vars consumed by
`crates/termlink-cli/src/main.rs::Version`. The MCP server's
`termlink_version` tool in `crates/termlink-mcp/src/tools.rs` uses the
exact same `env!` / `option_env!` pattern, but `crates/termlink-mcp/`
has no `build.rs` — so the env vars are unset (commit/target) or fall
back to `termlink-mcp/Cargo.toml` (0.9.0 verbatim).

T-1909 v0.1's `parity_version` test caught this and is `#[ignore]`d
pending convergence. Detected via in-process MCP fixture + CLI assert
diff. Full diagnostic in `crates/termlink-mcp/tests/parity.rs:223-248`.

## Acceptance Criteria

### Agent
- [x] `crates/termlink-mcp/build.rs` exists and mirrors
      `crates/termlink-cli/build.rs` (CARGO_PKG_VERSION override from
      `git describe --tags`, GIT_COMMIT from `git rev-parse --short HEAD`,
      BUILD_TARGET from `$TARGET`). Same `cargo:rerun-if-changed=` paths.
- [x] `cargo build -p termlink --release` rebuilds the binary; running
      `target/release/termlink mcp-stdio` or invoking the MCP server
      in-process now returns matching `version`/`commit`/`target` values
      from both `termlink_version` (MCP) and `termlink version --json`
      (CLI).
- [x] `parity_version` test in `crates/termlink-mcp/tests/parity.rs`
      is un-ignored (`#[ignore]` attribute removed; in-source diagnostic
      comment updated to note convergence date and the structural fix).
- [x] `cargo test --release --test parity -p termlink-mcp --
      --test-threads=1` exits 0 with `test result: ok. 3 passed; 0 failed;
      2 ignored` (was: 2 passed; 0 failed; 3 ignored — version now passes
      parity).
- [x] No regression of `parity_hub_status` or `parity_negative_self_test`
      (still pass). Other ignored tests stay ignored — T-1910 and T-1911
      are independent fixes.

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

test -f crates/termlink-mcp/build.rs
grep -q "GIT_COMMIT" crates/termlink-mcp/build.rs
grep -q "BUILD_TARGET" crates/termlink-mcp/build.rs
grep -q "CARGO_PKG_VERSION" crates/termlink-mcp/build.rs
cargo test --release --test parity -p termlink-mcp -- --test-threads=1 2>&1 | tail -2 | grep -qE "test result: ok\. 3 passed; 0 failed; 2 ignored"

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-06-01T11:34:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1912-converge-termlinkversion-mcpcli-data-sou.md
- **Context:** Initial task creation

### 2026-06-01T11:42:27Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
