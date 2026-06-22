# TermLink communications analysis (read-only discovery)

<!-- DISCOVERY-termlink-comms-analysis-2026-06-22 / task T-2241 -->
<!-- BINDING: read-only investigation. No fixes, no instrumentation, no state change.
     This document characterizes; it does not recommend or authorize a fix.
     Candidate causes are named with evidence; no fix/verb/config is proposed. -->

> **TermLink comms analysis. Window: 2026-05-28 → 2026-06-17 (20.1 d).
> Scale: 10 local sessions / 1420 topics / 5 hosts (4 up). Dominant traffic class:
> presence (70.5%; `agent-presence` alone = 68.9% of all messages). Cost located in:
> fan-out / presence accumulation (matches the T-1991 prediction), NOT hub serialization.
> Top failure signature: restart-blind-window — presence producers did not re-register
> after the 2026-06-17 hub restart, leaving ~30k stale heartbeats as the bulk of the store;
> spoke-side losses (discards/flaps/RTT) are UNMEASURABLE. Spoke-plane: NOT CAPTURED
> (Q1 = out-of-band default; no client-side instrumentation exists to collect from).
> Verdict matches T-1991 prediction: YES — with a new finding that the cv_index discovery
> mitigation (T-2107) is INACTIVE on this hub.**

## Q1 — Sovereign choice (SURFACED, not answered)

The spoke-plane collection method (in-band vs out-of-band) is a Sovereign decision reserved
for the human. It was **unresolved at investigation start**, so per the brief's §2 I took the
**default**: out-of-band for failure/historical data; in-band only for a single live presence
snapshot.

In the event the choice is **moot for this round**: Phase 0 found that no client-side
spoke-plane diagnostics are persisted anywhere (see §C / F-INSTRUMENTATION), so neither method
has a dataset to collect — out-of-band dumps do not exist and in-band querying would only
return the same live presence the hub already holds. **The default stands and is flagged as a
pending Sovereign decision** for any future round that first adds the missing instrumentation.

---

## §A Capability map (Phase 0) — what was readable, what was absent

| Signal | Read path | Status |
|---|---|---|
| Fleet/session/topic scale | `termlink fleet_status`, `list_sessions`, `channel list` | **readable** |
| SQLite store location | `ls`/`find` under `/var/lib/termlink/bus/` | **readable (files)** |
| SQLite **schema** | `sqlite3 .schema` | **ABSENT** — blocked by G-020 task-gate hook / T-559 project-boundary allowlist; schema read from source const instead |
| Topic inventory + retention | `termlink channel list` (`count`, `retention`) | **readable** |
| Per-topic volume/rate | `termlink channel topic-stats` (first/last_ts) | **readable** |
| Traffic-class split | `channel list`, name-classified | **readable (derived)** |
| Fan-out (posts × subscribers) | `channel info`/`members` | **PARTIAL** — topics persist *senders* only, no subscriber roster; delivered-volume only estimable |
| Per-subscriber cursor lag | `termlink channel receipts` | **ABSENT for heaviest topic** — `receipts=[]` on `agent-presence`; present (316) on chat-arc |
| Presence share + cv_index mitigation | `channel cv-keys`, `hub status --governor` | **readable** |
| Governor counters | `termlink hub status --governor --json` | **readable** (via CLI; MCP `governor_status` was DENIED) |
| Inbox spool / queue depth | `inbox_status`, `channel queue-status` | **readable** |
| Hub restart history | `ps lstart/etime`; `rpc-audit.jsonl` | **PARTIAL** — current uptime readable; restart *count* not individually countable |
| **Spoke-plane: discards/flaps/CB/reconnect/RTT** | code + `~/.termlink/*` logs | **ABSENT** (F-INSTRUMENTATION — see §C) |

**Tooling caveats encountered (recorded plainly, not worked around):** `sqlite3` against the
live store was blocked by the task-gate hook (T-559 boundary), so DB schemas were read from the
authoritative source constant rather than a live `.schema` dump. Several MCP verbs
(`governor_status`, `agent_presence_now`, `agent_listeners_fleet`, `substrate_status`) returned
DENIED to the collector; equivalent `termlink` CLI subcommands were used instead and are cited
per metric.

