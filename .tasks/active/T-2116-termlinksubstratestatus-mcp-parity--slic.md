---
id: T-2116
name: "termlink_substrate_status MCP parity — Slice 6 (T-2111 arc, T-2018 §6)"
description: >
  termlink_substrate_status MCP parity — Slice 6 (T-2111 arc, T-2018 §6)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-10T08:24:10Z
last_update: 2026-06-10T08:24:10Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── BVP scoring fields (T-1918, arc-006). See docs/reports/T-1915-bvp-inception.md for semantics. ──
# bvp_scores:                     # confirmed per-driver scores 0-5, set by `fw bvp confirm` (T-1924).
#                                 # Sovereignty boundary — only set after human or agent confirmation.
#                                 # Shape: {D1: <int 0-5>, D2: <int 0-5>, D3: <int 0-5>, D4: <int 0-5>, [<free-driver-id>: <int>]...}
# bvp_scores_proposed:            # estimator-proposed scores (T-1922 worker). Persists when ≥2 delta
#                                 # from bvp_scores: on any driver (M3 v2-delta). Shape: list of timestamped entries.
# cost_estimate:                  # F8 composite: 0.6×blast_radius + 0.3×tier + 0.1×effort.
#                                 # Q2 fallback: T-shirt S/M/L/XL mapped to 2/4/6/8 when blast_radius is not yet computable.
---

# T-2116: termlink_substrate_status MCP parity — Slice 6 (T-2111 arc, T-2018 §6)

## Context

Slice 6 of T-2111 substrate-status observability roll-up arc — MCP-tier
parity for the one-shot `substrate status` CLI verb shipped in Slice 1
(T-2111). Mirror of T-2063 / T-2071 (`termlink_fleet_governor_status`
MCP). Agent-callable companion so an investigating MCP-attached agent
can get the substrate health rollup without shelling out.

Pattern parity:
- T-2063 `termlink_fleet_governor_status` (MCP for fleet governor-status)
- T-2077 `termlink_channel_claims_summary_all` with `only_stuck` param
- T-1689 `termlink_fleet_bootstrap_check` (subprocess-self pattern)

Design — subprocess-self pattern (mirror T-1689 bootstrap-check):
- New `SubstrateStatusParams { only_pressured: Option<bool>, timeout_secs: Option<u64> }`
- New `termlink_substrate_status` async tool:
  - Resolves `current_exe()` (own binary)
  - Spawns `<exe> substrate status --json [--only-pressured]
    [--timeout <N>]` under `tokio::time::timeout` (default 12s — CLI's
    8s timeout + 4s buffer for subprocess startup + JSON deserialization)
  - `kill_on_drop(true)` + null stdin so a wedged sub-RPC can't leak
  - Decorates parsed envelope with `ok` from exit code
  - Timeout → `{ok:false, verdict:"timeout", error:"..."}`
- Register in tool listing
- Add 2 unit tests for the param shape (only_pressured default + timeout
  default + envelope decoration)

Subprocess (not direct RPC composition) is chosen because:
1. CLI substrate-status already composes 4 sub-fetches in parallel via
   `tokio::join!`; re-implementing in MCP doubles maintenance cost
2. T-1689 set the precedent for "subprocess CLI from MCP" — established
   pattern; readers know the shape
3. Single source of truth for the 4-section JSON envelope

Right-sized — ~120 LOC + 1-2 unit tests. Slice 7 (MCP parity for
`substrate history`) follows with the read-only file-walking pattern.

## Acceptance Criteria

### Agent
- [x] New `SubstrateStatusParams` struct: `only_pressured: Option<bool>
      (default false)`, `timeout_secs: Option<u64> (default 12, clamped
      1..=120)`. Mirror of `FleetBootstrapCheckParams` shape.
- [x] New `termlink_substrate_status` async MCP tool registered in the
      tool list. Resolves `current_exe()`, subprocesses `<exe> substrate
      status --json` + flag passthrough, under `tokio::time::timeout`.
- [x] `kill_on_drop(true)` + null stdin set on the subprocess Command
      (mirror T-1689 — prevents leaked processes).
- [x] Timeout returns `{ok:false, verdict:"timeout", error:"timeout
      after <N>s"}` (mirror T-1689 timeout shape).
- [x] Subprocess output parsed as JSON; envelope decorated with `ok`
      (true if exit code 0) + `exit_code`. Non-JSON output surfaces raw
      stdout/stderr + exit code in the error path.
- [x] Tool description string declares the T-2018 §6 substrate-status
      rollup contract + lists the 4 sections (DISPATCH/CLAIM/RESILIENCE/
      BACKPRESSURE).
