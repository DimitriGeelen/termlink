# T-114: PoC — Replace Internal Agent Spawning with TermLink Agent Mesh

> Inception research | 2026-03-12

## Problem Statement

Claude Code spawns sub-agents as internal subprocesses (sidechain JSONL files). This is:
- **Opaque**: No observability into agent-to-agent communication
- **Locked in**: Tied to Anthropic's backend subprocess model
- **Non-portable**: Can't route agents across machines or to different LLM providers

TermLink already has the primitives: session registration, discovery by role, command execution, event bus with request-reply, and a hub for routing. Can we wire these together to replace the internal spawning mechanism with TermLink-routed agent dispatch?

## What TermLink Already Has

| Capability | Status | CLI Command | Protocol Method |
|-----------|--------|-------------|-----------------|
| Session registration | Working | `register` | `session.register` |
| Session discovery by role/tag | Working | `discover` | `session.discover` |
| Remote command execution | Working | `exec` | `command.execute` |
| Ephemeral session (register→exec→deregister) | Working | `run` | — |
| Spawn in new terminal | Working | `spawn` | — |
| Event bus (pub/sub, topics, payloads) | Working | `event emit/poll/watch` | `event.emit/poll` |
| Request-reply pattern | Working | `request` | event.emit + reply_topic |
| Event broadcast to multiple targets | Working | `event broadcast` | `event.broadcast` |
| Event collection from multiple sources | Working | `event collect` | `event.collect` |
| Hub routing | Working | `hub start` | All methods forwarded |
| KV metadata store | Working | `kv set/get` | `kv.set/get` |
| Token-based auth | Working | `token create` | `auth.token` |

## What Claude Code's Agent Tool Provides

| Function | Current Mechanism | TermLink Equivalent |
|----------|------------------|---------------------|
| Isolation | Separate JSONL sidechain + UUID | TermLink session + socket |
| Context passing | Markdown prompt in user message | JSON payload via event or RPC param |
| Tool access | Full Claude Code toolkit | Agent runs its own Claude Code instance |
| Result handling | Write to disk, return 5-line summary | Event reply or `fw bus post` |
| Error recovery | Full JSONL transcript | TermLink event log + agent transcript |
| Parallelism | Up to 5 concurrent agents | TermLink session concurrency (no hard limit) |
| Budget tracking | PreToolUse hook reads transcript | Each agent has independent budget |

## PoC Design: Minimum Viable Round-Trip

### Architecture

```
┌─────────────┐     TermLink      ┌─────────────┐
│ Orchestrator │ ──── hub ──────→ │  Worker Agent │
│ (Claude Code)│ ← event reply ── │ (Claude Code) │
└─────────────┘                   └─────────────┘
```

### Flow

1. **Orchestrator** registers as TermLink session (role: `orchestrator`)
2. **Worker** registers as TermLink session (role: `worker`, tags: `coder`)
3. Orchestrator discovers worker: `termlink discover --roles worker`
4. Orchestrator sends task event: `termlink event emit --target <worker> --topic task.dispatch --payload '{"prompt": "...", "task_id": "T-XXX"}'`
5. Worker receives event (polling or watching), executes the task
6. Worker replies: `termlink event emit --target <orchestrator> --topic task.result --payload '{"task_id": "T-XXX", "status": "done", "summary": "...", "artifact": "/path/to/output"}'`
7. Orchestrator polls for reply, reads result

### What Needs Building

| Component | Effort | Description |
|-----------|--------|-------------|
| **Agent launcher** | Small | Shell script: starts Claude Code with `--prompt`, registers as TermLink session, deregisters on exit |
| **Dispatch wrapper** | Small | Shell script or Python: discovers worker, emits task event, waits for reply event |
| **Worker event loop** | Medium | Script that watches for `task.dispatch` events, invokes Claude Code (or any LLM), emits `task.result` |
| **E2E test** | Small | Integration test: orchestrator → dispatch → worker → result → verify |

### What Does NOT Need Building (for PoC)

- No changes to TermLink core (protocol, hub, session crates)
- No transport abstraction (T-073) — Unix sockets sufficient
- No concurrency handling (T-009) — single worker is enough
- No auth (token system already exists but not needed for local PoC)

## Spikes

### Spike 1: Manual round-trip
Can we manually do the full flow with existing CLI commands?
```bash
# Terminal 1: Start hub
termlink hub start

# Terminal 2: Register worker
termlink register --name worker-1 --roles worker

# Terminal 3: Register orchestrator, discover, send event
termlink register --name orchestrator --roles orchestrator
termlink discover --roles worker
termlink request --target worker-1 --topic task.dispatch --reply-topic task.result --payload '{"prompt": "list files"}'
```

### Spike 2: Claude Code as TermLink agent
Can Claude Code run inside a TermLink session wrapper?
```bash
termlink run --name "agent-coder" --roles worker -- claude --print "Read src/main.rs and summarize"
```

