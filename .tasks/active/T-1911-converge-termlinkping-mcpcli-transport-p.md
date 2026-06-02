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
last_update: 2026-06-02T10:38:09Z
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
- [x] Root cause identified and recorded in `## Decisions`. The original
      transport-divergence hypothesis was wrong; the real cause was the
      test fixture's tokio runtime flavor.
- [x] `parity_ping` un-ignored. In-source comment block (parity.rs:141-156)
      replaces the original wrong-hypothesis ignore reason with the actual
      root cause and the multi_thread fix.
- [x] `parity_status` un-ignored. Same multi_thread fix applied. Source-side
      envelope work from T-1921 already in place; this just unblocks
      the test.
- [x] `termlink_ping` MCP envelope converged with CLI: `{ok:true, target,
      ...result}`. Was bare `{display_name, id, state}` — caller would
      have seen shape drift if the CLI subprocess had ever reached the
      session. Now structurally identical to `termlink ping <name> --json`
      (modulo CLI-only `latency_ms`, stripped by the parity test).
- [x] `cargo test --release --test parity -p termlink-mcp --
      --test-threads=1` exits 0 with `test result: ok. 14 passed;
      0 failed; 0 ignored` (was 12 passed + 2 ignored). Both previously-
      ignored tests now exercise the real socket roundtrip.
- [x] No regression of any other parity test.

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

**Symptom:** `parity_ping` (and later `parity_status`) test failed with
the CLI subprocess's ping returning `{"ok":false,"error":"Ping timed out
after 5s","latency_ms":5030}` even though the MCP in-process call against
the same session, in the same TERMLINK_RUNTIME_DIR, on the same socket,
succeeded immediately. The T-1909 v0.1 ignore comment hypothesized a
transport divergence (MCP in-process / CLI through-hub) but inspection of
`call_session` (commands/target.rs:188) showed both paths take the same
local-socket route when `--hub` is absent.

**Root cause:** `call_cli` (parity.rs:63) uses synchronous
`std::process::Command::output()` to spawn and wait on the CLI subprocess.
`#[tokio::test]` defaults to the `current_thread` runtime — single-threaded
cooperative scheduling. The test's `start_session` spawns
`server::run_accept_loop` via `tokio::spawn`. When the test then calls
`call_cli`, the test thread blocks for up to 5s inside `.output()`. The
accept_loop task is ready but the runtime is blocked — it never gets to
run. The CLI subprocess connects to the unix socket file (kernel queues
the connection), sends its JSON-RPC request, waits 5s for a response,
times out. The test process never `accept()`s.

MCP-side works because its transport is `tokio::io::duplex` — in-memory
async pipes that progress cooperatively without needing the kernel or
a thread to make I/O progress.

**Why structurally allowed:** The parity harness was added in T-1909 with
the default `#[tokio::test]` attribute. No prior parity test exercised a
socket roundtrip with synchronous subprocess I/O (parity_topics looked
similar but tested an empty topic state, so both sides returned empty —
the test passed accidentally without proving the socket worked). The bug
was latent until parity_ping/parity_status — the first tests that needed
a real wire-level roundtrip — were added and immediately hit it.

**Prevention:**
1. **The fix itself blocks the next instance.** Any future parity test
   that needs a socket roundtrip MUST use `#[tokio::test(flavor =
   "multi_thread", worker_threads = 2)]`. The parity.rs:141-156 comment
   block documents why; the hub-less tests (parity_topics et al.) stay
   on current_thread to keep cheap.
2. **PL-199 (this learning)** captures the pattern for future agents
   writing harness code that mixes async-runtime tasks with sync
   subprocess I/O.

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

### 2026-06-02 — runtime flavor vs. spawn_blocking vs. tokio::process
- **Chose:** `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`
  on the two affected tests (parity_ping, parity_status). Keep
  `call_cli` synchronous via `std::process::Command::output()`. Leave
  hub-less tests on the default current_thread flavor.
- **Why:** Smallest blast radius. The fix sits at the test boundary
  (test attribute) without changing the harness's `call_cli` helper
  that the hub-less tests already rely on. `worker_threads = 2` is
  the minimum that lets accept_loop run while the test thread is
  blocked on subprocess I/O (one worker for each).
- **Rejected:**
  - Switching `call_cli` to `tokio::process::Command::output().await`:
    would force the whole harness async-only and propagate the change
    into ~12 hub-less tests that don't need it. More churn.
  - Using `tokio::task::spawn_blocking` to wrap `.output()`: still
    needs a multi-thread runtime to actually run the blocking pool, so
    it's the same fix with extra boilerplate.

### 2026-06-02 — termlink_ping envelope: spread-merge wrap
- **Chose:** Wrap the hub's `result` into `{ok:true, target,
  ...result}` matching CLI `cmd_ping` (commands/session.rs:677-685).
  `latency_ms` stays CLI-only; the parity test strips it via the
  ignore list since MCP in-process has no meaningful wall-clock
  analog.
- **Why:** Consistent with PL-198's wrap_ok spread-merge pattern. The
  bare hub result lacked `ok` and `target`, which would have surprised
  any caller expecting CLI parity.
- **Rejected:** Echoing only `{ok, target, id, display_name, state}`
  by hand-listing fields — would break forward-compat the moment the
  hub adds a new field to the ping result. The spread-merge picks up
  new fields automatically.

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

### 2026-06-01T12:58:08Z — status-update [task-update-agent]
- **Change:** status: started-work → captured

### 2026-06-02T10:38:09Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