- [x] `cargo check -p termlink-mcp` passes (13.41s).
- [x] `cargo test -p termlink-mcp --lib` passes (861/861). Pre-existing
      6 mcp_integration test failures (test_list_sessions_empty et al)
      are UNRELATED to this slice — confirmed via `git stash` round-trip
      against the Slice 5 commit (a2a8c04c). Filed as follow-up gap.
- [x] Live smoke against local hub: built binary, invoked subprocess
      `./target/debug/termlink substrate status --json --timeout 8`.
      Validated JSON envelope shape: top-level keys
      `[backpressure, claim, dispatch, ok, only_pressured, resilience, ts]`,
      each sub-section's `.ok` field is parseable. Matches MCP wrapper's
      expected decoration target.
- [x] Side-fix: registered missing `termlink_channel_cv_keys` (T-2106)
      in help registry — pre-existing failure that blocks any new tool
      registration until cleared.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `## Verification` instead of a Human AC here. Only keep [REVIEW] if
     verification genuinely needs human taste (tone, feel, layout rhythm).
     See CLAUDE.md §AC Classification Guidance for the conversion rule.

     [REVIEW] example (genuine human judgment):
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error

     [REVIEWER] example (static-scan-verifiable — convert to Agent AC + Verification):
       - [ ] [REVIEWER] Block message names both bypass mechanisms
         **Steps:**
         1. Run `bin/fw reviewer T-XXX`
         **Expected:** Verdict: PASS; no findings on `block-message-completeness`
         **If not:** Inspect hook block-message string and add missing mechanism
       Conversion: this AC should be moved to ### Agent and
       `bin/fw reviewer T-XXX 2>&1 | grep -q "Overall:.*PASS"` added to ## Verification.
-->

## Verification

cargo check -p termlink-mcp 2>&1 | tail -5
cargo test -p termlink-mcp substrate 2>&1 | tail -10

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# Pipefail/SIGPIPE hint (L-387): P-011 runs each command under `set -eo pipefail`.
# `cmd | grep -q PATTERN` exits 141 (SIGPIPE) when grep matches and closes stdin
# while the upstream is still writing — verification then "fails" even though
# the pattern was present. Safe pattern: capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Or:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before this hint.
#
# Single pipe only — no intermediate tail/awk/sed stages between capture and grep
# (T-2090): `echo "$out" | tail -3 | grep -q PAT` re-introduces the SIGPIPE risk
# the capture step closed off — the middle stage is what `grep -q` slams its
# stdin on. `echo "$out"` is small and immediate; grep scans the whole captured
# string anyway, so the tail-3 was cosmetic. Drop it: `echo "$out" | grep -q PAT`.
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

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

### 2026-06-10T08:30:00Z — slice 6 shipped end-to-end
- **Action:** Implemented `termlink_substrate_status` MCP tool.
  tools.rs: added `SubstrateStatusParams` struct (mirror of
  `FleetBootstrapCheckParams` shape), tool body (subprocess-self pattern
  mirror of T-1689 `termlink_fleet_bootstrap_check`), and entry in the
  `fleet` category of the help registry. Side-fix: registered missing
  `termlink_channel_cv_keys` (T-2106) in `channel_data` category —
  pre-existing registry gap that blocks any new tool registration.
- **Verification:**
  - `cargo check -p termlink-mcp` — PASS (13.41s)
  - `cargo test -p termlink-mcp --lib` — 861/861 PASS
  - `cargo test -p termlink-mcp --lib help_registry` — PASS (both
    `help_registry_covers_all_real_tools` +
    `help_registry_has_no_phantom_entries`)
  - 6 pre-existing `mcp_integration` test failures (test_list_sessions_*
    et al) confirmed UNRELATED via `git stash` round-trip against Slice
    5 commit a2a8c04c — same 6 fail on stashed state. Pre-existing tech
    debt, filed as follow-up.
  - Live smoke (subprocess that MCP wrapper would invoke):
    `./target/debug/termlink substrate status --json --timeout 8`
    returns top-level keys `{backpressure, claim, dispatch, ok,
    only_pressured, resilience, ts}` — exact match for MCP wrapper's
    `ok`+`exit_code` decoration target.
- **Outcome:** Slice 6 closes the MCP-tier parity for `substrate status`
  one-shot. Pattern: subprocess-self with `tokio::time::timeout` +
  `kill_on_drop(true)` + null stdin. Default subprocess timeout 12s
  (CLI default 8s + 4s buffer). Slice 7 (MCP parity for `substrate
  history`) follows — file-walking pattern, mirror of T-2087 (queue-
  history MCP) / T-2069 (governor-history MCP). After Slice 7 the
  substrate-status observability arc is complete at both CLI + MCP
  tiers.
- **Context:** T-2018 §6 observability roll-up arc — T-2111 Slice 6
  (MCP-tier parity for one-shot).

### 2026-06-10T08:24:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2116-termlinksubstratestatus-mcp-parity--slic.md
- **Context:** Initial task creation
