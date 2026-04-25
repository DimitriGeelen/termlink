# T-1229 Inception: `inbox.status` migration via channel aggregation

**Status:** GO
**Decision date:** 2026-04-25
**Workflow:** inception → build (sub-tasks to be created on GO)
**Related:** T-1163 (channel mirror), T-1166 (retirement), T-1220 (parent receiver migration), T-1226-1232 (list migration wedges)

## Problem

After T-1226/27/28/32 the `inbox.list` receiver migration is complete. The
parallel migration for `inbox.status` is *not* a drop-in: status returns
an aggregate `{total_transfers, targets: [{target, pending}]}` while the
channel surface is per-topic (`channel.subscribe(topic="inbox:<target>")`).

T-1166 will retire `inbox.status` along with `inbox.list` and `inbox.clear`,
so a migration target is required — keep-legacy is not viable for the long
horizon. Five call sites depend on the aggregate form (~150 LOC):

- `cmd_inbox_status` — `crates/termlink-cli/src/commands/infrastructure.rs:766`
- `termlink_inbox_status` MCP — `crates/termlink-mcp/src/tools.rs:4518`
- `cmd_remote_inbox_inner` Status arm — `crates/termlink-cli/src/commands/remote.rs:1253`
- `termlink_remote_inbox_status` MCP — `crates/termlink-mcp/src/tools.rs:~4684`
- Fleet-doctor inbox check — `crates/termlink-cli/src/commands/remote.rs:2810`

## Recommendation

**Option A — Add hub-side `channel.list_topics(prefix="inbox:")` RPC.**

Single round-trip aggregation that mirrors the existing
`crate::inbox::list_all_targets()` semantics on the channel surface.
Returns `[{topic, pending_count, latest_offset}]` for every topic matching
the prefix. ~30 LOC of new server code, then per-site replacement of
`inbox.status` calls is mechanical.

## Go / No-Go

**GO.** Option A is the only path that preserves fleet-doctor's
correctness invariant *and* satisfies T-1166's retirement scope.
Sub-tasks (one per call site) to be created post-decision.

## Q1 — Aggregation source: A vs B vs C

### Option A: hub-side `channel.list_topics(prefix="inbox:")` RPC — **CHOSEN**

**Mechanism.** New router method in `crates/termlink-hub/src/router.rs`
delegates to a thin wrapper over the existing
`crate::inbox::list_all_targets()`. Filters topics by prefix. Returns
`[{topic: "inbox:<target>", count: <pending>}, ...]`. Optionally annotates
with the latest channel offset so callers can build "since-cursor" delta
counts in the future.

**Pros.**

- Single RPC round-trip per `inbox.status` call (parity with current).
- Hub already has the data — implementation is exposing the existing
  spool walk via the channel surface verb.
- Preserves fleet-doctor's invariant (sees all pending transfers
  regardless of subscriber online status — see Q2).
- Aligns with T-1166: when legacy `inbox.status` is removed, the new
  verb stays.
- Generalises beyond inbox: future `event:*`, `learnings:*` aggregations
  reuse the same RPC surface.

**Cons.**

- Adds new server method to T-1166's already-large retirement scope —
  but offset by removing three legacy methods, so net surface decreases.
- New verb needs a name decision: `channel.list_topics` (matches
  list/subscribe family) vs `channel.topics` (terser) vs
  `channel.aggregate` (more general). **Recommended: `channel.list_topics`.**

### Option B: client-side enumeration via `session.discover` + per-topic subscribe — **REJECTED**

**Mechanism.** Client calls `session.discover` to enumerate known sessions,
then issues `channel.subscribe(topic="inbox:<each>", limit=0)` per session
to pull a count.

**Why rejected.**

1. **N+1 round-trips per call** — fleet-doctor runs this on every
   `fleet doctor` invocation; on a 50-node fleet that is 51 RPCs vs 1.
2. **Correctness regression.** `session.discover` returns active sessions
   only. The hub spool can hold pending transfers for *offline* targets
   and for *targets that never registered a session* (the verbatim use
   case for inbox: "queued for offline sessions"). Per-session enumeration
   silently drops these transfers from the count. Fleet-doctor would
   stop warning on the exact failure mode it exists to surface.
