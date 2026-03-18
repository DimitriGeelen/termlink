---
id: T-100
name: "Inception — TermLink output capture as conversation logger"
description: >
  Explore and investigate whether TermLink's terminal output capture capability can be
  used to log Claude Code conversation turns. TermLink can attach to a terminal session
  and stream its output. This exploration asks: can we wrap a Claude Code session as a
  TermLink session, capture stdout, and extract conversation turns from the raw stream?
  Explore only — no implementation.
status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [termlink, output-capture, conversation-logging, exploration]
components: []
related_tasks: [T-094, T-101, T-102, T-099]
created: 2026-03-11T12:00:00Z
last_update: 2026-03-18T21:29:51Z
date_finished: 2026-03-18T21:29:51Z
---

# T-100: Inception — TermLink Output Capture as Conversation Logger

## Research Artifact

`docs/reports/T-100-termlink-output-capture-conversation-logger.md`

## Problem Statement

Can TermLink's existing output capture / stream / attach capabilities be used to capture
Claude Code conversation turns — without any changes to Claude Code itself?

## What We Know

TermLink can:
- Register any terminal session as a named TermLink session
- Attach to a session and stream its stdout (T-007, bidirectional injection)
- Capture output, inject input
- Store captured data in KV or emit as events

Claude Code's conversation appears in the terminal as formatted text with:
- ANSI escape codes (colors, bold, box-drawing)
- Spinner animations during tool execution
- Human message rendering
- Assistant response rendering
- Tool use indicators

## Questions to Explore

1. What exactly does Claude Code's stdout look like at the byte level? Are there
   structural markers (e.g., consistent patterns before/after each message turn)?
2. Can TermLink attach to a Claude Code process that's already running, or does it
   need to wrap it at launch?
3. How much ANSI stripping is needed to get clean text? Is there tooling for this?
4. Can we distinguish a "conversation turn" from "tool execution output" in the raw stream?
5. How does this compare to the JSONL transcript approach (T-101) in terms of:
   - Structure (raw vs. parsed)
   - Completeness (what's captured vs. what's missed)
   - Complexity (how hard to implement)
   - Reliability (what can go wrong)
6. Is there a risk of TermLink capture interfering with Claude Code's operation?

## Relationship to Other Options

- **T-101 (JSONL transcript):** Structured vs. raw. JSONL is cleaner but requires
  file access. TermLink capture is richer (captures everything) but noisier.
- **T-099 (Anthropic PR):** If Anthropic adds PostMessage hook, TermLink capture
  becomes unnecessary. But TermLink capture works TODAY without Anthropic changes.
- **T-102 (orchestrator constraint):** Orthogonal — forces tool calls rather than
  capturing conversation passively.

## Scope Fence

**IN:** Understand the approach, assess feasibility, identify unknowns
**OUT:** Any implementation — this is exploration only

## Acceptance Criteria

### Agent
- [x] TermLink output capture mechanics understood and documented
- [x] Claude Code stdout structure analyzed (what does it look like?)
- [x] Comparison table: TermLink capture vs. JSONL transcript completed
- [x] Feasibility assessment: go/no-go for next phase

### Human
- [ ] Exploration findings reviewed and discussed

## Verification

test -f docs/reports/T-100-termlink-output-capture-conversation-logger.md
grep -q "NO-GO" docs/reports/T-100-termlink-output-capture-conversation-logger.md

## Decisions

**Decision**: NO-GO

**Rationale**: JSONL transcript superior

**Date**: 2026-03-18T21:29:51Z
## Decision

**Decision**: NO-GO

**Rationale**: JSONL transcript superior

**Date**: 2026-03-18T21:29:51Z

## Updates

### 2026-03-18T21:18:53Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-03-18T21:19:19Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-18T21:25:25Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** NO-GO
- **Rationale:** JSONL transcript (T-101) provides strictly superior structured data for conversation logging. TermLink capture is fragile (ANSI parsing), version-dependent, and redundant — /capture skill already exists via JSONL. TermLink value is real-time observation/injection, not logging.

### 2026-03-18T21:29:45Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** NO-GO
- **Rationale:** JSONL transcript superior

### 2026-03-18T21:29:51Z — status-update [task-update-agent]
- **Change:** owner: human → agent

### 2026-03-18T21:29:51Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** NO-GO
- **Rationale:** JSONL transcript superior

### 2026-03-18T21:29:51Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: NO-GO
