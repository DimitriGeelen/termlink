# T-2296 — V6: Direct transport-first + hub fallback + per-conversation journaling — Implementation Design

> arc-003 reliable-comms APEX. Read with: `.tasks/active/T-2296-...md`,
> `docs/reports/T-2291-cross-agent-comms-inception.md` (§Dialogue Log Step-6 V6 pivot),
> `.context/working/T-2291-V6-spike.md`. Perspective: **smallest-safe-first, lean on
> already-shipped primitives** (the spike's headline: V6 is discovery-population +
> fallback-orchestration + journaling, NOT greenfield transport).

## 0. The code realities (verified file:line)

- **Direct P2P already exists & ships.** `connect_remote_hub()`
  (`crates/termlink-cli/src/commands/remote.rs:719`) parses `host:port`, opens
  `TransportAddr::Tcp` (`remote.rs:774`) via `client::Client::connect_addr_with_timeout`
  (`remote.rs:775`, 10s bound), auths with a 64-hex HMAC token over `hub.auth`
  (`remote.rs:770,783`) + TOFU TLS. `remote inject` → `command.inject` RPC
  (`remote.rs:1546`, DIRECT); `remote exec` → `command.execute` (`remote.rs:2011`,
  DIRECT); `remote call` → generic JSON-RPC passthrough. The receiving end is a
  **normal hub** on the peer host listening on :9100 — there is no separate "agent
  daemon". This is the load-bearing fact: a direct 1:1 message is a `channel.post`
  RPC issued against the *peer's own hub* instead of the *local* hub.

- **Hub-mediated conversational path today** is `scripts/agent-send.sh`: posts the
  turn as `channel post <dm-topic> --msg-type turn` (line 220), rings the doorbell,
  polls the dm-topic for a `msg_type=receipt` envelope (lines 259-264). When the peer
  is on a remote hub it already threads `--hub <peer_hub>` through every leg
  (`hub_args`, lines 216-217, 245-249) — resolved from fleet presence
  (`scripts/agent-listeners-fleet.sh`, lines 116-146). **The "try-direct/fall-back"
  decision point belongs exactly here**: today every send already targets *a* hub
  (`hub_args`); V6 inserts a transport-selection step that picks {direct peer hub :
  local hub store-and-forward} *before* the post.

- **The store/firehose write** is `handle_channel_post_with`
  (`crates/termlink-hub/src/channel.rs:512`) → appends the signed envelope to the
  topic log on whichever hub received the RPC. There is no per-conversation
  partitioning: dm: turns + 31.5k heartbeats all land in one bus store (the 70.5%
  obfuscation, T-2291 §Dialogue "Audit"). **Journaling-off-firehose = where dm:
  turns get persisted.**

- **Three ack signals (the ladder inputs):**
  - **(A) `msg_type=receipt` envelope.** `agent-respond.sh:94` posts `channel post
    --msg-type receipt --metadata up_to=<offset>`; `agent-send.sh:259-266` polls it
    offset-aware (`up_to >= post_offset`). This is the **conversational** receipt and
    V3b standardized on it (`--no-await-ack` opt-out, `agent-send.sh:55,228-236`).
  - **(B) `channel.receipts` frontier.** `crates/termlink-session/src/ack_retry.rs`
    (`ReceiptRow`/`recipient_acked` at `ack_retry.rs:101,107`; `awaiting_ack.sqlite`
    at `ack_retry.rs:264,279`) + hub handler `handle_channel_receipts`
    (`channel.rs:957`, routed `router.rs:106`). This is `channel post --await-ack` /
    `channel ack` — the **hub-frontier** confirm.
  - **(C) reply-turn / `agent contact --ack-required`.** `agent.rs:1266-1313`
    (cli `cli.rs:4534`), exit 10 on ack-timeout — a full reply turn is the ack.

