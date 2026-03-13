---
id: T-123
name: "Agent mesh concurrent builds — isolation strategy for parallel write tasks"
description: >
  Inception: Agent mesh concurrent builds — isolation strategy for parallel write tasks

status: work-completed
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-12T20:40:23Z
last_update: 2026-03-12T20:56:55Z
date_finished: 2026-03-12T20:56:55Z
---

# T-123: Agent mesh concurrent builds — isolation strategy for parallel write tasks

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

### 2026-03-12 — Isolation strategy for parallel build tasks
- **Chose:** Git worktree per worker + CARGO_TARGET_DIR per worktree
- **Why:** Structural filesystem isolation eliminates all file conflict, build contention, and git corruption risks. Worktrees are lightweight (~1-2MB, shared .git/objects). ~20 lines of bash in dispatch.sh. Cold build penalty (30-60s) amortized by parallel wall-clock savings.
- **Rejected:** File partitioning (breaks on overlapping files — T-120/T-121/T-122 share server.rs), CARGO_TARGET_DIR only (half-measure, no source isolation), CoW copies (overkill, painful merge), layered patches (impractical for AI agents)

## Updates

### 2026-03-12T20:56:13Z — inception-decision [inception-workflow]
- **Action:** GO decision recorded
- **Decision:** GO — worktree isolation
- **Rationale:** See docs/reports/T-123-mesh-concurrent-builds.md for full analysis (5 strategies, risk matrix, file overlap analysis)

### 2026-03-12T20:56:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
