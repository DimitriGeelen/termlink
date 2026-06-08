---
id: T-2060
name: "T-2028 Track C: surface governor counters into termlink hub status"
description: >
  T-2028 Track C: surface governor counters into termlink hub status

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/infrastructure.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-08T18:57:40Z
last_update: 2026-06-08T19:06:29Z
date_finished: 2026-06-08T19:06:29Z
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

# T-2060: T-2028 Track C: surface governor counters into termlink hub status

## Context

Substrate ADR (`docs/architecture/parallel-execution-substrate.md`) §6 #10
shipped the `hub.governor_status` RPC (T-2048) + `termlink_hub_governor_status`
MCP tool. Operators on a console cannot read backpressure telemetry without
shelling out to MCP — `termlink hub status` only shows PID/socket/runtime_dir.
This is the operator-UX surface for the substrate primitive #10 (Track C of
T-2028); RPC and MCP halves already shipped (Track B).

Track A (T-2057) audited retention policy. Track B (T-2048) built governor +
dedupe counters. Track C closes the loop: operators reading the console
should see the counters that prove the substrate is healthy without
needing a separate MCP probe.

## Acceptance Criteria

### Agent
- [x] `termlink hub status --governor` calls `hub.governor_status` RPC and renders the counters under a `Governor:` section in human mode (connections_active/max, capacity_hits_total, rate_buckets_active/rate_hits_total/max_rate_per_sec, dedupe_entries_active/hits_total/ttl_ms)
- [x] `termlink hub status --governor --json` merges the RPC response into the existing status JSON envelope under a `"governor"` key
- [x] When hub is `not_running` or `stale`, `--governor` is a no-op (skip the RPC; render the existing not_running/stale path unchanged)
- [x] When hub is running but the RPC times out or errors, the Governor section renders `(unavailable: <reason>)` rather than silently dropping
- [x] `termlink hub status` (without `--governor`) behaves identically to today — no extra RPC call, no extra latency
- [x] Default RPC timeout for the governor probe matches the doctor command's pattern (≤2s) so a wedged hub can't hang the status verb
- [x] At least one unit test on the pure-helper rendering function (formats a known governor JSON value into expected human-mode lines) — 2 tests added: known-value + missing-fields tolerance
- [x] `cargo build -p termlink` (alias termlink-cli) green; full `cargo test -p termlink --bin termlink` green: 823/823 (existing 821 + 2 new governor tests)

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

out=$(cargo build -p termlink 2>&1); echo "$out" | tail -5 | grep -q "Finished"
out=$(cargo test -p termlink --bin termlink render_governor 2>&1); echo "$out" | grep -qE "test result: ok\."
grep -q "fn render_governor_section" crates/termlink-cli/src/commands/infrastructure.rs
grep -q "hub.governor_status" crates/termlink-cli/src/commands/infrastructure.rs
out=$(target/debug/termlink hub status --help 2>&1); echo "$out" | grep -q -- "--governor"

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

### 2026-06-08 — n/a vs -1 sentinel for missing fields

- **What changed:** First pass rendered absent fields (e.g. dedupe_* on a pre-T-2049 hub) as `-1` sentinels. Live smoke against .107 produced `Dedupe: -1 entries (hits_total=-1, ttl_ms=-1)` which read awkwardly to an operator and could be mistaken for a real negative metric.
- **Plan impact:** Renderer changed from `i64`-with-`-1`-fallback to `String`-with-`"n/a"`-fallback. Test pinning updated to match. Information content preserved (operator can still see fields are missing → hub binary is older than client) but the format reads cleanly.
- **Triggered:** Single iteration on my own code; no new sub-task. The version-skew detection is now a side-effect of running `--governor` against a heterogeneous fleet — operators see immediately which hubs need binary refresh.

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

### 2026-06-08T18:57:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2060-t-2028-track-c-surface-governor-counters.md
- **Context:** Initial task creation

### 2026-06-08T19:06:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
