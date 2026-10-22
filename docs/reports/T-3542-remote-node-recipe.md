# T-3542 — Remote node (node1) fleet-join: findings and recipe options

Scope: read-only investigation of the code at `main` (HEAD `86600a273`). No source edited.
Question: can an unprivileged external node (node1, own ed25519 identity, own hub on
`127.0.0.1:9100`, reachable only to home hub `192.168.10.122:9100` over WireGuard) join the
fleet **without holding the `.122` hub secret**, while still getting its sidecar inbox,
presence heartbeats, DMs, chat-arc posts and fleet visibility, and **without** being able to
inject or exec into other sessions?

Answer in one line: **no, not today.** Every hub-facing client path today requires the raw
32-byte hub secret (or a token minted from it), and the hub has no per-client key, no
scope ceiling per identity, and no per-topic ACL. Details follow.

---

## Findings

### A. Authentication handshake and scope today

1. **TCP connections start with zero scope; Unix same-UID connections start at Execute.**
   `crates/termlink-hub/src/server.rs:1033-1046` — Unix same-UID connections are handed
   `Some(PermissionScope::Execute)` with no auth. `crates/termlink-hub/src/server.rs:1093-1095`
   — TCP connections are handed `None` (unauthenticated) and go through the TLS acceptor.
   The `handle_connection` doc states this at `server.rs:1189-1193`.

2. **Unauthenticated requests are refused except `hub.auth`.**
   `crates/termlink-hub/src/server.rs:1900-1925` — `hub.auth` is always allowed; any other
   method with `granted_scope == None` gets `AUTH_REQUIRED`. So a TCP client sees nothing,
   not even `hub.version`, until it authenticates.

3. **The handshake is `hub.auth` with an HMAC-signed bearer token.**
   `crates/termlink-hub/src/server.rs:851-900` (`handle_hub_auth_token`) reads
   `params.token`, validates it with `auth::validate_token(&secret, token, None)`, and on
   success sets `granted_scope` for the rest of the connection. The hub's `token_secret` is
   the 32-byte `hub.secret` (`server.rs:492-495`, `generate_and_write_hub_secret` at
   `server.rs:162`). Re-auth is possible on the same connection (test at `server.rs:3453`).

4. **Token format:** `base64url(payload).base64url(HMAC-SHA256(secret, payload_b64))`.
   `crates/termlink-session/src/auth.rs:421-446` (`create_token`). Payload is
   `{scope, session_id?, issued_at, expires_at, nonce}` (`auth.rs:231-245`).
   `validate_token` checks the MAC (`auth.rs:488`), expiry (`auth.rs:503`) and optional
   `session_id` (`auth.rs:467-520`). **There is no nonce store**: the nonce is random but
   never checked, so a captured token is replayable until expiry.

5. **TTL ceiling is 7 days by default**, overridable by `TERMLINK_MAX_TOKEN_TTL_SECS`
   (`auth.rs:386`, `auth.rs:~392-398`). Tokens carry no identity: there is no claim binding a
   token to a sender fingerprint or pubkey.

6. **Scope enum and hierarchy:** `Observe < Notify < Interact < Control < Execute`
   (`crates/termlink-session/src/auth.rs:129-152`). `Notify` (T-3533) sits below `Interact`
   and is the sidecar doorbell's fixed action only (`auth.rs:177-212` maps
   `command.notify → Notify`).

