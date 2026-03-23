---
id: T-245
name: "Interactive session picker for attach/mirror/stream — list and select when no target given"
description: >
  Inception: Interactive session picker for attach/mirror/stream — list and select when no target given

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-23T15:40:22Z
last_update: 2026-03-23T22:00:27Z
date_finished: 2026-03-23T22:00:27Z
---

# T-245: Interactive session picker for attach/mirror/stream — list and select when no target given

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

- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Go/No-Go decision made

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

**Rationale**: Straightforward UX improvement. When target-requiring commands run without a target and stdin is TTY: list sessions (numbered), auto-select if 1, prompt if 2+. Applies to ~15 interactive commands (attach, mirror, stream, ping, status, watch, topics, output, interact, inject, kv, events, wait, remote ping, remote status). Shared utility function. Works for local and remote (--hub) sessions.

**Date**: 2026-03-23T15:54:48Z
## Decision

**Decision**: GO

**Rationale**: Straightforward UX improvement. When target-requiring commands run without a target and stdin is TTY: list sessions (numbered), auto-select if 1, prompt if 2+. Applies to ~15 interactive commands (attach, mirror, stream, ping, status, watch, topics, output, interact, inject, kv, events, wait, remote ping, remote status). Shared utility function. Works for local and remote (--hub) sessions.

**Date**: 2026-03-23T15:54:48Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-23T15:54:48Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Straightforward UX improvement. When target-requiring commands run without a target and stdin is TTY: list sessions (numbered), auto-select if 1, prompt if 2+. Applies to ~15 interactive commands (attach, mirror, stream, ping, status, watch, topics, output, interact, inject, kv, events, wait, remote ping, remote status). Shared utility function. Works for local and remote (--hub) sessions.

### 2026-03-23T22:00:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
