---
id: T-1937
name: "fleet_verify status string — converge probe-fail → probe-failed (matches tofu_verify, T-1927)"
description: >
  T-1927 settled tofu_verify on 'probe-failed'. fleet_verify, fleet_doctor --include-pin-check, and fleet_reauth_all all still emit 'probe-fail' — both CLI and MCP. Aligning gives operators and LLM consumers a single status string to recognize across the rotation-protocol family. Scope: only string literal values that appear in 'status' / 'verdict' fields + operator-visible print strings + exit-code match arms. Out of scope: JSON field names (probe_fail_count, any_probe_fail) and internal Rust variable names.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-02T23:03:44Z
last_update: 2026-06-02T23:40:15Z
date_finished: 2026-06-03T00:34:17Z
---

# T-1937: fleet_verify status string — converge probe-fail → probe-failed (matches tofu_verify, T-1927)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `cmd_fleet_verify` in remote.rs emits `"probe-failed"` (not `"probe-fail"`) as per-hub `status` value — remote.rs:6002, 6017
- [x] `cmd_fleet_verify` aggregate `verdict` uses `"probe-failed"` and exit-code match arm updated — remote.rs:6029, 6049, 6067
- [x] `cmd_fleet_doctor` --include-pin-check pin-check summary uses `"probe-failed"` as per-hub status AND as aggregate verdict — remote.rs:3833, 4120, 4129, 4134
- [x] `cmd_fleet_reauth_all` task-state classification uses `"probe-failed"` — remote.rs:5819, 5833, 5843
- [x] MCP `termlink_fleet_verify` in tools.rs emits `"probe-failed"` symmetrically with CLI — tools.rs:12624, 12639, 12652, 12655
- [x] Plain-mode print strings (operator-visible) updated to `probe-failed` — remote.rs:4134
- [x] No remaining `"probe-fail"` (status value) literal in remote.rs or tools.rs — verified via `grep -nE '"probe-fail"'` returning no match (exit 1)
- [x] `cargo build --release -p termlink -p termlink-mcp` is warning-free — verified 2026-06-02 (release build finished in 8m07s, zero warnings)
- [x] Existing parity tests still pass — verified: 24 passed, 0 failed (no regression from prior 24/24 baseline)

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

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
! cargo build --release -p termlink -p termlink-mcp 2>&1 | grep -q "warning:"
! grep -nE '"probe-fail"' crates/termlink-cli/src/commands/remote.rs
! grep -nE '"probe-fail"' crates/termlink-mcp/src/tools.rs
cargo test --release -p termlink-mcp --test parity 2>&1 | grep -qE "test result: ok\."

## RCA

**Symptom:** Three rotation-protocol commands (`fleet_verify`, `fleet_doctor --include-pin-check`, `fleet_reauth_all`) emitted the status string `"probe-fail"` while sibling `tofu_verify` (converged in T-1927) emitted `"probe-failed"`. Operators and LLM consumers parsing rotation-protocol output had to handle BOTH strings depending on which verb produced the line.

**Root cause:** When T-1927 converged `tofu_verify` on `"probe-failed"`, the sibling commands using the same conceptual status were not part of that slice. No structural mechanism existed to detect divergent status strings within a tool family — T-1927 was a per-verb fix, not a family-wide convention.

**Why structurally allowed:** The parity harness validates per-pair envelope shapes (CLI vs MCP for the same verb), not cross-verb consistency within a family. There is no lint asserting "all rotation-protocol verbs use the same enumeration for analogous status states." Two valid envelope shapes can disagree on a status spelling and both still pass parity.

**Prevention:** This RCA is informational — the convergence itself prevents the next user-facing instance. A family-level consistency lint (assert: rotation-protocol family status strings drawn from one enum) would be the structural prevention, but that's deferred until the family grows enough to justify the tooling cost. For now, the matched 24/24 parity test count plus the elimination grep (`! grep '"probe-fail"'`) in this task's Verification block guards regression.

**Note on title-triggered classification:** The word "fail" in "probe-fail" triggered the G-019 bug-class gate even though this is alignment/convention work, not a bug fix. RCA is filled for substrate clarity rather than to document a defect.

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

### 2026-06-02T23:03:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1937-fleetverify-status-string--converge-prob.md
- **Context:** Initial task creation
