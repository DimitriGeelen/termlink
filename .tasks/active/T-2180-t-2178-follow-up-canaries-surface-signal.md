---
id: T-2180
name: "T-2178 follow-up: /canaries surface signal-bearing log line not misleading last row"
description: >
  T-2178 follow-up: /canaries surface signal-bearing log line not misleading last row

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
created: 2026-06-11T20:01:53Z
last_update: 2026-06-11T20:01:53Z
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

# T-2180: T-2178 follow-up: /canaries surface signal-bearing log line not misleading last row

## Context

Discovered at T-2175/T-2176/T-2177/T-2178 close-time smoke (this session):
`canary-status.sh classify()` uses `tail -n 1` of the log to render the
`↳ <latest_entry>` line. For canaries that emit one multi-line block per
cron run (e.g. fleet-doorbell-mail prints a `DRIFT` header, summary
counters, then one row per hub), the last line is typically a passing
per-row entry like `- local-test@127.0.0.1:9100: verdict=pass`. The
actually-actionable signal (`- laptop-141@...: verdict=setup-fail`) is
buried mid-log.

Reproduced this turn: `bash scripts/canary-status.sh` rendered
fleet-doorbell-mail FIRING with `↳ verdict=pass` — operator-misleading.
The bug is presentation-only; classification is correct (FIRING because
log mtime ≥ heartbeat mtime). Fix is to prefer a problem-signaling line
(fail|drift|stale|warn|error|behind, case-insensitive) over the literal
last line, falling back to first-non-empty-line of the recent tail when
no signal found.

## Acceptance Criteria

### Agent
- [x] `classify()` in `scripts/canary-status.sh` selects `latest_entry` from a signal-bearing line (fail|drift|stale|warn|error|behind, case-insensitive) when present in the recent tail.
- [x] Falls back to first non-empty line of recent tail when no signal-bearing line found (still better than the misleading last line).
- [x] Smoke against the current FIRING fleet-doorbell-mail log surfaces `setup-fail` (the laptop-141 line), not `verdict=pass` (the local-test last-row).
- [x] HEALTHY canaries with non-empty logs (e.g. release-mirror with historical drift entries) still render a sensible `↳` line — no regression to existing behavior.
- [x] Header docstring updated to document the signal heuristic so a future maintainer doesn't revert it.

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
out=$(bash scripts/canary-status.sh 2>&1); echo "$out" | grep -q "setup-fail"
out=$(bash scripts/canary-status.sh 2>&1); ! echo "$out" | grep -q "local-test.*verdict=pass"
grep -q "signal-bearing" scripts/canary-status.sh

## RCA

**Symptom:** `/canaries` rendered fleet-doorbell-mail FIRING with `↳ verdict=pass elapsed=216ms` (the local-test passing row at end of the per-run multi-line block), while the actual failure signal `setup-fail elapsed=8004ms` for laptop-141 was buried mid-log and never surfaced. Operator reading the digest sees a contradiction: "FIRING but the only entry shown says pass".

**Root cause:** `classify()` used `tail -n 5 | grep -v '^$' | tail -n 1` — naive last-non-empty-line. For canaries with per-row entries inside a per-run block (doorbell prints DRIFT/summary/N row entries), the last row is typically a passing entry; the actionable signal lives earlier in the block.

**Why structurally allowed:** Canary log formats are heterogeneous (mirror canary is short multi-line per run, doorbell canary has variable row counts, substrate-preflight is framed). The "show last line" heuristic worked for mirror canary by accident — `behind origin` happens to be the last line. The doorbell canary's row-ordered output broke it. No structural distinction between "summary line" and "row line" in plain text.

**Prevention:** This task — prefer signal-bearing lines (fail|drift|stale|warn|error|behind, case-insensitive) over literal last line. Falls back to first non-empty line of recent tail (which is typically the per-run header like `Fleet doorbell+mail health: DRIFT`) when no signal found.

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

### 2026-06-11T20:01:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2180-t-2178-follow-up-canaries-surface-signal.md
- **Context:** Initial task creation
