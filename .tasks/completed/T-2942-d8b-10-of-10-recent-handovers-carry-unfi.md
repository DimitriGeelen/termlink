---
id: T-2942
name: "D8b: 10 of 10 recent handovers carry unfilled TODO sections"
description: >
  arc-008 cycle-1 audit finding. Full census: .context/audits/arc-008-cycle1-census.md

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components:
  - .agentic-framework/agents/handover/handover.sh
related_tasks: []
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
created: 2026-09-09T17:59:45Z
last_update: 2026-09-09T23:08:34Z
date_finished: 2026-09-09T23:08:34Z
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
      F-RECALL: 1
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=4 (body:fw-audit-or-doctor); D3=0
      (no-signal); D4=0 (no-signal); F-RECALL=1 (body:episodic-only); F-ORCH=0 
      (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-09T18:03:15Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 6
    rationale: blast_radius=1 (single-component); tier=2 (workflow:build); 
      effort=6 (lines=121,acs=4)
    rubric_sha: e4a00f38e801
---

# T-2942: D8b: 10 of 10 recent handovers carry unfilled TODO sections

## Context

`fw audit` FAILs on D8b: **10 of 10** recent handovers carry unfilled `[TODO]` sections. T-2941 covers the current LATEST.md; this task covers the mechanism, because a 10-of-10 rate is not individual sessions forgetting — it is the generator emitting placeholders that nothing requires anyone to fill.

G-019 applies: fix the symptom, then ask why the framework was blind. Here the framework is *not* blind — the D8b check detects it correctly — so the gap is that detection carries no consequence. The handover is written, committed and pushed with the placeholders intact, by an auto-handover path (the pre-compact hook) where no session is present to fill them at all.

The generator (`.agentic-framework/agents/handover/handover.sh`) is **vendored**, so per G-062 a local patch is erased by the next re-vendor and the finding is filed upstream rather than fixed here. What is local and durable is the measurement and the register entry.

## Acceptance Criteria

### Agent
- [x] Measured, not estimated. Across the 10 most recent handovers the count is **uniformly 5**, except S-2026-0909-1934 at 1 — the single handover a session filled by hand. The five are: one instructional comment in the file header (the P-073 defect), plus four content sections named `## Decisions Made This Session`, `## Things Tried That Failed`, `## Open Questions / Blockers`, `## Gotchas / Warnings for Next Session`.
- [x] Confirmed, and it inverts the task's premise. Each of the four content markers carries its own justification inside the text D8b counts: *"T-2882: the generator cannot know this — an unfilled section must read as unfilled."* The generator is not failing to fill them; it is deliberately refusing to, so a handover that looks complete is never one that is not. D8b is therefore firing on designed behaviour, and the two guards contradict each other.
- [x] Filed upstream at `framework:pickup` **offset 119** as P-076, per G-062. The proposal was reframed by the measurement: NOT "the generator should omit sections it cannot fill" (T-2882 deliberately forbids that) but "D8/D8b must stop counting T-2882's intentional markers as rot" — either exclude them, or replace D8b's rolling-window rot framing with a measure of whether the CURRENT handover was enriched, which is actionable.
- [x] `.vendor-divergence.yaml` checked — and the first answer was WRONG. I made no change this session, but a registered divergence for `.agentic-framework/agents/handover/handover.sh` already exists (line 88, dated 2026-09-03, `status: filed-upstream`, `kind: fix`). **T-2882 is a LOCAL fix, not upstream design.** It replaced fabricated narrative constants — `None` for Decisions / Things Tried / Open Questions, `See gaps register above.` for Gotchas — with honest `[TODO:]` markers, because a fabricated constant is indistinguishable from a reasoned answer in the one artefact every session start reads. So D8b fires precisely BECAUSE this project applied an integrity fix. No new registration is needed; the existing entry covers it.

## Verification

# Premise: the generator emits the markers D8b counts.
test -f .agentic-framework/agents/handover/handover.sh
# The four content markers carry T-2882's deliberate-unfilled justification.
grep -q 'an unfilled section must read as unfilled' .context/handovers/LATEST.md
# The finding is filed upstream as a durable envelope (any pickup location).
cat .context/pickup/inbox/P-076-bug-report.yaml .context/pickup/processed/P-076-bug-report.yaml .context/pickup/auto-deferred/P-076-bug-report.yaml 2>/dev/null > /tmp/.p-076; test -s /tmp/.p-076
# The filing names the contradiction, not merely the symptom.
grep -q 'T-2882' /tmp/.p-076

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

### 2026-09-09 — the premise inverted twice
- **What changed (first):** Filed as "handovers are rotting — 10 of 10 carry unfilled sections".
  Measurement killed that: the count is uniformly **5** across the 10 most recent, except the
  one handover a session filled by hand (1). A constant is not decay. D8b was restating the
  generator's fixed output and calling it rot.
- **What changed (second, larger):** The four content markers carry their own justification
  inside the text D8b counts — *"T-2882: the generator cannot know this — an unfilled section
  must read as unfilled."* Checking `.vendor-divergence.yaml` for AC4 then showed **T-2882 is a
  LOCAL divergence in this project**, not upstream design: it replaced fabricated constants
  (`None`, `See gaps register above.`) with honest markers. So the audit is penalising this
  project for an integrity fix it applied to its own vendored copy.
- **Plan impact:** The upstream proposal was rewritten. Not "make the generator omit sections it
  cannot fill" — T-2882 deliberately forbids exactly that — but "D8/D8b must stop counting
  deliberate markers as rot". D8b's stated remediation is also unreachable: the only honest way
  to fill a past session's "Decisions Made This Session" is to have been in that session, so
  satisfying the gate would require fabrication, which is what T-2882 exists to prevent. A gate
  satisfiable only by lying is one operators learn to ignore.
- **Triggered:** Filed as P-076 at `framework:pickup` offset 119, plus a correction addendum
  once the local-divergence provenance was established (the original filing implied T-2882 was
  upstream's, which upstream would not recognise). **Raises the cost of the T-2950 sovereign
  re-vendor question:** T-2882 is `filed-upstream`, i.e. NOT confirmed landed, so a re-vendor
  deletes it and silently restores fabricated narrative constants to the first artefact every
  session reads. That consequence was not visible when the re-vendor question was surfaced.

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

### 2026-09-09T17:59:45Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2942-d8b-10-of-10-recent-handovers-carry-unfi.md
- **Context:** Initial task creation

### 2026-09-09T18:22:54Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-7a8fcb6c
- **Timestamp:** 2026-09-09T23:08:35Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-09T23:08:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
