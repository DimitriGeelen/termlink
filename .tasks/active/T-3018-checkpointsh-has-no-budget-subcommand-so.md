---
id: T-3018
name: "checkpoint.sh has no 'budget' subcommand, so the G-087-safe budget read prescribed
  by /resume and CLAUDE.md crashes"
description: >
  The /resume skill and CLAUDE.md Session Start Protocol both prescribe '.agentic-framework/agents/context/checkpoint.sh
  budget' as the G-087-safe way to read context budget, explicitly warning NOT to
  raw-cat .context/working/.budget-status because a stale or foreign-session cache
  reads back as a plausible {level:ok,tokens:0}. The vendored checkpoint.sh implements
  only {post-tool|reset|status}: invoking 'budget' prints usage, exits 1, and triggers
  a 'HOOK CRASHED (exit 1)' banner. So the documented safe path does not exist and
  an agent following the instruction either crashes or falls back to the exact raw
  read G-087 forbids. Observed twice across sessions (2026-09-18 and 2026-09-20) and
  not previously filed. Workaround: 'checkpoint.sh status' reports tokens and percentage.
  Vendored (G-062) so the fix is upstream.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: [.agentic-framework/agents/context/checkpoint.sh, .agentic-framework/agents/context/budget-gate.sh, scripts/check-task-id-collisions.sh]
