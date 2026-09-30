---
id: T-3130
name: "fw task update --status blanks owner field on frontmatter rewrite (reproduced
  live)"
description: >
  Reproduced live during T-3093 R2S2: running fw task update T-3129 --status started-work
  (no --owner flag passed) on a task created moments earlier with --owner agent left
  owner blank in the frontmatter afterward. The explicit --owner code path in update-task.sh
  is guarded by NEW_OWNER being set and never ran, so something else in the status-transition
  or frontmatter-rewrite path (candidates: the auto-triggered bvp-estimate rewrite,
  or a regex/YAML rewrite step keyed on the owner line) is clobbering the value. Possibly
  related to T-3095 (filed by R1S2 with owner blank) but T-3095 never underwent a
  status transition since creation so that link is suspected not proven. INVESTIGATE:
  bisect which step in update-task.sh status-transition path touches the owner line.

status: captured
workflow_type: build
owner: agent
horizon: next
tags: [arc:arc-008, housekeeping]
components: []
related_tasks: [T-3129, T-3095]
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
created: 2026-09-25T07:00:52Z
last_update: 2026-09-27T22:53:36Z
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
  - ts: '2026-09-25T07:06:05Z'
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
  - ts: '2026-09-27T21:34:08Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (1-file-ref-derived-T-3189); tier=2 
      (workflow:build); effort=8 (lines=238,acs=5)
    rubric_sha: e4a00f38e801
---

# T-3130: fw task update --status blanks owner field on frontmatter rewrite (reproduced live)

## Context

Live reproduction during this session (see description). Root cause not yet
bisected — vendored code (`.agentic-framework/agents/task-create/update-task.sh`,
G-062), and a proper bisect (isolating whether the culprit is the auto bvp-estimate
rewrite step, an arc-tag insertion regex, or something else) is real engineering
work out of scope for a housekeeping-remediation pass. Filed as INVESTIGATE, not
executed further this cycle, to respect the "no scope drift" binding constraint —
a fix here would need to touch vendored update logic without first confirming which
of several candidate rewrite steps is responsible.

