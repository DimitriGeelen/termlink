---
id: T-262
name: "Inception: Attach-self — register existing shell as TermLink endpoint"
description: >
  Pickup from fw-agent T-600. Add termlink attach command to wrap current shell as TermLink endpoint, discoverable via hub. Key questions: can register already do this, lifecycle on shell exit, SSH forwarding interaction, security model.

status: captured
workflow_type: inception
owner: agent
horizon: next
tags: [pickup, cli]
components: []
related_tasks: []
created: 2026-03-24T09:27:26Z
last_update: 2026-03-24T09:27:26Z
date_finished: null
---

# T-262: Inception: Attach-self — register existing shell as TermLink endpoint

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

### 2026-03-24T09:27:26Z — task-created [pickup from fw-agent T-600 on .107]
- **Source:** File transfer via TermLink (`termlink-pickup-003-attach-self.md`)
- **Pickup message:** Add `termlink attach` to wrap current shell as TermLink endpoint. Gap: `register` is for spawned processes, not existing interactive shells. Use case: SSH into remote, run `termlink attach`, local agent connects via hub.
