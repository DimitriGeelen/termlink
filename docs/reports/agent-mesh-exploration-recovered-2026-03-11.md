# Agent Mesh Exploration — Recovered Research
## Session: 2026-03-11 (content lost, recovered from next session)

> **Status:** Recovered content. No task was created during the original session.
> A follow-on inception task should be created to continue this work.
> Recovery date: 2026-03-11 | Triggered T-094.

---

## Core Vision

**What we have:**
- **TermLink**: Cross-terminal session communication. Sessions register on Unix sockets,
  exchange JSON-RPC commands, stream binary data, coordinate through a hub. 26 CLI commands,
  dual-plane architecture.
- **Agentic Engineering Framework**: Governance system for AI agents. Task tracking, context
  management, handovers, healing loops, episodic memory, component fabric.

**The Core Insight:** Right now, the framework runs agents as isolated Claude Code sessions —
each one is a single process talking to files on disk. There's no live communication between
agents. The handover system is "write a document, hope the next session reads it."

TermLink gives us **real-time inter-agent communication.**

---

## What This Enables

| Pain Point Today | TermLink Solution |
|---|---|
| Handover is lossy — compaction destroys working memory | Sessions persist independently of Claude Code context; store state in KV, retrieve it fresh |
| No agent coordination — specialists are independent watchers | Live fleet: coder emits `review.request`, reviewer picks it up, posts `review.complete` |
| No observability — can't see what an agent is doing | `termlink attach/stream` — watch any agent's terminal live |
| Sequential bottleneck — one agent at a time | Multiple agents work in parallel, coordinate through hub events |

---

## Naming Candidates

| Name | Vibe | Why it works | Why it might not |
|---|---|---|---|
| **Agent Mesh** | Network topology | Accurately describes peer connectivity | "Mesh" implies equal peers, but we want orchestration |
| **Agent Fabric** | Woven infrastructure | Matches "Component Fabric" naming | Collision with existing fabric concept |
| **Agent Bus** | Message passing | We already have `fw bus` | "Bus" feels like a single pipe, not a topology |
| **Agent Hive** | Collective intelligence | Evocative, memorable | Maybe too cute |
| **Agent Lattice** | Structured connections | Implies order within connectivity | Obscure |
| **Agent Chorus** | Coordinated voices | Each agent has a voice, they harmonize | Too metaphorical |
| **Agent Nexus** | Connection hub | Strong, clear | Implies centralization |
| **Agent Weave** | Interlocked threads | Each agent is a thread in the fabric | Novel, might confuse |
| **Agent Pulse** | Heartbeat/liveness | Emphasizes the "alive" nature | Too narrow |
| **Agent Span** | Distributed reach | Implies spanning machines/networks | Collision with tracing "span" |
| **Agent Ring** | Connected topology | Ring networks are resilient | Implies specific topology |
| **Agent Grid** | Distributed compute | Familiar concept | Overloaded term |
| **Agent Nerve** | Neural network | Signals between agents, fast reflexes | Might imply AI/ML specifically |
| **Agent Wire** | Direct connection | Simple, technical, accurate | Maybe too low-level |

**Top 3 candidates:**
1. **Agent Mesh** — strongest. A mesh can have coordinators. Captures exactly what we're
   building: agents that can find and talk to each other in a network topology.
2. **Agent Nexus** — if we lean into the orchestrator concept. Works even in distributed mode.
3. **Agent Hive** — if we want to emphasize collective intelligence and antifragility.
   A hive survives losing any individual. Workers are interchangeable.

**Human preference (from conversation):** Agent Mesh was the favorite. Agent Hive also liked.
Brainstorming was ongoing — no final decision made.

---

## Architectural Question 2: Agent Identity

**The core question:** Should each Claude Code session be one TermLink session, or should a
single session host multiple logical agents?

### Option A: One Claude Code session = One TermLink session
```
Terminal 1: claude → registers as "coder-01"    → 1 TermLink session
Terminal 2: claude → registers as "reviewer-01" → 1 TermLink session
Terminal 3: claude → registers as "tester-01"   → 1 TermLink session
```
**Pros:** Dead simple, easy to observe, natural cleanup on terminal close, matches mental model
**Cons:** 3 agents = 3x API cost, heavy (full Claude Code per agent), switching roles = new session

### Option B: One Claude Code session hosts multiple logical agents
One session uses the Task/Agent tool to spawn sub-agents, each with their own TermLink session.
**Pros:** Efficient (one API session, multiple TermLink identities), sub-agents share parent context
**Cons:** Sub-agents are ephemeral, can't have persistent agents waiting for events

### Option C: Hybrid — Core session + satellite sessions
```
Terminal 1: claude "orchestrator" (primary, human-interactive)
  ├── TermLink session: "orchestrator"
  ├── Spawns Terminal 2: claude --headless "reviewer"
  │   └── TermLink session: "reviewer" (persistent, event-driven)
  └── Spawns Terminal 3: claude --headless "tester"
      └── TermLink session: "tester" (persistent, event-driven)
```
**Pros:** Best of both worlds — persistent agents that can wait for events, clean separation
**Cons:** More complex lifecycle, multiple API sessions, need spawn/supervision mechanism

**Recommendation:** Start with Option A (simplest), design protocol for Option C.
The protocol shouldn't care — a TermLink session is a TermLink session.

---

## Architectural Question 3: Hub vs Peer-to-Peer

### Centralized Hub (current TermLink design)
All discovery and routing through hub. Simple, observable, central access control.
**Weakness:** Single point of failure, bottleneck at scale, needs network transport for cross-machine.

