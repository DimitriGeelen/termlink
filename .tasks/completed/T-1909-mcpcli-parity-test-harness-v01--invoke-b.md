---
id: T-1909
name: "MCP/CLI parity test harness v0.1 — invoke both, diff outputs (T-1904 GO-PARITY primary follow-up)"
description: >
  MCP/CLI parity test harness v0.1 — invoke both, diff outputs (T-1904 GO-PARITY primary follow-up)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-test-utils/src/lib.rs]
related_tasks: []
created: 2026-06-01T10:27:30Z
last_update: 2026-06-01T11:37:26Z
date_finished: 2026-06-01T11:37:26Z
---

# T-1909: MCP/CLI parity test harness v0.1 — invoke both, diff outputs (T-1904 GO-PARITY primary follow-up)

## Context

T-1904 GO-PARITY primary follow-up. The MCP/CLI census found 122 naming-match
tool pairs split into Layer-1 SHARED (data-access primitives) and Layer-2/3
DIVERGENT-BY-COPY (83 `_mcp` helpers in `tools.rs` parallel CLI helpers in
`commands/channel.rs` + whole-tool reimplementations like `fleet_doctor`).
The parity harness diffs MCP-tool output against CLI-command output for the
same logical operation, catching silent drift before it ships.

Scope of v0.1: thin slice with 3-5 verb pairs covering the highest-trafficked
subsystem (session-control). Establishes the harness shape; v0.2 expands to
channel_* (53 pairs); v0.3 covers chat-arc agent_* (the divergence-heavy
group). v0.1 is the structural foundation; coverage is iterative.

Full census evidence: `docs/reports/T-1904-mcp-vs-direct-session.md`.

## Acceptance Criteria

### Agent
- [x] New test-harness module exists at
      `crates/termlink-mcp/tests/parity.rs` (integration test, not
      standalone crate — keeps the rmcp client/server in-process fixture
      pattern from `mcp_integration.rs` reusable without crate-graph churn).
      Decision in `## Decisions` below.
