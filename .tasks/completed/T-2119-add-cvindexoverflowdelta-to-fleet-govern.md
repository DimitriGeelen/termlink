---
id: T-2119
name: "Add cv_index_overflow_delta to fleet governor-status watch/notify/log/history (T-2118 observability follow-up)"
description: >
  Add cv_index_overflow_delta to fleet governor-status watch/notify/log/history (T-2118 observability follow-up)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-parallel-substrate, substrate-primitive-10, cv-index-pressure]
components: [crates/termlink-cli/src/commands/remote.rs, crates/termlink-mcp/src/tools.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-10T09:38:45Z
last_update: 2026-06-10T09:53:46Z
date_finished: 2026-06-10T09:53:46Z
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

# T-2119: Add cv_index_overflow_delta to fleet governor-status watch/notify/log/history (T-2118 observability follow-up)

## Context

Direct follow-up to T-2118. T-2118 shipped the static predicate axis (cv_index_overflow_total > 0 fires `--only-pressured`); this slice closes the **observability loop** at the watch/notify/log/history layer.

Current state: `fleet governor-status --watch` tracks cap_hits / rate_hits / dedupe_hits as transition events with old/new/delta exposed via change-line render, `--notify` env vars (TERMLINK_GOV_*_DELTA), `--log` NDJSON entries, and `fleet governor-history` aggregation (including the MCP `termlink_fleet_governor_history` mirror). cv_index_overflow_total exists in the underlying RPC response (T-2110) and is now surfaced in `--only-pressured` (T-2118) — but **its delta is never captured as a transition event**. An operator can SEE that overflow > 0 today, but they cannot get PAGED the moment a producer first starts mis-emitting cv_key.

Pattern parity: T-2065 introduced the `--notify` env-var contract for cap_hits/rate_hits; T-2066 the `--log` NDJSON schema; T-2068 the history aggregation; T-2069 the MCP history mirror. This slice mirrors that same arc one more time for cv_index_overflow_total. Closes the cv_index pressure observability loop: T-2110 (telemetry) → T-2118 (static predicate) → T-2119 (transition events + audit + history).

WatchGovernorState today: `(bool, i64, i64, i64, i64, Option<i64>)` = (reach, conn_active, conn_max, cap_hits, rate_hits, dedupe_hits). This slice extends it to 7 fields adding cv_overflow as `Option<i64>` (Option for pre-T-2110-hub backward compat — same shape as dedupe_hits which was added T-2049).

## Acceptance Criteria

### Agent
- [x] `WatchGovernorState` tuple extended to include `cv_index_overflow_total: Option<i64>` (`None` for pre-T-2110 hub envelopes that lack the field)
- [x] Change-line render (`render_governor_watch_change_line`) emits `cv_overflow=A→B(+delta)` segment, mirror of `cap_hits=` / `rate_hits=` / `dedupe_hits=` segments; `n/a` sentinel when either side is `None`
- [x] `build_governor_notify_env` emits `TERMLINK_GOV_OLD_CV_OVERFLOW` / `TERMLINK_GOV_NEW_CV_OVERFLOW` / `TERMLINK_GOV_CV_OVERFLOW_DELTA` env vars, schema-stable across all event kinds (`transition`/`new`/`removed`), empty string when either side missing (`[ -z "$VAR" ]` gate matches dedupe convention)
- [x] `build_governor_log_entry` writes `old_cv_overflow` / `new_cv_overflow` / `cv_overflow_delta` JSON fields with the same numeric-or-null convention as `dedupe_hits_delta`
- [x] `aggregate_governor_entries` (CLI in remote.rs) reads `cv_overflow_delta` and aggregates it as `cv_overflow_hits_total` in the per-hub footer; one new render column in `fleet governor-history` text output
- [x] `aggregate_governor_entries` (MCP mirror in tools.rs) parity: `GovernorHubAggMcp` adds `cv_overflow_hits` field; `termlink_fleet_governor_history` envelope's `per_hub` returns `cv_overflow_hits_total`
- [x] CLAUDE.md mega-table T-2069 entry updated to include `cv_overflow_hits_total` in the documented `per_hub` shape; T-2118 line extended with the full T-2119 observability-loop closure
- [x] 8 new unit tests (CLI side): cv_overflow render-line positive + n/a sentinel; notify_env positive + None/Some mix; log_entry positive + null serialization; aggregate sums cv_overflow_delta; render_governor_history_line renders cv_overflow segment
- [x] 1 new unit test (MCP side): aggregate sums cv_overflow_delta + pre-T-2119 backward compat
- [x] Backward compat: a log line missing `cv_overflow_delta` (pre-T-2119 entry) parses cleanly and contributes 0 to the aggregate (mirror of existing `dedupe_hits_delta` Option<i64> handling pattern at remote.rs:3057-3060)

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

out=$(cargo test -p termlink watch_governor 2>&1); echo "$out" | grep -q "test result: ok"
out=$(cargo test -p termlink build_governor_notify_env 2>&1); echo "$out" | grep -q "test result: ok"
out=$(cargo test -p termlink-mcp --lib governor 2>&1); echo "$out" | grep -q "test result: ok"

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

### 2026-06-10 — direct sequel to T-2118; observability loop closure
- **What changed:** T-2118 shipped the static predicate axis (`--only-pressured` fires on `cv_index_overflow_total > 0`). What was missing for a complete operator-actionable loop: the watch-loop transition events (change-line render, --notify env vars, --log NDJSON, --history aggregation) that surface the moment a producer FIRST mis-emits cv_key. This slice mirrors the existing cap_hits/rate_hits/dedupe_hits arc one more time for cv_overflow. The decision was a tuple extension (6 → 7 fields with `cv_overflow: Option<i64>` at position 6, mirroring the dedupe-Option pattern from T-2049). Same backward-compat treatment: pre-T-2110 hubs → None → "n/a" sentinel / null JSON / empty env var.
- **Plan impact:** Single-slice ship across 7 surfaces (tuple definition, render-line, notify env, log entry, aggregate, history footer, CLI history JSON, MCP history JSON) — bulk update of 16 existing test tuple literals via Python regex + 9 new positive tests for the cv_overflow axis.
- **Triggered:** Closes the §6 #9↔#10 observability cross-reference end-to-end (T-2110 telemetry → T-2118 static predicate → T-2119 transition events + audit + history). No further follow-ups needed at this layer. Future: if a slash-skill `/governor` exposes per-hub cv_overflow_hits_total in its presentation layer, that's a separate operator-UX slice.

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

### 2026-06-10T09:38:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2119-add-cvindexoverflowdelta-to-fleet-govern.md
- **Context:** Initial task creation

### 2026-06-10T09:53:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
