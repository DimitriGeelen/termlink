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
| [Q5-framework-feature](T-233-Q5-framework-feature.md) | FOR framework ownership | Framework owns tasks, budgets, governance; TermLink is transport |

**Consensus:** Strong agreement across all three agents on the **separation of concerns**:
- **TermLink** owns the **mechanism**: spawn, route messages, collect results, session lifecycle
- **Framework** owns the **policy**: what to delegate, who to delegate to, when to stop, result aggregation
- **Start embedded in framework**, using TermLink as transport adapter. Extract to independent layer after 20+ real tasks prove the interfaces
- **Litmus test** (Q5-framework): "If we replaced TermLink with raw Unix pipes, would orchestration still work?" Answer must be yes → orchestration = framework, transport = pluggable

## Q1 Refined: Execution Model (from D1 dialogue)

The three-tier lifecycle model (hot/warm/cold) was rejected as answering the wrong question. The real question is about **execution governance**, not resource management.

### Deterministic-First with Stochastic Fallback
1. Work is executed by **deterministic capabilities** (scripts, skills, tools) whenever possible
2. When deterministic path **fails**, a stochastic agent remediates
3. Remediation gets **codified back** into deterministic path (antifragility)
4. System matures over time: less stochastic, more deterministic

### Supervision Model
- Not a run counter — a **qualitative risk assessment** every time
- Three axes: script maturity, context familiarity, blast radius
- Script promoted to new project → context resets, maturity carries
- A script that has **failed and recovered** is MORE trustworthy than one with perfect record (antifragility)

### Supervision Ramp-Down (from dialogue)
Human challenged the "3-5 runs then autonomous" model:
- A script that runs 3 times in one project may get promoted as a skill to another project
- Previous successes in Project A tell you NOTHING about Project B (different env, deps, OS)
- The ramp-down is NOT a counter — it's contextual and qualitative
- **Script maturity** (error handling breadth) travels with the script
- **Context familiarity** (run history in THIS environment) resets on promotion
- **Blast radius** (what happens if it fails) may differ per project

### Unsolved Design Problem: Script Error Yielding
How does a deterministic script yield errors to a stochastic agent WITHOUT crashing?
- Today: binary (exit 0 or non-zero) — script succeeds or fails entirely
- Desired: script yields mid-execution errors to a supervising agent that can remediate, then script continues
- Options explored:
  1. **Checkpoint-based execution** — script runs in steps, supervisor retries failed steps
  2. **Error stream + remediation loop** — stderr piped to agent, agent injects fixes
  3. **TermLink as bridge** — script in TermLink session, errors as events, agent remediates via inject
- Option 3 fits existing primitives but design is incomplete
- Captured as open question for further exploration

### Evidence Assessment (6 agents)
- Enforcement Tiers: **WORKING** (3 real blocks logged, 538 commits survived) — use as supervision foundation
- Healing Loop: **DORMANT** (0 invocations in 210 tasks) — data structures exist, loop never exercised
- Component Fabric: **REGISTRATION strong, ANALYSIS dormant** (65 cards, 0 blast-radius runs)
- Recommendation: Build supervision on tiers (proven). Use healing/fabric as enrichment data only.

## Q2 Refined: Capability Discovery (from D1 dialogue)

Original Q2 ("how does orchestrator know what to delegate?") reframed:
- Not "which agent to route to" but "which capability to invoke"
- The discovery is a **dialogue pattern** between agent, orchestrator, and specialist:

```
Agent needs to do X
  ├─ PRE-CHECK: Local template cache → known? → execute directly
  ├─ CACHE MISS: Ask orchestrator → "here's the specialist + request format"
  ├─ NEGOTIATION: Agent ↔ specialist back-and-forth (2-3 rounds)
  ├─ CACHE UPDATE: Agent saves template locally for next time
  └─ BYPASS: Orchestrator says "do it locally, no specialist needed"
```

Progressive autonomy: first time = full round-trip, second time = direct, eventually = local bypass.

### Q2b: Detailed Discovery Design (5 agents)

| Report | Aspect | Key Finding |
|--------|--------|-------------|
| [Q2b-routing-decision](T-233-Q2b-routing-decision.md) | Pre-execution routing | 3-way branch: bypass registry → route cache → orchestrator. Cache is YAML with confidence scores, TTL, hit counts. Partial match triggers refinement query. |
| [Q2b-negotiation-protocol](T-233-Q2b-negotiation-protocol.md) | Agent↔specialist dialogue | 4-phase protocol: introduce → attempt → correct → accept. Orchestrator brokers intro then steps back. Max 5 rounds. JSON Schema as wire format. |
| [Q2b-template-caching](T-233-Q2b-template-caching.md) | Progressive learning | 3-layer cache: agent-local → shared registry → specialist canonical. Promotion at 5 uses/0 corrections. Lazy invalidation via schema hash. Pull-on-miss, not push. |
| [Q2b-bypass-mechanism](T-233-Q2b-bypass-mechanism.md) | Local execution bypass | Bypass = Tier 3 operationalized. Registry of safe commands. Commands earn bypass via track record. Agents cannot self-promote. Failed bypass → de-promoted. |
| [Q2b-termlink-mapping](T-233-Q2b-termlink-mapping.md) | TermLink primitives | Every primitive exists. Hub IS the capability registry. Orchestrator = hub enhancement (~100 LOC). New RPC: `orchestrator.route`. |

