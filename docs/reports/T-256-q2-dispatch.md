# T-256 Q2: Current Dispatch Architecture

**Task:** T-256 — Interactive Multi-Agent Communication
**Question:** What is the current dispatch architecture and what would change for push-based result delivery?
**Date:** 2026-03-23

---

## 1. Current Architecture Overview

The framework uses a **file-based result ledger** pattern — not a daemon, worker pool, or message broker. Sub-agents are ephemeral Claude Code `Task` tool invocations. Results flow through the filesystem.

**Design philosophy** (from T-108 research): "The framework doesn't need a communication bus. It needs a structured result ledger with a read-before-write protocol."

### Flow Diagram

```
Orchestrator (main Claude session)
  ↓ Task tool dispatch (with preamble)
Sub-agent (ephemeral)
  ├── Writes details → /tmp/fw-agent-{name}.md
  ├── Posts summary  → fw bus post --task T-XXX --agent TYPE --summary "..."
  └── Returns ≤5 lines to orchestrator
         ↓
Orchestrator polls
  ├── fw bus read T-XXX     (envelope summaries)
  └── Read /tmp/fw-agent-*  (if details needed)
```

## 2. `fw bus` — The Result Ledger

**Implementation:** `/usr/local/opt/agentic-fw/libexec/lib/bus.sh` (365 lines)
**Routing:** `fw` dispatches at line 1116 via `source "$FW_LIB_DIR/bus.sh"; do_bus "$@"`

### Commands

| Command | Purpose |
|---------|---------|
| `fw bus post` | Write YAML envelope to task channel |
| `fw bus read T-XXX [R-NNN]` | Read all summaries or one full result |
| `fw bus manifest [T-XXX]` | Compact table view of results |
| `fw bus clear T-XXX` | Delete all results + blobs for task |

### Storage Layout

```
.context/bus/
  results/T-XXX/R-001.yaml   # YAML envelopes (metadata + small payloads)
  blobs/T-XXX/R-001.blob     # Large payloads (≥2048 bytes)
  inbox/                      # Cross-session message queue (stub, not active)
```

### Envelope Schema

```yaml
id: R-001
task_id: T-108
agent_type: explore
timestamp: 2026-02-17T10:30:00Z
type: artifact          # artifact | discovery | warning | dependency
summary: "Found 3 issues in auth module"
size_bytes: 245
payload: "inline text"       # if <2048B
payload_ref: "/path/to/blob" # if ≥2048B
```

**Size gating:** `BUS_SIZE_GATE=2048`. Payloads below threshold are inline YAML; above are written to blob files and referenced via `payload_ref`.

**ID auto-increment:** R-001, R-002, R-003... per task channel.

## 3. Worker Spawning

**There are no dispatch/spawn scripts.** The framework has no worker pool, daemon, or process manager. Sub-agents are spawned entirely through Claude Code's `Task` tool — the framework only provides:

1. **Dispatch preamble** (`agents/dispatch/preamble.md`) — mandatory prompt header for all sub-agent dispatches. Enforces the ≤5-line output rule that prevents context explosion.

2. **Dispatch guard** (`agents/context/check-dispatch.sh`) — PostToolUse hook that fires on `Task`/`TaskOutput` tool calls. Measures response size; warns at 5K chars, critical at 20K chars. Advisory only (exit 0 always).

3. **Dispatch templates** (`agents/dispatch/`) — `investigate.md`, `enrich.md`, `audit.md`, `develop.md` — standardized prompt patterns for common dispatch scenarios.

### Dispatch Constraints (from CLAUDE.md)

| Rule | Value | Origin |
|------|-------|--------|
| Max parallel agents | 5 | T-073 (9 agents → 177K spike → crash) |
| Token headroom before dispatch | 40K free | Sub-Agent Dispatch Protocol |
| Content generators | Write to disk, return path + summary | Result Management Rules |
| Investigators | Return structured summary <2K tokens | Result Management Rules |
| Background agents | `run_in_background: true` if >500 tokens expected | Preamble |

## 4. Result Collection: Pull-Based

The current model is **exclusively pull-based:**

