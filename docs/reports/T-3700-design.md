# T-3700 design — P-12 / B-2: inbox consumer list, proven/claimed readers, bind by key

Prepared 2026-11-19. Implements AL-30 (06a 2.14) with operator ruling E on F-1 (T-3700 `## Decisions`): every inbox
read and claim names its reader; a reader with a key card signs; consumers are `proven` or `claimed`; a bind holds by
key. Spec sources and F-1 evidence: `T-3700-spec-extract.md`.

## 1. Scope

1. Topics: `inbox:*` only. arc-011 conversations ride inside inbox topics (`conv` is a payload field,
   `crates/termlink-sidecar/src/wire.rs:55`), so AL-30's "or conversation topic" adds no other topic today.
2. Consumer actions on an inbox (all three are recorded, FCW-209):
   1. read: `channel.subscribe` (poll, long-poll, read; the only read RPC);
   2. claim: `channel.claim`;
   3. ack: `channel.post` with `msg_type: receipt` (`termlink-mcp/src/tools.rs`, the ack tool), already signed.
3. Out of scope: posts of other types (senders post into inboxes; never refused by a bind), cross-hub copies (PH-3).

## 2. Wire (additive, old clients keep working)

1. `channel.subscribe` and `channel.claim` gain an optional `reader` object:
   `{agent, instance, ts_ms, signer_pk?, sig?}`.
   1. `agent`: plain agent name (`key_card::valid_name`); `instance`: the sidecar instance id (as `from_instance`).
   2. `sig` (ed25519, hex) over the domain-separated bytes
      `termlink-inbox-read/v1\n<hub_id>\n<topic>\n<method>\n<cursor-or-offset>\n<ts_ms>\n<agent>\n<instance>`.
      The prefix makes a read signature useless as a post signature and vice versa; `hub_id` stops replay on another
      hub; `ts_ms` must be within ±120 s of the hub clock (P-23 skew bound) and is not otherwise remembered.
2. A call with no `reader` is recorded as reader `(unnamed)`, instance = the transport kind (`local` / `tcp:<peer ip>`).
3. Receipt posts need no new field: the poster's `sender_pubkey_hex` is already verified; the hub maps it to an agent
   through the key cards (`proven`) or records the payload's claimed `from_agent`/`(unnamed)` (`claimed`).

## 3. Hub

1. **Proof.** A reader is `proven` when all hold: signature verifies; `ts_ms` in window; the agent's key card exists
   and its signing key equals `signer_pk`; the key is not on the revocation list (P-15). Otherwise `claimed`.
   A `sig` that is present but fails to verify (CHANNEL_SIGNATURE_INVALID), one by a key that is not on the named
   agent's card (CHANNEL_KEY_CARD_MISMATCH), or a revoked key (KEY_REVOKED) **refuses the call** — a broken proof is
   never downgraded to a claim (codes corrected after review round 1, CC 14).
   A valid signature with `ts_ms` outside the window is **not** refused (a skewed clock must not cut a sidecar off its
   inbox; P-23 reports the skew): the reader is listed `claimed (clock skew)`, and after a bind it is refused like any
   unproven reader.
2. **Consumer table.** Per topic, keyed by (agent, instance): `{agent, instance, strength, first_ms, last_ms, actions}`.
   1. 16 entries per topic; on overflow evict the oldest by `last_ms` and write an admin-log row `inbox-consumer-evicted`.
   2. Durable: `<runtime>/inbox-consumers.json`, atomic write. Written when an entry is new, changes strength or action
      set, or `last_ms` moved by ≥ 60 s since the last write (deviation from 2.14.f(3) "written with every RPC": the
      sidecar long-polls every tick; a crash loses ≤ 60 s of `last_ms`, inside the 24 h window and the 60 s flag).
   3. Updated and checked under the same per-topic lock as the bind (2.14.f(4)).
3. **RPC `inbox.consumers {topic, window_s?}`** (Observe; default window 86 400 s): entries with `last_ms ≥ now − window`,
   plus `bound: {agent, key_fp} | null`. An unreadable table answers an error, never an empty list (FCW-125).
4. **Bind.** `hub.inbox_bind {topic, agent}` / `hub.inbox_unbind {topic}` (Execute, local socket only, in ADMIN_METHODS):
   admin-log row first, then `<runtime>/inbox-bindings.json` (durable, atomic). The binding stores the agent and the
   sha256 fingerprint of its card's signing key at bind time; binding an agent with no card is refused.
   CLI: `termlink hub inbox consumers|bind|unbind`. Binds reach the operator through runme only.
5. **Refusal after bind (FCW-124).** On a bound topic, a read, claim or receipt post passes only when it is `proven` as
   the bound agent with the bound key fingerprint; everything else is refused with
   `inbox bound to <agent>` and an admin-log row (rate-capped like the T-3703 refused rows, 10/min).
   A card rotation of the bound agent therefore needs a re-bind (stated, visible in the refusal reason).

