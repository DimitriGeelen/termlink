# T-2229 — Triage: ring20 cross-hub federation + heartbeat-freeze RCA

**Source:** ring20-management filed a high-severity RCA to the `framework:pickup`
hub topic (offset 42). It sat ~27h unprocessed; a live termlink agent picked it
up 2026-06-21. Two faults claimed; very different verdicts.

## Fault 1 — "cross-hub federation broken" → WORKING-AS-DESIGNED

ring20 reported that registry + channel (agent-chat-arc, DM) state no longer
replicates across hubs. **This is not a regression.** TermLink has **no
inter-hub federation primitive and never did** — confirmed by:
- PL-176 ("TermLink has NO inter-hub channel-topic federation primitive")
- G-060 / CLAUDE.md §"Channel Topic Semantics — Per-Hub State"
- `docs/operations/channel-topic-semantics.md`

Cross-hub visibility is by explicit client-driven cross-post
(`termlink channel post --hub <addr>` / `remote call`). **ring20's "presence
beacon" IS the supported mechanism**, not a band-aid. No code change.

The residual issue is *discoverability*: consumers keep mis-filing "federation
broken." Whether to add an operator-facing `fleet federation-status` verb (or a
louder doc pointer) is an open operator decision — see IW-3 (deferred).

## Fault 2 — "heartbeat freezes on hub restart" → REAL BUG, FIXED (broader than diagnosed)

ring20 framed it as "heartbeat does not survive a hub restart." Investigation
(Explore agent, 2026-06-21) found the bug is **deeper**: `termlink register`
**never advanced its heartbeat at all**.

- `cmd_register` (crates/termlink-cli/src/commands/session.rs) set `heartbeat_at`
  once in `Registration::new`, then blocked in `server::run_accept_loop` with no
  heartbeat timer.
- `Registration::touch_heartbeat` (registration.rs:325) had **zero production
  callers** — only tests.
- TermLink sessions are file-based: the register process owns its own socket and
  holds no connection to the hub, so a hub restart provides no signal to react to.
  The "across hub restart" framing was a misdiagnosis of a permanent freeze.
- The existing `touch_heartbeat_updates_timestamp` test tolerated an unchanged
  timestamp, so a permanently-frozen heartbeat passed CI.

**Fix (T-2230, commit b182a803):** periodic self-heartbeat task in `cmd_register`
advancing `heartbeat_at` in both the in-memory registration (query.status RPC)
and the on-disk JSON (hub sweep), every `TERMLINK_HEARTBEAT_INTERVAL_SECS`
(default 30s), aborted on shutdown. Regression test
`heartbeat_strictly_advances_over_time` asserts strict advancement in-memory and
on-disk. Learning PL-221: a test that tolerates the buggy value is not coverage.

## Meta-finding — why it sat 27h (G-063)

`framework:pickup` has **no automatic consumer** on termlink. A high-severity
filing landed and nothing surfaced it. **Closed by T-2231**: a daily
`framework:pickup` freshness canary (empty-log = healthy; `/canaries` discovers
it; `--ack` after triage). The next unprocessed filing surfaces within a day.

## Disposition

| IW | Question | Disposition |
|----|----------|-------------|
| IW-1 | Federation regression or by-design? | answered — by design (no primitive) |
| IW-2 | Heartbeat-freeze a real bounded bug? | answered — confirmed + fixed (T-2230) |
| IW-3 | Add operator federation-status verb? | deferred — operator decision |

**Spawned:** T-2230 (fault-2 fix, shipped), T-2231 (G-063 canary, shipped).
**Reply to ring20:** agent-chat-arc @ .122, offset 2299.
**Ready for `fw inception decide T-2229`** (human authority).
