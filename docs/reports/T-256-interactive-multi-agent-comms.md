# T-256: Interactive Multi-Agent Communication Research

## Problem Statement

Current multi-agent dispatch is fire-and-forget: orchestrator spawns workers, workers write files, orchestrator polls or waits blindly. TermLink already provides bidirectional messaging primitives (send, events, topics, watch, wait). The question is how to wire these into the dispatch workflow so spawned agents can talk back to the spawning agent in real-time — no polling.

## Research Questions

### Q1: TermLink Messaging Primitives Inventory
What existing TermLink CLI commands and protocol-level RPCs support bidirectional, real-time communication between sessions? What gaps exist?

### Q2: Current Dispatch Architecture
How does dispatch.sh currently spawn workers, pass context, and collect results? What would need to change to support push-based result delivery?

### Q3: Claude Code Execution Model Constraints
What does Claude Code's tool execution model allow for blocking/async communication? Can an agent block on `termlink wait`? How do background tasks interact with the foreground agent?

## Research Findings

Full reports: `T-256-q1-primitives.md`, `T-256-q2-dispatch.md`, `T-256-q3-execution-model.md`

### Q1: Primitives — Strong Foundation, One Key Gap
- TermLink has 15+ messaging primitives: emit, broadcast, watch, wait, collect, request, agent ask/listen, send, stream
- All event consumption is **poll-based** (250-500ms intervals). No true push subscription.
- **Critical gap (G1/G5):** Workers can only emit to their *own* event bus. No `emit-to <target>` RPC. Orchestrator must poll via `event.collect` to discover results.
- **Workaround exists:** Workers emit to self, orchestrator uses `collect` (hub fan-in) — works but adds latency = poll interval.
- The `task.*` topic schema (delegate/accepted/progress/completed/failed) is already defined in the protocol but has no CLI wrapper.
- `agent ask`/`agent listen` provides typed request-reply with progress updates.
- Ring buffer overflow risk (1024 events) under high fan-in.

### Q2: Dispatch — File-Based Pull Model
- Current flow: Claude Code `Task` tool → sub-agent writes files → `fw bus post` envelope → orchestrator reads
- No worker pool, no daemon, no process manager — sub-agents are ephemeral Task tool invocations
- `fw bus` provides structured result ledger (YAML envelopes, size gating, R-NNN IDs)
- Dormant `bus-handler.sh` inbox was designed for push-based delivery but never activated
- Pain points: no real-time notification, no streaming progress, no cross-machine dispatch
- Migration path: TermLink adds push transport layer while preserving bus envelope schema

### Q3: Execution Model — Wait > Poll by 10x
- Claude Code Bash tool supports `run_in_background` — zero context cost while waiting
- `termlink event collect --count 3 --timeout 300` as background task: 2 tool calls, ~800 tokens
- Polling equivalent: 36 tool calls, ~10K-18K tokens (7-12% of budget wasted)
- 10-minute hard ceiling on Bash timeout (600,000ms)
- Agent tool sub-agents can't access parent's TermLink session — prefer `termlink spawn`
- Recommended pattern: spawn via TermLink, collect via background `event collect`

## Dialogue Log

### 2026-03-23 — User initiates T-256
- **User:** "the spawned agent can talk to the spawning agent" — no polling should be necessary
- **Course correction:** User insisted on TermLink mesh agents (not Claude Code Agent tool) for research dispatch
- **Learning:** `termlink spawn --backend background` doesn't work for Claude instances (no real terminal); `--backend tmux` with `--permission-mode bypassPermissions` works

## Synthesis

### The Core Problem
Workers can emit events to their *own* bus, but there's no direct `emit-to <orchestrator>` path. The orchestrator must always poll (via `collect`) to discover worker results. This is the single structural gap.

### Two Possible Solutions

**Option A: `emit-to` RPC (new protocol feature)**
- Add a `target` parameter to `event.emit` so workers push directly to orchestrator's event bus
- Orchestrator uses `termlink event wait` — true blocking, no polling
- Requires: protocol change + session handler change + CLI change
- Cleanest solution but highest implementation cost

**Option B: Collect-based fan-in (no code changes needed)**
- Workers emit `task.completed` to their own bus
- Orchestrator runs `termlink event collect --topic task.completed --count N --timeout T` in background
- Hub does the polling (500ms interval) — orchestrator sees a single blocking call
- Already works today. Latency = 500ms (not visible to orchestrator).

### Recommended Path
**Option B is already viable** — the hub abstracts the polling away. The orchestrator experience is: spawn workers → background collect → get notified. The 500ms hub-level poll interval is invisible at the agent level.

**Option A is the ideal target** for a future build — true push eliminates even hub-level polling. But Option B can ship immediately with just a dispatch convention change.

### Minimal Deliverable
1. **Convention:** Workers emit `task.completed` event with bus envelope payload on completion
2. **Convention:** Workers emit `task.progress` events during work
3. **Orchestrator pattern:** `termlink event collect --count N --timeout T` as background Bash
4. **dispatch.sh / fw dispatch:** Inject parent session ID + topic convention into worker env
5. **Optional CLI:** `termlink task dispatch` wrapping the full cycle