- **Durable per-host stores exist** under `~/.termlink/`: `outbound.sqlite`
  (offline queue, `offline_queue.rs:108,122` — `pending_posts`+`dead_letters` tables),
  `awaiting_ack.sqlite` (sender-side tracker, `ack_retry.rs:264,279`). **No
  per-conversation journal exists yet.** `kv` is per-session/in-memory (spike B.4 +
  `notify-sidecar.sh:17-22` comment) — wrong scope.

- **V3a notify sidecar** (`scripts/notify-sidecar.sh`, T-2294): no-LLM loop,
  remote-reads mail → writes `~/.termlink/notify/<agent>.{flag,heartbeat}`;
  consumer `scripts/notify-check.sh` returns exit 10 MAIL / 3 DEAF / 0 CLEAR at the
  agent's yield points. **This is the read-receipt (level-3) producer** and the wake
  mechanism for any transport.

- **T-2297 (V2b) NOT yet shipped.** `peer_addr` IS available at `server.rs:640,670`
  and passed into `handle_connection`/`process_request` (`server.rs:684,761,785`),
  but it is **NOT** threaded through `route_request → handle_channel_post`
  (`router.rs:94` calls `handle_channel_post(id, &req.params)` with no addr; the
  envelope has no `observed_addr`). T-2297 adds that thread + makes
  fleet-presence/resolve prefer the hub-attested addr. **V6 consumes T-2297's output
  as the direct-transport target** (`agent_id → observed host:port`).

## 1. Slicing (5 sub-slices, dependency-ordered)

V6 is L→XL; one session cannot land it. Slices are independently shippable and each
leaves the system in a working state (direct path stays opt-in until S5).

| # | Slice | Deliverable (one line) | Size | Depends on |
|---|---|---|---|---|
| **S1** | **Per-conversation journal (read-side mirror)** | dm: turns mirrored into `~/.termlink/journals/<convo>.sqlite`, mineable via `agent journal` query verb; firehose still authoritative | **S→M** | nothing (pure additive) |
| **S2** | **Reachability probe + transport-select seam** | `agent-send.sh` gains a `--transport auto\|direct\|hub` flag + a dry-run that prints the chosen plan; `direct` = post to peer's own hub addr | **S** | T-2297 (addr) *or* self-report addr (ships in T-2293) |
| **S3** | **Direct delivery confirm via sidecar journaled-receipt** | on the direct path, level-2 "delivered" comes from a receipt the recipient's sidecar journals (mechanism A), NOT the hub frontier | **M** | S1, S2, V3a sidecar |
| **S4** | **Try-direct / fall-back-to-hub orchestration (default)** | `--transport auto`: try direct, on host-unreachable fall back to hub store-and-forward + frontier confirm; failover is loud | **M** | S2, S3 |
| **S5** | **Journal-authoritative + firehose suppression for dm:** | durable dm: turns stop landing in the hub firehose (go to journals); fleet forensic = aggregate-on-demand across journals | **M→L** | S1–S4 |

**Smallest safe first step = S1.** It is pure-additive (a read-side mirror, firehose
unchanged), needs no peer, no transport change, no protocol change — and it
de-risks S5 (the journal schema + query surface is proven before anything is moved
*off* the firehose). It also delivers standalone value: AC4 ("journal is mineable")
is met by S1 alone.

## 2. Per-slice detail

### S1 — Per-conversation journal (read-side mirror) — smallest safe first step
- **New store:** `~/.termlink/journals/<convo_id>.sqlite` (or one DB,
  `messages(convo_id, offset, sender_id, msg_type, ts, payload, observed_addr)` +
  index on `(convo_id, offset)`). Mirror the discipline of
  `offline_queue.rs:108-150` (DEFAULT_FILE_NAME, `open()` creating parent dir).
- **Files/functions:**
  - New module `crates/termlink-session/src/conversation_journal.rs` modeled on
    `offline_queue.rs` (rusqlite, `Mutex<Connection>`, `open()/append()/query()`).
  - Populate read-side: a small loop verb (script first, mirroring
    `notify-sidecar.sh` so no Rust rebuild is needed for v1) that
    `channel subscribe dm:*` and appends new envelopes to the journal. Reuse the
    `dm:*` enumeration already in `notify-sidecar.sh:161-176`.
  - New CLI query verb `agent journal <convo_id> [--since] [--json]` (cli.rs +
    `commands/agent.rs`) — or a `scripts/agent-journal.sh` for v1.
