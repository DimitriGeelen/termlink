---
id: T-3260
name: "Guard layer three-tier severity FAIL/WARN/INFO with 14-day WARN escalation
  and approved markings"
description: >
  Operator-approved successor to T-3258: replace BLOCKING/ADVISORY with FAIL/WARN/INFO
  on the guard-layer marker; WARN never blocks either gate but is annotated (::warning::)
  and escalates via the release canary after 14 days red; INFO printed only; apply
  the approved markings.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [guard-layer, ci, release]
components: []
related_tasks: [T-3258, T-3211, T-3243]
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
created: 2026-09-29T21:53:12Z
last_update: 2026-09-29T22:08:12Z
date_finished: 2026-09-29T22:08:12Z
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
  - ts: '2026-09-29T21:54:24Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=2 (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-29T21:54:24Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 7
      tier: 2
      effort: 8
    rationale: blast_radius=7 (9-file-refs-derived-T-3189); tier=2 
      (workflow:build); effort=8 (lines=239,acs=12)
    rubric_sha: e4a00f38e801
---

# T-3260: Guard layer three-tier severity FAIL/WARN/INFO with 14-day WARN escalation and approved markings

## Context

Operator-approved successor to T-3258 (rulings recorded in `docs/reports/T-3258-guard-classification-draft.md` §Operator rulings, and in `.context/runs/T-3211-operator-note-R9.txt`). T-3258 shipped two classes (BLOCKING/ADVISORY) in `scripts/run-guard-layer.sh`, `scripts/check-guard-severity-markers.sh`, `tests/guard-layer-severity-fixtures.sh`; `.github/workflows/release.yml` runs `--gate release`, `.github/workflows/doc-lint.yml` runs the default gate. Escalation extends `scripts/check-release-publication-freshness.sh` (T-3243). Release run 36633929858 (v0.12.1) failed in the guard layer on two members that the operator approved as WARN.

## Acceptance Criteria

### Agent
- [x] `run-guard-layer.sh` parses `# guard-layer: source [fail|warn|info] [args]  # <reason>`; no word = FAIL; the T-3258 word `advisory` is accepted and mapped to WARN (documented as a compatibility alias)
- [x] A WARN or INFO member with FAIL or ERROR does NOT set the exit code under EITHER gate (`all` and `release`); a FAIL member red sets it under both
- [x] Each red WARN member (FAIL or ERROR) prints a GitHub `::warning::` annotation line and is listed in the runner summary; ERROR on a WARN member is reported as WARN, never as PASS; INFO members never print a red verdict and never emit an annotation
- [x] A WARN/INFO marker with no `# <reason>` is refused: the runner treats it as FAIL and reports MALFORMED, and `check-guard-severity-markers.sh` exits 1 naming it
- [x] `cargo test` and `tests/*fixtures*.sh` are always FAIL regardless of any marker word (enforced in the runner and by the marker checker)
- [x] Durable first-seen-red record: the runner records, per WARN member, the date it was first seen red and clears it when the member passes; the record's location and its CI-reset limit are documented in CLAUDE.md
- [x] Escalation: `check-release-publication-freshness.sh` (or a sibling wired into the same canary) fires (exit 1) on a WARN member red for more than 14 days and not before
- [x] The approved markings are applied exactly: INFO invocation-usage; WARN voi-prompt, check-go-propagation, check-human-ac-escalation, check-human-ac-steps-heading, check-handover-staleness, check-pickup-deferred-freshness, check-stranded-finalized-tasks, check-task-id-collisions, check-arc-slice-drift, check-installed-binary-drift, check-receiver-ack-lag, check-audit-warning-acknowledgement, check-budget-ladder-drift, check-episodic-parse, check-vendor-divergence, fabric-workflow-link; every other member stays FAIL (no word)
- [x] Fixtures with mutants cover: WARN red passes both gates but IS annotated; FAIL red fails both; missing reason refused; INFO never red; escalation fires past 14 days and not before
- [x] CLAUDE.md (above `## Core Principle`) documents the three tiers; the draft's rulings section is not edited

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
       Pipefail/SIGPIPE section below forbids, and this line used to prescribe it.
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
# ── Pipefail/SIGPIPE: grepping a command's output (L-387, T-2090, T-2743, T-2738) ──
#
# THE DEFAULT — redirect to a file, then grep the file:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
#     curl -sf "$(bin/fw watchtower url)/page" -o /tmp/.out && grep -q "PAT" /tmp/.out
# Correct at any output size, and `&&` keeps the PRODUCING command's exit code in
# the verdict. Reach for this first; the alternative below is the special case.
#
# NEVER `cmd | grep -q PAT` (L-387) — why: P-011 runs each line under `set -eo
# pipefail`. When grep matches it exits and closes stdin while cmd is still
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
# precisely when the build could not run. Measured in this corpus, not invented.
# Keep the producer's exit code in the verdict:
#
#     cargo clippy --workspace > /tmp/.out 2>&1 && ! grep -q "^error" /tmp/.out
#
# T-3144 censused 2853 task files: 71 absence assertions, 41 already correct, 30
# not. The convention mostly works — this note is here so the next one is written
# right, because a vacuous leg is invisible until the day the path moves.
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
# REHEARSING A LINE BY HAND DOES NOT REHEARSE THE GATE (T-2743). Your interactive
# shell has no `set -eo pipefail`. A line has returned 0 by hand and 141 under
# P-011, from the same directory, the same second. To rehearse for real:
#     bash -c 'set -eo pipefail; <your verification line>'
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

