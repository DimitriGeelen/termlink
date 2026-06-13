---
id: T-2068
name: "fleet governor-history retrospective verb (Track G read-side, T-2018 §6 #10 closure)"
description: >
  fleet governor-history retrospective verb (Track G read-side, T-2018 §6 #10 closure)

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
created: 2026-06-09T06:29:16Z
last_update: 2026-06-09T06:29:16Z
date_finished: 2026-06-09T07:12:32Z
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

# T-2068: fleet governor-history retrospective verb (Track G read-side, T-2018 §6 #10 closure)

## Context

Closes T-2028 §6 #10 substrate-governor arc by shipping the read-side
companion to T-2066's Track G `--log <PATH>` audit trail. Mirror of T-1671
`fleet history` (which reads `rotation.log`) but pointed at `governor.log`.
Read-only, no auth, no network — answers "has this hub been
backpressured / rate-limited recently?" without keeping a watch terminal
open. Was explicitly called out as deferred read-side in T-2066/T-2067
docs.

## Acceptance Criteria

### Agent
- [x] `termlink fleet governor-history` CLI subcommand exists, accepts `--since` (1..=365 default 7), `--hub <NAME>`, `--log <PATH>` (override default), and `--json` flags; out-of-range `--since` errors with a useful message.
- [x] Default path resolves to `$HOME/.termlink/governor.log`; missing log prints a hint pointing at `fleet governor-status --watch --log <path>` (NOT "rotation" hint — must reference the governor watch command).
- [x] Reading a populated log renders one human-format line per matching entry (`<ts>  <hub>  <kind>  conn=A→B cap=X→Y(+d) rate=X→Y(+d) dedupe=…`), filters by `--since` window AND `--hub` if set, and prints a per-hub aggregate footer summing `cap_hits_delta`, `rate_hits_delta`, `dedupe_hits_delta`.
- [x] `--json` mode emits one NDJSON line per matching entry plus a single summary JSON object on a final line (shape symmetric to T-1671 `fleet history --json`: `{total, per_hub, since_days, hub_filter, malformed_lines_skipped, log_path}`).
- [x] Malformed NDJSON lines are skipped with stderr warnings (first 3) and counted in `malformed_lines_skipped`; the command never panics on garbage input.
- [x] Pure render helper `render_governor_history_line` extracted and covered by ≥2 unit tests (full-data line + dedupe-null line); `cargo test -p termlink --release --bin termlink render_governor_history_line` passes.
- [x] `cargo build -p termlink --release` succeeds.
- [x] CLAUDE.md BACKPRESSURE row updated with the new verb; `docs/operations/substrate-governor.md` gains a "Recipe — `fleet governor-history`" section.

## Verification

cargo build -p termlink --release 2>&1 | tail -5
cargo test -p termlink --release --bin termlink render_governor_history_line 2>&1 | tail -5
grep -q "fleet governor-history" CLAUDE.md
grep -q "Recipe — \`fleet governor-history\`" docs/operations/substrate-governor.md

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

### 2026-06-09T06:29:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2068-fleet-governor-history-retrospective-ver.md
- **Context:** Initial task creation