7. **Per-method scope at the hub** (`crates/termlink-hub/src/server.rs:752-810`,
   `hub_method_scope`), falling through to `auth::method_scope` (`auth.rs:177-212`) for
   forwarded methods:
   - **Observe:** `session.discover`, `session.whoami`, `event.collect`, `hub.version`,
     `hub.bus_state`, `hub.governor_status`, `channel.list`, `channel.subscribe`,
     `channel.receipts`, `channel.claims`, `channel.claims_summary`, `channel.cv_keys`,
     `dispatch.get`, `agent.find_idle` (`server.rs:759-776`).
   - **Interact:** `channel.post`, `channel.claim`, `channel.renew`, `channel.release`,
     `dispatch.ask/answer/ack`, `event.emit_to`, `event.broadcast`, `session.register_remote`,
     `session.heartbeat`, `session.deregister_remote` (`server.rs:778-791`).
   - **Control:** `channel.create`, `channel.set_retention`, `channel.transfer_claim`,
     `channel.force_release`, `channel.trim`, `channel.sweep` (`server.rs:792-801`).
   - **Execute:** `channel.delete` (`server.rs:807`), and any unknown method (deny-by-default,
     `auth.rs:210-211`).
   - Forwarded session methods: `command.inject` and `command.signal` are **Control**,
     `command.execute` is **Execute** (`auth.rs:177-212`). An Interact-scoped token therefore
     cannot inject or exec **at the hub** — the rejection is at
     `server.rs:1908-1935`, which is the enforcing check.

8. **Client-side: the CLI mints a hub-level token from the raw secret.**
   `crates/termlink-cli/src/commands/remote.rs:760-785` reads the profile's secret, calls
   `auth::create_token(&secret, perm_scope, "", 3600)` for any scope, then connects over TCP and
   sends `hub.auth`. Whoever holds `hub.secret` can mint **any** scope (including Execute) for
   any identity. `channel.rs:519-~560` (`mint_tcp_hub_token`) does the same for `channel post`
   and the WS push path (`channel.rs:623-630`).

9. **The sidecar daemon's own hub RPC is unauthenticated, so it cannot speak TCP today.**
   `crates/termlink-session/src/client.rs:306-316` (`rpc_call_addr_with_timeout`) connects and
   calls the method with **no `hub.auth` step**. The sidecar's send and pull paths use this
   (`crates/termlink-sidecar/src/send.rs:363-367`, `pull.rs:235-237`; `check_pin` calls
   `hub.version`, `send.rs:369-372`). Against a TCP hub every such call gets `AUTH_REQUIRED`
   (`server.rs:1910-1925`). The sidecar is therefore effectively Unix-only, and only same-UID
   local use works (`crates/termlink-sidecar/src/daemon.rs:29`, `hub: TransportAddr`).

### B. Session-level vs hub-level tokens

10. **Session-level tokens** are minted by `termlink token create <session>` from that
    session's registration `token_secret` (`crates/termlink-cli/src/commands/token.rs:5-7`,
    `token.rs` lines 14-25; registration field at `crates/termlink-session/src/registration.rs:108`).
    The session server only accepts them on its own socket (`crates/termlink-session/src/server.rs:22-51`,
    `handle_auth_token` at `server.rs:109-120`). Default scope with a `token_secret` is
    **Observe** until `auth.token` upgrades it (`server.rs:250`). This is the mechanism that
    lets the sidecar `command.notify` doorbell carry a Notify-only token.

11. **Hub-level tokens** are the same HMAC scheme keyed by `hub.secret`, minted by any holder of
    that secret (finding 8). **There is no hub RPC that mints or delegates a token.** The only
    issuers are the CLI (`remote.rs:771`) and MCP (`crates/termlink-mcp/src/tools.rs:7805`, which
    reads `token_secret` from a session registration or a profile). No per-client key, no
    scope ceiling per key, no revocation list. Revoking a node's access today means rotating
    `hub.secret` for the whole hub.

12. **Consequence for node1:** the only way for node1 to obtain any authenticated hub
    connection is for the operator to place the `.122` secret (or a token minted from it) on
    node1. That is exactly what the threat model forbids.

### C. Channel posts, signatures, ACLs

13. **Posts are signed by the sender's ed25519 key, and the hub verifies the signature.**
    `crates/termlink-hub/src/channel.rs:867-895` requires `sender_pubkey_hex` and
    `signature_hex`, verifies with `verifying_key.verify(...)` (`channel.rs:~913-916`), and
    **binds `sender_id` to the pubkey fingerprint** (`channel.rs:925-932`). So node1 cannot post
    as another agent's identity. Its own key works (node1 has an ed25519 identity).

