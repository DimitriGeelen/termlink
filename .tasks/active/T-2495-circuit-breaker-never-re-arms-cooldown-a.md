---
id: T-2495
name: "circuit breaker never re-arms cooldown after failed half-open probe (protection evaporates)"
description: >
  CircuitState::record_failure (crates/termlink-hub/src/circuit_breaker.rs:41) guards the opened_at stamp with '&& self.opened_at.is_none()', so a failure on the half-open PROBE never re-stamps opened_at. After the first 60s cooldown, is_half_open() stays true forever => should_skip()=false forever => the breaker is silently defeated: every orchestrator.route to a dead session re-pays the full timeout (the cascading-delay it exists to prevent). Only a success ever re-closes it, which a dead session can't produce. Fix: drop the is_none() guard so any failure at/over threshold re-arms the cooldown. Applies to both CircuitBreakerRegistry + ModelCircuitBreaker (shared CircuitState). directive-#2 silent-failure.

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
created: 2026-08-02T09:54:20Z
last_update: 2026-08-02T09:54:20Z
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

# T-2495: circuit breaker never re-arms cooldown after failed half-open probe (protection evaporates)

## Context

The per-session/per-model circuit breaker (`crates/termlink-hub/src/circuit_breaker.rs`)
protects `orchestrator.route` (router.rs:1428) from cascading timeouts on dead sessions.
`should_skip` returns `false` for a half-open circuit to allow ONE probe; if that probe
fails, `record_failure` must re-open (re-arm the 60s cooldown). It does not: the
`&& self.opened_at.is_none()` guard means `opened_at` is stamped only the first time,
so after the first cooldown a dead session is permanently half-open and the breaker is
silently defeated. Verified in code: `record_failure` fires only on an *attempted* route
(router.rs:1511/1528), never during the skipped/open window, so re-stamping on every
threshold-failure is correct breaker semantics with no regression.

## Acceptance Criteria

### Agent
- [x] `CircuitState::record_failure` re-stamps `opened_at = Some(now)` on any failure at/over `FAILURE_THRESHOLD` (drop the `&& self.opened_at.is_none()` guard), so a failed half-open probe re-arms the cooldown
- [x] Fix lives in the shared `CircuitState` so it covers BOTH `CircuitBreakerRegistry` (session) and `ModelCircuitBreaker` (model)
- [x] New regression test: open a circuit, backdate `opened_at` to `now - COOLDOWN - 1s` (half-open), call `record_failure`, assert `should_skip` is `true` again (fails on current code, passes after fix)
- [x] All existing `circuit_breaker` tests still pass (no behavior change below threshold or on the success/close path)

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

cargo test -p termlink-hub --lib circuit_breaker
cargo check -p termlink-hub

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

**Symptom:** After a session's circuit opens and its first 60s cooldown expires, the
breaker never skips that session again. Every `orchestrator.route` to the (still-dead)
session re-pays the full connect `timeout` (default 5s) before failing over — the exact
cascading-timeout the breaker exists to prevent — with only a `debug!` line, no surfaced
error.

**Root cause:** `CircuitState::record_failure` stamped `opened_at` only when
`opened_at.is_none()` — i.e. only on the *first* opening. A failed half-open probe
(which happens precisely when `opened_at` is already `Some`) therefore left `opened_at`
pinned at its original time, so `is_half_open()` (`elapsed() >= COOLDOWN`) stayed `true`
forever and `should_skip()` returned `false` forever. The half-open→failure transition
had no re-arm; only `record_success` (which a dead session can never emit) could reset it.

**Why structurally allowed:** the unit tests covered opening, closing on success, the
below-threshold path, and reaching half-open — but never the half-open-**then-fails**
re-probe transition, the one path the guard broke. A one-line "only open once" guard
looked like an idempotency optimization; nothing exercised the case where re-stamping is
load-bearing.

**Prevention:** a regression test drives the exact transition (half-open → record_failure
→ must skip again). Learning captures the class: a circuit breaker's failure handler must
(re)open on EVERY threshold failure, not just the first — "open once" is never correct for
a state that decays with time.

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

### 2026-08-02T09:54:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2495-circuit-breaker-never-re-arms-cooldown-a.md
- **Context:** Initial task creation