## §B Hub-plane dataset (Phase 1) — observed, reliable

**Window:** `agent-presence` spans 2026-05-28 → 2026-06-17 = **20.09 days (482 h)**, retention
`forever` (no compaction in effect). Collected 2026-06-22 on local hub `/var/lib/termlink`
(pid 3024).

**Scale:** 5 hubs configured (4 up; `laptop-141` down). Local hub: **10 sessions, 1420 topics,
44,696 total messages.** *(read: `fleet_status`, `list_sessions`, `channel list`)*

### Traffic-class breakdown (the load-bearing measurement)

| Class | Messages | % of total | Topics | Members |
|---|---|---|---|---|
| **presence/heartbeat** | 31,527 | **70.5%** | 86 | `agent-presence`, `agent-listeners-*` |
| announcement/coordination | 4,758 | 10.6% | 8 | `agent-chat-arc`, `framework:*`, `broadcast:global`, `channel:learnings`, `health:*` |
| work/dispatch/result | 3,903 | 8.7% | 345 | `dm:*`, `agent-conv*`, `claim`/`dispatch`, `substrate*`, `stress*` |
| other (smoke/test/T-*) | 4,508 | 10.1% | 981 | test fixtures |

*(read: `channel list`, name-classified)*

### Per-item detail (read path cited)

1. **Store layout** — `bus/meta.db` 5.0 MB + **1393 per-topic append `.log` files = 118 MB** +
   **`rpc-audit.jsonl` = 1.36 GB, unbounded** (no rotation/compaction). *(ls/find)*
2. **Topic inventory** — 1420 topics; retention split: **forever = 1166, messages = 251,
   days = 3**. Heaviest: `agent-presence` 30,782 (forever), `agent-chat-arc` 3,786 (forever),
   `broadcast:global` 534 (msgs/1000), `channel:learnings` 285 (forever). *(channel list)*
3. **Volume/rate** — `agent-presence` = **1.06 posts/min sustained over 20.09 d**. *(topic-stats)*
4. **`agent-presence` is the elephant** — **68.9% of ALL hub messages live in this one topic**
   (30,782 / 44,696). Single sender (`d1993c2c3ec44c94` — the shared-host key); 30,781 of 30,782
   are `heartbeat` events. *(channel members/info)*
5. **Fan-out** — not directly measurable (no persisted subscriber roster). Estimated delivered
   volume ≈ posts × ~10 readers ⇒ ~307k delivered messages for presence alone over the window.
   *(channel members)*
6. **Cursor lag** — **not readable on the heaviest topic** (`receipts=[]` on `agent-presence`),
   so reader-keeping-up cannot be assessed where it would matter most. *(channel receipts)*
7. **cv_index mitigation — INACTIVE on this hub.** `cv-keys agent-presence` count = 0; governor
   `cv_index_entries_active = 0`, `topics_active = 0`, `overflow = 0`. Producers are **not**
   emitting `metadata.cv_key` here, so the T-2107 O(K) discovery fast-path is not engaged — late
   joiners still pay the O(N_heartbeats) ≈ 30k-envelope walk. *(cv-keys, governor)*
8. **Governor** — connections 2/256; **capacity_hits = 0, rate_hits = 0, dedupe_hits = 0,
   cv_index_overflow = 0** → no active refusals/backpressure firing now. BUT
   **`rate_buckets_evicted_total = 599,406`** = very high sender/connection turnover over uptime.
   *(`hub status --governor --json`)*
9. **Inbox/queue** — hub inbox: **19 pending transfers across 7 stale smoke targets, never
   drained** (no compaction). Client offline queue: drained (pending = 0, cap 1000).
   *(inbox_status, queue-status)*
