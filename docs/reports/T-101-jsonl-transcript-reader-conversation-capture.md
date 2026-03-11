# T-101: JSONL Transcript Reader — Conversation Capture
## Research Artifact | Created: 2026-03-11

---

## Problem

Agent sessions produce valuable exploratory conversations. When a session ends without
explicit capture, all conversation content is lost (triggered T-094). The JSONL
transcript file written by Claude Code contains the full conversation — this inception
explored whether it can be used as a capture source.

## What We Discovered

### File location and structure

`~/.claude/projects/<project-dir-encoded>/<session-uuid>.jsonl`

Found by budget-gate.sh via:
```
PROJECT_DIR_NAME=$(echo "$PROJECT_ROOT" | sed 's|/|-|g')
find "$HOME/.claude/projects/$PROJECT_DIR_NAME" -maxdepth 1 -name "*.jsonl" ! -name "agent-*"
```

Event types present:
| Type | Contains | Useful for capture? |
|---|---|---|
| `user` | Human message turns (text content) | YES |
| `assistant` | Agent response turns (text + tool_use blocks) | YES |
| `progress` | Tool execution intermediate states | No |
| `system` | Session metadata, duration | No |
| `file-history-snapshot` | File state snapshots | No |
| `queue-operation` | Internal queue state | No |
| `tool_result` (inside `user`) | Tool results, errors (`is_error: true`) | YES for T-103/T-104 |

### Written in real-time

Timestamps confirmed — events appear as they happen, not batched.
The file is live and growing during an active session.

### This session in numbers (2026-03-11)

- 25 human turns, 146 assistant turns
- 79 tool calls, 12 errors flagged with `is_error: true`
- Session duration: ~6 hours
- File size: ~1MB (612 lines at time of analysis)

### Cost analysis

| Use case | File scope | Size | Parse time | Concern? |
|---|---|---|---|---|
| T-101 conversation capture | Current session only | ~1MB | Milliseconds | None |
| T-104 cross-session tool counting | All sessions | ~53MB total | Seconds | Cache needed |

Prior large session: 52MB, 20,500 lines (outlier — unusually long session).
Growth estimate: ~1MB/active session. Cache/index needed for T-104, not T-101.

### Format stability

The JSONL event structure mirrors the **public Claude API message format**
(`user`/`assistant` roles, `content` arrays, `tool_use`/`tool_result` types).
Anthropic is unlikely to break internal format without also changing the public API.
Risk mitigation: format canary — validate expected event types present on read;
emit warning if not found rather than silent empty output.

### Sub-agent transcripts (gap — separated to T-107)

Background agents write to `/tmp/claude-501/.../tasks/<agent-id>.output` with
`isSidechain: true`. These files are NOT in the project JSONL and are ephemeral.
Sub-agent results are captured via `fw bus`, but reasoning trails are lost.
This is a separate concern — T-107 addresses it.

---

## Decisions Made

### Capture scope: current session only
Reading only the current session's JSONL file (identified as most recent non-agent-* file).
Cross-session aggregation belongs to T-104 (tool call store), not T-101.

### Capture boundary: topic start (Interpretation A)
`/capture` extracts back to where the current conversation topic started — the agent
identifies the natural topic boundary from transcript content. Fallback: last `/capture`
invocation if no clear boundary exists.

Rejected interpretation B (last capture invocation only) — less useful, doesn't recover
a topic that started before `/capture` was invoked.

### Sub-agent gap separated
T-101 captures human↔orchestrator conversation only. Sub-agent reasoning trails are a
distinct problem with distinct storage requirements — T-107.

---

## GO Decision

**GO** — 2026-03-11

All key questions answered, no blockers identified:
- Format stable ✓
- Cost negligible ✓
- Structure confirmed ✓
- Scope clearly defined ✓
- Known gap (sub-agents) separated to T-107 ✓

Next step: build task for JSONL transcript reader + `/capture` skill implementation.

---

## Dialogue Log

**Human:** "JSON transcript reader — already have that, can you please explain me how it works,
how is it different from the TermLink output capture."

**Agent:** Explained JSONL structure, confirmed it contains semantic conversation turns vs
raw terminal bytes for TermLink capture.

**Human (on tool errors):** "On three — does the internal tool call also note the tool call errors?"
**Agent:** Confirmed — `tool_result` entries with `is_error: true`, full error content captured.
ANSI codes present in some error content, needs stripping for clean output.

**Human (on JSONL risk):** "I'm not really sure whether to use JSONL — you noted this is an
Anthropic internal format which can change."
**Agent:** Clarified public API alignment reduces risk. Canary detection mitigates silent failure.

**Human (on sub-agents):** "Why do we only care about the human/orchestrator conversation?
Is that really true?"
**Agent:** Acknowledged the challenge was correct. Sub-agent reasoning trails ARE valuable and
ephemeral. Separated to T-107 to keep T-101 focused.

**Human (on Q1 cost):** "Sounds good, then it's a go I guess, as we have no concern right?"
**Agent:** Confirmed — no cost concern for T-101 scope.

**Human (on Q3):** "A" — capture back to topic start, agent judges boundary.
