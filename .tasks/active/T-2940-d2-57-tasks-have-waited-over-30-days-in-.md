---
id: T-2940
name: "D2: 57 tasks have waited over 30 days in the human review queue"
description: >
  arc-008 cycle-1 audit finding. Full census: .context/audits/arc-008-cycle1-census.md

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components:
  - .context/audits/arc-008-cycle1-census.md
related_tasks: [T-2194]
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
created: 2026-09-09T17:56:23Z
last_update: 2026-09-18T15:41:49Z
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
  - ts: '2026-09-09T18:03:07Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 4
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=4 (body:fw-audit-or-doctor); D3=0
      (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-09T18:03:15Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 4
    rationale: blast_radius=1 (single-component); tier=2 (workflow:build); 
      effort=4 (lines=125,acs=2)
    rubric_sha: e4a00f38e801
---

# T-2940: D2: 57 tasks have waited over 30 days in the human review queue

## Context

`fw audit` FAILs on D2: **57 tasks have waited more than 30 days** in the human review queue, the oldest 132 days (T-1417). The full list is in `.context/audits/arc-008-cycle1-census.md`.

This is a sovereignty-boundary finding, not a code defect. Every one of these tasks is blocked on a human decision the agent is structurally forbidden to make (CLAUDE.md: completing human-owned tasks is not delegated by autonomous mode). The agent cannot clear this queue; only triage can.

The queue's size is itself the finding: a review queue with 57 items aged past a month is one nobody reads, which is the same attention-exhaustion failure the guard layer documents elsewhere (T-2818: a gate that fires wrongly often enough teaches its operator to stop reading it).

## Acceptance Criteria

### Agent
- [x] Finding is reproduced and recorded with the exact audit line, and the remediation command is verified to be the correct one before the human runs it

### Human
- [ ] [REVIEW] Review queue triaged to a readable size
  **Steps:**
    `cd /opt/termlink && .agentic-framework/bin/fw review-queue` to list them with verdicts. For each: close it, re-own it to agent if the remaining work is not actually a human judgement, or explicitly defer it with `revisit_at` set (T-1451/G-053) so it has a structural reminder instead of sitting silent.
  **Expected:** the D2 FAIL either clears or reports a materially smaller count, and every remaining item is one a human genuinely still needs to decide.
  **If not:** if the 57 are mostly mis-owned rather than genuinely pending, that is a separate structural finding about ownership assignment at task creation — file it rather than bulk-reassigning.

## Verification

```bash
test -f .context/audits/arc-008-cycle1-census.md
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

### 2026-09-18 — the finding is a moving target, and the audit's own list overstates it
- **What changed:** The count is not static — it moved 57 → 58 between filing and reproduction (T-2723 aged past 30d), so any triage that takes weeks is chasing a growing queue. Separately, the audit's printed ID list is unfiltered (68 IDs shown, 10 under threshold) while its header count is correctly filtered — a reader trusting the list over-counts.
- **Plan impact:** The Human AC's "materially smaller count" expectation should be read against the live count at triage time, not 57. The list/header mismatch is a vendored check-quality defect: recorded here, not patched (G-062).
- **Triggered:** No new task — the mismatch is cosmetic and the census (`.context/audits/arc-008-cycle1-census.md`) plus T-2194's classification already carry the authoritative breakdown.

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

**Recommendation:** GO
**Rationale:** The agent half is complete and verified: the D2 finding is reproduced from a full `fw audit` run (58 tasks >30d, oldest T-1417 at 141d, saved `.context/audits/2026-09-18.yaml`), and the remediation entry point the Human AC hands you — `fw review-queue` — was executed and confirmed to render the queue with verdicts (7 pending inception decisions + 138 Human-AC verdicts). The remaining work is pure sovereignty: 58 individual close / re-own / defer-with-`revisit_at` decisions the agent is structurally forbidden to make. T-2194 (still open, owner: human) already classified the queue — 4 RUBBER-STAMP-only tasks (T-1696, T-1722, T-1296, T-1723), ~50 tasks with REVIEW ACs, 3 mixed — so triage can start from that classification rather than from zero.
**Evidence:**
- Audit line reproduced 2026-09-18: `[FAIL] D2: Human review queue — 58 task(s) waiting >30d` (count moved 57 → 58 since filing; T-2723 crossed the threshold)
- `cd /opt/termlink && .agentic-framework/bin/fw review-queue` verified as the correct triage verb; `fw inception sweep` verified NOT to be an alternative (it only finalizes tasks already carrying a `## Decision` block)
- Check-quality defect recorded (vendored, not patched — G-062): the audit's printed ID list is unfiltered and over-counts by 10 under-threshold tasks; header count is correct
- Fastest first cut: the 4 RUBBER-STAMP-only tasks from T-2194's classification are mechanical approvals

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

### 2026-09-09T17:56:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2940-d2-57-tasks-have-waited-over-30-days-in-.md
- **Context:** Initial task creation

### 2026-09-18T15:41:49Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-09-18T16:05:00Z — reproduced + remediation verified [agent, autonomous run]
- **Reproduced (full `fw audit`, 2026-09-18, saved `.context/audits/2026-09-18.yaml`; summary 374 pass / 79 warn / 5 fail):**
  `[FAIL] D2: Human review queue — 58 task(s) waiting >30d: T-1417(141d) T-1419(141d) T-1435(140d) … T-2723(34d)` — oldest T-1417 at 141d. Count moved 57 → 58 since filing (T-2723 crossed 30d).
- **Check-quality note (vendored audit, not fixed here):** the header count is correctly filtered to >30d (58), but the printed list is unfiltered — it shows 68 IDs including 10 under threshold (T-2409 25d, T-2706 21d, T-2709 22d, T-2711 22d, T-2822 25d, T-2836 25d, T-2839 22d, T-2861 18d, T-2873 16d, T-2878 16d). Cosmetic, but a reader trusting the list over-counts by 10.
- **Remediation verified:** `cd /opt/termlink && .agentic-framework/bin/fw review-queue` runs and renders the queue with verdicts (7 pending inception decisions + 138 Human-AC verdicts) — it is the correct triage entry point for the Human AC. The audit's own mitigation line (`fw inception sweep`, T-1514) is NOT an alternative: it only ticks-and-finalizes tasks that already carry a recorded `## Decision` block, so it is the batch step AFTER triage decisions are recorded, not a way to shrink the queue by itself.
- **Prior work, linked not merged (arc-008 rule):** T-2194 (2026-08-20, owner: human, still open) already classified this queue — 4 RUBBER-STAMP-only tasks (T-1696, T-1722, T-1296, T-1723), 56 REVIEW ACs across ~50 tasks, 3 mixed — and recorded the strategy. This task is the arc-008 re-filing of the same audit line; it adds the reproduction and the remediation check, nothing else. Added to `related_tasks`.
- **Sovereignty:** every item in the queue is a human decision; the agent's work on this task is complete at "reproduced and verified". Nothing in the queue was touched.
