---
id: T-2576
name: "DESIGN: signal LeasedClaim holder when ownership is in doubt (renew contract half)"
description: >
  DESIGN needs-human: the claim-renew loop observability half shipped (T-2575 warns when the lease deadline passes). The deeper fix is a CONTRACT change: actively signal the LeasedClaim holder that ownership is in doubt (e.g. an at-risk flag the worker consults before acting) so a careful worker stops acting on a doubtful claim instead of only finding out on the next failed renew. Touches the claim-ownership contract consumed by worker code — human/design decision. From T-2468 sweep, paired with T-2575.

status: captured
workflow_type: build
owner: human
horizon: later
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-09T15:10:18Z
last_update: 2026-08-09T15:10:18Z
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

# T-2576: DESIGN: signal LeasedClaim holder when ownership is in doubt (renew contract half)

## Context

Filed from T-2468 silent-swallow sweep, paired with T-2575 (which shipped the
OBSERVABILITY half — the renew loop now `warn!`s once `now_ms >= claimed_until`).
This task is the deeper CONTRACT half. Today a `LeasedClaim` holder learns its
lease lapsed only when the NEXT renew returns NotFound/Expired/NotOwned and breaks
the loop — by which point another worker may already have claimed the reopened
offset and done duplicate work. The holder has no way to consult "is my ownership
currently in doubt?" before acting on the claimed unit.

Needs-human because it touches the claim-ownership contract consumed by worker
code (`crates/termlink-session/src/claim_client.rs` `LeasedClaim` +
`examples/parallel_worker.rs`): adding an at-risk signal changes what a correct
worker must check, and the exact semantics (advisory flag vs hard stop; how a
worker recovers) is a design decision, not a mechanical fix.

## Acceptance Criteria

### Human
- [ ] [REVIEW] Decide the contract shape: an at-risk signal on `LeasedClaim` (e.g.
      `ownership_in_doubt()` derived from `now >= claimed_until`, or a state the
      renew loop sets on repeated transient failure past the deadline) that a
      worker consults before acting — advisory vs mandatory, and how the worker is
      expected to recover (re-claim, abort unit, checkpoint).
- [ ] [REVIEW] Decide whether the renew loop should also proactively surface this to the
      operator/orchestrator (beyond T-2575's warn!) — e.g. a metric or presence
      annotation — or whether the holder-side check is sufficient.
- [ ] [RUBBER-STAMP] If "change the contract", file a build task with concrete ACs + a
      load-bearing test (a worker that sees ownership-in-doubt stops acting); if
      "warn! is sufficient", record that T-2575 closes the practical risk and this
      is knowingly deferred.

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

## Recommendation

**Recommendation:** GO on the advisory half only — expose the at-risk predicate the
code already computes, and make the example worker consult it. Leave
advisory-vs-mandatory as a separate, later decision.

**Rationale:** The task was filed as an expensive contract change. Reading the code
today, the plumbing already exists and the remaining work is a few lines. The
renew task publishes `claimed_until` into a shared atomic; `LeasedClaim` already
exposes it publicly; and the exact at-risk predicate is already written and in use
— it is just private and wired only to a `warn!`. So the holder-side signal is not
a design problem, it is an accessor. What genuinely remains a human decision is the
*semantics* (advisory flag vs hard stop, and how a worker recovers), and that half
does not need to block the cheap half.

**Evidence (measured 2026-08-27, `crates/termlink-session/src/claim_client.rs`):**
- `pub fn claimed_until(&self) -> i64` at **line 599–601** — already public, reads
  `Arc<AtomicI64>` that the renew task stores into at line 679. So a worker can
  compute its own at-risk state today, from the public API, with no contract change
  at all.
- `fn transient_renew_lease_at_risk(now_ms, claimed_until) -> bool { now_ms >= claimed_until }`
  at **line 662–663** — the predicate the task describes as needing design is
  already written and documented (inclusive boundary, matching the hub's own
  `claimed_until <= now` expiry). It is private and consumed only by the T-2575
  `warn!` at line ~697.
- `crates/termlink-session/examples/parallel_worker.rs:170` — the canonical worker
  sleeps for `FAKE_WORK_MS`, then acks. It **never consults `claimed_until()`**.
  Its comment even asserts "long sleeps wouldn't lose the slot", which is the belief
  this task exists to correct. The example teaches the unsafe pattern.
- Whether duplicate work has actually occurred in the field: **not measured**. The
  risk is structural, not an observed incident.

**What you are actually deciding.** The task bundles two questions of very
different cost. Separating them:

| Option | Cost |
|---|---|
| **Advisory accessor + fix the example** (recommended GO) | Small and additive: promote the existing predicate to a public `ownership_in_doubt()`, have the example check it before `ack()`, add a test that a lapsed lease reports true. Nothing existing breaks. |
| Mandatory hard-stop (`ack()` refuses when in doubt) | Changes what a correct worker is, and can refuse an ack for work that genuinely completed inside the lease. Needs your call. |
| Do nothing; T-2575's `warn!` is enough | Free, but the warning goes to the operator, not to the worker — the holder still cannot ask, and the example still teaches the unsafe pattern. |

**Why I should not decide the second row alone.** Whether a doubtful claim must
*stop* a worker is a correctness-vs-availability trade in your workload: a hard stop
prevents duplicate work at the price of discarding work that actually finished in
time. That depends on whether your units are idempotent, which is a property of the
work, not of the substrate.

**If you take the GO:** the third Human AC asks for a build task with concrete ACs
and a load-bearing test. The load-bearing one is the example: a worker that observes
ownership-in-doubt and declines to ack, failing if the accessor is reverted. Note
that this recommendation narrows the filed scope — the mandatory half stays open,
and should be recorded as such rather than treated as closed by the cheap half
shipping.

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

### 2026-08-09T15:10:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2576-design-signal-leasedclaim-holder-when-ow.md
- **Context:** Initial task creation
