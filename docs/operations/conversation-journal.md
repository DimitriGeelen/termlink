# Per-conversation journal (V6 slice S1)

arc-003 reliable-comms V6 (apex, T-2296) **slice S1**. A durable, mineable,
per-conversation record of `dm:` turns kept in local SQLite — the read-side mirror
that later V6 slices (S3 sidecar journaled-receipt, S5 firehose suppression) build
on. Ships **script-first** (no Rust rebuild), mirroring the V3a notify-sidecar
precedent.

Pure-additive and read-only against the bus: the hub firehose stays authoritative;
this mirror only `channel subscribe`s and writes its own sqlite. Moving `dm:` turns
OFF the firehose is S5 (not this slice).

## Store

`~/.termlink/journals/journal.sqlite` (override with `TERMLINK_JOURNAL_PATH` or
`--journal PATH`). One table:

```
messages(topic, offset, conversation_id, sender_id, msg_type, ts, payload,
         observed_addr, PRIMARY KEY(topic, offset))
```

The `(topic, offset)` primary key makes the mirror **idempotent** — re-running over
the same offsets inserts nothing new (`INSERT OR IGNORE`), so a full re-scan is cheap
and the safe default. `observed_addr` is reserved for the T-2297 hub-stamped source
address (empty until then).

## Populate — `scripts/journal-mirror.sh`

```bash
# Mirror every dm:* topic on the local hub (idempotent; safe to re-run / cron):
bash scripts/journal-mirror.sh

# One topic, from a remote hub, JSON summary:
bash scripts/journal-mirror.sh --topic dm:a:b --hub 192.168.10.122:9100 --json
```

Run it periodically to keep the journal fresh. In V6 slice S3 the V3a sidecar will
call it on each inbound-mail detection so the journal advances without a cron.

## Query — `scripts/agent-journal.sh`

Reads the journal (NOT the hub) — no network, no auth. `<conversation>` matches a
`dm:` topic name OR a `conversation_id`:

```bash
bash scripts/agent-journal.sh dm:a:b                     # human-readable history
bash scripts/agent-journal.sh dm:a:b --since-offset 40   # only newer turns
bash scripts/agent-journal.sh cid-1234 --json            # by thread, JSON envelope
```

## Test

`bash scripts/test-journal-mirror.sh` — 8 hub-independent checks (self-post →
mirror → row → query round-trip, idempotency, conversation_id resolution,
`--since-offset`, missing-journal error, empty query). SKIPs cleanly with no hub.

## Scope / next slices

- **S2** transport-select seam, **S3** direct-path sidecar journaled-receipt (writes
  here), **S4** try-direct/fall-back orchestration, **S5** journal-authoritative +
  firehose suppression. See `docs/plans/T-2296-v6-direct-transport-first-design.md`.
- **Open (deferred):** journal retention/compaction — the store grows unbounded;
  a reaper mirroring the offline-queue `dead_letters` discipline is a follow-up
  (design §5 Q6).
