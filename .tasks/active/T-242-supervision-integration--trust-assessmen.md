---
id: T-242
name: "Supervision integration — trust assessment via enforcement tiers + fabric cards"
description: >
  Integrate supervision into orchestration. Trust = f(script_maturity, context_familiarity, blast_radius). Build on enforcement tiers (proven). Fabric cards as enrichment data. Failed-and-recovered scripts score higher (antifragility). See T-233 research: Q1b evidence reports.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [T-233, orchestration, supervision]
components: []
related_tasks: [T-233, T-237, T-238]
created: 2026-03-23T13:27:59Z
last_update: 2026-03-24T07:55:00Z
date_finished: null
---

# T-242: Supervision integration — trust assessment via enforcement tiers + fabric cards

## Context

Evaluate whether trust-based supervision (graduated autonomy based on script maturity, blast radius, context familiarity) is needed beyond existing enforcement tiers. Research: `docs/reports/T-233-Q1b-*.md` (tiers, fabric, healing evidence).

## Problem Statement

Should supervision be extended beyond binary enforcement tiers (block/allow) to include trust scoring and graduated autonomy for specialist agents?

## Assumptions to Validate

- A1: Enforcement tiers are insufficient for multi-agent supervision (agents need more than block/allow)
- A2: Script maturity can be meaningfully measured (run count, failure diversity)
- A3: The healing loop provides usable supervision data
- A4: Fabric cards are the right place for trust metadata
- A5: Graduated autonomy (vs. binary block/allow) provides measurable benefit

## Research Questions

### Q1: What Does Current Enforcement Already Provide?
Do Tier 0-1 hooks already supervise mesh agents? What gap remains?

### Q2: Is Trust Scoring Data Available?
Do run/fail histories exist for scripts? Is the healing loop functional enough to feed trust scores?

### Q3: What Would Supervision Change in Practice?
Concrete scenarios where trust scoring would produce different behavior than current tiers.

## Acceptance Criteria

### Agent
- [x] Research artifact created at `docs/reports/T-242-supervision-inception.md`
- [x] Each research question answered with evidence
- [x] All assumptions validated/invalidated with evidence
- [x] GO/NO-GO decision recorded with rationale
- [x] `TrustAssessment` struct in `crates/termlink-hub/src/trust.rs` with 3-axis qualitative model
- [x] Three axes: script_maturity (Hardened/Proven/Developing/Unknown), context_familiarity (High/Medium/Low/New), blast_radius (High/Medium/Low/None)
- [x] `SupervisionLevel` enum: Unsupervised, Monitored, Supervised, Blocked
- [x] Assessment function: `TrustAssessment::assess()` + `from_bypass_stats()` convenience
- [x] Integration ready: trust module callable from `handle_orchestrator_route` via bypass registry stats
- [x] Tests: 14 tests covering scoring, level determination, antifragility (hardened), denylists, familiarity thresholds

## Verification

test -f docs/reports/T-242-supervision-inception.md
grep -q "GO\|NO-GO" docs/reports/T-242-supervision-inception.md
test -f crates/termlink-hub/src/trust.rs
/Users/dimidev32/.cargo/bin/cargo test -p termlink-hub trust 2>&1 | grep "test result: ok"

## Decisions

### 2026-03-24T06:27:32Z — Original NO-GO (REVERSED)
- **Decision:** NO-GO (now overridden)
- **Original rationale:** Trust scoring inverted — workers already have max autonomy via `--dangerously-skip-permissions` (T-119). No runtime data exists. Healing loop dormant (0 invocations in 233+ tasks).
- **Research:** `docs/reports/T-242-supervision-inception.md` — Q1 found tiers completely bypassed for mesh workers, Q2 found zero trust scoring data, Q3 found supervision would ADD restrictions (not relax them)
- **Assumptions tested:** A1 (tiers insufficient) PARTIALLY VALID — tiers don't apply at all to mesh workers. A3 (healing provides data) DISPROVED — 0 invocations. A4 (fabric cards right for trust) VALID but needs runtime overlay. A2/A5 UNVALIDATABLE without data.
- **Valid finding preserved:** Build on enforcement tiers (proven: 3 real blocks, 538 commits). Use healing/fabric as enrichment only. The healing loop needs activation before it can feed trust scoring.

### 2026-03-24T07:55:00Z — Reversed to GO (human decision)
- **Chose:** GO — build supervision integration
- **Why:** T-242 is the governance layer of the T-233 approved architecture (D-008: qualitative trust supervision). The current full-bypass model (`--dangerously-skip-permissions`, T-119) is a deliberate shortcut for early development. The T-233 architecture envisions graduated autonomy where scripts earn trust via `f(script_maturity, context_familiarity, blast_radius)`. Without building the trust infrastructure, there's no path from "bypass everything" to "supervised execution." The research correctly identified that the healing loop is dormant — the build should focus on tiers + fabric (proven) with healing as future enrichment.
- **Rejected:** Original NO-GO — treated the current bypass-everything model as the desired end state rather than a temporary shortcut. Evaluated T-242 as isolated feature rather than building block of approved architecture (T-258 root cause analysis).

## Updates

### 2026-03-23T13:27:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-242-supervision-integration--trust-assessmen.md
- **Context:** Initial task creation

### 2026-03-24T06:24:19Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-24T06:27:32Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** NO-GO
- **Rationale:** Trust scoring inverted: workers already have max autonomy. No runtime data, healing loop unused, no persistent specialists.

### 2026-03-24T06:27:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: NO-GO

### 2026-03-24T07:55:00Z — reopened [human decision]
- **Action:** NO-GO reversed to GO by human
- **Reason:** T-242 is the governance layer of T-233 architecture (D-008). Current bypass-everything is a shortcut, not the end state. Trust scoring infrastructure is needed to enable graduated autonomy.
- **Context:** T-258 context amnesia investigation revealed NO-GO was based on missing architectural context
