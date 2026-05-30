# /reply — one-keystroke ad-hoc reply to a peer (T-1880)

Wraps `scripts/agent-reply.sh`. The SEND/RECEIVE-symmetric companion to
`/agent-handoff`: `/agent-handoff` opens a thread; `/reply` answers one
that already exists.

Operator has an existing DM thread with a peer and wants to post one
composed answer. Without this skill the flow is:

1. `/check-arc` → see unread DM
2. `/recent-dm <peer-short> --since 720` → read content + extract `cid`
3. `bash scripts/agent-respond.sh --topic dm:... --conversation-id <cid> --reply '<text>'`

Three steps + manual cid extraction. With `/reply`:

```
/reply <peer-short> "<text>"
```

One step. The script auto-resolves self-fp (PL-195 canonical chain), the
DM topic (substring match against `dm:*` topic names that contain BOTH
self-fp AND the peer-substring), and the `conversation_id` (the highest-
offset cid on the topic). It then delegates to `agent-respond.sh` for the
actual receipt + reply (so the protocol stays in one place).

**Invocation:**

| Form | Action |
|------|--------|
| `/reply <peer-short> "<text>"` | Reply to peer's latest cid with `<text>` |
| `/reply <peer-short> "<text>" --ensure-cid` | Mint fresh cid if topic has none |
| `/reply <peer-short> "<text>" --dry-run` | Print resolved topic + cid + delegated command, do NOT post |
| `/reply <peer-short> "<text>" --hub addr` | Restrict topic discovery to one hub |
| `/reply <peer-short> "<text>" --self ID` | Override self-fp resolution |
| `/reply <peer-short> "<text>" --json` | Emit JSON envelope after success |

The peer-substring is matched against `dm:*` topic names. It typically
is a 16-hex envelope sender_id (or a recognizable infix). It must be
specific enough to yield exactly one topic — multi-match refuses with
the candidate list.

## Step 1: Pre-flight

Run:

```
bash scripts/agent-reply.sh --help >/dev/null
```

If exit non-zero: **stop**. Print:

```
reply: wrapper not found at scripts/agent-reply.sh.
Ensure you're in the TermLink project root (cd /opt/termlink).
```

## Step 2: Parse arguments

The operator's tail is `<peer-substring> <text>` plus optional flags. The
shell-skill responsibility is just to forward them — the wrapper validates
the rest.

If the tail is empty or has only one positional: **stop**. Print:

```
reply: need <peer-substring> <text>.
Usage: /reply <peer-short> "<text>" [--ensure-cid] [--dry-run] [--hub addr]
```

Do NOT default to a placeholder text. An empty reply is noise — refuse
instead.

## Step 3: Run the wrapper

Execute via Bash:

```
bash scripts/agent-reply.sh <peer-substring> "<text>" [<flags>]
```

The wrapper prints two stderr lines for the discovery + dispatch, plus
one stdout line per envelope landed (the `agent-respond.sh` output passes
through):

```
agent-reply: posting to 'dm:...' (cid=<cid>, self=<fp>)
agent-respond: receipt posted to 'dm:...' (cid=<cid>, up_to=<N>)
agent-respond: reply posted to 'dm:...' (cid=<cid>, offset=<N>)
```

Surface the wrapper's output verbatim. With `--json` the success envelope
`{ok, topic, conversation_id, self, peer_substring}` is emitted to
stdout last — pass through too.

## Step 4: Surface refusals distinctly

The wrapper refuses (exit 2) in these cases:

| Refusal | Wrapper says |
|---------|--------------|
| Peer matches zero topics | "no dm:* topic matches peer=... AND self=..." with hint to use `/agent-handoff` |
| Peer matches multiple topics | "matches N dm:* topics — refusing to guess" + candidate list |
| Topic has no cid + no `--ensure-cid` | "no envelope on '...' carries metadata.conversation_id" with `--ensure-cid` hint |
| Self-fp unresolvable | "could not resolve own envelope sender_id" with `/be-reachable` hint |

In each case the wrapper's hint IS the operator's next step — surface
verbatim and stop. Do NOT improvise an alternative.

## Rules

- **This skill writes state.** Every successful invocation produces TWO
  real envelopes on the resolved DM topic (a receipt and a reply turn).
  There is no undo. Pair with `/check-arc` and `/recent-dm` for
  read-side context before firing.
- **Never auto-broadcast.** This skill is a TARGETED reply on a
  resolved DM topic. It will never post to `agent-chat-arc`. For fleet
  broadcasts use `/broadcast-chat`.
- **Never silently disambiguate.** Multi-match refusal is load-bearing
  — guessing the wrong thread is worse than refusing. The operator
  always disambiguates with a longer substring.
- **Use `--ensure-cid` deliberately.** Minting a fresh cid effectively
  starts a new thread on an existing DM topic. If you're answering an
  existing structured thread, the default (require existing cid) is
  the right behavior. `--ensure-cid` is for converting a chat-style
  thread to a structured one, or for first-reply on a topic that was
  created by direct `channel dm`.
- **Do not use `AskUserQuestion`.** Just run and report.

## Pair with

- `/check-arc` — RECEIVE-side discovery; tells you which topics have
  unread + who sent them. The natural prerequisite to `/reply`.
- `/recent-dm <peer> --since 720` — per-peer DM history; read the thread
  before composing your reply.
- `/agent-handoff <target> <task-id> "<msg>"` — opens a NEW thread.
  `/reply` continues an EXISTING one.
- `scripts/agent-respond.sh` — the underlying transport. `/reply`
  saves you from having to know topic + cid; `agent-respond.sh` directly
  if you already know both.

## Common patterns

**Quick ack on a single thread:**

```
/check-arc                          # see unread topic for peer 9219671e
/recent-dm 9219671e --since 720     # read what they said
/reply 9219671e "got it, working on it"
```

**Targeted reply with explicit hub (cross-host):**

```
/reply 9219671e "see commit abc123" --hub 192.168.10.122:9100
```

**Mint fresh cid on a chat-style DM topic:**

```
/reply 9219671e "starting structured thread re T-1880" --ensure-cid
```

## Related

- T-1880 (the underlying script + this skill)
- T-1431, T-1429 (the SEND-side counterparts — `/agent-handoff` +
  `termlink agent contact`)
- T-1805 (`scripts/agent-respond.sh` — the transport this delegates to)
- T-1804 (`scripts/agent-send.sh` — the SEND-side transport)
- T-1862, T-1878 (`/recent-dm` — the canonical read companion)
- T-1810 (`/check-arc` — the unread inbox; respond-mode in there is the
  batch-iterate-unread complement to `/reply`'s targeted-one-thread case)
- PL-195 (envelope sender_id vs whoami's identity_fingerprint — the
  identifier conflation that motivated the canonical resolution chain
  used here)
