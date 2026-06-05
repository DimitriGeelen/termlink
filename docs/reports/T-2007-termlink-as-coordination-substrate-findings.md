# T-2007 — TermLink as Coordination Substrate: Findings Report

**Date:** 2026-06-05
**Owner:** human (decision); agent (research)
**Status:** research complete; design choice deferred to human per task scope
**Source-of-truth:** /opt/termlink Rust workspace + scripts/ shell wrappers at HEAD
  `4b69db68` (see `git log -1 --format=%H` at report time)

## Question

Can TermLink, as it exists today, serve as the coordination substrate for parallel
multi-agent task execution across a distributed homelab (multiple hosts, multiple
agents per host, communicating over IP including loopback for same-host agents)?

The intended model is hub-and-spoke. Isolated code work happens on per-host
checkouts (code plane). All shared/governance state — task ledger, audit logs, arc
YAML, episodic memory — is serialized through one central hub; spokes never write it
directly (governance plane). Safety comes from disjoint write-sets between
concurrently-running tasks, not from merging.

This report **describes what is**, with file:line citations. It does not recommend an
architecture; the design decisions belong to the human.

## Method

Five parallel `Explore` subagents read the actual TermLink Rust source plus the
shell wrappers. Findings were synthesized into the answers below. Where a feature
is absent, that is stated plainly; "it doesn't do this" is a finding, not a failure.

Files read (sample): `crates/termlink-hub/src/{server,router,channel,inbox,supervisor,circuit_breaker,aggregator,remote_store,tls}.rs`,
`crates/termlink-bus/src/{lib,log,meta}.rs`,
`crates/termlink-protocol/src/{control,transport,events,lib}.rs`,
`crates/termlink-session/src/{client,auth,events,executor,scrollback}.rs`,
`crates/termlink-cli/src/commands/{agent,channel,dispatch,execution,help}.rs`,
`crates/termlink-mcp/src/tools.rs`,
`scripts/{listener-heartbeat,be-reachable,agent-send,agent-listeners}.sh`.

## 1. Messaging shape

Four patterns coexist; **all are mediated through the hub**.

- **Point-to-point.** `event.emit_to` — JSON-RPC unicast to a named session.
  `crates/termlink-protocol/src/control.rs:73`, dispatch in
  `crates/termlink-hub/src/router.rs:77`.
- **Broadcast.** Legacy `event.broadcast` is retired; the surviving form is a
  hub-side mirror that dual-writes into the `broadcast:global` channel topic.
  `crates/termlink-hub/src/channel.rs:72-143`.
- **Publish-subscribe (the primary surface).** `channel.create`, `channel.post`,
  `channel.subscribe`, `channel.list` over the bus crate — a SQLite-backed per-topic
  append-only log with monotonic offsets.
  `crates/termlink-bus/src/lib.rs:125-138`, `crates/termlink-hub/src/channel.rs:1-10`.
  Any agent posts; any agent reads at its own cursor. Topics are not addressed in
  the message — every subscriber sees every post.
- **Blackboard-style shared state.** Two ingredients: (a) the `agent-presence`
  topic that agents heartbeat into via `listener-heartbeat.sh`
  (`scripts/listener-heartbeat.sh:10-26`, `scripts/be-reachable.sh:46-80`);
  (b) a hub-side in-memory `PresenceTracker` keyed by
  `(conversation_id, agent_id)` queryable via `dialog.presence`.
  `crates/termlink-hub/src/channel.rs:28-70`. Both are convention-driven — the hub
  does not constrain what gets posted there.

Pub-sub is the load-bearing primitive. A "blackboard many can read, many can
write, no addressing" exists as `channel.post` to a shared topic name.

## 2. Topology

**Strict star.** Sessions do not connect to each other. Every cross-session RPC
goes through the local hub via `forward_to_target`
(`crates/termlink-hub/src/router.rs:115, 284, 342`), which resolves the target
through the local session manager or the remote-hub store and then opens a
hub→target socket. Cross-hub paths still loop through both hubs
(`crates/termlink-hub/src/remote_store.rs`). DMs are not addressed inline — they
are posted to a topic name derived from both fingerprints
(`crates/termlink-cli/src/commands/agent.rs:878`).