1. **Orchestrator dispatches** sub-agent via Task tool with preamble
2. **Sub-agent writes** output to `/tmp/fw-agent-*.md` and/or `fw bus post`
3. **Orchestrator polls** by reading the Task tool's return value (≤5 lines) or `fw bus read`
4. **If details needed:** orchestrator reads the `/tmp/` file or blob reference

### Bus Inbox (Dormant)

`bus-handler.sh` exists at `agents/context/bus-handler.sh` — designed for systemd.path triggers when files appear in `.context/bus/inbox/`. Currently a stub: it logs and moves files to `.processed/` but performs no real processing. This was a T-110 spike that never shipped.

## 5. What Would Change for Push-Based (TermLink) Result Delivery

### Current Pain Points

1. **No real-time notification:** Orchestrator must poll or wait for Task tool completion. Background agents require manual `tail -5` of output files.
2. **File-based only:** Results live on one machine. No cross-machine dispatch.
3. **No streaming:** A long-running sub-agent can't send intermediate progress.
4. **Inbox is dead code:** The systemd.path handler was designed for push but never activated.

### What TermLink Enables

TermLink provides: session discovery (`discover`), targeted messaging (`send`), request/response (`request` via the protocol crate), event streaming (`events`/`emit`), and session metadata (`tag`, `info`).

### Required Changes

**A. Result Delivery (replace pull with push)**

| Current | TermLink |
|---------|----------|
| Sub-agent writes `/tmp/fw-agent-*.md` | Sub-agent calls `termlink send --to orchestrator --data '{"summary": "...", "blob": "/path"}'` |
| Orchestrator reads `fw bus read T-XXX` | Orchestrator receives via `termlink events` or `termlink attach` |
| Polling via `tail -5 /tmp/.../tasks/*.output` | Push notification on completion |

The bus envelope format (YAML with size gating) remains valid — the transport changes, not the schema.

**B. Worker Spawning (new: TermLink-aware dispatch)**

```bash
# Proposed: fw dispatch --task T-XXX --agent explore --prompt "..."
# 1. Spawns sub-agent session (claude -c with preamble)
# 2. Registers it via termlink register --tag "task:T-XXX,agent:explore"
# 3. Orchestrator discovers workers: termlink discover --tag "task:T-XXX"
# 4. Sub-agent sends result: termlink send --to orchestrator --data envelope.yaml
```

This replaces the "hope the Task tool returns ≤5 lines" convention with structural enforcement: sub-agents push results through a typed channel.

**C. Progress Streaming (new capability)**

```bash
# Sub-agent emits progress events
termlink emit --topic "task:T-XXX:progress" --data '{"step": 3, "total": 5, "msg": "Scanning tests..."}'

# Orchestrator watches
termlink watch --topic "task:T-XXX:progress"
```

Currently impossible — orchestrator has zero visibility into sub-agent progress until completion.

**D. Cross-Session Communication (activates dormant inbox)**

The `bus-handler.sh` inbox pattern becomes real:
- Instead of systemd.path watching a directory, `termlink events` listens for incoming messages
- The handler routes messages to the appropriate bus channel
- Enables multi-machine dispatch (sub-agent on remote server, results pushed back)

### What Stays the Same

1. **Bus envelope schema** — YAML format, size gating, R-NNN IDs
2. **Dispatch preamble** — output budget rules still apply (context protection)
3. **Dispatch guard** — PostToolUse size checking remains as defense-in-depth
4. **fw bus post/read/manifest/clear** — file-based ledger remains as durable record; TermLink adds real-time notification layer on top

### Migration Path

1. **Phase 1:** Add `termlink send` as optional result notification alongside file writes (backward compatible)
2. **Phase 2:** Orchestrator listens via `termlink events` instead of polling (replaces `tail -5`)
3. **Phase 3:** `fw dispatch` command wraps session creation + TermLink registration + preamble injection
4. **Phase 4:** Cross-machine dispatch via TermLink hub relay

---

## Summary

The current dispatch architecture is a **file-based pull model** with structural guardrails (preamble, size guard, bus envelopes). It works but has no real-time notification, no streaming, and no cross-machine capability. TermLink would add a **push-based transport layer** while preserving the existing schema and safety mechanisms. The dormant inbox handler is evidence that push-based delivery was always the intended direction — TermLink provides the transport that systemd.path couldn't deliver in a portable way.
