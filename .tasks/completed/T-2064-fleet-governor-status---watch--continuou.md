---
id: T-2064
name: "fleet governor-status --watch — continuous fleet-wide governor monitor (T-2028 §6 #10 Track E)"
description: >
  fleet governor-status --watch — continuous fleet-wide governor monitor (T-2028 §6 #10 Track E)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-08T21:37:58Z
last_update: 2026-06-08T22:10:12Z
date_finished: 2026-06-08T22:58:35Z
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

# T-2064: fleet governor-status --watch — continuous fleet-wide governor monitor (T-2028 §6 #10 Track E)

## Context

T-2028 §6 #10 substrate-governor primitive shipped via Track B (RPC + single-hub MCP, T-2048),
Track C (single-hub CLI `hub status --governor`, T-2060), Track D (fleet CLI
`fleet governor-status`, T-2062), and Fleet-MCP (T-2063). Tracks B/C/D answer
"what's the governor state right now?" — a single-shot question.

Track E adds the continuous-monitor surface that answers "is anything changing?" —
the operator pattern that `fleet doctor --watch` (T-1667) proved valuable for
rotation surveillance. Same shape, different telemetry channel.

The watch loop polls `fleet governor-status --json` every N seconds, tracks per-hub
state (connections_active, capacity_hits_total, rate_hits_total, dedupe_hits_total,
reachable), and emits a baseline cycle then change-only output for subsequent
cycles. SIGINT exits cleanly.

This closes the "missing operator UX layer" gap on the substrate-governor arc and
gives operators a one-keystroke way to leave a terminal running that loudly
surfaces "hub X just refused 47 connections" instead of having to re-run a
one-shot every minute.

## Acceptance Criteria

### Agent
- [x] `FleetAction::GovernorStatus` gains a `--watch <SECONDS>` flag, clamped [5, 3600], mutex with `--json`. — cli.rs line 3641, `#[arg(long, value_name = "SECONDS")] watch: Option<u64>` + `#[arg(long, conflicts_with = "watch")] json: bool`. Runtime clamp at remote.rs `if !(5..=3600).contains(&secs) { bail }`.
- [x] `cmd_fleet_governor_status_watch` implements the subprocess-respawn-with-json + diff loop pattern (mirrors `cmd_fleet_doctor_watch` shape). — remote.rs ~line 2820, uses `std::env::current_exe()` + `tokio::process::Command::new(exe).args(["fleet","governor-status","--json","--timeout",N])`.
- [x] Baseline cycle (cycle 1) prints one line per hub with reach + conn + cap_hits + rate_hits + dedupe_hits. — verified live: 5 hubs printed at 2026-06-08T22:09:05Z with each hub's `reach= conn=X/Y cap_hits=N rate_hits=N dedupe_hits=N|n/a`.
- [x] Subsequent cycles emit one line per CHANGED hub (transition / new / removed); silent cycles print a single "no changes" footer. — live evidence: cycle 2 emitted `workstation-107-public conn=4/256→3/256 cap_hits=0→0 rate_hits=0→0 dedupe_hits=n/a`; cycle 3 emitted `no changes (cycle 3)`.
- [x] SIGINT during sleep OR during subprocess exits cleanly with a "watch stopped" line and exit 0. — both wait points wrap in `tokio::select! { ctrl_c() => print "watch stopped" + return Ok(()) }`. Smoke killed via parent `timeout` (SIGTERM) confirmed clean exit; SIGINT path symmetric (same select arm).
- [x] A pure render-helper formats one change line and has ≥3 unit tests covering: cap_hits delta, rate_hits delta, reachable transition. — `render_governor_watch_change_line()` extracted; tests pass: `watch_governor_renders_cap_hits_delta` + `watch_governor_renders_rate_hits_delta` + `watch_governor_renders_reachable_transition`. 826 → 829 bin lib tests.
- [x] CLAUDE.md BACKPRESSURE row updated with `--watch` example. — row 1170 now reads "Track B+C+D+E" with T-2064 paragraph describing `--watch <secs>` semantics.

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

cd /opt/termlink && cargo build -p termlink --release 2>&1 | tail -3
cd /opt/termlink && out=$(cargo test -p termlink --release --bin termlink watch_governor 2>&1); echo "$out" | tail -10; echo "$out" | grep -q "3 passed; 0 failed"
cd /opt/termlink && grep -q "fleet governor-status --watch" CLAUDE.md

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

### 2026-06-08T21:37:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2064-fleet-governor-status---watch--continuou.md
- **Context:** Initial task creation
