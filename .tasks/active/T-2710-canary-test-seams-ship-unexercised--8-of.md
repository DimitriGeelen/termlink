---
id: T-2710
name: "Canary test seams ship unexercised — 8 of 9 have no fixture suite"
description: >
  Nine runtime canaries ship a *_TEST_JSON seam (PL-213) explicitly so their exit-code contract can be verified without a live hub, but only check-charter-drift has a fixture suite that uses it. The other eight seams exist and nothing runs them — dormant tooling (PL-168) one level up. Concretely: the stuck-claims canary's HEALTHY path had never been asserted, which is why it could fire daily for ~62 days on latched debris (T-2709) with no test noticing. Adds tests/stuck-claims-check-fixtures.sh and files the remaining seven.

status: captured
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
created: 2026-08-14T16:59:52Z
last_update: 2026-08-14T16:59:52Z
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

# T-2710: Canary test seams ship unexercised — 8 of 9 have no fixture suite

## Context

Found while closing T-2709 (the stuck-claim latch), asking the G-019 question:
not "why did the predicate break" but "why was the framework blind to it for
~62 days?"

The answer is that the canary was never executed by any test. It ships a
`TERMLINK_STUCK_CLAIMS_TEST_JSON` seam — added in T-2556 specifically so its
verdict could be verified without a live hub — and nothing ever used it. That
is PL-168's "a canary without a trigger is dormant tooling", one level up: here
the *test seam* is the dormant thing.

**Measured state — 9 canaries carry a test seam, 1 has a fixture suite:**

| canary | seam | fixture suite |
|---|---|---|
| `check-charter-drift-freshness.sh` | ✅ | ✅ `charter-drift-check-fixtures.sh` |
| `check-stuck-claims-freshness.sh` | ✅ | ✅ **added here** (T-2710) |
| `check-fleet-binary-freshness.sh` | ✅ | ❌ |
| `check-forever-archival-freshness.sh` | ✅ | ❌ |
| `check-topic-growth-freshness.sh` | ✅ | ❌ |
| `check-session-control-freshness.sh` | ✅ | ❌ |
| `check-dead-letter-freshness.sh` | ✅ | ❌ |
| `check-waker-liveness-freshness.sh` | ✅ | ❌ |
| `check-unconfirmed-delivery-freshness.sh` | ✅ | ❌ |

**Honest scope limit — what this does and does not buy.** A fixture feeds the
canary canned JSON, so it verifies the canary's *translation* of a hub verdict
into an exit code. It would NOT have caught T-2709 itself, because that defect
lived upstream in the CLI predicate that computes `stuck_count`. Claiming
otherwise would be exactly the over-scoped green this session has been
correcting elsewhere (T-2680).

What it does buy is real: the exit-code contract every canary depends on
(`0 healthy / 1 firing / 2 tooling`) is currently unverified for 8 of 9, and
that contract is load-bearing — T-2557 states plainly that keeping tooling
errors out of the firing class is what makes a firing log meaningful, and T-2685
found the same distinction being thrown away in the crontab redirects. A canary
that returns 0 when it should return 2 reports a broken substrate as healthy.
Nothing tests that today.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The gap is measured, not estimated: 9 canaries carry a `*_TEST_JSON` / `*_TEST_RC` seam, 1 has a fixture suite, 8 do not
- [x] `tests/stuck-claims-check-fixtures.sh` exists and passes, covering the full exit-code contract: healthy (0), firing (1), and BOTH tooling paths (2) — malformed envelope, unparseable JSON, empty output
- [x] The healthy path is asserted, including that `--quiet` prints NOTHING on a healthy cycle — that silence is precisely what makes "empty log = healthy" true for the cron redirect, and it had never been tested
- [x] `--quiet` is also asserted to still speak when firing (a silent canary is a dead canary)
- [x] A fetch error (`ok:false` topic) is proven NOT to fire on its own while still being counted and warned about
- [x] A load-bearing T-2709 regression: a topic with `expired_count: 81` and `active_count: 0` that the hub reports NOT stuck must exit 0 — pinning that the canary keys on the hub's `stuck_count`/`potentially_stuck` verdict and never on `expired_count`
- [x] The converse is also pinned — the same topic DOES fire when the hub flags it — so the fixture cannot be satisfied by ignoring expired topics wholesale
- [x] Fixtures run with `--no-heartbeat` so a test run can never refresh the real cron heartbeat and mask a dead cron from the T-1723 meta-canary
- [x] The suite is picked up by `scripts/run-guard-layer.sh` automatically via the `tests/*fixtures*.sh` naming convention (verified with `--list`)
- [x] `bash scripts/run-guard-layer.sh` passes with the new member included
- [ ] The remaining 7 unexercised seams are named in this task so the gap is visible rather than implied

### Human
- [ ] [REVIEW] Decide whether the remaining 7 canaries get fixture suites now or later
  **Steps:**
  1. Read the list of 7 in the Context section below
  2. Decide: build all 7 now, or file as backlog
  **Expected:** a call on scope — each is ~20 assertions of the same shape as the stuck-claims suite
  **If not now:** say so and they stay filed; the point of this AC is that they are not silently forgotten

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

### 2026-08-14T16:59:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2710-canary-test-seams-ship-unexercised--8-of.md
- **Context:** Initial task creation
