---
id: T-2943
name: "D8 audit check can never PASS: generator comment counts toward its own TODO
  tally"
description: >
  arc-008 cycle-1 finding, discovered while executing T-2941. Not present in the audit
  output — found by doing the work.

status: captured
workflow_type: build
owner: agent
horizon: now
tags: []
components:
  - .agentic-framework/agents/audit/audit.sh
  - .agentic-framework/agents/handover/handover.sh
related_tasks:
  - T-2941
  - T-2942
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
created: 2026-09-09T18:05:42Z
last_update: '2026-09-09T18:07:35Z'
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
  - ts: '2026-09-09T18:07:35Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 1
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=0 (no-signal); 
      D4=0 (no-signal); F-RECALL=1 (body:episodic-only); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-09T18:07:35Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 6
    rationale: blast_radius=3 (2-components); tier=2 (workflow:build); effort=6 
      (lines=132,acs=4)
    rubric_sha: e4a00f38e801
---

# T-2943: D8 audit check can never PASS: generator comment counts toward its own TODO tally

## Context

Found while executing T-2941, not by reading the audit output.

The D8 handover-quality check counts placeholder markers with a raw `grep -c` over the whole file (`audit.sh:5086`). The handover generator emits an instructional comment at `handover.sh:712` that itself contains the literal marker. So **every** handover the generator produces carries at least one occurrence, and D8's floor is 1.

Consequences, all measured on this repo rather than inferred:

- `pass "D8: Handover quality — no [TODO] in LATEST.md"` (`audit.sh:5098`) is **unreachable dead code**. No handover produced by the current generator can satisfy it.
- A fully and carefully filled handover still reports `[WARN] D8: Handover quality — LATEST.md has 1 [TODO] section(s)`. The operator is told to "Fill remaining [TODO] sections" when there are none to fill.
- Prose *about* the marker also counts, so documenting the defect inflates it — observed directly while writing T-2941's handover sections (count went 1 → 2 and back).

This is the attention-exhaustion shape the repo already documents from the other direction in T-2818: a guard that reports a problem which cannot be fixed teaches its operator to stop reading that guard. Here it is worse than noise, because the residual WARN is indistinguishable from a genuinely half-filled handover.

D8b (`audit.sh:5108`) uses a per-file threshold of `>3` rather than `>0`, so it is **not** affected — it clears correctly once sections are filled. The two checks read the same tally with different thresholds, which is why the FAIL is fixable and the WARN is not.

Both files are **vendored** (G-062): a local patch is erased by the next re-vendor, so the fix belongs upstream and this task's local deliverable is the measurement, the filing, and the register entry.

## Acceptance Criteria

### Agent
- [ ] The floor is demonstrated, not asserted: a handover with every fillable section completed still yields a non-zero count, captured as the exact command and its output
- [ ] The two thresholds are stated precisely — D8 warns at `>0` (floor 1, unreachable PASS) while D8b stales at `>3` (reachable) — so that a future reader does not "fix" D8b by mistake
- [ ] The defect is filed upstream at `framework:pickup` per G-062 with a concrete proposal (count only markers in section-body position, or have the generator not emit the literal in its own instruction text), and the filing offset is recorded in this task
- [ ] `.vendor-divergence.yaml` is updated if any local change is made to either vendored file, or it is stated explicitly that none was made and why

## Verification

```bash
c=$(grep -c '\[TODO' .context/handovers/LATEST.md || true); test "$c" -ge 1
grep -q 'no \[TODO\] in LATEST.md' .agentic-framework/agents/audit/audit.sh
```

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

### 2026-09-09T18:05:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2943-d8-audit-check-can-never-pass-generator-.md
- **Context:** Initial task creation