The hub CAN hold shared state visible to all spokes: that is exactly what channel
topics + the bus log + the in-memory presence tracker do. It can re-broadcast via
the `event.broadcast` → `broadcast:global` mirror.

## 3. Transport uniformity

The hub listens on BOTH a Unix socket AND TCP+TLS in parallel:
`UnixListener::bind` (`crates/termlink-hub/src/server.rs:187`) plus `tokio::select!`
over TCP (`crates/termlink-hub/src/server.rs:198-210, 457-592`).

**Loopback TCP works:** `crates/termlink-session/src/client.rs:54-98` handles
`TransportAddr::Tcp { host, port }` uniformly; same-host TCP (`127.0.0.1:9100`)
uses the local pinned cert at `runtime_dir/hub.cert.pem` (line 60).

But the transports are **not semantically identical**:
- Unix socket trusts on UID via `SO_PEERCRED` and grants `Execute` scope
  unconditionally (`crates/termlink-hub/src/server.rs:500-515`,
  `crates/termlink-session/src/auth.rs:28-53`). No HMAC, no TLS.
- TCP requires a `hub.auth` HMAC challenge plus TLS cert pinning before any
  non-auth RPC is allowed (`crates/termlink-hub/src/server.rs:537-574, 669-691, 371-448`).

Client selection is **explicit, not auto** — the caller passes either
`TransportAddr::unix(path)` or `TransportAddr::tcp(host, port)`
(`crates/termlink-protocol/src/transport.rs:12-44`). MCP defaults to the Unix
socket for the local hub; remote calls go TCP. No RPCs are transport-restricted;
the JSON-RPC payload is identical on either side.

Same-host and cross-host CAN ride the same path if the same-host client elects
loopback TCP. Out of the box, they do not — the default same-host path is
auth-bypassed UDS.

## 4. Serialization primitive

**Yes, but only one form: append-only ordered logs.** `channel.post` writes through
a per-topic `LogAppender` guarded by a `tokio::sync::Mutex<File>`
(`crates/termlink-bus/src/log.rs:27-65`); concurrent posters serialize on the lock
and each receives a monotonically increasing 64-bit offset assigned by SQLite
metadata (`crates/termlink-bus/src/lib.rs:125-138`). Each append `flush()`es to
disk (`crates/termlink-bus/src/log.rs:63`).

Per-subscriber read cursors are persistent: `put_cursor`/`get_cursor` over
`INSERT ... ON CONFLICT DO UPDATE` in `crates/termlink-bus/src/meta.rs:197-225`.
`channel.receipts` is a read-ack convention layered on top — receipt envelopes
carry `metadata.up_to` and the latest per-sender wins
(`crates/termlink-hub/src/channel.rs:661-731`). Receipts do not remove the
underlying message; nothing claims it; nothing locks it.

**Not found:** no `lock`, `lease`, `mutex`, `acquire`, or CAS RPC verb anywhere in
the crates. KV (`termlink_kv_*`) is a plain in-memory `HashMap` per session with
no version or compare-and-set surface.

## 5. Work distribution

**Push only.** A sender chooses a recipient explicitly:
- `agent contact <id>` resolves the peer via the `agent-presence` heartbeat topic
  (read by `scripts/agent-listeners.sh`) and posts to a derived DM topic
  (`crates/termlink-cli/src/commands/agent.rs:751-1001`, `scripts/agent-send.sh:88-162`).
- The "doorbell + mail" arc = (a) `channel.post` to the DM topic, (b) `inject` a
  wake-up into the peer's PTY session, (c) poll for a receipt envelope
  (`scripts/agent-send.sh:166-199`). It is broadcast-style with manual ack — not
  a job queue. Multiple readers of the DM topic would all see the same message.
- `termlink dispatch` spawns N named workers and then aggregates by polling each
  via hub-side `event.collect` (`crates/termlink-hub/src/router.rs:477-630`,
  `crates/termlink-cli/src/commands/dispatch.rs:170-756`). The orchestrator picks
  who runs what — workers do not pick up work.

