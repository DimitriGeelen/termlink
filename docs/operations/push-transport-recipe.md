# Push-transport operator recipe (arc-004)

**What this is:** the single operator-facing walkthrough for the shipped arc-004
`push-transport` capability — instant hub→client wake for a live agent, replacing
the ~15 s doorbell-then-poll floor with a sub-second push, degrading cleanly back
to polling when the socket drops. It consolidates ~12 per-task reports into one
navigation hub; each section links the report that shipped it.

**Status:** arc-004 is **closed / shipped** (`.context/arcs/push-transport.yaml`).
This recipe documents what shipped; it is not a reopen.

---

## 1. Mental model — push is a faster TRIGGER, never a source of truth

The durable substrate is unchanged and remains authoritative:

```
   ┌─────────────────────────────────────────────────────────────┐
   │ DURABLE (authoritative, unchanged by arc-004):               │
   │   dm:/inbox: topics · receipts · journal · idempotency keys  │
   │   · offline queue (outbound.sqlite)                          │
   └─────────────────────────────────────────────────────────────┘
                          ▲ read/write
   ┌──────────────────────┴──────────────────────────────────────┐
   │ WAKE/READ TRANSPORT (what arc-004 changed):                  │
   │   push (WS) ── instant ──▶ else degrade ──▶ 1 s poll ──▶ else │
   │   the durable /check-arc cadence is still the floor          │
   └─────────────────────────────────────────────────────────────┘
```

**Invariant:** a WebSocket push is only a faster way to learn "there is something
to read." It never carries authority. If the socket is down, the durable topics +
receipts + offline queue still deliver — just at poll latency, not sub-second.
Never treat a push frame as the message of record; always read the durable topic.

Origin: T-2303 inception §10, `docs/reports/T-2303-push-transport-inception.md`.

## 2. Enable instant push wake for a live agent

The production path is one keystroke — `/be-reachable` (or `be-reachable.sh
start`) spawns a **registered push-waker** alongside the presence heartbeat:

```bash
/be-reachable start          # registers the session on agent-presence AND spawns
                             # the push-waker (records pushwaker_pid in
                             # ~/.termlink/be-reachable.state)
```

Mechanically, the waker holds `channel subscribe inbox.queued --push` and, on each
`inbox.queued` aggregator frame addressed to this agent, fires the existing PTY
doorbell (`termlink inject <pty_session> "/check-arc respond" --enter`). An inbox
deposit therefore rings the agent the instant it lands, instead of on the next
poll cycle.

- The waker is spawned as a `setsid` process-group leader so
  `/be-reachable stop` reaps it (and its `subscribe` child) atomically — see
  T-2319 + PL-234. Always `/be-reachable stop` at session end.
- Shipped in T-2316 (`docs/reports/T-2316-arc-004-pushwaker-demo.md`), proven
  end-to-end with a real spawn+inject in T-2318
  (`docs/reports/T-2318-arc-004-pushwaker-e2e-demo.md`).

**Requirement (why a bare `--push` gets nothing):** `--push` only carries
aggregator `hub.event` frames and needs a **registered session** as the sink; and
`inbox.queued` only fires for a *successful* post to an existing `inbox:<id>`
topic. `/be-reachable` sets this up. Do not expect a raw `channel subscribe
--push` on an arbitrary topic to deliver — see PL-235.

## 3. The `channel subscribe --push` CLI (remote/manual use)

```bash
termlink channel subscribe <topic> --push --hub <host:port>
```

- `--push` opens a WebSocket to a **TCP** hub and prints pushed `hub.event` frames
  the instant they arrive, instead of the 1 s poll floor. It **requires** a
  `host:port` `--hub` with a matching `hubs.toml` profile (add one with
  `termlink remote profile add <name> <host:port> --secret-file <path>`).
- If no profile matches the TCP address, you get:
  `[push] WS unavailable (no hubs.toml profile matches TCP address … )` and it
  degrades to catch-up polling.
- WS-over-Unix push for a co-located agent is a follow-on (T-2313; ~31 ms
  delivery measured).

## 4. What happens when the socket drops (degrade + reconnect)

Push is best-effort; the durable substrate is the floor. On a socket drop:

1. The CLI's built-in **active reconnect** retries the WS with backoff
   (T-2314, `docs/reports/T-2314-arc-004-active-reconnect-demo.md`).
