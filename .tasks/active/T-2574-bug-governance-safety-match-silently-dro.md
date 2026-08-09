---
id: T-2574
name: "BUG: governance safety-match silently dropped under channel backpressure"
description: >
  BUG: a detected governance/safety pattern match in session output is silently dropped (only a debug! line) when the governance channel is full or closed (governance_subscriber.rs:88, try_send is_err). The safety signal never reaches the enforcer with no warn/metric. Fix: elevate to warn! + a dropped-events counter + load-bearing test. Found in T-2468 silent-swallow sweep.

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
created: 2026-08-09T14:56:52Z
last_update: 2026-08-09T14:56:52Z
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

# T-2574: BUG: governance safety-match silently dropped under channel backpressure

## Context

Found in T-2468 silent-swallow sweep (Reliability directive: no silent failures).
`governance_subscriber.rs:88` drops a matched governance/safety event with only a
`debug!` line when `governance_tx.try_send` fails (channel full or receiver gone).
The governance frame is the coordination signal that a session emitted something
requiring intervention (e.g. a `FATAL ERROR` / secret-leak pattern) — on a brief
consumer stall (bounded channel → `Full`) it never reaches the enforcer, the
producing side moves on, and nothing at `warn`+ or any counter records it. Note
the SAME file already uses `warn!` for the Lagged-drop case (line 99) — the
try_send drop is the inconsistent one. Fix: elevate to `warn!` and add a monotonic
`dropped_events` counter so the loss is observable.

## Acceptance Criteria

### Agent
- [x] The try_send-drop branch elevates from `debug!` to `warn!` AND increments a
      monotonic `dropped_events` counter on the subscriber, so a dropped
      governance/safety match is both loud and countable (parity with the existing
      Lagged-drop `warn!`).
- [x] The counter is observable (a getter) for tests/operators.
- [x] Load-bearing test: with a capacity-1 governance channel held full, a matching
      output line drives a try_send failure and the `dropped_events` counter
      increments; proven load-bearing (removing the increment makes the test fail).
- [x] `cargo test -p termlink-session governance` green.

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

cargo test -p termlink-session dropped_governance_event_is_counted_not_silent 2>&1 | grep -q "1 passed"
grep -q "SAFETY event dropped" crates/termlink-session/src/governance_subscriber.rs
grep -q "dropped_events.fetch_add" crates/termlink-session/src/governance_subscriber.rs

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

**Symptom:** A governance/safety pattern (e.g. `FATAL ERROR`, a secret-leak
regex) matched in session output but did not reach the enforcer when the bounded
governance mpsc was momentarily full or its receiver gone — the producing side
moved on with only an invisible `debug!` trace; no operator-visible signal, no
count.

**Root cause:** The `try_send` failure branch (`governance_subscriber.rs`) logged
at `debug!` and did nothing else — no counter, no `warn!`. The same file's
Lagged-drop branch already used `warn!`, so the drop-visibility convention existed
but was applied inconsistently to the one branch that drops a matched safety event.

**Why structurally allowed:** Non-blocking `try_send` correctly chooses to drop
rather than block the output pump, but "drop silently" was conflated with "drop
best-effort." A dropped *cosmetic* event is best-effort; a dropped *safety-match*
event is a lost coordination signal — the two were not distinguished, and nothing
counted the loss so no canary/metric could ever surface it.

**Prevention:** (1) Fix — elevate to `warn!` and increment a monotonic
`dropped_events` counter (loud + countable). (2) Load-bearing regression test
`dropped_governance_event_is_counted_not_silent`, proven via temp-revert (removing
the `fetch_add` fails it). (3) The counter is now an observable getter, so a future
canary/metric can gate on `dropped_events > 0`. PL captured on the class:
non-blocking `try_send` drops on the SAFETY/coordination path must be counted +
`warn!`, never `debug!` — "non-blocking" is not license for "silent."

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

### 2026-08-09T14:56:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2574-bug-governance-safety-match-silently-dro.md
- **Context:** Initial task creation
