---
id: T-1911
name: "Converge termlink_ping MCP/CLI transport path (T-1909 second-catch)"
description: >
  MCP termlink_ping reaches session via in-process lookup; CLI termlink ping routes through hub and times out without one. Either align MCP to use hub-routing, or give CLI an in-process fallback when hub absent. Un-ignore parity_ping when converged.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-1904, T-1909]
created: 2026-06-01T11:34:57Z
last_update: 2026-06-01T12:56:45Z
date_finished: null
---

# T-1911: Converge termlink_ping MCP/CLI transport path (T-1909 second-catch)

## Context

T-1909 v0.1's `parity_ping` test fails because:

- **MCP `termlink_ping`** (in-process within the test) succeeds against
  the local session created by `start_session()` in the test fixture.
- **CLI `termlink ping <name> --json`** (separate subprocess) returns
  `{"ok":false,"error":"Ping timed out after 5s","latency_ms":5030,...}`
  even though it points at the same TERMLINK_RUNTIME_DIR.

**Surface analysis (NOT yet root-caused):**

Both code paths look semantically identical:

- MCP (`crates/termlink-mcp/src/tools.rs:7839`):
  `manager::find_session(target)` → `client::rpc_call(socket_path, "termlink.ping", {})`
- CLI (`crates/termlink-cli/src/commands/session.rs:643` →
  `crates/termlink-cli/src/target.rs:188`): with `opts.hub = None`,
  takes the local path:
  `manager::find_session(opts.session)` → `client::rpc_call(socket_path, "termlink.ping", {})`

The local path does NOT route through a hub when `--hub` is absent.
So the "MCP uses in-process, CLI routes through hub" hypothesis in the
T-1909 ignore comment is wrong. The real failure mode is unknown.

**Working hypotheses for the timeout:**

1. The test's tokio accept-loop fixture handles in-process callers
   (MCP) but blocks/stalls on cross-process unix-socket callers
   (CLI subprocess connecting to the same socket).
2. CLI subprocess inherits a different runtime/env that causes
   `find_session` to look in a different directory than MCP.
3. `client::rpc_call` framing or protocol-version handshake differs
   between in-process and subprocess callers in a way that the test
   fixture's accept-loop can't satisfy.
4. The `start_session` fixture only services one connection then
   panics/exits (race between MCP and CLI calls in same test).

**Detection evidence:**
`crates/termlink-mcp/tests/parity.rs::parity_ping` reproduces the
divergence deterministically. Test is `#[ignore]`d pending this work.

## Acceptance Criteria

### Agent
- [ ] Root-cause the timeout via direct experiment: run `termlink ping`
      against a manually-started session in the same TERMLINK_RUNTIME_DIR
      where `termlink_ping` MCP also succeeds, BOTH from a fresh shell
      AND from inside an integration test fixture. Document which side
      of the in-process/subprocess split actually fails and why in the
      `## Decisions` section.
- [ ] Fix the root cause OR document why the divergence is intentional
      (and remove `parity_ping` from the harness rather than leaving it
      ignored forever).
- [ ] `parity_ping` test in `crates/termlink-mcp/tests/parity.rs` is
      un-ignored (or removed). In-source diagnostic comment updated
      with the actual root cause + resolution.
- [ ] `cargo test --release --test parity -p termlink-mcp --
      --test-threads=1` exits 0 with `test result: ok. 5 passed; 0 failed;
      0 ignored` (was: 4 passed; 0 failed; 1 ignored — ping converged or
      removed) OR `test result: ok. 4 passed; 0 failed; 0 ignored` if
      the test is removed.
- [ ] No regression of any other parity test.

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

cargo test --release --test parity -p termlink-mcp -- --test-threads=1 2>&1 | tail -2 | grep -qE "test result: ok\. [45] passed; 0 failed; 0 ignored"

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
- **Output:** /opt/termlink/.tasks/active/T-1911-converge-termlinkping-mcpcli-transport-p.md
- **Context:** Initial task creation

### 2026-06-01T12:56:45Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
