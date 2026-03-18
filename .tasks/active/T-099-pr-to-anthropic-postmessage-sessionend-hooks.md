---
id: T-099
name: "PR to Anthropic — PostMessage / SessionEnd hook request for Claude Code"
description: >
  Draft and submit a feature request PR (or issue) to Anthropic / Claude Code asking
  for two new hook event types: PostMessage (fires after each assistant response) and
  SessionEnd (fires on normal session exit). The case is well-evidenced by this project.
  Before drafting, research the correct submission format and channel for Claude Code
  contributions/feature requests.
status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [anthropic, claude-code, hooks, framework, governance]
components: []
related_tasks: [T-094, T-095, T-096, T-100, T-101, T-102]
created: 2026-03-11T12:00:00Z
last_update: 2026-03-18T21:39:32Z
date_finished: 2026-03-18T21:37:58Z
---

# T-099: PR to Anthropic — PostMessage / SessionEnd Hook Request

## Problem Statement

Claude Code's hook system fires only on tool-boundary events (PreToolUse, PostToolUse,
PreCompact, SessionStart). This makes it impossible to enforce framework governance on
pure conversations — sessions with zero tool calls are invisible to all enforcement.

Two missing hook types would close this gap completely:
- **PostMessage** — fires after each assistant response (enables N-exchange guard)
- **SessionEnd** — fires on normal session exit (enables mandatory handover)

## Research Artifact

`docs/reports/T-099-postmessage-sessionend-hook-request.md`

## Background (for PR body)

The Agentic Engineering Framework is a governance system for AI agents built on top of
Claude Code. It uses hooks extensively for: task enforcement, context budget management,
tier-0 protection, plan-mode blocking, error watchdog. The framework is a production
system managing multi-session, multi-agent workflows with episodic memory, decision
capture, and structured handovers.

Full research: `docs/reports/T-094-volatile-conversation-prevention.md`
Evidence event: Agent Mesh research session lost 2026-03-11 (no hooks fired, pure conversation)

## Research Questions (before drafting)

- Where do Claude Code feature requests / PRs go? (GitHub repo? Feedback form? Discord?)
- What format does Anthropic prefer for hook-related requests?
- Is there prior art — has anyone else requested conversation-level hooks?
- What's the difference between a "feature request" and a "PR" for Claude Code?
  (Is Claude Code open source? Can we submit actual code?)

## Exploration Plan

1. Research Claude Code's public repo / contribution guidelines
2. Find existing feature request threads for hook improvements
3. Draft the request with: problem statement, evidence, proposed API design, use case
4. Review with human before submitting

## Scope Fence

**IN:** Research submission format, draft the request, submit with human approval
**OUT:** Implementing the hooks ourselves (Claude Code is Anthropic's product)

## Acceptance Criteria

### Agent
- [x] Submission channel identified (GitHub issues at anthropics/claude-code)
- [x] Correct format researched and documented
- [x] Research found both hooks already exist: Stop = PostMessage, SessionEnd = implemented
- [x] Research artifact written with gap analysis and recommendation

### Human
- [ ] Draft approved
- [ ] Submission made

## Verification

test -f docs/reports/T-099-postmessage-sessionend-hook-request.md
grep -q "NO-GO" docs/reports/T-099-postmessage-sessionend-hook-request.md

## Decisions

**Decision**: NO-GO

**Rationale**: Both hooks already exist in Claude Code (Stop + SessionEnd). No PR needed.

**Date**: 2026-03-18T21:37:58Z
## Decision

**Decision**: NO-GO

**Rationale**: Both hooks already exist in Claude Code (Stop + SessionEnd). No PR needed.

**Date**: 2026-03-18T21:37:58Z

## Updates

### 2026-03-18T21:34:23Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-03-18T21:34:23Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-18T21:37:58Z — status-update [task-update-agent]
- **Change:** owner: human → agent

### 2026-03-18T21:37:58Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** NO-GO
- **Rationale:** Both hooks already exist in Claude Code (Stop + SessionEnd). No PR needed.

### 2026-03-18T21:37:58Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: NO-GO
