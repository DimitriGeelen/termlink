---
id: T-3440
name: "WARN escalation: check-audit-warning-acknowledgement.sh red 14d (since 2026-09-30T00:33:22Z)
  — fix or reclassify"
description: >
  WARN-tier guard member check-audit-warning-acknowledgement.sh has been red for more
  than 14 days (first seen red 2026-09-30T00:33:22Z). Operator binding condition (T-3258
  rulings): a stale WARN becomes an action.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [warn-escalation, guard-layer]
components: []
related_tasks: []
# write_set:                      # T-3512: optional — globs (relative to PROJECT_ROOT)
#                                 # naming the files this task intends to write. Declared
#                                 # at CAPTURE, unlike components: which the framework
#                                 # resolves from git history at close. Feeds TWO things:
#                                 #   1. `fw write-set check T-A T-B` — without it the
#                                 #      comparison has nothing to compare and every real
#                                 #      pair exits 2 (undecidable). 0 of 3032 tasks
#                                 #      declared it, so that gate has never had an input.
#                                 #   2. BVP blast_radius before close — the 0.6-weighted
#                                 #      cost term, unavailable for 85% of rankable tasks
#                                 #      because components: only exists once the task is
#                                 #      finished (T-3471).
#                                 # Example: write_set: ["lib/bvp.sh", "tests/unit/t*_bvp*"]
#                                 # An EMPTY list is a real declaration ("writes nothing"),
#                                 # which is not the same as omitting the field. Omitted
#                                 # means unknown, and unknown must never score as cheap.
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
# demo_target: true               # T-2286: optional — marks task as reserved for an orchestrated demo
#                                 # worker (e.g. arc-010 HM-A dispatches via mcp__fw__work_on). When set,
#                                 # `fw work-on T-XXX` refuses unless --i-am-demo-orchestrator (CLI) or
#                                 # FW_I_AM_DEMO_ORCHESTRATOR=1 (env) is passed. Prevents the parent
#                                 # session from consuming the captured→started-work transition the demo
#                                 # worker expects to drive. Origin OBS-057.
created: 2026-10-14T05:42:07Z
last_update: 2026-10-18T22:13:06Z
date_finished:
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
  - ts: '2026-10-17T22:02:53Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 4
      D3: 3
      D4: 2
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=4 (body:fw-audit-or-doctor); D3=3
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=2 (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3440: WARN escalation: check-audit-warning-acknowledgement.sh red 14d (since 2026-09-30T00:33:22Z) — fix or reclassify

## Context

<!-- warn-escalation: member=check-audit-warning-acknowledgement.sh -->
Filed automatically by scripts/warn-escalation-file.sh (T-3267) from the host release canary.

- **Member:** `check-audit-warning-acknowledgement.sh` (tier WARN — never gates CI or a release)
- **First seen red:** 2026-09-30T00:33:22Z (14d ago; escalation threshold 14d)
- **Tier reason (operator classification):** arc-008 success condition; process state, operator-approved WARN (T-3211 Part 2)

**Current output:**

```
$ bash scripts/check-audit-warning-acknowledgement.sh --no-heartbeat   # rc=1
  WARN  BVP driver AS_DELIVERY_TRUTH has neither a handler nor a scoring spec
        mitigation: Give it a mechanism: draft a scoring: spec, check it with 'bin/fw bvp driver --validate-scoring <file>', try it with 'bin/fw bvp driver --explain AS_DELIVERY_TR
  WARN  BVP driver AS_FAILURE_SURFACED has neither a handler nor a scoring spec
        mitigation: Give it a mechanism: draft a scoring: spec, check it with 'bin/fw bvp driver --validate-scoring <file>', try it with 'bin/fw bvp driver --explain AS_FAILURE_SUR
  WARN  BVP driver AS_OPERATOR_SIMPLICITY has neither a handler nor a scoring spec
        mitigation: Give it a mechanism: draft a scoring: spec, check it with 'bin/fw bvp driver --validate-scoring <file>', try it with 'bin/fw bvp driver --explain AS_OPERATOR_SI
  WARN  BVP driver AS_COORDINATION_TRUTH has neither a handler nor a scoring spec
        mitigation: Give it a mechanism: draft a scoring: spec, check it with 'bin/fw bvp driver --validate-scoring <file>', try it with 'bin/fw bvp driver --explain AS_COORDINATIO
  WARN  BVP driver AS_FAILURE_OBSERVABILITY has neither a handler nor a scoring spec
        mitigation: Give it a mechanism: draft a scoring: spec, check it with 'bin/fw bvp driver --validate-scoring <file>', try it with 'bin/fw bvp driver --explain AS_FAILURE_OBS
  WARN  BVP driver AS_RESOURCE_FOOTPRINT has neither a handler nor a scoring spec
        mitigation: Give it a mechanism: draft a scoring: spec, check it with 'bin/fw bvp driver --validate-scoring <file>', try it with 'bin/fw bvp driver --explain AS_RESOURCE_FO
  WARN  Continuous-run wrapper last recorded ARMED but the turn driver is not
        mitigation: fw continuous arm --hours N --iterations N --directive '...' (T-3225). Same drift class as the 74-day and 8-day prior incidents (G-099).
  WARN  Fabric: 543 under-populated card(s)
        mitigation: Run: bin/fw fabric enrich --describe-only (fills purpose/subsystem from each file's own header; names what it refuses). Zero-edge cards want a look, not a re-ru

acknowledged (counted and reported, non-firing):
  ack   All onboarding-seed corpus references resolve to existing maps — NOT EVALUATED: candidate set empty (0 onboarding-seed c
        reason: T-3097: NOT a real gap — audit.sh:1144 resolves the framework-owned lib/seeds/tasks via $PROJECT_ROOT, so in a vendored consumer it scans a path that does not exist and the candidate set is empty by construction. The seed corpus holds 12 `corpus explain` references; the check walks 0. Vendored (G-062), filed upstream at framework:pickup offset 178 severity:high with the three-line root fix. DELETE THIS ENTRY when upstream resolves those dirs via $FRAMEWORK_ROOT and the check reports a non-empty candidate set.
  ack   free driver F-ORCH: retire_when condition appears met (T-1643 completed cleanly OR G-064 closed) — review whether to ret
        reason: T-3240 (R7, 2026-09-29): the signal is a task-ID namespace collision, not evidence. audit.sh:1484-1493 (orch_signal) tests "T-1643 completed cleanly" against THIS project's .tasks/, but the retire_when text means the FRAMEWORK's T-1643 (orchestrator substrate); termlink's T-1643 is an unrelated task ("propose fw vendor manifest hardening", completed). Independently, retiring a driver edits policy/value-drivers.yaml — BVP driver weights are §ACD sovereignty-gated and an agent may not change calibration. Vendored heuristic (G-062). DELETE THIS ENTRY when the operator decides F-ORCH's retirement, or upstream scopes the T-1643 lookup to the framework's own task namespace.
  ack   No PROJECT_ROOT resolution of framework-owned assets (T-2648, OBS-097) — NOT EVALUATED: candidate set empty (0 Python fi
        reason: T-3098: same root cause as the entry above, and the sharpest instance of it — audit.sh:1771 IS the OBS-097 split-root lint whose stated purpose is catching framework dirs resolved via PROJECT_ROOT, and it is disabled by the class it guards. It would walk 135 .py files; it walks 0. Its own comment predicts the failure ("the failing path structurally cannot fire where the code is developed") because in AEF's own repo PROJECT_ROOT == FRAMEWORK_ROOT. Filed at offset 178. DELETE WHEN upstream fixes the root resolution.
  ack   No stale-slice-references (L-417) — NOT EVALUATED: candidate set empty (0 file(s) under web/templates web/blueprints lib
        reason: T-3099: third symptom of the same audit.sh wrong-root defect (line 1818). Would walk 264 files under web/templates, web/blueprints and lib; walks 0. Filed at offset 178 as one cause, not three bugs. DELETE WHEN upstream fixes the root resolution.
  ack   Fabric: 401/639 cards have no edges
        reason: T-3009 / PL-161 (2026-05-17): the audit's prescribed mitigation is structurally a no-op. MEASURED 2026-09-26: `fw fabric scan` over 556 cards enriched 4 and added 6 edges — the remaining ~290 edgeless cards have no detectable code-deps to find, so re-running enrich cannot move them. PL-161 already named the correct resolution: tag genuinely-edgeless cards `standalone: true` (the audit supports the opt-out at audit.sh:658) instead of chasing the no-op. That is a ~290-card triage and its own task, not a side-effect of clearing a warning. DELETE THIS ENTRY when the edgeless cards are triaged for standalone:true, or when the audit measures coverage over non-standalone cards only.
  ack   Fabric: 9 card(s) point at files no watch pattern covers
        reason: T-3103: all 10 are carded singletons in categories the watch file deliberately does not cover, and `.fabric/watch-patterns.yaml` documents the decision in its own comments — "never hand-shape a glob around one file to dodge the check", citing the PL-341 trap. Measured: 8 are `.bpmn` designer diagrams inside the VENDORED .agentic-framework tree (50 cards over 8 files) and are not this project's source at all; 1 is .claude/commands/capture.md, 1 of 34 slash-commands carded; 1 is .context/cron/canary-aliveness-sweep.crontab, 1 of 30 crontabs carded. Each honest resolution is a category-level scope decision — register all 34 commands / all 30 crontabs, or drop the singleton cards, and decide whether vendored designer artifacts belong in our registry. DELETE THIS ENTRY when those scope decisions are taken. Widening the globs to make this warning disappear is the fabrication the watch file warns against.

stale ledger entries (non-firing) — pattern matches no current warning, so it is either fixed or the pattern drifted:
  stale line 35: GO-scope-not-propagated inception(s)
        reason on file: T-3240 (R7, 2026-09-29): same class as scripts/check-go-propagation.sh, which the operator explicitly RESERVED on 2026-09-29 (SQ-9 walk-through: "NOT yours ... 9 human-decided GO inceptions — leave both red"). That check stays red and carries this finding visibly, so acknowledging the audit's duplicate hides nothing; triaging human-decided GO inceptions is the operator's. DELETE THIS ENTRY when check-go-propagation.sh exits 0.

SCOPE CAVEAT — the counts above cover ONLY the section(s) this audit file recorded (structure). `fw audit` runs per-section and the pre-push hook runs the structure section alone; a full run emits several times as many warnings (measured 2026-09-26: 8 for structure, 39+ for a full run). Do NOT read a low unexamined count here as "the audit is nearly clean" — re-run `fw audit` without a section filter to see the whole surface.

scope: classifies WARN findings in one audit file as acknowledged-with-a-reason or unexamined. It does NOT verify that a cited reason is TRUE, does not re-run the audit, and cannot acknowledge a FAIL. A green means every warning has been looked at and explained - never that the project has no warnings.

verdict: FIRING — 15 unexamined warning(s)
```

Do not hand-edit the ledger date (.context/checks/guard-warn-first-red) to silence this. Closing this task
while the member is still red restarts a 14-day window from the close date, then files again.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] `check-audit-warning-acknowledgement.sh` is green, or the operator has reclassified it (tier change recorded in docs/reports/T-3258-guard-classification-draft.md)

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
       `bin/fw reviewer T-XXX > /tmp/.rev 2>&1 && grep -q "Overall:.*PASS" /tmp/.rev`
       added to ## Verification. NEVER `... 2>&1 | grep -q ...` — that is the shape the
       Pipefail/SIGPIPE section below forbids (T-3370: the 1.8.3 template prescribed it again).
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
# Build/refactor/decommission tasks with NO command here are refused. If nothing can be
# run, add the line `# verification: none — <why nothing can be run>`; it is serviced
# (queued for review: add a check / other evidence / accept). See docs/reports/T-4024-servicing-loop.md.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# ── Mutable-corpus anchor (T-3326) ────────────────────────────────────────────
# Do NOT anchor a verification line (or a unit test it runs) to MUTABLE corpus
# state — an exact live count, or a grep of live `fw audit`/`fw doctor` output
# for a specific corpus entity (a named arc, a task count, a census number).
# The corpus moves under the check, and the line rots: it goes red (or vanishes
# its pattern) for reasons unrelated to the code under test, blocking closes.
# Pin the INVARIANT (categories sum, count > 0, property holds) or run the code
# against a COMMITTED FIXTURE — never the live count or a live-audit line.
# Origin: T-2969 line grepping live audit for one arc's status; T-2871's census
# test pinning exact live counts (56→74 files) — both blocked closes (OBS-377).
#
# ── Pipefail/SIGPIPE: grepping a command's output (L-387, T-2090, T-2743, T-2738) ──
#
# THE DEFAULT — redirect to a file, then grep the file:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
#     curl -sf "$(bin/fw watchtower url)/page" -o /tmp/.out && grep -q "PAT" /tmp/.out
# Correct at any output size, and `&&` keeps the PRODUCING command's exit code in
# the verdict. Reach for this first; the alternative below is the special case.
#
# NEVER `cmd | grep -q PAT` (L-387) — why: P-011 runs each line with PIPEFAIL LIVE
# (errexit is not — see below). When grep matches it exits and closes stdin while cmd is still
# writing, cmd takes SIGPIPE, the pipeline exits 141 — verification "fails" with
# the pattern present. Captured 4× (T-1716, T-1838, T-1862, T-1863).
#
# THE EXCEPTION — capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Valid ONLY while "$out" fits the 65536-byte pipe buffer, and it is on you to
# know that it does. Above that the form inverts and becomes the very failure
# L-387 describes: echo blocks on the full pipe, grep -q exits, echo takes
# SIGPIPE, rc=141 (T-2743 — measured on a 146,366-byte Watchtower page, 3/3 runs,
# deterministic not racy; rendered routes run 50-200KB, so anything that curls a
# page is over the line). It also discards cmd's exit code, so a 404 yields an
# empty capture that grep merely fails to match rather than a failed line.
# If you do use it: single pipe only, no intermediate tail/awk/sed stage between
# capture and grep (T-2090) — the middle stage is what `grep -q` slams its stdin
# on, and grep scans the whole captured string anyway, so the `tail -3` was
# cosmetic. `echo "$out" | grep -q PAT`, nothing between.
#
# ── Asserting an ABSENCE: prove the search could have succeeded (T-3144) ──
# (Restored after the AEF 1.8.3 re-vendor dropped it, T-3370.)
#
# `! grep -q "PATTERN" file` exits 0 when the pattern is absent. It ALSO exits 0
# when the file was renamed, deleted, or is empty — so the leg cannot distinguish
# "the bad thing is not there" from "I could not look", and the gate reports green
# over a check that never ran. Pair every absence assertion with something that
# fails if the search could not happen:
#
#     test -f path/to/file && ! grep -q "PATTERN" path/to/file    # existence first
#     grep -q "KNOWN_MARKER" f && ! grep -q "PATTERN" f           # positive companion
#     cmd > /tmp/.out 2>&1 && ! grep -q "PATTERN" /tmp/.out       # &&-joined producer
#
# Count-equals-zero is the same defect wearing a different hat, and it is the one
# that bites hardest over a COMMAND's output rather than a file:
#
#     [ "$(cargo clippy --workspace 2>&1 | grep -c "^error")" = "0" ]   # WRONG
#
# If cargo is missing, or dies before emitting diagnostics, there are no `^error`
# lines, the count is 0, and the leg passes — a build gate that goes green
# precisely when the build could not run. Keep the producer's exit code in the verdict:
#
#     cargo clippy --workspace > /tmp/.out 2>&1 && ! grep -q "^error" /tmp/.out
#
# TEST RUNNERS need a guard either way (T-2738). `set -e` is suppressed inside the
# `if` condition the gate runs each line in, so in `cmd1; cmd2` only cmd2 is the
# verdict — and the pass marker you grep for survives a partial failure: a suite
# printing "3 failed, 9 passed" satisfies `grep -q "9 passed"`, and generalising
# to `grep -qE "[0-9]+ passed"` matches the same output. Keep the exit code:
#     python3 -m pytest <file> -q > /tmp/.out 2>&1 && grep -q passed /tmp/.out
# or add the guard the exit code used to supply:
#     out=$(python3 -m pytest <file> -q 2>&1); echo "$out" | grep -q passed && ! echo "$out" | grep -q failed
#     out=$(bats <file> 2>&1); echo "$out" | grep -q '^ok 1 ' && ! echo "$out" | grep -q '^not ok'
# The close gate refuses the unguarded form. Bypass: FW_ALLOW_UNJUDGED_TEST_RUN=1.
#
# ── A SKIPPED BATS TEST REPORTS `ok` (T-3217) ─────────────────────────────────
#
# `! grep -q "^not ok"` does NOT mean the suite ran. Bats emits a skip as
#     ok 6 <name> # skip <reason>
# which is not a `not ok`, so the gate passes and the report says ok while the
# thing the test covers was measured NOWHERE. Origin: T-3213 guarded a test with
# `[ "$(id -u)" -eq 0 ] && skip` — the suite runs as root here and in CI, so it
# skipped on every run that mattered, for as long as it existed.
#
# Add a skip clause to any bats verification line. `# skip` is the marker bats
# writes; counting it is the whole check:
#     timeout 300 bats <file> > /tmp/.out 2>&1 && ! grep -q "^not ok" /tmp/.out
#     test "$(grep -c '# skip' /tmp/.out)" -eq 0
# Two lines, because they answer different questions — "did anything fail" and
# "did everything run". If some skips are legitimate on your host (an optional
# dependency is genuinely absent), assert the COUNT you expect rather than zero,
# and say in the task why that number is right.
#
# Corpus-wide, the same check runs from `bin/fw test lint`
# (tools/bats-silent-skip-lint.py): static mode flags guards that are fixed for
# a deployment rather than probing an optional dependency, and `--tap FILE`
# reports the skips a real run actually fired.
#
# REHEARSING A LINE BY HAND DOES NOT REHEARSE THE GATE (T-2743). Your interactive
# shell has no pipefail. A line has returned 0 by hand and 141 under P-011, from
# the same directory, the same second. To rehearse for real:
#     bash -c 'set -o pipefail; <your verification line>'
#
# NOTE THE MISSING `-e` — it is not a typo (T-3203). This file used to prescribe
# `set -eo pipefail` here, which is NOT the gate: it adds errexit the gate does
# not have, so it FAILS lines the gate PASSES. Measured, 10 lines, 3 diverged:
#     line                            gate    set -eo (old)   set -o (this)
#     false; true                     PASS    FAIL  wrong     PASS  ok
#     cd /nonexistent; echo ok        PASS    FAIL  wrong     PASS  ok
#     grep -q MISS file; true         PASS    FAIL  wrong     PASS  ok
# The divergence is one-directional and that is the trap: the old rehearsal only
# ever fails lines the gate accepts, so it produces false REDS, and an author
# who "fixes" a line to satisfy it is fixing something that was never broken —
# while the line that actually is broken (`cmd1; cmd2` where cmd1 fails) passes
# both. Re-derive rather than trust this table — it is pinned, not asserted:
#     bats tests/unit/t3203_p011_gate_semantics.bats
#
# ── `cmd1; cmd2` IS JUDGED ONLY ON cmd2 (T-3203) ──────────────────────────────
#
# The gate runs each line as the CONDITION of an `if` (update-task.sh:1215), and
# POSIX suppresses errexit for a compound command in an `if` condition — through
# the subshell. So pipefail applies and `set -e` does not, and in a sequence only
# the LAST command's status reaches the verdict. `cd /nonexistent; echo ok` passes.
# 2,644 of 10,997 verification lines in this corpus contain `;` (re-derive with
# the query in docs/reports/T-3203-p011-gate-semantics.md).
#
# SAFE SHAPES — both verified biting, each against a passing control:
#   A. one command whose own status is the verdict (prefer this):
#        out=$(cmd 2>&1); echo "$out" | grep -q PAT && ! echo "$out" | grep -q BAD
#      the leading assignments are setup; the trailing `&&` chain is the verdict.
#   B. an explicit sub-shell, whose errexit the outer `if` cannot reach into:
#        bash -c 'set -eo pipefail; cmd1; cmd2'
#      use when you genuinely need every command in the sequence to count.
#
# The rule of thumb: put the assertion LAST, and make sure it is an assertion.
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

