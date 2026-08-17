---
id: T-2779
name: "session-selftest suite has no automated invocation path — guard layer cannot discover scripts/test-*.sh"
description: >
  session-selftest suite has no automated invocation path — guard layer cannot discover scripts/test-*.sh

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
created: 2026-08-17T06:06:37Z
last_update: 2026-08-17T06:06:37Z
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

# T-2779: session-selftest suite has no automated invocation path — guard layer cannot discover scripts/test-*.sh

## Context

Found while closing T-2695: the guard layer could not see `scripts/test-session-selftest.sh`
at all, so the suite T-2695 had just extended was about to lose its last invocation path.
The stranded file turned out to be one of 57. Filed separately from T-2695 per "one bug =
one task" — that task was about PTY stages; this is a guard-layer coverage defect.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/test-*.sh` and non-`*fixtures*` `tests/*.sh` can JOIN the guard layer via the same `# guard-layer: source` header marker a static check uses — membership stays opt-in, because many of these suites need a live hub/network and would violate the layer's "safe to run anywhere" contract
- [x] Un-marked suites are reported as **unclassified** rather than being invisible — this is the actual defect: the pre-existing note counted only `check-*.sh`, so 38 `scripts/test-*.sh` suites sat outside the accounting entirely while the runner reported `45/45 members clean`
- [x] The unclassified wording no longer says "check script(s)", since the set is no longer only check scripts
- [x] `scripts/test-session-selftest.sh` carries the marker and actually runs in the layer — it is hermetic (every case drives the PL-213 test seams; no live hub, no tmux)
- [x] No existing member changes kind, name, or invocation; the pre-existing 45 still run and still pass
- [x] `*fixtures*.sh` under `tests/` is not double-added by the new scan
- [x] `bash tests/guard-layer-runner-fixtures.sh` still passes, extended with cases covering marker-opt-in and unclassified-visibility for the new locations
- [x] Load-bearing, proven by mutation not assertion: removing the marker from `test-session-selftest.sh` drops it from members and moves it to unclassified; restoring returns it
- [x] The runner's `--help`/header documents that suites join by marker from both locations, so the next author does not have to read the discovery loop to learn how to opt in

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
       Conversion: this AC should be moved to ### Agent and this line added to
       ## Verification (herestring, not a pipeline — see the L-387 hint below):
         out=$(bin/fw reviewer T-XXX 2>&1 || true); grep -q "Overall:.*PASS" <<< "$out"
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
# Pipefail/SIGPIPE hint (L-387, corrected by T-2775): P-011 runs each command
# under `set -eo pipefail`. NEVER write `cmd | grep -q PATTERN`: it exits 141
# (SIGPIPE) when grep matches and closes stdin while the upstream is still
# writing — verification then "fails" BECAUSE the check succeeded, and the
# earlier the match, the more reliably it fails.
#
# USE ONE OF THESE — both measured rc=0 at 3M lines:
#     out=$(cmd 2>&1 || true); grep -q "PATTERN" <<< "$out"   # herestring (preferred)
#     test -n "$(cmd | grep -m1 PATTERN)"                     # pipeline inside $( )
#
# The herestring is preferred: a herestring spawns no producer process, so there
# is nothing to SIGPIPE and it cannot regress as output grows. In the second form
# the pipeline sits inside a command substitution, whose status is discarded — the
# OUTER `test` decides.
#
# DO NOT capture-then-pipe. This template previously prescribed
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"     # UNSAFE above ~64KB
# and it is size-dependent, not safe: `echo`/`printf` is a producer like any
# other, so once $out exceeds the pipe buffer it is still writing when `grep -q`
# exits and pipefail propagates 141. The capture bounds the DATA but does not
# remove the PRODUCER. Anything wrapping `cargo test`, `fleet doctor --json`, or a
# full log is already in that size range. (T-2775 measured this; 999-AEF L-613 and
# 050-email-archive PL-161 published the capture-then-pipe form before the
# correction — both have since adopted the herestring.)
#
# Corollary (T-2090): intermediate stages are just as fatal — `... | tail -3 |
# grep -q PAT` re-introduces the same risk. With a herestring the question does
# not arise; grep scans the whole captured string anyway.
#
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before the hint;
# T-2775 then measured 1490 exposed lines across 802 tasks despite the hint, which
# is why `scripts/check-verification-pipefail.sh` now enforces it structurally.
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

