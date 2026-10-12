# T-3427: FX-5 disclosure baseline (PH-0, taken 2026-10-13)

FX-5 (docs/design/interactive-agent-communication-03-floor-and-phasing.md:134): "count plaintext
content fields on the hub per phase: never above the PH-0 baseline". No baseline was recorded
before the first real-agent wave; this is it.

## Classification rule (applied to every row)

A **content field** carries message content or text derived from it: a body, a reply, a
free-text evidence note, a topic description. **Not counted:** routing metadata (ids,
conversation ids, offsets, timestamps, signatures, digests, roles, addresses, capability lists,
stage names). One count = one field of one rail that the hub stores readable. Base64 is
encoding, not protection, so `payload_b64` counts. Signing proves origin and hides nothing.

## Baseline: 8 fields

| # | Rail | Field | Evidence |
|---|---|---|---|
| 1 | channel.post (any topic) | envelope `payload` (decoded from `payload_b64`, stored as bytes) | crates/termlink-hub/src/channel.rs:805-811, stored at :1001 |
| 2 | dm turn (send) | envelope `payload` = the message text | scripts/agent-send.sh:514 |
| 3 | dm turn (reply) | envelope `payload` = the reply text | scripts/agent-respond.sh:129 |
| 4 | chat-arc broadcast | envelope `payload` = broadcast text | scripts/chat-arc-broadcast.sh:153 |
| 5 | receipt (read ack) | `metadata.evidence`, free-text note | scripts/notify-ack-read.sh:176-178 |
| 6 | topic description | `metadata.description` on a `topic_metadata` post | crates/termlink-cli/src/commands/channel.rs:4750 (`cmd_channel_describe`) |
| 7 | inbox mirror | `inbox:<target>` envelope `payload.payload`, the hub's copy of a deposit | crates/termlink-hub/src/channel.rs:272-275, caller router.rs:380 |
| 8 | sidecar (PH-1) | wire `payload.body`, plaintext per arc-011-ph1-architecture 5.1 | crates/termlink-sidecar/src/wire.rs:60 |

Per rail: channel.post 1, dm-turn 2, chat-arc 1, receipt 1, topic 1, inbox-mirror 1, sidecar 1.
Total 8. Rows 2 and 3 write the same hub field as row 1 through different senders; they are
counted separately so that a sender that stops using the plaintext field lowers the count.

## Examined and counted as zero

- **Sidecar stage record** (`sidecar.stage.v1`, wire.rs `Stage`): from, conv, msg_id, stage name,
  ts, chain_head. No content. Sidecar post metadata (send.rs:356-362, 397-401): conversation_id,
  msg_id, digest, pk, sig. Routing only. The digest is a sha256 of the body, not the body.
- **Agent-presence heartbeat** (scripts/listener-heartbeat.sh:166-190): `payload` is the role
  label; metadata is agent_id, role, listen_topics, host, pty_session, capabilities, addr, cv_key.
  Identity and routing; no message content.
- **routing:lint mirror** (channel.rs:321-352): role and topic-prefix policy warning built by
  `topic_lint::warning_payload`. No message text.
- **event.broadcast mirror to broadcast:global** (channel.rs:199-231): `mirror_event_broadcast`
  has NO production caller (only a unit test at channel.rs:4127). Not live; not counted. If a
  caller is added it must enter the manifest.
- **Artifacts** (`artifact_ref`, termlink-bus artifact_store.rs): a file-transfer blob store.
  Not message content in the sense of the rails above. **Could not verify** whether the FX-5
  author intended it to count; a ruling to count it adds 1.

## How to re-derive

`bash scripts/count-hub-plaintext-fields.sh [--max 8] [--json]`. Exit 0 counted, 1 above `--max`,
2 tooling. Two mechanisms: a manifest of the 8 fields, each counted only while its anchor text is
still in its file; and a drift scan of every `--metadata key=` in the sender scripts and every
metadata key in the sidecar sender, where a key outside the routing allow-list is counted as an
unclassified content field. So an added free-text metadata key raises the total; an added
payload-bearing rail does not until someone adds it to the manifest. Fixtures:
`bash tests/count-hub-plaintext-fields-fixtures.sh` (11 assertions).

## Limits (read a green narrowly)

1. The script cannot see a brand-new rail whose payload write uses a shape the manifest does not
   name. The drift scan covers metadata keys only. A PH-2+ reviewer must add each new content-bearing
   write to the manifest; compare with `--max 8` from then on.
2. It counts fields in code, not records on a live hub, so it does not show what a deployed hub holds.
3. Vendored AEF rails (`framework:pickup` and inbox topics written by AEF code) are outside this
   tree (G-062). Their payloads are plaintext content fields too, so the true fleet baseline is
   this 8 plus those. **Not measured.**
4. The fixtures copy the real anchor files, so they pin today's tree rather than a canned sample: a
   legitimate edit of an anchor line changes the fixture result and forces a conscious baseline update.
