# T-2057 — Track A retention audit (T-2028 follow-through)

**Status:** Audit complete (read-only). Findings classify each substrate
topic-creation path and surface the residual `Retention::Forever`
operator-default path responsible for T-1991 / G-058 topic-growth
incidents.

**Source:** T-2028 inception (PARTIAL GO, 2026-06-08), §4 Track A.
**Method:** static grep of `crates/` non-test code + live-hub probe via
`termlink channel list --json` on the local hub (.107).
**Decision authority:** none (audit-only — produces follow-up tasks).

## §1 Scope

This audit covers every path that produces a SQLite-backed channel topic
on a TermLink hub:

- Substrate code paths — `bus.create_topic(name, retention)` calls
  inside `crates/` (non-test, production paths only).
- Operator-facing RPC — `channel.create(name, retention?)` JSON-RPC.
- Operator-facing CLI — `termlink channel create <name> [--retention K[:V]]`.

Out of scope:
- The actual compaction/eviction implementation behind each retention
  policy. That's `crates/termlink-bus/src/compaction.rs`, audited by its
  own tests.
- `Retention::Latest` enum addition. That's the Track A build half,
  filed below as a follow-up if the audit confirms the gap.

## §2 Substrate-code topic creation sites

Found via `grep -n 'bus.create_topic\|create_topic(' crates/ --include='*.rs'`
filtered to non-test paths:

| File | Line | Topic | Retention | Verdict |
|---|---|---|---|---|
| `crates/termlink-hub/src/channel.rs` | 85 | `BROADCAST_GLOBAL_TOPIC` (`broadcast:global`) | `Messages(1000)` | **OK** — bounded, ~14× the typical fleet broadcast rate. |
| `crates/termlink-hub/src/channel.rs` | 174 | `inbox:<recipient>` (per-recipient inbox, lazy) | `Messages(1000)` | **OK** — bounded; matches inbox semantic of recent history not archive. |
| `crates/termlink-hub/src/channel.rs` | 235 | `ROUTING_LINT_TOPIC` | `Messages(1000)` | **OK** — bounded; routing lint posts are diagnostic, not durable record. |
| `crates/termlink-hub/src/channel.rs` | 332 | (RPC default fallback) | `Retention::Forever` if caller omits `retention` field | **OPERATOR-GAP** — see §3 below. |
| `crates/termlink-hub/src/channel.rs` | 889 | (channel.describe / info fallback) | `Retention::Forever` if caller omits | **OPERATOR-GAP** — same path as L332. |

**Verdict on substrate-code:** the three explicit code-created topics
(`broadcast:global`, `inbox:*`, `routing-lint`) are correctly bounded at
`Messages(1000)`. No code-side gap.

## §3 RPC default behavior

`channel.create(name, retention?)` parses the optional `retention` field
via `crates/termlink-hub/src/channel.rs:289-298`:

```rust
"forever" => Some(Retention::Forever),
"days"    => u32::try_from(value).ok().map(Retention::Days),
"messages"=> u64::try_from(value).ok().map(Retention::Messages),
```

When the field is absent, `crates/termlink-hub/src/channel.rs:332`
applies the fallback:

```rust
let retention = parsed_retention.unwrap_or(Retention::Forever);
```

**Operator-facing impact:**
- `termlink channel create <name>` with no `--retention` flag creates the
  topic with `Retention::Forever`.
- Skills and scripts that call `channel.create` without specifying
  retention inherit `Retention::Forever`.
- `--ensure-topic` callers in CLI's `channel post` (line 446) call
  `channel.create` without specifying retention — every implicit
  topic creation is `Forever` unless explicitly overridden.

**This is the T-1991 / G-058 root-cause path.** `agent-presence` and
`agent-chat-arc` were created via this default path during the foundation
soak and never had their retention re-set.

## §4 Live-hub evidence

`termlink channel list --json` on the local hub (107) at audit time:

| Metric | Value |
|---|---|
| Total topics | **1,331** |
| `Retention::Forever` count | **1,152 (87%)** |
| `Retention::Messages(N)` count | **179 (13%)** |
| `Retention::Days(N)` count | **0** |
| Largest topic | `agent-presence` (13,443 envelopes) on `Forever` |
| Second-largest topic | `agent-chat-arc` (2,950 envelopes) on `Forever` |

**Code-controlled topics behave correctly:**

| Topic | Live retention | Live count | Expected | Match? |
|---|---|---|---|---|
| `broadcast:global` | `Messages(1000)` | 533 | `Messages(1000)` | ✓ |
| `framework:pickup` | `Forever` | 37 | varies (no code-side default) | n/a (operator-created) |

**Operator-created topics show the gap:**

