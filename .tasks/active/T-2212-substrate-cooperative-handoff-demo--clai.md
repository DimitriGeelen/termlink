---
id: T-2212
name: "substrate cooperative-handoff demo — claim-transfer ownership proof"
description: >
  substrate cooperative-handoff demo — claim-transfer ownership proof

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-13T14:19:03Z
last_update: 2026-06-13T14:26:01Z
date_finished: 2026-06-13T14:23:41Z
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

# T-2212: substrate cooperative-handoff demo — claim-transfer ownership proof

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `scripts/substrate-cooperative-handoff-demo.sh` exists, is executable, and passes `bash -n` (syntax-clean).
- [x] The demo proves the orchestrator→worker cooperative-handoff mechanic end-to-end against the live local hub, composing ONLY shipped verbs (`channel create/post/claim/claim-transfer/renew/release/claims-summary`) — no new primitive, no hub change.
- [x] Hard assertions cover BOTH the positive path (orchestrator claims → transfers to worker → worker renews → worker releases with --ack) AND the `CLAIM_NOT_OWNED` ownership-enforcement negative path (stale `--by` transfer refused; non-owner release refused).
- [x] Captured evidence written to `docs/reports/T-2212-substrate-cooperative-handoff-demo.md` (≥1 live PASS run with exit code + per-step output) and the script is wired alongside the T-2211 drain demo as the second arc-001 proof.

### Human
- [ ] [REVIEW] Confirm the cooperative-handoff demo is a faithful, useful proof of the canonical orchestrator pattern documented in `docs/operations/substrate-orchestrator-recipe.md`.
  **Steps:**
  1. `bash scripts/substrate-cooperative-handoff-demo.sh`
  2. Read `docs/reports/T-2212-substrate-cooperative-handoff-demo.md`
  **Expected:** Exit 0, "COOPERATIVE-HANDOFF DEMO PASS", every step's assertion green; report matches observed output.
  **If not:** Re-run with `--json` and attach output; file a bug task.

## Verification
bash -n scripts/substrate-cooperative-handoff-demo.sh
test -x scripts/substrate-cooperative-handoff-demo.sh
test -f docs/reports/T-2212-substrate-cooperative-handoff-demo.md

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

### 2026-06-13T14:19:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2212-substrate-cooperative-handoff-demo--clai.md
- **Context:** Initial task creation

## Recommendation

**Recommendation:** GO (partial-complete)

**Rationale:** All 4 Agent ACs verified green. The demo proves the
orchestrator→worker cooperative-handoff mechanic (substrate primitive #3,
claim-transfer) end-to-end against the live hub, composing only shipped verbs
(no new primitive, no hub change). It is the directed-assignment complement to
T-2211's work-stealing proof and exercises the canonical orchestrator pattern
documented in docs/operations/substrate-orchestrator-recipe.md. The single
Human [REVIEW] AC (faithful/useful proof?) is taste — mechanically green, the
operator's call. Owner → human.

**Evidence:**
- scripts/substrate-cooperative-handoff-demo.sh — exists, executable, `bash -n` clean
- 7/7 assertions green: 3 positive lifecycle steps (claim → transfer → renew → release) + 3 CLAIM_NOT_OWNED ownership-gate refusals (stale --by transfer, ex-owner renew, ex-owner release)
- 4 consecutive PASS runs, exit 0, bounded topic (no per-run growth)
- docs/reports/T-2212-substrate-cooperative-handoff-demo.md — live-captured runs
- arc-parallel-substrate.yaml demo_evidence wired as proof #2 (closure still human-gated on T-2022/24/25/26)
- P-011 verification gate: 3/3 passed

### 2026-06-13T14:23:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