3. No way to detect missing transfers from the client side without
   already knowing every target name — circular.

### Option C: keep `inbox.status` legacy forever — **REJECTED**

T-1166 acceptance criteria explicitly list `inbox.status` for router
removal (see `crates/termlink-hub/src/router.rs:74-75` and T-1166 AC
§"Router methods removed"). The retirement is not negotiable per the
T-1155 §Phase 4 plan. Option C only works as a temporary stop-gap, but
Option A exists and is cheap, so the stop-gap is pointless.

## Q2 — Fleet-doctor sensitivity

**Risk under Option B:** Real and disqualifying. `fleet-doctor` inbox
check at `crates/termlink-cli/src/commands/remote.rs:2810` warns on any
pending transfer; under Option B it would silently miss transfers
addressed to offline sessions. This is a correctness regression vs the
current legacy behavior.

**Risk under Option A:** None. Server-side aggregation walks the same
spool directory as `inbox::list_all_targets()` does today — the data
source is identical, only the RPC verb changes. The invariant
"`fleet-doctor` warns iff a pending transfer exists" is preserved.

**Verification.** Sub-task verification commands MUST include a positive
test: deposit a transfer for an offline target via the hub spool, run
the new `channel.list_topics(prefix="inbox:")` against it, assert the
target appears in the result with `count >= 1`. This catches any
regression to the offline-session blind spot during build.

## Q3 — T-1166 alignment

T-1166 explicitly retires `inbox.status` (see `crates/termlink-hub/src/router.rs:74,755`
and T-1166 §"Router methods removed"). Confirmed via Read of the active
T-1166 task file (status `captured`, horizon `next`, 60-day parallel
operation gate).

**Implication for build sub-tasks:**

1. The new `channel.list_topics` verb must be in production for at least
   60 days before T-1166 can retire `inbox.status` — same parallel-
   operation gate as T-1163 used for `inbox.list`.
2. The aggregation feature can be added to the hub *before* the call
   sites migrate (additive change, no breaking impact).
3. Migration order: (a) hub method, (b) client helper in
   `inbox_channel.rs` mirroring `list_with_fallback`, (c) per-site
   migrations one wedge at a time.

## Build Sub-Tasks (post-GO)

Following the T-1226-T-1232 wedge pattern (one task = one deliverable):

| ID | Scope | LOC est |
|----|-------|---------|
| T-1229a | Hub: add `channel.list_topics(prefix)` router method + tests | ~80 |
| T-1229b | termlink-session: add `status_with_fallback{,_with_client}` helper in `inbox_channel.rs` (parity with T-1231 helper) | ~120 |
| T-1229c | CLI local: migrate `cmd_inbox_status` (infrastructure.rs:766) | ~20 |
| T-1229d | MCP local: migrate `termlink_inbox_status` (tools.rs:4518) | ~25 |
| T-1229e | CLI remote: migrate `cmd_remote_inbox_inner` Status arm (remote.rs:1253) | ~25 |
| T-1229f | MCP remote: migrate `termlink_remote_inbox_status` (tools.rs:~4684) | ~30 |
| T-1229g | Fleet-doctor: migrate inbox check (remote.rs:2810) — last, with regression test for offline-target visibility | ~20 |

Total: ~320 LOC across 7 sub-tasks. Each sub-task has its own
verification + ACs; T-1229g must include the offline-target regression
test from Q2.

## Dialogue Log

This inception was conducted as a sequential analysis pass after the
T-1226/27/28/32 list-migration wedges shipped. No live human dialogue —
the design questions were posed by the inception task itself (Q1-Q3 in
the task file) and answered by reading T-1166's retirement scope, the
existing hub implementation at router.rs:1502-1520, and the fleet-doctor
call site at remote.rs:2810. The CHOSEN option (A) was confirmed against
all three Q's without conflict.

If the human disagrees with Option A, the natural alternative is Option C
plus an exception in T-1166's retirement scope — but that requires
relitigating the T-1155 Phase 4 plan and is strictly worse than Option A.
