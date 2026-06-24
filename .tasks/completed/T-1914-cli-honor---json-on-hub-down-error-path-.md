---
id: T-1914
name: "CLI: honor --json on hub-down error path (T-1913 fourth-catch)"
description: >
  CLI 'termlink channel list --json' writes to stderr + empty stdout on hub-down (exit 1). MCP emits structured JSON error. Make CLI emit JSON error on stdout when --json is set, matching MCP shape. Likely affects more commands than just channel list — broader audit needed. Un-ignore parity_channel_list_no_hub when converged.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: [T-1904, T-1909, T-1913]
created: 2026-06-01T13:12:25Z
last_update: 2026-06-01T14:20:03Z
date_finished: 2026-06-01T14:31:33Z
---

# T-1914: CLI: honor --json on hub-down error path (T-1913 fourth-catch)

## Context

Operator-visible bug surfaced by the T-1913 parity harness expansion:
`termlink channel list --json | jq` produces nothing parseable when the
hub is down. CLI writes "Error: Hub is not running…" to stderr with
empty stdout (exit 1). The MCP equivalent (`termlink_channel_list`)
correctly returns a structured JSON error.

**Mechanism (`crates/termlink-cli/src/commands/channel.rs:8371-8389`):**
`cmd_channel_list` calls `hub_socket(hub)?` first thing. When no hub
is running, `hub_socket` does `anyhow::bail!("Hub is not running …")`
which propagates up via `?` before the `if json_output { ... }` branch
is ever reached. The framework's top-level error handler then prints
the anyhow error to stderr.

**Scope choice — narrow first slice:** fix `cmd_channel_list` only.
The broader audit (every CLI command that errors before reaching its
--json branch) is filed as a follow-up so this slice stays small.

There's already a helper for this: `commands::mod::json_error_exit`
prints a JSON value to stdout, flushes, and exits 1.

## Acceptance Criteria

### Agent
- [x] `cmd_channel_list` in `crates/termlink-cli/src/commands/channel.rs`
      catches the `hub_socket` error: when `json_output` is true, emits
      `{"ok": false, "error": "Hub is not running …"}` to stdout via
      `json_error_exit` (matches MCP's shape from `tools.rs:24860`).
- [x] `parity_channel_list_no_hub` test in `crates/termlink-mcp/tests/
      parity.rs` un-ignored (`#[ignore]` attribute removed; in-source
      diagnostic comment updated to note convergence date + structural fix).
- [x] `cargo test --release --test parity -p termlink-mcp --
      --test-threads=1` exits 0: `test result: ok. 6 passed; 0 failed;
      1 ignored` (was: 5 passed; 0 failed; 2 ignored — channel_list_no_hub
      now passes; ping T-1911 remains ignored).
- [x] Manual smoke: `target/release/termlink channel list --json` with
      no hub running emits parseable JSON to stdout (validated via `jq`).
- [x] Filed T-1915 follow-up for the broader audit ("identify all CLI
      commands that error before reaching their --json branch").

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

grep -q "json_error_exit" crates/termlink-cli/src/commands/channel.rs
cargo test --release --test parity -p termlink-mcp -- --test-threads=1 2>&1 | tail -2 | grep -qE "test result: ok\. 6 passed; 0 failed; 1 ignored"
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

## RCA

**Symptom:** `termlink channel list --json | jq` produced nothing
parseable when the hub was down. Operator pipelines silently broke at
the `jq` step (empty stdin). The MCP equivalent
(`termlink_channel_list`) correctly returned a JSON error in the same
condition.

**Root cause:** `cmd_channel_list` called `hub_socket(hub)?` as its
first line. When no hub was running, `hub_socket` returned
`anyhow::bail!("Hub is not running …")`. The `?` operator propagated
the error to anyhow's top-level handler, which formatted it to stderr.
Control flow exited `cmd_channel_list` BEFORE the `if json_output { …
}` branch was reached — so `--json` was effectively ignored on this
error path.

**Why structurally allowed:** the project had no MCP/CLI parity test
for channel-list error paths. Success-path JSON shape was tested
(via existing integration tests), but error-path JSON behavior had no
coverage. The MCP and CLI implementations were free to diverge in
their error envelopes without anything failing.

**Prevention:** T-1909 parity harness now covers `parity_channel_list_no_hub`
(re-enabled here). Any future regression that drops the JSON-on-stdout
contract for hub-down will fail this test. T-1915 expands the same
coverage to other CLI verbs with the same early-error pattern.

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

### 2026-06-01T13:12:25Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1914-cli-honor---json-on-hub-down-error-path-.md
- **Context:** Initial task creation

### 2026-06-01T13:35:40Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