bash -n scripts/run-guard-layer.sh
bash tests/guard-layer-severity-fixtures.sh > /tmp/.t3260-sev 2>&1 && grep -q "32 passed, 0 failed" /tmp/.t3260-sev
bash tests/guard-layer-runner-fixtures.sh > /tmp/.t3260-run 2>&1 && grep -q ", 0 failed" /tmp/.t3260-run
bash tests/release-publication-canary-fixtures.sh > /tmp/.t3260-rel 2>&1 && grep -q "M2 with escalation muted" /tmp/.t3260-rel
bash scripts/check-guard-severity-markers.sh > /tmp/.t3260-mk 2>&1 && grep -q "16 warn, 1 info" /tmp/.t3260-mk
bash scripts/run-guard-layer.sh --list --json > /tmp/.t3260-list 2>&1 && test "$(jq -r '[.members[]|select(.class!="fail")|.class+":"+.name]|sort|join(",")' /tmp/.t3260-list)" = "info:invocation-usage.sh,warn:check-arc-slice-drift.sh,warn:check-audit-warning-acknowledgement.sh,warn:check-budget-ladder-drift.sh,warn:check-episodic-parse.sh,warn:check-go-propagation.sh,warn:check-handover-staleness.sh,warn:check-human-ac-escalation.sh,warn:check-human-ac-steps-heading.sh,warn:check-installed-binary-drift.sh,warn:check-pickup-deferred-freshness.sh,warn:check-receiver-ack-lag.sh,warn:check-stranded-finalized-tasks.sh,warn:check-task-id-collisions.sh,warn:check-vendor-divergence.sh,warn:fabric-workflow-link.sh,warn:voi-prompt.sh"
grep -q "Severity tiers (T-3260" CLAUDE.md
test -f .context/checks/guard-warn-first-red

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

**Symptom:** v0.12.1 Release run 36633929858 failed its guard layer on two members that describe environment/process state, not the released code (check-task-id-collisions, check-receiver-ack-lag). Before that, Doc Lint had been red for weeks on members of the same kind (voi-prompt, check-go-propagation), so push CI's red meant little.
**Root cause:** the layer had one severity for push CI (every red gates) and, from T-3258, only a two-way split for releases. A guard that reads host, ledger or bus state could therefore hold a release or keep push CI permanently red, with no path for "real but non-blocking".
**Why structurally allowed:** membership was designed as "safe to run anywhere" (hermetic), but the classification of what a red MEANS was never part of the marker contract until T-3258. The operator classified all 59 static members on 2026-09-29 (draft § Operator rulings).
**Prevention:** the tier is now declared per member, next to its reason, and `check-guard-severity-markers.sh` refuses an unexplained demotion. WARN reds are annotated on every CI run and escalate through the release-publication canary after 14 days, so a warning cannot quietly become permanent. The two root-cause bugs are fixed separately (T-3261, T-3262), because a demotion alone would have hidden them.

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

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-09-29T21:53:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3260-guard-layer-three-tier-severity-failwarn.md
- **Context:** Initial task creation

### 2026-09-29T21:58:14Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-951c3700
- **Timestamp:** 2026-09-29T22:08:58Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Reviewer:** inline
- **Findings:** none

### 2026-09-29T22:08:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