10. **Restart/uptime** — pid 3024 up **4 d 18 h** (since 2026-06-17 17:21), single continuous run.
    `agent-presence` `last_ts` **predates this start** ⇒ the presence producers detached at the
    last restart and **did not re-register** (the local `be-reachable.log` reads "Terminated").
    Restart *count* not individually countable from current state. *(ps lstart/etime)*

## §C Spoke-plane dataset (Phase 2) — INSTRUMENTATION GAP (F-INSTRUMENTATION)

Per the Q1 default the spoke-plane was to be collected out-of-band. Phase 0 found **there is
nothing to collect**: TermLink persists essentially **no durable spoke-plane diagnostics**.
Every spoke-side failure signal is either handled loudly in-memory and discarded, or computed
on-demand at read time and printed. Verdict per signal (code paths cited):

| Signal | Status | Evidence |
|---|---|---|
| **A. Discarded outbound posts** | **ABSENT** | `offline_queue.rs` schema is a single `pending_posts` table (id, post_json, enqueued_ms, attempts); `pop()` DELETEs on both delivery AND poison-drop → no dead-letter, no trace. Poison drop (after `POISON_THRESHOLD=10`) is a `tracing::warn!` (ephemeral stderr) + an in-memory `FlushReport.dropped_poison` counter that is returned and discarded. *(bus_client.rs:181-237, offline_queue.rs:106)* |
| **B. Liveness flaps** | **ABSENT** | `liveness.rs` is a boolean `is_alive()` (PID `kill(0)` + socket check). No LIVE/STALE/OFFLINE enum, no transition log. LIVE/STALE is computed at render time and printed. *(session.rs:1123, agent.rs:1544)* |
| **C. Circuit-breaker trips** | **ABSENT (spoke-side)** | Only circuit breaker is hub/router-side (`termlink-hub/src/circuit_breaker.rs`, router failover + MCP model fallback), in-memory, tracing-only, no durable trip log. No client transport breaker exists. |
| **D. Reconnect attempts/failures** | **ABSENT** | No dedicated reconnect loop; "reconnect" = the BusClient background flush re-attempting every ~5 s (±25% jitter). Per-post `attempts` lives in the pending row and is deleted on success; transport failures emit `tracing::debug!` only. *(bus_client.rs:112-145)* |
| **E. Client send/RTT latency** | **ABSENT as a series** | Measured ad-hoc in one-shot paths (ping `latency_ms`, exec, pty) and printed to stdout/JSON only — never persisted. MCP `response_latency` computes from **hub** event timestamps server-side, not a client-measured send RTT. *(session.rs:687/700)* |

**Client-side files found in `~/.termlink/`:** `outbound.sqlite` (pending posts only; deletes on
delivery/drop), `rotation.log` (3 lines — auth/cert transitions, not failure diagnostics),
`be-reachable.log` (11 bytes, "Terminated"), `cursors.json` (read offsets). **Absent:**
`queue.log`, `governor.log`, `heal.log`, `find-idle.log`, `claims.log` — these exist only when an
operator opts in with `--log` on a `--watch` loop, and even then record coarse binary state flips,
not per-event discards.

**Consequence:** the failure class most worth analyzing (a worker dropping a "complete" during a
hub blip) is, by construction, invisible at both planes. This is a successful finding, not a
failed collection — the measurement surface for spoke-plane failures is thin to the point of
absence.

## §D Findings (F-series) — evidence / confidence / implication

**F1 — Cost is located in presence/announcement fan-out, NOT hub serialization (the §6 fork
resolved).**
*Evidence:* presence class = 70.5% of all messages; `agent-presence` alone = 68.9% (30,782 /
44,696), retention `forever`, growing monotonically at 1.06 posts/min. Serialization shows no
bottleneck: 2/256 connections, zero capacity/rate/dedupe hits. *Confidence:* HIGH that presence
dominates raw volume (direct count); MEDIUM that it is THE operational cost, because the
delivered (fan-out) volume is estimated not measured and discovery latency is not instrumented.
*Implication:* matches the substrate design's T-1991 prediction.

