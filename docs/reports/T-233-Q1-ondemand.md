# T-233 Q1: The Case FOR On-Demand Specialist Agents

**Position:** Specialists should be spawned by the orchestrator when needed, not kept as persistent long-running sessions.

## Core Argument: Fresh Context Is a Feature, Not a Bug

The single biggest advantage of on-demand spawning is **zero context pollution**. A specialist agent starts with exactly the context it needs — no residual state from previous tasks, no accumulated prompt debris, no stale assumptions. In a system where context window is a finite, non-renewable resource (per CLAUDE.md P-009), starting clean is optimal resource allocation.

## Resource Efficiency

Persistent agents consume resources whether working or idle. In a typical session, the orchestrator might need a research agent for 10 minutes, then nothing for an hour, then an infrastructure agent for 5 minutes. On-demand spawning means:

- **Zero idle cost** — no sessions sitting in memory waiting for work
- **No context rot** — persistent agents accumulate stale state that degrades decision quality over time
- **Natural cleanup** — when a specialist completes, its resources are fully reclaimed
- **Scalable** — spawn 5 parallel researchers for a spike, then 0; no need to pre-allocate or manage a pool

## Simpler Lifecycle

Persistent agents require lifecycle management: health checks, restart logic, state recovery after crashes, graceful shutdown coordination. On-demand agents have a trivial lifecycle:

1. Orchestrator decides delegation is needed
2. Spawn specialist with task-specific context manifest
3. Specialist works, posts results to `fw bus`
4. Specialist exits — done

No heartbeat monitoring. No reconnection logic. No "is the research agent still alive?" checks. The orchestrator's job is simpler because it only tracks active work, not idle capacity.

## Context Loading Strategies (Addressing Startup Latency)

The main objection to on-demand is startup cost. Three mitigations make this manageable:

1. **Context manifests** — Pre-built YAML files listing exactly what a specialist type needs (relevant CLAUDE.md sections, fabric cards, episodic memories). Loading a curated 2KB manifest is faster than a persistent agent re-reading its full context after compaction.

2. **Warm templates** — TermLink's `spawn` command can accept a role definition and initial context injection. A "research-agent" template pre-loads research patterns; a "coder-agent" template pre-loads the component fabric. Template creation is a one-time cost.

3. **Amortized over task duration** — A specialist that runs for 5+ minutes amortizes a 10-second startup across the entire task. The latency only matters for sub-second delegation, which isn't the use case here.

## Alignment with Existing Architecture

The framework already follows this pattern. The Sub-Agent Dispatch Protocol in CLAUDE.md describes spawning fresh agents per task. The `fw bus` result ledger assumes agents that produce output and terminate. Episodic memory captures completed task histories — a natural fit for agents with clear start/end boundaries.

TermLink's `spawn` + `agent ask` primitives are designed for request-response patterns, not persistent connections. On-demand aligns with the existing primitive semantics.

## When On-Demand Breaks Down

Honest acknowledgment: on-demand is suboptimal when a specialist needs **accumulated session state** across multiple interactions (e.g., a debugging agent that builds understanding over repeated runs). This argues for a hybrid model where most specialists are on-demand but specific roles can be promoted to persistent when justified by usage patterns.

## Verdict

On-demand spawning should be the **default** for specialist agents. It maximizes resource efficiency, eliminates context rot, simplifies lifecycle management, and aligns with existing TermLink and framework primitives. Persistent specialists should be the exception, justified by demonstrated need for cross-interaction state accumulation.

---
*Research agent: Q1-ondemand | Task: T-233 | Date: 2026-03-23*
