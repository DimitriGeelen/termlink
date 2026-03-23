# T-233 Q5: Should Specialist Orchestration Be a Native TermLink Feature?

**Position:** FOR — TermLink should absorb orchestration as a first-class capability.

## The Argument: TermLink Already Has 80% of the Stack

TermLink's existing primitives map directly to orchestration concerns:

| Orchestration Need | TermLink Primitive Today |
|---|---|
| Spawn specialist | `termlink spawn` (Terminal.app, tmux, background PTY) |
| Route request to specialist | `termlink agent ask` (typed request-response with timeout) |
| Wait for completion | `termlink agent ask` (status polling built in) |
| Fan-out to N specialists | `termlink event broadcast` + `termlink event collect` |
| Discover available specialists | `termlink discover --role specialist --tag rust` |
| Monitor progress | `termlink event watch --topic agent.*` |
| Cross-machine dispatch | `termlink remote exec` via TCP hub |
| Capability gating | `termlink token create --scope agent.ask` |

The missing 20% is **orchestration logic**: manifest parsing, routing decisions, context loading, and result aggregation. These are thin layers over existing transport.

## Advantages of Tight Integration

**1. Zero-serialization dispatch.** An orchestrator built on TermLink uses the same Unix socket / binary frame transport that sessions already speak. No HTTP overhead, no JSON-RPC-over-HTTP wrapper, no second IPC layer. Dispatch latency drops to ~1ms per specialist.

**2. Topology awareness.** TermLink's hub already knows which sessions exist, their roles, tags, and capabilities. An orchestrator that queries `discover` can make routing decisions based on live topology — not a static manifest. A specialist that crashes is immediately visible via the hub's session lifecycle events.

**3. Event-native progress tracking.** The `event` subsystem already supports topics, sequence numbers, fan-in collection, and real-time watching. Orchestration progress (queued → running → completed) maps naturally to event topics. No need to invent a separate status bus.

**4. Cross-machine for free.** TermLink's `remote` subsystem already tunnels commands over authenticated TCP. An orchestrator built on TermLink primitives gets multi-machine dispatch without additional networking code. The hub on machine A can route to a specialist on machine B using the same `agent ask` protocol.

**5. Unified CLI UX.** Users already know `termlink spawn`, `termlink agent ask`, `termlink discover`. Adding `termlink orchestrate` or `termlink delegate` follows the existing mental model. Compare this to a separate `fw mesh dispatch` command that shells out to TermLink underneath — an unnecessary indirection layer.

## Proposed Surface Area

Two new subcommands, not a new subsystem:

- **`termlink delegate`** — Send a task to a specialist (spawn-if-needed + agent ask + collect result). Thin wrapper over existing primitives with manifest-based routing.
- **`termlink orchestrate`** — Fan-out a task to N specialists with a merge strategy. Built on broadcast + collect + result aggregation.

Both compose existing primitives. No new protocol messages, no new IPC mechanisms.

## Risks of Scope Creep

**1. Orchestration policy is opinionated.** "Which specialist handles this task?" is a policy decision. TermLink has been deliberately policy-free — it provides transport, not opinions. Adding routing logic means TermLink must understand manifests, agent capabilities, and fallback strategies. This is a genuine category shift.

**2. Framework coupling.** If `termlink delegate` assumes the agentic framework's task format, manifest schema, or directory layout, TermLink loses portability. Mitigation: keep the interface generic (JSON payload in, JSON result out) and let the framework layer provide the policy via configuration.

**3. Testing surface.** TermLink's test suite is already substantial (~40 integration tests touching hub, sessions, events). Orchestration adds stateful multi-agent scenarios that are harder to test deterministically.

**4. Maintenance burden.** Two more commands to maintain, document, and keep backward-compatible.

## Verdict

The advantages outweigh the risks **if** the implementation stays thin: `delegate` and `orchestrate` as composites of existing primitives, with routing policy injected via configuration rather than hardcoded. TermLink should own the *mechanism* of orchestration (spawn, route, collect, report) while the framework owns the *policy* (which specialist, what context, how to merge).

The alternative — a separate orchestration layer that shells out to TermLink — duplicates session management, event watching, and error handling that TermLink already does well. That's the stronger argument: not "TermLink should do orchestration" but "nobody else should reimplement TermLink's transport just to add a routing table on top."
