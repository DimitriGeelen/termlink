# Deterministic notify sidecar (V3a)

> Arc-003 `reliable-comms`, slice V3a (T-2294). RC3a of the T-2291 inception RCA:
> *the recipient has no deterministic way to learn a message arrived.* This is the
> **notify** half of delivery; the **confirm** half (recipient auto-ack +
> unconfirmed-send canary) is V3b (T-2295). Design trail:
> `docs/reports/T-2291-cross-agent-comms-inception.md`,
> `docs/architecture/parallel-execution-substrate.md` §5.

## The problem it solves

A turn-based agent (Claude Code and peers) only notices the world at its own
**yield points** — between turns, not mid-turn. The previous wake mechanism was a
preemptive **PTY doorbell** (T-1800): the sender typed `/check-arc` straight into
the recipient's terminal. If those keystrokes land *mid-turn* they are dropped and
the recipient never wakes — the T-2285 miss-gap. Worse, the failure is **silent**:
a sender believes it "rang the bell," the recipient never heard it, and nothing
surfaces the loss.

The §5 fix inverts the direction. Instead of the sender *pushing* into the
recipient's PTY, a **no-LLM sidecar** on the recipient host *pulls* the agent's
mail off the hub and drops it into a **local flag file** alongside a **fresh
heartbeat timestamp**. The agent reads that flag cooperatively at its yield points.

**Determinism comes from the timestamp, not the transport.** The *absence* of a
fresh heartbeat delta is itself a signal: if the sidecar stopped beating, the agent
can no longer trust "no flag = no mail," so it **halts** rather than proceeding
blind. A broken listener is self-detected (G-019: the framework is no longer blind
to its own deaf ear), never silently missed.

## The two pieces

| Script | Role | Runs |
|---|---|---|
| `scripts/notify-sidecar.sh` | **The ears.** Polls this agent's `dm:<self>:*` mail on the hub; writes `<agent_id>.flag` (pending count) + `<agent_id>.heartbeat` (epoch-ms proof-of-life) every cycle. No LLM. | Long-lived, like `listener-heartbeat.sh`. |
| `scripts/notify-check.sh` | **The self-check.** Reads the local flag + heartbeat and returns a verdict. Pure local file read — no hub, no LLM, no keystroke. | At each agent yield point. |

### Why a file and not `termlink kv`

`termlink kv` is **session-scoped, in-memory, and hub-mediated** (a per-session
`HashMap`, lost on session exit, requires the hub to read/write). The self-check
must stay trustworthy **precisely when the hub is unreachable** — that is exactly
when an agent most needs to learn its listener went deaf. So the flag lives on the
local filesystem under `~/.termlink/notify/`, mirroring the offline-queue's path
discipline. Writes are atomic (write-temp-then-rename) so a yield-point read never
sees a half-written flag.

## Verdicts (notify-check.sh exit codes)

| Exit | Verdict | Meaning | Agent action |
|---|---|---|---|
| `10` | **MAIL** | Fresh flag says N>0 pending. | Wake — run `/check-arc` and read. |
| `3` | **DEAF** | Heartbeat missing or older than `--deaf-after`. | **HALT.** Re-establish the listener (restart `notify-sidecar.sh`) before acting — do NOT assume "no mail." |
| `0` | **CLEAR** | Listener alive AND no mail. | Proceed. |
| `2` | usage error | Bad flags. | Fix invocation. |

## Recipient recipe (the agent being notified)

```bash
# 1. Start the ears once (long-lived; 15s probe cadence).
nohup setsid scripts/notify-sidecar.sh --agent-id claude-A --interval 15 \
    >/var/log/notify-sidecar-claude-A.log 2>&1 &

# 2. At each yield point, self-check before proceeding.
scripts/notify-check.sh --agent-id claude-A --deaf-after 45   # 3x the interval
case $? in
  10) echo "new mail" ;  /check-arc ;;        # MAIL  → read it
   3) echo "listener DEAF — restarting ears, NOT proceeding blind"
      # restart notify-sidecar.sh, then re-check
      ;;
   0) : ;;                                     # CLEAR → proceed
esac
```

Set `--deaf-after` to **3× the sidecar `--interval`** (default 45s for a 15s probe):
two missed beats are noise, three means the listener is genuinely down.

## Sender recipe (closing the loop: missing-ack → retry)

V3a's notify is only half a delivery guarantee unless the sender can learn the wake
failed. That is the already-shipped T-2286 path — reuse it directly:

```bash
termlink channel post "dm:<you>:<peer>" --payload "<turn>" \
    --await-ack --retry --max-attempts 3 --ack-timeout-secs 30
```

