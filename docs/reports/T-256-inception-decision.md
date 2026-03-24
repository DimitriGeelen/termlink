# T-256: Push Messaging Inception Decision

## Summary of Prior Research

Three research agents (T-256 Q1-Q3) investigated TermLink's messaging primitives, dispatch architecture, and Claude Code execution model.

### Key Findings

1. **All event consumption is poll-based** — `event.collect` polls each worker at 500ms intervals via hub. No push notification mechanism exists.

2. **T-257 solved the user-visible problem** — Collect-based fan-in reduced orchestrator context cost from ~10K-18K tokens (polling loop) to ~800 tokens (single `collect --count N` call). The user's original pain was context budget, not latency.

3. **500ms latency is negligible for current workloads** — Claude agents take 1-10 minutes per task. A 500ms event delivery delay is <0.1% of total task time.

4. **Ring buffer (1024 events) is adequate** — Current max fan-in is ~10 workers, each emitting 2-3 events (progress + completed). Peak: ~30 events per dispatch. Buffer overflow requires ~34x current load.

5. **Push would require Rust changes** — An `event.emit_to` RPC needs: new handler in `control.rs`, target session socket resolution, cross-session event injection. Backward-compatible but non-trivial.

## Assumption Validation

| Assumption | Status | Evidence |
|------------|--------|----------|
| A1: 500ms latency is problematic | **DISPROVED** | Agent tasks take minutes; 500ms is noise |
| A2: Ring buffer overflow risk | **DISPROVED** | Peak ~30 events vs 1024 capacity (~34x headroom) |
| A3: Backward-compatible change | **LIKELY VALID** | Optional `target` field on emit, old clients ignore |
| A4: Push reduces CPU load | **VALID but immaterial** | Polling CPU is negligible on modern hardware |

## Decision: NO-GO

T-257 collect-based fan-in handles all current scenarios. The 500ms poll latency and CPU overhead are not measurable problems at current scale (3-10 workers, minutes-per-task).

**The emit-to-target design is valid.** Revisit when:
- Worker count exceeds ~50 per dispatch (ring buffer pressure)
- Tasks complete in <5 seconds (latency becomes proportionally significant)
- Real-time streaming use cases emerge (not batch dispatch)