- **New surface:** `agent journal` read verb only. No RPC change.
- **Test without a peer:** self-post to a `dm:self:self` topic, run the mirror once,
  assert the journal row exists and `agent journal` returns it. Loopback hub
  (`127.0.0.1:9100`, already in hubs.toml) covers the subscribe path. No second host.

### S2 — Reachability probe + transport-select seam
- **Decision logic (lives in `agent-send.sh`, the existing routing brain):**
  after fleet-resolve (`agent-send.sh:104-166`) we already have `peer_hub`
  (the addr that saw the heartbeat). Add:
  - `--transport auto|direct|hub` (default `hub` in S2; flips to `auto` in S4).
  - `direct` = the peer's reachable `host:port` IS the post target (`hub_args`
    already does this — direct is just "the peer hub is the peer's *own* hub, not a
    relay"). The semantic difference from today's cross-hub send is **the confirm
    source** (S3), not the post mechanics.
  - reachability probe = a bounded TCP connect to `host:port`, mirroring
    `connect_addr_with_timeout`'s 10s bound (`remote.rs:775-780`); expose as
    `termlink remote ping <addr>` which already exists (`cmd_remote_ping`,
    `remote.rs:1071`). Use it as the GO/NO-GO gate.
- **Addr source:** prefer T-2297 hub-stamped `observed_addr`; fall back to T-2293
  self-reported addr (already shipped). **If T-2297 has not landed, S2 still ships**
  on the self-report addr (the spike notes cross-host auth is already symmetric;
  self-report is the renumber-risk fallback, acceptable for v1 on the flat /24).
- **New surface:** `--transport` flag + extend the existing `--dry-run RESOLVED`
  line (`agent-send.sh:205-207`) to print `transport=direct|hub reachable=yes|no`.
- **Test without a peer:** `--dry-run` asserts the chosen plan from canned presence
  JSON (the `LISTENERS_VERB` fixture seam already exists, `agent-send.sh:116-117`).
  Loopback: probe `127.0.0.1:9100` (up) vs a closed port (down) to exercise both
  branches. **Real peer genuinely required only** for an end-to-end direct post to a
  *different* host — defer that to the canary, not the unit test.

### S3 — Direct delivery confirm via sidecar journaled-receipt
- **Ladder mapping (see §3).** On the direct path, level-2 "delivered" = the
  recipient's **sidecar** journals the inbound turn and emits a `msg_type=receipt`
  (mechanism A) — the SAME envelope shape `agent-respond.sh:94` already produces and
  `agent-send.sh:259-266` already polls. **No hub frontier (B) on the direct path**
  (AC2).
- **Files/functions:**
  - `notify-sidecar.sh`: on detecting an inbound dm: turn (it already probes
    `channel unread`, lines 167-176), additionally (a) append to the S1 journal and
    (b) auto-post the level-2 receipt (`channel post --msg-type receipt --up-to`).
    This makes the direct path **store-and-forward**: the journal+receipt survive a
    recipient restart (T-2291 §Dialogue confirm-ladder point 2).
  - `agent-send.sh`: on `--transport direct`, the receipt poll target is the peer's
    own hub topic (already handled by `hub_args`), but the *producer* is the sidecar,
    not a woken interactive agent — so the doorbell-ring loop becomes optional on the
    direct path (the sidecar acks without an LLM turn).
- **New surface:** none new; reuses mechanism A. The sidecar gains a journaling +
  auto-receipt responsibility (document in
  `docs/operations/deterministic-notify-sidecar.md`).
- **Test without a peer:** run sidecar `--once` against a self-posted dm: turn
  (TERMLINK_NOTIFY_TEST_UNREAD hook, `notify-sidecar.sh:146-148`), assert (a) journal
  row, (b) a receipt envelope appears on the topic. Then run `agent-send.sh
  --transport direct` against loopback and assert DELIVERED off that receipt.