`--await-ack` polls the recipient's `channel.receipts` frontier; `--retry` re-posts
(reusing the same `client_msg_id`, so T-2049 dedupe makes it exactly-once) up to
`--max-attempts` times, then **exits non-zero** if no ack ever arrives. A missing
ack therefore triggers a sender retry and finally a loud failure — never a silent
"sent." The recipient-side auto-ack that satisfies this is wired in V3b (T-2295);
the sidecar is its natural home.

## Test hook (hub-independent)

The sidecar's mail probe honours `TERMLINK_NOTIFY_TEST_UNREAD=<N>` (and
`TERMLINK_NOTIFY_TEST_LATEST_TOPIC`), mirroring the `TERMLINK_GROWTH_TEST_JSON`
convention. This lets `scripts/test-notify-sidecar.sh` prove every verdict path —
MAIL / CLEAR / DEAF-stale / DEAF-missing — with no live hub, which is the whole
point: the self-check has to work when comms are down.

```bash
bash scripts/test-notify-sidecar.sh    # 14 tests, hub-independent
```

## Known property: cold-DM off-by-one (inherited from `channel unread`)

The sidecar reports exactly what `termlink channel unread --sender <self>` reports —
the same primitive `/check-arc` uses. That primitive, when the reader has **no
receipt frontier yet** (a brand-new conversation), defaults the frontier to the
first offset and counts envelopes *past* it, so the very first message in a
zero-receipt DM is not counted until a follow-up arrives. Once the recipient has
acked even once (establishing a frontier), every later message counts correctly.
This is a property of the shared RECEIVE path, not the sidecar; the sidecar
inherits it deliberately so its counts always agree with `/check-arc`. If
first-message cold-start wake ever needs closing, fix it once in `channel unread`
so both surfaces benefit.

## `--auto-confirm` — recipient-side journaled receipt (V6 slice S3, T-2300)

By default the sidecar is a pure *reader*: it materializes a local flag + heartbeat
and posts nothing. `--auto-confirm` turns it into the direct path's **L2-delivered
producer**. Each cycle, for every `dm:<self>:*` topic carrying unread mail, it:

1. **journals** the topic into the S1 per-conversation journal
   (`~/.termlink/journals/journal.sqlite` via `scripts/journal-mirror.sh`), and
2. **auto-posts a mechanism-A receipt** —
   `channel post <topic> --msg-type receipt --metadata stage=delivered --metadata
   up_to=<latest-content-offset>` — confirming delivery with **no LLM turn**.

This makes the direct path **store-and-forward**: the journal row and the receipt
survive a recipient restart, so a sender's `agent-send.sh` sees DELIVERED even if
the recipient agent was mid-turn or briefly down. The confirm is mechanism **A**
(a durable receipt envelope), never the hub `channel.receipts` frontier
(mechanism B) — B stays the fallback-only producer (design §3).

**Idempotency (no ack spam).** The watermark acked is the latest *content* offset
(meta types — receipts/reactions/… — are excluded, exactly as `channel unread`
computes). A durable per-topic guard file under the notify dir records the last
acked offset; a new receipt is posted only when the content offset advances.
Excluding receipts from the watermark is load-bearing: otherwise the sidecar's own
receipt would bump the offset and it would re-ack every cycle.

**The `stage` ladder.** `stage=delivered` (sidecar, auto, no LLM) is level 2 of the
3-level confirm ladder — below `stage=read` (the agent at its yield point, a future
slice) and the reply turn itself (`acted`, mechanism C, already covered by
`agent-send.sh --await-reply`). It is one metadata key on the existing mechanism-A
envelope — no new receipt namespace. `agent-send.sh`'s receipt poll surfaces it:
`agent-send: DELIVERED (stage=delivered) — …`. An un-tagged receipt (pre-S3 / V3b)
still reads as plain `DELIVERED` — backward compatible.

**Not in this slice.** The sender's try-direct/fall-back routing branch, and making
the doorbell-ring optional on the direct path (the sidecar acks without a woken
interactive agent), are **S4** (T-2296 apex). S3 ships the recipient auto-acker +
the `stage` semantic + the sender's stage-aware *recognition* only.

```bash
# Recipient runs the sidecar as a journaling auto-acker:
bash scripts/notify-sidecar.sh --agent-id claude-code-A --auto-confirm --interval 15

# Test (hub-independent; TERMLINK_NOTIFY_TEST_TOPICS scopes to a throwaway topic):
bash scripts/test-sidecar-auto-confirm.sh    # 5 checks
```

## Relationship to the PTY doorbell

This **replaces** the preemptive doorbell as the *notify* mechanism. The doorbell
scripts (`agent-send.sh` / `agent-respond.sh`) remain in place for now; the
migration to flag-drop-only sending lands with V6 (T-2296), where the direct
transport and per-conversation journaling restructure the send path end-to-end.
Until then, the sidecar is the deterministic, self-detecting alternative an agent
can opt into today.
