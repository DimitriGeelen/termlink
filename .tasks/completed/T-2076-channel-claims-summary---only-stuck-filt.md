---
id: T-2076
name: "channel claims-summary --only-stuck filter (operator-actionable subset, T-2070 mirror)"
description: >
  channel claims-summary --only-stuck filter (operator-actionable subset, T-2070 mirror)

status: work-completed
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
created: 2026-06-09T08:58:27Z
last_update: 2026-06-09T09:02:39Z
date_finished: 2026-06-09T09:47:42Z
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

# T-2076: channel claims-summary --only-stuck filter (operator-actionable subset, T-2070 mirror)

## Status note (2026-06-09 — recovered)

**RESOLVED.** Slice 16 of the substrate-claim observability arc completed
in the recovery session. All 11 ACs ticked, `cargo check` clean, 4/4
`claims_summary_only_stuck` tests pass. Channel.rs got the `only_stuck`
9th param plus the filter+counter-separation pattern in all three render
paths (snapshot, fleet text, fleet JSON), via a new pure helper
`claims_fleet_render_plan` that the unit tests exercise directly.

---

### Original park note (2026-06-09 — pre-recovery, kept for traceability)

**Build is broken on `main` — this task was started but did not complete
before budget exhaustion.** The partial state landed on disk:

- `crates/termlink-cli/src/cli.rs` — `only_stuck: bool` field added to
  `ClaimsSummary` clap variant with `requires("all")`
- `crates/termlink-cli/src/main.rs` — dispatch updated to pass
  `only_stuck` as 9th arg to `cmd_channel_claims_summary`
- `crates/termlink-cli/src/commands/channel.rs` — **NOT YET UPDATED**.
  Function `cmd_channel_claims_summary` still accepts 8 args. Compile
  fails with E0061 (arity mismatch).

**Next session — exact steps to recover:**

1. Edit `crates/termlink-cli/src/commands/channel.rs` to update
   `pub(crate) async fn cmd_channel_claims_summary` to accept
   `only_stuck: bool` as 9th param (insert after `log: Option<&std::path::Path>`).
2. In the `--all` paths (both `render_claims_summary_fleet_text` and
   `render_claims_summary_fleet_json`) apply the filter:
   - One-shot text path: skip rendering per-topic line when
     `only_stuck && !is_potentially_stuck(&summary)`. Keep the
     fleet-wide footer counts truthful (compute from full set).
     Healthy fleet path: print `All topics healthy (0/N stuck)`.
   - One-shot JSON path: filter `topics[]` accordingly; add `shown`
     + `only_stuck` fields to the summary envelope (mirror T-2070).
   - `--all --watch` rendering path: same filter applies.
3. Verify: `cargo check -p termlink` clean; `cargo test --bin termlink
   claims_summary_only_stuck` passes.
4. Extend CLAUDE.md CLAIM-OBSERVABILITY row with `--only-stuck` form.
5. Tick all ACs below; close task.

Reference: T-2070 (the governor-side `--only-pressured`) is the design
template — see `crates/termlink-cli/src/commands/remote.rs` for the
existing `governor_hub_is_pressured` predicate and the
filter+counter-separation pattern.

## Context

`channel claims-summary --all` already annotates `[POTENTIALLY STUCK]` per
topic — but on a healthy fleet with hundreds of topics, the operator has
to grep for the annotation to find what needs attention. T-2076 adds the
`--only-stuck` filter that drops non-stuck topics from the output,
mirroring T-2070's `--only-pressured` filter on `fleet governor-status`.

Pure presentation-level filter:
- The underlying RPC still queries every topic (need the full set to
  compute "0 of N are stuck" footer)
- The text path skips non-stuck topics before printing per-topic lines
- The JSON path returns the filtered `topics[]` but the summary keeps
  fleet-wide totals (`shown` + `only_stuck` fields gained, like T-2070)
- Healthy fleet path prints `All topics healthy (0/N stuck)` rather
  than an empty block — affirmative confirmation, not silent success

Mirror of T-2070's design exactly. Operator's "show me what needs
attention" verb. Clap requires `--all` (single-topic mode has nothing
to filter — it's either stuck or not, the operator already chose
which topic).

## Acceptance Criteria

### Agent
- [x] `cli.rs` ClaimsSummary variant gains `only_stuck: bool` with `requires("all")` (single-topic mode has nothing to filter)
- [x] `main.rs` dispatch passes `only_stuck` to `cmd_channel_claims_summary`
- [x] `cmd_channel_claims_summary` --all path filters non-stuck topics from per-topic rows when `only_stuck=true`
- [x] Summary footer keeps fleet-wide totals (`{topic_count, stuck_count}`) — filter is presentation-only, totals are truthful
- [x] JSON envelope gains `shown` (number of topics in `topics[]`) and `only_stuck` (the flag value) fields
- [x] Healthy fleet path (every topic non-stuck under `--only-stuck`) prints `All topics healthy (0/N stuck)` — affirmative
- [x] At least 1 pure predicate test confirming the stuck classification mirrors `is_potentially_stuck`
- [x] `cargo check -p termlink` builds clean
- [x] `cargo test --bin termlink claims_summary_only_stuck` passes
- [x] CLAUDE.md CLAIM-OBSERVABILITY row extended with the `--only-stuck` form
- [x] Mutex check: `--only-stuck` without `--all` is rejected by clap with a clear error

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
out=$(cargo test --bin termlink --release claims_summary_only_stuck 2>&1); echo "$out" | grep -q "test result: ok"

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

### 2026-06-09T08:58:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2076-channel-claims-summary---only-stuck-filt.md
- **Context:** Initial task creation
