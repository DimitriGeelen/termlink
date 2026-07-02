# T-2312 — arc-004 follow-on inception: WS-over-Unix push for co-located agents

**Type:** inception (one question, go/no-go)
**Arc:** arc-004 `push-transport` (follow-on, past the GO(scoped) surface)
**Agent recommendation:** **GO** (client-only, low effort). Final decision: human.
**Date:** 2026-07-02

---

## The one question

`channel subscribe <topic> --push` gives sub-second DM push (~90 ms, T-2310), but
only over a **TCP** hub. Should it also work over the **Unix socket**, so co-located
agents (same host as the hub) get push instead of the 1 s poll floor?

## Headline finding — the hub already supports it

Reading the hub (`crates/termlink-hub/src/server.rs`):

- `handle_connection` (`:760`) is called by **both** the Unix accept path (`:618`)
  and the TCP-after-TLS path. It runs the T-2305 **first-byte sniff** (`:785`,
  `is_ws = first == 'G'`), so a Unix client sending an HTTP `GET` upgrade routes to
  `handle_ws_connection`.
- `handle_ws_connection<S>` (`:830`) is **generic over `AsyncRead + AsyncWrite`** —
  it does `accept_async(stream)` over whatever stream it's given, **with no TLS
  assumption**. Over a Unix connection that's a raw WS handshake over the UDS.
- Unix connections are accepted with `Some(PermissionScope::Execute)` (`:620`), and
  `maybe_handle_ws_subscribe` (`:975`) permits `hub.ws_subscribe` for any scope that
  satisfies `Observe` (`:987-1010`). Execute does — so a Unix WS client can
  subscribe **with no `hub.auth` at all**.

**Conclusion:** the hub needs **zero change**. WS-over-Unix is entirely a
**client-side** gap.

## The only blocker — client rejects Unix

`crates/termlink-session/src/client.rs:112` `connect_tls_stream` returns
`Unsupported` for Unix targets and always wraps rustls TLS:

```rust
TransportAddr::Unix { .. } => return Err(Unsupported("… WS-over-Unix is a follow-on")),
```

And the CLI `--push` branch degrades immediately on `Unsupported`
(`channel.rs` ~8493). So the whole change is: give the client a way to open a raw
WS over the Unix socket and route `--push` through it for Unix hubs.

## Recommended build path (client-only, one slice)

1. **`connect_ws_unix`** in `termlink-session` — `UnixStream::connect(path)` then
   `tokio_tungstenite::client_async("ws://localhost/", stream)` over the **raw**
   stream (no TLS, no rustls). Mirror of the TCP `connect_tls_stream` → `client_async`
   shape, minus the TLS layer.
2. **Route Unix in `ws_consumer` / `run_ws_push`** — for a Unix addr, use
   `connect_ws_unix` and **skip the token mint** (`mint_tcp_hub_token` is TCP-only;
   Unix needs no auth). Send `hub.ws_subscribe` directly.
3. **Drop the `Unsupported` rejection** for Unix in the CLI `--push` branch so it no
   longer degrades immediately for co-located hubs.
4. **Spike first (validates IW-2):** confirm `client_async` over the raw UDS
   completes the handshake the hub's `accept_async` expects (the hub is generic, so
   this is expected to just work — but prove it before wiring the CLI).

Suggested wire evidence: extend `scripts/demo-ws-push.sh` (or a sibling) with a
Unix-hub variant that subscribes `--push` over the Unix socket and shows the same
sub-second push — the direct analogue of the T-2310 TCP demo.

## Why GO

- **Bounded + cheap:** client-only, no hub/protocol change, no auth path (Unix is
  peer-cred-trusted). Smaller than T-2311 (active-reconnect).
- **Real value:** co-located agents on shared hosts (e.g. the multiple co-resident
  agents on .107) currently get **no** live push; this gives them the same ~90 ms
  path as remote agents.
- **Low risk:** preserves the no-miss invariant (degrade-to-poll from the durable
  cursor unchanged); Unix same-UID access is already full-trust so skipping auth
  weakens nothing.

## Why you might NO-GO

- If a build spike reveals the raw Unix WS handshake needs a hub-side tweak after
  all (would break the "client-only" premise — re-scope then).
- If co-located agents rarely need sub-second cross-agent DM because same-host
  agents can coordinate via shared memory/files directly — a legitimate product call.

## Relationship to T-2311

Independent follow-on. T-2311 (active reconnect-to-WS) hardens the **existing** TCP
push path; T-2312 (this) **extends** push to a new transport (Unix). They don't
depend on each other and can be decided/built in either order. If both GO, a natural
order is T-2311 first (hardens what's shipped) then T-2312 (widens reach) — but
T-2312 is the cheaper of the two.

## Dialogue Log

- **2026-07-02** — Operator selected "option 2" (follow-on scope decision). Agent
  scoped active-reconnect (T-2311) first, then this (T-2312) so both follow-ons are
  decidable together. Investigation upgraded the prior belief ("two-sided change,
  needs hub work") to "hub already supports it — client-only". Filed with advisory
  **GO**. Awaiting human go/no-go via `fw task review T-2312` / `fw inception decide`
  (sovereignty-gated; agent cannot self-decide).
