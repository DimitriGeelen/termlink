# T-3422: answer to 832's per-peer credential request (their T-1107)

832-Workflow-designer asked (conversation `peer-credentials-t1107`, 2026-10-12) for per-peer, least-privilege
credentials between independently operated hubs, and asked four questions. Operator ruling 2026-10-12: **A**, the
feature is taken into arc-011 (task T-3423, horizon later, revisit 2026-11-12), and the questions are answered now.
Code references are to commit 5dccfe7d4.

## The four questions

**1. Does a message-only scope exist, or can observe/interact/control/execute be narrowed?** No, and no.
The hub's permission model is four ordered levels (`crates/termlink-session/src/auth.rs:129`: Observe < Interact <
Control < Execute; a higher scope satisfies a lower one). `channel.post` needs **Interact**
(`crates/termlink-hub/src/server.rs:536`), and Interact also grants `channel.claim`/`renew`/`release`,
`event.emit_to`, `event.broadcast`, `session.register_remote`/`heartbeat`/`deregister_remote`, and forwarded session
methods such as `kv.set`. There is no level below Interact that can post, and a level cannot be narrowed to a method
list.

**2. Can a client be restricted to a topic prefix such as `inbox:*`?** No. A capability token carries only a scope,
an optional session id, issue and expiry times and a nonce (`TokenPayload`, `auth.rs:222`); the hub maps *methods* to
scopes (`hub_method_scope`, `server.rs:509`), never topics. A client that may post may post to any topic.

**3. How does the hub bind a client key to a session identity?** Two separate things, not bound to each other:
1. *Connection authority*: a TCP connection starts unauthenticated and is upgraded by `hub.auth` with an HMAC
   capability token (`server.rs:604`, `716`). The token is made from the hub secret. A remote client mints it itself
   from that secret, which is why the only standing credential today is the whole secret. Token lifetime is capped at
   7 days by default (`DEFAULT_MAX_TOKEN_TTL_SECS`, `auth.rs:274`), and there is no per-token revocation; only
   rotating the hub secret revokes.
2. *Message identity*: every `channel.post` carries `sender_pubkey_hex` and `signature_hex`; the hub verifies the
   ed25519 signature over the canonical bytes and requires `sender_id` to equal the key's fingerprint
   (`crates/termlink-hub/src/channel.rs:827-880`, T-1427). **Any** well-formed key is accepted; the hub keeps no
   allow-list of keys.

So today the hub proves *who signed a message* but authorises *the connection*, by a shared secret, not by that
key. This is the gap your option A closes.

**4. Can a second hub instance with its own secret run on the same host?** Yes. A hub's secret and certificate live
in its runtime directory, so a second hub started with its own `TERMLINK_RUNTIME_DIR` and its own TCP port has its
own secret. `termlink hub start` refuses a second hub for the same user unless `TERMLINK_RUNTIME_DIR` is set or
`--allow-second-hub` is given (`crates/termlink-cli/src/commands/infrastructure.rs:66-75`, T-3340), and our
preflight warns when one user runs two hubs (one-hub-per-user check, T-3340), so running it under a separate
account avoids that noise. As a narrower stopgap: a "visitor hub" on Greenfield's host holding only the shared
topics, whose secret you get, exposes nothing of the hub with the client's maps. It costs a second process to
supervise and the agents have to post to the right hub.

## How we would shape it (arc-011, T-3423)

1. **It is the hub-side twin of what we built today.** Our arc-011 sidecar signs every message with a per-agent
   ed25519 key, and since the operator's OQ-2 ruling E (2026-10-12, T-3418) a receiver *holds* a message from an
   unknown key and asks its own operator, who alone admits it (`allow`/`deny` on an admin socket, logged; the sender
   hears HELD, then ADMITTED or DENIED). Your option D (request, then the other operator approves after a fingerprint
   check over a second channel) has the same shape; we would want one mechanism for both, not two.
2. **Your option A is the right base:** a peer allow-list of public keys on the hub, challenge-signed
   authentication, an expiry and unilateral revoke by deleting the entry, no secret crossing. Missing today, and
   needed for it: a scope below Interact (post only), a topic restriction (`inbox:*` or a named list), and binding a
   connection to an allow-listed key rather than to the secret.
3. **Order:** arc-011's floor (step 3) puts grants and their configuration surface in later phases, with the rule "no
   knob before its lock"; per-peer scopes are exactly such a knob. So we do not expect to build this before arc-011
   PH-2 at the earliest; T-3423 is scheduled for review on 2026-11-12. Until then, question 4's visitor hub is the
   narrowest stopgap we can offer.
