---
id: T-2503
name: "aggregator poll loop cursor stalls on missing next_seq — silent duplicate-event re-send storm"
description: >
  aggregator poll loop cursor stalls on missing next_seq — silent duplicate-event re-send storm

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-hub/src/aggregator.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-02T17:02:12Z
last_update: 2026-08-02T17:05:18Z
date_finished: 2026-08-02T17:05:18Z
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

# T-2503: aggregator poll loop cursor stalls on missing next_seq — silent duplicate-event re-send storm

## Context

The hub event aggregator's per-session long-poll loop
(`EventAggregator::add_session`, crates/termlink-hub/src/aggregator.rs:130)
advances its cursor only via `if let Some(next) = data["next_seq"].as_u64() {
cursor = next; }`. If a poll returns a **non-empty** `events` array but
`next_seq` is absent or non-integer, the `if let` silently no-ops → `cursor` does
not advance → the next 5 s poll re-fetches the **identical batch** and re-sends
every event to all subscribers (Watchtower, mirror-grid). With `next_seq`
persistently absent this is an **indefinite duplicate-event re-send storm**, with
no error surfaced — the same silent class T-2496 fixed one screen up (the RPC-error
hot-loop), left in the same function's `PollAction::Deliver` arm.

The fix is defensive robustness against a malformed/buggy/hostile upstream session
server: prefer the server-provided `next_seq`, but when it is absent-despite-
delivered-events, fall back to `max(delivered seq) + 1` so the loop always
advances past the batch it just delivered, and `warn!` once (a healthy server
always stamps `next_seq`). No wire/protocol change — internal loop hardening only,
factored into a pure, unit-testable helper per the T-2496 convention already in
this file.

No-silent-failures campaign (directive #2 Reliability) — sibling of
T-2496/T-2497/T-2498/T-2499/T-2500/T-2501/T-2502.

## Acceptance Criteria

### Agent
- [x] The Deliver arm advances the cursor via a pure helper `next_cursor(data, delivered_max_seq, current)` — `next_seq` when present, else `max(delivered_seq)+1` when events were delivered, else unchanged. (aggregator.rs — `next_cursor` + rewired Deliver arm)
- [x] When the `next_seq` fallback is used the loop emits a `tracing::warn!` (loud, not silent) naming the session — a well-behaved server always stamps `next_seq`. (warn! with session/from_cursor/to_cursor)
- [x] The pure helper is unit-tested: (a) `next_seq` present → used verbatim; (b) `next_seq` absent + events delivered → `max_seq+1` with fallback flag set; (c) `next_seq` absent + no events → cursor unchanged, no fallback; (d) `next_seq` present-but-non-integer → treated as absent (fallback). (4 `next_cursor_*` tests, all pass)
- [x] The existing `poll_action_*` tests stay green; event delivery semantics (which events reach subscribers) are unchanged — only cursor advancement is hardened. (8/8 aggregator tests; delivery loop unchanged except max-seq tracking)
- [x] `cargo test -p termlink-hub --lib aggregator` passes. (8 passed; full lib 458 passed, 0 failed)

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

out=$(cargo test -p termlink-hub --lib aggregator 2>&1); echo "$out" | grep -q "test result: ok"

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

**Symptom:** If a session's `event.subscribe` poll returns events but no valid
`next_seq`, the aggregator re-fetches and re-broadcasts the same batch every 5 s
indefinitely — a silent duplicate-event storm to every subscriber (Watchtower /
mirror-grid see the same events repeat forever), with no error logged.

**Root cause:** cursor advancement was gated solely on `if let Some(next) =
data["next_seq"].as_u64()`. A bare `if let` with no `else` means "malformed /
missing `next_seq`" silently maps to "do nothing" — the cursor stalls at its
current value, so the next poll's `since=cursor` re-requests the already-delivered
batch. There was no independent way to advance past events that were in fact
delivered.

**Why structurally allowed:** T-2496 hardened the same function's transport/RPC
*outcome* classification (via the pure `poll_action`) but scoped its fix to "which
outcomes back off vs deliver" — it did not touch the *payload-field extraction*
inside the Deliver arm, where this second bare-`if let` swallow lived. The Deliver
path implicitly trusts a well-formed upstream (`next_seq` always present), and no
test exercised a Deliver with events-but-no-next_seq.

**Prevention:** (1) The fix routes cursor advancement through a pure
`next_cursor` helper that falls back to `max(delivered seq)+1` when `next_seq`
is absent-despite-events, guaranteeing forward progress; a `warn!` makes the
malformed-upstream case loud. (2) Four unit tests pin the helper's behavior
(present / fallback / no-events / non-integer), so a regression to the bare
`if let` breaks a test. (3) Generalizes the T-2496 lesson: a bare `if let
Some(..)` on a payload field the loop's *progress* depends on is the same silent
class as a bare `if let Ok(..)` on an outcome — reduce it to an explicit,
tested decision.

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

### 2026-08-02T17:02:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2503-aggregator-poll-loop-cursor-stalls-on-mi.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-def4f43d
- **Timestamp:** 2026-08-02T17:05:23Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-02T17:05:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
