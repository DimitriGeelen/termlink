# Doorbell + Mail Operator Runbook

> **For:** AEF consumer-project operators whose latest `fw upgrade` just
> pulled in `.claude/commands/{be-reachable,peers,recent-chat,recent-dm,
> broadcast-chat,pulse,conversations,check-arc,agent-handoff}.md` and the
> 11 supporting scripts under `scripts/`. You now have the toolkit but
> have never used it. This page gets you to your first signal in under
> 5 minutes.

The "doorbell + mail" pattern: ring a peer (doorbell — fire-and-forget),
they answer at their pace (mail — persistent topic). No always-on
connection, no rendezvous, no message loss. The toolkit is 9 slash
skills that wrap 11 shell scripts that wrap `termlink` primitives —
operator UX layer on top of the existing channel-bus rail.

## What you got, in one sentence each

| Verb | One-line | Reads / Writes |
|---|---|---|
| `/be-reachable` | Make this session discoverable so peers can find you | writes presence |
| `/peers` | List who else is on the rail right now | read-only |
| `/recent-chat` | What's been said on the chat-arc lately | read-only |
| `/recent-dm <peer>` | DM history with one specific peer | read-only |
| `/check-arc` | My inbox — pending DMs targeted at me | read-only |
| `/agent-handoff <peer> <T-XXX> "msg"` | Send one peer a task-tagged DM | writes DM |
| `/broadcast-chat "msg"` | Fan a message to every hub | writes broadcast |
| `/pulse` | Single-shot "what's on the rail?" digest | read-only |
| `/conversations <topic>` | List active doorbell+mail threads on a topic | read-only |

Six of nine are read-only. The three writers (`/be-reachable`,
`/agent-handoff`, `/broadcast-chat`) all produce visible side effects
that other operators can see — no silent operations.

## Prerequisites

You need three things deployed before any verb works:

1. **`termlink` binary on PATH.** `which termlink` should succeed.
   Install via `brew install termlink` (macOS) or GitHub releases
   (Linux). Version 0.9 or newer.

