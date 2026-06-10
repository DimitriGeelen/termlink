---
id: T-2112
name: "substrate status --watch: continuous monitor (T-2111 arc Slice 2 — pattern parity with T-2064)"
description: >
  substrate status --watch: continuous monitor (T-2111 arc Slice 2 — pattern parity with T-2064)

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
created: 2026-06-10T07:25:24Z
last_update: 2026-06-10T07:25:24Z
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

# T-2112: substrate status --watch: continuous monitor (T-2111 arc Slice 2 — pattern parity with T-2064)

## Context

T-2111 shipped `termlink substrate status` — Slice 1 of the substrate-status
observability arc (T-2018 §6 observability roll-up). This task adds **Slice 2:
`--watch <SECS>`** — a continuous monitor that re-polls every N seconds (5..=3600
clamped) and emits change-only output after a baseline cycle. Pattern parity
with `fleet governor-status --watch` (T-2064), `agent find-idle --watch` (T-2078),
`channel claims-summary --watch` (T-2041), `channel queue-status --watch` (T-2083).

Design: substrate-status --watch is the ROLLUP monitor — it tracks high-level
substrate-health counters (idle agents, stuck claims, queue depth, pressured
hubs) per cycle, NOT per-entity diffs. Operators wanting per-entity drilldown
use the underlying verb's own `--watch` loop. This verb answers "is substrate
health DEGRADING?" in one terminal.

Per tick, capture a SubstrateRollup struct: idle_count, claim topic_count +
stuck_count, resilience pending, backpressure total_hubs + pressured_hubs,
plus per-section ok flags so a failed sub-read shows up as a "section
unavailable" transition rather than silently masking metrics as 0.

Cycle 1 = baseline (print full rollup). Cycle N>1 = emit one change-line per
rollup-field change, or one silent-cycle marker if no changes. SIGINT exits
cleanly.

Implementation: subprocess `termlink substrate status --json` per cycle, parse
JSON, diff against prior rollup, render. Subprocess pattern matches
`cmd_fleet_governor_status_watch` (T-2064) — keeps each tick state-independent.

**Mutex constraints (mirror T-2078 / T-2064 / T-2041 conventions):**
- `--watch` ⊕ `--json` (streaming text only, not parseable NDJSON-on-cleared-screen)
- `--watch` ⊕ `--only-pressured` (the filter would silently drop "moved out of
  pressure" transitions — same reasoning as T-2070 for governor watch)

Not in scope (deferred to later slices):
- `--notify <CMD>` event hook (Slice 3, mirrors T-2079 / T-2065)
- `--log <PATH>` audit trail (Slice 4, mirrors T-2080 / T-2066)
- `substrate history` retrospective CLI (Slice 5, mirrors T-2081 / T-2068)
- MCP parity (Slice 6+)

## Acceptance Criteria

### Agent
- [x] `SubstrateAction::Status` gains `--watch <SECS>` flag, mutex with
      `--json` AND `--only-pressured` via clap `conflicts_with`.
- [x] When `--watch` is present, main.rs dispatches to new
      `cmd_substrate_status_watch` (in `commands::substrate`).
- [x] `cmd_substrate_status_watch` validates SECS clamp 5..=3600; rejects
      out-of-range with a clear error.
- [x] Each tick subprocesses `termlink substrate status --json` (mirrors
      the pattern in `cmd_fleet_governor_status_watch`).
- [x] Per-cycle JSON → `SubstrateRollup` struct via pure helper
      `parse_substrate_rollup(json)`.
- [x] Cycle 1 (baseline) prints full rollup: idle/topic/stuck/pending/pressured
      counts + per-section ok flags.
- [x] Cycle N>1 prints one change-line per rollup field change with shape
      `<ts>  <field>: <old>→<new>`. When NO field changed, a single
      `<ts>  (no changes)` marker prints — affirmative, never silent.
- [x] SIGINT (`ctrl-c`) exits cleanly with a final summary line showing
      total cycle count.
- [x] Sub-verb subprocess failure (JSON unparseable, nonzero exit) prints
      one stderr line and the loop continues to the next tick.
- [x] At least 3 unit tests: (a) `parse_substrate_rollup` extracts each
      field correctly from synthetic JSON; (b) `diff_substrate_rollup`
      returns no events on identical inputs; (c) `diff_substrate_rollup`
      surfaces each field change as a distinct event. (Shipped 7 new tests.)
- [x] Live smoke: `termlink substrate status --watch 5` against local hub
      prints baseline + at least one silent-cycle marker.

### 2026-06-10T07:32:00Z — Slice 2 implemented + smoked end-to-end
- **Code shipped:**
  - `crates/termlink-cli/src/commands/substrate.rs` — added `SubstrateRollup`
    struct + `parse_substrate_rollup` + `diff_substrate_rollup` +
    `render_substrate_baseline` + `cmd_substrate_status_watch` (~270 lines)
  - `crates/termlink-cli/src/cli.rs` — added `--watch <SECONDS>` flag to
    `SubstrateAction::Status` with `conflicts_with = "watch"` on `--json`
    and `--only-pressured`
  - `crates/termlink-cli/src/main.rs` — branched dispatch on `watch.is_some()`
- **Tests:** 13/13 substrate unit tests pass (7 new for the watch arc);
  909/909 CLI regression (was 902 baseline).
- **Live smoke** — `timeout 28 termlink substrate status --watch 6`:
  ```
  2026-06-10T07:30:56Z substrate-watch: polling every 6s; ctrl-c to stop
  2026-06-10T07:30:56Z baseline: substrate rollup
  2026-06-10T07:30:56Z   dispatch:     ok=true idle_count=0
  2026-06-10T07:30:56Z   claim:        ok=true topic_count=1336 stuck_count=0
  2026-06-10T07:30:56Z   resilience:   ok=true pending=0
  2026-06-10T07:30:56Z   backpressure: ok=true total=5 pressured=5
  2026-06-10T07:31:02Z  (no changes)
  2026-06-10T07:31:09Z  (no changes)
  2026-06-10T07:31:15Z  (no changes)
  2026-06-10T07:31:22Z  (no changes)
  ```
  Baseline + 4 silent cycles in 28s with 6s interval — exactly as designed.
- **Mutex constraints verified:**
  - `--watch 5 --json` → clap rejects with "cannot be used with --json"
  - `--watch 5 --only-pressured` → clap rejects with "cannot be used with --only-pressured"
  - `--watch 1` → handler rejects with "interval must be 5..=3600 seconds"

## Verification
cargo check -p termlink 2>&1 | tail -5
cargo test -p termlink --bin termlink substrate 2>&1 | tail -10
./target/debug/termlink substrate status --help 2>&1 | grep -q "watch"

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

### 2026-06-10T07:25:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2112-substrate-status---watch-continuous-moni.md
- **Context:** Initial task creation
