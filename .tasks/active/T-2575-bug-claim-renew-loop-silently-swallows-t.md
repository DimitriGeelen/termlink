---
id: T-2575
name: "BUG: claim-renew loop silently swallows transient failures until lease lapses"
description: >
  BUG observability: background claim-renew loop swallows transient renew failures at debug and retries silently until the lease lapses. Fix: warn once now_ms past claimed_until. From T-2468 sweep.

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
created: 2026-08-09T15:04:08Z
last_update: 2026-08-09T15:04:08Z
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

# T-2575: BUG: claim-renew loop silently swallows transient failures until lease lapses

## Context

Found in T-2468 silent-swallow sweep. The background renew loop
(`crates/termlink-session/src/claim_client.rs:660-667`, `spawn_renew_task`) logs a
transient renew failure at `debug!` and retries on the next tick — with NO check of
whether the lease deadline (`claimed_until`, published to a shared atomic) has
already passed. Across a prolonged hub blip the lease lapses hub-side (the slot can
reopen to another worker → double-work) while this loop keeps quietly retrying at
`debug!`; the holder is not warned that ownership is now in doubt. The holder only
finds out on the NEXT renew that returns NotFound/Expired/NotOwned (which breaks the
loop) — by which point the double-work may already have happened.

**Scope of THIS task (observability half only).** Escalate the transient-failure
log from `debug!` to `warn!` once `now_ms >= claimed_until` (the lease deadline has
passed → ownership in doubt), keeping `debug!` while still within lease (genuinely
transient). This is a pure logging-severity change — no behavior/contract change.
The deeper fix (actively signal the `LeasedClaim` holder that ownership is in doubt,
e.g. a flag the worker can consult before acting) touches the claim-ownership
contract and is filed separately as needs-human (T-2576).

## Acceptance Criteria

### Agent
- [x] The transient-renew-failure branch escalates to `warn!` (naming the claim_id,
      error, claimed_until, now) when `now_ms >= claimed_until` — lease deadline
      passed, ownership in doubt — and keeps `debug!` while still within lease.
- [x] The lapse decision is a pure, unit-testable predicate (not inlined `>=`
      buried in the loop) so it can be asserted at the boundary.
- [x] Load-bearing test on the predicate: at the deadline (`now == claimed_until`)
      and past it → at-risk (true); strictly before → not-at-risk (false). Proven
      load-bearing (flipping the comparison boundary breaks a case).
- [x] `cargo test -p termlink-session claim` green; no behavior/contract change
      (renew cadence + retry loop unchanged — only log severity + a predicate added).

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

out=$(cargo test -p termlink-session transient_renew_lease_at_risk_boundary 2>&1); echo "$out" | grep -q "1 passed"
grep -q "ownership of this offset is in doubt" crates/termlink-session/src/claim_client.rs
grep -q "fn transient_renew_lease_at_risk" crates/termlink-session/src/claim_client.rs

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

**Symptom:** During a prolonged hub blip the background renew loop kept retrying
at `debug!` while the lease deadline silently passed; the holder was never warned
that the hub may have reopened the offset to another worker (double-work risk).

**Root cause:** The transient-failure branch logged `debug!` unconditionally,
never comparing the current time to the last-known `claimed_until` — so a
"transient" failure that has actually outlived the lease was indistinguishable
from a benign one-tick blip.

**Why structurally allowed:** The renew loop was written for the common case
(one-off blip, next tick succeeds). The tail case (blip outlives the lease) shares
the same code path, and `debug!` made it invisible — no severity signal, so no
operator or canary could see a lease silently lapse under a held claim.

**Prevention (this task — observability half):** escalate to `warn!` once
`now_ms >= claimed_until` via a pure, unit-tested predicate
`transient_renew_lease_at_risk`, proven load-bearing by temp-flipping the boundary.
The deeper contract fix (actively signal the `LeasedClaim` holder so a careful
worker can stop acting on a doubtful claim) is filed as **T-2576** (needs-human —
touches the ownership contract). PL captured: a retry loop over a lease/TTL must
compare against the deadline and escalate severity once crossed — "keep retrying
quietly" past a lease expiry hides a correctness hazard (double-ownership).

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

### 2026-08-09T15:04:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2575-bug-claim-renew-loop-silently-swallows-t.md
- **Context:** Initial task creation
