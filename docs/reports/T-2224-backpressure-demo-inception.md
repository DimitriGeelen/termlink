# T-2224 — substrate BACKPRESSURE demo feasibility (arc-001 #10 proof)

**Task:** T-2224 (inception; originally filed as build, retyped 2026-06-13)
**Question:** can the governor backpressure path (substrate primitive #10) be
proven by a self-contained smoke-gate demo, the way T-2211/2212/2214 prove the
claim primitive?
**Outcome:** **NO-GO** (recommendation; Tier-0 decide is the human's).

> Note on provenance: this artifact consolidates the investigation trail that
> was recorded in the task file's Recommendation section on 2026-06-13. It was
> written to docs/reports/ when the C-001 commit gate (correctly) refused
> further T-2224 commits without a research artifact. Findings are unchanged
> from the original investigation; source line references are as inspected on
> 2026-06-13.

## Findings

### 1. Rate-limit refusal (RATE_LIMITED, -32008) — infeasible from sequential CLI posts

The hub's rate governor keys buckets by `sender_key` with precedence
`params.from` > `peer_addr` > `peer_pid` (`crates/termlink-hub/src/server.rs:756`).
`channel.post` sets envelope `sender_id` but never `params.from`
(`bus_client.rs::post_to_params`), so on a Unix-socket hub the key falls
through to `peer_pid`. Every CLI invocation is a fresh process → a fresh pid →
a distinct rate bucket that never accumulates. A demo firing N sequential
posts presents to the governor as N distinct senders and structurally cannot
hit the per-sender limit.

### 2. Capacity refusal (HUB_AT_CAPACITY, -32019) — feasible but flaky-risk

Forceable with `TERMLINK_MAX_CONNECTIONS=N` plus N+1 concurrently-held
`channel subscribe` connections. But holding an exact live-connection count
and capturing the (N+1)th refusal from shell is timing-dependent: connection
teardown races the next dial, and a slow tick flips the observed refusal to a
success. A flaky stage in the smoke gate erodes trust in the whole suite —
disqualifying for a deterministic regression gate.

### 3. Existing coverage — the demo would add little

The governor layer is already asserted by `governor.rs` unit tests
(`rate_hits_total` counting, bucket eviction). A shell demo would re-prove the
same layer with worse determinism.

## Investigation payoff (captured as PL-218)

The `peer_pid` keying is the mechanism behind PL-209's rate-bucket bloat:
short-lived posters each mint a bucket (`governor.rs:312` notes
`rate_buckets_active=258_236` against a ~5-agent fleet). T-2137's idle-TTL
eviction mitigates but does not remove the root. A deeper fix — keying UDS
buckets on the T-1427 verified identity fingerprint instead of the ephemeral
pid — is a security tradeoff (client-asserted `sender_id` is spoofable;
`peer_pid` is kernel-trusted but ephemeral) and is the natural follow-up
inception if bucket bloat is judged worth fixing beyond eviction.

## Recommendation

**NO-GO** on a backpressure/governor smoke-gate demo:
demo is not load-bearing (finding 1 makes the primary path untestable from
shell, finding 2 makes the secondary path flaky), and the layer is already
unit-covered (finding 3). Investigation value is preserved as PL-218.

**Decision route (Tier-0, human):**

```
cd /opt/termlink && .agentic-framework/bin/fw inception decide T-2224 no-go --rationale "demo not LC; governor covered by unit tests; root-cause captured as PL-218"
```

No code was shipped under this task.
