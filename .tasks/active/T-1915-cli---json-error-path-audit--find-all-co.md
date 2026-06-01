---
id: T-1915
name: "CLI --json error-path audit — find all commands like cmd_channel_list (T-1914 broader)"
description: >
  T-1914 fixed cmd_channel_list to honor --json on hub-down. Audit all other CLI commands for the same pattern: early bail/?-propagation before reaching the --json branch. Expected suspects: any cmd_channel_*, cmd_event_*, cmd_kv_* that contact a hub. Add parity tests for each as caught.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-1904, T-1909, T-1913, T-1914]
created: 2026-06-01T14:06:37Z
last_update: 2026-06-01T16:37:04Z
date_finished: null
---

# T-1915: CLI --json error-path audit — find all commands like cmd_channel_list (T-1914 broader)

## Context

T-1914 fixed `cmd_channel_list` to honor `--json` on hub-down by inline match/json_error_exit. Auditing channel.rs reveals 48 `let sock = hub_socket(hub)?;` sites; 45 are in cmd_channel_* functions that already accept `json_output: bool` (3 are internal helpers with no json_output, correctly skipped).

Approach: introduce a single helper next to `hub_socket` — `hub_socket_or_json_exit(hub, json_output) -> Result<TransportAddr>` — and mechanically convert the 45 cmd_channel_* sites. DRY in one source of truth, future commands inherit by using the helper.

## Acceptance Criteria

### Agent
- [x] Helper `hub_socket_or_json_exit(hub: Option<&str>, json_output: bool) -> Result<TransportAddr>` added next to `hub_socket` in `crates/termlink-cli/src/commands/channel.rs`. On `Err`, if `json_output` is true, emit `{"ok":false,"error":"..."}` via `super::json_error_exit` (which exits 1); otherwise return the `Err` unchanged.
- [x] All 45 `let sock = hub_socket(hub)?;` call sites in functions accepting `json_output: bool` converted to `let sock = hub_socket_or_json_exit(hub, json_output)?;`. The 3 internal-helper sites (fetch_topic_msgs, fetch_topic_msgs_paginated, fetch_chat_arc_full) remain `hub_socket(hub)?`. Final count: 46 helper sites (cmd_channel_list's T-1914 inline fix also collapsed to helper), 3 direct sites.
- [x] `cargo build -p termlink --release` succeeds (7m 08s).
- [x] Existing parity test `parity_channel_list_no_hub` still passes (regression check on T-1914's fix mechanism).
- [x] New parity test `parity_channel_create_no_hub` added — exercises a separate converted site and proves `termlink channel create <name> --json` produces parseable `{ok:false,error:...}` JSON when hub absent.
- [x] `cargo test --release --test parity -p termlink-mcp -- --test-threads=1` exits 0: `test result: ok. 7 passed; 0 failed; 1 ignored` (was 6 + 1 ignored — channel_create_no_hub is the new green test).

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

cargo test --release --test parity -p termlink-mcp -- --test-threads=1 2>&1 | tail -3 | grep -qE "test result: ok\. [0-9]+ passed; 0 failed"

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

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

### 2026-06-01T14:06:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1915-cli---json-error-path-audit--find-all-co.md
- **Context:** Initial task creation

### 2026-06-01T16:37:04Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
