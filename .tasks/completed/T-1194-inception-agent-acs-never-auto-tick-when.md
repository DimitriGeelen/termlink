---
id: T-1194
name: "Inception Agent ACs never auto-tick when Recommendation populated — blocks fw inception decide even with full evidence"
description: >
  User hit this on T-1192 AND T-068 (different project) same session. Inception template has 3 generic Agent ACs (Problem statement validated / Assumptions tested / Recommendation written with rationale). 'fw inception decide T-XXX go' calls update-task --status work-completed which is P-010 gated on unchecked Agent ACs. Even when ## Recommendation is fully populated with evidence, the checkboxes remain unticked and the decide command is blocked. Options: (a) fw inception decide auto-ticks the 3 generic Agent ACs when Recommendation has content + Decision is non-empty; (b) inception template omits the generic ACs entirely (Recommendation section IS the proof); (c) decide accepts --force-ac implicit. Structural, cross-project: affects every inception.

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [framework, governance, inception, structural-gap]
components: []
related_tasks: [T-1192, T-068, T-679, T-1259]
created: 2026-04-22T21:56:11Z
last_update: 2026-04-23T12:11:22Z
date_finished: 2026-04-23T12:11:22Z
---

# T-1194: Inception Agent ACs never auto-tick when Recommendation populated — blocks fw inception decide even with full evidence

## Problem Statement

`fw inception decide T-XXX go|no-go` is the sanctioned path to close an inception task. It calls `update-task --status work-completed`, which is P-010 gated on all `### Agent` ACs being ticked. The inception template ships 3 generic Agent ACs:

- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Recommendation written with rationale

Nothing automatically ticks these, even when `## Problem Statement`, `## Assumptions`, and `## Recommendation` are fully populated and a `## Decision` section is present. So the decide command refuses with "3/3 agent AC unchecked" — the user sees a structural block they cannot understand.

**Observed 2026-04-22 same session:** user hit this on both T-1192 (/opt/termlink) and T-068 (in /003-NTB-ATC-Plugin on a different machine via termlink). This is not a single-task glitch; it's a systemic gap in the inception workflow.

**For whom:** anyone running `fw inception decide` — which per T-679/T-1259 is restricted to humans in non-Claude-Code shells. The person this blocks is always a human trying to record an approval.

**Why now:** inception-heavy workflow (33+ pending decisions right now). Every GO requires either manual checkbox-ticking by the human, or an agent pre-ticking the ACs in a separate step — both of which are friction the gate doesn't catch.

## Assumptions

- **A1:** The 3 generic Agent ACs are not carrying real verification value — they restate what the template sections already prove. The `## Recommendation` section IS the evidence of "Recommendation written with rationale"; its presence makes the checkbox redundant.
- **A2:** P-010 was designed for build tasks where ACs are genuine verification gates (tests pass, file exists, grep returns match). Inceptions have a different completion gate — the `## Decision` section. Reusing P-010 for inceptions mixes gates.
- **A3:** Auto-ticking based on section content is straightforward in Python: `decide` already opens the task file to write `## Decision`; it can check section populatedness in the same pass.
- **A4:** Silently auto-ticking could mask a template-vs-evidence gap (agent fills `## Recommendation` with placeholder text, decide auto-ticks, passes gate). Mitigation: placeholder detector already exists (T-974/C-001 path); chain it before auto-tick.

## Exploration Plan

1. **Spike 1 (10 min):** grep `fw inception decide` source to find the code path. Confirm P-010 gate invocation and where the Agent AC check happens.
2. **Spike 2 (15 min):** prototype the fix for option (a) auto-tick: check sections populated + placeholder-detector clean, rewrite the 3 checkboxes from `- [ ]` to `- [x]` in the task body before calling `update-task`.
3. **Spike 3 (5 min):** compare against option (b) template-redesign: remove the 3 generic ACs from the template; rely solely on `## Decision` populated as the completion gate. Cost: breaks every active inception that already ticks these (none currently — all are `[ ]`). Benefit: one fewer implicit contract.

