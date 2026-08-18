# T-2789 — RCA: cross-UID TermLink delivery from 0503-Codex to the root-owned AEF hub

**Status:** inception / exploration
**Requested by:** operator, on behalf of the 0503-Codex agent (`dimitri-mint-dev`, `/opt/0503-codex-cli-playground`)
**Date:** 2026-08-18
**Constraint from the request:** no world-writable sockets, no broad `chmod`/`chown`, no
copied secrets, no unaudited bypass, no interactive `sudo` for normal governed
collaboration, and delivery must remain distinct from receipt.

---

## Headline

**TermLink as it stands today cannot satisfy the stated constraints.** Every path
that gets the Codex agent talking to the root-owned hub grants it *full `Execute`
authority* over that hub — including the two options the request lists as candidates
(socket ACLs, shared secret). The four-level capability model that looks like the
answer is **client-asserted, not server-granted**, so it provides no privilege
separation between principals who do not already trust each other.

That is an uncomfortable answer, and it is the reason this report does not open with
a recipe. A recipe would have worked, looked least-privilege, and quietly handed a
second UID root-equivalent authority over the AEF hub.

The near-term fix that *does* satisfy the constraints is a **broker** (Option D
below) — because it moves the secret to a process the Codex agent cannot read,
rather than trying to constrain a client that holds it.

---

## 1. Evidence-backed RCA

### 1.1 Why the user-side call fails

Measured on this host:

```
/var/lib/termlink            root:root 700
/var/lib/termlink/sessions   root:root 700
/var/lib/termlink/hub.sock   root:root 755
/var/lib/termlink/hub.secret root:root 600
```

Two independent barriers, either sufficient on its own:

1. **Directory traversal.** `/var/lib/termlink` is `0700 root:root`. A non-root
   process cannot traverse it, so it cannot reach `hub.sock` *or* enumerate
   `sessions/*.json`. This is what produces `session not found`.
2. **Socket connect permission.** `hub.sock` is `0755`. Connecting to a UNIX socket
   requires **write** permission on the socket inode; `0755` grants group/other
   `r-x` only. Even with the directory opened up, `connect()` would still return
   `EACCES`.

The symptom "session not found / no hub available" is therefore a **filesystem
permission** outcome, not a discovery-logic outcome.

### 1.2 Discovery is *not* the cause — with one exception

`crates/termlink-session/src/discovery.rs:41-58` — `all_runtime_dirs()` already
includes `/var/lib/termlink` as a well-known persistent location, so a user-side
`termlink list` *would* look there.

The exception is `discovery.rs:44-47`:

```rust
// If TERMLINK_RUNTIME_DIR is explicitly set, it's an exclusive override —
// the caller wants exactly this dir (tests, systemd units, manual config).
// Multi-dir scanning only kicks in for the default resolution path.
if std::env::var("TERMLINK_RUNTIME_DIR").is_ok() {
    return vec![runtime_dir()];
}
```

The request reports Gemini running with `TERMLINK_RUNTIME_DIR=/home/dimitri-mint-dev/.termlink`.
If the Codex session inherits that variable, multi-dir scanning is **disabled
entirely** and `/var/lib/termlink` is never consulted — a second, independent
reason for `session not found`, with a different fix. Worth confirming which of the
two is active before anything is changed; they are indistinguishable from the error
message alone.

### 1.3 Why `/var/lib/termlink` is root-owned in the first place

This is deliberate and should not be undone. `CLAUDE.md` §"Special case — volatile
runtime_dir (T-1290 / T-1294)" records that a hub on `/tmp` loses its HMAC secret and
TLS cert on every reboot (tmpfs, or `systemd-tmpfiles` `D /tmp` rule), rotating both
credentials and breaking every pinned client — concern **PL-021**, gap **G-058**. The
migration to `/var/lib/termlink` under a systemd unit (T-935) is the fix. The root
ownership follows from the unit running as root, not from a security decision about
peers.

### 1.4 The load-bearing finding: filesystem access *is* full authority

`crates/termlink-hub/src/server.rs:813` and `:873` — two comments, three lines apart
in the accept loop:

```rust
// Unix same-UID connections get full access (no auth needed)
...
handle_connection(stream, Some(PermissionScope::Execute), ...)
```

```rust
// TCP connections start with zero scope (unauthenticated)
```

So the hub's authorisation model is:

| Transport | Initial scope | Upgrade path |
|---|---|---|
| UNIX socket | `Execute` (maximum), unconditional | n/a — already maximal |
| TCP + TLS | *none* | `hub.auth` with an HMAC-signed capability token |

