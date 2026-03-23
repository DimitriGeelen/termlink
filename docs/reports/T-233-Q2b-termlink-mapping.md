# T-233 Q2b: Mapping Orchestrator-Specialist to TermLink Primitives

## Finding: TermLink Already Has Every Primitive Needed

The agent-orchestrator-specialist interaction pattern maps directly onto existing TermLink primitives with **zero new protocol work**. The hub is the natural orchestrator.

## Primitive Mapping

| Interaction Step | TermLink Primitive | How |
|---|---|---|
| Agent asks orchestrator | `agent.ask <hub> <action>` | Agent emits `agent.request` event to hub session |
| Orchestrator finds specialist | `session.discover --roles specialist --caps <needed>` | Hub queries its own session manager — no RPC needed |
| Orchestrator delegates | `event.emit` on specialist's bus | Hub emits `agent.request` with correlation `request_id` |
| Specialist responds | `agent.response` event | Hub polls specialist, relays back to requesting agent |
| Broadcast capability changes | `event.broadcast` topic `capability.update` | Specialists announce new/removed capabilities |
| Agent caches result | `kv.set` on agent's session | Local key-value store per session |

## The Hub as Orchestrator

The hub is the **only component that already knows all sessions**. It maintains the session manager with tags, roles, and capabilities queryable via `session.discover`. Making the hub the orchestrator avoids a separate "orchestrator session" — the routing logic lives where the registry already lives.

**What the hub already does:** Registration, discovery (AND-logic filtering by tags/roles/capabilities), event fan-out (`event.broadcast`), event aggregation (`event.collect`), session health (heartbeats, stale detection).

**What the hub would need:** A thin routing layer that receives `agent.request` events, runs `session.discover` to find a matching specialist, forwards the request, and relays the response. This is ~100 lines of Rust on top of existing primitives.

## Key Primitives in Detail

### Discovery as Capability Registry (No New Registry Needed)
`session.discover` already accepts `capabilities: [string]` filters. Specialists self-register capabilities at session creation via `SessionConfig.capabilities` and can update them at runtime via `session.update`. The hub's session manager IS the capability registry — `find_by_capability()`, `find_by_role()`, `find_by_tag()` are already implemented.

### Agent Request/Response Protocol (Already Specified)
The `agent.request`/`agent.response`/`agent.status` event schema (v1.0) already handles: correlation via `request_id`, timeout, progress reporting (`phase: accepted|running|finalizing`), error propagation. The CLI `agent ask` command implements the full request-poll-correlate cycle.

### Inject/Interact for Format Corrections
`command.inject` sends keystrokes to a PTY session — useful when a specialist is an interactive tool (editor, REPL) that needs format corrections mid-conversation. This is a niche case; most specialist interactions would use the structured `agent.request`/`agent.response` path.

## Architectural Recommendation

**The orchestrator should be a hub enhancement, not a separate session.** Reasons:

1. **Hub already has the registry** — no need to sync session state to another process
2. **Hub already handles fan-out** — `event.broadcast` and `event.collect` are hub-native
3. **Latency** — hub-local discovery is a function call, not an RPC round-trip
4. **Simplicity** — one fewer session to manage, monitor, and restart

The implementation is a new hub RPC method (e.g., `orchestrator.route`) that combines discover → delegate → relay into a single call from the agent's perspective.

## What `discover` Already Solves

`termlink discover --roles specialist --caps "build"` finds all sessions that declared themselves as build specialists. No orchestrator needed for simple capability lookup — the orchestrator adds value for **negotiation** (try specialist A, fall back to B) and **load balancing** (pick least-busy specialist via `query.status`).