<!-- T-2945: same shape as inception.md's block — the gate that reads it
     (audit_inception_recommendation, lib/task-audit.sh:117) is shared, so the
     shape is copied rather than reinvented.

     REQUIRED once this task reaches partial-complete: Agent ACs done, at least
     one `### Human` AC still unticked. `lib/review.sh:205-211` (T-2421) BLOCKS
     `fw task review` emission for build/refactor/test/decommission tasks in that
     state with no substantive block here — the operator would otherwise open
     /review/<id> to a blank Recommendation card and be asked to approve a form.

     Not required while every Human AC is ticked or the task has none: the gate
     only fires on the partial-complete transition. It is here from the start so
     you write it while you still have the evidence, not when the gate refuses.

     Format (the parser wants the `**Recommendation:**` line at the start of a
     line; a leading `-` or `*` bullet is also accepted):
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence — what shipped, what was proven, what remains)
     **Evidence:**
     - Finding 1
     - Finding 2

     DEFER is for evidence gaps, not confidence gaps (CLAUDE.md §Presenting Work
     for Human Review). If the artefact is complete and you still don't want to
     commit, that is a calibration failure — recommend GO or NO-GO.
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

### 2026-10-18 — 5a: the 405 zero-edge fabric cards (operator ruling)
- **Chose:** B. Add edges from literal path references between carded files (comment-only mentions excluded, ~286 cards), delete orphan cards whose file no longer exists (~29), mark the genuinely isolated rest `standalone: true` and acknowledge only that remainder in the audit-warning ledger, and file upstream that `underpopulated.py` ignores `standalone`.
- **Why:** most of the 405 are fixable here (the enricher only looks at source/exec positions); the ledger's rule is fixable gets fixed, not listed. Measured 2026-10-18: 286 linkable, ~29 orphans, ~90 isolated; `underpopulated.py:51` counts edges only.
- **Rejected:** A (acknowledge all: hides fixable work), C (no acknowledgement: escalation keeps firing on a fixed-and-filed remainder), D (nothing).
- **Precondition stated in the brief:** verify first that a later `fw fabric enrich` keeps hand-added edges; if it overwrites them, come back before writing. Verified by reading: enrich.py:1353-1356/1374-1377 append to existing edges, and 1281-1285 derive depended_by from existing depends_on.
- **Correction after the ruling (agent error):** there are NO orphan cards. All 411 zero-edge cards point at existing files; the "~29 orphans" came from keying the measurement by `location`, which collapsed the 50 workflow-designer cards that share 8 .bpmn files. Step "delete orphans" is void; nothing is deleted.