**F2 — The cv_index discovery mitigation (T-2107) is INACTIVE on this hub — a NEW finding.**
*Evidence:* `cv-keys agent-presence` count = 0; governor cv_index fields all 0. Producers are not
emitting `metadata.cv_key`. *Confidence:* HIGH (direct read). *Implication:* the documented O(K)
fast-path that was supposed to neutralize the T-1991 walk is not engaged here; late joiners still
pay the O(~30k-envelope) presence walk. The mitigation exists in code but is not in force on this
hub.

**F3 — Top failure signature is the restart-blind-window, and it is currently realized.**
*Evidence:* `agent-presence` `last_ts` predates the 2026-06-17 17:21 hub start; local
`be-reachable.log` = "Terminated". *Confidence:* HIGH. *Implication:* after the last restart the
presence producers did not re-register, so the 30k-message topic is now a large, mostly-stale
accumulation — discovery reads volume that no longer reflects live agents. This is the
restart-blind-window mechanism observed in the wild, not hypothetically.

**F-INSTRUMENTATION — Spoke-plane failures are not captured anywhere.**
*Evidence:* §C table (A–E all ABSENT, code paths cited). *Confidence:* HIGH. *Implication:* loss
mechanisms (discarded-outbound, cursor-lag-drop, restart-blind-window) cannot be ranked by
frequency × impact because only the third is even partially observable from the hub. Any future
efficiency work that needs failure rates must first add the missing instrumentation; this
discovery cannot supply them.

**F4 — Secondary storage-growth signals, unbounded.**
*Evidence:* `rpc-audit.jsonl` = 1.36 GB with no rotation; 19 inbox transfers to 7 stale smoke
targets never drained; 981 of 1420 topics are smoke/test fixtures; `rate_buckets_evicted_total =
599,406`. *Confidence:* HIGH (direct read). *Implication:* growth/churn is real but is an
audit-log and namespace-hygiene concern, distinct from the message-path cost in F1.

**F5 — No work-class coordination-volume smell (A1.11 check).**
*Evidence:* the work class is 8.7% across 345 topics (avg ≈ 11 msgs/topic); no single
agent-pair or dm:* topic shows runaway coordination volume. The high volume is entirely machine
heartbeat chatter, not human-meaningful coordination. *Confidence:* MEDIUM (per-pair detail
limited by sender-only rosters). *Implication:* there is no task-decomposition smell to flag —
the volume problem is infrastructure broadcast, not over-coupled work.

## §E What I could not measure — explicit, named gaps

1. **SQLite schemas** — `sqlite3` blocked by the task-gate / T-559 boundary; schema read from the
   source constant, not a live `.schema` dump.
2. **Exact fan-out amplification** — topics persist senders only, no subscriber roster; delivered
   volume is an estimate (×~10 readers), not a measurement.
3. **Per-subscriber cursor lag on the heaviest topic** — `receipts=[]` on `agent-presence`; the
   backpressure symptom cannot be read where it matters most.
4. **All spoke-plane failure rates** — discards, flaps, circuit-breaker trips, reconnects, client
   RTT (F-INSTRUMENTATION).
5. **Dispatch round-trip / discovery latency from the requester side** — measured ad-hoc, never
   persisted; no time series exists.
6. **Restart history / count** — only current uptime is readable; `rpc-audit.jsonl` may hold the
   history but was not parsed (1.36 GB; out of scope for a read-only pass).
7. **The agent-count scale knee** — this hub has a SINGLE presence sender, so the T-1991
   ~20-agent inflection cannot be observed here; the growth seen is duration-driven, not
   agent-count-driven. Confirming the knee needs a multi-agent hub.
8. **Fleet-wide hub-plane** — only the local hub (`/var/lib/termlink`) was collected in depth;
   the other 3 reachable hubs were not deep-sampled.

---

*Boundary statement: this report names candidate causes tied to evidence. It proposes no fix, no
new verb, no instrumentation, and no config change. Improvement-derivation is a separate
Sovereign + design step. Research is not authorization.*
