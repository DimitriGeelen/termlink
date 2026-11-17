# T-3704 (arc-011 P-16, B-6) — design note: agent key card, post-time check, signed dead letter

Source of the spec: `docs/reports/T-3704-spec-extract.md` (verbatim B-6, R-44/R-62/R-75, E-69/E-70/E-169, AL-33,
AL-34, AL-23 step 4, AL-20, and a map of the current code). This note settles the 15 gaps that extract lists, splits the
unit into two slices, and states what each slice proves. Written 2026-11-17, before any code.

## 1. What exists today (measured)

1. The hub already verifies every `channel.post` signature, but against the key the poster supplies
   (`sender_pubkey_hex`), and requires `sender_id == fingerprint(key)` (`channel.rs:906-968`). Any fresh key passes:
   nothing binds a key to an agent or a project. That binding is what B-6 adds.
2. The arc-011 sidecar signs posts with the agent's own key over `canonical_sign_bytes` (`wire.rs:213`), the same bytes
   the hub verifies; the claimed agent (`from_agent`) and instance are only inside the signed payload (`wire.rs:39-53`).
3. Agent public keys are already provisioned by runme as root-owned register files
   (`/etc/termlink-sidecar/keys/<agent>.pub`, P7-2, T-3438), and the encryption keys likewise (`<agent>.x25519.pub`).
4. No AL-20 (RFC 8785) implementation exists in `crates/`; P-15 (T-3703, captured) is the first unit that needs it.
5. The hub has no instance registry and no notion of "retired".

## 2. Slices

The unit splits cleanly at AL-20:

- **Slice A (this task, now): the card and the post-time check.** A card store at the hub, written by a runme-only
  admin path; every post of an arc-011 message type to `inbox:*` is checked against the card of the agent it claims to
  be from (FCW-133, FCW-135); the receiver checks `from_agent` against the key too (AL-23 step 4). Needs no AL-20:
  the card is written by a privileged path, not by a signed statement. Delivers R-75 "verified as P's agent" and the
  E-69 post half and E-169's "poster without a card".
- **Slice B (after P-15 has built AL-20): key replacement by old-key statement (FCW-134) and the signed DEAD letter
  (AL-34, FCW-136, E-70).** Both are AL-20-signed statements; building AL-20 twice would fork the canonical form that
  P-15's test vectors pin. So the order becomes P-16 slice A, then P-15 (T-3703), then P-16 slice B. The plan lists
  P-16's only dependency as P-10 (built), so slice A is in order.

## 3. The 15 gaps, settled (numbers as in the extract)

1. **Card key.** Cards are keyed by agent name; one card per agent name per hub, enforced at registration. The hub
   reads `from_agent` from the `sidecar.msg.v1` / `sidecar.stage.v1` payload (it already parses stage payloads in
   `mail_guard.rs`) and requires `card.signing_key == sender_pubkey_hex`; the existing signature verify then stands
   as is. A post whose payload does not parse is refused like a post without a card (fail closed).
2. **Bytes verified.** The transport signature stays (`canonical_sign_bytes`); the card adds key == card key. AL-20 is
   used only for signed hub state (slice B).
3. **Project binding (R-75.e).** Agent names are unique per hub (gap 1), and the card carries `project`; a session of
   project Q that claims to be P's agent must post with P's agent key, which it does not hold. The receiver also checks
   `from_agent` against the key (gap 11). No wire change.
4. **Which message types.** Exactly `sidecar.msg.v1`, `sidecar.stage.v1` and `sidecar.liveness.v1` (the last on
   `liveness:*`). Every other type on `inbox:*` (AEF's `sidecar.receipt`, chat posts, hub-originated envelopes) is
   untouched: the check never sees them (never-decommission rule).
5. **Roll-out.** A hub setting with three values: `off` (default in this build), `log` (record what would be refused,
   refuse nothing), `enforce`. P-31 publishes the cards; the operator moves the setting to `log`, then `enforce`, by
   runme, after the log shows no false refusal. "Check off at the hub; cards kept" is the rollback.
6. **DEAD needs the hub public key.** Slice B: `hub.version` gains `signing_public_key` (64 hex); the sidecar checks
   sha256 of it against its pin before using it. (The pin stays the fingerprint.)
7. **Where DEAD lives.** Slice B, settled there with P-15's form; recorded as open.
8. **DEAD seq rule.** Slice B: refuse a lower seq; an equal seq only for byte-identical content (R-60.a's wording;
   AL-34's `>` read as "rises or is the same record").
9. **Who may write a card.** An admin RPC `hub.key_card_put` that accepts only `execute` scope over the local unix
   socket, writes an admin-log row (B-3) before it answers, and is called only by runme. While agents run as root
   this is a convention plus a log, not a barrier: a root agent could call it. Recorded as a known limit (RR-2 class),
   visible in the admin log; the barrier comes with the non-root agent work.
10. **Name clash.** The new state is `DEAD` (R-44); the existing sidecar poison event stays `DEAD_LETTER`. Slice B
    labels them apart in status.
11. **Receiver.** The receiver keeps its roster (the keys allowed to send) and adds the missing check: the signer key
    must be the key of the `from_agent` the message claims (the receiver reads the same register files P7-2 provisions,
    or the hub's card over the pinned path). This closes "post as `fx-a` signed with `fx-b`'s key" at the receiver
    when both keys are rostered.
12. **Needs-attention at the hub.** The hub writes an admin-log row per refusal and counts refusals in `hub.status`;
    the sidecar raises the needs-attention entry when its post is refused (it already does for refusals).
13. **Canonical JSON.** Slice A signs nothing new. Slice B uses P-15's AL-20 implementation with the V-REV vectors.
14. **Two levels.** E-169 (L2) on the hub fake; E-69 (L4) on a throwaway real hub, as T-3366 did.
15. **Key format.** The card stores `signing_key` and `encryption_key` as `ed25519:<64 hex public key>` and
    `x25519:<64 hex>`.

## 4. Slice A, what gets built

1. Hub: card store `<runtime_dir>/key-cards/<agent>.json` (0600, atomic write, `seq` compare-and-set under a per-agent
   lock), schema `termlink.key-card/1` with `agent, project, signing_key, encryption_key, instances, seq, by`.
2. Hub: `hub.key_card_put` / `hub.key_card_get` (admin; put needs `execute` on the unix socket; admin-log row first).
3. Hub: the post check in `handle_channel_post` for the three types, behind the `off|log|enforce` setting; refusals
   `CHANNEL_NO_KEY_CARD` (FCW-135) and `CHANNEL_KEY_CARD_MISMATCH` (FCW-133), each with an admin-log row.
4. CLI: `termlink hub key-card put|get|list` for runme.
5. Sidecar receiver: `from_agent` must name the agent whose register key signed it (FCW-98).
6. Tests: E-169 card case on the fake; E-69's post half (`fx-a` signed with `fx-b`'s key; hub secret only; no card) on
   a throwaway real hub, at the hub and, with the hub check off, at the receiver; the mutant
   `fc-01-verify-accepts-any-signature` and mutants per refusal.
7. No runme action in this slice: publishing cards is P-31.
