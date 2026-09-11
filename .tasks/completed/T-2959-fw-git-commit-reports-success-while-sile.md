---
id: T-2959
name: "fw git commit reports success while silently omitting unstaged task-file edits"
description: >
  fw task update --status work-completed does a git mv, which stages the rename. A
  subsequent fw git commit then commits that staged rename and reports success (1
  file changed, rename 100%) while silently omitting any content edits made to the
  task file before the update ran. Plain git commit with nothing staged refuses; here
  the staged rename masks the omission, so the commit is non-empty and looks correct.
  Measured live on T-2951: commit 3fd1159ef recorded the rename with 0 insertions
  while HEAD kept the superseded framing and the corrected body sat unstaged on disk.
  Caught only by reading the stat line and diffing the blob. Directive 2 class: the
  operation reported success and the substance was missing.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-008]
components: []
related_tasks: []
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
created: 2026-09-11T20:36:14Z
last_update: 2026-09-11T20:41:38Z
date_finished: 2026-09-11T20:41:38Z
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
  - ts: '2026-09-11T20:37:39Z'
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
---

# T-2959: fw git commit reports success while silently omitting unstaged task-file edits

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Reproduced from the live incident rather than a synthetic case: commit `3fd1159ef` is shown to contain the task-file rename with **zero** content lines, while the corrected body it was supposed to carry was still unstaged on disk — i.e. HEAD carried the superseded framing and the working tree carried the correction, with no error at any step.
- [x] The mechanism is named precisely, and distinguished from ordinary git semantics: `git commit` with nothing staged REFUSES, which is the safety property being relied on. Here `fw task update --status work-completed` has already staged a `git mv`, so the commit is non-empty and succeeds; the staged rename **masks** the omitted edits. The trap is the composition of the two verbs, not either alone.
- [x] Blast radius measured, not assumed, and the measurement is allowed to contradict the hypothesis: the last 400 task commits are scanned for the same shape (sole change = a `.tasks/` rename, 0 insertions / 0 deletions). **Result: 11 prior instances, and sampling shows they are the BENIGN ordering** — the task's content was committed first and the rename committed after (T-2928 `76e573db5`→`83e27d06d`, T-2876 `175c8bbfb`→`db5869f01`, T-2833 `18c1b22b2`→`ff4222530`). No evidence of prior silent loss. T-2951 is the anomaly because the edits were made and the task closed with no intervening commit.
- [x] Filed upstream per G-062 with the measured evidence — `agents/git/git.sh` is vendored, so a local patch is erased by the next re-vendor (`.vendor-divergence.yaml` records ~one vendor event every two months).
- [x] Local prevention is either shipped or explicitly declined with a reason, and the reason survives contact with the measurement above. The cheap guard — "fire on a commit whose sole change is a `.tasks/` rename with 0 insertions" — is **declined**: the blast-radius scan proves it would fire on all 11 benign commits, making it permanently red, which is the exact alarm-fatigue failure T-2818 and T-2833 document from both directions. The precise check is a different one and is named in the upstream filing.

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
#
# AC1 — the incident commit carries a rename with a 0/0 numstat.
git show --numstat --format= 3fd1159ef > /tmp/.t2959-a.out 2>&1
grep -qE "^0[[:space:]]+0[[:space:]]" /tmp/.t2959-a.out
# AC1 — and the body it should have carried was committed separately, after.
git show --numstat --format= 78a56a55f > /tmp/.t2959-b.out 2>&1
awk "\$3 ~ /T-2951/ && \$1+0 > 0 {f=1} END {exit f?0:1}" /tmp/.t2959-b.out
# AC3 — the benign counter-example: content commit PRECEDES the rename commit.
git show --numstat --format= 175c8bbfb > /tmp/.t2959-c.out 2>&1
grep -q "T-2876" /tmp/.t2959-c.out
# AC4 — the upstream filing is readable on the hub and names the mechanism.
termlink channel subscribe framework:pickup --cursor 123 --limit 1 --json > /tmp/.t2959-hub.json 2>&1
python3 -c 'import json,base64; d=json.loads(open("/tmp/.t2959-hub.json").read().strip().splitlines()[0]); open("/tmp/.t2959-body.txt","wb").write(base64.b64decode(d["payload_b64"]))'
grep -q "staged rename" /tmp/.t2959-body.txt
# AC5 — the decline is recorded with the count that justifies it.
grep -q "11 benign" /tmp/.t2959-body.txt

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

**Symptom:** `fw git commit` reported `1 file changed, 0 insertions(+), 0
deletions(-)` and succeeded, while the task-file edits it was meant to carry
stayed unstaged on disk. HEAD kept the superseded framing; the working tree had
the correction. No error at any step.

**Root cause:** composition of two verbs, neither wrong alone. `fw task update
--status work-completed` performs a `git mv`, which **stages the rename**. A
following `fw git commit` therefore has something staged, so the commit is
non-empty and succeeds — and the staged rename masks the omitted content edits.

**Why structurally allowed:** the safety property being relied on is git's own
refusal to make an empty commit, which is what normally makes a forgotten
`git add` harmless. The staged rename disables exactly that refusal, at exactly
the moment a task file is most likely to have just been edited. The commit's
own stat line is the only signal, and it reads as plausible.

**Prevention:** the cheap guard is **declined on measured grounds** — a check
firing on "rename-only `.tasks/` commit with 0 insertions" would fire on all 11
benign prior commits (content-first, rename-after) and be permanently red, the
alarm-fatigue failure T-2818/T-2833 document. The precise check belongs at
finalize time: before the `git mv`, refuse or auto-stage when the task file has
unstaged modifications. That code is vendored, so it is filed upstream at
`framework:pickup` offset 123 rather than patched locally (G-062).

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

### 2026-09-11 — the blast-radius scan killed the fix this task was opened to ship

- **What changed:** The task was filed expecting the scan to show prior silent
  losses and to justify a guard on "rename-only task commit with 0 insertions".
  The scan returned 11 instances of the shape and **none of them are losses** —
  sampling shows content committed first and the rename after, which is the
  normal workflow. T-2951 was the anomaly, not the rule.
- **Plan impact:** The proposed guard is inverted from useful to harmful: it
  would fire on all 11 benign commits and be permanently red. AC 5 changed from
  "ship it or say why not" to a decline with the measurement as the reason, and
  the upstream filing carries the precise check (unstaged-edits test at finalize
  time) instead of the cheap one.
- **Triggered:** No local guard. Filed upstream at `framework:pickup` offset 123
  with both the mechanism and the negative result, so the next reader does not
  re-propose the guard the data rules out.

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

### 2026-09-11T20:36:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2959-fw-git-commit-reports-success-while-sile.md
- **Context:** Initial task creation

### 2026-09-11T20:37:39Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-84a8faf1
- **Timestamp:** 2026-09-11T20:41:40Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#4 (Agent)** — Filed upstream per G-062 with the measured evidence — `agents/git/git.sh` is vendored, so a local patch is erased by the next re-vendor (`.vendor-divergence.yaml` records ~one vendor event every two m
  - **AC-verify-mismatch** (narrow, heuristic) — `path=agents/git/git.sh in: Filed upstream per G-062 with the measured evidence — `agents/git/git.sh` is vendored, so a local patch is erased by the next re-vendor (`.vendor-dive`

### 2026-09-11T20:41:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
