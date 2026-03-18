# T-102: Orchestrator Mandatory Tool Call Constraint
## Research Artifact | Created: 2026-03-18

---

## Problem

Agent conversations are ephemeral — when context compacts or a session ends, unstructured
dialogue disappears. The framework captures tool calls (via hooks), task updates, and commits,
but pure conversation between human and agent is invisible to the system.

T-094 explored four approaches: T-099 (Anthropic hook), T-100 (TermLink capture), T-101
(JSONL transcript), and T-102 (this task — architectural constraint). T-100 was NO-GO.
T-101/T-109 provide a working capture mechanism. This inception asks: should we go further
and architecturally prevent invisible conversations?

## Variant Analysis

### Variant A — Mandatory `fw note` Per Response

**Concept:** Every substantive orchestrator response must include a `fw note "..."` tool call.
Pure text responses without a tool call are flagged or blocked.

| Dimension | Assessment |
|-----------|------------|
| **Traceability** | HIGH — every turn becomes a hookable tool event |
| **Implementation** | MEDIUM — new `fw note` command + PostToolUse enforcement hook |
| **UX impact** | NEGATIVE — forces artificial tool calls, breaks conversational flow |
| **Enforcement** | HARD — how to define "substantive"? Greeting vs. design discussion? |
| **Cost** | LOW — no extra API calls or agents |

**How it would work:**
1. `fw note "summary"` writes to `.context/working/conversation-log.jsonl`
2. PostToolUse hook on response completion checks if any `fw note` was called
3. If not, warns (advisory) or blocks next tool call (strict)

**Fatal flaw:** The enforcement boundary is wrong. Claude Code doesn't expose a "response
complete" hook — hooks fire on individual tool calls, not on response boundaries. We can't
detect "agent responded with pure text and no tool call" because there's no hook for that.
PreToolUse fires BEFORE a tool call (can't check if previous response had one).
PostToolUse fires AFTER (too late to enforce on the same turn).

### Variant B — Spawn-for-Conversation

**Concept:** Orchestrator never explores inline. Any discussion or exploration is delegated
to a spawned sub-agent that returns structured results.

| Dimension | Assessment |
|-----------|------------|
| **Traceability** | HIGH — all exploration produces artifacts via fw bus |
| **Implementation** | LOW — already how dispatch.sh + fw bus work |
| **UX impact** | SEVERE — human can't have a quick back-and-forth with orchestrator |
| **Enforcement** | IMPOSSIBLE — can't prevent the model from generating text |
| **Cost** | HIGH — every exploration spawns an agent ($, latency) |

**Fatal flaw:** This is architecturally impossible to enforce. The LLM generates text
responses — you can't prevent it from having a conversation. You can only add hooks around
tool calls. Even if you instructed "always spawn an agent," the instruction itself is
conversational and unenforceable structurally.

### Variant C — Scribe Agent (TermLink Events)

**Concept:** A lightweight TermLink session acts as a conversation logger. The orchestrator
routes responses through the scribe, which logs them as persistent TermLink events.

| Dimension | Assessment |
|-----------|------------|
| **Traceability** | HIGH — events are persistent, queryable, replayable |
| **Implementation** | MEDIUM — needs scribe session + event emission wiring |
| **UX impact** | NONE — transparent to the human |
| **Enforcement** | IMPOSSIBLE — same problem as Variant B |
| **Cost** | LOW — event emission is lightweight |
| **Prerequisite** | Agent mesh + TermLink session wrapping (T-156 exists) |

**Problem:** Same enforcement gap. The scribe can only log what's routed to it — if the
orchestrator has a pure text conversation, the scribe never sees it. You'd need Variant A's
enforcement to ensure routing, which has its own fatal flaw.

## Cross-Cutting Analysis

### The Fundamental Problem

All three variants share the same enforcement gap: **Claude Code has no "response boundary"
hook.** The hook system operates on tool calls:

```
PreToolUse  → fires BEFORE a tool is called
PostToolUse → fires AFTER a tool completes
```

There is no:
- `PreResponse` — before the model generates text
- `PostResponse` — after the model finishes a response
- `ResponseComplete` — when a full turn (text + tools) is done

Without a response boundary hook, you cannot detect or enforce "every response must include
a tool call." This is exactly what T-099 (Anthropic PR) proposes to fix with PostMessage.

### What Already Works

| Mechanism | Captures | Status |
|-----------|----------|--------|
| **JSONL transcript** (T-101) | Full conversation including pure text | Working |
| **`/capture` skill** (T-109) | Extracts and formats from JSONL | Working |
| **fw bus** | Sub-agent results and artifacts | Working |
| **Hook system** | All tool call events | Working |
| **Commit messages** | Decision summaries | Working |
| **Task updates** | Status transitions with context | Working |

The gap is narrow: pure conversational turns between human and orchestrator that don't
produce tool calls. This is primarily:
- Clarification questions ("did you mean X or Y?")
- Quick answers ("yes, go ahead")
- Brief status updates ("done, what's next?")

These are low-value turns for traceability — the substantive work always involves tool calls.

### Impact on Human-Agent Interaction

Enforcing mandatory tool calls would fundamentally damage the UX:

1. **Latency tax:** Every response needs an extra tool call round-trip
2. **Cognitive overhead:** Agent must decide what to "note" vs. what to do
3. **Artificial ceremony:** Simple exchanges become multi-step procedures
4. **Conversation feel:** Transforms fluid dialogue into bureaucratic protocol

The framework already captures ~95% of substantive content through existing mechanisms.
The remaining ~5% (pure text turns) can be captured post-hoc via JSONL transcript.

### Relationship to Agent Mesh Roadmap

Agent mesh dispatch (T-143, dispatch.sh) already enforces structured results for sub-agents:
- Sub-agents MUST write output to disk (fw bus post)
- Results are size-gated (check-dispatch.sh PostToolUse guard)
- Orchestrator reads manifests, not raw content

This is Variant B applied to sub-agents — and it works because sub-agents are spawned
programmatically. The orchestrator itself can't be constrained the same way because it's
the interactive agent in conversation with the human.

## Go/No-Go Recommendation

**NO-GO** for all three variants.

**Rationale:**
1. **Enforcement gap:** No response boundary hook exists in Claude Code — the fundamental
   mechanism needed for mandatory tool calls is absent
2. **UX damage:** Mandatory tool calls per response would severely degrade conversational flow
3. **Low marginal value:** Existing mechanisms (JSONL, hooks, bus, commits) capture ~95% of
   substantive content; the remaining pure-text turns are low-value for traceability
4. **Better path exists:** T-099 (Anthropic PostMessage hook) would solve this at the
   platform level without architectural contortion
5. **T-101/T-109 already work:** JSONL transcript + /capture skill provide conversation
   capture today without any architectural changes

**Recommended instead:**
- Continue using JSONL transcript capture (T-101/T-109) as the primary mechanism
- Advocate for PostMessage hook with Anthropic (T-099) as the long-term fix
- Accept that pure conversational turns are low-value and don't warrant architectural overhead
- Focus agent mesh enforcement on sub-agents (where it's structurally enforceable)