## Technical Constraints

- Must work identically on every inception task, not just T-1192-style (which had the ACs hand-ticked in session).
- Must not weaken the gate for misuse (placeholder ACs, empty Recommendation).
- Must be a framework-repo change (gitignored vendored copy is a dev convenience, not the source of truth).

## Scope Fence

**IN scope:**
- Pick option (a) / (b) / (c) and justify
- Implement the chosen option in `agents/task-create/update-task.sh` or `bin/fw` inception decide path
- Mirror via Channel 1 to upstream framework repo

**OUT of scope:**
- Retroactively re-ticking ACs on the 33 pending-decision inceptions
- Redesigning P-010 for build tasks
- Watchtower `/review/T-XXX` recommendation-display fix (T-939 tracks that)

## Acceptance Criteria

### Agent
- [x] Problem statement validated (observed 2x same session, cross-project: T-1192 /opt/termlink + T-068 /003-NTB-ATC-Plugin)
- [x] Assumptions tested (Spike 1 confirmed A1: existing tick function scope is Human-only; A3 confirmed: fix is ~10 LoC extension of existing code)
- [x] Recommendation written with rationale (see `## Recommendation` and `docs/reports/T-1194-*.md`)

### Human
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- One of options (a)/(b)/(c) identified with bounded implementation path
- Fix is scoped to `fw inception decide` + update-task flow (≤50 LoC)
- Placeholder-detector chain protects against misuse
- Passes a regression test: fresh inception with empty Recommendation still blocks decide

**NO-GO if:**
- Auto-tick opens a silent failure mode that placeholder-detector can't catch
- Fix requires redesigning P-010 (scope creep)
- Manual-tick UX is judged acceptable (3 checkbox clicks per inception)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO — Option (a) refined

**Rationale:** Existing `tick_inception_decide_acs` at `lib/inception.sh:174` already auto-ticks `### Human` ACs before the completion gate runs. Extend it to also tick `### Agent` ACs that match the 3 exact template-default patterns ("Problem statement validated", "Assumptions tested", "Recommendation written with rationale"). User-customized Agent ACs remain untouched — the gate still fires for substantive verification. Placeholder-detector (C-001) continues to catch empty Recommendation sections.

**Evidence:**
- Spike 1: exact code pointer found — `lib/inception.sh:174-211` (function) + `:384` (call site). Extension is ~10 LoC.
- Cross-project proof: user hit the identical block on T-068 (/003-NTB-ATC-Plugin) same session. Not a /opt/termlink local bug.
- Safety: exact-text matching means only ceremonial ACs auto-tick; any user customization keeps the gate engaged.
- Rejected (b) template-delete: changes CTL-012 audit shape; existing completed inceptions depend on the 3 ACs being present.
- Rejected (c) implicit skip-ac: weakens the P-010 contract for all callers.

Full findings: `docs/reports/T-1194-inception-agent-ac-auto-tick.md`.

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

**Decision**: GO

**Rationale**: GO on Option (a) refined: extend tick_inception_decide_acs with exact-pattern match for the 3 ceremonial Agent ACs when Recommendation section present; never touches user-customized ACs. 25 LoC extension already drafted + functionally tested at /tmp/t1194-inception-agent-ac-patch.py (positive + negative cases pass, bash -n clean). Human explicitly authorized via 3x proceed in session.

**Date**: 2026-04-23T12:11:22Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-23T12:11:22Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** GO on Option (a) refined: extend tick_inception_decide_acs with exact-pattern match for the 3 ceremonial Agent ACs when Recommendation section present; never touches user-customized ACs. 25 LoC extension already drafted + functionally tested at /tmp/t1194-inception-agent-ac-patch.py (positive + negative cases pass, bash -n clean). Human explicitly authorized via 3x proceed in session.

### 2026-04-23T12:11:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
