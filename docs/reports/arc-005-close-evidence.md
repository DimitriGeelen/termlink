# arc-005 (mcp-slimming) — closure evidence

**Produced by:** T-2860
**Date:** 2026-08-30
**Arc:** `arc-005` / slug `mcp-slimming`, anchor T-2406, created 2026-07-11
**Constituent tasks:** T-2406 (S1), T-2407 (S2) — both `work-completed`

**Headline mechanic under test:**

> Every fleet agent receives slimmer MCP tool descriptions and executes tool calls
> with fewer malformed-argument retries — reclaiming a large share of the ~39k
> tokens/agent tool-catalog context

This document separates what was **measured** from what was **not**, because the
mechanic makes two claims and only one of them is demonstrated. Read the two
sections in that light. Prior art on why this matters: **PL-245** — arc-004 was
recorded closed=shipped and "live-verified" on loopback-only evidence.

---

## 1. Description trim — MEASURED

Baseline recorded in the arc description at creation (2026-07-11):
**273 tools · 156,525 bytes (~39k tokens) · max single 11,751 · 24 over 1000 · 94 over 600.**

Measured on this checkout at 2026-08-30 via `scripts/test-mcp-desc-budget.sh --report-only`
and a direct scan of `crates/termlink-mcp/src/tools.rs`:

| Metric | Arc-start baseline | Now | Change |
|---|---|---|---|
| Total description bytes | 156,525 | **105,516** | −51,009 (−32.6%) |
| ~tokens per agent per session | ~39,000 | **~26,379** | −12,600 |
| Max single description | 11,751 | **1,546** | −86.9% |
| Descriptions over 1000 chars | 24 | **1** | −96% |
| Descriptions over 600 chars | 94 | **23** | −76% |
| Median description | (not recorded) | 420 | — |
| Tool count | 273 | 261 | −12 |

### Attribution — read the total carefully

The headline −51,009 is **not all this arc's work**, and it would be wrong to claim it is.
The arc's own two slices are recorded in their commits:

- `cb5d74ca3` (T-2406, S1): 156,525 → **133,220**, tool count unchanged at 273
- `ebdd5f5f2` (T-2407, S2): 133,220 → **112,319**, tool count unchanged at 273

**arc-005's own contribution is therefore 156,525 → 112,319 = 44,206 bytes (~11,050
tokens/agent/session), with the tool count held constant — i.e. entirely text trimmed,
no tools removed.**

The further 112,319 → 105,516 (6,803 bytes) came from the tool count dropping 273 → 261.
That is unrelated P4 surface-reduction work (T-2471 / T-2478 deprecations), not trimming,
and it is excluded from the arc's claim.

The max-single and over-1000/over-600 figures are unaffected by that caveat: they are
pure trimming outcomes.

## 2. Anti-regrowth guard — MEASURED, and shown load-bearing

`scripts/test-mcp-desc-budget.sh` exists (T-2406 AC2) and passes at the tightened ceilings
(`MAX_DESC_CEILING=1560`, `TOTAL_DESC_CEILING=109000`).

A passing guard is only evidence if it can fail. Demonstrated 2026-08-30:

| Run | Ceiling | Result |
|---|---|---|
| Mutant A | `MAX_DESC_CEILING=1000` | **rc=1** — `FAIL: a tool description (1546 chars) exceeds MAX_DESC_CEILING (1000).` |
| Mutant B | `TOTAL_DESC_CEILING=100000` | **rc=1** — `FAIL: total description bytes (105516) exceed TOTAL_DESC_CEILING (100000).` |
| Control | defaults | **rc=0** |

Both arms fire with the specific overage named, and the control passes. The guard is
load-bearing, not vacuous.

## 3. Fleet delivery — INFERRED FROM VERSION, not directly measured

"Every fleet agent receives" is the half PL-245 warns about, so it is stated as an
inference with its basis shown, not as a measurement.

The trim landed at **VERSION 0.11.467** (commits `cb5d74ca3` / `ebdd5f5f2`, 2026-07-11).
A host therefore carries the trimmed catalog only if its installed binary is at or above
that. Per `scripts/check-fleet-binary-freshness.sh` on 2026-08-30:

| Hub | Served version | Carries trim (≥0.11.467)? |
|---|---|---|
| workstation-107-public | 0.11.1716 | yes |
| local-test | 0.11.1716 | yes |
| ring20-management | 0.11.1411 | yes |
| ring20-dashboard (.121) | 0.11.588 | yes — same lineage per T-2467 |
| laptop-141 (.141) | **unreachable** | **UNKNOWN** |

**Two limits on this row, stated rather than glossed:**

1. **It is a version proxy, not a probe of the catalog.** MCP tool descriptions are
   served by each host's own `termlink-mcp` binary to the agent running there — not by
   the hub over the wire. There is no remote RPC that returns a peer's tool-description
   bytes, so hub version is the best available stand-in for "what binary is installed on
   that host". A host could in principle run a hub and an MCP binary of different ages.
2. **Cross-lineage patch numbers are not ordered** (see CLAUDE.md § fleet-binary canary,
   T-2359). The `.121` comparison is only valid because T-2467 established it is on our
   own lineage. It is not a general rule.

**4 of 5 hubs infer as carrying the trim; 1 (.141) is unknown because it is down.**

## 4. Fewer malformed-argument retries — NOT MEASURED

The arc description motivates the work partly with a retry cost: *"raised odds a model
malforms argument JSON (an agent looped 9x on a rejected recent_dm call)"*.

**This clause is not demonstrated by anything in this document.** No before/after
malformed-argument retry rate was captured, and none is recoverable now — the original
9x-loop observation was anecdotal, from a single session, and no counter was instrumented
at the time to compare against.

What would settle it: a counter on MCP argument-validation rejections, sampled over a
fixed window before and after a catalog change. That does not exist, and building it is
not in this arc's scope.

The mechanism is plausible — the tool whose description drove the anecdote is inside the
trimmed set, and the descriptions that most invited malformed calls were the longest —
but plausible is not measured, and this arc should be closed on claim 1, not claim 2.

---

## Recommendation

Close arc-005 on the strength of §1–§3, with §4 explicitly unproven.

The arc has been `in-progress` with both constituent tasks `work-completed` since
2026-07-11, and has fired a stale-arc audit WARN for 5 consecutive audits. The work is
done; only the closure paperwork was outstanding.

**Closure is a human sovereignty action** (`fw arc close` is agent-refused under
`$CLAUDECODE=1`). Routed to the operator via `fw arc review arc-005`.