related_tasks: [T-2950, T-2961, T-2949, T-3028, T-3029]
arc_id: arc-008
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
# demo_target: true               # T-2286: optional — marks task as reserved for an orchestrated demo
#                                 # worker (e.g. arc-010 HM-A dispatches via mcp__fw__work_on). When set,
#                                 # `fw work-on T-XXX` refuses unless --i-am-demo-orchestrator (CLI) or
#                                 # FW_I_AM_DEMO_ORCHESTRATOR=1 (env) is passed. Prevents the parent
#                                 # session from consuming the captured→started-work transition the demo
#                                 # worker expects to drive. Origin OBS-057.
created: 2026-09-20T10:34:39Z
last_update: 2026-09-20T15:10:37Z
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
  - ts: '2026-09-20T13:16:21Z'
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
  - ts: '2026-09-20T13:16:27Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (single-component); tier=2 (workflow:build); 
      effort=8 (lines=204,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3018: checkpoint.sh has no 'budget' subcommand, so the G-087-safe budget read prescribed by /resume and CLAUDE.md crashes

## Context

Filed 2026-09-20 as a fresh arc-008 finding. Executing AC2 (measure the blast radius) disproved
two of the three premises in the filing, and disproved them by measurement rather than argument.
The corrected position is recorded in `## Correction` below; the criteria have been rewritten to
match what is true, with the original text preserved there.

The defect itself is real and reproduces on demand (AC1). What is false is that it was new, and
that CLAUDE.md prescribes it.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The defect is demonstrated, not asserted. `.agentic-framework/agents/context/checkpoint.sh budget`
      prints `Usage: checkpoint.sh {post-tool|reset|status}`, exits **1**, and trips the framework's
      `HOOK CRASHED: checkpoint (exit 1)` banner. The dispatch `case` carries exactly three arms —
      `:277 post-tool)`, `:427 reset)`, `:439 status)` — and a catch-all at `:457` that prints the
      usage line and `exit 1`. The script's own header documents the same three (`:13-15`). There is
      no `budget` arm anywhere in its 461 lines.
- [x] The blast radius is measured, and the measurement **contradicts the filing**. Exactly **one**
      surface prescribes `checkpoint.sh budget`: the `/resume` skill, which is **user-level**
      (`userSettings:resume`), not in this repo. `.claude/` contains **zero** occurrences.
      **CLAUDE.md does not prescribe it** — it prescribes `checkpoint.sh status`, which exists
      (`CLAUDE.md:2906`, `:2919`), and contains **zero** mentions of G-087. The in-repo occurrences
      are all *about* the defect, never prescriptions of it: `.vendor-divergence.yaml:94`,
      `.agentic-framework/agents/context/lib/safe-commands.sh:660`, and
      `tests/safe-commands-checkpoint-fixtures.sh:52,70`.
- [x] The G-087 fallback hazard is stated concretely **and its limits stated with it**. An agent
      following the skill runs `budget`, gets exit 1, and falls back to `cat .context/working/.budget-status`.
      Measured live this run: the cache read `{"level":"ok","tokens":195853}` while the true figure
      was **209,271** — **13,418 tokens stale** after ~20 minutes, with no freshness or
      session-ownership field a reader could check. **The `level` was nonetheless correct**, because
      the live gate's thresholds are 75/85/95% of a 300K window (`budget-gate.sh:106-108`) and
      195,853 is ~65%. So this reading was stale-but-benign and is **not** itself a G-087 instance;
      it demonstrates the missing freshness check, not the dangerous outcome. The dangerous outcome
      is cited, not re-measured: G-087/T-222 recorded 0 vs 297,923 and 0 vs 70,549 tokens.
- [x] **Not filed upstream — and the original criterion demanding it was wrong.** This finding was
      already closed as **T-2950** on 2026-09-09, eleven days before T-3018 was filed, under the same
      arc. T-2950's own AC4 established by measurement that the cause is **version skew, not a missing
      upstream feature** (vendored `.agentic-framework` is **1.6.29**, baseline **2026-06-08**; CLAUDE.md
      references upstream **v1.6.295**), and its AC5 recorded a deliberate decision **not** to file,
      because upstream almost certainly already ships the arm and a likely-already-fixed report is
      exactly the register noise **P-075 (T-2949)** objects to. **T-2961** reached the same conclusion
      independently on 2026-09-19 and recorded it in `.vendor-divergence.yaml:94`. Filing now would
      contradict two standing recorded decisions and manufacture the phantom debt both name. No local
      patch to `.agentic-framework/` was made — stated, per G-062. Remediation remains the re-vendor,
      which is a **Sovereign** decision already surfaced under T-2950/T-2949 and is not taken here.

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

# T-3018 — each line rehearsed under `bash -c 'set -eo pipefail; <line>'` before being
# written here (T-2743), and the first two mutant-tested: adding a `budget)` arm to a
# copy of checkpoint.sh fails line 1, appending the string to a copy of CLAUDE.md fails
# line 3. Both are load-bearing, neither is vacuous (T-2831).
test "$(grep -cE '^    budget\)' .agentic-framework/agents/context/checkpoint.sh)" = "0"
test "$(grep -cE '^    (post-tool|reset|status)\)' .agentic-framework/agents/context/checkpoint.sh)" = "3"
test "$(grep -c 'checkpoint\.sh budget' CLAUDE.md)" = "0"
grep -q 'checkpoint\.sh status' CLAUDE.md
test -z "$(grep -rl 'checkpoint\.sh budget' .claude/ 2>/dev/null)"
grep -q '^status: work-completed' .tasks/completed/T-2950-the-g-087-safe-budget-read-verb-the-resu.md
grep -q '^date_finished: 2026-09-09' .tasks/completed/T-2950-the-g-087-safe-budget-read-verb-the-resu.md
grep -q 'checkpoint.sh budget' .vendor-divergence.yaml
test -z "$(git status --porcelain .agentic-framework/agents/context/checkpoint.sh)"
grep -q 'CONTEXT_WINDOW \* 95 / 100' .agentic-framework/agents/context/budget-gate.sh
bash tests/safe-commands-checkpoint-fixtures.sh > /tmp/.t3018f 2>&1 && grep -q 'ALL PASS' /tmp/.t3018f


## RCA

**Symptom.** `.agentic-framework/agents/context/checkpoint.sh budget` — the read the `/resume`
skill names as the G-087-*safe* one — prints usage, exits 1, and trips the `HOOK CRASHED` banner.
It fired again this run, unprompted, during the `/resume` that opened the session.

**Root cause.** Version skew, established under measurement by **T-2950**, not by this task. The
vendored tree is `1.6.29` with a declared baseline of `2026-06-08`; CLAUDE.md references upstream
`v1.6.295`. The skill is user-level and tracks current upstream, which ships the arm; the vendored
build is three months behind and does not. Nothing is broken in the sense of a bug to patch — two
components at different versions disagree about a verb.

**Why structurally allowed — and this is the finding worth keeping.** T-3018 is a **duplicate of
T-2950**, filed eleven days later, in the same arc, by the same route (executing the resume
protocol). It was filed, scored through the estimators, selected as the arc's top Q1 item, and
started, before anyone noticed the finding was already closed. The framework did not catch it
because **every axis of the duplicate-work checker is cross-branch**:
`scripts/check-task-id-collisions.sh` builds its candidate set as *IDs not already in the base*
(`:168`, `:171`) and then runs all four axes over that set. Two tasks that both live on `main` are
excluded by construction before any axis looks. Run this session against the real tree, the checker
reported *"no colliding IDs, no duplicate files (7 branch(es) scanned against main)"* — correct on
its own terms, and blind to the duplicate sitting in front of it. Axis B's rare-word scorer would
comfortably have fired on these two titles (shared rare terms: *G-087-safe*, *budget*, *read*,
*resume*); it never got the chance.

**Prevention.** Filed as its own governed task rather than fixed here — the arc-008 rule is that
each finding becomes its own task, and this one is a change to a guard, not to this defect. Filing
upstream was **correctly refused** by two standing recorded decisions (T-2950 AC5, T-2961's
`upstream_note` in `.vendor-divergence.yaml:94`), both citing P-075/T-2949: a report about a
three-month-stale vendored build is phantom register debt, not a contribution. The real remediation
is the re-vendor, a Sovereign decision already surfaced under T-2950/T-2949 and not taken here.

## Correction

**Two of the three premises in the filing were false, and measurement is what disproved them.**

1. *"CLAUDE.md Session Start Protocol prescribes `checkpoint.sh budget`."* **False.** CLAUDE.md
   prescribes `checkpoint.sh status` (`:2906`, `:2919`) — which exists — and mentions G-087 zero
   times. The sole prescribing surface is the user-level `/resume` skill, outside this repo.
   `.claude/` contains no occurrence.
2. *"not previously filed."* **False.** T-2950 closed it on 2026-09-09 with five ticked criteria,
   including the version-skew diagnosis and the deliberate do-not-file decision. T-2961 reached the
   same conclusion independently on 2026-09-19 and fixed the downstream consequence (the P-002
   allowlist gap that gated the read when focus was null), with 13 green fixtures.
3. *The defect itself.* **True and reproduced** (AC1). That half of the filing stands.

The original AC4 read: *"Filed upstream to `framework:pickup` per G-062 ... offset recorded here."*
It was written on premise 2 and is therefore unexecutable: complying would contradict two standing
decisions. It is rewritten above rather than silently dropped, and the original is preserved here —
same convention applied to T-3025's AC2 last session.

**What this task's execution actually produced** is not a fix for the budget verb, which needed
none. It is the measurement that the arc re-filed its own closed finding, and the identification of
why no guard could see it.


## Evolution

### 2026-09-20 — the task was the duplicate it was filed to describe

- **What changed:** Two of the three premises in the filing were false, and AC2 — the criterion
  that says *measure* the blast radius rather than assert it — is what disproved them. CLAUDE.md
  never prescribed `checkpoint.sh budget`; it prescribes `status`, which exists. And the finding
  was not new: **T-2950** closed it eleven days earlier, in this same arc, discovered by the same
  route (executing the resume protocol), with the version-skew diagnosis already established and a
  deliberate do-not-file decision already recorded. **T-2961** then reached the same conclusion
  independently on 2026-09-19 and fixed the downstream P-002 allowlist consequence. Three arrivals
  at one finding, by three separate paths, none of which saw the others.
- **Plan impact:** AC4 became unexecutable. It instructed filing upstream per G-062; complying
  would have contradicted two standing recorded decisions and produced exactly the phantom register
  debt P-075/T-2949 names. It was rewritten to the corrected disposition with the original text
  preserved in `## Correction` — the convention used on T-3025's AC2 last session — rather than
  quietly dropped. The task's deliverable changed from *fix a broken verb* (nothing to fix; it is
  version skew awaiting a Sovereign re-vendor) to *measure why the arc could re-file its own closed
  finding*.