### S4 — Try-direct / fall-back-to-hub orchestration (default)
- **Decision logic:** `--transport auto` (now default): probe peer host (S2). If
  TCP-reachable → direct (S3 confirm). If **host unreachable** (not merely
  agent-busy — the sidecar journal covers busy, T-2291 §Dialogue) → fall back to
  hub store-and-forward + frontier confirm (B) via existing `--await-ack` path.
- **Failover behavior:** bounded probe (≤2-3s, well under the 10s connect budget);
  on timeout emit a LOUD `agent-send: FALLBACK host <addr> unreachable → hub
  store-and-forward` line, then run the existing hub leg. Never silently downgrade.
- **Interaction with V3a/V3b:** V3a sidecar wakes the recipient on *either*
  transport (it polls the hub topic regardless). V3b confirm-default stays the
  contract — the SINGLE sender-API confirm (DELIVERED/FAILED loud) is unchanged; only
  the *receipt source* differs by transport (A on direct, B on fallback). One confirm
  contract, two producers (T-2291 §Dialogue).
- **Files/functions:** `agent-send.sh` orchestration block (rewrite the
  `hub_args`/ring/poll section, lines 216-307, into a transport-branch).
- **Test without a peer:** loopback-up → direct branch; closed-port (simulated
  down) → fallback branch, assert the FALLBACK line + DELIVERED via frontier.
  **Real second host** required for the genuine "host down mid-fleet" canary only.

### S5 — Journal-authoritative + firehose suppression for dm:
- **The actual "off the firehose" move (AC3).** Two options:
  - (a) **Hub-side suppression:** `handle_channel_post_with` (`channel.rs:512`)
    routes `dm:` + `msg_type in {turn,receipt}` to a per-conversation journal table
    instead of the main topic log (presence/broadcast/store-and-forward stay).
  - (b) **Client-side authoritative journal + retention-trim of dm: on the
    firehose.** Lower blast radius; journal (S1) becomes the read source, a reaper
    trims dm: turns from the bus store after journaling.
  - **Recommend (b) for v1** (smaller blast radius, reversible; the firehose stays a
    short-window store-and-forward buffer, journals are the durable record). (a) is
    the cleaner end-state but touches the hot post path — defer or do behind a flag.
- **Fleet forensic:** aggregate-on-demand across journals = the
  `/recent-dm`-walks-hubs pattern (`scripts/agent-recent-dm.sh`) re-pointed at
  journals instead of `channel subscribe`.
- **Files/functions:** `channel.rs:512` (if option a) OR a journal reaper +
  re-point `agent-recent-dm.sh`/`agent journal` (option b).
- **Test without a peer:** post N dm: turns + M heartbeats to loopback; after
  journal+trim, assert dm: turns absent from `channel subscribe` but present in
  `agent journal`; assert heartbeats untouched. AC3 is loopback-verifiable.

## 3. The 3-level confirm ladder

| Level | Meaning | Direct path producer | Fallback path producer |
|---|---|---|---|
| **L1 TCP-ack** | bytes on wire | (ignored as delivery — T-2291 §Dialogue) | (ignored) |
| **L2 delivered** | in recipient mailbox | **(A)** sidecar journals turn + posts `msg_type=receipt` (`agent-respond.sh:94` shape) | **(B)** `channel.receipts` frontier (`ack_retry.rs:107`, `--await-ack`) |
| **L3 read/acted** | consumed at yield point | **(A)** read-receipt: agent's `notify-check.sh` exit-10 → posts a second receipt / reply turn **(C)** | same — read-receipt is transport-agnostic |

