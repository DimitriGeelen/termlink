---
id: T-957
name: "Pickup: Cross-host agent access requires three things aligned: hub + named sessions + shared runtime dir (from termlink)"
description: >
  Auto-created from pickup envelope. Source: termlink, task T-940. Type: learning.

status: work-completed
workflow_type: inception
owner: human
horizon: later
tags: [pickup, learning]
components: []
related_tasks: []
created: 2026-04-12T08:41:02Z
last_update: 2026-04-18T15:05:05Z
date_finished: 2026-04-12T15:59:22Z
---

# T-957: Pickup: Cross-host agent access requires three things aligned: hub + named sessions + shared runtime dir (from termlink)

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

### Agent
- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Recommendation written with rationale

### Human
- [ ] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-957, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

## Go/No-Go Criteria

**GO if:**
- Hub + named persistent sessions + shared runtime_dir alignment is verifiable via `fw fleet doctor`
- Structural work (T-940 runtime-dir unification, T-942 multi-dir scan, T-941 persistent session templates) absorbs the learning

**NO-GO if:**
- Cross-host access requires a fundamentally different model (e.g., centralized session registry)
- The three-part alignment proves fragile in production and needs a redesign

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO
**Rationale:** Requirement captured as a learning and fed into multiple structural fixes rather than a single build: T-940 (runtime-dir unification), T-942 (multi-dir hub scanning), T-941 (persistent agent session templates). No standalone build task needed — absorbed across subsystem work.
**Evidence:**
- T-940 / T-942 / T-941 all trace back to this learning
- Named sessions + hub + shared runtime_dir alignment validated end-to-end by `fw fleet doctor`
- Learning stored in `.context/project/learnings.yaml`

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

**Rationale**: Recommendation: GO
Rationale: Requirement captured as a learning and fed into multiple structural fixes rather than a single build: T-940 (runtime-dir unification), T-942 (multi-dir hub scanning), T-941 (persistent agent session templates). No standalone build task needed — absorbed across subsystem work.
Evidence:
- T-940 / T-942 / T-941 all trace back to this learning
- Named sessions + hub + shared runtime_dir alignment validated end-to-end by `fw fleet doctor`
- Learning stored in `.context/project/learnings.yaml`

**Date**: 2026-04-18T15:05:05Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T15:59:22Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-12T15:59:22Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Learning captured, no build work needed

### 2026-04-16T05:39:44Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-16T21:08:45Z — programmatic-evidence [T-1090]
- **Evidence:** Hub secret, TOFU trust, and session discovery all confirmed working: hub.secret in /tmp/termlink-0/, tofu list shows 4 entries, discover shows 8 sessions
- **Verified by:** automated command execution

### 2026-04-18T15:05:05Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO
Rationale: Requirement captured as a learning and fed into multiple structural fixes rather than a single build: T-940 (runtime-dir unification), T-942 (multi-dir hub scanning), T-941 (persistent agent session templates). No standalone build task needed — absorbed across subsystem work.
Evidence:
- T-940 / T-942 / T-941 all trace back to this learning
- Named sessions + hub + shared runtime_dir alignment validated end-to-end by `fw fleet doctor`
- Learning stored in `.context/project/learnings.yaml`