### 2026-10-18 — 5b: the two stale arcs (operator ruling)
- **Chose:** A. Close arc-001 (arc-parallel-substrate, 43/46 done, headline demonstrated by T-2211 substrate-drain-demo) and arc-002 (arc-substrate-fitness, 11/12 done, headline measured live 2026-10-18: cv-keys agent-presence = 2 entries, dead-letter table + canary working) as shipped. The open members (T-2022, T-2024, T-2026, T-2250: human DEFER inceptions with ripe revisit_at) stay as standalone deferred tasks.
- **Why:** the headline mechanic of each arc is demonstrated; what kept them open is four deferred human questions that already carry their own revisit reminder, not agent work. Closing is the fix the ledger rule asks for.
- **Rejected:** B (acknowledge open arcs: lists a fixable finding), C (decide the four inceptions first: four briefs on June questions nothing currently depends on), D (nothing).
- **REVISED by the operator the same day (supersedes the ruling above):** "No closed recommendation, no evidence. I cannot close it. So it is not closed ... Same thing as a task. If it's not completed, we cannot close it. We still need stuff to do." The arcs stay OPEN: an arc with open member work is not complete, exactly like a task. The Watchtower close page showed no recommendation and no evidence (the agent had put the evidence in a report and the hand-off text, not on the arc). Agent error: the brief treated "headline demonstrated" as "complete" and framed four open members as not-work. Consequence: the stale-arc WARN is a true finding (the remaining work is not moving) and stays unacknowledged; the remedy is to do the remaining members. docs/reports/T-3440-arc-002-close-evidence.md is kept as a progress record, not a close.

