# T-233 Q5: The Case FOR Framework-Owned Orchestration

## Position

Specialist agent orchestration should be a **framework feature**, not a TermLink feature. TermLink provides transport; the framework provides governance. Orchestration is governance.

## Core Argument: The Framework Already Owns the Primitives

The Agentic Engineering Framework already has every building block orchestration needs:

1. **Task system** — Tasks are the unit of delegation. The framework knows task types, horizons, owners, acceptance criteria, and workflow state. An orchestrator that delegates "research X" is creating/assigning a task — that's `fw task create`, not a transport concern.

2. **Sub-Agent Dispatch Protocol** — CLAUDE.md already codifies rules for parallel vs. sequential dispatch, token budgets, result format (write-to-disk, return path+summary), and max concurrency (5 agents). This IS orchestration policy. It just lacks a runtime engine.

3. **`fw bus` result ledger** — Typed YAML envelopes with auto size-gating (<2KB inline, >=2KB blob). This is the result aggregation layer. Orchestration needs to read manifests, correlate results, and synthesize — all framework-level concerns.

4. **Episodic memory** — Specialist context loading (Q4) is about giving agents the right knowledge. The framework owns episodic summaries, learnings, patterns, and decisions. A "research specialist" needs `learnings.yaml` and `patterns.yaml`, not TCP frames.

5. **Context budget management** — The framework tracks token usage, enforces gates at 120K/150K/170K, and triggers handovers. Orchestration MUST respect these budgets when spawning sub-agents. Only the framework has this visibility.

## What TermLink Provides (and Should Only Provide)

TermLink's role is **transport and session lifecycle**:
- `spawn` — start a process in a named session
- `agent ask/listen` — typed message passing between sessions
- `inject/interact` — keystroke-level control
- `events` — pub/sub signaling
- Hub — routing

These are pipes. They don't know what flows through them, and they shouldn't. TermLink doesn't know what a "task" is, what "acceptance criteria" means, or when context budget is exhausted.

## Separation of Concerns: The Advantages

| Concern | Owner | Why |
|---------|-------|-----|
| "What to delegate" | Framework | Requires task classification, skill registry, workflow type knowledge |
| "Who to delegate to" | Framework | Requires capability manifests, specialist profiles, governance rules |
| "How to send the message" | TermLink | Transport: Unix sockets, JSON-RPC, binary frames |
| "How to track results" | Framework | `fw bus` already does this |
| "When to stop" | Framework | Context budget, AC verification, P-011 gate |

If orchestration lives in TermLink, TermLink must understand tasks, budgets, and governance — violating Directive 4 (Portability). TermLink becomes framework-coupled. If orchestration lives in the framework, TermLink remains a clean, reusable transport that any project can adopt.

## What the Framework Needs to Add

1. **Orchestrator agent** (`agents/orchestrator/`) — Reads task, selects dispatch strategy (parallel/sequential), spawns sub-agents via TermLink transport, aggregates results via `fw bus`.

2. **Specialist registry** — YAML manifest mapping specialist types to their context profiles (which CLAUDE.md sections, which episodic entries, which skills to pre-load).

3. **Dispatch engine** — Upgrades the current Sub-Agent Dispatch Protocol from "rules agents should follow" to "runtime the framework enforces" — like how P-011 turned verification from guidance into a gate.

4. **Transport adapter interface** — Abstract the "how to spawn and communicate" so the orchestrator works with TermLink today but could use Claude Code's Task tool, MCP, or any other transport tomorrow.

## The Litmus Test

Ask: "If we replaced TermLink with raw Unix pipes, would orchestration still work?"

If orchestration is a framework feature: **yes** — the framework dispatches tasks, tracks results, enforces budgets. Transport is pluggable.

If orchestration is a TermLink feature: **no** — you'd lose the orchestration layer when you swap transport.

The answer should be yes. Orchestration is framework. TermLink is transport.
