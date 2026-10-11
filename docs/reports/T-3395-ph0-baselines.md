# T-3395 PH-0 baselines (read-only)

Date: 2026-10-11 (UTC ~19:00). Host: this host, local hub (pid from `/var/lib/termlink/hub.pid`). Step 3 v0.2.1 PH-0 6.2.1.a (2), exit (4).
Counts and sizes only. No message content or secret was printed, copied or stored; the scan script kept only (family, decoded size, utf-8 flag, artifact flag, ts, msg_type, offset) per message, in /tmp (not in the repo).

## Common method

1. `termlink channel list --json` gave 374 topics with `count` summing to 8,763 retained messages.
2. Each topic was read with `termlink channel subscribe <topic> --json --limit 1000 --cursor N`, paged by `next_cursor`. 8,476 envelopes were read (97%); the 287 difference is `count` including offsets that retention or compaction had dropped, or non-message entries. Zero topic read failures.
3. Size = length of the base64-decoded `payload_b64` (the content bytes), not the whole envelope.
4. Topic families: `inbox:` 276 topics, `dm:` 24, `agent-conv-` 8, `aef-` 13, the singletons agent-presence, agent-chat-arc, channel:learnings, framework:pickup, broadcast:global, and 48 other.

Limits: a snapshot of what retention still holds, not a history. Producers on this host are few (about 4 identity fingerprints) and dominated by sidecar consult/receipt traffic (sidecar.consult 1,931, sidecar.receipt 1,191, chat 1,030, heartbeat 1,000 of 8,476 messages), so the shape reflects today's workload.

## H-5 message sizes (threshold: p99.9 at most 4 KiB)

Source: method above.

| Scope | n | p50 | p95 | p99 | p99.9 | max |
|---|---|---|---|---|---|---|
| all | 8,476 | 207 B | 3,129 B | 7,440 B | 40,000 B | 83,902 B |

Messages above 4,096 B: 292 (3.4%). Above 8,192 B: 42 (0.5%).

By family (p50 / p95 / max bytes, n): inbox 210 / 2,410 / 27,063 (3,764); agent-chat-arc 149 / 895 / 8,944 (1,000); agent-presence 11 / 11 / 11 (1,000); channel:learnings 231 / 1,083 / 5,297 (865); dm 323 / 2,305 / 11,868 (598); framework:pickup 3,300 / 7,824 / 77,530 (324); aef- 1,434 / 2,822 / 6,862 (182); other 100 / 8,000 / 83,902 (710).

Result against the hypothesis: p99.9 is 40,000 B, about 10x the 4 KiB threshold, so H-5 FAILS today as stated. The messages above 8 KiB are concentrated in framework:pickup and "other" (large inline reports). An 8 KiB inline limit (PN-7) would send 42 of today's messages (0.5%) to blobs.

## H-6 agents per project (threshold: maximum at most 32)

Sources: `termlink discover --json` (115 sessions, all `ready`, all uid 0); `termlink channel list --json` (inbox topic names); `termlink channel subscribe agent-presence --json`.

- Sessions grouped by `metadata.cwd`: 23 distinct working directories. Largest: /opt/055-agentic-fleet-cockpit 36, /opt/1409-sprind 34, /opt/999-Agentic-Engineering-Framework 13, /opt/termlink 11.
- Largest number of registrations in one project = 36, above 32. H-6 FAILS on registrations (sessions) as the measure.
- Distinct `identity_fingerprint` among the 115 sessions: 4 (one fingerprint is on 112 of them). Distinct identities in any one cwd: at most 3. So distinct agent identities per project are far below 32, while registrations per project are not.
- Per-project mail: of the 276 `inbox:` topics, 273 have the form `inbox:<hub>/<project>`; they name 270 distinct project parts (3 repeat under another hub id). The other 3 carry no project part.
- agent-presence: 2 cv_key entries; 2 distinct senders in 1,000 heartbeats.

Limit: "agents of one project" has no directory today, and session `metadata` has no project field (0 of 115 carry one), so cwd is a proxy. NOT MEASURED: a true per-project agent count from a directory.

## H-7 blob sizes (threshold: largest at most 8 MiB)

Sources: `find /var/lib/termlink/bus/artifacts -type f` (0 files); envelopes with a non-null `artifact_ref` (3 of 8,476: 1 in "other", 2 in framework:pickup).