**No idle agent → pull verb exists.** No "claim next available" semantics. No
exclusive delivery — every subscriber to a topic sees every message.

**No idle/busy registry inside the hub.** Heartbeats land in the `agent-presence`
topic and that is all the hub does with them. Liveness classification
(`LIVE`/`STALE`/`OFFLINE`) is computed client-side by `jq` on the envelope's
`last_seen_ts` (`scripts/agent-listeners.sh:236-238`). The hub has no
`agent_state` table, no in-flight counter.

## 6. Live state / announcements

- **Streaming.** `event.subscribe` is long-poll JSON-RPC with a cursor and
  timeout (`crates/termlink-protocol/src/control.rs:61, 75`; bus aggregator at
  `crates/termlink-hub/src/aggregator.rs:71-138`). No SSE, no WebSocket, no
  server-push channel.
- **Presence/announcements.** Heartbeats to the `agent-presence` topic carry
  `agent_id`, `role`, `listen_topics`, `pty_session`
  (`scripts/listener-heartbeat.sh:10-26, 129-145`). A shared "everyone sees what
  everyone says" arc exists as the `agent-chat-arc` topic — any subscriber tails
  the log. Distinct from `agent contact`, which is point-to-point.
- **Filesystem observation — does not exist.** Exhaustive grep across the crates
  for `inotify`, `fanotify`, `notify::`, `audit`, `fs watch` turns up only
  build-time `cargo:rerun-if-changed` directives — nothing at runtime. The hub
  cannot see what files an agent touches; an agent that wants to advertise a
  path-claim must self-declare it by posting to a topic or setting a KV entry.

## 7. Integration / git

Almost none in the runtime. `build.rs` reads `.git/HEAD` to derive the binary
version; that is a developer-host build artifact, not runtime behavior.
`SpawnParams`, `RunParams`, `AgentRequest`/`AgentResponse` carry no git fields;
`termlink_file_send` is raw bytes into the inbox spool
(`crates/termlink-hub/src/inbox.rs`).

**One narrow exception.** `termlink dispatch --isolate` does shell out to git: it
creates a per-worker `git worktree` and optionally auto-commits + auto-merges
branches at cleanup time
(`crates/termlink-cli/src/commands/dispatch.rs:129-134, 287-305, 533-540, 579-667`).
This is the only place the Rust binary knows that git exists at all. There is no
`agent.checkout` / `agent.commit` / `agent.merge` RPC; outside dispatch's
worktree wrapper, TermLink is git-agnostic and treats agents as opaque processes.

## 8. Failure modes

- **Hub-side detection of dead spokes.** Passive only. The hub finds out a client
  is gone when its next write returns an I/O error and the read loop breaks
  (`crates/termlink-hub/src/server.rs:722-747`). There is no server-side ping or
  keepalive.
- **Supervisor.** Polls every 30 s, probes liveness via `kill(pid, 0)` + socket
  existence, emits a `session.exited` event to surviving sessions, cleans up
  stale registration files. No restart logic.
  `crates/termlink-hub/src/supervisor.rs:18, 23-168`.
- **Per-request timeouts.** Caller-supplied `timeout_ms` is honoured for
  forwarded RPCs via `tokio::time::timeout`
  (`crates/termlink-hub/src/router.rs:372, 410, 565`). `event.collect` divides
  total budget across targets (router.rs:540-541, min 100 ms per target).
- **Client-side reconnect — does not exist.** `crates/termlink-session/src/client.rs:134-154`
  returns immediately on `ConnectionClosed`. No retry loop, no backoff, no
  offline send buffer.
- **Channel durability.** Persistent. Append-only log + SQLite metadata under
  `<runtime_dir>/bus/`; each post flushes (`crates/termlink-bus/src/log.rs:63`);
  after a hub restart, `Bus::open()` reopens existing topics and subscribers
  resume from their persisted cursors (`crates/termlink-bus/src/lib.rs:51-61, 234-248`).
- **Inbox durability.** File-spool under `<runtime_dir>/inbox/<target>/<transfer_id>/`
  survives hub restart and is drained when the recipient registers
  (`crates/termlink-hub/src/inbox.rs:53-100`).
