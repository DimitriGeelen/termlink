---
id: T-2223
name: "substrate RESILIENCE demo — offline-queue absorbs hub blip + exactly-once drain (arc-001 #5 proof)"
description: >
  substrate RESILIENCE demo — offline-queue absorbs hub blip + exactly-once drain (arc-001 #5 proof)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/substrate-resilience-demo.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-13T20:16:49Z
last_update: 2026-06-13T20:22:51Z
date_finished: 2026-06-13T20:22:51Z
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
---

# T-2223: substrate RESILIENCE demo — offline-queue absorbs hub blip + exactly-once drain (arc-001 #5 proof)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `scripts/substrate-resilience-demo.sh` exists, is executable, and passes `bash -n` (syntax clean).
- [x] The demo runs a fully ISOLATED throwaway hub (temp `TERMLINK_RUNTIME_DIR` + `TERMLINK_IDENTITY_DIR`) and never touches the operator's live hub or `~/.termlink/outbound.sqlite`.
- [x] `scripts/substrate-resilience-demo.sh --json` exits 0 with `verdict:"PASS"` and asserts the three RESILIENCE properties (PL-213): (a) a post during a hub blip is QUEUED not silent-dropped, (b) the queue auto-drains on the next post once the hub returns, (c) a replay of the same `client_msg_id` is absorbed exactly-once (topic count unchanged).
- [x] The demo is wired into `scripts/substrate-smoke.sh` as a regression stage alongside the T-2211/2212/2214 arc-demo gates.


## Verification

out=$(bash scripts/substrate-resilience-demo.sh --json 2>&1); echo "$out" | jq -e '.verdict=="PASS"' >/dev/null
bash -n scripts/substrate-resilience-demo.sh
test -x scripts/substrate-resilience-demo.sh
grep -q substrate-resilience-demo scripts/substrate-smoke.sh


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

### 2026-06-13T20:16:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2223-substrate-resilience-demo--offline-queue.md
- **Context:** Initial task creation

### 2026-06-13T20:22:51Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
