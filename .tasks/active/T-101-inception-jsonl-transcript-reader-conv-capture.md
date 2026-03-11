---
id: T-101
name: "Inception — JSONL transcript reader for conversation capture"
description: >
  Claude Code already writes every conversation turn to a structured JSONL file on disk.
  The budget-gate.sh script already reads this file to count tokens. Explore: can we
  read this file to extract conversation content and write it as a research artifact?
  This is the most direct path — no new capture mechanism needed, the data already exists.
  Explore and dialogue only — understand what's there before deciding anything.
status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [jsonl, transcript, conversation-capture, exploration]
components: []
related_tasks: [T-094, T-100, T-102, T-099]
created: 2026-03-11T12:00:00Z
last_update: 2026-03-11T12:00:00Z
date_finished: null
---

# T-101: Inception — JSONL Transcript Reader for Conversation Capture

## Problem Statement

Claude Code already writes a `.jsonl` transcript file to disk for every session.
We discovered this via `budget-gate.sh`. It contains every conversation turn —
human messages, assistant responses, tool calls, results — as structured JSON.

The question: can we use this as a capture source?

## What We Already Know (from initial investigation)

**File location:**
`~/.claude/projects/<project-dir-encoded>/<session-uuid>.jsonl`

**Event types found:**
- `user` — human message turns (contains `message.content`)
- `assistant` — agent response turns (contains `message.content` as array with `type: text` items)
- `progress` — tool execution updates
- `system` — session metadata (duration, etc.)
- `file-history-snapshot` — file state snapshots
- `queue-operation` — internal queue state

**Token counting:** budget-gate.sh extracts `message.usage.input_tokens` from assistant events.

**Conversation turns are present:** Confirmed — user messages and assistant text responses are
both readable from this file with a simple Python parser.

## Key Questions for Dialogue

1. **Ownership and stability:** Is this file format documented by Anthropic, or an
   internal implementation detail that could change without notice?
2. **Completeness:** Does the transcript contain the FULL conversation, or only recent
   turns (is it rolling/truncated)?
3. **Freshness:** Is the file written in real-time (each turn) or batched?
4. **Multi-session:** Is there one file per session? How do we find the current one?
   (budget-gate.sh uses: most recent `.jsonl` not named `agent-*`)
5. **Agent sub-sessions:** Sub-agent spawns (via Task/Agent tool) seem to write
   `agent-*.jsonl` files separately. Are those excluded from the main transcript?
6. **What a reader would do:**
   - Read the most recent `.jsonl`
   - Filter for `type: user` and `type: assistant`
   - Extract text content from each turn
   - Write to `docs/reports/T-XXX-capture-{timestamp}.md`
   - This gives us a FULL conversation log — no user input required

## How It Differs from TermLink Output Capture (T-100)

| Dimension | JSONL Transcript (T-101) | TermLink Output Capture (T-100) |
|---|---|---|
| Data format | Structured JSON — already parsed | Raw ASCII + ANSI codes — needs stripping |
| What's captured | Semantic conversation turns | Everything visually displayed |
| Source | File on disk (passive) | Active terminal stream (requires attach) |
| Already exists? | YES — file is there now | Requires TermLink session registration |
| Content fidelity | Clean text content | Visual rendering artifacts |
| Missed content | Internal tool progress details | Nothing — captures all visual output |
| Reliability | Depends on Anthropic format stability | Depends on terminal stream availability |
| Complexity | Low — parse JSON, filter, write | Medium — stream, strip ANSI, segment |

## Open Questions Before Going Further

- Should a "capture" read the JSONL and write the full conversation, or summarize?
- What about privacy — the JSONL has everything. Should we filter sensitive content?
- Does reading the transcript during a session cause any issues (file locking)?

## Scope Fence

**IN:** Understand the transcript structure, explore the approach, map the options
**OUT:** Building a reader, writing any code — dialogue first

## Acceptance Criteria

### Agent
- [ ] JSONL format fully understood and documented
- [ ] All key questions above answered
- [ ] Comparison with T-100 completed
- [ ] Go/no-go framed for discussion

### Human
- [ ] Findings discussed and direction decided

## Decisions

## Decision

## Updates