14. **Authorization is scope only. There is no per-topic ACL.** `handle_channel_post_with_peer`
    (`channel.rs:829`) and `handle_channel_subscribe_with` (`channel.rs:1433`) check no
    membership, owner, or allowlist. Any connection with **Interact** can post to **any** topic
    (including another agent's `inbox:*` or `dm:*` topic); any connection with **Observe** can
    **read every topic** via `channel.subscribe` (`server.rs:764-770` — `CHANNEL_SUBSCRIBE` is
    Observe). A grep for ACL/allowlist/membership in the channel module finds only unrelated
    matches (`channel.rs:441`, `channel.rs:458`, a test-data fixture). The `dm:` wake logic
    (`channel.rs:1146-1160`) picks an addressee from the topic name but does not gate access
    to it.

15. **Scope granularity vs the requirement.** A node that holds an Interact token can read
    (Observe is implied) and post, but also write into any other agent's inbox topic. Confidentiality
    of other agents' inboxes is therefore **not** enforced by scope. That is a gap for node1
    even if a scoped token existed.

### D. Federation

16. **There is no hub-to-hub federation.** Charter non-goal #1, enforced by a tripwire test:
    `crates/termlink-hub/tests/no_federation_tripwire.rs:1-40` asserts the hub never learns peer
    hub addresses, never builds a hub-speaking client, and that every outbound connect in hub code
    targets a session registration, not a hub. Cross-hub visibility is **client-driven
    cross-posting** (`channel post --hub <addr>`, `channel.rs:519`/`623`), as stated in CLAUDE.md
    "Channel Topic Semantics".

17. **Consequence:** node1's hub and `.122` do not replicate. A node1 agent's presence or inbox is
    visible from `.122` only when some client on the `.122` side explicitly reads node1's hub,
    or when a client on node1 explicitly posts to `.122`. Both require the respective secret
    on the client side (findings 8 and 9).

### E. Sidecar inbox, presence, and the T-3462 / T-3470 rails

18. **Sidecar transport to the hub is the CLI with `--hub`.**
    `.agentic-framework/lib/sidecar/termlink_transport.py:8` and `:121` build
    `termlink channel post <topic> [--hub <addr>] --client-msg-id <id>`. A remote `--hub` therefore
    goes through `mint_tcp_hub_token` (finding 8), which needs the profile secret. Ring20-side
    hubs.toml profile + TOFU is the mechanism that carries it
    (`termlink_transport.py:30-32`).

19. **Inbox topics the reader must consult (T-3462, one definition):**
    `.agentic-framework/lib/sidecar/inbox.py:88-108` (`read_topics`) returns
    `[inbox_topic] + v9_topics + legacy_topics`. `inbox_topic` is `inbox:<hubid>/<agent>`
    (`crates/termlink-sidecar/src/send.rs:207`). A node1 reader must subscribe to all three
    classes or the sweep misses replies (`inbox.py:88-100`, measured 2026-09-25).

20. **Keep-until-stored sender (T-3462 / retry ladder):** an un-posted (STORED) message is
    reposted on every rung; a posted-but-unread message is nudged then escalated
    (`.agentic-framework/lib/sidecar/retry.py:20-32`). The outbox row is kept until an ack
    state changes; expiry only flips STORED → UNKNOWN (`outbox.py:17-33`, `outbox.py:218-231`).
    node1 must run this loop locally, with the outbox on node1's disk.

21. **Receiver dedupe:** `inbox.py:~395-420` keys duplicates on `client_msg_id` plus a payload
    hash, because the hub's dedupe TTL is about 5 minutes (`inbox.py:6-16`). A node1 receiver
    must keep that `seen` set; it is per-project state under `.context/sidecar/`.

22. **Receiver bind is loopback by default.** `.agentic-framework/lib/sidecar/http_server.py:137`
    defaults `host="127.0.0.1"`. A remote peer's direct send (`direct.py:148-152`, Bearer token from
    the peer's `token_file`, `lifecycle.py:45-46`, `lifecycle.py:110-116`) therefore needs an
    explicit bind to the WireGuard address, and the token must be exchanged out of band.

23. **Presence heartbeats** use `channel.post` on `agent-presence` with `metadata.cv_key`
    (CLAUDE.md, `/be-reachable`, `scripts/listener-heartbeat.sh`). That is an Interact-scope post,
    so it needs a scoped credential or the secret (finding 14, 8).

24. **T-3470 checkout / commit** (`crates/termlink-cli/src/commands/agent_work.rs:1-13`,
    `cmd_agent_checkout` at `agent_work.rs:732`, `cmd_agent_commit` at `:776`):
    - Ownership is `channel.renew` (`agent_work.rs:249-263`), which is **Interact** at the hub
      (`server.rs:779-783`).
    - Git runs locally under `TERMLINK_WORK_DIR` (default `/var/lib/termlink/work`,
      `agent_work.rs:101-106`). The clone source is the `repo` argument (`agent_work.rs:344`,
      `:483`), so node1 needs read access to the repo, over WG or via a mirror.
    - Each verb posts an event to `agent-work:<topic>` (`agent_work.rs:40`, `:253`). That is also
      Interact.
    So node1 needs an Interact-capable hub credential for checkout and commit, in addition to the
    git source access.

25. **Own identity vs host identity.** The sidecar inbox output run during this investigation
    reports the project "has no TermLink identity of its own, so it is signing as the host-shared
    one" (`fw sidecar inbox` output, dm rails keyed by `d1993c2c3ec44c94`). node1 has its own key,
    which is the correct shape; the same per-project identity is needed on the home side for the
    dm rail to be private.

26. **Inject / exec / session routing are not reachable at Interact.** `command.inject` and
    `command.signal` require Control (`auth.rs:177-212`, enforced at `server.rs:1908-1935`);
    `command.execute` requires Execute. `event.emit_to` and `event.broadcast` are Interact
    (`server.rs:778-791`). **Not verified:** whether `event.emit_to` can deliver into a session's
    event queue in a way that reaches an agent prompt. That needs a read of the event-consumer path
    before relying on it as a boundary.

---

## Recipe options

### Option 1 — Scoped hub credential minted by the hub for node1 (needs BUILD)

What it needs, and what today's code does:

- **Today:** not possible without giving node1 the secret. Findings 8, 11, 12.
- **Build (a):** a per-client credential. Either an ed25519 key registered on the hub with a scope
  ceiling (the hub verifies a signed request, not an HMAC over the shared secret), or a hub RPC
  `hub.issue_token` gated by Control that returns a token with a short TTL and a scope cap.
  Either change touches `auth.rs` (new token kind, `validate_token`), `server.rs:851-900`
  (`handle_hub_auth_token`), and `hubs.toml` (per-profile key).
- **Build (b):** a per-topic ACL, because scope alone cannot keep node1 out of other agents'
  inboxes (findings 14, 15). Needed: topic prefix → allowed identities, checked in
  `channel.rs` post and subscribe.
- **Build (c):** nonce or `jti` tracking, to close the replay gap (finding 4).
- **Build (d):** `hub.auth` over TCP must be usable by the sidecar daemon (finding 9). The
  `rpc_call_addr_with_timeout` path needs the auth step, or the sidecar needs to shell out to the
  CLI.
- **Gaps that remain even after the build:** the Observe scope reads every topic (finding 14), so
  a node1 presence heartbeat is visible to everyone who can read `agent-presence`. That is the
  current fleet model, and it is acceptable only if the ACL build decides so explicitly.
- Injection safety: **already holds** at the hub for Interact tokens (finding 26). The build does
  not weaken it; the risk is the replay and mint gaps above.

### Option 2 — `.122` dials node1's hub, node1 holds only its own secret (partly exists, needs firewall + profile work)

What it needs:

- node1 runs its hub on the WG tunnel address (`crates/termlink-hub/src/server.rs:492-495`, hub bound
  by `--tcp`). node1 holds node1's own `hub.secret` only. Nothing from `.122` lands on node1.
- `.122` gets a `hubs.toml` profile for node1: `address`, `secret_file` (node1's secret, out of band),
  and a TOFU pin (`crates/termlink-cli/src/config.rs:7-8`, `:69-86`; the hub-probe and TOFU verbs in
  CLAUDE.md). Then `.122`-side clients post and read node1's topics with `channel post --hub <node1>`
  (`channel.rs:519`, `:623`) — explicit cross-posting, consistent with finding 16.
- The network must allow `.122 → node1:9100`. Today it is one-way (per the brief: node1 reaches `.122`,
  `.122` reaches node1 only from ring20/operator addresses). That is a firewall/WireGuard policy change, not code.
- Inverse trust: `.122` now holds a credential that can act on node1's hub (Execute, for an
  operator-minted token). This is the reverse of the stated threat model, and it is acceptable only if
  the operator decides ring20 trusts node1 less than home. Decision, not a code gap.
- node1-side sidecar still needs the TCP `hub.auth` fix in finding 9 **only if** the sidecar daemon
  talks to a remote hub. Under this option the sidecar talks to node1's **local** hub over Unix, which
  works today (finding 1). So this option avoids the sidecar TCP gap.
- Injection: node1's own agents can inject into node1 sessions (local, same-UID). `.122` cannot inject
  into node1 unless it holds Control, which the operator would mint explicitly.

### Option 3 — node1 is a direct-path receiver, no hub for messaging (exists today, partial)

What it needs:

- Consults and replies travel the sidecar **direct** rail: the receiver HTTP API with a per-agent
  Bearer token (`direct.py:148-171`, `http_server.py:137`). No hub secret touches node1 for this path.
- node1's receiver must bind the WG address, not `127.0.0.1` (finding 22). The token is exchanged
  out of band (`lifecycle.py:110-116`, `direct.py:166-171`), so the operator copies it, not a hub
  secret.
- Presence, DMs, chat-arc and fleet visibility still need a hub. This option covers the consult path
  only. It fails the stated requirement that node1 post presence and read `dm:*`, which needs a
  scoped hub credential (Option 1) or the secret (Option 2).
- Gaps: direct rail is a point-to-point HTTP path, so it does not give fleet visibility. The
  hub-rail fallback (`termlink channel post --hub`) still needs a credential on node1.

---

## Recommendation

1. **Do not give node1 the `.122` secret.** Nothing in the code can scope it today, so the secret
   would be full Execute on `.122`.
2. **Take Option 2 for the fleet join now**, with no code build: node1 holds only its own
   `hub.secret`, `.122` holds a `hubs.toml` profile for node1 (secret + TOFU pin), and the operator
   opens `.122 → node1:9100` on WireGuard. This keeps node1 unable to reach `.122` at all, which
   is the strict form of the threat model. The cost is the inverse trust (finding under Option 2),
   which is a decision for ring20 and the operator.
3. **Use Option 3 for consults** if node1 cannot accept the inverse trust, since it needs no hub
   credential.
4. **Build Option 1 only if node1 must post to `.122` without `.122` holding node1's secret.** It is
   the largest piece of work: a per-client key or a delegating mint RPC, a per-topic ACL, nonce
   tracking, and a TCP `hub.auth` step in the sidecar daemon.
5. **Required before any of the three:** the sidecar daemon's remote path (finding 9) matters only
   for Option 1; under Options 2 and 3 it is out of scope. The sidecar's hub transport is the CLI
   (`termlink_transport.py:8`), so the CLI credential path, not the daemon, is what node1 needs.

**Most important gap:** there is no per-client credential and no per-topic ACL. Scope is the only
access control, and the only way to get any scope is the raw hub secret (findings 8, 11, 14). Every
recipe either hands node1 that secret or relies on the inverse trust of Option 2.

**Not verified in this investigation:** whether `event.emit_to` (Interact) reaches an agent prompt
(finding 26); the full Observe read surface of `channel.subscribe` beyond the code reviewed; and
the WireGuard firewall state, which is outside this repository.
