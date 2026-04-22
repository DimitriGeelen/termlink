---
id: T-176
name: "Audit framework hook config against 24 available Claude Code hooks"
description: >
  Framework uses only 4 of 24 available Claude Code hooks. Audit which new hooks (PostCompact, PostToolUseFailure, ConfigChange, InstructionsLoaded, etc.) would improve enforcement. See docs/reports/T-099 for full hook inventory.

status: captured
workflow_type: inception
owner: human
horizon: next
tags: [framework, hooks, audit]
components: []
related_tasks: []
created: 2026-03-18T21:39:25Z
last_update: 2026-04-22T04:52:50Z
date_finished: null
---

# T-176: Audit framework hook config against 24 available Claude Code hooks

## Problem Statement

Framework uses 4 of ~24 available Claude Code hook types (11 matchers). Are we missing enforcement opportunities? Specifically: response-level governance (Stop), session exit (SessionEnd), and sub-agent result validation (SubagentStop) are not wired. The 4 configured types cover tool boundaries well but leave gaps between tool calls.

## Assumptions

1. More hook types = better enforcement — PARTIALLY TRUE: Stop and SessionEnd add real value; others are marginal
2. Claude Code hook API is stable and documented — PARTIALLY TRUE: known reliability bugs in SessionEnd (#17885, #20197)
3. Adding hooks has negligible performance overhead — UNTESTED: PostToolUse hooks already fire on every tool call

## Exploration Plan

1. Inventory all available Claude Code hook types from docs and T-099 report
2. Map current usage from .claude/settings.json
3. Assess value of each unused hook for agentic framework enforcement
4. Prioritize by enforcement gap severity

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
- [x] Problem statement validated
- [x] Assumptions tested
- [x] Recommendation written with rationale (see `docs/reports/T-176-hook-audit.md`)

### Human
- [x] [REVIEW] Review hook audit findings and confirm priority order
  **Steps:**
  1. Read `docs/reports/T-176-hook-audit.md`
  2. Confirm Stop (T-173), SessionEnd (T-174), SubagentStop (T-175) priority
  3. Run: `fw inception decide T-176 go|no-go --rationale "your rationale"`
  **Expected:** Decision confirms priority order for hook wiring
  **If not:** Adjust priority based on your operational experience

## Go/No-Go Criteria

**GO if:**
- At least one unused hook type adds enforcement value
- Framework can wire new hooks without breaking existing configuration

**NO-GO if:**
- All useful hooks are already configured
- Hook API is too unreliable for production use

## Verification

<!-- Shell commands that MUST pass before work-completed. One per line.
     Lines starting with # are comments. Empty lines ignored.
     The completion gate runs each command — if any exits non-zero, completion is blocked.
     For inception tasks, verification is often not needed (decisions, not code).
-->

## Decisions

### 2026-03-27 — Go/No-Go
- **Chose:** GO — proceed with hook audit
- **Why:** Framework now uses 11 hooks (up from 4). Worth auditing remaining 13 for enforcement gaps.
- **Rejected:** NO-GO — hooks are sufficient as-is

## Recommendation

_Backfilled 2026-04-19 under T-1139/T-1112 scope — inception decide ran before `## Recommendation` became a required section. Content mirrors the `## Decision` block below for audit compliance (CTL-027)._

**Decision (retro-captured from Decision block):** GO

**Rationale:** Framework uses 11/24 available hooks. Audit remaining 13 for enforcement value.

## Decision

**Decision**: GO

**Rationale**: Framework uses 11/24 available hooks. Audit remaining 13 for enforcement value.

**Date**: 2026-03-27T12:56:46Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-22T17:22:24Z — status-update [task-update-agent]
- **Change:** horizon: later → later

### 2026-03-26T13:30:00Z — staleness-review [T-293]
- **Status:** Parked inception awaiting human prioritization. Framework now uses 11 hooks (up from 4 when captured). Re-evaluate scope when ready.

### 2026-03-27T12:56:46Z — inception-decision [inception-workflow]
- **Action:** GO decision recorded via Watchtower (human-approved)

### 2026-04-22T04:52:50Z — status-update [task-update-agent]
- **Change:** horizon: later → next
- **Change:** status: started-work → captured (auto-sync)