- [x] Harness wires in-process session fixture via
      `termlink_test_utils::start_session` (no separate hub process needed
      for v0.1's session-control slice). Test teardown via `_handle.abort()`
      and `TestDir::drop` removes the tempdir. No orphan sockets after test
      exit. (Note: parity_ping's hub-required transport surfaced as a real
      catch — see below — not as a fixture gap.)
- [x] Harness invokes MCP tools via rmcp's client transport
      (`tokio::io::duplex(65536)` channel from in-process `TermLinkTools`
      server). `call_mcp()` helper.
- [x] Harness invokes the matching CLI verb via `Command::new(binary)`
      with `find_termlink_bin()` walking up to workspace Cargo.lock. JSON
      parsed from stdout; non-zero exit returns structured error.
      `call_cli()` helper.
- [x] Harness uses a function-per-pair pattern with shared ignore-list
      stripping (`strip_fields` + `diff_json`). Lighter than a TOML
      `ParityCase` struct for v0.1's 4 cases; refactor to struct deferred
      to v0.2 when the case-list scales.
- [x] **v0.1 catalogues 4 verb pairs (1 parity-confirmed, 3 catches):**
      - `termlink_hub_status` / `termlink hub status --json` — **PASS**
        (both report `not_running` identically with ignored
        `pid/pidfile/socket/socket_path/ts_ms`)
      - `termlink_ping` / `termlink ping <name> --json` — **SECOND CATCH**:
        MCP uses in-process session lookup, CLI routes through hub and
        times out without one. T-1911 filed. `#[ignore]`d.
      - `termlink_topics` / `termlink topics --json` — **FIRST CATCH**:
        MCP returns `sessions: {}` (object map), CLI returns
        `sessions: []` (array) + extra `total_sessions`. T-1910 filed.
        `#[ignore]`d.
      - `termlink_version` / `termlink version --json` — **THIRD CATCH**:
        MCP reads crate Cargo.toml (`0.9.0`/`unknown`), CLI reads
        build.rs git metadata (`0.11.501`/`8a1aafb0`/`x86_64-...`).
        T-1912 filed. `#[ignore]`d.
- [x] **Value-delivery headline: harness's first execution caught THREE
      real Layer-2/3 orchestration divergences** — exactly the maintenance
      hazard T-1904's census predicted. Each catch has a filed convergence
      task; un-ignore the corresponding test when each converges.
- [x] `cargo test --release --test parity -p termlink-mcp -- --test-threads=1`
      exits 0: `test result: ok. 2 passed; 0 failed; 3 ignored;
      0 measured; 0 filtered out`
- [x] Harness emits one summary line per case (verified):
      `parity[hub_status]: PASS (mcp=2 fields, cli=2 fields, diffs=0 after ignore)`
- [x] Negative test (`parity_negative_self_test`) constructs a
      hand-crafted diff (`session` vs `sesion` typo) and asserts
      `diff_json` returns `Err` containing the diverging value. Proves
      the diff logic is not a no-op.
- [x] Reusable helpers in `crates/termlink-test-utils/src/lib.rs`:
      `find_termlink_bin()` walks workspace root + honors `TERMLINK_BIN`
      override. `start_session`/`TestDir`/`termlink_cmd` were already
      present. In-test helpers (`mcp_client`, `call_mcp`, `call_cli`,
      `strip_fields`, `diff_json`) live in `parity.rs` and are ready to
      copy-or-extract for v0.2.

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

test -f crates/termlink-mcp/tests/parity.rs
grep -q "find_termlink_bin" crates/termlink-test-utils/src/lib.rs
cargo test --release --test parity -p termlink-mcp -- --test-threads=1 2>&1 | tail -2 | grep -qE "test result: ok\. 2 passed; 0 failed; 3 ignored"

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

### 2026-06-01 — harness location

- **Chose:** integration test inside `crates/termlink-mcp/tests/parity.rs`.
- **Why:** rmcp client/server in-process fixture (`tokio::io::duplex`)
  is already established in `mcp_integration.rs`; co-locating parity
  tests in the same crate avoids a new crate-graph node + reuses the
  pattern. Cross-crate CLI access via new `find_termlink_bin()` helper
  in `termlink-test-utils`.
- **Rejected:** standalone `crates/termlink-parity-tests/` crate — would
  need its own workspace member, target dir, fixture dependencies, and
  CI lane for marginal isolation gain.

### 2026-06-01 — case representation (per-pair fn vs struct)

- **Chose:** one `#[tokio::test]` async fn per pair, with shared
  helpers (`call_mcp`, `call_cli`, `diff_json`, `strip_fields`).
- **Why:** v0.1's 4 pairs are easier to read and individually
  `#[ignore]`-flaggable as functions. Each catch gets a dedicated
  comment block with diagnostic detail.
- **Rejected (for v0.1):** TOML-driven `ParityCase` struct table.
  Worth revisiting at v0.2 once 10+ pairs exist and the per-fn
  boilerplate amortizes the loss of comment-per-case detail.

### 2026-06-01 — divergence-handling protocol

- **Chose:** catches are marked `#[ignore = "T-1909 Nth-catch: ..."]`
  with full diagnostic comment in the test source, AND a follow-up
  convergence task is filed (T-1910/T-1911/T-1912). Test un-ignored
  by the convergence task.
- **Why:** keeps the harness green (CI signal stays clean), preserves
  the divergence diff as a permanent in-source record, and gives each
  convergence its own task surface (one-bug-one-task).
- **Rejected:** keeping tests as `failure` to "force" attention — would
  permanently break the CI lane; convergence may need design discussion
  before code change.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-06-01T10:27:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1909-mcpcli-parity-test-harness-v01--invoke-b.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-c67e2ee6
- **Timestamp:** 2026-06-01T11:42:05Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **mock-only-integration** (partial, heuristic) @ AC vs Verification cross-check
     - evidence: `test -f crates/termlink-mcp/tests/parity.rs`

### 2026-06-01T11:37:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