- The hub artifact store holds 0 files. Largest blob sent today: NOT MEASURED (no stored blob exists; the sizes behind the 3 `artifact_ref` envelopes were not read).
- Nearest stand-in, the largest inline payload: 83,902 B (about 0.08 MiB), far under 8 MiB.

## H-15 share of messages that reach an agent's transcript (baseline, no threshold)

Method: the 284 distinct `client_msg_id` values in the sidecar ledgers (`.context/sidecar/direct-ack.jsonl`, `receipts-sent.jsonl`, `receipts.jsonl`) were searched as fixed strings across all `~/.claude/projects/**/*.jsonl` (9.9 GB, 1,871 files) with `grep -roF -f ids.txt`. Only (file, id) pairs were kept and counted.

Result:
- 218 of 284 ids (76.8%) occur in at least one transcript.
- 126 of 284 (44.4%) occur in two or more distinct transcript files (92 in 1 file, 90 in 2, 36 in 3 or more).

Reading: one file is probably the sender writing the id into a tool call; two or more means a second session also holds it. 44.4% is the better proxy for "reached an agent other than the sender".

Limits: (a) only sidecar-tracked sends since 2026-10-06 are covered; inbox/dm/chat messages without a ledger `client_msg_id` are outside the sample; (b) an id in a transcript shows a session held it, not that the agent read or acted on it; (c) a resumed session can hold the id in two files; (d) transcripts are not scoped to a recipient, so a second file may be an unrelated session that read a ledger. Treat 44% as an upper-bound estimate of delivery to a distinct reader and 77% as an upper bound of visibility anywhere. NOT MEASURED: the share of all 8,476 hub messages.

## H-30 age of re-sends (threshold: 0 re-sends older than 14 days)

Sources: the sender-side sidecar ledgers above (285 distinct ids, 183 with more than one row); the offline queue `~/.termlink/outbound.sqlite` opened read-only (pending_posts 0, dead_letters 11).

- Ledger time span: 2026-10-06 11:33 UTC to 2026-10-11 16:58 UTC (about 5.2 days).
- Largest spread between the first and last row of one id: 45.4 hours.
- Re-sends older than 14 days: 0, but the observable window is 5.2 days, so the 14-day test CANNOT be answered. NOT MEASURED beyond 5.2 days; the sidecar ledgers keep accruing and the PH-2 re-measure applies.
- 11 dead-letter rows exist in the offline queue; their ages were not read (NOT MEASURED).

## FX-5 baseline: plaintext content fields a topic read token returns, per topic family

Source: the same scan. A "plaintext content field" is a message whose `payload_b64` decodes to valid UTF-8 and is non-empty; a topic read token receives all of them as readable text.

| Family | Messages read | Non-empty plaintext payloads | Empty payloads | Opaque (non UTF-8) |
|---|---|---|---|---|
| inbox: | 3,764 | 3,188 | 576 | 0 |
| agent-chat-arc | 1,000 | 1,000 | 0 | 0 |
| agent-presence | 1,000 | 1,000 | 0 | 0 |
| channel:learnings | 865 | 865 | 0 | 0 |
| other | 710 | 705 | 5 | 0 |
| dm: | 598 | 540 | 58 | 0 |
| framework:pickup | 324 | 320 | 4 | 0 |
| aef- | 182 | 182 | 0 | 0 |
| broadcast: | 17 | 17 | 0 | 0 |
| agent-conv- | 16 | 16 | 0 | 0 |
| Total | 8,476 | 7,833 | 643 | 0 |

The PH-0 baseline for FX-5 is 7,833 plaintext payload fields across 8,476 messages; no payload on the hub is encrypted or opaque today (every payload decoded as UTF-8).

Limits: this counts `payload_b64` only. Plaintext string values inside `metadata` (for example `from_project`, `observed_addr`) and `sender_id` are also readable by a token holder and were NOT counted. The scan used this host's own (root) credentials; a probe with a restricted topic read token was NOT MEASURED. Counts move with retention; later phases must use the same method.

## Reproduce

The scan script is kept outside the repo: list topics, page each with `termlink channel subscribe`, decode `payload_b64`, record sizes only. The H-15 grep and the H-30 ledger pass use the commands named above.