**This refutes candidate option "a dedicated TermLink group plus narrow socket ACLs"
outright.** An ACL can control *who* opens the socket. It cannot constrain *what
they may then do*, because the scope is a hardcoded constant at accept time. Adding
`dimitri-mint-dev` to a `termlink` group and setting the socket `0770` grants that
UID `Execute` on the AEF hub — the ability to run `command.execute` against any
registered session, delete topics, and mint anything. It is not a narrow grant; it
is the widest grant the system has, delivered through a mechanism that looks narrow.

### 1.5 The scope model is client-asserted, not server-granted

The capability model itself is real and well-built —
`crates/termlink-session/src/auth.rs:270-346`: a hierarchical
`Observe < Interact < Control < Execute`, a per-method mapping (`method_scope`),
signed tokens with `expires_at`, and a TTL ceiling of 7 days clamped at the choke
point (`auth.rs:415-464`, T-2449). The hub verifies them at
`server.rs:644-712` and a regression test
(`router.rs:2809 tcp_forward_rejected_when_scope_insufficient`) proves forwarded
calls are not a scope bypass.

But look at where tokens are *minted* in production code:

```
crates/termlink-cli/src/commands/remote.rs:770    create_token(&secret, perm_scope,           "", 3600)
crates/termlink-cli/src/commands/remote.rs:6417   create_token(&secret, perm_scope,           "", 3600)
crates/termlink-mcp/src/tools.rs:7770             create_token(&secret, perm_scope,           "", 3600)
crates/termlink-cli/src/commands/channel.rs:428   create_token(&secret, PermissionScope::Execute, "", 3600)
crates/termlink-cli/src/commands/channel.rs:472   create_token(&secret, PermissionScope::Execute, "", 3600)
```

Every one of these runs **on the client**, signing with the client's own copy of
`hub.secret`, choosing its own `perm_scope`. Two of them hardcode `Execute`.

The consequences are the whole answer:

- **There is no server-side issuance.** No principal asks the hub for a credential;
  each client manufactures its own.
- **Scope is a self-declaration.** A client holding `hub.secret` mints `Execute`
  whenever it likes — `channel.rs:428` does so unconditionally. The hierarchy
  therefore structures what a *cooperating* client requests; it is not a boundary
  against one that does not cooperate.
- **Possession of `hub.secret` ≡ `Execute`.** So "give the Codex agent a scoped
  token instead of the secret" is not available: tokens live 3600s with no refresh
  path other than re-minting from the secret, so any durable grant is a secret grant.
- **There is no principal identity and no revocation.** Tokens carry
  `{scope, session_id, issued_at, expires_at, nonce}` — no subject. Two holders of
  the secret are indistinguishable in the hub's logs, and withdrawing access means
  rotating `hub.secret` and re-pinning *every* client.

`token.rs:5-28` (`termlink token create`) does not close this: it mints from a
**session's** `reg.token_secret` — a different secret, requiring the session to have
been spawned `--token-secret` — not from the hub's. So the hub has a verifier with no
matching operator-facing minter. That is the inverse of the T-2699 shape this project
already documented (a builder with zero callers): here it is a **verifier with no
issuer**, and it reads as a working least-privilege system precisely because the
verify half is complete and tested.

### 1.6 The boundary being defended does not exist at the OS level

Measured live:

```
$ id dimitri-mint-dev
uid=1000(dimitri-mint-dev) gid=1000(dimitri-mint-dev)
groups=1000(dimitri-mint-dev),0(root),4(adm),20(dialout),21(fax),24(cdrom),
       25(floppy),26(tape),27(sudo),29(audio),30(dip),44(video),46(plugdev),
       100(users),105(lpadmin),113(netdev),118(nopasswdlogin),121(scanner),
       124(sambashare),128(vboxusers),986(docker),985(ollama),984(nordvpn)

$ sudo -u dimitri-mint-dev ls /var/lib/termlink/
ls: cannot open directory '/var/lib/termlink/': Permission denied

$ sudo -u dimitri-mint-dev test -w /var/lib/termlink/hub.sock   →  rc=1
```

The first two lines confirm §1.1 empirically — both barriers are live and either
alone is decisive. Group `0(root)` membership does **not** help, because `0700`
grants owner-only bits.

But the same output shows `dimitri-mint-dev` is in **`sudo`** and in **`docker`**.
Docker-group membership is root-equivalent by construction (`docker run -v /:/host`
mounts the host filesystem into a container the user controls), and it needs no
password. So the account this exercise is trying to constrain **already holds
root-equivalent authority by two independent routes**, one of them non-interactive.

