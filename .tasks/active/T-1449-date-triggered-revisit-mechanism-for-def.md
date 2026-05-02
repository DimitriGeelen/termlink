---
id: T-1449
name: "Date-triggered revisit mechanism for DEFER inceptions and sentinel audits (G-053)"
description: >
  Inception: design how the framework structurally captures and surfaces 'when do we revisit this DEFER?' Currently sentinel tasks (T-1428 for T-1425) carry the date in description prose; nothing scans for it; if 2026-05-14 passes silently the deadline goes unnoticed. Five concrete deliverables on the table — need design choices before build: (1) revisit_at frontmatter field shape, (2) cron job that surfaces ripe revisits, (3) fw task revisit-due CLI, (4) backfill T-1428's ACs from prose recipe, (5) optional deferred-decisions.yaml register. Inception scope = decide the shape; build tasks land downstream.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [framework, governance, deferred-decisions, sentinel, G-053]
components: []
related_tasks: [T-1425, T-1428, T-1448]
created: 2026-05-02T21:47:02Z
last_update: 2026-05-02T21:56:30Z
date_finished: null
---

# T-1449: Date-triggered revisit mechanism for DEFER inceptions and sentinel audits (G-053)

## Problem Statement

When an inception decides DEFER (or any task names a future revisit date), the date lives only in description prose. Nothing scans for it. If the date passes and nobody opens Watchtower or runs `fw task list`, the deadline goes silently — the framework cannot proactively surface "this DEFER is ripe for re-verdict."

Concrete instance: **T-1425** decided DEFER on 2026-04-30, named **T-1428** as the sentinel firing 2026-05-14. T-1428 was created with empty placeholder ACs (`[First criterion]`, `[Second criterion]`); the audit recipe lives only in the description prose. If 2026-05-14 arrives and no agent picks up T-1428, the verdict on T-1425 stays DEFER indefinitely.

