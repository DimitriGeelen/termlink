# T-107: Inception — Sub-Agent Transcript Persistence

> Created: 2026-03-12 | Status: Research complete — GO/NO-GO pending

## Problem Statement

Background agents write their transcripts to `/tmp` with `isSidechain: true`, NOT to
the project JSONL. Concern: these files are ephemeral and lost on reboot.

## Findings

### The problem doesn't exist.

Claude Code already persists sub-agent transcripts to a durable location.

**Two-layer architecture (discovered):**

| Layer | Location | Lifecycle |
|-------|----------|-----------|
| Session (fast) | `/tmp/claude-501/<project>/tasks/<agent-id>.output` | Symlinks only — ephemeral |
| Persistent | `~/.claude/projects/<project>/<session-id>/subagents/agent-<id>.jsonl` | Durable — survives reboot |

**Confirmed:**
- 8 sub-agent JSONL files in `~/.claude/projects/.../2392cce9-.../subagents/`
- Total: 3.6 MB of persisted sub-agent transcripts from this session
- Format: Standard JSONL with `isSidechain: true`, `agentId`, `sessionId`, `parentUuid`
- Permissions: 0600 (user-only), appropriate for sensitive reasoning trails
- Progress events (PreToolUse/PostToolUse) captured — tool call details present

**The /tmp files are symlinks to the persistent files, not the source of truth.**

### What the persistent files contain
- Full reasoning trails (tool calls, hypotheses, intermediate findings)
- Tool use details (command, args, results)
- `progress` events with PreToolUse/PostToolUse hook data
- `assistant` messages with the agent's thinking

### Path stability
- `/tmp/claude-501/` uses the actual unix uid (501 on this machine)
- Stable within a machine; not stable across machines (different uid = different path)
- Persistent path (`~/.claude/`) is stable and standard

## GO/NO-GO Recommendation

**NO-GO** — the problem T-107 was created to solve is already solved by Claude Code.

Sub-agent transcripts are durable. No archival mechanism needs building.

### What might still be worth doing (scope pivot)

If the inception is re-scoped:
1. **T-104 alignment** — The persistent sub-agent JSONLs are a data source for the
   tool call capture store. T-104's parser should be aware of `~/.claude/.../subagents/`
   as an input location.
2. **Lifecycle policy** — How long do these files live? Is there a cleanup mechanism?
   `~/.claude/` could grow unbounded across sessions. Worth documenting but not urgent.
3. **Cross-session access** — The subagent files live under the session UUID directory.
   To query "what did sub-agents do across all sessions", you need to walk the session
   dirs. T-104's data layer should handle this aggregation.

## Dialogue Log

**Agent (research):** Checked `/tmp/claude-501/-Users-dimidev32-001-projects-010-termlink/tasks/` — found symlinks pointing to `~/.claude/projects/.../<session-id>/subagents/`. Confirmed 8 agent JSONL files, 3.6MB total, durable.

## Open Questions

- Does `~/.claude/` grow indefinitely? Is there a retention policy?
- At what point does T-104 consume sub-agent JSONLs vs. main session JSONL?