- **Presence tracker.** In-memory `RwLock<HashMap>` only — lost on hub restart
  (`crates/termlink-hub/src/channel.rs:36-70`). Heartbeats reappear within the
  heartbeat interval; the hub does not eject stale agents from the
  `agent-presence` topic.
- **Circuit breaker.** Hub-side, in-memory: 3 consecutive transport failures
  opens a 60 s circuit per session, with half-open probe on cooldown expiry
  (`crates/termlink-hub/src/circuit_breaker.rs:1-111`). Lost on restart. No
  spoke-side equivalent.
- **Spoke-side resilience when the hub dies.** None. Posts fail immediately; the
  agent has no local queue for deferred delivery.

## 9. Agent lifecycle

Launch is shell-mediated and surprisingly capable.

`termlink spawn` (`crates/termlink-cli/src/commands/execution.rs:312-466`):
- Picks a backend (`Terminal` via osascript on macOS, `tmux new-session -d` on
  Linux, `setsid sh -c` fallback).
- Builds a shell that backgrounds `termlink register --name <name>` and then
  runs the user command, with explicit env exports.
- Controllable fields (`SpawnOpts`, lines 298-310): `name`, `roles`, `tags`,
  `cap`, `env_vars`, `shell`, `command`, `wait`, `wait_timeout`, `backend`.