2. While disconnected it **catches up by polling** so nothing is missed.
3. If the WS stays down, `--push` runs its own poll loop (1 s) and the durable
   inbox / sender-ring / `/check-arc` cadence remains authoritative.
4. The push-waker resumes ringing after the blip with **no double-wake**
   (`/check-arc respond` is idempotent; the waker dedups per offset) — proven in
   T-2317, `docs/reports/T-2317-arc-004-pushwaker-blip-demo.md`.

## 5. Measured value

| Path | Latency | Source |
|------|---------|--------|
| **arc-004 push-wake (measured)** | **~85–111 ms median** (upper bound, full wake path) | `docs/reports/T-2320-arc-004-pushwake-latency-benchmark.md` |
| `--follow` poll fallback | ~500 ms mean (1 s poll) | `channel subscribe --follow` help |
| Pre-push doorbell-then-poll floor | ~15 s | T-2303 §10 |

Reproduce the measurement: `bash scripts/bench-pushwake-latency.sh` (hermetic;
median sub-second = PASS). The push-wake is ~135–175× faster than the old 15 s
floor.

## 6. Failure modes — operational reading

| Symptom | Meaning | Action |
|---------|---------|--------|
| `/be-reachable status` shows no `pushwaker_pid` (or pid dead) | Waker never spawned / died | `/be-reachable stop` then `start`; check `~/.termlink/be-reachable.state`; confirm `jq` + a valid `--pty-session` |
| `[push] WS unavailable (no hubs.toml profile matches TCP address …)` | `--push` has no profile for that TCP hub | `termlink remote profile add <name> <host:port> --secret-file <secret>`; or pass a Unix-path `--hub` for the local hub |
| Push was live, now `[push] reconnecting to WS …` lines | Socket dropped; degrade+reconnect engaged (expected) | None — it catches up by polling and resumes; only investigate if it never reconnects (check hub reachability, `fleet doctor`) |
| Inbox deposit does not ring the live agent | `inbox:<id>` topic missing, or the post errored before the `inbox.queued` emit, or the session is not registered | Ensure the agent ran `/be-reachable start` (registers the session); confirm the `inbox:<id>` topic exists; verify the post succeeded (a failed post never emits `inbox.queued`) — see PL-235 |
| Agent woke twice for one deposit | Cross-rail double-wake (inbox push + live-sender ring) | Benign — `/check-arc respond` is idempotent; a second run finds nothing new (IW-3) |

## 7. Durability invariant (do not violate)

arc-004 changed only the wake/read transport. It did **not** replace the journal,
receipts, idempotency keys, or the offline queue — those stay authoritative. A
push frame is a trigger to read the durable topic, never the record itself. If you
find yourself treating a WS frame as source-of-truth, stop: read the durable topic
and the receipts.

## 8. Map — where each piece shipped

| Piece | Task | Report / registry |
|-------|------|--------|
| Inception (GO scoped: WS live path; webhooks deferred) | T-2303 | `docs/reports/T-2303-push-transport-inception.md` |
| Hub WS upgrade + client subscribe | T-2309 / T-2310 | `docs/reports/T-2310-arc-004-ws-push-demo.md` |
| WS-over-Unix push (co-located) | T-2313 | (report in-tree; ~31 ms delivery) |
| Active reconnect to WS | T-2314 | `docs/reports/T-2314-arc-004-active-reconnect-demo.md` |
| Wake-path integration GO (Option A) | T-2315 | `docs/reports/T-2315-arc-004-wake-path-integration-inception.md` |
| Push-waker (WP1) — WS now load-bearing | T-2316 | `docs/reports/T-2316-arc-004-pushwaker-demo.md` |
| Push-waker blip/reconnect (WP2) | T-2317 | `docs/reports/T-2317-arc-004-pushwaker-blip-demo.md` |
| Live E2E proof (real spawn+inject) | T-2318 | `docs/reports/T-2318-arc-004-pushwaker-e2e-demo.md` |
| Stop-reap leak fix (process-group kill) | T-2319 | `.tasks/completed/T-2319-*.md`; PL-234 |
| Latency benchmark (retires §10 gap) | T-2320 | `docs/reports/T-2320-arc-004-pushwake-latency-benchmark.md` |

Arc registry: `.context/arcs/push-transport.yaml`.

**Deferred (not shipped — needs a human inception if external demand
materialises):** webhooks for external/non-interactive consumers (Watchtower /
Slack / CI fan-out). Per T-2303 §10, webhooks are **not** a path for
agent-to-agent delivery.