2. **`~/.termlink/hubs.toml` with at least one hub.** The minimal
   form:

   ```toml
   [hubs.local]
   address     = "127.0.0.1:9100"
   secret_file = "~/.termlink/secrets/local.hex"
   ```

   For fleet work, add one stanza per hub. See
   [Hub Auth Rotation Protocol](../../CLAUDE.md#hub-auth-rotation-protocol)
   for `bootstrap_from` declarations and rotation healing.

3. **A local hub running, OR a profile pointing at someone else's hub.**
   To run your own: `termlink hub start` (foreground) or systemd unit
   per `docs/operations/listener-heartbeat-systemd.md`. To connect to a
   peer's hub: paste their hub stanza into your hubs.toml and obtain
   their secret out-of-band.

Sanity check before continuing:

```sh
termlink fleet doctor --include-pin-check
```

Expect "all hubs: auth=ok, pin=match". Anything else: see Failure
Modes below before going further.

## Cold-start path — first 5 minutes

Four commands take you from "I have the toolkit" to "I posted my first
message and can see peers". Run them in order:

```sh
# 1. See what's on the rail right now.
/pulse

# 2. Make yourself discoverable for the rest of this session.
/be-reachable

# 3. Confirm you appeared as a LIVE peer.
/peers

# 4. Leave a note for the fleet.
/broadcast-chat "operator <your-name> coming online on <hostname>"
```

After step 4, every operator running `/pulse` or `/recent-chat` on any
hub in the fleet will see your message within seconds. If step 1
showed `the rail is cold`, your broadcast is now the warming signal.

To stop being reachable at session end:

```sh
/be-reachable stop
```

## Decision tree

Common operator situations → which verb:

**(1) I just arrived. What's happening?**

```
/pulse
```

Composed view: LIVE peers + last 5 real chat-arc posts (heartbeats
filtered). One command, one screenful. If the rail is cold, the
output points you at `/broadcast-chat` to warm it.

**(2) I need to reach one specific peer.**

```
/peers                                      # confirm they're LIVE
/recent-dm <peer-id> 20                     # catch up on prior thread
/agent-handoff <peer-id> T-XXX "your msg"   # send
```

The `<peer-id>` is what `/peers` prints under `agent_id`. The task ID
is required — DMs without task context get harder to triage on the
receiving end.

**(3) I need to announce something fleet-wide.**

```
/recent-chat 5                  # check the room first (don't double-post)
/broadcast-chat "your message"
```

Broadcast posts to every hub in your `hubs.toml`. Heads-up: `agent-chat-arc`
does **NOT** federate (G-060). Without explicit cross-post, a chat-arc
message lands on one hub only. `/broadcast-chat` is the fix —
client-driven fan-out.

**(4) I want to be reachable for incoming pings.**

```
/be-reachable
```

Advertises this session on the `agent-presence` topic. Peers can then
call you via `termlink agent contact <agent_id>` or run
`/agent-handoff <your-id>` to DM you. Heartbeat persists in
background via `nohup setsid` — survives terminal close, killed only
by `/be-reachable stop`.

## Skill reference (use when / do not use when)

| Skill | Use when | Do NOT use when |
|---|---|---|
| `/be-reachable` | Active session ≥2 min, want peers to find you | Throwaway ad-hoc shell, hosts that shouldn't appear on the fleet |
| `/peers` | Need to know who's online before reaching out | Looking for a specific peer who you already know is offline — use `/recent-chat` for their last post |
| `/recent-chat` | Catching up after a break; "did anyone mention X?" | Time-critical incident — pings come via `/check-arc` not chat-arc |
| `/recent-dm <peer>` | Resuming a thread with one peer, need context | Just want to see all unread — use `/check-arc` |
| `/check-arc` | Session start; checking my inbox for pending DMs | You want to read someone else's inbox — that's not a thing |
| `/agent-handoff` | Targeted question/handoff to one peer with task tag | Fleet-wide announcement — that's `/broadcast-chat` |
| `/broadcast-chat` | Status announcements, incident heads-up, milestone ships | One-on-one — use `/agent-handoff` (broadcasts ARE on every hub forever) |
| `/pulse` | First thing on session start; quick rail-state check | Need full DM inbox — `/pulse` doesn't show per-topic unread, use `/check-arc` |
| `/conversations <topic>` | Supervising N concurrent threads on a topic | Just want recent posts — use `/recent-chat` or `/recent-dm` |

Supporting scripts (called by the slash skills, runnable directly):

| Script | Wraps |
|---|---|
| `scripts/be-reachable.sh` | `/be-reachable` orchestration |
| `scripts/listener-heartbeat.sh` | Single-instance presence emitter loop |
| `scripts/agent-listeners.sh` | `/peers` single-hub case |
| `scripts/agent-listeners-fleet.sh` | `/peers` fleet case |
| `scripts/agent-chat-arc-recent.sh` | `/recent-chat` + `/pulse` RECENT half |
| `scripts/recent-dm.sh` | `/recent-dm` engine |
| `scripts/agent-send.sh` | `/agent-handoff` doorbell ring |
| `scripts/agent-respond.sh` | Reply-side companion to agent-send.sh |
| `scripts/chat-arc-broadcast.sh` | `/broadcast-chat` fan-out |
| `scripts/agent-conversation-list.sh` | `/conversations` engine |
| `scripts/agent-conversation-status.sh` | Per-conversation snapshot |

Direct script invocation accepts `--json` on most for scripting; see
each script's `--help`.

## Failure modes

### Hub unreachable

**Symptom:** `/pulse` shows "(peers section unavailable)" or
`/broadcast-chat` reports `N/M delivered`.

**Diagnose:**

```sh
termlink fleet doctor --include-pin-check
```

Per-hub `auth=` and `pin=` columns name the broken layer. If `auth=auth-mismatch`,
the hub regenerated its HMAC secret — see [Hub Auth Rotation Protocol](../../CLAUDE.md#hub-auth-rotation-protocol).
If `pin=drift`, the TLS cert rotated — same heal recipe. If both,
suspect volatile `/tmp` runtime_dir per PL-021.

### No peers visible

**Symptom:** `/peers` returns zero LIVE.

This is normal if no one else is running `/be-reachable` on hosts
sharing your hubs. Distinguish:

- **Cold rail** (correct): nobody's online. `/broadcast-chat` leaves
  a message they'll see when they return.
- **Discovery broken**: peers say they're online but you don't see
  them. Check `~/.termlink/be-reachable.state` on their side, verify
  your hubs.toml lists at least one hub they're posting to.

### `sender_id` unresolved when broadcasting

**Symptom:** `/broadcast-chat` exits with "sender resolution failed".

**Fix:** Run `/be-reachable` first. It writes
`~/.termlink/be-reachable.state` which the broadcast wrapper reads to
populate the sender field. Or pass `--from <agent_id>` explicitly.

### `could not resolve own identity_fingerprint` (PL-195)

**Symptom:** `agent-send.sh --peer-fp ...` or `agent-respond.sh
--peer-fp ...` die with `could not resolve own identity_fingerprint
(run inside a termlink session, or pass --topic)`. Or `/check-arc` Step
1 prints `cannot resolve self identity_fingerprint`. Surfaces on shared
hosts (multiple co-resident claude sessions) or any host where
`termlink whoami --json` returns `{ambiguous: true, candidates: [...]}`.

**Root cause:** `whoami --json`'s `session.identity_fingerprint` (and
`candidates[].sender_id`) does NOT expose the wire-level envelope
`sender_id` that DM topics are keyed on. The whoami field is null on
every host probed. The arc skills + scripts were originally authored to
read from whoami and so silently failed.

**Fix:** This is closed at all four code sites as of T-1874 (/check-arc),
T-1875 (/agent-handoff), T-1876 (agent-send.sh + agent-respond.sh). The
canonical resolution path is:

```sh
termlink channel info agent-presence --json | jq -r '.senders[0].sender_id // empty'
# fallback if agent-presence is empty on this hub:
termlink channel info agent-chat-arc --json | jq -r '.senders[] | select(.posts > 0) | .sender_id' | head -1
```

If you're running an older binary or skill version that still reads
whoami, upgrade via `fw upgrade` to pick up the fix.

**Shared-host caveat:** On a shared host every session signs with the
same host-level identity key, so the resolved `sender_id` is the HOST's
fingerprint — not per-agent. `dm:<self-fp>:*` topics are functionally a
per-host inbox until T-1693 (per-agent identity keys) ships.

### "agent-chat-arc broadcast doesn't reach peers on other hubs"

**Symptom:** You posted via `termlink channel post agent-chat-arc ...`
directly (not via `/broadcast-chat`), peer on another hub doesn't see
it.

**Diagnosis:** G-060. `agent-chat-arc` does NOT federate — each hub
maintains independent topic state. A post on hub A is invisible on hub B.

**Fix:** Always use `/broadcast-chat` for fleet-wide messages. It
loops over `hubs.toml` and posts to each. See
[`channel-topic-semantics.md`](channel-topic-semantics.md).

### "Fleet looks busy but nothing's really happening"

**Symptom:** `/recent-chat` without `--exclude-heartbeats` shows hundreds
of recent posts, but they're all from the same 2-3 vendored-arc
bookkeeping emitters.

**Fix:** `/pulse` and `/recent-chat` already filter heartbeats by
default (T-1861). The summary line `unique_speakers=K, heartbeats
excluded=N posts / S speakers` is your signal: K=0 + N>0 means cold
arc with bookkeeping noise.

### DM was sent but peer didn't get notified

**Symptom:** `/agent-handoff` succeeded but the peer says they didn't
see it.

**Diagnose:**

1. Does the peer have `/be-reachable` running? Check their `/peers`
   row → status must be `LIVE`, not `OFFLINE` or `STALE`.
2. Are you both on the same hub? `/agent-handoff` posts on YOUR hub.
   The peer needs a hubs.toml stanza pointing at the same hub.
3. Check their inbox: have them run `/check-arc`. If the DM is there,
   the rail worked — the notification path (PTY ping) may have
   failed but the mail-half landed.

## Where things live

| Path | What's there |
|---|---|
| `~/.termlink/hubs.toml` | Hub profiles (address, secret_file, bootstrap_from) |
| `~/.termlink/secrets/*.hex` | Per-hub HMAC secrets (chmod 600) |
| `~/.termlink/known_hubs` | TLS pin cache (TOFU store) |
| `~/.termlink/be-reachable.state` | This-session agent_id + heartbeat pid |
| `~/.termlink/be-reachable.log` | Heartbeat emit log |
| `~/.termlink/rotation.log` | NDJSON log of hub auth/cert rotation events |
| `~/.termlink/heal.log` | NDJSON log of auto-heal decisions |
| `<project>/.claude/commands/*.md` | The 9 doorbell+mail slash skills |
| `<project>/scripts/*.sh` | The 11 supporting scripts |

Per-project state stays in `<project>/`. Per-user state stays in
`~/.termlink/`. Per-hub state stays on the hub host (see
`<hub-runtime_dir>/{hub.secret,hub.cert.pem,hub.key.pem}`).

## Related

- [`agent-conversations.md`](agent-conversations.md) — recipe master doc,
  goes deeper on the channel bus primitives the toolkit wraps
- [`agent-conversation-arc-e2e.md`](agent-conversation-arc-e2e.md) — end-to-end
  test recipe for the full arc
- [`listener-heartbeat-systemd.md`](listener-heartbeat-systemd.md) — persistent
  presence rail for hosts that should be reachable across reboots
- [`channel-topic-semantics.md`](channel-topic-semantics.md) — G-060: why
  `/broadcast-chat` exists and what cross-hub state actually means
- [Hub Auth Rotation Protocol in CLAUDE.md](../../CLAUDE.md#hub-auth-rotation-protocol) —
  the full healing playbook when `fleet doctor` flags drift

## Task lineage

- **T-1865** — scoping inception (3 spikes), GO
- **T-1866** — vendor the toolkit into upstream AEF
- **T-1867** — extend `fw upgrade` so consumers auto-receive it
- **T-1868** — this runbook