- The command is arbitrary — there is no built-in allowlist at spawn time. (A
  per-session `allowed_commands` prefix list exists in
  `crates/termlink-session/src/executor.rs:42-51`, but it only filters
  subsequent `command.execute` RPCs, not the spawn's own user command.)
- Parent env is NOT inherited by default — only `TERMLINK_RUNTIME_DIR` and
  explicitly-passed `--env KEY=VALUE` pairs.

`termlink run` is one-shot exec in the current process with a default 30 s
timeout (`cmd_run`, lines 15-142).

`termlink dispatch` adds N-worker fan-out: per-worker `--isolate` (git worktree),
`--workdir` (cd before user command), `--env`, `--model` (Claude model
selector), and a structured cleanup path (SIGTERM, optional auto-commit,
optional auto-merge). `crates/termlink-cli/src/commands/dispatch.rs:170-756`.

`agent_ask` is **not LLM invocation** — it posts an `agent.request` event to a
named target session and polls for `agent.response`. The receiver implements the
handler. Peer-to-peer delegation, hub-mediated.
`crates/termlink-cli/src/commands/agent.rs:14-200`.

So: yes, `termlink spawn` can launch an agent that does
`cd /tmp/worktree-1 && git checkout abc && claude -p "fix bug"` and then commit.
Working directory and env are operator-supplied. The launched process is fully
privileged with respect to the local filesystem and git; TermLink does not
sandbox it.

## 10. Scale & limits

Hard limits actually present in the code:

- Data-plane frame payload: **16 MiB** (`crates/termlink-protocol/src/lib.rs:40`,
  decoded in `data.rs:102`).
- Per-session event ring buffer: **1024** events; oldest drops
  (`crates/termlink-session/src/events.rs:13`).
- Per-session scrollback: **1 MiB** default (`crates/termlink-session/src/scrollback.rs:101`).
- Per-execute command length: **64 KiB** (`crates/termlink-session/src/executor.rs:14`).
- Default channel retention: **1000 messages** for `broadcast:global` and inbox
  topics (`crates/termlink-hub/src/channel.rs:85, 174`). Per-topic policy can
  also be `Days(N)` or `Forever`.
- Graceful-shutdown drain: 5 s (`crates/termlink-hub/src/server.rs:595-603`).
- Supervisor poll interval: 30 s (`crates/termlink-hub/src/supervisor.rs:18`).

**Limits NOT found** — nothing enforces them in code:

- No cap on concurrent hub connections. Accept loop `tokio::spawn`s per
  connection with no semaphore; bound is OS file-descriptor limit.
- No hard size cap on JSON-RPC frames (the 16 MiB cap is data-plane only).
- No channel-log rotation/compaction beyond per-topic retention.
- No cap on number of topics, number of subscribers per topic, or KV size.
- No rate limiter or backpressure governor.

There is a known production pain point with the `agent-presence` topic
specifically — T-1991 documents discovery slowdown from monotonic growth on
`.122` and `.107` and led to a client-side filesystem cache mitigation (T-1992).
That is the only scale incident recorded against the foundation primitives.

For 10–30 mostly-idle agents with occasional bursts: no visible code limit
bites. The agent-presence topic growth from heartbeating is the one observed
real-world pressure; otherwise the runtime is built for far higher concurrency
than this.

## Gaps

In priority order, capabilities that are ABSENT or PARTIAL relative to the
hub-and-spoke / disjoint-write-set / governance-serialization model:

1. **TermLink would need exclusive-delivery / claim semantics on a queue.**
   Channel topics are broadcast pub-sub with per-subscriber cursors; nothing in
   the bus or KV layer offers "exactly one consumer takes this message" or
   "lock this key for N seconds". `channel.receipts` are advisory read-acks,
   not claims.

2. **TermLink would need an idle/busy registry the hub owns.** Heartbeats land
   in a topic; LIVE/STALE/OFFLINE classification is computed by client-side
   `jq`. There is no hub-tracked agent state, no "next idle worker for role X"
   lookup, no in-flight counter to scope dispatch decisions on.

3. **TermLink would need a pull verb for work distribution.** Every existing
   path is push: the sender chooses the recipient (by id, by topic name, by
   dispatched worker list). There is no "give me the next unit" RPC.

4. **TermLink would need any form of filesystem-write observation.** Nothing in
   the runtime watches agent file I/O. Path-claim coordination ("I am now
   touching path X") is entirely on the honour system: an agent must post a
   declaration to a topic, and other agents must consult that topic before
   acting. The hub cannot enforce or even verify the claim.

5. **TermLink would need client-side reconnect / outbound queueing for spokes.**
   When the hub is unreachable, the client returns an error immediately and
   discards the call. A spoke whose host briefly loses the hub has no local
   buffer for posts-in-flight; the agent's own code is responsible for
   retrying. Channel durability protects readers, not writers.

6. **TermLink would need symmetric authentication across same-host and
   cross-host transports if both are to be treated as the same path.** Today,
   same-host UDS bypasses HMAC and TLS (UID trust); cross-host TCP requires
   both. They produce identical JSON-RPC semantics but with non-equivalent
   trust models, so "everything over IP/loopback uniformly" is achievable only
   by configuring same-host clients to dial loopback TCP explicitly — which
   the default code paths do not do.

7. **TermLink would need hub-persistent presence and circuit-breaker state.**
   The `PresenceTracker` and the per-session circuit breaker live in
   `RwLock<HashMap>` and reset on hub restart. Channel logs and the inbox spool
   DO survive, so message durability is intact, but liveness inference resets
   to "everyone unknown" for the heartbeat interval after every restart.

8. **TermLink would need a typed agent-launch surface that knows about
   source-tree handoff.** `termlink spawn` accepts arbitrary commands and env;
   `termlink dispatch --isolate` adds git worktree creation and optional
   auto-commit/auto-merge. That is the entire git surface — there is no
   `agent.checkout(ref)`, `agent.commit(scope)`, or `agent.publish(branch)`
   RPC. Coordinating a code plane vs. a governance plane today means a
   shell-script convention sitting above TermLink, not a TermLink concept.

9. **TermLink would need a way to fan a single value to all current and future
   subscribers atomically.** The current "everyone sees it" mechanism is a
   topic post + cursor read; a late joiner reads from the persisted log. There
   is no broadcast-with-implicit-replay or "current value" key that the hub
   would surface to a subscriber on registration. KV is per-session and not
   subscribable.

10. **TermLink would need a published throughput / connection-count budget.**
    No connection cap, no rate limiter, and the one observed scale incident
    (T-1991 agent-presence bloat) was found in production rather than
    predicted from the code. 10–30 agents fit comfortably within the visible
    limits, but the headroom above that is uncharted.

## Decision posture

Recommendation: **DEFER** (per task scope — the human asked for "what is", not
"what to build"). The findings above are the deliverable; the architectural
decision (extend TermLink, change the design, or accept the gaps) belongs to
the human.