This does not make the request moot, and it should not be read as "so just grant
it". The meaningful distinction is between **ambient** authority and **available**
authority:

- *Available* — the human operator can become root deliberately.
- *Ambient* — an agent process running as that UID exercises the authority
  automatically, with no human in the loop, as a side effect of a tool call.

An autonomous agent is exactly the case where that distinction bites: a compromised
or merely confused agent uses whatever is ambient, and never uses what requires a
human decision. Least privilege for the *process* therefore remains worth building
even when the *account* is privileged — which is also why the `sudo -n` password
prompt the request describes as an obstacle is doing useful work and should not be
removed with a NOPASSWD rule.

It does change one thing: **the broker's security value is containment of agent
behaviour, not defence against a hostile UID.** Anyone designing or reviewing
Option D should size the effort against that honest goal rather than against a
boundary the host does not actually enforce. If a hard boundary is wanted, that is a
host-hardening question (drop `docker` and `sudo` from the agent's account, or run
the agent under a dedicated unprivileged UID) and it is prior to anything TermLink
can offer.

### 1.7 Delivery vs receipt — already correct, already named

The request's insistence that "a successful post is not proof the target read it" is
already this project's recorded position. **PL-247**: *"Silent-delivery is the comms
failure class: termlink guarantees the WRITE (durable) but not the READ."*

The primitives exist and need no new work:

- `channel post --await-ack` (T-2286) writes a durable obligation row to
  `~/.termlink/awaiting_ack.sqlite`.
- `channel awaiting-ack` (T-2287) surfaces every send still unconfirmed, including
  rows retained after their retry loop was exhausted.
- The unconfirmed-delivery canary (T-2295) fires when a row is outstanding past a
  threshold.

So obligation #3 ("receive/verify a substantive response or explicit receipt") is
satisfiable today by the durable-messaging path, *independently* of how the transport
question is resolved.

---

## 2. Fix options and trade-offs

| | Option | Authority actually granted | Meets constraints? |
|---|---|---|---|
| **A** | `termlink` group + socket `0770`, dir `0750` | **Full `Execute`** (`server.rs:813`) | **No** — the request forbids broad permission weakening; this is maximal authority wearing a narrow-looking mechanism |
| **B** | Copy `hub.secret` to `dimitri-mint-dev` | **Full `Execute`** (client mints own scope, §1.5) | **No** — explicitly forbidden ("no copied secrets"), and equivalent in power to A |
| **C** | Build server-side token issuance | Exactly the granted scope | **Yes, eventually** — but it is a trust-model change, not a patch (see below) |
| **D** | Root-owned broker with an allowlisted RPC surface | Exactly the allowlisted methods | **Yes, today** |

**A and B are the same grant.** That is worth stating plainly, because they *look*
different: one is a permission change and one is a credential change, and both are
routinely described as "narrow". Under `server.rs:813` and §1.5 they confer the
identical capability — everything.

### Option C — server-side issuance (the correct long-term fix)

Make the hub the issuer: a principal presents an identity, the hub mints a token
bound to `{subject, scope, ttl}`, and the client can no longer self-elevate. Concretely
that requires (i) an issuance surface, (ii) a subject field in `TokenPayload` and a
principal registry, (iii) revocation, and — the hard part — (iv) **removing the
client's ability to mint**, which today follows from every remote client holding
`hub.secret`. Step (iv) is a breaking change to how every hub client authenticates,
so this is an arc, not a task. It is the right destination and should be filed as one.

### Option D — broker (recommended now)

A small root-owned helper holds `hub.secret` and exposes a *second* UNIX socket,
group-readable by a dedicated group containing `dimitri-mint-dev`. It forwards only
an allowlisted set of methods to the hub and refuses everything else.

Why this satisfies the constraints when A does not: the authority ceiling is set by
**the broker's allowlist**, not by socket permissions. The Codex agent never holds
`hub.secret`, so it cannot mint `Execute` and go around the broker. The broker is
also the natural place to enforce the two guarantees TermLink structurally cannot
provide today — a per-principal audit identity, and revocation (remove from group).

Cost is honest: it is new code, it is a new trust boundary, and an allowlist that
drifts toward permissive is a real failure mode. It should ship with the
allowlist under version control and a check that fires when it grows.

### Not recommended: a shared per-user hub

Running a second hub as `dimitri-mint-dev` does not solve the problem — it relocates
it. G-060 records that topics are **per-hub state with no federation primitive**, so
cross-posting between the two hubs is client-driven and needs credentials on both
ends. The authorisation question returns unchanged.

