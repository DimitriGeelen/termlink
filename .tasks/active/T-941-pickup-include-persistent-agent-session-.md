---
id: T-941
name: "Pickup: Include persistent agent session service templates in framework deploy scaffold (from termlink)"
description: >
  Auto-created from pickup envelope. Source: termlink, task T-940. Type: feature-proposal.

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [pickup, feature-proposal]
components: []
related_tasks: []
created: 2026-04-12T07:49:06Z
last_update: 2026-04-12T21:29:28Z
date_finished: 2026-04-12T21:29:28Z
---

# T-941: Pickup: Include persistent agent session service templates in framework deploy scaffold (from termlink)

## Problem Statement

Framework deploy scaffold lacks templates for persistent TermLink agent sessions (systemd units). T-931 proved the pattern works but the service file was hand-crafted. Standardizing templates in the framework would help new consumer projects.

DEFER: Framework-side work. The systemd unit exists (T-931) and works. Template standardization belongs in the framework repo.

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

### Agent
- [x] Problem statement validated (T-931 systemd unit works; template belongs in framework)
- [x] Assumptions tested (framework deploy scaffold is the right location)
- [x] Recommendation written with rationale (DEFER: framework-side)

### Human
- [ ] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-941, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

## Go/No-Go Criteria

**GO if:**
- Evidence supports recommendation
- No blocking dependencies

**NO-GO if:**
- Evidence supports recommendation
- No blocking dependencies

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** DEFER

**Rationale:** Framework-side work. T-931 systemd unit proves the pattern works. Template standardization belongs in the framework deploy scaffold, not the termlink consumer project.

**Evidence:**
- T-931 systemd unit works correctly (hub supervised with Restart=on-failure)
- Deploy scaffold is a framework repo concern

## Decisions

**Decision**: DEFER

**Rationale**: Recommendation: DEFER

Rationale: Framework-side work. T-931 systemd unit proves the pattern works. Template standardization belongs in the framework deploy scaffold, not the termlink consumer proj...

**Date**: 2026-04-12T17:16:11Z
## Decision

**Decision**: DEFER

**Rationale**: Recommendation: DEFER

Rationale: Framework-side work. T-931 systemd unit proves the pattern works. Template standardization belongs in the framework deploy scaffold, not the termlink consumer proj...

**Date**: 2026-04-12T17:16:11Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T17:16:11Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** DEFER
- **Rationale:** Recommendation: DEFER

Rationale: Framework-side work. T-931 systemd unit proves the pattern works. Template standardization belongs in the framework deploy scaffold, not the termlink consumer proj...

### 2026-04-12T21:29:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** DEFER decision recorded
