---
id: T-2213
name: "Harden substrate arc demos — fix T-2211 unbounded-topic fallback + regression-protect both demos via smoke suite"
description: >
  Harden substrate arc demos — fix T-2211 unbounded-topic fallback + regression-protect both demos via smoke suite

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
created: 2026-06-13T14:44:29Z
last_update: 2026-06-13T14:47:41Z
date_finished: 2026-06-13T14:47:26Z
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

# T-2213: Harden substrate arc demos — fix T-2211 unbounded-topic fallback + regression-protect both demos via smoke suite

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `scripts/substrate-drain-demo.sh` creates its topic with the working `--retention "messages:N"` single-arg form (was silently falling back to retention=forever because `--retention messages --retention-value N` is rejected by the shipped CLI). Demo still PASSes after the fix.
- [x] `scripts/substrate-smoke.sh` gains two new stages that run the two arc demos as regression gates: the N-way work-stealing race (T-2211) and the cooperative-handoff ownership-gate demo (T-2212) — coverage the existing happy-path smoke lacked.
- [x] `scripts/substrate-smoke.sh` still exits 0 end-to-end with the new stages wired in (verified live on the local hub).
- [x] `bash -n` clean on both edited scripts.

### Human
- [ ] [REVIEW] Confirm the smoke suite is the right home for the two arc-demo regression gates (vs a separate CI entry).
  **Steps:**
  1. `bash scripts/substrate-smoke.sh`
  **Expected:** Exit 0; stages include the drain-demo (work-stealing) and cooperative-handoff (ownership-gate) gates.
  **If not:** Re-run with `--json`; file a bug task.

## Verification
bash -n scripts/substrate-drain-demo.sh
bash -n scripts/substrate-smoke.sh
grep -q 'messages:' scripts/substrate-drain-demo.sh

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

### 2026-06-13T14:44:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2213-harden-substrate-arc-demos--fix-t-2211-u.md
- **Context:** Initial task creation

## Recommendation

**Recommendation:** GO (partial-complete)

**Rationale:** Two correctness/robustness wins for the flagship arc proofs, both
verified live. (1) Fixed a silent Reliability defect — substrate-drain-demo.sh
created its topic with `--retention messages --retention-value N`, a spelling the
shipped CLI rejects, so it silently fell back to retention=forever; the demo's
"bounded topic" claim was false. Now uses the working `--retention "messages:N"`
form (confirmed: a fresh topic reports retention=messages:N). (2) Wired both arc
demos into substrate-smoke.sh as regression gates — the N-way work-stealing race
(T-2211) and the CLAIM_NOT_OWNED ownership gates (T-2212), coverage the existing
happy-path smoke lacked. The single Human [REVIEW] AC (is the smoke suite the
right home vs a separate CI entry?) is a placement judgment — owner → human.

**Evidence:**
- scripts/substrate-drain-demo.sh — retention fix; fresh `--topic` reports `retention: {kind: messages, value: N}`
- scripts/substrate-smoke.sh — now 8 stages (was 6); new `drain-demo` + `handoff-demo` gates
- Full smoke run: 8/8 stages PASS, exit 0 (verified live on local hub)
- `bash -n` clean on both edited scripts
- P-011 verification gate: 3/3

## RCA

**Symptom:** `scripts/substrate-drain-demo.sh` claimed (in its header and to
operators) to use a "bounded retention-capped topic," but on the shipped CLI
the created topic was actually `retention=forever` — messages accumulated
without bound across repeated runs.

**Root cause:** The create call used `--retention messages --retention-value N`,
a two-flag spelling the CLI does not accept (`error: unexpected argument
'--retention-value'`). The script swallowed that failure (`>/dev/null 2>&1`)
and fell through to a bare `channel create "$TOPIC"`, which defaults to
`retention=forever`. The correct spelling is the single-arg `--retention
"messages:N"`.

**Why structurally allowed:** (1) The demo asserted only claim *exclusivity*,
never the *retention* of the topic it created, so the silent fallback produced
green runs. (2) The `|| fallback` pattern intentionally tolerates create
failure (for idempotency / older hubs), which also masked the wrong-flag error.
(3) The flag spelling was never exercised against the installed binary — it was
likely copied from a newer/aspirational CLI surface (the catalog documents
`--retention-value`, but this binary, 0.11.1230, predates it).

**Prevention:** (a) Fixed the spelling to `--retention "messages:N"` and
verified a fresh topic reports `retention: {kind: messages, value: N}`. (b)
Wired the demo into `substrate-smoke.sh` as a regression gate so it runs under
the standard smoke check. (c) Learning for future demo authors: assert the
*property you claim* (here, bounded retention), not just the happy-path
outcome; and never `>/dev/null 2>&1` a create whose flags you have not verified
against the actual installed binary.

### 2026-06-13T14:47:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