### Full Discovery Flow (Refined)
```
Agent needs X
  ├─ Bypass registry (Tier 3)? → local execution, no orchestration
  ├─ Route cache hit (confidence ≥ 0.8, not expired)? → direct to specialist
  ├─ Partial cache match? → refinement query to orchestrator
  ├─ Cache miss → orchestrator.route (hub RPC)
  │    ├─ Hub discovers specialist via session.discover
  │    ├─ Introduces agent ↔ specialist (negotiate.offer)
  │    ├─ Agent ↔ specialist negotiate format (2-3 rounds, direct)
  │    ├─ Agent caches route + template (Layer 1)
  │    └─ After 5 uses, 0 corrections → promote to Layer 2 (shared)
  └─ Specialist fails → stochastic agent fallback (Q1 supervision model)
```

## Q4: Context Loading Design (from dialogue)

**Architecture confirmed with human:**
- **Static specialists**: Pre-built codified context manifests (checked-in YAML/MD artifacts)
- **Dynamic specialists**: Orchestrator assembles from capability manifest (index of skills, tools, scripts)
- **Self-discovery feedback loop**: Specialist signals orchestrator when it needs additional capabilities → orchestrator codifies into manifest
- **The manifest is the living brain**: Grows as specialists discover gaps

## Emerging Architecture Summary (Refined)

```
┌────────────────────────────────────────────────────────────┐
│  Agent needs to do X                                        │
│                                                             │
│  ┌─────────────┐   ┌──────────────┐   ┌─────────────────┐ │
│  │ Bypass       │   │ Route Cache  │   │ Orchestrator    │ │
│  │ Registry     │──►│ (.cache/     │──►│ (Hub RPC:       │ │
│  │ (Tier 3)     │   │  routes/)    │   │  orchestrator   │ │
│  │              │   │              │   │  .route)        │ │
│  └──────┬───────┘   └──────┬───────┘   └───────┬─────────┘ │
│    local exec         direct to            discover +       │
│                       specialist           introduce        │
└─────────┼──────────────┼────────────────────┼──────────────┘
          │              │                    │
          │              │         ┌──────────▼──────────┐
          │              │         │  Negotiation        │
          │              │         │  Agent ↔ Specialist  │
          │              │         │  (2-3 rounds)       │
          │              │         └──────────┬──────────┘
          │              │                    │
          ▼              ▼                    ▼
┌─────────────────────────────────────────────────────────────┐
│  EXECUTION (Deterministic-First)                             │
│                                                              │
│  ┌──────────────────┐       ┌──────────────────────────┐    │
│  │ Script/Skill/Tool │──OK──►│ Result → fw bus / events │    │
│  │ (deterministic)   │       └──────────────────────────┘    │
│  │                   │                                       │
│  │                   │──FAIL──► ┌────────────────────────┐   │
│  └───────────────────┘          │ Stochastic Agent       │   │
│                                 │ (diagnose + remediate) │   │
│                    ┌────────────┤                        │   │
│                    │            └────────────────────────┘   │
│               RISK CHECK                                     │
│            ┌───────┴───────┐                                 │
│            │ Low risk      │ High risk                       │
│            │ Auto-fix      │ Human approval                  │
│            └───────────────┘                                 │
│                                                              │
│  SUPERVISION: f(tier, script_maturity, context_familiarity)  │
│  Trust ledger: Enforcement Tiers (proven) + Fabric cards     │
└──────────────────────────────────────────────────────────────┘
          │
          ▼
┌──────────────────────────────────────┐
│  CODIFICATION (Antifragility Loop)   │
│  Stochastic fix → new/improved       │
│  script → deterministic next time    │
│  Template cache updated              │
│  Manifest grows                      │
└──────────────────────────────────────┘
```

## Research File Index

Total: 23 research files produced by 22 mesh agents + 1 main artifact.

### Round 1: Initial 5 Questions (11 agents)
- Q1: [persistent](T-233-Q1-persistent.md), [ondemand](T-233-Q1-ondemand.md), [hybrid](T-233-Q1-hybrid.md)
- Q2: [interactive](T-233-Q2-interactive.md), [pretool](T-233-Q2-pretool.md), [reactive](T-233-Q2-reactive.md), [domain-triggers](T-233-Q2-domain-triggers.md), [evaluation](T-233-Q2-evaluation.md)
- Q5: [termlink-feature](T-233-Q5-termlink-feature.md), [framework-feature](T-233-Q5-framework-feature.md), [independent](T-233-Q5-independent.md)

### Round 2: Supervision Design + Evidence (6 agents)
- Design: [tiers-as-supervision](T-233-Q1b-tiers-as-supervision.md), [healing-as-supervision](T-233-Q1b-healing-as-supervision.md), [fabric-as-trust](T-233-Q1b-fabric-as-trust.md)
- Evidence: [evidence-tiers](T-233-Q1b-evidence-tiers.md), [evidence-healing](T-233-Q1b-evidence-healing.md), [evidence-fabric](T-233-Q1b-evidence-fabric.md)

### Round 3: Discovery Refinement (5 agents)
- [routing-decision](T-233-Q2b-routing-decision.md), [negotiation-protocol](T-233-Q2b-negotiation-protocol.md), [template-caching](T-233-Q2b-template-caching.md), [bypass-mechanism](T-233-Q2b-bypass-mechanism.md), [termlink-mapping](T-233-Q2b-termlink-mapping.md)