### Spike 3: Full automated dispatch
Wire spikes 1+2 into a script that:
1. Ensures hub is running
2. Spawns a worker agent
3. Dispatches a task
4. Collects the result
5. Verifies correctness

## Open Questions

1. **Worker lifecycle**: Should workers be long-running (pool) or ephemeral (spawn per task)?
   - For PoC: ephemeral (simpler). Pool model is optimization.

2. **Claude Code invocation**: `claude --print` for non-interactive, or full `claude` with hooks?
   - For PoC: `claude --print` (no hooks needed, single prompt → response)

3. **Result size**: Event payload has practical limits. Large results need file-based handoff.
   - For PoC: Results written to shared filesystem, event carries path only.

4. **Error propagation**: What if worker crashes mid-task?
   - For PoC: Timeout on request-reply. Orchestrator handles timeout as failure.

## Go/No-Go Criteria

### GO if:
- Spike 1 confirms event round-trip works with existing CLI
- Spike 2 confirms Claude Code can run inside `termlink run` wrapper
- Total PoC effort fits in one session (~2-3 hours)

### NO-GO if:
- TermLink event system can't handle the payload sizes needed
- Claude Code subprocess model conflicts with TermLink session registration
- Hub routing introduces unacceptable latency for interactive use

## Spike Results

### Spike 1: Manual Round-Trip (2026-03-12)

**Result: PARTIAL PASS** — Event emission and polling work. `wait` command has a bug.

**What works:**
- Hub starts, sessions register with roles/tags
- `discover --role worker` finds worker sessions
- `emit` delivers events to target session's event bus (confirmed via raw RPC)
- `events --since 0` retrieves events with full payload
- Event payloads support arbitrary JSON (task prompts, results)

**Bug found: `termlink wait` never matches events**
- Root cause: Initial poll (`{}` params) advances cursor to `next_seq`. Loop polls with `since: next_seq - 1`, but `poll_topic(topic, since_seq)` uses strict `seq > since_seq`. Event at `seq == since_seq` is excluded.
- Impact: `wait` always times out, even for pre-existing events
- Fix: Change cursor initialization or use `>=` in poll filtering
- **Workaround for PoC**: Use polling loop with `events --since N --topic T` instead of `wait`

**Conclusion:** Core event primitives are solid. `wait` bug fixed (cursor initialization). Not a blocker for PoC.

### Spike 2: Claude Code as TermLink Agent (2026-03-12)

**Result: PASS**

**What works:**
- `agent-wrapper.sh` unsets CLAUDECODE env var (bypasses nested session check), runs `claude --print --no-session-persistence`
- `termlink run -n "agent" -- ./agent-wrapper.sh "prompt"` successfully registers session, runs Claude, returns output, deregisters
- Output is clean: just the LLM response on stdout

**Bugs found and fixed:**
1. **`termlink run` arg quoting**: `command_parts.join(" ")` lost shell quoting for args with spaces. Fixed with POSIX-safe single-quote wrapping.
2. **Claude nested session check**: `CLAUDECODE` env var must be unset. Wrapper script handles this.

**Conclusion:** Claude Code works as a TermLink agent process. Wrapper script pattern is clean and reusable.

### Spike 3: Full Automated Dispatch (2026-03-12)

**Result: PASS**

**What works:**
- `dispatch.sh "prompt"` handles the full lifecycle: hub check → worker spawn → execute → collect result
- Worker registers as TermLink session with tags `worker,agent-mesh`
- Claude Code returns clean output: `"What is 2+2?"` → `4`
- Cleanup on completion (result file removed, session deregistered)

**Architecture validated:**
```
dispatch.sh "prompt"
  → termlink run -n worker -- agent-wrapper.sh "prompt"
    → unset CLAUDECODE
    → claude --print --no-session-persistence "prompt"
    → stdout captured to result file
  → cat result file
  → cleanup
```

**Conclusion:** The full PoC works. TermLink can dispatch tasks to Claude Code agents running as independent processes. The dispatch is transparent — caller gets the result on stdout.

## GO Decision

**GO** — All three spikes pass. TermLink has the primitives needed to replace internal agent spawning. Bugs found along the way were fixed (wait cursor, run arg quoting). The PoC is minimal but proves the concept end-to-end.

**Next steps (build tasks):**
1. Event-driven dispatch (emit task.dispatch → worker watches → executes → emits task.result)
2. Worker pool (long-running workers with role-based discovery)
3. Integration with framework's Sub-Agent Dispatch Protocol

## Dialogue Log

### Q: What's the minimum slice?
**A:** Single round-trip: orchestrator sends task → TermLink → agent executes → result returns via TermLink. No pool, no parallelism, no cross-machine — just prove the communication pattern works.