- **Triggered:** **T-3028** (BVP 57 / cost 2.0, Q1) — every axis of `check-task-id-collisions.sh`
  is cross-branch; its candidate set is "IDs not already in the base", so two tasks both on `main`
  are excluded by construction before any axis runs. The checker reported clean this session while
  the duplicate sat in front of it. **T-3029** (BVP 63 / cost 3.2, Q1, now the arc's top item) —
  CLAUDE.md's budget ladder contradicts the live gate; found while establishing AC3's arithmetic,
  and it has a measured cost: this very task was parked unexecuted last session at ~178K citing
  "~80% context", which is 59% of the real window and level `ok`.
- **The lesson that outlasts the instance:** AC1 ("demonstrated, not asserted") and AC2 ("measured")
  were written last session purely as scoping discipline. They are what caught this. An acceptance
  criterion that forces measurement before the title is believed will sometimes disprove the task
  it belongs to — and that is the criterion working, not failing.


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

### 2026-09-20T10:34:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3018-checkpointsh-has-no-budget-subcommand-so.md
- **Context:** Initial task creation

### 2026-09-20T14:06:42Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## PARKED (arc-008 autonomous run — scoped, no investigation done)

Selected as the next Q1 unit and started via `fw work-on`, then parked at ~80% context with
no investigation performed. Real acceptance criteria were written (above) because G-020
correctly refused every edit while they were template placeholders; scoping is all that was
done. Status is `started-work` only because `work-on` sets it — treat the work as untouched.

Why selected over T-3017 (both hv-lc, tied BVP 57 / cost 2.0): CLAUDE.md and the `/resume`
skill both prescribe `checkpoint.sh budget` as the G-087-SAFE budget read. That subcommand
does not exist, so the documented safe path fails and the reader falls back to
`cat .budget-status` — the exact hazard G-087 names (measured 0 vs 297,923 tokens in
production). A live safety gap on every session start; T-3017 is reporting noise.

Why not executed: the run hit ~80% of the context window immediately after closing T-3025.
CLAUDE.md's Work Proposal Rule permits only wrap-up above 75%, and this needs a full unit.

Next session: top Q1 item in arc-008. The ACs above are ready to execute against.
