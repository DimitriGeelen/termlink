---
id: T-2624
name: "list-topics handler omits unreachable sessions from total_topics (silent session
  omission)"
description: >
  list-topics handler omits unreachable sessions from total_topics (silent session
  omission)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/events.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-11T22:54:59Z
last_update: '2026-08-18T18:59:14Z'
date_finished: 2026-08-12T06:11:22Z
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
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:54Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 3
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=3 
      (body:component-silent-failure); D3=2 (body:default-change); D4=2 
      (body:env-class-handled); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:14Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2624: list-topics handler omits unreachable sessions from total_topics (silent session omission)

## Context

**FILED, NOT BUILT** (reliability-hunt-2 Finding #4, LOW confidence, async
fan-out over live sessions — not cleanly unit-testable without extracting the
per-session aggregation into a pure helper first, and lower-impact than a
health-gate silent-pass because this is a diagnostic list command).

`crates/termlink-cli/src/commands/events.rs:1043-1075` — the list-topics
handler fans out an `event.topics` RPC to every registered session under a
`timeout_secs` bound, then:
- line 1059 `Ok(Err(_)) | Err(_) => continue` — drops any session that
  timed out or whose transport errored;
- line 1047 `if let Ok(result) = client::unwrap_result(resp)` — drops any
  session whose RPC returned an error *result* (the `else` is empty);
- line 1048 `&& let Some(topics)` — drops any response lacking a `topics` array.

None of those three drop paths is counted. `total` (1063) and
`total_sessions` (1074) reflect ONLY the sessions that answered with a
non-empty topic list. The JSON envelope and human summary carry no
"N session(s) unreachable/errored" field. Result: an operator running
list-topics against a fleet where some sessions are wedged sees a smaller
topic set with no indication that anything was skipped — a silent omission
(Directive #2 "no silent failures / observable"). Distinct from a wrong
green pass: the danger is a *false-complete inventory*, not a false-clean gate.

## Acceptance Criteria

### Agent
- [x] The per-session fan-out accumulates an `unreachable`/`errored` count (sessions dropped at 1059) AND a `no_topics_field`/`bad_result` count (dropped at 1047/1048), distinguished if cheap
- [x] The JSON envelope gains `sessions_unreachable` (and/or a combined `sessions_skipped`) so a consumer can tell the inventory is partial
- [x] The human-mode summary prints a "N of M session(s) unreachable — inventory may be incomplete" line when any session was skipped
- [x] The topic-aggregation logic is extracted into a pure helper that takes per-session `Result`-like outcomes and returns `(session_topics, skipped_count)` so the skip-counting path is unit-testable (prerequisite — the current inline async loop is not testable)
- [x] Regression test: a fixture with 5 session outcomes where 2 are `Err`/timeout yields `skipped==2` and a `total_topics` covering only the 3 reachable, plus the summary/envelope surfaces the 2 skips
- [x] Test proven load-bearing via temp-revert (restore the bare `continue` with no counter → skipped==0 → test fails)

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

cargo test -p termlink --bins aggregate_topics_probes

## RCA

**Symptom:** `list-topics` against a fleet where some registered sessions are wedged (timeout) or errored returns a topic inventory covering only the sessions that answered, with no indication that others were skipped — the operator reads a smaller-than-real inventory as complete.

**Root cause:** the per-session fan-out loop (`events.rs:1043-1061`) has three silent drop paths — timeout/transport error (`Ok(Err(_)) | Err(_) => continue`, 1059), error RPC result (`if let Ok(result) = ...`, 1047, empty else), and missing `topics` array (`&& let Some(topics)`, 1048) — none of which increments any counter. `total` (1063) and `total_sessions` (1074) are computed purely from `session_topics`, which by construction excludes every dropped session.

**Why structurally allowed:** the loop uses `continue` and `if let` early-exits to skip failures rather than accumulating a skip tally, so the "how many did we not reach?" signal is discarded at the point of failure. The aggregation is inline in an async loop with no pure-helper seam, so no test exercised the partial-fleet path. Same observability-gap family as T-2619/T-2621/T-2623 (silent laundering of failures into a clean-looking count), here for a fan-out inventory rather than a scalar sum.

**Prevention:** extract the per-session outcome→aggregate step into a pure helper returning `(session_topics, skipped_count)`; surface `sessions_unreachable` in the JSON envelope and an "N of M session(s) unreachable — inventory may be incomplete" line in human mode; unit-test the partial-fleet path. Failure scenario: 5 registered sessions, 2 time out under `timeout_secs` → output shows topics for 3 with `total_topics`/`total_sessions` covering only those 3, no note that 2 were unreachable.

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

### 2026-08-11T22:54:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2624-list-topics-handler-omits-unreachable-se.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-8edee4a3
- **Timestamp:** 2026-08-12T06:11:53Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-12T06:11:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
