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
