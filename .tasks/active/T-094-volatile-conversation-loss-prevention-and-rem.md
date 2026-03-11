---
id: T-094
name: "Volatile conversation loss — prevention and remediation"
description: >
  Research conversations produce valuable insights that live only in conversation
  context. When a session ends without explicit artifact capture, all content is
  permanently lost. This inception explores the structural gap, designs preventions,
  and produces implementation tasks for both the termlink project and the framework agent.
status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [framework, governance, session-capture, antifragility]
components: []
related_tasks: [T-012, T-095, T-096, T-097, T-098, T-099, T-100, T-101, T-102]
created: 2026-03-11T11:00:00Z
last_update: 2026-03-11T11:00:00Z
date_finished: null
---

# T-094: Volatile Conversation Loss — Prevention and Remediation

## Problem Statement

On 2026-03-11, a full session of rich exploratory research on "Agent Mesh" (integrating
TermLink into the framework for multi-agent coordination) was conducted. The session
ended without creating a task, writing a research artifact, or committing anything.
All content was permanently lost.

Root cause: The framework's task gate and enforcement hooks only fire on file operations
(Write/Edit/Bash). A long conversation that produces no file writes bypasses all
structural enforcement completely. Agent discipline is the only guard — and it failed.

This is a systemic gap, not a one-off error.

## Research Artifact

See: `docs/reports/T-094-volatile-conversation-prevention.md`

## Assumptions

- The framework currently has no hook or mechanism that fires purely on conversation length
- Claude Code does not expose conversation content to hooks (hooks only see tool calls)
- Agent discipline alone is insufficient as a reliability mechanism
- The fix must be structural (codified rules + tooling), not behavioral

## Exploration Plan

Five parallel investigate agents:
1. **Hook Coverage** — Audit all hooks; determine if conversation-triggered enforcement is possible
2. **Session Lifecycle** — Map the full session start/end flow; find where capture can be injected
3. **C-001 and Protocol Rules** — Audit current CLAUDE.md rules; find the exact gaps
4. **Skill Infrastructure** — Explore how a `/capture` skill would work technically
5. **Episodic Pattern Mining** — Search episodic memory for prior loss events; quantify severity

## Technical Constraints

- Claude Code hooks fire on PreToolUse/PostToolUse/PreCompact — NOT on message generation
- Conversation content is not accessible to hooks
- Any structural fix must work within Claude Code's hook model

## Scope Fence

**IN:**
- Prevention mechanisms for the termlink project (CLAUDE.md rules, skills, hooks)
- Framework-level fix design (for the framework agent to implement as a PR)
- Recovery of the Agent Mesh research content

**OUT:**
- Changes to Claude Code internals
- Prevention for non-Claude agents

## Acceptance Criteria

### Agent
- [ ] 5 explore agents completed and findings synthesized
- [ ] Research artifact complete at `docs/reports/T-094-volatile-conversation-prevention.md`
- [ ] Remediation tasks created (implementation tickets)
- [ ] OneDev issue created for framework agent
- [ ] Console prompt generated for framework agent pickup

### Human
- [ ] Remediation approach approved
- [ ] OneDev entry reviewed

## Go/No-Go Criteria

**GO (implement remediations) if:**
- Structural fix is possible within Claude Code hook model
- A `/capture` skill is feasible
- CLAUDE.md rule addition would close the gap

**NO-GO if:**
- Claude Code model makes structural enforcement impossible
- Gap can only be fixed upstream (Claude Code product team)

## Verification

# Research artifact must exist
test -f docs/reports/T-094-volatile-conversation-prevention.md

## Decisions

## Decision

**Decision**: GO — 2026-03-11
5-agent investigation confirmed structural gap. Multiple remediation paths identified.
Spawned T-095–T-102. T-101 (JSONL transcript) active for dialogue now.
See: `docs/reports/T-094-volatile-conversation-prevention.md`

## Updates
