---
id: T-3612
name: "Re-derive TermLink purpose, intent and direction from the bleeding edge (5
  external seats, synthesis, operator dialogue, sharpened single purpose/objectives/drivers)"
description: >
  Inception: Re-derive TermLink purpose, intent and direction from the bleeding edge
  (5 external seats, synthesis, operator dialogue, sharpened single purpose/objectives/drivers)

status: work-completed
workflow_type: inception
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-10-27T21:26:18Z
last_update: 2026-11-01T16:03:43Z
date_finished: 2026-11-01T16:03:43Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.9                    # operator 2026-10-27 (voi-prompt ledger). float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
                                  # T-3172: deliberately EMPTY — set a considered value. 0.5 (and 0.3/0.4) score 2, the same as "not assessed".
bvp_scores_proposed:
  - ts: '2026-10-27T21:27:37Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 2
      F-ORCH: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal); F-RECALL=2 (no-signal); F-ORCH=2 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3612: Re-derive TermLink purpose, intent and direction from the bleeding edge (5 external seats, synthesis, operator dialogue, sharpened single purpose/objectives/drivers)

## Problem Statement

TermLink's stated purpose has drifted apart in three places: the charter's purpose sentence ("durable append-log
message bus with terminal endpoints", never blessed, T-2470), the operator-revised objectives of 2026-10-21
(`.context/project/objectives.yaml`: "trusted coordination substrate … dispatch and hand back work"), and value
drivers that are AEF-generic and name none of TermLink's own outcomes. The 2026-10-27 value-review run (T-3609)
exposed it: its yardstick could not be confirmed. The operator asked (2026-10-27) for five independent external
reads of the bleeding-edge repo (code, arcs, task discussions and design documents of the last two months) to
derive what TermLink is and what it is evolving into; a synthesis of commonality and divergence; a dialogue round
with the operator; and a sharpening round into one purpose with objectives and value drivers. Research artifact:
`docs/reports/T-3612-purpose-rederivation.md`.

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

## Open Questions

<!-- T-2190 (T-2186 Slice 4): every IW-N question must be disposed before
     --status work-completed. Disposition gate (agents/task-create/update-task.sh
     check_disposition_gate) refuses on under-disposed inceptions.

     Per-question shape:

       - **IW-1: <question text>**
         confidence: 0-3      (your confidence in your current answer; 0=guess, 3=verified)
         disposition: answered | deferred | dissolved
         rationale: <one-line evidence — file:line, decision id, dialogue ref>

     Never bare yes/no — the gate refuses bare checkboxes. See 050-Inceptions.md
     §Disposition Gate. Bypass: --skip-disposition-gate "rationale" (direct) or
     FW_SKIP_DISPOSITION_GATE=1 (env-var, T-1890 producer/consumer parity).
-->

- **IW-1: What is TermLink for, and what is it becoming, as the bleeding-edge repo and the last two months of arcs, tasks and design documents show it (not as the charter says)?**
  confidence: 0.8
  disposition: answered
  rationale: round 1, five independent external seats read the repo (docs/reports/T-3612-seats/round1-*.md); answered by the operator's Q1-Q7 (docs/reports/T-3612-purpose-rederivation.md Dialogue Log 5-12): a delivery substrate that makes sure a message reaches an agent that can act, not an orchestrator or a bus
- **IW-2: Where do the five reads agree and diverge, and which divergences are real choices for the operator?**
  confidence: 0.8
  disposition: answered
  rationale: docs/reports/T-3612-synthesis-round2.md (agreement and divergence); each real choice put to the operator one at a time and ruled (Dialogue Log 5-20; T-3624 Decisions)
- **IW-3: What single purpose, objectives and value drivers follow, and do they survive a second external review?**
  confidence: 0.8
  disposition: answered
  rationale: draft v0.9 (docs/reports/T-3612-round4-draft.md): purpose ruling E, P-1..P-7, non-goals, two driver sets; survived two outside review rounds (T-3612-round4-review-synthesis.md, T-3612-round5-review-synthesis.md: four seats, all "adopt with the listed changes", every finding dispositioned in v0.9); adoption is the operator's (sovereign)

## Exploration Plan

<!-- How will we validate assumptions? Spikes, prototypes, research? Time-box each. -->

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

<!-- What's IN scope for this exploration? What's explicitly OUT? -->

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [x] Problem statement validated
<!-- @auto-tick-on-decide -->
- [x] Assumptions tested
<!-- @auto-tick-on-decide -->
- [x] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** GO

**Rationale:**

Adopt draft v0.9 (`docs/reports/T-3612-round4-draft.md`). Shaping is complete: five independent reads, a synthesis,
seven operator questions (Q1-Q7) and eight further rulings on the draft's open points, and two outside review rounds
(the cap) whose every finding is accepted, rejected or deferred in v0.9. GO means: the operator blesses the charter
sentence (`docs/CHARTER.md:17-19`), and a build task applies draft section 6 in one commit (charter, README,
architecture doc, charter-sentence check and its tests, objectives.yaml, driver files, tripwires) with the release
gate green. The product driver set stays declarative until AEF T-4961 (operator ruling A, 2026-11-01).

**Evidence:**

- Round 1 reads: `docs/reports/T-3612-seats/round1-*.md`; synthesis `docs/reports/T-3612-synthesis-round2.md`.
- Operator rulings: `docs/reports/T-3612-purpose-rederivation.md` Dialogue Log items 5-20; T-3624 Decisions.
- Outside review round 1 (v0.3): `docs/reports/T-3612-round4-review-synthesis.md`.
- Outside review round 2 (v0.8, final): `docs/reports/T-3612-round5-review-synthesis.md` — four seats, all "adopt
  with the listed changes"; three blocking findings fixed in v0.9 (one by operator ruling A on the product drivers).
- Deferred with stated risk: F-SURFACE's overlap with D3/D2, to the first quarterly driver review.

<!-- Add evidence bullets as exploration progresses (file paths,
     commit hashes, test results). The filing-time recommendation
     can be revised before fw inception decide. -->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

### 2026-10-28 — Q7: is governance and operations machinery part of TermLink? D′ two layers (operator ruling)
- **Chose:** D′. The repo holds two layers, as AEF holds the framework plus a vendored instance of itself:
  (1a) TermLink the product, which is what a release ships (`cargo build -p termlink`, release.yml:169; plus install
  script and Homebrew formula); (1b) the TermLink development project, run as an AEF consumer (tasks, runme.sh, the
  98 check-*.sh guards, 46 cron files, review ledgers, and the operation of our own fleet). Governance machinery is
  part of 1b, not of the product, and rightly lives in this repo. Concretely:
  6a AEF's chores leave runme.sh for AEF's `fw runme` (actions 11, 11b, 11c, 13, 16) and ring20's for ring20
  (action 22); 6b governance guards stay as dev-project tooling, generic ones offered upstream only where AEF lacks
  them; 6c a cap on dev-project canaries (one in, one out); 6d label the layers: objectives and value drivers split
  into product and project sets, README/charter say which layer they describe, external briefs say "the product is
  crates/ and the release; the rest is how we build it".
- **Why:** operator, 2026-10-28: "differentiate between the TermLink development project … and the TermLink
  application … just as AEF does it, we also eat our own dog food". The round-1 reviewers read the repo as one thing
  and counted dev-project guards as product surface; objectives.yaml and policy/value-drivers.yaml mix both layers,
  so D1 scores a new dev-project canary as product progress.
- **Rejected:** A move everything out (takes product-proving canaries away from their code, cross-repo cron
  migration risk); B cap only (leaves AEF chores going stale here); C lower D1 only (moves nothing); D as first
  briefed (treated governance guards as misplaced, which the two-layer view dissolves).
- **Left open:** the cap's number (6c) and D1's weight (option C) fold into round 4's driver work; running our own
  fleet is counted as 1b, and the ring20-side part moves to ring20 if they come to own their hosts. Round 4 writes the
  single purpose, objectives and drivers for 1a, plus a short separate 1b set.

### 2026-10-28 — round 4: draft now, operator reads first, then outside review (operator ruling C)
- **Chose:** C. Draft from Q1–Q7, operator reads and corrects it, then at most 2 rounds of outside review by the
  subscription seats (claude, codex, antigravity, opencode). Q6 stays marked open (pending T-3615 data).
  Draft v0.1: docs/reports/T-3612-round4-draft.md.
- **Why:** the operator's answers are the draft's substance; review then tests a draft the operator stands behind.
- **Rejected:** A (review before the operator reads: may spend a round on wording the operator rejects); B (wait for
  telemetry: weeks on a known-wrong yardstick); D (stop: leaves the mixed-layer yardstick, against Q7).

### 2026-10-28 — the purpose sentence: E (operator ruling)
- **Chose:** E. "TermLink makes sure every message reaches the recipient it is meant for: it finds that recipient,
  wakes it in its terminal when it must, hands the message over, and shows where it is until an agent that can act
  has it, or it is loudly stuck." Scope sentence after it: "It is the base layer: it records work handed off between
  agents, and leaves what work to do, and who should do it, to the framework above."
- **Why:** operator: the essence is reaching the recipient, and more than brokerage: the recipient is found,
  addressed and activated. Handed-off work is message content, so naming it in the sentence reads as orchestration.
  "Meant for" means the addressed recipient or role holder, never the best one for the job. "Or loudly stuck" answers
  the reviewers: TermLink cannot force an agent to act, but makes every stop visible.
- **Rejected:** A, A-short, B, C, D (dialogue log item 15; review synthesis C.14).
- **Left open:** adoption of the whole draft (charter, its guard and copies change only then).

## Decision

**Decision**: GO

**Rationale**: Adopt draft v0.9 (`docs/reports/T-3612-round4-draft.md`). Shaping is complete: five independent reads, a synthesis,
seven operator questions (Q1-Q7) and eight further rulings on the draft's open points, and two outside review rounds
(the cap) whose every finding is accepted, rejected or deferred in v0.9. GO means: the operator blesses the charter
sentence (`docs/CHARTER.md:17-19`), and a build task applies draft section 6 in one commit (charter, README,
architecture doc, charter-sentence check and its tests, objectives.yaml, driver files, tripwires) with the release
gate green. The product driver set stays declarative until AEF T-4961 (operator ruling A, 2026-11-01).

**Date**: 2026-11-01T16:03:42Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-10-27T21:27:36Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-11-01T16:03:42Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Adopt draft v0.9 (`docs/reports/T-3612-round4-draft.md`). Shaping is complete: five independent reads, a synthesis,
seven operator questions (Q1-Q7) and eight further rulings on the draft's open points, and two outside review rounds
(the cap) whose every finding is accepted, rejected or deferred in v0.9. GO means: the operator blesses the charter
sentence (`docs/CHARTER.md:17-19`), and a build task applies draft section 6 in one commit (charter, README,
architecture doc, charter-sentence check and its tests, objectives.yaml, driver files, tripwires) with the release
gate green. The product driver set stays declarative until AEF T-4961 (operator ruling A, 2026-11-01).

### 2026-11-01T16:03:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO

## Reviewer Verdict (v1.5)

- **Scan ID:** R-1c011695
- **Timestamp:** 2026-11-01T16:03:44Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Scope:** static anti-pattern scan; PASS means no anti-pattern was found, not that the criteria hold
- **Needs Human:** no
- **Findings:** none

## Recommendation Verdict (v1.0)

- **Scan ID:** RC-bfb4f8f3
- **Timestamp:** 2026-11-01T16:03:44Z
- **Overall:** CONTRADICTED
- **Claims:** 8

| Claim | Type | Status |
|-------|------|--------|
| `docs/reports/T-3612-round4-draft.md` | file | ✓ pass |
| `docs/CHARTER.md:17-19` | file | ✗ fail — file not found at PROJECT_ROOT |
| `docs/reports/T-3612-synthesis-round2.md` | file | ✓ pass |
| `docs/reports/T-3612-purpose-rederivation.md` | file | ✓ pass |
| `docs/reports/T-3612-round4-review-synthesis.md` | file | ✓ pass |
| `docs/reports/T-3612-round5-review-synthesis.md` | file | ✓ pass |
| `T-4961` | task | ✗ fail — no task file in .tasks/{active,completed}/ |
| `T-3624` | task | ✓ pass |
