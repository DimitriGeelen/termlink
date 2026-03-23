# T-233 Q1-hybrid: Persistent vs On-Demand Specialist Agents

## Research Question

Should specialist agents use a hybrid model (some persistent, some on-demand)? What criteria determine which category a specialist falls into?

## Finding: Hybrid is the Only Viable Model

Pure-persistent wastes resources on idle specialists. Pure-on-demand pays startup cost every time, losing accumulated context. A hybrid model is not a compromise — it's the correct architecture.

## Classification Criteria

### Hot-Path (Persistent) Specialists

A specialist should be persistent when **all three** conditions hold:

1. **High invocation frequency** — called >3 times per session on average
2. **Expensive cold start** — context loading takes >5 seconds or requires loading >10 files
3. **Stateful across calls** — benefits from remembering prior interactions within a session

**Examples:** Code specialist (always needed, loads crate topology), infrastructure specialist (SSH connections are expensive to re-establish), orchestrator itself.

**Implementation:** TermLink sessions with `termlink spawn --persist`, kept alive for the session duration. Context pre-loaded via capability manifest at spawn time.

### Cold-Path (On-Demand) Specialists

A specialist should be on-demand when **any** condition holds:

1. **Low frequency** — called <1 time per session on average
2. **Cheap cold start** — context fits in a single prompt injection (<2K tokens)
3. **Stateless** — each invocation is independent (no cross-call memory needed)

**Examples:** Audit specialist (runs once at session end), documentation specialist (occasional), release specialist (rare).

**Implementation:** `termlink spawn` on demand, `termlink destroy` after result collected. Use `fw bus post` for result handoff.

### Warm Standby (The Third Category)

Some specialists don't fit cleanly. **Warm standby** means: not running, but pre-configured so spawn is near-instant.

**Criteria:** Medium frequency (1-3 times per session) AND moderate cold start (2-5 seconds).

**Examples:** Test specialist (needed intermittently, benefits from cached test topology), research specialist (bursts of activity then idle).

**Implementation:** Pre-built context manifests stored on disk. `termlink spawn --manifest research.yaml` loads context in <1 second vs 5+ seconds for cold assembly. Session destroyed after idle timeout (configurable, default 5 minutes).

## Session Pooling

For on-demand specialists, a **pool of pre-warmed generic sessions** avoids repeated spawn overhead:

- Pool size: 2-3 generic sessions kept alive with minimal context
- On dispatch: claim a pooled session, inject specialist context via manifest
- On completion: strip specialist context, return to pool
- Benefit: eliminates ~2-3 second spawn overhead for cold-path specialists

**Cost-benefit threshold:** Pooling is worthwhile only when on-demand specialists are invoked >5 times per session total across all types. Below that, raw spawn is simpler.

## Decision Matrix

| Signal | Hot | Warm | Cold |
|--------|-----|------|------|
| Calls per session | >3 | 1-3 | <1 |
| Cold start cost | >5s | 2-5s | <2s |
| Cross-call state needed | Yes | Maybe | No |
| Context manifest size | >10 files | 3-10 files | <3 files |
| Recommended lifecycle | Session-scoped | Idle-timeout | Per-call |

## Key Insight

The classification is not static. A specialist can be **promoted** from cold to warm to hot based on observed invocation patterns. The orchestrator should track call frequency per specialist type and auto-promote after 3 sessions of consistent high usage. This makes the system antifragile — it adapts to actual usage rather than predicted usage.

## Recommendation

Implement all three tiers from the start. The infrastructure cost is minimal (TermLink already supports spawn/destroy), and trying to retrofit warm standby later would require replumbing session lifecycle management.
