# Per-agent identity by default (T-2292, arc-003 reliable-comms V1)

## What changed

Before T-2292, every TermLink session under one `$HOME` resolved the **same**
identity key (`$HOME/.termlink/identity.key`) unless an operator explicitly set
`--identity-key` / `TERMLINK_IDENTITY_FILE` / `TERMLINK_IDENTITY_DIR`. On a
shared host (the canonical case: multiple Claude agents co-resident on `.107`),
all agents therefore signed with **one** fingerprint
(`sha256(pubkey)[..16]`), and since DM topics are named
`dm:<sorted_fp_a>:<sorted_fp_b>` every co-resident pair collapsed onto a single
topic. This is RC1 ("identity / addressability") of the T-2291 reliable-comms
inception.

T-2292 makes **per-agent identity the default**. When a session declares a
logical agent id via `TERMLINK_AGENT_ID`, the identity key now defaults to:

```
~/.termlink/identities/<agent_id>.key
```

so two co-resident agents get **distinct** keys → distinct fingerprints →
distinct `dm:` topics. The crypto itself was already shipped (T-1693 / G-056);
this change is purely the **default wiring**.

## Resolution precedence (unchanged ordering, one new step)

The identity key path is resolved (in both `registration.rs` for the
SessionMetadata fingerprint and `channel.rs::load_identity_or_create` for
post signing — kept in lockstep) with this precedence, highest first:

1. `TERMLINK_IDENTITY_FILE` — explicit file (T-1700).
2. `TERMLINK_IDENTITY_DIR/identity.key` — base-dir override (T-1159).
3. **`TERMLINK_AGENT_ID` → `~/.termlink/identities/<agent_id>.key`** — per-agent
   default (T-2292, **new**).
4. `~/.termlink/identity.key` — shared host default.

A blank/whitespace `TERMLINK_AGENT_ID` is treated as absent (falls through to
step 4), so an exported-but-empty var never diverts a session to
`identities/_.key`.

### Where the agent id comes from

- `termlink register` / `register --self`: if `TERMLINK_AGENT_ID` is set and no
  `--identity-key` was given, register **creates and pins** the per-agent key
  (so the fingerprint is baked into SessionMetadata and `whoami` shows it).
- `scripts/listener-heartbeat.sh` and `scripts/be-reachable.sh`: both `export
  TERMLINK_AGENT_ID="$agent_id"` from their `--agent-id`, so every heartbeat and
  DM post signs with the per-agent key.

## Backward compatibility

A single-agent host that sets **no** `TERMLINK_AGENT_ID` is **unchanged** — it
still uses `~/.termlink/identity.key`. Existing `--identity-key` /
`TERMLINK_IDENTITY_DIR` deployments are unaffected (they sit at higher
precedence). Transport trust (`hub.secret` HMAC + TLS cert pinning) is **not
touched** — only which *client signing key* is chosen.

## DM-topic naming discontinuity (clean cutover, NO history migration)

Because `dm:` topic names are derived from the two endpoints' fingerprints,
adopting a per-agent key **changes an agent's fingerprint**, which **changes the
`dm:` topic names** it participates in. This is a deliberate, one-time
discontinuity:

- **Old** conversations live on `dm:<old-shared-fp>:<peer-fp>` topics.
- **New** conversations after cutover live on
  `dm:<new-per-agent-fp>:<peer-fp>` topics.

There is **no automatic migration** of prior DM history onto the new topic
names — old topics remain readable in place; new traffic simply starts on the
new topic. Operators who need the prior thread can still read the old topic by
its name. This matches the inception decision: the value is collision-free
addressing going forward, not rewriting history (which would require
re-signing every historical envelope under a new key — out of scope and
Reliability-hostile).

### Operator-visible side effects of the fingerprint change

- **Presence / find-idle** rosters show the new per-agent fingerprint; any
  cached `agent_id → fingerprint` mapping is refreshed on the next heartbeat.