### Peer-to-Peer (direct discovery)
Agents discover via file-based registry, connect directly.
**Weakness:** Every agent needs discovery logic, no central observability, broadcast requires N connections.

### Hybrid: Hub-Assisted, Direct-Data
Hub for discovery + routing policy. Direct connections for data transfer and high-frequency events.
Analogous to DNS for discovery + direct connection for data.

**Recommendation:** Hub-primary with graceful degradation:
1. Hub is default — discovery, broadcast, collect
2. Direct connection as optimization — once discovered, connect directly for high-frequency streams
3. Hub-less fallback — file-based registry if hub is down (degraded mode: no broadcast, P2P still works)

This maps to the **antifragility directive** — reduced capacity on failure, not collapse.

---

## Architectural Question 4: Security Boundaries (Staggered)

### Phase 1: Same Machine, Same User (current)
- Security: UID check via SO_PEERCRED / LOCAL_PEERCRED
- Transport: Unix domain sockets
- Trust: Full — all agents are "me"

### Phase 2: Same Machine, Different Users
- Security: UID check + capability tokens (HMAC-SHA256)
- Transport: Unix domain sockets
- Trust: Scoped — tokens grant specific permissions
- The capability token system (T-086/087/088) already supports this

### Phase 3: Same Network, Different Machines
- Security: TLS + capability tokens
- Transport: TCP sockets (or WebSocket)
- Trust: Mutual TLS for machine identity, tokens for agent permissions
- Requires T-073 (transport abstraction) — swap Unix for TCP without changing protocol
- **VPN agent** can be used to test this phase

### Phase 4: Different Networks (NAT Traversal)
- Security: TLS + tokens + relay authentication
- Transport: WebSocket through relay, or STUN/TURN
- Options: Relay server, STUN/TURN hole-punching, Tailscale/ZeroTier overlay

**Key design principle:** Each phase is a pure transport layer change. The agent mesh protocol
stays identical. An agent shouldn't know whether its peer is on the same machine or across the ocean.

---

## Architectural Question 5: Bootstrap & Orchestration

### Bootstrap Sequence
```
fw context init
  ├── Start hub (if not running): termlink hub start
  ├── Register this session: termlink register --name "orchestrator" --role orchestrator
  ├── Discover existing agents: termlink discover
  └── If agents found: reconnect / if not: we're the first
```

Config approach (default on, switchable off):
```yaml
agent_mesh:
  enabled: true
  auto_start_hub: true
  auto_register: true
  role: orchestrator
```

### Spawn Modes
```bash
# Local spawn
fw mesh spawn --name "reviewer" --role reviewer

# Remote connect (edge session started independently)
fw mesh connect reviewer@192.168.1.50
```

### Orchestrator Election (Phase 3-4, antifragility feature)

**Why election matters:** Fixed orchestrator = single point of failure.
If orchestrator runs out of context or crashes, the fleet is headless.

**Election protocol:**
1. Every agent has an "orchestration score": context budget, time alive, role capability, human proximity
2. Current orchestrator heartbeats every 10s: `termlink emit hub "orchestrator.heartbeat"`
3. If heartbeat missing 30s → election triggers
4. All agents emit "orchestrator.candidate" with score
5. After 5s window, highest score wins, emits "orchestrator.elected"
6. Human override always available: `fw mesh elect reviewer-01`

This is Raft-family leader election simplified for our use case (leader election only, not full consensus).

**Phase 1:** Fixed orchestrator = the session where the human types.

---

## Phased Roadmap

| Phase | Name | What it delivers | Depends on |
|---|---|---|---|
| 0 | Foundation | Transport abstraction in TermLink (TCP alongside Unix sockets) | T-073 |
| 1 | Local Mesh | `fw mesh` commands, auto-registration, event coordination between local agents | Phase 0 |
| 2 | Live Delegation | Replace file-based dispatch with TermLink request/reply, shared KV working memory | Phase 1 |
| 3 | Remote Mesh | TLS transport, cross-machine agents, VPN testing | Phase 0 + security |
| 4 | Resilient Mesh | Orchestrator election, self-healing fleet, graceful degradation | Phase 2 + 3 |

---

## Open Questions (unanswered at session end)

1. **Naming** — Final decision between Agent Mesh, Agent Hive, and others pending
2. **Phase 0 first?** — Should T-073 (transport abstraction) be promoted from backlog as foundation?
3. **Orchestrator model** — Should Phase 1 orchestrator be the framework itself (`fw` commands
   route through hub) or a specific agent session (one Claude instance is "the boss")?
4. **Roadmap task type** — Inception (exploration/go-no-go) or straight to build task for Phase 1?
5. **Scope** — Confirmed: general capability any framework-using project could adopt (not just termlink)

---

## Dialogue Log

**Human:** Scope question answered — redesigning as a general capability any framework-using
project could adopt.

**Human on Hub vs P2P:** "Explain the concept, pros, contrast possibilities, think about what
we're trying to do."

**Human on Security:** "Start simple and expand. I can imagine agents working across different
machines, even different networks, with NAT traversal. We also have a VPN agent to test that."

**Human on Bootstrap:** "Option for default to start but also default to be switched off.
Main session should be able to spawn sessions. Remote sessions start up and then main session
connects to edge sessions."

**Human on Orchestration (key insight):** "Do we really want to make it distributed? That would
be more interesting. Not sure about that actually because we do want to have an orchestration
function. But in principle everybody could have the orchestrator role — if the orchestrator is
gone we can have an election for a new orchestrator. We don't need that from the beginning but
that's a very good antifragility feature to build in."

---

*This document was recovered in the following session after the original session ended without*
*capturing this content. This is the loss event that triggered T-094.*