bash -n scripts/run-guard-layer.sh
bash tests/guard-layer-runner-fixtures.sh
bash scripts/run-guard-layer.sh
# The suite this task exists for must actually be a MEMBER, not merely present on disk.
# Asserted against JSON `members[]`, NOT against `--list` text: --list prints the
# unclassified names too, so a text grep would pass even if the suite had been dropped
# to unclassified — i.e. it would still read green under the exact regression this
# task exists to prevent.
out=$(bash scripts/run-guard-layer.sh --json 2>&1 || true); test "$(jq -r '[.members[] | select(.name=="test-session-selftest.sh")] | length' <<< "$out")" -eq 1
out=$(bash scripts/run-guard-layer.sh --json 2>&1 || true); test "$(jq -r '.members[] | select(.name=="test-session-selftest.sh") | .kind' <<< "$out")" = "suite"
out=$(bash scripts/run-guard-layer.sh 2>&1 || true); grep -q 'PASS .* test-session-selftest.sh' <<< "$out"
# The previously-invisible suites are now COUNTED. Pre-T-2779 the note said 20 and
# omitted every scripts/test-*.sh; anything at/below 20 means the scan regressed.
out=$(bash scripts/run-guard-layer.sh --json 2>&1 || true); test "$(jq -r '.summary.unclassified // 0' <<< "$out")" -gt 20
# Wording no longer claims the unclassified set is only check scripts.
out=$(bash scripts/run-guard-layer.sh 2>&1 || true); test -z "$(grep -o 'check script(s) carry no' <<< "$out")"

## RCA

**Symptom.** `scripts/test-session-selftest.sh` was referenced only by task Verification
blocks — T-2563 (already archived to `completed/`) and T-2695. Not by CI, not by the guard
layer. On T-2695's closure it would have reached **zero** automated invocation paths: 23
assertions covering the charter's "control terminal sessions" verb, run only if a human
happened to type the command.

**Root cause.** `run-guard-layer.sh` discovery enumerated exactly two shapes —
`scripts/check-*.sh` carrying a `# guard-layer: source` marker, and `tests/*fixtures*.sh`
by naming convention. `scripts/test-*.sh` matched neither, and no marker could rescue it
because the marker loop only ever iterated `check-*.sh`. This was not one stranded file:
**38** suites live under `scripts/test-*.sh`, plus several non-`*fixtures*` files in
`tests/`.

**Why structurally allowed.** The runner reported `45/45 members clean` alongside a note
that `20 check script(s) carry no marker`. That pairing reads as complete accounting — a
member count plus an explicit remainder. But the remainder counted only `check-*.sh`, so
57 suites were in neither number. They were not un-run-but-known; they were **invisible**,
and invisible is what stops anyone asking. Same shape as T-2680, where the charter-drift
canary reported `{checked:214, live_off_charter:0}` while only ever looking for six known
families: in both cases the *number was true* and the *impression was false*. A guard's
green is read as a claim about its whole surface, so a guard that cannot enumerate its
surface cannot be believed about it.

**Prevention.** Discovery now scans both new locations, so an un-marked suite is counted
as unclassified rather than dropped — the count moved 20 → 77, which is the defect made
visible. Membership stays marker-gated because most of these suites need a live hub and
would make a run-anywhere layer flaky; the fix is *visibility*, not mass enrolment.
`tests/guard-layer-runner-fixtures.sh` gains 10 cases pinning both halves (marked ⇒
member; unmarked ⇒ unclassified, not silently dropped), and the un-marked half is the
load-bearing one: stripping the marker from `test-session-selftest.sh` moves it 46/77 →
45/78, proving the marker — not the filename — is what confers membership.

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

### 2026-08-17 — visibility, not mass enrolment

- **Chose:** Scan the two new locations, but keep membership gated on the existing
  `# guard-layer: source` marker; un-marked suites become **unclassified** (counted and
  named) rather than members.
- **Why:** The guard layer's contract is "safe to run anywhere — no live hub, no network,
  no host state". Most of the 38 `scripts/test-*.sh` suites drive real transports
  (`test-agent-send.sh`, the relay suites, the doorbell suites); enrolling them wholesale
  would make the layer flaky, and a flaky guard is worse than a missing one because it
  trains its operator to stop reading it — the same dynamic recorded for the latched
  stuck-claims canary (T-2709). The defect was never "these do not run"; some legitimately
  cannot. It was that nothing said they existed.
- **Rejected:** (a) auto-enrolling every `test-*.sh` — makes the layer flaky and would have
  reported ERROR for hub-dependent suites on any machine without a hub, which is exactly
  the false-alarm class this layer exists to avoid; (b) renaming the suite to
  `tests/session-selftest-fixtures.sh` to satisfy the existing convention — that fixes one
  file and leaves 56 invisible, treating the symptom while the enumeration gap survives;
  (c) marking every hermetic suite in this task — real value, but unbounded scope for a
  task filed on a specific defect, and each needs its hermeticity *verified* rather than
  assumed. Only `test-session-selftest.sh` was marked, because this session actually ran
  it under the seams and watched all 23 cases pass without a hub.

### 2026-08-17 — the unclassified count is the deliverable

- **Chose:** Treat the count moving 20 → 77 as the headline result, and assert `> 20` in
  the gate rather than pinning an exact number.
- **Why:** 57 suites went from invisible to counted; that is the fix. An exact pin would
  fire on every legitimately-added suite and turn a real signal into churn, whereas `> 20`
  fires only if the new scan regresses — which is the property worth defending.
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

### 2026-08-17T06:06:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2779-session-selftest-suite-has-no-automated-.md
- **Context:** Initial task creation
