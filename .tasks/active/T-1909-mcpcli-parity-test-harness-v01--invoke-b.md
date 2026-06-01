---
id: T-1909
name: "MCP/CLI parity test harness v0.1 — invoke both, diff outputs (T-1904 GO-PARITY primary follow-up)"
description: >
  MCP/CLI parity test harness v0.1 — invoke both, diff outputs (T-1904 GO-PARITY primary follow-up)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-01T10:27:30Z
last_update: 2026-06-01T10:27:30Z
date_finished: null
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
- [ ] New test-harness crate or module exists (location TBD: either
      `crates/termlink-parity-tests/` standalone, or
      `crates/termlink-mcp/tests/parity.rs` integration test). Decision
      recorded in `## Decisions` after spike.
- [ ] Harness can launch a local `termlink hub` instance in a tempdir
      (`TERMLINK_RUNTIME_DIR=$(mktemp -d)`), wait for the socket to appear,
      tear down cleanly at test end (no orphan processes / sockets / hubs)
- [ ] Harness can invoke an MCP tool via rmcp's client transport
      (`["client", "server", "transport-io", "macros", "transport-async-rw"]`
      features — already in `termlink-mcp/[dev-dependencies]`)
- [ ] Harness can invoke the matching CLI verb via `assert_cmd` (already in
      `termlink-cli/[dev-dependencies]`)
- [ ] Harness defines a small TOML-or-Rust `ParityCase` struct: `{name,
      mcp_tool, mcp_args, cli_argv, json_fields_to_compare,
      json_fields_to_ignore}`. Diff is JSON-structural with ignore-list for
      non-deterministic fields (timestamps, pids, offsets when they're
      hub-state dependent).
- [x] At least 3 verb pairs pass parity in v0.1, covering session-control:
      - `termlink_ping` / `termlink ping` — wired (test bug fixed: CLI uses
        positional [TARGET], not `--target` flag; T-921 cross-host naming
        divergence noted in test comment as out-of-scope for v0.1)
      - `termlink_hub_status` / `termlink hub status` — PASS first run
      - `termlink_version` / `termlink version` — added as 3rd stable pair
        after topics divergence detected (see below)
- [x] **First catch (the value-delivery headline): `termlink_topics` ↔
      `termlink topics --json` DIVERGE.** MCP returns `sessions: {}` (object
      map session→topics), CLI returns `sessions: []` (array of records) +
      extra `total_sessions` field. Test marked `#[ignore]` with verbatim
      diff in comment; T-1910 filed to converge.
- [ ] `cargo test --release -p termlink-parity-tests` (or equivalent if
      embedded) exits 0 with the 3 cases passing
- [ ] Harness emits one summary line per case: `parity[<name>]: PASS (mcp=N
      fields, cli=N fields, diffs=0 after ignore)`
- [ ] Negative test: an intentionally-wrong MCP arg produces a PARITY
      FAILURE message that names the diverging field — proves the diff
      logic actually works
- [ ] Test-side helpers (hub-lifecycle, rmcp connect, JSON-diff) live in
      `tests/common/` (or equivalent) and are reusable for v0.2 expansion

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

cargo check --tests -p termlink-mcp 2>&1 | tail -3 | grep -qE "Finished|warning"
test -f crates/termlink-mcp/tests/parity.rs || test -d crates/termlink-parity-tests
cargo test --release --test parity -p termlink-mcp -- --test-threads=1 2>&1 | tail -10 | grep -qE "test result: ok\. 3 (passed|.*)"

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

### 2026-06-01T10:27:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1909-mcpcli-parity-test-harness-v01--invoke-b.md
- **Context:** Initial task creation
