# T-107: Inception — Sub-Agent Transcript Persistence

> Created: 2026-03-12 | Two deep-dive agents run | Status: NO-GO (original) + new task spawned

## Problem Statement

Background agents write transcripts to `/tmp` with `isSidechain: true`. Concern:
ephemeral — lost on reboot. Are sub-agent reasoning trails being lost?

## Finding 1: Original problem doesn't exist

Claude Code already persists sub-agent transcripts durably.

**Two-layer architecture:**

| Layer | Location | Lifecycle |
|-------|----------|-----------|
| Session | `/tmp/claude-501/<project>/tasks/<agent-id>.output` | Symlinks only |
| **Persistent** | `~/.claude/projects/<project>/<session-id>/subagents/agent-<id>.jsonl` | **Durable** |

The `/tmp` files are symlinks to `~/.claude/`. Reboots clear the symlinks, not the data.

**Confirmed:** 8 sub-agent JSONL files from current session, 3.6 MB. 60 files across 2 sessions with sub-agents, 200+ MB. Full reasoning trails present.

---

## Finding 2: New problem discovered — unbounded growth

| Metric | Value |
|--------|-------|
| Sessions in `~/.claude/projects/...` | 15 (4 days) |
| Total size | 261 MB |
| Growth rate | ~65 MB/day |
| 30-day projection | ~1.8 GB |
| 6-month projection | ~8 GB |
| Cleanup mechanisms | **None** |
| Retention policy | **None** |

One heavy multi-agent session (30 sub-agents) = 201 MB alone.

Meta.json files contain only `{"agentType":"Explore"}` — too sparse for indexing or selective cleanup.

**This is a real and active problem.** `~/.claude/` will grow to gigabytes with no automatic relief valve.

---

## Finding 3: Sidechain files have better structure than main JSONL

Main session JSONL lacks structured tool_result events — errors appear as prose in `assistant` messages.

**Sidechain files have:**
- `tool_result` events with `is_error: true/false` — structured error data ✓
- `progress` events with tool name, command, exit code ✓
- Full reasoning trails (hypotheses, intermediate findings) ✓
- `agentId`, `sessionId`, `parentUuid` for identity ✓
- Agent task context (what it was asked to do) ✗ — NOT stored

**Implication for T-104:** The cross-session tool call store MUST consume both the main session JSONL and sub-agent sidechain files. Sidechains are actually the richer source for tool errors.

---

## Finding 4: Cross-session queryability

Walking `~/.claude/.../*/subagents/*.jsonl` to aggregate tool calls across sessions:
- Complexity: medium-low (~100 line Python script)
- No natural index — session dirs keyed by UUID, not date
- Feasible with filesystem walk + caching above 50 sessions

---

## GO/NO-GO Decision

**T-107 as scoped: NO-GO** — original problem (ephemerality) is already solved by Claude Code.

**New task warranted (T-110):** Transcript retention policy + `fw transcripts clean` command.
Scope: 30-day TTL default, `--older-than N` flag, dry-run mode. ~1 day of build work.

**T-104 design note:** Unified parser must handle both main JSONL and sidechain files.
Sidechain files are the primary source of structured tool errors.

---

## Options Explored

| Option | Verdict |
|--------|---------|
| Build archival mechanism for /tmp files | NO-GO — /tmp are symlinks, data already in ~/.claude |
| Accept status quo (no cleanup) | NO-GO — 8 GB in 6 months is untenable |
| Retention policy + cleanup command (T-110) | **GO — separate task** |
| T-104 unified parser consuming sidechains | **GO — T-104 design constraint** |

## Open Questions

- Should T-110 compress archived transcripts or just delete? (delete is simpler)
- Should `fw handover` report `~/.claude/` size as a health signal?
- Should T-104 task include a note that sidechain files are the primary error source?