### 2026-10-18 — 5c: six BVP drivers with no scoring mechanism (operator ruling, after external review)
- **Chose:** A′. Fix first: local patch so `fw arc approve-driver --scoring-file` persists the validated `scoring:` block inline and can attach one to an existing driver without losing approval history (T-3472, registered in .vendor-divergence.yaml, filed upstream together with the reviewer/estimator mismatch); a guard that fires when a reviewer recorded "scorable" but no `scoring:` is stored; then six specs, arc-substrate-fitness first, each tested across its arc's member tasks, keeping only specs that discriminate; the operator attaches; rescore with before/after rankings.
- **Why:** both reviewers (codex, claude -p; opencode failed twice) confirmed `arc.sh` drops the spec (entry built at ~1665 without scoring or rationale; duplicate check at 1548-1569 refuses re-attach) and that the estimator reads only an inline block. Attaching through the broken verb would repeat 2026-10-06.
- **Rejected:** B (remove all six: discards priorities over a bug), D′ (arc-substrate-fitness only: leaves the active arc's axes fake), E (hand migration without the fix: the bug stays and the next approval loses its spec again). C was out per both reviewers (hides the bug's evidence).
- **Comes back to the operator:** per arc, the specs with their score spread; the attaching itself; for arc-011, whether re-attaching falls under the T-3184 ruling. Reviews: scratchpad consult-5c-codex.md, consult-5c-claude.md.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-10-14T05:42:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3440-warn-escalation-check-audit-warning-ackn.md
- **Context:** Initial task creation

### 2026-10-17T22:02:52Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