- **Delivered-vs-read split (V3b deferred this to V6):** today V3b's single receipt
  conflates them — `agent-send.sh` treats the first `msg_type=receipt` as DELIVERED.
  V6 splits by **a `stage` metadata field on the existing mechanism-A envelope**:
  `--metadata stage=delivered` (sidecar, auto, no LLM) vs `--metadata stage=read`
  (agent, at yield point) vs the reply turn = `acted` (C). **No new receipt
  namespace** — extends mechanism A with one metadata key, honoring V3b's decision
  that conversational paths use mechanism-A envelopes. `agent-send.sh:262` poll
  becomes stage-aware; `--await-reply` (line 277) already covers L3-acted.
- **Mechanism B stays fallback-only** (AC2: "no hub receipts-frontier on the direct
  path"). B is invoked solely by S4's hub fallback leg.

## 4. Direct-first + hub-fallback decision logic (summary)

```
resolve peer (fleet presence) -> agent_id, observed_addr (T-2297) | self_addr (T-2293)
if --transport == hub:    go hub leg (today's path)
if --transport in {auto,direct}:
    probe = remote ping observed_addr  (bounded ~2-3s, remote.rs:1071)
    if probe OK:   DIRECT  -> post to peer's own hub; confirm via mechanism A (sidecar)
    elif auto:     FALLBACK (loud) -> hub store-and-forward; confirm via mechanism B (frontier)
    else (direct): FAIL loud (host unreachable; no fallback requested)
```
- Busy-but-up recipient ⇒ direct succeeds (sidecar journals while agent is mid-turn).
- Host-down ⇒ fallback. Fallback is the ONLY trigger for the hub leg + frontier.
- V3a sidecar must be running on the recipient for L2-direct; if its heartbeat is
  stale, `notify-check.sh` returns DEAF (exit 3) and the recipient self-halts — the
  sender's missing L2 receipt then drives retry/fallback. (Self-detecting deafness,
  T-2291 RC3a.)

## 5. Risks / open questions for the human

1. **T-2297 sequencing.** S2 can ship on self-reported addr (T-2293, shipped) but
   that is renumber/spoof-prone. Block S2 on T-2297 for the hub-attested addr, or
   ship S2 on self-report now and harden later? (Recommend: ship S2 on self-report
   for the flat /24, mark a follow-up to prefer T-2297 addr.)
2. **Firehose suppression blast radius (S5).** Option (a) hub-side touches the hot
   `channel.post` path and changes what `channel subscribe dm:*` returns fleet-wide
   (any tooling reading dm: off the firehose breaks). Option (b) client-side is
   safer but leaves a short firehose window. Which end-state does the human want?
3. **Sidecar as auto-acker (S3).** Auto-posting an L2 receipt from a no-LLM sidecar
   means "delivered" no longer implies a *cognitively present* agent. That is the
   intended semantic (delivered != read), but confirm it is acceptable that a
   running-but-wedged agent still reports delivered.
4. **Same-host / loopback direct path** depends on T-2024 (DEFERRED). V6 cross-host
   is fine (auth already symmetric on LAN per spike B.2); same-host direct is
   out-of-scope until T-2024.
5. **mDNS Tier-1 discovery** (T-2291 two-tier) was rejected once in T-006. This plan
   uses the existing fleet-presence/`remote_store` registry (Tier-2) only and does
   NOT build mDNS. Confirm Tier-1 broadcast is out of V6 scope.
6. **Journal compaction/retention.** New per-conversation SQLite stores grow
   unbounded; need a retention policy (mirror `dead_letters` discipline). Defer to a
   follow-up or include in S1?

## 6. Critical files
- `crates/termlink-cli/src/commands/remote.rs` (`connect_remote_hub:719`, `cmd_remote_ping:1071`, `command.inject:1546`)
- `scripts/agent-send.sh` (transport-select + orchestration brain)
- `scripts/notify-sidecar.sh` + `scripts/notify-check.sh` (sidecar journaling + L2/L3 receipts)
- `crates/termlink-hub/src/channel.rs` (`handle_channel_post_with:512`, `handle_channel_receipts:957`)
- `crates/termlink-session/src/offline_queue.rs` (journal store template) + `ack_retry.rs` (frontier confirm)
