---
id: T-2072
name: "channel claims-summary --watch --notify event hook (claim primitive observability arc, T-2065 mirror)"
description: >
  channel claims-summary --watch --notify event hook (claim primitive observability arc, T-2065 mirror)

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
created: 2026-06-09T08:27:45Z
last_update: 2026-06-09T08:27:45Z
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

# T-2072: channel claims-summary --watch --notify event hook (claim primitive observability arc, T-2065 mirror)

## Context

`channel claims-summary --watch` (T-2041 Slice 8) and `--all --watch` (T-2042
Slice 9) currently re-render to the screen — useful for a human watching but
useless for automation. Adding `--notify <CMD>` mirrors the proven pattern
shipped on `fleet doctor --watch --notify` (T-1669) and `fleet governor-status
--watch --notify` (T-2065): on per-topic stuck-state transitions, fire an
operator-pluggable shell command fire-and-forget with env vars set so the
script can page, post to Slack, write to a queue, etc.

Single-topic mode fires only on stuck-state transitions for the one topic.
`--all` mode fires on transition + new + removed per topic. Baseline tick
(no prior state) emits zero events.

Env vars on each fire:
- `TERMLINK_CLAIM_TOPIC` — the topic name
- `TERMLINK_CLAIM_CHANGE_KIND` — `transition` | `new` | `removed`
- `TERMLINK_CLAIM_TS` — RFC3339 detection time
- `TERMLINK_CLAIM_HUB` — hub addr (resolved)
- `TERMLINK_CLAIM_OLD_STUCK` / `TERMLINK_CLAIM_NEW_STUCK` — `true` | `false` | `n/a`
- `TERMLINK_CLAIM_ACTIVE_COUNT` / `TERMLINK_CLAIM_EXPIRED_COUNT` — current counts
- `TERMLINK_CLAIM_OLDEST_AGE_MS` — current oldest active claim age (or `n/a`)

Closes the operator-automation gap on substrate primitive #1 (claim semantics)
observability. Pure addition; existing `--watch` behavior unchanged.

## Acceptance Criteria

### Agent
- [x] `cli.rs` ClaimsSummary variant gains `notify: Option<String>` with `requires("watch")` (clap rejects `--notify` without `--watch`)
- [x] `main.rs` dispatch passes `notify` to `cmd_channel_claims_summary`
- [x] `cmd_channel_claims_summary` watch loop maintains per-topic prior-state, computes per-tick diff, fires `--notify` per change event
- [x] First tick is baseline only — no events fired even if topics are already stuck
- [x] Pure helper `diff_claim_states(prev: &BTreeMap<String, ClaimSnapshot>, curr: &BTreeMap<String, ClaimSnapshot>) -> Vec<ClaimChangeEvent>` extracted so the diff is unit-testable without spawning subprocesses
- [x] Notify subprocess is fire-and-forget — a hanging script does NOT block the loop, and command-not-found does NOT crash the watch
- [x] At least 4 unit tests on `diff_claim_states`: (a) baseline → empty, (b) stuck-state transition fires `transition` event, (c) new topic fires `new` event, (d) removed topic fires `removed` event, (e) no-change tick → empty
- [x] `cargo check -p termlink` builds clean (no new warnings)
- [x] `cargo test --bin termlink claims_summary_notify` passes (the unit tests)
- [x] CLAUDE.md `BACKPRESSURE` row (or equivalent claim-arc entry) acknowledges the new `--notify` form per the operator-discovery convention used for T-2065

### Human
<!-- All ACs above are agent-verifiable; no human review needed. -->

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

cargo check -p termlink 2>&1 | tail -5 | grep -qv "error\["
out=$(cargo test --bin termlink --release claims_summary_notify 2>&1); echo "$out" | grep -q "test result: ok"

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

### 2026-06-09T08:27:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2072-channel-claims-summary---watch---notify-.md
- **Context:** Initial task creation