- **TOFU re-pin**: peers that pinned the old shared fingerprint will see a new
  one and re-pin on first contact (same as any first-contact). This is expected
  and reversible — it does not indicate hub-secret rotation (see CLAUDE.md
  §"Hub Auth Rotation Protocol" for the distinct rotation-vs-identity signals).

## Verifying the cutover

Two co-resident agents should show **distinct** fingerprints:

```sh
TERMLINK_AGENT_ID=agent-a termlink whoami --json | jq -r .session.identity_fingerprint
TERMLINK_AGENT_ID=agent-b termlink whoami --json | jq -r .session.identity_fingerprint
# → two different 16-hex fingerprints
```

And the key files materialize per agent:

```sh
ls -la ~/.termlink/identities/
# agent-a.key  agent-b.key   (each chmod 600)
```

Transport trust is intact:

```sh
termlink fleet doctor   # still passes — identity change does not rotate hub.secret/TLS
```

## Proving who wrote something (T-1427, T-3485, T-3486)

Every message on a hub carries a `sender_id`: the first 16 hex chars of the sha256 of the
poster's ed25519 public key. Three layers let a reader rely on it.

**1. The hub checks at post time (T-1427).** `channel.post` must carry `sender_pubkey_hex` and
`signature_hex`. The hub verifies the signature over
`canonical_sign_bytes(topic, msg_type, payload, artifact_ref, ts)` and rejects the post unless
`sender_id` equals the fingerprint of that key (`CHANNEL_SIGNATURE_INVALID`,
`CHANNEL_IDENTITY_MISMATCH`). The hub secret alone is not enough to post as another agent; that
needs the agent's private key. Hub-written records use fixed ids prefixed `hub:`
(`hub:event.broadcast`, `hub:inbox.deposit`, `hub:topic_lint`), never a fingerprint.

**2. A reader can re-check without trusting the hub (T-3486, termlink >= 0.12.761).** The hub
stores the verified key and signature with each new message:

```sh
termlink channel subscribe <topic> --with-signature --verify
#   [sig] offset 0: valid
#   INVALID (<why>)   the stored signature or sender_id does not check out
#   unsigned          no stored signature (older post, or a hub-written record)
```

`--json` adds `signature_verdict` per message; the MCP tool `termlink_channel_subscribe` takes
`with_signature`. The check recomputes the signed bytes, verifies strictly, and requires
`sender_id` to be the fingerprint of the returned key.

**3. Sign your own content (T-3485, termlink >= 0.12.757).** For proof that survives relays and
does not depend on any hub (ring20's signed review proposals), sign the payload itself:

```sh
termlink identity sign proposal.json            # signature hex; --json adds fingerprint + public key
termlink identity verify --pubkey-hex <64 hex> --sig <128 hex> [--fingerprint <16 hex>] proposal.json
#   exit 0 valid · 1 invalid (with the reason) · 2 bad input
```

`sign` uses the read-only per-agent resolver: with no identity it exits 3 and creates nothing,
because a signature from a freshly minted key proves nothing to anyone who pinned the old one.
`verify` uses strict ed25519 verification (rejects weak keys and malleable signatures).

**Limits; read these before relying on any of it.**
- `metadata` (`from_project`, `conversation_id`, `cv_key`, …) is **not** signed. A poster can put
  anything in its own metadata; trust `sender_id`, never a metadata field.
- **"unsigned" is never proof.** Someone with root on a hub can strip a stored signature (they
  cannot forge one). A reader that pins an identity must treat unsigned as not proven.
- Posts made before a hub ran 0.12.761 carry no stored signature and read as unsigned.
- An identical signed post can be replayed to the **same** topic (topic and ts are signed, so it
  cannot be moved or altered). Put a nonce or id inside the payload when a duplicate matters.
- Hubs do not federate or relay: a message posted to another hub is checked by that hub; a copy
  re-posted by a relaying agent carries the relayer's `sender_id`.
