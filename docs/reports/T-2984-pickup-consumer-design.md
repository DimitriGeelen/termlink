# T-2984 — A structural consumer for `framework:pickup` (G-063)

**Status:** inception, exploration complete. No build work authorised.
**Owner of the decision:** human. This artifact informs a go/no-go; it does not make one.

## Why this artifact exists (C-001)

Inception research is written to a file *before* the research runs, and updated as
findings land. Conversations are ephemeral; this file is the thinking trail.

## Measurements

### A consumer exists — it is wired to a different rail (IW-1)

Two distinct things in this project are called "pickup":

| rail | fed by | consumer | status |
|---|---|---|---|
| **file inbox** `.context/pickup/inbox/` | `fw pickup send` | `fw pickup process`, cron **every 60s** + every 15min, both flock-guarded | automatic, draining |
| **hub topic** `framework:pickup` | peer projects over TermLink | — | **nothing reads it** |

`.agentic-framework/lib/pickup.sh` — the processor — contains **zero** references to
`framework:pickup`. The only code that touches the topic is
`.agentic-framework/lib/pickup-channel-bridge.sh`: **106 lines, outbound only**
(`termlink channel post`, no `channel subscribe` anywhere in it).

`.context/working/.pickup-bridge.log` is 39 lines and every one reads
`posted via=channel.post`. It is a send log, not a receive log.

So the gap is not "no consumer". It is that the outbound half is fully automated —
cron-driven, flock-guarded, dedup-logged — and the inbound half does not exist.
Pipeline state: inbox 0, processed 64, rejected 1, auto-deferred 1.

### The genuine inbound backlog is ONE filing (IW-2)

Canary acked to offset 125. Six filings since. Attributed against our own commit log:

| offset | origin | evidence |
|---|---|---|
| 126 | **ours** | commit `64523bdf5` "filed at framework:pickup@126" |
| 127 | **ours** | commit `786014647` "…@127" |
| 128 | **ours** (P-081, `from=root`) | sits between two confirmed own filings |
| 129 | **ours** | commit `363c69579` "…@129" |
| 130 | **ours** | commit `01351f6f1` "…@130" |
| 131 | **PEER** | `from=proxmox-ring20-management` — the only genuine inbound item |

The canary suppressed 3 as ours and **fired on 3, of which 2 were also ours**
(126 and 128 arrive as `from=root` — attribution unknown, which by design fires:
*"a false fire is cheap while a false silence is the whole point of G-063"*).

**The firing set is 67% self-echo.** The leak mechanism is already on record:
`metadata.from_project` is not sender-settable (recorded on T-3025), so our own
filings cannot identify themselves and fall through to the unknown-attribution arm.

### The existing auto-filer already duplicates (IW-4)

`Pickup:` tasks corpus-wide: **70 total, 9 duplicate name-groups, 11 extra task files
— a 16% duplication rate.**

The spacing names the cause:

```
T-2936  created 2026-09-09T15:40:02Z  ┐ same name
T-2937  created 2026-09-09T15:41:01Z  ┘ 59s apart
T-2944  created 2026-09-09T18:07:01Z  ┐ same name
T-2945  created 2026-09-09T18:08:01Z  ┘ 60s apart
```

That is exactly the `pickup-drain-1m` interval: the envelope is re-filed on the next
minute's run. `dedup.log` did not stop it. The two cron jobs share a flock
(T-2870), so this is not a race between them — it is one envelope surviving into a
second drain.

The two worst groups carry **three** copies each, and both are **our own filings**
(`(from 010-termlink)`) — T-3019/T-3020/T-3021 are three tasks for one filing.

### The destination is the bottleneck (IW-4)

**150 tasks awaiting human review**, 19 pending decisions, **143 active tasks owned
by `human`**. `Pickup:`-prefixed tasks are visibly part of that backlog, sitting
2–11 days at `NO-REC`.

## Findings

**F1 — The premise holds; the implied remedy is the one thing that would make it
worse.** `framework:pickup` genuinely has no inbound consumer, and that is a real
G-063 write-only sink. But "build a structural consumer that auto-files tasks"
points a second automated filer at a queue 150 deep that nobody drains, using a
filer that already duplicates 16% of what it files.

**F2 — This is a signal-to-noise problem, not a throughput problem.** One genuine
peer filing is outstanding. One. It is not buried under peer volume; it is buried
under *our own echo* — the canary's firing set is 2 parts self to 1 part signal.
Auto-consumption would automate the processing of our own bug reports back into our
own queue.

**F3 — The cheapest high-value fix is upstream of the consumer question.** Make the
self-filter able to identify our own filings (the `metadata.from_project` gap,
T-3025) and the canary's firing set becomes exactly the genuine inbound work. The
backlog then reads "1 filing", which is tractable by the manual triage that already
exists, and no new automation is required to clear it.

**F4 — The duplicate-filing defect is a separate, confirmed bug and it gates any
consumer work.** Wiring a second source into a filer that re-files on the next
minute doubles a known defect rather than extending a working one. One bug = one
task; it is not fixed here.

## IW-3 — what a consumer would have to decide, and who may decide it

Per filing, a consumer must choose: ignore / file a task / route to an existing task
/ defer pending a blocker. The middle two are the ones with teeth, and both are
judgements about *whether a peer's report matters to this project* — which is the
same judgement the 150-deep queue exists to record and the human has not been able
to keep up with. Automating the judgement does not add capacity; it adds volume to
the side that is already saturated. Routing is agent-delegable only once there is a
drain; until then it is sovereign.

## Recommendation

**NO-GO on the filing's scope. GO on a materially smaller one, in dependency order.**

1. **Fix the attribution leak** so the canary fires only on genuine inbound work
   (`metadata.from_project` not sender-settable — T-3025). Smallest change, largest
   effect: the backlog reads 1 instead of 3, and the canary becomes trustworthy.
2. **Fix the 60-second duplicate re-file** in the existing pickup processor. Its own
   task; it is a live defect independent of this inception.
3. **Only then** consider a topic → inbox bridge. The inbound half would reuse the
   same processor, so it must not be wired to a filer that duplicates.
4. **The drain is sovereign.** 150 tasks awaiting review, 143 active `owner: human`.
   No consumer design changes that number, and any auto-filer enlarges it.

**Explicitly OUT:** building the consumer; changing the canary's
unknown-attribution-fires rule (it is correct — a false silence is the failure this
canary exists to prevent, so the fix belongs at attribution, not at the firing gate).

## Side findings (not this task's to fix)

- **The inception Open-Questions gate refuses read-only commands.**
  `sed -n '70,100p' <file>` — no `-i`, writes nothing — was blocked as *"it matches a
  file-write pattern"*. Same family as the P-002 read-only allowlist gaps: the
  matcher cannot distinguish `sed -n 'N,Mp'` from `sed -i`.
- **`checkpoint.sh budget` crashed again** (9th unprompted reproduction of T-3018),
  on the first command after compaction.
