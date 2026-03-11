---
id: T-097
name: "Register G-005 conversation-only session loss + update FP-004"
description: >
  Register the specific gap: "conversation-only sessions are invisible to all framework
  enforcement" as G-005 in gaps.yaml. Also update failure pattern FP-004 to explicitly
  cover this sub-variant (previously FP-004 covered context exhaustion before handover
  generally, but not the conversation-only case). This makes the problem visible in
  future audits and watchtower scans.
status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [framework, governance, gaps, patterns]
components: []
related_tasks: [T-094]
created: 2026-03-11T11:30:00Z
last_update: 2026-03-11T23:22:33Z
date_finished: 2026-03-11T23:22:33Z
---

# T-097: Register G-005 + Update FP-004

## Context

Spawned from T-094 inception. Agent 5 confirmed this matches FP-004 but the specific
sub-variant (conversation-only, not tool-heavy) was not explicitly registered.
See: `docs/reports/T-094-volatile-conversation-prevention.md` (Agent 5 findings)

## Acceptance Criteria

### Agent
- [x] G-005 added to `.context/project/gaps.yaml` — "Conversation-only sessions bypass all framework enforcement"
- [x] FP-004 updated in `.context/project/patterns.yaml` with sub-variant: "conversation-only session" explicitly named
- [x] G-005 references T-094 as discovery task and T-095/T-096 as mitigation tasks

## Verification

grep -q "G-005" /Users/dimidev32/001-projects/010-termlink/.context/project/gaps.yaml
grep -q "conversation-only" /Users/dimidev32/001-projects/010-termlink/.context/project/patterns.yaml

## Updates

### 2026-03-11T23:18:33Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-11T23:22:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
