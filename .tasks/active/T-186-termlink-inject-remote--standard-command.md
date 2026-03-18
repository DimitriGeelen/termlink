---
id: T-186
name: "termlink inject-remote — standard command for cross-machine prompt injection"
description: >
  Design a standard CLI command for repeatable cross-machine prompt injection. Currently requires: hex secret parsing, HMAC token generation, TOFU TLS, hub routing — all manual via tofu_test example. Need: 'termlink inject-remote host:port session-name "message" --secret-file /path'. Should handle auth, TOFU, and split-writes automatically. Also consider 'termlink connect-hub' for persistent hub connections.
status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [cli, cross-machine, ux]
components: []
related_tasks: [T-182, T-183, T-184, T-185]
created: 2026-03-18T23:28:42Z
last_update: 2026-03-18T23:28:42Z
date_finished: null
---

# T-186: termlink inject-remote — standard command for cross-machine prompt injection

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

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->
