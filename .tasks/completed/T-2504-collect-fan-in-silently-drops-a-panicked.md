---
id: T-2504
name: "collect fan-in silently drops a panicked session poll task (JoinError swallowed, no log)"
description: >
  collect fan-in silently drops a panicked session poll task (JoinError swallowed, no log)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-hub/src/router.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-02T17:09:37Z
last_update: 2026-08-02T17:12:58Z
date_finished: 2026-08-02T17:12:58Z
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

# T-2504: collect fan-in silently drops a panicked session poll task (JoinError swallowed, no log)

## Context

`handle_event_collect` (crates/termlink-hub/src/router.rs:665) fans out one
`event.collect` RPC per session into a `JoinSet`, then drains results with
`while let Some(result) = join_set.join_next().await { if let Ok(Some((sid,
events, next_seq))) = result { ... } }`. `result` is
`Result<Option<..>, JoinError>`. The bare `if let Ok(Some(..))` silently discards
**two** cases: `Ok(None)` (session unreachable/timeout — already logged inside the
task, fine to drop) AND **`Err(JoinError)`** (the collector task itself **panicked**
or was cancelled). The panic case is logged **nowhere** — that session's events
silently vanish from the collect pass and the operator gets a partial result set
with no indication a task died.

The fix routes the join outcome through a pure `collect_join_action` classifier
(the T-2496 convention already used in `aggregator.rs`) so the three cases are an
explicit, tested decision, and adds a `tracing::warn!` on the `Err` arm — a
panicked poll task is surfaced loudly instead of swallowed. A real `JoinError`
is constructible in a test by joining a panicking task in a `JoinSet`, so the
classifier is unit-testable without a live hub. No wire/protocol change.

No-silent-failures campaign (directive #2 Reliability) — sibling of
T-2496…T-2503.

## Acceptance Criteria

### Agent
- [x] The collect drain loop routes each join result through a pure `collect_join_action` classifier: `Ok(Some)` → deliver, `Ok(None)` → skip (already logged in-task), `Err(JoinError)` → failed. (router.rs — `collect_join_action` + rewired match)
- [x] The `Err(JoinError)` case emits a `tracing::warn!` naming the failure (a panicked/cancelled collector task is surfaced, not silently dropped); event-delivery semantics for the `Ok(Some)`/`Ok(None)` cases are unchanged. (warn! on the Failed arm; Deliver/Skip logic identical)
- [x] The classifier is unit-tested: a real panicking task joined via a `JoinSet` yields `Err` → classified `Failed`; `Ok(Some)` → `Deliver`; `Ok(None)` → `Skip`. (2 `collect_join_action_*` tests, real panic → JoinError)
- [x] `cargo test -p termlink-hub --lib` passes (existing router behavior unbroken). (460 passed; 0 failed)
- [x] `cargo test -p termlink-hub --lib collect_join` passes (new classifier tests). (2 passed)

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

out=$(cargo test -p termlink-hub --lib collect_join 2>&1); echo "$out" | grep -q "test result: ok"
out=$(cargo test -p termlink-hub --lib 2>&1); echo "$out" | grep -q "test result: ok"

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

**Symptom:** If a per-session `event.collect` poll task panics (or is
cancelled), that session's events are silently omitted from the aggregated
collect result and nothing is logged — the caller (Watchtower / a collect
consumer) receives a partial set with no signal that a session's task died.

**Root cause:** the drain loop used `if let Ok(Some((sid, events, next_seq))) =
result`, where `result: Result<Option<..>, JoinError>`. A bare `if let Ok(Some)`
collapses the `Err(JoinError)` (task panic/cancel) into the same silent no-op as
`Ok(None)` (a benign unreachable-session that was already logged inside the task).
The two are semantically opposite — one is expected and pre-logged, the other is
an unexpected task death that should be surfaced — but the pattern treats them
identically and loudly logs neither.

**Why structurally allowed:** `JoinSet` drain loops idiomatically written as
`if let Ok(Some(..))` read as harmless because the happy path is obvious and the
`Err` arm is rarely hit; without a test exercising a panicking task, the swallow
is invisible. This is the same bare-`if let` silent class the campaign has been
closing across the hub (T-2496 outcome, T-2503 payload field) — here on a
`JoinSet` result.

**Prevention:** (1) The fix routes the join outcome through a pure
`collect_join_action` classifier making the 3-way decision explicit, and adds a
`warn!` on the `Err` arm so a panicked collector task is surfaced. (2) A unit test
constructs a real `JoinError` (join a panicking task in a `JoinSet`) and asserts
it classifies as `Failed`, plus `Ok(Some)`→`Deliver` / `Ok(None)`→`Skip`, so a
regression to the bare `if let` breaks a test. (3) PL-289 already generalizes the
bare-`if let`-that-silently-drops lesson; this extends it to `JoinSet` drains.

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

### 2026-08-02T17:09:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2504-collect-fan-in-silently-drops-a-panicked.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-5854c8f8
- **Timestamp:** 2026-08-02T17:13:54Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-02T17:12:58Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
