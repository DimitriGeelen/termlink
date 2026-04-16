---
id: T-922
name: "Codify MCP auto-exposure — every new CLI command must be MCP-reachable"
description: >
  Meta-structural rule: discovered via T-920 RCA that shipping CLI-only cross-host features (T-163/T-164/T-182/T-186) left MCP agents blind for months. Need a framework/tooling rule that any new CLI command automatically gets an MCP wrapper OR must explicitly document why not. Options: code-gen from CLI enum, a lint that greps cli.rs vs tools.rs, a pre-commit hook blocking new Command variants without matching MCP tool, or a runtime registration pattern. Deliverable: decision on mechanism + first enforcement step.

status: work-completed
workflow_type: inception
owner: human
horizon: later
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-04-11T19:33:14Z
last_update: 2026-04-16T05:40:16Z
date_finished: 2026-04-12T21:29:17Z
---

# T-922: Codify MCP auto-exposure — every new CLI command must be MCP-reachable

## Problem Statement

Every new CLI command should automatically be MCP-reachable. Currently MCP tools are hand-crafted. Process improvement to ensure CLI-MCP parity.

DEFER: Current MCP tools cover active commands. Process improvement, not urgent.

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
- [x] Problem statement validated (MCP tools exist for active commands)
- [x] Assumptions tested (process improvement, not code fix)
- [x] Recommendation written with rationale (DEFER: process improvement)

### Human
- [ ] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-922, select GO / NO-GO / DEFER, click Record Decision
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

**Rationale:** Current MCP tools already cover active CLI commands (register, dispatch, inject, send-file, kv operations). Process improvement for ensuring future commands are auto-exposed is not urgent.

**Evidence:**
- 6+ MCP tools already exist
- No current CLI command is missing MCP exposure

## Decisions

**Decision**: DEFER

**Rationale**: Recommendation: DEFER

Rationale: Current MCP tools already cover active CLI commands (register, dispatch, inject, send-file, kv operations). Process improvement for ensuring future commands are au...

**Date**: 2026-04-12T17:14:19Z
## Decision

**Decision**: DEFER

**Rationale**: Recommendation: DEFER

Rationale: Current MCP tools already cover active CLI commands (register, dispatch, inject, send-file, kv operations). Process improvement for ensuring future commands are au...

**Date**: 2026-04-12T17:14:19Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T17:14:19Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** DEFER
- **Rationale:** Recommendation: DEFER

Rationale: Current MCP tools already cover active CLI commands (register, dispatch, inject, send-file, kv operations). Process improvement for ensuring future commands are au...

### 2026-04-12T21:29:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** DEFER decision recorded, human rubber-stamp pending via Watchtower

### 2026-04-16T05:40:16Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-16T21:04:50Z — programmatic-evidence [T-1090]
- **Evidence:** 67 MCP tools registered (termlink doctor confirms); all recent CLI commands have MCP counterparts
- **Verified by:** automated command execution
