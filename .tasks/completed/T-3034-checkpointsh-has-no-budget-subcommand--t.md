---
id: T-3034
name: "checkpoint.sh has no 'budget' subcommand — the G-087-safe read /resume prescribes
  is unimplemented"
description: >
  The /resume skill instructs: read the budget cache via '.agentic-framework/agents/context/checkpoint.sh
  budget' and explicitly NOT via a raw cat of .context/working/.budget-status, citing
  G-087/T-222 (a failed transcript scan or foreign-session cache reads back as a plausible
  healthy zero; measured twice in production, 0 vs 297923 and 0 vs 70549). The vendored
  checkpoint.sh implements only {post-tool|reset|status} — 'budget' exits 1 and trips
  the hook-crash banner. So the ONLY read the framework certifies as safe is the one
  that does not exist, and the documented alternative is the one it forbids. Reproduced
  twice this session. Vendored under .agentic-framework/ so per G-062 this is filed
  upstream at framework:pickup, not patched locally.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [governance, arc:arc-009]
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
created: 2026-09-21T07:35:12Z
last_update: 2026-09-21T08:14:31Z
date_finished: 2026-09-21T08:14:31Z
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
  - ts: '2026-09-21T07:36:49Z'
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

# T-3034: checkpoint.sh has no 'budget' subcommand — the G-087-safe read /resume prescribes is unimplemented

## Context

The `/resume` skill, step 6, prescribes `.agentic-framework/agents/context/checkpoint.sh
budget` as the G-087-safe budget read and in the same breath forbids the raw
`cat .context/working/.budget-status`, citing G-087/T-222 (a failed transcript scan or a
foreign-session cache reads back as a plausible `{"level":"ok","tokens":0}` — measured
twice in production, 0 vs 297,923 and 0 vs 70,549). The vendored `checkpoint.sh`
implements only `{post-tool|reset|status}`. So the only read the framework certifies as
safe does not exist, and the only read that exists is the one it forbids.

Filed upstream at `framework:pickup@134` (G-062 — vendored code is not patched here).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The defect is reproduced by a recorded check, not asserted: invoking
      `.agentic-framework/agents/context/checkpoint.sh budget` exits non-zero and its
      own usage line offers only `{post-tool|reset|status}` — `budget` is absent
- [x] The contradiction is stated in this task: `/resume` step 6 prescribes the
      `budget` subcommand AND explicitly forbids the raw `cat` of
      `.context/working/.budget-status` (G-087/T-222), so the only read the framework
      certifies as safe is unimplemented and the only implemented read is forbidden
- [x] No local patch is made under `.agentic-framework/` (G-062: a local fix to
      vendored code is deleted by the next re-vendor) — verified by a clean porcelain
- [x] The defect is filed upstream at `framework:pickup` and the resulting offset is
      recorded in this task file, per CLAUDE.md: if you are writing a report to a file
      so a human can relay it, post it to the rail instead

## Verification

# T-3034. Safe redirect form only (L-387): never `cmd | grep -q PAT`.
# The subcommand is EXPECTED to fail, so capture first, then assert on the text.
.agentic-framework/agents/context/checkpoint.sh budget > /tmp/.t3034-b 2>&1 || true
grep -qF "post-tool|reset|status" /tmp/.t3034-b
grep -q "Usage: checkpoint.sh" /tmp/.t3034-b
# G-062: the vendored tree must carry no local patch.
git status --porcelain .agentic-framework/ > /tmp/.t3034-g 2>&1
test ! -s /tmp/.t3034-g
# The upstream filing offset must be recorded in this task file.
grep -q "framework:pickup@" .tasks/active/T-3034-checkpointsh-has-no-budget-subcommand--t.md

## RCA

**Symptom:** `checkpoint.sh budget` exits non-zero, prints `Usage: checkpoint.sh
{post-tool|reset|status}`, and trips the framework's HOOK CRASHED banner — which tells the
operator to report a malfunction for a subcommand that was never implemented. Reproduced
3 times across 2 sessions.

**Root cause:** The documented contract for `budget` is not "print the cache"; it is to
REJECT a cache that is stale, from another session, or self-marked `unknown`, answering
`level: unknown` plus a reason. That provenance-checking logic is what G-087/T-222 exists
to provide, and it was never written. `status` is not a substitute — it prints a number
with no provenance check, which is the same failure surface as the raw `cat`, just
prettier.

**Why structurally allowed:** The prescription and the implementation live in different
artefacts with nothing tying them together — `/resume` is a skill body, `checkpoint.sh` is
vendored code, and no check asserts that a command a skill instructs the operator to run
actually exists. This is the guard-layer shape this repo keeps finding: the safe path is
documented, prescribed, and absent, so in practice everyone takes the unsafe one and
believes they are compliant. Compounding it, the missing verb fails LOUDLY as a hook crash,
which trains the reader that the safe read is broken tooling rather than absent tooling.

**Prevention:** Filed upstream at `framework:pickup@134` with the suggested implementation
(including the specific point that `level: unknown` must exit 0 — it is an ANSWER, not a
crash — since the non-zero exit is what manufactures the false malfunction report). The
prevention is upstream's to land; this task's local contribution is the filing and the
recorded reproduction. A local detector asserting "every command a skill prescribes
resolves" is a plausible follow-up but was not built here — it is not this task's scope
and would be a new check, not a fix.

## Evolution

### 2026-09-21 — the missing verb is the smaller half

- **What changed:** At filing this read as "a subcommand is missing". Writing the upstream
  report made the sharper point visible: what is missing is not a printer but the
  provenance REJECTION logic (stale / foreign-session / self-marked-unknown), which is the
  entire substance of G-087. Someone could add a `budget` verb that cats the cache, close
  this defect, and leave G-087 exactly as unguarded as it is today.
- **Plan impact:** The upstream filing therefore leads with the contract, not the usage
  line, and states explicitly that `status` is not a substitute. It also names the
  either/or: if `status` IS meant to serve this role, then `/resume` step 6 is the thing
  that is wrong — but one of the two must move.
- **Triggered:** No new sub-task. The filing carries both branches so the decision is
  upstream's to make; deciding it here would be resolving a question that is not mine.

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

### 2026-09-21T07:35:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3034-checkpointsh-has-no-budget-subcommand--t.md
- **Context:** Initial task creation

### 2026-09-21T07:36:49Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-13d59ae8
- **Timestamp:** 2026-09-21T08:14:32Z
- **Catalogue:** v1.3-seed
- **Overall:** FAIL
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **swallowed-errors** (severe, deterministic) @ Verification:line 3
     - evidence: `.agentic-framework/agents/context/checkpoint.sh budget > /tmp/.t3034-b 2>&1 || true`

### 2026-09-21T08:14:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
