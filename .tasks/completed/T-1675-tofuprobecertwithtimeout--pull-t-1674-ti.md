---
id: T-1675
name: "tofu::probe_cert_with_timeout — pull T-1674 timeout into the primitive; migrate all 8 callers"
description: >
  tofu::probe_cert_with_timeout — pull T-1674 timeout into the primitive; migrate all 8 callers

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-17T20:59:40Z
last_update: 2026-05-17T21:09:48Z
date_finished: 2026-05-17T21:09:48Z
---

# T-1675: tofu::probe_cert_with_timeout — pull T-1674 timeout into the primitive; migrate all 8 callers

## Context

T-1674 wrapped one of 8 `probe_cert` call sites with `tokio::time::timeout`. The other 7 (`fleet verify`, `hub probe`, `tofu verify`, plus three MCP equivalents) still hang on the OS TCP retry budget for unreachable hubs. Centralize the timeout at the primitive level: add `probe_cert_with_timeout(addr, timeout)` in `crates/termlink-session/src/tofu.rs`, migrate all 8 callers (T-1674's inline wrapper collapses to a single helper call), and the systemic latency win lands fleet-wide.

`probe_cert` (no-timeout) is retained for API stability but deprecated in favor of the timeout variant. Existing direct-callers in tests can continue to use it.

## Acceptance Criteria

### Agent
- [x] `probe_cert_with_timeout(addr: &str, timeout: Duration) -> Result<(Vec<u8>, String), String>` added to `crates/termlink-session/src/tofu.rs:447-468`, returning `TLS probe to {addr} timeout after {N}s` on timeout
- [x] All 8 call sites migrated: `remote.rs` (cmd_fleet_doctor + cmd_fleet_verify), `infrastructure.rs` (cmd_hub_probe + cmd_tofu_verify), `tools.rs` (termlink_fleet_verify + termlink_hub_probe + termlink_tofu_verify). `grep probe_cert | grep -v probe_cert_with_timeout` returns zero matches in those files
- [x] T-1674's inline `tokio::time::timeout` wrapper in `cmd_fleet_doctor` collapsed to a single `probe_cert_with_timeout` call — DRY achieved
- [x] `cargo check --workspace` passes (1 pre-existing termlink-mcp warning unchanged)
- [x] Live smoke: `fleet verify` total wall time **10.02s** with laptop-141 unreachable (was unbounded; T-1667 commit notes reported 2m16s for the parallel-spawn case). `hub probe 192.168.10.141:9100` and `tofu verify 192.168.10.141:9100` both bail at 10.01s with the canonical error string
- [x] Unit test `tofu::tests::probe_cert_with_timeout_errors_on_unreachable` passes — probes RFC 5737 TEST-NET-1 with 1s bound, asserts error path includes addr + duration if timeout fires

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

cargo check --workspace
bash -c "grep -q 'probe_cert_with_timeout' crates/termlink-session/src/tofu.rs"
bash -c "[ $(grep -rn 'probe_cert\b\|probe_cert_with_timeout' crates/termlink-cli/src/commands/remote.rs crates/termlink-cli/src/commands/infrastructure.rs crates/termlink-mcp/src/tools.rs | grep -v 'tofu::probe_cert_with_timeout\|// ' | wc -l) -le 1 ]"

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

### 2026-05-17T20:59:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1675-tofuprobecertwithtimeout--pull-t-1674-ti.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-6ee91c95
- **Timestamp:** 2026-05-17T21:09:49Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-17T21:09:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
