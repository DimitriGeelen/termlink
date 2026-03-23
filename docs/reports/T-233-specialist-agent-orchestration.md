# T-233: Specialist Agent Orchestration — Research Artifact

## Problem Statement

Today, a single Claude Code agent handles everything: research, coding, infrastructure, design, testing. This creates two problems:

1. **Context pollution** — a coding agent's context fills up with research findings, design exploration, and infra commands that dilute its core task
2. **No specialization** — each agent starts from zero; there's no way to pre-load domain context (e.g., "you're the infrastructure agent, here's what you know about our servers")

The vision: an **orchestrator agent** that recognizes "I need research" or "I need infrastructure work" and delegates to **specialist agents** that are pre-loaded with relevant context, running as TermLink sessions.

## What Exists Today

### TermLink primitives that could support this:
- **`termlink spawn`** — start a new session with name/roles/tags
- **`termlink agent ask`** — typed request-response between agents (ask/listen protocol)
- **`termlink interact`** — inject a command into a session and capture output
- **`termlink inject`** — send keystrokes to a session
- **`termlink mirror`** (NEW) — observe what an agent is doing
- **Hub** — central routing for multi-agent coordination
- **Events** — pub/sub for agent-to-agent signaling

### Framework primitives:
- **Sub-Agent Dispatch Protocol** (CLAUDE.md) — rules for using Claude Code's Task tool
- **`fw bus`** — result ledger for sub-agent outputs
- **Episodic memory** — completed task histories for context

## Dialogue Log

### 2026-03-23 D1 — Five inception questions

**Q1: Who spawns the specialists?**
- Human: Both persistent and on-demand are feasible. Needs multi-perspective research.
- Action: 3 research agents dispatched to evaluate different perspectives.

**Q2: How does the orchestrator know WHAT to delegate?**
- Human: Multiple discovery mechanisms worth exploring:
  - Interactive discovery (human directs)
  - Evaluation/parsing via PreToolUse hook (framework intercepts)
  - Reactive by instruction from agent or human
  - Domain-specific triggers (keywords, patterns)
  - Other patterns TBD
- Action: 5 research agents dispatched, one per mechanism.

**Q3: Communication pattern?**
- Human: Depends on Q1/Q2 outcomes. Deferred.

**Q4: Specialist context loading?**
- Human: All three approaches valid (CLAUDE.md, injected prompt, pre-loaded files). Key design:
  - **Static specialists**: Pre-built with codified context manifests (checked-in artifacts)
  - **Dynamic specialists**: Orchestrator assembles from a capability manifest (index of skills, tools, scripts, commands)
  - **Self-discovery feedback loop**: Specialist discovers it needs additional capabilities mid-task → signals back to orchestrator → orchestrator codifies into manifest for future use
  - **The manifest is the living brain**: Grows as specialists discover gaps. Orchestrator is custodian.
- Shared understanding: Confirmed.

**Q5: TermLink feature vs framework feature vs independent?**
- Human: Not sure, could be independent. Needs exploration.
- Action: 3 research agents dispatched to evaluate architectural ownership options.

## Research Results — Agent Reports

### Q1: Specialist Lifecycle (3 agents)
| Report | Position | Key Insight |
|--------|----------|-------------|
| [Q1-persistent](T-233-Q1-persistent.md) | FOR persistent | Amortized startup, state accumulation, idle cost minimal with TermLink |
| [Q1-ondemand](T-233-Q1-ondemand.md) | FOR on-demand | Fresh context, zero idle cost, aligns with existing dispatch protocol |
| [Q1-hybrid](T-233-Q1-hybrid.md) | Hybrid model | Hot/warm/cold tiers with promotion based on observed usage |

**Consensus:** Hybrid lifecycle with three tiers:
- **Hot** (persistent): >3 calls/session, expensive cold start, cross-call state needed
- **Warm** (standby): 1-3 calls/session, manifest-based fast spawn, idle-timeout
- **Cold** (on-demand): <1 call/session, cheap startup, stateless
- **Promotion rule:** Auto-promote after 3 sessions of consistent usage patterns

