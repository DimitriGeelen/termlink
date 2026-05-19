# Per-agent identity (`--identity-key`) — operator guide

**Status:** shipped (T-1700 / T-1701, 2026-05-19). Per-agent signing
identity is now opt-in at session registration time. The host-shared
default is unchanged.

## Why this exists (PL-166)

TermLink's envelope signing is keyed by a single ed25519 key per
`(host, user)` — `$HOME/.termlink/identity.key`, shipped in T-1159
(envelope signing) and T-1436 (fingerprint baked into
`SessionMetadata`). When two or more agents run as the same UID on the
same host (Claude sessions on `.107`, framework-agent + cohort-agent
co-resident on a workstation, etc.) they share that key — every
envelope they sign reports the same `from_fingerprint`. T-1427's
strict-reject path (`-32014 CHANNEL_IDENTITY_MISMATCH`) confirmed the
collision empirically: 19 envelopes labeled "Pen" all signed by
`d1993c2c…`.

That's PL-166 in a sentence: **on a shared host, envelope
`from_fingerprint` identifies the host, not the agent.** Audit trails,
multi-tenant trust, defence-in-depth against per-agent compromise — all
were structurally unanswerable until each agent could present its own
key.

T-1693 inception decided GO on Shape 1 (agent-managed key files, no
new hub infra). T-1700 + T-1701 shipped the wiring.

## CLI surface

```
termlink register --identity-key <PATH>           # PTY-backed session
termlink register --self --identity-key <PATH>    # event-only session
```

`<PATH>` is a full file path. The file is auto-created at `chmod 600`
on first use; subsequent runs load the same key (stable fingerprint
across restarts). When `--identity-key` is unset, behavior is
unchanged — the host default key is used.

## Environment-variable precedence

For non-CLI callers (subprocesses, MCP tools, embedded clients), the
same override is reachable through environment variables. Resolution
order, highest precedence first:

1. **`TERMLINK_IDENTITY_FILE`** — explicit file path (T-1700). `--identity-key` exports this.
2. **`TERMLINK_IDENTITY_DIR`** — base directory; the loader appends `/identity.key` (T-1159).
3. **`$HOME/.termlink/identity.key`** — host default.

Code path: `crates/termlink-session/src/registration.rs::resolve_identity_key_path`.

Both the registration metadata path (T-1436) and the
`channel.post` signing path (T-1427) honor this precedence, so the
wire envelope and the session's advertised `identity_fingerprint`
always agree.

## Per-project secrets convention

Mirroring the existing `instance/secrets/*.key` pattern used by hub
auth, agents on shared hosts SHOULD keep their key under their own
project tree:

```
<project-root>/instance/secrets/<agent-name>_identity.key   # chmod 600
```

Examples:

- `/opt/termlink/instance/secrets/termlink-agent_identity.key`
- `/opt/999-AEF/instance/secrets/framework-agent_identity.key`
- `/opt/penelope/instance/secrets/pen_identity.key`

Naming hint: include the agent's name as the filename prefix so a
`ls instance/secrets/` produces an obvious roster. The basename itself
is free — `Identity::load_or_create_from_file` does not require
`identity.key` (that suffix only applies to the legacy base-dir API).

## Worked example — registering with a per-agent key

```bash
# One-time: pick a path for the agent's key.
KEY=/opt/myproject/instance/secrets/myagent_identity.key

# Register a PTY session bound to this key.
termlink register --shell --name myagent --identity-key "$KEY"

# Or an event-only (--self) session for heartbeats / chat-arc presence.
termlink register --self --name myagent-presence --identity-key "$KEY"
```

The first invocation creates `$KEY` at `chmod 600`. The fingerprint is
printed at startup:

```
Identity (T-1700): /opt/myproject/instance/secrets/myagent_identity.key (13bc46a23bf4d18f)
Session registered: ...
```

Re-running with the same `$KEY` reuses the key (same fingerprint).
Subprocesses spawned by this session inherit `TERMLINK_IDENTITY_FILE`
and so will sign with the same key (this is desired — the agent's
shell-spawned helpers continue to identify as the agent).

## Rotation story

Minimum viable, accepted at T-1693 inception:

1. `mv "$KEY" "$KEY".bak-$(date +%s)` (or delete) — the old envelope
   trail stays verifiable via TOFU pin history.
2. Restart the agent's `termlink register` process; a fresh key
   auto-generates.
3. Peers re-pin the new fingerprint the next time the agent posts.

There is no central registry to update, no hub-side rotation, no
out-of-band coordination required. The cost is that envelopes signed
by the old key remain valid until peers' TOFU stores age them out —
acceptable for the current threat model (agent-key compromise is
contained to that agent only, and revocation is "burn the old key").

## When to keep the host default

You don't need `--identity-key` if:

- Exactly one agent runs as that UID on that host.
- Audit-trail attribution at envelope granularity isn't a constraint
  for your use case.
- The agent is short-lived and only emits to ephemeral topics.

The host default key is fine — it ships everywhere, costs zero setup,
and `termlink whoami` will continue to surface it.

You SHOULD use `--identity-key` if:

- ≥2 agents share a UID on the same host (the common shared-host case).
- You need per-agent audit attribution on a chat-arc-style channel.
- The agent posts decisions / approvals / signed contracts that must
  be traceable to that agent specifically, not the host.

## Verifying the override took effect

Once the session is up, the registration JSON under
`/var/lib/termlink/sessions/<id>.json` (or `/tmp/termlink-$UID/sessions/`
on legacy installs) has the fingerprint baked into
`metadata.identity_fingerprint`. Quick check:

```bash
jq -r '.display_name + " " + .metadata.identity_fingerprint' \
  /var/lib/termlink/sessions/*.json
```

Two sessions registered with distinct `--identity-key` paths must
report distinct fingerprints. If they don't, either `TERMLINK_IDENTITY_FILE`
wasn't picked up (check the env at registration time) or the same key
file was passed to both invocations.

## Related work

- **T-1693** — inception that authorized this work (Shape 1 GO).
- **T-1700** — `--identity-key` on `termlink register` (PTY path).
- **T-1701** — `--self` mode parity for `--identity-key`.
- **T-1704** — `termlink whoami` discoverability hint: prints `↳ shared with N other sessions on this hub` under `Identity FP:` when the local key is the host default. Operators see the PL-166 condition at the place they'd naturally look. JSON callers also get `session.identity_shared_with`.
- **T-1705** — `termlink doctor` CLI identity check: emits `identity` warn line `N sessions share M identity FP [fp×N] — pass --identity-key (T-1700)` when 2+ live sessions share an FP. Surfaces PL-166 from the diagnostic path.
- **T-1706** — MCP `termlink_doctor` parity: same `identity` check shape exposed to LLM agent callers (closes the silent-divergence gap from G-057's pattern).
- **T-1159** — ed25519 keyring foundation.
- **T-1436** — `identity_fingerprint` in `SessionMetadata`.
- **T-1427** — strict-reject path that exposed PL-166.
- **PL-166** — the structural gap closed by this work.
- **G-056** — resolved 2026-05-19 (T-1700 + T-1701 + T-1702 + T-1704 + T-1705 + T-1706 collectively).

Out of scope here (future follow-ups if needed): rotation tooling
(`termlink identity rotate-file`) and a Watchtower view of fleet-wide
agent identity assignments. The `termlink doctor` hint listed here in
earlier revisions shipped as T-1705 + T-1706.
