---
id: T-003
name: "IT-001: Paradigm decision — injection vs message bus vs hybrid"
description: >
  IT-001: Paradigm decision — injection vs message bus vs hybrid

status: work-completed
workflow_type: inception
owner: agent
horizon:
tags: []
components: []
related_tasks: []
created: 2026-03-08T14:10:53Z
last_update: '2026-08-18T18:58:40Z'
date_finished: 2026-03-08T14:27:12Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:38Z'
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
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:40Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 4
      effort: 4
    rationale: blast_radius=0 (no-signal); tier=4 (no-signal); effort=4 
      (no-signal)
    rubric_sha: missing
---

# T-003: IT-001: Paradigm decision — injection vs message bus vs hybrid

## Problem Statement

<!-- What problem are we exploring? For whom? Why now? -->

## Assumptions

<!-- Key assumptions to test. Register with: fw assumption add "Statement" --task T-XXX -->

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

- [x] Problem statement validated
- [x] Assumptions tested
- [x] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- [Criterion 1]
- [Criterion 2]

**NO-GO if:**
- [Criterion 1]
- [Criterion 2]

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
-->

## Decisions

**Decision**: GO

**Rationale**: Three independent research streams converge: 75% of use cases need messaging, prior art (kitty/Zellij/Wezterm) converges on control/data plane separation, MCP maps well as control plane. Paradigm: message bus with injection adapter.

**Date**: 2026-03-08T14:27:12Z
## Decision

**Decision**: GO

**Rationale**: Three independent research streams converge: 75% of use cases need messaging, prior art (kitty/Zellij/Wezterm) converges on control/data plane separation, MCP maps well as control plane. Paradigm: message bus with injection adapter.

**Date**: 2026-03-08T14:27:12Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-08T14:26:58Z — status-update [task-update-agent]
- **Change:** owner: agent → agent

### 2026-03-08T14:27:12Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Three independent research streams converge: 75% of use cases need messaging, prior art (kitty/Zellij/Wezterm) converges on control/data plane separation, MCP maps well as control plane. Paradigm: message bus with injection adapter.

### 2026-03-08T14:27:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