| Topic | Live retention | Live count | Why | Action |
|---|---|---|---|---|
| `agent-presence` | **Forever** | **13,443** | Created via default path during soak; never re-set. | **OPERATOR ACTION**: re-create with `Messages(200)` or apply `channel.set-retention` if/when available. |
| `agent-chat-arc` | **Forever** | **2,950** | Same default path; broadcast topic. | **OPERATOR ACTION**: re-create with `Messages(N)` matching desired retention window. |
| `channel:learnings` | **Forever** | 251 | Intentional — durable learnings record. | OK as-is. |
| `dm:*` (many) | **Forever** | 21-37 each | Per-pair DM topics; small but unbounded. | LOW PRIORITY — naturally small per pair; would benefit from `Messages(500)` cap. |
| `stress-fanin-*` (many) | **Forever** | 50 each | Test artifacts from soak. | **OPERATOR ACTION**: bulk-delete or apply test-cleanup hygiene. |

## §5 Findings

Classification of each path:

1. **OK — Code-controlled topic creation** (`broadcast:global`,
   `inbox:*`, `routing-lint`). All three carry explicit `Messages(1000)`
   retention. T-2028's "ensure each topic created by the substrate
   sets a retention" criterion is **met for code-driven creation**.

2. **OPERATOR-GAP — `channel.create` RPC default** (lines 332, 889).
   When the caller omits `retention`, the topic is created with
   `Forever`. This is the path T-1991 walked: `agent-presence` accumulated
   13,443 envelopes before the wedge surfaced. The RPC behavior is
   technically correct (let the operator decide), but the **default
   is the wrong default** for high-rate operational topics — and there's
   no nudge or warning when a creator specifies `Forever` for a topic
   whose name pattern matches high-rate conventions (`agent-*`, `dm:*`).

3. **OPERATOR-GAP — Test cleanup** (`stress-fanin-*` × 12-ish, plus
   several `agent-conv-*` and `agent-listeners-test-*`). The hub
   accumulates test-artifact topics over the lifetime of the soak —
   each is small but the directory entry pollutes `channel list` and
   adds non-zero SQLite footprint. Cleanup is operator-side, not a
   primitive concern, but worth flagging.

4. **NO CODE-GAP CONFIRMED.** No substrate-code path creates a topic
   with `Retention::Forever`. The `Retention::Latest` enum addition
   (T-2028 Track A build half) is therefore **not load-bearing for
   fixing the running-system gap** — it's useful for future
   broadcast-with-replay primitives (T-2027 bundle) but not for closing
   T-1991/G-058. **Recommend deferring the build half** until a
   concrete consumer exists.

## §6 Follow-up tasks

Concrete, scoped:

1. **Operator-side: `agent-presence` retention** — single-step recipe to
   reset the topic to `Messages(200)` (or measure-informed N). Must
   acknowledge the destructive-recreate aspect (or wait for
   `channel.set-retention` if landed). **Recommend file**: small
   operator runbook task tagged `operations`. ~30 lines doc.

2. **Code-side: nudge on `channel.create` with `Forever` for
   high-rate topic-name patterns** — `agent-*`, `dm:*`. Emit a
   `tracing::warn!` and (optionally) a structured RPC response field
   when an operator creates such a topic with `Forever`. Loud-not-silent
   per IW-3. **Recommend file**: small build task, ~40 LOC. Could ship
   alongside Track C observability.

3. **Operator-side: stress-fanin / agent-conv-test cleanup hygiene** —
   Soak cleanup pattern: after each foundation soak, drop test topics
   matching `stress-fanin-*` and `agent-conv-*-demo-*`. **Recommend
   file**: small operator runbook task or a `make soak-cleanup`
   target. Low priority.

4. **Track A build half (`Retention::Latest`)** — **DEFERRED**. No
   substrate primitive currently consumes it. File as captured-only
   when a downstream consumer surfaces (likely T-2027 broadcast-with-
   replay if/when promoted).

## §7 Recommendation

Track A audit half (this task): **COMPLETE**. The audit confirms:

- Substrate code is well-behaved on retention.
- The residual gap is operator-side (default-Forever on `channel.create`).
- The build half (`Retention::Latest`) does NOT close the running-system
  gap and should remain deferred.

Filing items 1-3 above as small follow-up tasks closes the practical
Track A scope without requiring the build half. Item 4 stays in this
audit report as a future-reference pointer.

## Related

- T-2028 — parent inception (PARTIAL GO, 2026-06-08).
- T-2048 — Track B governor (shipped).
- T-2049 — Track B dedupe ride-along (shipped).
- T-1991 — original silent-growth incident.
- G-058 — the gap this audit's findings address.
- `docs/operations/substrate-governor.md` — operator surface for Track B
  telemetry; Track C observability is its retention-side sibling.