### Q2: Delegation Discovery (5 agents)
| Report | Mechanism | Confidence | Best For |
|--------|-----------|-----------|----------|
| [Q2-interactive](T-233-Q2-interactive.md) | @-mention, /delegate | High | Human-directed, explicit routing |
| [Q2-pretool](T-233-Q2-pretool.md) | PreToolUse hook | High | Deterministic pattern matching (ssh→infra) |
| [Q2-domain-triggers](T-233-Q2-domain-triggers.md) | File/tool/keyword patterns | Medium-High | Configurable automatic routing |
| [Q2-evaluation](T-233-Q2-evaluation.md) | Task AC parsing | Medium | Structured task decomposition |
| [Q2-reactive](T-233-Q2-reactive.md) | Agent self-assessment | Fallback | Escape valve when other mechanisms miss |

**Consensus:** Layer all five mechanisms with clear precedence:
1. @-mention / /delegate (explicit, no confirmation)
2. PreToolUse hook denial (structural, deterministic)
3. Domain trigger scoring (configurable, threshold-based)
4. Task evaluation (pre-dispatch decomposition)
5. Reactive self-assessment (agent-initiated fallback)

All produce a unified `specialist.request` event shape — only the `trigger` field differs.

### Q5: Architectural Ownership (3 agents, 1 pending)
| Report | Position | Key Insight |
|--------|----------|-------------|
| [Q5-termlink-feature](T-233-Q5-termlink-feature.md) | FOR TermLink native | 80% of primitives exist; `delegate` and `orchestrate` as composites |
| [Q5-independent](T-233-Q5-independent.md) | Right destination, wrong start | Premature abstraction; start embedded, extract after 20+ tasks |
| Q5-framework-feature | PENDING | Agent still running |

**Emerging consensus:** TermLink owns the **mechanism** (spawn, route, collect), framework owns the **policy** (which specialist, what context, how to merge). Start as TermLink feature with framework policy injection via config, extract to independent layer if/when portability is needed.

## Q4: Context Loading Design (from dialogue)

**Architecture confirmed with human:**
- **Static specialists**: Pre-built codified context manifests (checked-in YAML/MD artifacts)
- **Dynamic specialists**: Orchestrator assembles from capability manifest (index of skills, tools, scripts)
- **Self-discovery feedback loop**: Specialist signals orchestrator when it needs additional capabilities → orchestrator codifies into manifest
- **The manifest is the living brain**: Grows as specialists discover gaps

## Emerging Architecture Summary

```
┌─────────────────────────────────────────────┐
│  Human / Orchestrator Agent                  │
│  ┌─────────┐  ┌──────────┐  ┌────────────┐ │
│  │@mention  │  │/delegate │  │ NL routing  │ │
│  └────┬─────┘  └────┬─────┘  └─────┬──────┘ │
│       └──────────────┴──────────────┘        │
│                      │                        │
│          ┌───────────▼───────────┐            │
│          │  Routing Engine       │            │
│          │  (trigger scoring +   │            │
│          │   manifest lookup)    │            │
│          └───────────┬───────────┘            │
└──────────────────────┼────────────────────────┘
                       │
        ┌──────────────┼──────────────┐
        │              │              │
   ┌────▼────┐   ┌────▼────┐   ┌────▼────┐
   │ HOT     │   │ WARM    │   │ COLD    │
   │code-spec│   │test-spec│   │audit-sp │
   │(persist)│   │(standby)│   │(spawn)  │
   └─────────┘   └─────────┘   └─────────┘
        │              │              │
        └──────────────┼──────────────┘
                       │
              ┌────────▼────────┐
              │  fw bus / events │
              │  (result ledger) │
              └─────────────────┘
```

