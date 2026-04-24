# T-1220 Inception: CLI/MCP inbox receiver migration

**Status:** Research artifact (not a decision). Answers five design questions
so the human can go/no-go with context instead of from scratch.

**Parent:** T-1163 hub-side dual-write shim (shipped 2026-04-24, commit
`a2a2d6d0`). Every successful `inbox::deposit` now also appends an envelope
into `channel:inbox:<target>` via `channel::mirror_inbox_deposit`. This task
rewrites the **read side** (CLI + MCP + remote verbs) to subscribe to the
channel topic instead of reading the inbox spool directly.

**Consumer surface** (from T-1163 call-site audit):

| Layer | File | Functions |
|---|---|---|
| CLI local | `infrastructure.rs` | `cmd_inbox_status/clear/list` |
| CLI remote | `remote.rs` | `cmd_remote_inbox_*` + `fleet_doctor` inbox check |
| MCP local | `tools.rs` | `termlink_inbox_{status,clear,list}` |
| MCP remote | `tools.rs` | `termlink_remote_inbox_*` |

Total: 3 local + 3 remote + 3 MCP-local + 3 MCP-remote = 12 call sites
(matches T-1163's 12-entry-point audit).

---

## Q1. Cursor persistence — where does per-caller read cursor live?

**Options considered:**

- **A. Per-caller file** at `~/.termlink/cursors/<caller-id>-<target>.seq`
  (caller-id = process UUID or CLI invocation hash). Survives CLI restart.
  **Risk:** filename explosion; every MCP tool invocation creates a new file
  unless caller-id is stable across invocations.
- **B. Per-(binary-path,target) file** at `~/.termlink/cursors/<hash(binary+target)>.seq`
  — stable across invocations of the same CLI. **Risk:** Multiple parallel
  CLI invocations race on the same file.
- **C. SQLite cursor table** inside `~/.termlink/cursors.db`. Stable, concurrent-safe.
  **Risk:** dependency weight; needs schema migration plan.
- **D. In-memory only** — every fresh CLI invocation starts from `cursor=0` and
  reads all history. **Risk:** O(N) cost per `inbox list` call as topic grows.
  **Mitigation:** Retention policy caps the topic at Messages(1000), so the
  worst case is bounded. Latency-wise acceptable.

**Leaning:** D (in-memory only). Reason: existing `inbox.list/status/clear`
semantics are "show me what's currently pending" — not "show me what's new
since I last looked". Users already expect to see the full pending list on
every invocation. `cursor=0 + limit=1000` matches current mental model and
avoids cursor state entirely.

**Deferred:** per-MCP-session cursor (distinct from CLI) if watchtower or
another long-running consumer wants "new since I last polled" semantics —
that's a build sub-task, not T-1220 scope.

---

## Q2. Capabilities probe timing — when do we call `hub.capabilities`?

**Options considered:**

- **A. Per-invocation probe** — every `inbox list` call first does `hub.capabilities`.
  **Cost:** +1 RPC per inbox call (~5-20ms over TCP). Always fresh.
- **B. Per-session-per-target cache** (T-1215's `HubCapabilitiesCache`). First
  call probes, subsequent calls read from cache. **Cost:** 1 probe per hub per
  process lifetime. **Risk:** Stale after hub upgrade adds/removes methods.
- **C. Cache with TTL** (e.g., 5-minute expiry). Hybrid.
- **D. No probe — try channel.subscribe, fall back on method-not-found error.**
  **Cost:** 0 extra probes; fallback path runs on error. **Risk:** Error-as-
  control-flow; first inbox call against legacy hub pays the method-not-found
  roundtrip.

**Leaning:** B (T-1215 cache) for CLI; D (no probe) for MCP tools.

- CLI processes are short-lived; cache lives for one invocation anyway, so B and
  A are equivalent-cost.
- MCP tool calls are one-shot; cache gives zero benefit. D removes the probe
  tax entirely; fallback cost is amortized over the tool's lifetime.

**Alternative worth human input:** if strong preference for uniformity, pick
B everywhere and accept the one-time probe per MCP process.

---

## Q3. Fallback semantics — what happens when peer lacks `channel.*`?

**Options considered:**

- **A. Silent fallback** — channel.subscribe fails → try inbox.list → surface
  result. User sees identical output regardless. **Risk:** Hides the fact that
  peer is on legacy protocol; user doesn't know to prompt upgrade.
- **B. Warn-once fallback** — first time falling back per peer per process,
  log a stderr warning `[warn] peer <host> lacks channel.*; using legacy inbox.*
  (upgrade recommended)`. **Risk:** noisy if every command warns.
- **C. Structured flag in JSON output** — add `legacy_fallback: true` field
  when applicable. Programmatic consumers can detect; interactive output
  unchanged. **Risk:** schema change may break parsers.

**Leaning:** B (warn-once) for the interactive CLI path + C (flag) in JSON
output. Human users notice when something regressed; programmatic consumers
can detect without output noise.

---

## Q4. `inbox.clear` semantics — what does "clear" mean in a channel world?

Current `inbox.clear`:
- Deletes spool files at `<runtime>/inbox/<target>/<transfer-id>/...`
- Affects ALL consumers (there's only one spool).

Channel-backed `inbox.clear` options:

- **A. Advance my cursor to `latest_offset`** — local, per-caller. Only "my"
  view shifts; other consumers' cursors unchanged. **Risk:** feels wrong to
  users who `inbox clear` expecting to drain the spool.
- **B. Call a new `channel.trim` RPC on the hub** — mutates topic retention,
  drops messages before `<cutoff>`. Affects ALL consumers. **Risk:** no such
  RPC exists yet; retention is currently policy-based (Messages(1000)).
- **C. Hybrid — local-cursor by default, `--hard` flag calls `channel.trim`**
  with an explicit warning. **Risk:** complexity; two different semantics for
  one verb.

**Leaning:** A, with a doc-string change on the CLI help text clarifying that
`clear` advances local cursor and does not delete from peers' views. Users
who expect the nuclear option can be pointed at the forthcoming `fw retention
reset` (T-1166 scope).

**Open:** during transition (both legacy + channel active), should `clear`
also call legacy `inbox.clear` to delete spool files? That keeps current
behaviour intact. **Leaning: yes** — clear means clear until T-1166 retires
the dual-write shim.

---

## Q5. Mixed-mode rollout — channel-only readers miss legacy-only deposits

**The gap:**
- Pre-T-1163 deposits land ONLY in legacy inbox spool
- Post-T-1163 deposits land in BOTH legacy spool AND channel:inbox:<target>
- A channel-only reader misses everything before the T-1163 hub restart

**Options:**

- **A. Require dual-read during transition** — always read legacy AND channel,
  merge by (target, transfer_id) dedup. **Cost:** 2 RPCs per inbox list.
  **Guaranteed** to show everything.
- **B. Channel-first, legacy-only-on-miss** — read channel. If empty result
  AND hub was restarted recently (<retention window), fall back to legacy.
  **Risk:** fragile heuristic.
- **C. One-shot migration** — on hub upgrade, drain legacy spool into channel
  at startup. Then readers can ignore legacy. **Risk:** new hub code; transitional
  code lives forever unless explicitly retired.
- **D. Accept the gap** — during transition, some deposits are legacy-only
  and will show only via `inbox list --legacy` or equivalent. T-1166 retires
  legacy and closes the gap at that milestone.

**Leaning:** A for the transition window (1-2 releases), then drop to
channel-only at T-1166. The merge cost is bounded (Messages(1000) retention).

---

## Recommended wedge split (if GO)

1. **T-1220a** — `termlink-session` helper: `inbox_channel::list_with_fallback(target,
   hub_caps)` — encapsulates capabilities probe + channel.subscribe + legacy
   fallback + dedup-merge per Q5. Single cohesive unit, ~100 LOC + tests.
2. **T-1220b** — CLI local migration: `cmd_inbox_{list,status,clear}` switch
   to the helper. ~3 call sites.
3. **T-1220c** — CLI remote migration: `cmd_remote_inbox_*` + fleet-doctor
   inbox check. ~4 call sites.
4. **T-1220d** — MCP migration: `termlink_inbox_*` + `termlink_remote_inbox_*`.
   ~6 call sites.

Each wedge ships independently. 1 blocks 2/3/4. 2/3/4 are parallelizable.

---

## Go/No-Go decision

**Status:** Awaiting human.

**Questions the human should answer:**
- Q1: accept D (in-memory), or prefer persistent cursor?
- Q2: accept B+D split, or uniform B everywhere?
- Q3: accept B+C (warn-once + flag), or simpler A (silent)?
- Q4: accept A (local-cursor clear) + doc-string change, or design the hub-side `channel.trim` RPC first?
- Q5: accept A (dual-read transition) or push harder on C (hub-side migration)?

**Recommend GO only if:** answers to Q1-Q5 are settled. Otherwise defer this
task and re-inception after T-1164 (file.send/receive) ships — that migration
will surface whether the patterns above generalize.

**Recommend NO-GO if:** the T-1166 retirement date is <2 weeks out — just
wait for legacy to go away, avoid the transition-mode complexity entirely.
