# /check-outbox — OUTBOUND complement of /check-arc (T-1891)

Surfaces `dm:<self>:<peer>` topics where YOU posted but the PEER hasn't
acked — i.e. DMs accumulating in someone's silent inbox. Read-only.

`/check-arc` answers *"who's waiting for me to read them?"*.
`/check-outbox` answers *"whose mailbox am I filling without them reading?"*.

Canonical use case (T-1457 evidence): on 2026-05-31 the `.141` inbox held
5 DMs from this host with no receipts and no listener attached. Before this
skill, the operator had to manually inspect each `dm:*` topic per hub to
detect this. Now: one command.

**Invocation:**

| Form | Action |
|------|--------|
| `/check-outbox` | Local hub only — fast, no fleet probe |
| `/check-outbox --fleet` | Walk every hub in `~/.termlink/hubs.toml` (dedup by TLS fingerprint, T-1889 sibling) |
| `/check-outbox --with-presence` | T-1895: enrich each row with peer's `[LIVE]`/`[STALE]`/`[OFFLINE]` listener status. Adds one cross-reference call to `agent-listeners-fleet.sh`. UNKNOWN (suppressed marker) when peer can't be located on any reachable hub. |
| `/check-outbox --json` | Machine-readable; pair with `jq` |
| `/check-outbox --include-self` | Include `dm:<self>:<self>` (default skips) |

**Canonical use of `--with-presence`** (T-1457): T-1891 surfaced 5 unread DMs to peer `6604a2af` on `laptop-141`. Without `--with-presence`, the operator must run `/peers --all` separately to learn whether the peer has a LIVE listener. With it: the row reads `[OFFLINE] laptop-141 dm:6604a2af:d1993c2c peer=6604a2af… unread=5` — operator knows immediately that nudging via `/agent-handoff` is futile (no listener to ring), and `/broadcast-chat` is the only path. The suggestions tail auto-includes that hint when any row is OFFLINE/UNKNOWN.

The wrapper is `scripts/check-outbox.sh`. Read-only by contract — no posts,
no acks, no `KnownHubStore` writes.

## Step 1: Pre-flight

```
bash scripts/check-outbox.sh --help >/dev/null
```

If exit non-zero: **stop**. Print:

```
check-outbox: wrapper not found at scripts/check-outbox.sh.
Ensure you're in the TermLink project root (cd /opt/termlink).
```

## Step 2: Parse args

| Operator typed | Command emitted |
|------|------|
| `/check-outbox` | `bash scripts/check-outbox.sh` |
| `/check-outbox --fleet` | `bash scripts/check-outbox.sh --fleet` |
| `/check-outbox --json` | `bash scripts/check-outbox.sh --json` |
| `/check-outbox --fleet --json` | `bash scripts/check-outbox.sh --fleet --json` |

Pass other flags through verbatim. The wrapper validates and errors on
malformed input.

## Step 3: Run the wrapper

Execute via Bash. Surface stdout verbatim.

Typical successful human-mode output:

```
check-outbox: 3 topic(s) with unread peer (self=d1993c2c…)
  laptop-141     dm:6604a2af482f0cf7:d1993c2c3ec44c94  peer=6604a2af…  unread=5  (count=5, peer_acked=-1)
  workstation-…  dm:9219671e28054458:d1993c2c3ec44c94  peer=9219671e…  unread=21 (count=21, peer_acked=-1)
  workstation-…  dm:tl-kr4ulsog:d1993c2c3ec44c94       peer=tl-kr4u…   unread=2  (count=2, peer_acked=-1)

DMs you sent that peer hasn't acked. If peer is unreachable, consider:
  • /agent-handoff <peer> T-XXX "<follow-up>"     # nudge
  • /peers --all                                    # check if peer is LIVE
```

Empty state:

```
check-outbox: no outbound-unread DMs on local hub — all peers caught up (self=d1993c2c…)
```

## Step 4: Suggest next actions

After the digest:

- For each topic with `unread > 0` and a peer that's also LIVE (cross-ref
  `/peers`): the peer is online but ignoring. Either the doorbell wasn't
  rung, or they're busy. Nudge with `/agent-handoff <peer> <task> "..."`.
- For topics where the peer has no LIVE presence: the peer's host has no
  listener. Either deploy `/be-reachable` there or treat as broadcast-only.

Self-fp resolution chain (PL-195):
1. `channel info agent-presence --json | jq .senders[0].sender_id`
2. `channel info agent-chat-arc --json | jq .senders[]|select(.posts>0)`
3. Exit 2 with hint to run `/be-reachable` or `/broadcast-chat`.

The wrapper resolves ONCE from the local hub (shared-host case — all hubs
see the same key per PL-195/T-1693) and reuses across fleet hubs. This
avoids the per-hub fallback timeout cascade.

## Computation

For each `dm:<a>:<b>` topic where `a` or `b` matches `self_fp`:
- `peer_fp` = the non-self side
- `count` = `channel info <topic>`.count
- `peer_acked` = max(`r.up_to`) for `r in receipts` where `r.sender_id == peer_fp`, default `-1`
- `outbound_unread` = `count - 1 - peer_acked`
- Surface only when `outbound_unread > 0`

This is an upper bound (some envelopes in the count may be the peer's own
posts that I've read, not unread-by-them-of-mine). For `dm:` topics with
only two senders the approximation is close. Pair with `/recent-dm <peer>`
to inspect the actual envelope-by-envelope ground truth.

## Rules

- **Read-only by contract.** Never posts, never acks. Use `/agent-handoff`
  or `/reply` for the action.
- **Local-default.** Fleet mode is opt-in (`--fleet`) because hub probing
  + per-topic info-fetch can take seconds per hub. Operators usually want
  local-fast.
- **No `AskUserQuestion`** — just run and report.
- **PL-176 caveat:** DM topics may not federate. The same `dm:<a>:<b>`
  topic on hub X is distinct from `dm:<a>:<b>` on hub Y. Counts reported
  per-hub; the same logical conversation may show up multiple times in
  fleet mode if it exists on multiple hubs.

## Related

- T-1891 (this skill + wrapper)
- T-1810 (`/check-arc` — the INBOUND complement)
- T-1862 (`/recent-dm` — per-peer envelope history)
- T-1457 (the canonical .141 backpressure case)
- T-1431 (`/agent-handoff` — the SEND verb to nudge with)
- T-1841 (`/be-reachable` — what unreachable peers need before they can read)
- PL-195 / T-1693 (shared-host identity semantics)