---

## 3. What each proof obligation actually rests on (IW-4)

| # | Obligation | Owner | Status today |
|---|---|---|---|
| 1 | Unauthorised user remains refused | TermLink (`server.rs:873`, TCP zero-scope) | **Holds** for TCP; for the UNIX socket the refusal is filesystem permissions alone |
| 2 | Authorised path can preflight + deliver | TermLink | **Blocked** — no grant exists that is narrower than full `Execute` |
| 3 | Receipt observable, distinct from delivery | TermLink (`--await-ack` / `awaiting-ack`, T-2286/T-2287) | **Holds** — usable now |
| 4 | No secret value in logs/messages | Operator discipline + R3/G-011 | Holds by convention; not structurally enforced |
| 5 | Wrong-project/wrong-worktree target refused | **AEF, not TermLink** — the T-559 project-boundary hook | Holds in AEF sessions; a raw `termlink` call is not subject to it |

Obligation 5 deserves emphasis: it is enforced by the *framework* wrapping the agent,
not by the transport. It fired during this very investigation — `ls /opt/` was refused
with *"Bash commands must operate within the current project"*, and its printed remedy
was `fw termlink dispatch --project /opt/other`, i.e. run the work inside the target
project's own governed session. Any transport built here inherits that only if the
calling agent runs under AEF governance; a bare binary invocation does not.

---

## 4. Recommendation

1. **Do not grant socket or secret access.** Both are full `Execute` (§1.4, §1.5).
   This is the operative decision and it is available immediately.
2. **Unblock the actual need now via the durable-messaging path**, which requires no
   new authority to design against: DM topic + `--await-ack` + `awaiting-ack` gives
   bounded handoff *and* the delivery/receipt split the request asks for. It still
   needs hub credentials, so it does not bypass §1.5 — but it means the transport
   decision is the only open question, not the messaging semantics.
3. **Build Option D (broker)** as the governed path, with the allowlist version-controlled.
4. **File Option C as an arc** — server-issued, subject-bound, revocable tokens. Record
   §1.5 as a security finding in its own right: the capability model reads as a
   privilege boundary and is not one.

---

## 5. Open questions carried into the decision

- **IW-1 — which layer refuses?** *Answered:* filesystem permission (§1.1), with a
  possible second cause if `TERMLINK_RUNTIME_DIR` is inherited (§1.2). Confirm which
  before changing anything.
- **IW-2 — is there a capability-scoped credential?** *Answered, and the answer is
  the finding:* the type exists and is verified, but it is client-minted from the
  master secret, so it is not a boundary between distrusting principals (§1.5).
- **IW-3 — socket or TCP?** *Answered:* TCP. It is the only plane that starts
  unauthenticated. But it is not sufficient today, because the credential it accepts
  is derived from a secret the client must hold.
- **IW-4 — who owns each obligation?** *Answered:* §3. Obligation 5 is AEF's, not
  TermLink's, which matters for anything built here.

## 6. Human decisions required

1. **Host hardening — prior to everything else (§1.6).** Decide whether the Codex
   agent should keep running under a UID that is in `sudo` and `docker`. If a real
   boundary is wanted, run the agent as a dedicated unprivileged user. No TermLink
   change can substitute for this, and Option D's value depends on which answer you
   give: with hardening it is a security boundary, without it, containment of agent
   behaviour only.
2. Approve or reject **Option D** (broker) as the governed transport.
3. Approve filing **Option C** as an arc, with §1.5 recorded as a security finding in
   its own right — the capability model reads as a privilege boundary and is not one.
4. Confirm whether the Codex session has `TERMLINK_RUNTIME_DIR` set (§1.2) — one
   command on that host, and it distinguishes two different root causes:
   ```
   sudo -u dimitri-mint-dev env | grep TERMLINK_RUNTIME_DIR
   ```
5. **Do not** add a NOPASSWD sudo rule to work around the password prompt (§1.6) —
   that prompt is the only thing currently keeping the agent's root access
   non-ambient.

## 7. What was not done, and why

- **No live cross-UID delivery proof.** The request asks for one (deliverable 5).
  It was not produced because every available means of producing it — socket ACL,
  secret copy — is one of the options this report recommends *against*, and running
  it would have granted exactly the authority under review. A proof is available
  once Option D exists, and should be written against the broker.
- **Nothing was changed.** No permission, no group, no secret, no config. The two
  live checks in §1.6 are read-only.
- **No push.** Per the operator's instruction, local evidence only.
