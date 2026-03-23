# T-240 Q3: Alternatives to 4-Phase Negotiation Protocol

## Approaches Compared

### A) 4-Phase Negotiation (T-233 Proposal)

**How it works:** Orchestrator sends `negotiate.offer` with schema + example + semantic constraints. Agent attempts compliance. Specialist returns surgical corrections (`{field, expected, got, hint}`). Iterate up to 5 rounds until `negotiate.accept`.

| Dimension | Assessment |
|-----------|------------|
| **Complexity** | High. 4 new message types, correction/acceptance state machine, timeout handling, round counting, schema caching. Requires specialist agents to implement correction logic (not just validation). |
| **Latency** | 2-5 round-trips per negotiation (offer + 1-4 correction cycles). Best case 2 hops, worst case 10 hops. Direct agent-specialist dialogue avoids orchestrator bottleneck. |
| **Reliability** | ~99%. Iterative correction with surgical hints makes convergence near-certain within 5 rounds. Handles both structural and semantic issues. |
| **Cost** | Moderate-high. Each correction round costs tokens for both agent and specialist. 5-round cap bounds worst case. Schema caching amortizes cost for repeated interactions. |
| **Best for** | Complex, evolving formats where schema alone is insufficient. Semantic constraints ("findings must reference file:line"). Long-running collaborations where schema caching pays off. |

### B) Schema-in-Prompt

**How it works:** Include the expected JSON Schema + example + constraints directly in the dispatch prompt. Worker validates its own output against the schema before returning. No separate negotiation phase.

| Dimension | Assessment |
|-----------|------------|
| **Complexity** | Low. No new primitives. Requires: (1) schema attached to dispatch prompts, (2) worker self-validation logic (a JSON Schema check before returning). Both are straightforward. |
| **Latency** | 0 extra round-trips. Schema travels with the request. Worker validates locally. |
| **Reliability** | ~85-90%. Catches structural issues (missing fields, wrong types) reliably. Misses semantic constraints unless the prompt explains them well. LLM-based workers may still produce plausible-but-wrong output that passes structural validation. |
| **Cost** | Low. Schema in prompt adds ~200-500 tokens per dispatch. Self-validation is local (no extra messages). Failed self-validation may cause the worker to retry internally, but this is bounded. |
| **Best for** | Well-defined, stable formats. Cases where structural correctness is the main concern. High-volume dispatch where per-interaction negotiation cost is prohibitive. |

### C) Validate-and-Retry

**How it works:** Orchestrator dispatches without schema constraints. On return, validates the output. If invalid, re-dispatches the entire task with error details appended to the prompt ("your output was invalid because X, try again").

| Dimension | Assessment |
|-----------|------------|
| **Complexity** | Low-moderate. Requires: (1) orchestrator-side validation, (2) retry logic with error context injection. No new message types — reuses existing dispatch-collect. |
| **Latency** | 1 extra full round-trip per retry. Each retry re-executes the entire task, not just the formatting step. With a 3-retry cap: worst case 4x the base latency. |
| **Reliability** | ~90-95%. The retry prompt includes specific errors, so convergence is likely within 2-3 attempts. But full re-execution means the worker might change substantive content, not just formatting. |
| **Cost** | High on failure. Each retry pays the full token cost of the task, not just the correction. A task costing 2K tokens with 2 retries costs 6K. For rare failures this is acceptable; for frequent failures it's wasteful. |
| **Best for** | Low-frequency format issues. Cases where format failures are rare (<10% of dispatches) and the simplicity of "just retry" outweighs the cost of occasional re-execution. |

### D) No Enforcement

**How it works:** Trust workers to produce correct output based on task description. Handle format issues downstream (consumers adapt to what they receive, or humans fix it).

| Dimension | Assessment |
|-----------|------------|
| **Complexity** | Zero. Nothing to implement. |
| **Latency** | Zero overhead. |
| **Reliability** | ~60-70% for structural correctness, lower for semantic. Depends entirely on worker quality and prompt clarity. Format drift over time is likely. |
| **Cost** | Zero upfront. Hidden costs downstream: parsing failures, manual correction, debugging format mismatches. |
| **Best for** | Prototyping. Internal tools where the consumer and producer are the same developer. Exploratory tasks where output format genuinely doesn't matter. |

## Comparison Matrix

| | Complexity | Latency | Reliability | Cost | Sweet Spot |
|---|-----------|---------|-------------|------|------------|
| **A: 4-Phase** | High | 2-5 RT | ~99% | Moderate-high | Complex/evolving formats, semantic constraints |
| **B: Schema-in-prompt** | Low | 0 RT | ~85-90% | Low | Stable formats, structural validation |
| **C: Validate-retry** | Low-mod | 0-3 RT | ~90-95% | High on failure | Rare format issues, simple corrections |
| **D: No enforcement** | Zero | 0 RT | ~60-70% | Zero (hidden) | Prototyping, don't-care scenarios |

## Hybrid Strategy: Layered Enforcement

The approaches aren't mutually exclusive. They form a natural escalation ladder:

```
Layer 1: Schema-in-prompt (default for all dispatches)
   ↓ if output fails validation
Layer 2: Validate-and-retry (1 retry with error context)
   ↓ if retry also fails
Layer 3: Negotiate (only for complex/semantic issues)
```

**Why this works:**

1. **Schema-in-prompt catches 85-90% of issues at zero latency cost.** Most format problems are structural (missing field, wrong type). A schema in the prompt + worker self-validation handles these without any extra round-trips.

2. **Validate-and-retry catches another 5-8%.** The remaining issues are often "almost right" — a field in the wrong format, an array instead of a single value. One retry with specific error feedback resolves these.

3. **4-phase negotiation handles the last 2-5%.** These are the genuinely hard cases: semantic constraints, ambiguous requirements, evolving schemas. The full negotiation protocol is justified here because the issue requires iterative dialogue, not just retry.

**Implementation order:** Build Layer 1 first (trivial, immediate value). Add Layer 2 when evidence shows retry is needed. Build Layer 3 only if evidence shows negotiation-class problems exist at meaningful frequency.

## Recommendation for T-240 Go/No-Go

The 4-phase negotiation protocol should be **NO-GO as a standalone implementation** and **conditional GO as Layer 3 of the hybrid strategy**.

**Rationale:**
- Schema-in-prompt (Layer 1) delivers 85-90% of the value at ~5% of the complexity
- The T-240 assumption A1 ("interactions frequently require iterative correction") needs evidence — if most corrections are structural, Layers 1+2 suffice
- Building the full 4-phase protocol before proving Layers 1+2 are insufficient violates YAGNI
- The protocol design from T-233 remains valid and can be implemented later if evidence warrants it

**Concrete next step:** Implement schema-in-prompt for the dispatch-collect pattern (T-257). Track format failure rate. If >10% of dispatches fail after schema-in-prompt + one retry, that's the signal to build negotiation.
