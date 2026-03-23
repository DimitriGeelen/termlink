# T-240: Negotiation Protocol Inception

## Problem Statement

Is a formal 4-phase negotiation protocol (offer → attempt → correction → accept) needed for agent-specialist collaboration, or are simpler alternatives sufficient?

## Research Questions

### Q1: Evidence of Need — Format Correction in Task History
How often have agent outputs been corrected/reformatted? What kinds of corrections?

### Q2: Feasibility — Multi-Round Exchange on Existing Primitives
Can `agent ask`/`agent listen` support correlated multi-round negotiation without new RPCs?

### Q3: Alternatives Assessment
Compare 4-phase negotiation vs simpler approaches.

## Findings

### Q1 Findings — Zero Iterative Correction Instances
- 9 format-adjacent bugs across 233 completed tasks (field mismatches, serialization, schema gaps)
- **All were one-shot fixes** — no iterative correction rounds found
- Most impactful: T-128 (inconsistent worker output) was solved by prompt templates, not negotiation
- Silent failures are the real cost (T-177 showed 0, T-248 lost data)
- Format problem **scales with agent count** — prompt templates work for 1-2 agents, not for dynamic specialist discovery

### Q2 Findings — Feasible Without New RPCs
- `agent.request`/`agent.response`/`agent.status` all share `request_id` for correlation
- `params` and `result` are untyped JSON — can carry negotiation payloads
- `phase` field is free-form String — supports custom values like "negotiating"
- **Limitation:** `agent ask` is single-shot (exits on first response); multi-round needs raw `emit`+`poll`
- **Limitation:** `agent listen` is read-only — no built-in respond command
- Verdict: Protocol supports it fully. CLI needs ergonomic improvements for multi-round.

### Q3 Findings — Layered Enforcement Beats Monolithic Protocol
Four approaches compared:
| | Complexity | Reliability | Best for |
|---|---|---|---|
| 4-phase negotiation | High | ~99% | Complex/semantic issues |
| Schema-in-prompt | Low | ~85-90% | Stable formats |
| Validate-and-retry | Low-mod | ~90-95% | Rare failures |
| No enforcement | Zero | ~60-70% | Prototyping |

**Hybrid strategy recommended:**
1. Schema-in-prompt (default) — catches 85-90% at zero latency
2. Validate-and-retry (1 retry) — catches another 5-8%
3. 4-phase negotiation — only for the 2-5% that need iterative dialogue

Build Layer 1 first, track failure rates, add layers as evidence warrants.

## Synthesis

### Decision: NO-GO on 4-phase negotiation as standalone build

**Evidence:**
- A1 (iterative correction needed): **DISPROVED** — zero instances of multi-round correction in 233 tasks
- A2 (JSON Schema as wire format): **VALID** — works for structural validation
- A3 (existing primitives sufficient): **CONFIRMED** — no new RPCs needed, just convention
- A4 (5-round cap sufficient): **UNTESTABLE** — no real negotiation data exists
- A5 (schema caching valuable): **PREMATURE** — no cache-worthy schemas exist yet

**The format problem exists but is solved more cheaply:**
- Schema-in-prompt (Layer 1) delivers 85-90% coverage at ~5% complexity
- The T-233 protocol design is sound but premature — save it for when evidence shows Layers 1+2 are insufficient
- Signal to build: >10% format failure rate after schema-in-prompt + one retry

### Concrete Output
- **NO-GO** on T-240 (4-phase negotiation protocol build)
- **Recommend:** New task for schema-in-prompt convention (Layer 1) to add to T-257 dispatch convention
- **Archive:** T-233 Q2b protocol design remains valid for future Layer 3 if needed
