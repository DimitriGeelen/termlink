---
id: T-2938
name: "cron drift: agentic-audit.crontab differs from deployed /etc/cron.d copy"
description: >
  arc-008 cycle-1 audit finding. Full census: .context/audits/arc-008-cycle1-census.md

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components:
  - .context/cron/agentic-audit.crontab
  - scripts/check-cron-install-drift.sh
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
created: 2026-09-09T17:54:17Z
last_update: 2026-09-18T18:41:57Z
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
  - ts: '2026-09-09T18:03:06Z'
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
  - ts: '2026-09-09T18:03:14Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 4
    rationale: blast_radius=3 (2-components); tier=2 (workflow:build); effort=4 
      (lines=125,acs=2)
    rubric_sha: e4a00f38e801
---

# T-2938: cron drift: agentic-audit.crontab differs from deployed /etc/cron.d copy

## Context

`fw audit` and `fw doctor` both FAIL on cron drift: `.context/cron/agentic-audit.crontab` in the repo differs from the deployed `/etc/cron.d/agentic-audit-termlink`. The deployed copy is what actually runs, so the registry is not the source of truth for the audit's own schedule — the audit can be running stale or absent while the repo says otherwise.

Scope is `worktree` (host state under `/etc/cron.d`), so this does not block a push (T-3126) and is not present in any committed ref. It is still real: the schedule that runs is not the schedule declared.

Shares host-state root cause with T-2939 (a declared canary crontab never installed) and T-2943 (flock parity). Linked, not merged.

## Acceptance Criteria

### Agent
- [x] Finding is reproduced and recorded with the exact audit line, and the remediation command is verified to be the correct one before the human runs it
- [ ] [REVIEWER] The deployed crontab matches the registry — no drift, nothing uninstalled
  **Converted from a `### Human` `[RUBBER-STAMP]` AC on 2026-09-26 (T-3154, operator GO on SQ-1).**
  Its `**Expected:**` clause — *"command exits 0; re-running `fw audit --sections structure` no longer prints `[FAIL] Cron drift`"* — is settled by a deterministic command, so per CLAUDE.md T-1811/T-1878 it belongs here with the check in `## Verification`. The `sudo fw cron install` action was an operator act and has already been performed.
  **Measured 2026-09-26:** `check-cron-install-drift.sh` reports *healthy — 30 installed + matching, 0 acknowledged, 0 skipped*, `ok:true`, missing 0 / uninstalled_jobs 0 / drift 0, rc 0. The audit's structure section prints `[PASS] Cron registry in sync with /etc/cron.d/agentic-audit-termlink`.
  **Left unticked deliberately** — conversion changes who *can* verify, not whether the task is done.

### Human
_The `[RUBBER-STAMP]` AC that was here has moved to `### Agent` as a `[REVIEWER]` AC — T-3154, operator GO on SQ-1. Its Expected clause was settled by a deterministic command, so per T-1811/T-1878 it does not belong in the human queue. The original Steps (`sudo fw cron install`) were an operator act and have already been performed. The "If not" guidance is preserved here because it remains the right diagnosis if the check ever fires again: run `diff .context/cron/agentic-audit.crontab /etc/cron.d/agentic-audit-termlink`; if the deployed copy carries an operator-added job the registry lacks, add it to the registry FIRST — do not let `cron install` silently discard it._

## Verification

# T-3154: de-fenced. P-011 extracts plain non-comment lines, so a ```bash fence risks
# the fence markers being executed as commands — verification that cannot run gates
# nothing (T-2831 class).
diff -q .context/cron/agentic-audit.crontab /etc/cron.d/agentic-audit-termlink
# the converted [REVIEWER] AC: registry-wide, not just this one file
bash scripts/check-cron-install-drift.sh > /tmp/.t2938-cron.out 2>&1 && grep -q "healthy" /tmp/.t2938-cron.out

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

**Recommendation:** GO
**Rationale:** The Human AC's expected end state is ALREADY LIVE — the stamp is a confirmation, not an action. Reproduction was attempted 2026-09-18 and the finding no longer reproduces: `diff /opt/termlink/.context/cron/agentic-audit.crontab /etc/cron.d/agentic-audit-termlink` reports identical, and today's audit prints `[PASS] Cron registry in sync with /etc/cron.d/agentic-audit-termlink` (`.context/audits/2026-09-18.yaml`), where the 09-16 and 09-17 audits still carried the drift. The resolving event is traceable: T-2870 regenerated the registry crontab (commits 0d58b34d1, 5af8510f2) and the deployed copy was installed today at 17:47. This task can be closed on that evidence.
**Evidence:**
- Live check (2026-09-18): `diff -q` exits 0 — deployed copy byte-identical to the registry source; this task's own Verification line passes
- Audit trail: drift present in `.context/audits/2026-09-16.yaml` and `2026-09-17.yaml`; PASS in `2026-09-18.yaml`
- Resolving change: T-2870 flock/registry regeneration; `/etc/cron.d/agentic-audit-termlink` mtime 2026-09-18 17:47
- Remediation command in Steps (`sudo fw cron install` from the main checkout) verified correct-in-principle and now a no-op; the T-2815 worktree-basename caveat does not apply (this is the main checkout)

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

### 2026-09-09T17:54:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2938-cron-drift-agentic-auditcrontab-differs-.md
- **Context:** Initial task creation

### 2026-09-18T18:41:30Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-09-18T18:55:00Z — finding no longer reproduces; resolved in field by T-2870 [agent, autonomous run]
- **Reproduction attempt:** `diff` of registry source vs deployed copy → IDENTICAL. Audit 2026-09-18: `[PASS] Cron registry in sync with /etc/cron.d/agentic-audit-termlink`. The 09-16/09-17 audits still showed the drift.
- **Resolving event:** T-2870 regenerated `.context/cron/agentic-audit.crontab` from the registry (0d58b34d1, 5af8510f2) and the deployed copy was installed 2026-09-18 17:47.
- **Human AC evidence (not ticked — sovereignty):** its Expected state ("audit no longer prints [FAIL] Cron drift") is the current live state. Recommend close on the cited evidence per the Human Task Completion Rule.
