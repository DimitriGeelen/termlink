---
id: T-099
name: "PR to Anthropic — PostMessage / SessionEnd hook request for Claude Code"
description: >
  Draft and submit a feature request PR (or issue) to Anthropic / Claude Code asking
  for two new hook event types: PostMessage (fires after each assistant response) and
  SessionEnd (fires on normal session exit). The case is well-evidenced by this project.
  Before drafting, research the correct submission format and channel for Claude Code
  contributions/feature requests.
status: captured
workflow_type: inception
owner: human
horizon: next
tags: [anthropic, claude-code, hooks, framework, governance]
components: []
related_tasks: [T-094, T-095, T-096, T-100, T-101, T-102]
created: 2026-03-11T12:00:00Z
last_update: 2026-03-11T12:00:00Z
date_finished: null
---

# T-099: PR to Anthropic — PostMessage / SessionEnd Hook Request

## Problem Statement

Claude Code's hook system fires only on tool-boundary events (PreToolUse, PostToolUse,
PreCompact, SessionStart). This makes it impossible to enforce framework governance on
pure conversations — sessions with zero tool calls are invisible to all enforcement.

Two missing hook types would close this gap completely:
- **PostMessage** — fires after each assistant response (enables N-exchange guard)
- **SessionEnd** — fires on normal session exit (enables mandatory handover)

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
- [ ] Submission channel identified (GitHub issue / form / other)
- [ ] Correct format researched and documented
- [ ] Draft written with full background + T-094 evidence
- [ ] Draft reviewed and approved by human

### Human
- [ ] Draft approved
- [ ] Submission made

## Decisions

## Decision

## Updates
