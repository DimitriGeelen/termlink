---
id: T-3017
name: "D2 audit check prints 10 sub-threshold task IDs beneath a correctly-filtered
  count"
description: >
  arc-008 cycle-2 finding, unchanged since cycle 1. The D2 header count is correctly
  filtered (58 task(s) waiting >30d) but the printed list carries 68 IDs, ten of them
  under the 30-day threshold: T-2409(27d) T-2706(23d) T-2709(24d) T-2711(24d) T-2822(27d)
  T-2836(27d) T-2839(24d) T-2861(20d) T-2873(18d) T-2878(18d). A reader trusting the
  list over the header over-counts the backlog by 10. Recorded in T-2940's Updates
  at cycle 1 but never filed as its own finding. Check is vendored (G-062) so the
  fix is upstream. Census: .context/audits/arc-008-cycle2-census.md

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [.agentic-framework/agents/audit/audit.sh]
related_tasks: [T-2940, T-2194, T-3014]
arc_id: arc-008
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
created: 2026-09-20T10:31:21Z
last_update: 2026-09-20T17:30:57Z
date_finished: 2026-09-20T17:30:57Z
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
  - ts: '2026-09-20T10:32:41Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-20T10:32:41Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=204,acs=4)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-20T13:16:26Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (single-component); tier=2 (workflow:build); 
      effort=8 (lines=204,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3017: D2 audit check prints 10 sub-threshold task IDs beneath a correctly-filtered count

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

The D2 audit line reports a correctly age-filtered COUNT beside an unfiltered LIST.
Measured live: header 58, printed 68, surplus 10 — and the surplus is exactly the
14-30d warn-band population, because both age branches append to one shared
`d2_details` accumulator (audit.sh ~5029-5037) which the FAIL label then prints.
The file is vendored, so the fix is upstream (G-062); this task measures, locates the
mechanism, and files. Filed at `framework:pickup` offset 129.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The defect is MEASURED against the live audit check, not inferred from this
      filing: the D2 header count and the D2 printed list are both captured from a
      real run, and the number of printed IDs under the 30-day threshold is stated
      as an observed figure (which may be 0, disproving the filing).
- [x] The MECHANISM is identified at file:line in the vendored check — why the
      header count is age-filtered while the printed list is not, or evidence that
      both share one filter and the filing misread it.
- [x] Disposition is decided ON EVIDENCE and recorded: filed upstream, registered as
      a local divergence, or recorded as already-known/declined, with the reason.
      Provenance is established before filing — upstream's defect, or this project's
      own convention?
- [x] No local patch is made to vendored code under .agentic-framework/ (G-062); the
      vendored file is verified byte-clean at close.

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

# ── AC2: the mechanism. ONE accumulator, appended by BOTH age branches. ──
test "$(grep -c 'd2_details="$d2_details $t_id' .agentic-framework/agents/audit/audit.sh)" = "2"
# ── AC2: and printed under BOTH labels, so the >30d line carries warn-band IDs. ──
grep -q 'waiting >30d:$d2_details' .agentic-framework/agents/audit/audit.sh
grep -q 'waiting >14d:$d2_details' .agentic-framework/agents/audit/audit.sh
# ── AC2: the two band thresholds the branches select on (720h = 30d, 336h = 14d). ──
grep -q 'age_hours" -ge 720' .agentic-framework/agents/audit/audit.sh
grep -q 'age_hours" -ge 336' .agentic-framework/agents/audit/audit.sh
# ── AC1: measured from live audit output. The invariant holds BEFORE and AFTER an
# upstream fix (today 10==10; once fixed 0==0), so it asserts the mechanism, not the
# bug's presence. Mutant-tested: hardcoding thr=14 instead of reading it from the
# message reddens this line. ──
python3 -c 'import re; d=open(".context/audits/discoveries/LATEST.yaml").read(); m=re.search(r"D2: Human review queue . (\d+) task\(s\) waiting >(\d+)d:(.*?)\"", d, re.S); assert m; hdr=int(m.group(1)); thr=int(m.group(2)); ids=re.findall(r"T-\d+\((\d+)d\)", m.group(3)); under=[a for a in ids if int(a)<thr]; assert len(ids)-hdr==len(under)'
# ── AC4: no local patch to vendored code (G-062). ──
test -z "$(git status --porcelain .agentic-framework/agents/audit/audit.sh)"
test -z "$(git status --porcelain .agentic-framework/)"

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

**Symptom:** The D2 audit line reports `58 task(s) waiting >30d:` and then prints 68
task IDs, ten of them aged 18–27 days. A reader trusting the list over the header
over-counts the aged backlog by 17% and mis-triages ten tasks as month-forgotten.

**Root cause:** `audit.sh` accumulates ONE details string, `d2_details`, and appends to
it from BOTH mutually-exclusive age branches — the `>= 720h` (30d) arm and the
`>= 336h` (14d) arm. The counters `d2_fail` and `d2_warn` are correctly separate; only
the details string is shared. The `fail` line then prints `$d2_fail` (30d-only) beside
`$d2_details` (the union of both bands). The surplus is therefore not approximate: it
is exactly the warn-band population, which is what the measurement shows (68 − 58 = 10,
and the count of printed IDs under 30 days is also 10).

**Why structurally allowed:** the defect is **asymmetric**, and the asymmetry is what
hid it. When `d2_fail == 0` the `elif` warn branch renders, and `$d2_details` then
contains only warn-band tasks — so the check is *correct* in the warn case and wrong
only once it escalates to FAIL. Every cheap reading of the code in the healthy state
shows a consistent count and list. It misreports precisely when it is escalating, i.e.
when an operator is most likely to act on it. Nothing in the framework compares a
control's rendered *message* against the predicate that selected its contents, so a
count and a list can disagree indefinitely without any gate noticing.

**Prevention:** distinct from the fix. The invariant is corpus-independent and needs no
fixture: *every `T-NNNN(Xd)` printed beside a threshold must satisfy X ≥ that
threshold.* It is asserted in this task's `## Verification` in the stronger form
`surplus == sub-threshold population`, which holds both before an upstream fix (10==10)
and after it (0==0) — so it pins the mechanism rather than the bug, and does not redden
when the bug is fixed. It generalises to any audit line that pairs a filtered count with
a detail list, which is the reusable half.

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

### 2026-09-20 — the surplus is not approximately the warn band; it IS the warn band

- **What changed:** at filing this was "the list carries ten sub-threshold IDs" — a
  symptom with no mechanism, carried unfiled since T-2940 cycle 1. Executing AC1
  produced a sharper fact than the filing had: `printed − header == count(printed under
  threshold)`, exactly, 10 and 10. That equality is not what a sloppy filter looks like;
  it is the signature of a *shared accumulator*, and it located the mechanism from the
  output side before the source was opened. Reading `audit.sh` then confirmed it.
- **Plan impact:** the deliverable stayed diagnosis-and-filing (the file is vendored,
  G-062), but the evidence strengthened from "ten IDs look wrong" to a one-line
  structural claim with a corpus-independent invariant attached. The filing proposes the
  split-accumulator fix rather than a filter, because a filter on the print site would
  fix the FAIL line and leave the same shared-state shape in place.
- **What the two findings in this file have in common:** T-3016 (CTL-029, filed at
  offset 127 two commits ago) and this one are both defects in what a control *says*
  about what it found — CTL-029 prescribes a command R-033 forbids; D2 prints a list its
  own header contradicts. Neither is an error in the predicate. Nothing checks a
  control's rendered message against the selector that produced it. That blind spot is
  worth more than either instance and is named as such in the filing.
- **Triggered:** filed upstream at `framework:pickup` offset 129. No local patch
  (G-062). Also surfaced, unrelated to this task's ACs and recorded rather than acted
  on: two further P-002 allowlist shapes (below).

### 2026-09-20 — two more P-002 shapes, and one of them collapses two open items into one

- **What changed:** T-3030 closed recording five refusal shapes, and its handback added
  a sixth (`while read -r t`). This session hit a seventh, `for t in …`, and an eighth,
  a `>` **inside a quoted grep pattern** (`grep -n "waiting >30d\|…"`), which the write
  detector scored as a redirect on a line containing no redirect at all.
- **Plan impact:** shapes six and seven are **not two gaps**. `while` and `for` are
  shell *keywords*, not command names, so the allowlist's per-command leading-token
  check can never match either — one gap, two faces. That merge matters because T-3030's
  handback listed them as separate open items.
- **Why the eighth is sharper than anything T-3030 recorded:** T-3030's `>` finding was
  about a real redirect (`>/dev/null`) being classified as a write, which upstream has
  already considered and declined in a comment in the vendored file. This is different:
  there is no redirect on the line. The gate cannot see quoting, so a `>` inside a
  quoted argument is indistinguishable from an operator. That is a stronger case than
  the one upstream declined, and it is not the same claim.
- **Triggered:** nothing filed from this task — it is outside T-3017's acceptance
  criteria, and an activity that closes no criterion is not part of the task. Recorded
  here and carried to the handback so it is filed under its own task with its own
  measurement, not smuggled into this one.

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

### 2026-09-20T10:31:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3017-d2-audit-check-prints-10-sub-threshold-t.md
- **Context:** Initial task creation

### 2026-09-20T17:25:43Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-a7308884
- **Timestamp:** 2026-09-20T17:31:00Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-20T17:30:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