## 4. Sidecar

1. Every inbox `channel.subscribe` it makes carries a signed `reader` (`pull.rs`, and `peek`). The sidecar makes no
   hub `channel.claim` (its "claim" in `deliver.rs` is a local `O_EXCL` file); liveness reads use `channel.first_at`
   (4.4). Corrected after review round 1 (CC 10, OC 7).
2. Each tick: `inbox.consumers(MY_INBOX, 24 h)`.
   1. Err → status `consumers: unknown (<why>)` (never "one", FCW-125).
   2. Entries other than (MY_AGENT, MY_INSTANCE) → status `consumers: inbox consumed by two identities: A (proven),
      B (claimed)` and needs-attention coalesced on (topic, other agent+instance, UTC day). A same-agent, other-instance
      entry is the V-12 second copy and is named as such.
   3. Only own entry → `consumers: one (proven)`; a hub without the RPC → `consumers: unknown (hub predates consumer list)`.
3. Status also shows `bound: <agent>` when the hub reports a binding.
4. **Liveness is not a consumer (agent choice, 2026-11-19, found while building).** Every sidecar with an address book
   read one message of the *main's* inbox to judge "stuck" (`liveness.rs`, T-3646). As a content read by another
   agent it would flag the main's inbox and be refused after a bind. It now calls `channel.first_at {topic,
   from_offset}` (new, Observe), which returns only the offset and timestamp of the first record at or after
   `from_offset`, no content, and records no consumer. On a hub without it, the old one-message read is the fallback.
   No content is exposed, so this opens no BP-15 route. Reported to the operator; overrulable.

## 5. Diagnostic peeks (operator ruling D, 2026-11-19)

1. The sidecar gains a `peek` verb (CLI `termlink sidecar peek` and the local API): it reads the inbox from the hub with
   its own signed `reader`, returns the messages, and changes no cursor, stage or store state.
2. Direct key-less reads (CLI `channel subscribe`, MCP `channel_state`, AEF `fw sidecar inbox --peek` today) get no
   exception: listed `(unnamed)`/`claimed`, flagged, refused after a bind.
3. AEF is asked over TermLink to route its `--peek` through the sidecar (its code, a request only).

## 6. Evidence plan

1. E-66 (L4, real hub in a temp runtime dir, never the live one): fx-z subscribes and acks fx-a's inbox → flagged
   within 60 s naming both; bind fx-a → fx-z refused with reason + admin-log row; 17 identities → oldest evicted and
   logged; second instance of fx-a → flagged within one tick. E-66.6: read-only fx-z listed and flagged, refused after
   bind. Controls: mutants recording only subscribers / only acks+claims miss fx-z.
2. E-133: the per-tick check fires on injection (the E-66 run under the sidecar tick loop).
3. E-169 (B-2 part, L2): consumer RPC failing → `consumers: unknown`; control mutant defaulting to empty reports one.
4. Ruling-E cases: unsigned name equal to the bound agent refused after bind; bad signature refused before bind;
   revoked key refused; stale `ts_ms` (> 120 s) listed `claimed (clock skew)` before a bind and refused after it;
   a signature replayed on another topic or hub does not verify.
5. Mutant list (all must be killed): skip-signature-check, ignore-ts-window, bind-by-name, default-unreadable-to-empty,
   record-subscribers-only, record-acks-only, no-eviction-log, instance-ignored (V-12).

## 7. Changes after review round 1 (dispositions: `T-3700-reviews/r1-dispositions.md`)

1. Entries are keyed by (agent, instance, strength, key): a claimed reader never merges into a proven entry. A reader
   object needs an instance, and `-` (the ack sentinel) cannot be claimed. The sidecar counts as itself only an entry
   proven by its own key; anything else under its name is "claims this agent" (or "second copy" when proven by
   another key).
2. Only topics that exist on the bus are recorded; topics over 256 bytes are not recorded or bound; eviction and
   refusal rows are capped at 10 a minute per kind with a `suppressed` count; a full table never evicts an inbox read
   inside the 24 h window (the new topic goes unrecorded, with a capped `inbox-consumer-table-full` row).
3. A corrupt or unknown-schema table is moved to `inbox-consumers.json.corrupt-<ms>` (admin row) and the list is an
   error ("history incomplete since …") until the window has passed the reset.
4. The sidecar's `consumers` line carries `checked_ms`; older than 60 s it reads `unknown (no check since …)`; it is
   cleared at start.
5. A long-poll re-checks the binding when it returns. An ack proves only when the post's `ts` is within ±120 s.
   `bind` refuses a revoked card key. Liveness falls back to the content read only for an old hub (-32001).