The same shape applies to: amend windows on solo syntheses (T-1425's 14d window), foundation-soak audits (T-1428), G-053 itself once a fix lands (when do we audit that the fix worked?), and any "revisit in 2 weeks" pattern. The framework has the *capture* mechanism (sentinel tasks) but not the *surface* mechanism.

For whom: agents picking up sessions days/weeks after a decision was deferred — they shouldn't have to grep description prose to find ripe revisits. Why now: T-1428 fires 2026-05-14 (12 days out) and the framework currently has no way to make sure that fires.

## Assumptions

A-1: Task frontmatter is the right home for a `revisit_at` field. (Test: alternative is a separate `deferred-decisions.yaml` register; check whether per-task field captures the relationship without requiring a join.)

A-2: A single daily cron scan is sufficient surfacing — operators check Watchtower or read handovers daily. (Test: false if work happens in multi-day batches without daily check-in; in that case the surfacing must be heartbeat- or session-start-driven, not cron-driven.)

A-3: The set of revisit triggers fits one date. (Test: false if revisits need multiple criteria — "after T-X ships AND 14 days have passed" — in which case we need expression evaluation, not just a date compare.)

A-4: Backfilling T-1428's ACs from prose is mechanical — the recipe is recoverable. (Test: read T-1428 description + T-1425 Recommendation; confirm the audit steps are extractable into checkboxes.)

## Exploration Plan

Spike 1 — *Survey of existing revisit-pattern instances* (~30 min): grep `.tasks/active/` and `.context/project/` for "revisit", "sentinel", "amend window", "fires 2026-", "2-week check", "in N days". Count instances. Decide whether the field belongs on tasks, concerns, or both.

Spike 2 — *Frontmatter shape decision* (~30 min): try three shapes on T-1428 as testbed:
- (a) `revisit_at: 2026-05-14`
- (b) `revisit_at: 2026-05-14, revisit_evidence_needed: "T-1426 + T-1427 ship status"`
- (c) `revisit: { at: 2026-05-14, evidence_needed: "...", source: T-1425 }`
Pick the simplest that supports the surveyed instances.

Spike 3 — *Surfacing mechanism* (~45 min): pick between (i) daily cron writes a "ripe revisits" file consumed by handover banner + Watchtower home, (ii) on-demand `fw task revisit-due` CLI only, (iii) both. Wire the chosen mechanism end-to-end on T-1428 as testbed.

Spike 4 — *T-1428 AC backfill* (~30 min): regardless of mechanism choice, fill T-1428's empty ACs from the prose recipe so the 2026-05-14 audit is mechanically checkable.

No fifth spike. The optional `deferred-decisions.yaml` register is deferred to a follow-up if Spike 1 reveals revisit patterns that span multiple tasks (e.g., a single decision with multiple revisit dates).

## Technical Constraints

- Frontmatter changes must be backward-compatible: existing tasks have no `revisit_at` field and continue working. The field is opt-in.
- Cron job must be cheap (reads ~50 task files); fits in existing `.context/cron/` registry alongside the 11 active jobs.
- Surfacing must work without Watchtower running — handover banner is the primary channel, Watchtower /home is the secondary.
- T-1428's audit fire date is 2026-05-14 — whatever lands must be operational before then, or T-1428 itself becomes the first failure of the mechanism it's meant to validate.

## Scope Fence

**IN scope:**
- `revisit_at` frontmatter field on tasks
- Daily cron + handover banner + Watchtower surfacing of ripe revisits
- `fw task revisit-due` CLI
- T-1428 AC backfill (regardless of mechanism choice)

**OUT of scope:**
- Multi-criterion revisit triggers (e.g. "after T-X ships AND 14d") — punt to follow-up if Spike 3 reveals demand
- Revisit-on-event triggers (file modified, hub restarted) — different mechanism entirely
- Deferred-decisions.yaml register — punt unless Spike 1 reveals cross-task patterns
- Watchtower UI rework — this should fit existing /home banner / /attention if it exists; no new routes

## Acceptance Criteria

### Agent
- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Recommendation written with rationale

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- Spike 1 finds ≥3 revisit-pattern instances in active artifacts (problem is real, not just T-1425/T-1428)
- A single `revisit_at` field shape (Spike 2 winner) supports all surveyed instances without expression evaluation
- T-1428 AC backfill (Spike 4) is mechanically extractable — the recipe is recoverable from prose

**NO-GO if:**
- Spike 1 finds only 1-2 instances — gap is too narrow to justify framework change; tactical fix on T-1428 alone (manual ACs + calendar reminder) is sufficient
- Spike 2 reveals revisit triggers need expression evaluation (multi-criterion, event-driven) — that's a different problem (T-XXXX-conditional-revisit) and this inception's scope is wrong

**DEFER if:**
- T-1449 design lands but T-1428's 2026-05-14 audit needs to actually fire to validate the surfacing mechanism worked end-to-end. After T-1428 fires successfully, GO this inception's broader rollout. (This makes T-1428 itself the soak test for the mechanism.)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation: GO — Phase 1 (frontmatter + daily cron + CLI + T-1428 backfill); defer separate register.**

**Rationale:**

The gap is real and bounded. T-1425's DEFER decision named T-1428 as a sentinel firing 2026-05-14, but T-1428 has empty placeholder ACs and no surfacing mechanism — if the date passes silently, the verdict on T-1425 stays DEFER indefinitely. This is the framework's own structure failing to honor its own decision protocol. The fix is small (one frontmatter field, one cron, one CLI verb, ~50 LOC) and the alternative (manual calendar reminders) doesn't scale beyond a single instance.

**Evidence (pre-spike, refined post-Spike 1):**

- T-1428 created 2026-04-30, fires 2026-05-14, ACs empty (`[First criterion]`, `[Second criterion]`) — no operational recipe captured
- T-1425's "amend window" (14d from 2026-04-30T21:18Z) ends 2026-05-14 — no surfacing mechanism
- 11 active framework crons exist; none are date-triggered per-task. The infrastructure for cron-based surfacing is in place (`.context/cron/`)
- Existing primitives (`horizon: now/next/later`) lack date semantics — adding `revisit_at` is additive, no migration burden

**Phasing:**

- Phase 1 (this inception's downstream builds): `revisit_at` field, daily cron, `fw task revisit-due`, T-1428 ACs backfilled. Target: operational before 2026-05-14 so T-1428 itself surfaces correctly.
- Phase 2 (deferred): `deferred-decisions.yaml` register, multi-criterion expressions, event-driven triggers. Defer until Phase 1 reveals demand.

**Why GO not DEFER:** the DEFER criterion ("T-1428 fires successfully first") creates a chicken-and-egg — the mechanism that needs to surface T-1428 has to land *before* T-1428 fires. Going Phase 1 now and using T-1428 as the soak test inverts that correctly.

**Why GO not NO-GO:** Spike 1 is necessary to confirm ≥3 instances. Pre-spike inspection already turns up T-1425, T-1428, T-1448 (auto-finalize bug), G-052, G-053 itself (when fixed, when do we audit?), and the 14d amend windows on solo syntheses — that's already 5-6 instances. NO-GO is unlikely.

**Downstream build tasks (provisional, scope post-Spike completion):**

| # | Deliverable | Estimated size |
|---|---|---|
| 1 | `revisit_at` frontmatter field + template update | ~30 LOC |
| 2 | Daily cron `revisit-due-scan.sh` + handover banner integration | ~50 LOC |
| 3 | `fw task revisit-due` CLI verb | ~40 LOC |
| 4 | T-1428 AC backfill (no code) | ~10 lines of YAML/markdown |
| 5 | (deferred to Phase 2) `deferred-decisions.yaml` register | n/a this phase |

Total Phase 1: ~120 LOC + 1 task-file edit. Single session per build task.

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

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-05-02T21:56:30Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