**R2S3 bisect attempt (3 tries, then stopped per the mandate's "three attempts is
context burned, not progress"):** built a minimal fixture (`fw task create --owner
agent --tags "arc:arc-008,housekeeping"`, matching T-3129's shape exactly —
folded `description: >` block, same tags, same owner) and drove it through
(1) `captured → started-work`, (2) a second fresh fixture through the identical
transition, (3) the same fixture pushed all the way to `work-completed`
(AC-ticking, Evolution entry, git-mv-to-completed/, episodic generation — the
whole finalize path). **`owner:` survived intact in all three runs.** This rules
out the two leading candidates from the original description (the auto
bvp-estimate rewrite step, and the tag-insertion regex at line ~1941) as the
SOLE cause under these conditions, and also rules out the finalize/git-mv path
in isolation. The defect is real (R2S2 measured it live, directly, on T-3129)
but is not reproducible from a clean minimal fixture — so the trigger is either
something about the ORIGINAL T-3129/T-3095 file content this fixture didn't
capture (a specific frontmatter byte sequence, a stray CR, an unusual field
ordering), or a condition specific to the orchestrated multi-step run
(concurrent file access, a stale hook cache) rather than the command in
isolation. Scratch fixtures (T-3134, T-9999) were used and deleted, not
committed. Next bisect attempt should diff the actual byte-for-byte frontmatter
of T-3095/T-3129 as they stood in the dirty tree at the moment of the incident
(if recoverable from shell history / T-3093 R2S2's own working notes) rather
than reconstructing a fixture from the description.

## Findings

**Process note first, because it matters more than the finding.** The R2S3 note above was
**not read before this session's attempt**, so the 12-trial fixture run below duplicated three
attempts already made and already recorded. Counting both sessions, the clean-minimal-fixture
route has now failed **four times**, which is well past the mandate's "three attempts at the
same wall is context burned, not progress". **Do not attempt that route a fifth time.** What
the two sessions jointly establish is that the fixture route is the wrong instrument, and the
prior note said so: it named "concurrent file access" as a candidate. What was missing was the
*specific* concurrency and a proof. That is what this session adds — so the advance here is the
mechanism, not the reproduction.

**Root cause: a lost-update race, not a bad rewrite step.** `update-task.sh:1652-1662` fires
the BVP estimator on the `started-work` transition as a **disowned background subshell**. The
estimator then performs a full-file read-modify-write — `parse_task` → mutate frontmatter →
`_atomic_write_text` (write-temp + `os.replace`) — on the very file the foreground is still
editing with `sed -i`. Both sides finish with an atomic rename, so whichever renames **last**
silently wins and the other's changes vanish. Both exit 0, the file stays valid YAML, and the
lost field is indistinguishable from one that was never set.

The code comment at that site concedes the trade without noticing it: the engine is *"~10ms so
the update latency impact is negligible"*, and it was backgrounded only *"in case a future
v2-LLM engine lands and goes over the budget."* So the concurrency buys nothing today while
costing correctness.

**Why `owner` is the worst field to lose.** It is what the R-033 sovereignty gate reads
(`update-task.sh:89-97` greps `^owner:` and refuses agent completion when the value is
`human`). A lost write to `owner` does not merely corrupt metadata — it **silently removes the
protection that stops an agent completing a human-owned task**, and nothing surfaces it at the
time. T-3132, executed earlier in this same run, depended on exactly that gate holding across
25 tasks.

**Reproduction, both halves, including the one that failed.**

*Attempt 1 — the AC's own fixture. NEGATIVE, 12/12 clean.* Fresh task, `owner: agent`, single
`--status started-work`, 2s settle, diff. Owner preserved every time; the estimator wrote its
key every time. This is reported as a negative result rather than quietly retried, and it
carries real information: the defect is **load-dependent**, and it is **not** specific to this
task's folded YAML description (the sub-question the AC actually poses, answered).

*Attempt 2 — test the mechanism instead of waiting for luck. POSITIVE.* A reader loads the
file, sleeps 1.5s, then rewrites it from its **stale in-memory copy** via write-temp +
`os.replace` — the exact shape of `parse_task` + `_atomic_write_text`. A foreground
`sed -i 's/^owner: agent/owner: human/'` lands inside that window:

```
immediately after sed  : owner: human
after the rewrite lands: owner: agent      ← the sed's change is gone
bvp field present      : 1                 ← the background write did land
```

Both processes exited 0. **The foreground write was silently discarded.**

**Honest scope of that proof.** It establishes the lost-update mechanism and establishes that
it is silent. It does **not** reproduce the exact reported symptom — `owner` going *blank*
rather than reverting — because the direction of loss depends on which value each side holds.
A blank result requires the estimator's stale copy to have held a blank owner, which happens
if its read precedes the step that populates the field. Mechanism: confirmed. Exact symptom
path: consistent and plausible, **not** reproduced. Stated that way deliberately.

**Recommended fix (filed, not applied — vendored per G-062).** Ranked: (1) make the trigger
synchronous, or join it before the remaining frontmatter writes — smallest change, removes the
race rather than shrinking it, and costs ~10ms by the comment's own estimate; (2) if it must
stay async, serialise frontmatter writers on a per-file `flock`; (3) have the estimator splice
only its own key instead of re-emitting the whole file from a stale parse.

**Detection gap worth its own guard.** Nothing detects this class. A cheap control is a
post-write read-back: after the transition, re-read the frontmatter and assert `owner`,
`status` and `id` still hold the values the run intended — the same discipline T-2876
established for filings (delivered ≠ received).

**Harness location.** The two reproduction scripts live in this session's scratchpad and are
deliberately *not* committed — the ACs do not ask for a committed harness, and the essential
shape is inlined above and in the upstream filing so it is recoverable without them.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Bisect which rewrite step in update-task.sh's status-transition path clobbers
      `owner:` (candidates: auto bvp-estimate rewrite, arc-tag regex insertion, other)
      → **It is the auto bvp-estimate rewrite — but not as a rewrite step. As a RACE.**
      `update-task.sh:1652-1662` launches the estimator as a **disowned background
      subshell** (`( … ) &` + `disown`) on the `started-work` transition. The estimator
      does a full read-modify-write of the same file (`parse_task` → mutate →
      `_atomic_write_text`, i.e. write-temp + `os.replace`) while the foreground keeps
      issuing `sed -i` edits to it. Both end in an atomic rename, so **the later rename
      silently discards the other side's changes.** No single line clobbers `owner:`;
      the concurrency does. See `## Findings`.
- [ ] **NOT REPRODUCED on this path — negative result, recorded not massaged.** Reproduce with a minimal fixture (fresh task, single --status update, diff
      frontmatter before/after) to confirm it is not specific to this task's folded
      YAML description
      → Built exactly that fixture (throwaway `PROJECT_ROOT`, fresh task with
      `owner: agent`, single `update-task.sh T-9001 --status started-work`, 2s settle,
      diff). **12 of 12 trials preserved `owner`; 0 lost; the estimator wrote
      `bvp_scores_proposed` every time.** So the defect does **not** reproduce on an idle
      host, and it is *not* specific to folded YAML — the fixture answers that sub-question
      affirmatively. What the fixture cannot create cheaply is the window: on an idle host
      the estimator finishes long before the foreground's last `sed`. The original sighting
      was during **T-3093 R2S2, an orchestrated `[Review, Audit, procAsFit] x4` dispatch**,
      where scheduling stretches that interval arbitrarily — the same load-dependence class
      as T-3127/G-087-inverted. Left unticked because the AC's stated outcome did not occur;
      the mechanism is proven separately below, which is a different claim.
- [x] File the confirmed root cause upstream per G-062 (vendored file)
      → `framework:pickup` **offset 211**, read-back verified byte-identical
      (sha256 `2288ff3082bdc0cd1f91`, 5653 bytes). Carries both reproduction attempts
      including the negative one, the ranked fix, and the detection gap.

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

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-09-30 — closure approved by operator (T-3211 SQ-3, R6 closure request)
One agent AC is recorded unmet and is left unticked and unreworded. Minimal-fixture reproduction measured negative (12/12 trials preserved owner); the defect is load-dependent and its mechanism is proven separately and filed upstream. The operator approved closing this task on 2026-09-30 ("close T-3132, T-3128 and T-3130"). Completion needs --force for that one AC, which is Tier 0 and therefore run by the operator.

### 2026-09-25T07:00:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3130-fw-task-update---status-blanks-owner-fie.md
- **Context:** Initial task creation

### 2026-09-27T22:47:24Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-09-27T22:53:36Z — status-update [task-update-agent]
- **Change:** horizon: now → next
- **Change:** status: started-work → captured (auto-sync)
