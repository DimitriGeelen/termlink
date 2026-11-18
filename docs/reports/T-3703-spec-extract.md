# T-3703 spec extract — P-15 / B-5: hub-signed revocation list (K-9b)

Read-only research for T-3703 (arc-011 plan unit P-15, build unit B-5). Prepared 2026-11-18. Every passage under
"Spec" and "Cited parameters/invariants" is copied verbatim with `file:line` (paths relative to `docs/design/`);
analysis is only in the last two sections.

## Spec (verbatim, by item)

### 1. P-15 / B-5 (plan rows, gates, ordering, traceability)

`interactive-agent-communication-08-plan.md:158-158`

```text
| P-15 | build | PH-2 | new | B-5: revocation list (hub state, admin revoke, self-revoke, tick pull, stale rule DR-8 A) | R-46, R-63 | E-39, E-40, E-41, E-42, E-43, E-127, E-167 | P-10 (security: signing key before any signed list) | the E-n listed pass at the level 05 states; the B-5 specs pass | fixture suite for the unit exits 0 with a PASS line per listed E-n, and its mutant list is all killed (FP 3.1.b) | revocations stay recorded; pull off returns to PH-1 trust (status says so) |
```

`interactive-agent-communication-08-plan.md:246-249`

```text
5.1 **Hard security orderings, enforced as `security` edges:** signing key before any signed state (P-10 → P-15, P-16);
encryption before any remote put, never plaintext (P-6 → P-22); audit before execute (P-49 → P-52 the account helper,
P-49 → P-56 grants); deny by default before any grant path (P-55 → P-56); grants before the startup chain and before
typed work messages with execution-time authorisation (P-56 → P-67, P-68); break-glass only after the operator key
```

`interactive-agent-communication-08-plan.md:290-290`

```text
    P15["P-15 B-5<br/>B-5  revocation list (hub state, admin"]:::unit
```

`interactive-agent-communication-08-plan.md:416-417`

```text
credential migration P-31 all feed G-2, the T1 release; P-10 feeds P-15 and P-16 by security edges, P-6 feeds P-22 by a
security edge.
```

`interactive-agent-communication-08-plan.md:593-594`

```text
| 18 | P-10 | PH-2 | build | — | B-1 unlocks B-5, B-6 and P-11 (security order: key before signed state) |
| 19 | P-15 | PH-2 | build | P-10 | revocation, then status reads it |
```

`interactive-agent-communication-08-plan.md:598-598`

```text
| 23 | P-24 | PH-2 | build | P-12, P-15, P-23 | status: the operator's view of every later item |
```

`interactive-agent-communication-08-plan.md:620-620`

```text
| 45 | G-2 | — | gate (aggregate) | G-1, P-8, P-10, P-12, P-13, P-14, P-15, P-16, P-17, P-18, P-19, P-20, P-21, P-22, P-24, P-25, P-26, P-31 | operator cuts the T1 release; installed on .107 |
```

`interactive-agent-communication-08-plan.md:622-622`

```text
| 47 | P-37 | PH-2 | test | P-15, P-36 | break-glass drill at W3 scale |
```

`interactive-agent-communication-08-plan.md:875-875`

```text
| R-46 | P-15, P-62 |  |
```

`interactive-agent-communication-08-plan.md:892-892`

```text
| R-63 | P-7, P-15, P-39, P-55, P-56 |  |
```

`interactive-agent-communication-08-plan.md:928-928`

```text
| E-11 | P-7 | E-69 | P-16 | E-127 | P-15, P-33 |
```

`interactive-agent-communication-08-plan.md:956-960`

```text
| E-39 | P-15 | E-97 | P-39 | E-155 | P-33 |
| E-40 | P-15 | E-98 | P-7, P-39 | E-156 | P-1, P-2 |
| E-41 | P-15 | E-99 | P-39 | E-157 | P-1 |
| E-42 | P-15 | E-100 | P-8, P-39 | E-158 | P-1 |
| E-43 | P-15 | E-101 | P-39 | E-159 | P-7 |
```

`interactive-agent-communication-08-plan.md:968-968`

```text
| E-51 | P-17 | E-109 | P-50 | E-167 | P-15 |
```

`interactive-agent-communication-08-plan.md:1026-1026`

```text
| P-15 | PH-2 | T-3703 | new | next |
```

`interactive-agent-communication-08-plan.md:1130-1133`

```text
14.1 **CR-8.1 (step 4).** ARC 22 row B-16 depends on B-14 (status), but the operator's ruling A builds B-16 before the T1
hardening, and B-14 depends on B-2 and B-5 (so on B-1). The plan builds B-16 with the status lines it needs itself
(`activity`, `watchdog`, the macOS `degraded` line) and leaves the rest of B-14 to P-24 (plan decision PD-1). Request:
reword row B-16's dependency to "B-27; the status lines it adds itself; B-14 for the full status".
```

`interactive-agent-communication-04-architecture.md:1546-1546`

```text
| B-5 | P11 | Revocation list: hub state, admin revoke, self-revoke, tick pull, stale rule (6.2) | B-1 | new |
```

`interactive-agent-communication-04-architecture.md:1555-1555`

```text
| B-14 | P18 | Status strength flags, reachability card, CLEAR, skew, GP-4 lines (6.7, V-5, V-8, 8.2) | B-2, B-5 | new |
```

`interactive-agent-communication-04a-traceability.md:29-29`

```text
| SI-11 | K-9b revocation, bound one tick, stale rule (6.2) | PH-2 (RK-1), PH-4/PH-5 rest | B-5 |
```

### 2. Architecture 04: 6.2 P11 (in full), 6.1.4-6.1.7 (signing key, pin), and other K-9b / RK-1 / SI-11 / PN-13 / TH-58 lines

`interactive-agent-communication-04-architecture.md:386-408`

```text
### 6.2 P11 — revocation list (K-9b; RK-1, SI-11, OQ-h A)

6.2.1 **The record [P].** Signed numbered hub state, not a topic (OQ-h): `<runtime_dir>/revocations.json`, canonical
JSON (RFC 8785): `{schema: "termlink.revocation/1", hub_id, seq, issued_at, prev_digest, entries: [{scope: "agent_key",
key: "ed25519:<fpr>", project, agent, reason, revoked_at, by: "operator"|"self", event}], digest, signature}`.
`signature` is by the hub signing key over `termlink.revocation/1\n<digest>` (the domain-separated form of FP
10.3.2.c). `seq` rises by one per change; `prev_digest` chains the versions.
6.2.2 **Who revokes.** (a) The operator: `runme.sh` action → hub root-only local socket `admin revoke --key F --why`.
(b) The agent's own project: a statement signed by the old key (`revocation.request`), accepted by the hub without
the operator (OQ-h). Every revocation: an admin-log row, a needs-attention entry, the list re-signed.
6.2.3 **Pull each tick (PN-3).** The sidecar calls `revocation.head` (returns `seq` and `digest`, about 100 bytes) every
tick and `revocation.get` only when they changed. It verifies the signature against the pinned signing key (6.1.5),
refuses a `seq` lower than the one it holds (rollback, TH-58) and keeps the last verified list in its store.
6.2.4 **A failed pull is not an empty list (SI-11).** Status shows `revocation: verified seq N at T` or `unverified since
T`. After PN-13 (1 h, equal to PN-1) without a verified pull: the interrupt allow-list narrows to the agent's own
project's verified roster, urgent mail from others is downgraded (never dropped), and a needs-attention entry
appears. Messages from keys on the last verified list stay refused.
6.2.5 **Effect.** A message signed by a revoked key is refused at the receiver (and at the hub on post, once K-15's
key card exists) and recorded; the refusal is visible to the sender as a signed notice. Mail to the revoked agent
waits (RK-1, TM 11.1). Bound: the next tick with the hub reachable (SI-11).
6.2.6 Proof: revoke and send within one tick, require refusal; restore an older list, require refusal and the flag;
stop the hub for PN-13, require the narrowing. Mutant: treat a failed pull as an empty list. Build item B-5
(depends on B-1).
```

`interactive-agent-communication-04-architecture.md:371-384`

```text
6.1.4 **Signing key.** An ed25519 key `<runtime_dir>/hub.signing.key` (0600) is created with the id and persisted
like the secret (persist-if-present, T-933 pattern); separate from the TLS certificate (R-61.a). `hub.version`
returns `signing_key: "ed25519:<fingerprint>"`. A certificate regeneration changes neither the id nor the key
(R-61.e). Replacement of the key is C-2a (signed by the old key, policy route) or C-2b (human), which exist from
PH-3; in PH-2 the only path is a runme action that the operator runs, logged.
6.1.5 **Pinning the key in PH-2.** Each sidecar records the hub's signing-key fingerprint in its store on first
verified contact over the local socket (or TLS with the pinned certificate) and refuses a different key afterwards
with a needs-attention entry naming both fingerprints. Out-of-band pinning of card-signing keys is B′ stage 1, PH-3
(R-80); this TOFU pin is its interim, with the same limit as the TLS pin (RR-18).
6.1.6 Failure paths: unreadable `hub.id` or key → refuse to start, needs-attention via the backstop; the client sees
"hub unreachable" (UNKNOWN), never a different id.
6.1.7 Floor: FX-2 (id check stays), SI-25 (the key never printed), enables SI-11. Proof: fixtures for mint, keep,
refuse-on-partial-state, certificate rotation keeps id and key; mutant: derive from fingerprint on each call (the
rotation test goes red). Build item B-1.
```

`interactive-agent-communication-04-architecture.md:191-191`

```text
| K-9b | Revocation list, signed numbered hub state, pulled each tick (P11) | hub + sidecar | TermLink | new, 6.2 |
```

`interactive-agent-communication-04-architecture.md:294-297`

```text
5.3.3 The sidecar talks to its home hub over TLS with signed messages: inbox topics and stage writes, a revocation
pull each tick, ciphertext blobs in the artifact store. The hooks claim and renew the main lease at the hub. The
hub holds its minted id and its signing key, which signs the revocation list; its consumer list, admin log and
sweep refusal sit beside the inbox topics.
```

`interactive-agent-communication-04-architecture.md:524-527`

```text

6.7.1 **Status fields [P]**, each with its freshness stamp; a stale field reads "unknown" (FP-1 7.2.b): `mode`,
`binary` (built), `supervision` (built), `main` (built), `hub {id, declared, match}`, `revocation`, `quota`,
`consumers`, `reachability`, `clear`, `strength {per_agent_accounts: no, same_user_separation: "absent: agents run as
```

`interactive-agent-communication-04-architecture.md:893-896`

```text
  Note over SA,RA: (b) refusal by a revoked key
  H->>H: operator revokes key F, seq rises, list signed
  RS->>H: revocation.head each tick, then get
  SS->>H: post signed with key F
```

`interactive-agent-communication-04-architecture.md:991-991`

```text
9.5 Revocation unverified past PN-13: allow-list narrows, urgent downgraded (6.2.4).
```

`interactive-agent-communication-04-architecture.md:1045-1055`

```text
    HELD --> VACANT: clean release
  }
  state "Revocation list in a sidecar" as V {
    [*] --> UNVERIFIED
    UNVERIFIED --> VERIFIED: signed list, seq not lower
    VERIFIED --> VERIFIED: head unchanged or newer list verified
    VERIFIED --> STALE: no verified pull for PN-13
    STALE --> VERIFIED: verified pull
    VERIFIED --> ROLLBACK: lower seq offered
    ROLLBACK --> VERIFIED: refused, previous kept
  }
```

`interactive-agent-communication-04-architecture.md:1078-1080`

```text
10.2.3 (c) A sidecar's revocation list starts UNVERIFIED and becomes VERIFIED with a signed list whose sequence is not
lower. It stays VERIFIED while heads are unchanged or newer lists verify; PN-13 without a verified pull makes it
STALE until the next verified pull; a lower sequence is a ROLLBACK that is refused, keeping the previous list.
```

`interactive-agent-communication-04-architecture.md:1230-1230`

```text
| BP-10 | backups and snapshots | K-14 reconciliation, receipts' chain heads, revocation seq | monitored; RR-20 |
```

`interactive-agent-communication-04-architecture.md:1248-1248`

```text
| K-9b revocation | a revoked key is refused within one tick | status `revocation` freshness; drill | anything while the hub is down longer than PN-13 beyond narrowing |
```

`interactive-agent-communication-04-architecture.md:1404-1404`

```text
| DR-2 | Hub signing key pinning in PH-2 | A TOFU pin in each sidecar store, mismatch refused; B no pin until PH-3 out-of-band | **A** | revocation list unforgeable to a remote attacker now; same limit as TLS TOFU | proposed; operator rules it with the approval of this step (no separate O-n) | none yet |
```

`interactive-agent-communication-04-architecture.md:1459-1460`

```text
19.5 **AR-4** On .107 the hub runs on the agents' host, so a hijacked root agent holds the hub signing key: revocation
and dead letters are forgeable locally until the hub runs under its own account (Track A, FP 10.1.5 "hub access").
```

### 3. Pseudo-code 06a: AL-20 (in full, with test vectors), AL-21, AL-22 (in full), AL-18 (header and 2.2.h only; the pin form), AL-23 step 5

`interactive-agent-communication-06a-pseudo-code-ph2-hardening.md:140-190`

```text
### 2.4 AL-20 — Canonical form, digest and signature of signed hub state (ARC4 6.2.1; FP 10.3.2.c; test vectors)

2.4.a **SERVES:** SI-11, SI-12, SI-21, SI-3 (envelopes); TH-26, TH-58; AFC-22.
2.4.b **PROBLEM:** a signed JSON object must verify identically whatever its byte encoding, and a signature made for one
purpose must not verify for another. **APPROACH:** RFC 8785 (JCS) on a restricted value set, SHA-256 of the canonical
bytes, ed25519 over a domain-separated string `<schema>\n<hex digest>`.
2.4.c **ALGORITHM:**
```text
canonical(obj):
 1  obj passes strict JSON (06 3.3): UTF-8, no duplicate keys, depth <= 8, size <= 1 MiB, numbers only integers in
    [-(2^53-1), 2^53-1] (no fraction, exponent or -0)                                                         # FCW-88
 2  remove the fields "digest" and "signature" from the top level
 3  serialise by RFC 8785: object members sorted by UTF-16 code units of their names, no insignificant whitespace,
    strings with the minimal JSON escapes (\" \\ \b \f \n \r \t, other controls \u00xx lower-case), integers in
    shortest decimal form
digest(obj)  = lower-case hex SHA-256 of canonical(obj)
sign(k, obj) = ed25519_sign(k, utf8(obj.schema + "\n" + digest(obj)))
verify(pub, obj):
 1  require obj.digest == digest(obj)                     # recomputed, never trusted from the object
 2  require ed25519_verify(pub, utf8(obj.schema + "\n" + obj.digest), obj.signature)                            # FCW-89
```
2.4.c.1 **Test vectors** (computed by `.context/working/t3355/vectors.py`; public RFC 8032 test key 1, public key
`d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a`; never an estate key):

| Id | Input | Expected |
|---|---|---|
| V-REV-1 | revocation list, `seq` 7, one entry (fields below) | canonical 436 bytes, digest `044cfb1cf8c79cc60832cce9145895ff1bca2338d28c447823e687a3b6fe5767`, signature `02739ad55f80d54104dc59466efe9671ec5493df329023a71468926f193f76f6173b449790f2836e57af6bfc7ef8699c6f2f28f38a718846b4d4a0c8b2c7830e` |
| V-REV-2 | the same object with keys reordered, spaces added and `claude-termlink-alt` | canonical bytes and digest equal to V-REV-1 |
| V-HEAD-1 | revocation head `{schema: termlink.revocation-head/1, hub_id, seq: 7, digest: <V-REV-1 digest>}` | canonical 151 bytes, digest `cc363257e39103a2d922e6ec0000810659449f3c0098e0471ab8c9932f2b24dd`, signature `ea99514e54465969c9ff89430be6a72c3cde1a4e2bdaaa2a2c53f7db5ac44eedc70257e8c8d06b8336b0376f7f798a27daf215292901a0548fc20d98934c8202` |
| V-FWD-1 | forward envelope (06 AL-14), generation 12, hop 1 | canonical 320 bytes, digest `7a7e9bf16ae3f54de6f1007b0ec917cac6f613c9dd5ce7eed6e9a38764af421d`, signature `29495852a7b15ef34240f9471f42f49ae3c3090e76cd37eb0a0277cbbff3d0b0ffc471503e8b9839a5cd2abd9431e5ae31a9add9b6b812014b117ce834dcdc0a` |
| V-NEG-1 | V-REV-1 with `"seq":7.0`, or `"seq":7e0` | refused at step 1 (FCW-88), never hashed |
| V-NEG-2 | V-REV-1 with `"seq"` twice | refused (duplicate key) |
| V-NEG-3 | V-REV-1 signed over the bare digest (no `termlink.revocation/1\n` prefix): signature starting `3b0af0c2379828c0e02d6a1ee48ffa95` | does not verify (FCW-89) |
| V-NEG-4 | V-REV-1 with `seq` 9007199254740992 (2^53) | refused (out of range) |

2.4.c.2 V-REV-1's canonical bytes, exactly: `{"entries":[{"agent":"claude-termlink-alt","by":"operator","event":
"admin-revoke","key":"ed25519:3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29","project":
"010-termlink","reason":"key exposed","revoked_at":1762862399000,"scope":"agent_key"}],"hub_id":"cacc73ea32b121dd",
"issued_at":1762862400000,"prev_digest":"0000000000000000000000000000000000000000000000000000000000000000","schema":
"termlink.revocation/1","seq":7}` (shown wrapped; no whitespace or newline in the bytes).
2.4.d **INVARIANTS:** (1) two objects verify the same iff their canonical bytes are equal; (2) a signature for schema A
never verifies as schema B (the schema is the domain prefix).
2.4.e **FAILS CLOSED WHEN:** 2.4.e.1 **FCW-88** [AFC-22; E-42] a number that is not an integer in range, a duplicate
key, invalid UTF-8, depth over 8 or size over 1 MiB → refused before hashing. 2.4.e.2 **FCW-89** [AFC-22; E-42] the
digest does not recompute, or the signature is not over `<schema>\n<digest>` with the expected key → refused.
2.4.f **EDGE CASES:** (1) Unicode normalisation is **not** applied (RFC 8785 does not), so `é` precomposed and
decomposed are different objects; producers emit what they mean, verifiers compare bytes; (2) keys outside the BMP sort
by UTF-16 code units, not by code point (RFC 8785 3.2.3); no schema here uses such keys; (3) stateless.
2.4.g **EXECUTION SEMANTICS:** pure. 2.4.h **CRYPTOGRAPHY:** SHA-256, ed25519 (RFC 8032, pure, no pre-hash); the
algorithm is pinned by the schema version (`termlink.revocation/1`); a new algorithm needs a new schema version.
**OBS:** none (pure); the vectors are the fixture of E-42.
```

`interactive-agent-communication-06a-pseudo-code-ph2-hardening.md:192-230`

```text
### 2.5 AL-21 — Issuing a revocation at the hub (K-9b; ARC4 6.2.1, 6.2.2; B-5; OQ-h A)

2.5.a **SERVES:** RK-1, SI-11, SI-24; TH-26, TH-34, TH-56; AFC-22.
2.5.b **PROBLEM:** revoke an agent key by the operator or by the agent's own old key, as numbered signed state.
**APPROACH:** one serialised writer on the hub: log, build the next list, sign, persist atomically, then answer.
2.5.c **ALGORITHM:**
```text
revoke(req):                       # admin revoke --key F --why W (root-only local socket), or revocation.request
 1  if req is a revocation.request:
        require verify(F.pub, req) where the request is canonical {schema: termlink.revocation-request/1, key: F,
        project, agent, reason, at} and F is the key being revoked (or the key card's current key, AL-33)   # FCW-90
 2  with revocation.lock:
 3    admin_log.append({method: revoke, caller, params_digest, by: operator|self}); Err -> refuse              # FCW-91
 4    cur = load(revocations.json)
      if F already in cur.entries: return {ok, seq: cur.seq, already: true}             # idempotent
 5    nxt = cur with entries + [{scope: agent_key, key: F, project, agent, reason, revoked_at: wall(), by, event}],
            seq = cur.seq + 1, prev_digest = cur.digest, issued_at = wall()
      nxt.digest = digest(nxt); nxt.signature = sign(hub signing key, nxt)                                     # AL-20
 6    atomic_write(revocations.json, nxt); Err -> keep cur, refuse                                            # FCW-92
 7    attention("key revoked", F, by); return {ok, seq: nxt.seq}
revocation.head -> {schema: termlink.revocation-head/1, hub_id, seq, digest} signed (AL-20)      # ~250 bytes
revocation.get  -> the current list
```
2.5.d **INVARIANTS:** (1) `seq` rises by exactly one per change; (2) `prev_digest` chains every version; (3) every
revocation has an admin-log row before it takes effect.
2.5.e **FAILS CLOSED WHEN:** 2.5.e.1 **FCW-90** [new; E-43] a self-revocation request is not signed by the key it
revokes → refused and logged. 2.5.e.2 **FCW-91** [new; ZR-1] the admin-log row cannot be written → no revocation.
2.5.e.3 **FCW-92** [new; ZR-1] the new list cannot be persisted → the previous list stays, the call fails.
2.5.f **EDGE CASES:** (1) replayed self-request: idempotent (already revoked); (2) concurrency: the lock serialises
writers; (3) clock: `revoked_at` is the hub's wall clock, display only; (4) crash table:

| Crash between | State after restart | Recovery |
|---|---|---|
| log row and persist | row, list unchanged | the operator's call failed visibly; re-run is idempotent |
| persist and attention | new list, no entry | start-up check: a list `seq` newer than the last attention row raises the entry |

2.5.g **EXECUTION SEMANTICS:** the effect (the signed list) is atomic; sidecars apply it at their next pull (AL-22).
2.5.h **CRYPTOGRAPHY:** AL-20 with schema `termlink.revocation/1` and `termlink.revocation-head/1`.
**OBS:** OBS-39 admin-log rows; OBS-40 `revocation.head`.
```

`interactive-agent-communication-06a-pseudo-code-ph2-hardening.md:232-280`

```text
### 2.6 AL-22 — Revocation pull, verification and the stale rule at the sidecar (K-9b; ARC4 6.2.3..6.2.5; B-5; answer 06 4.7)

2.6.a **SERVES:** SI-11, SI-18, R-63, RK-1; TH-26, TH-56, TH-58; AFC-22; DR-8.
2.6.b **PROBLEM:** a failed pull must never read as "nothing revoked", a rollback must be refused, a restored sidecar
must notice a newer head. **APPROACH:** per tick one verified-or-failed pull (06 4.7), an `unverified_since` on
`mono()`, the PN-13 narrowing, and a start-up state that is unverified until the first verified contact.
2.6.c **ALGORITHM (D-1.8 draws it):**
```text
on start: held = store.revocation (last verified list, may be from a backup); state = UNVERIFIED_SINCE_START   # FCW-96
pull_tick():
 1  h = hub.revocation.head (10 s); Err -> failed("head: " + kind)                                             # FCW-93
 2  require verify(pin, h) (AL-20, AL-18)                    else failed("head-signature"), tamper = true
 3  if h.seq < held.seq: failed("rollback"), tamper = true                                                     # FCW-94
    if h.seq == held.seq and h.digest == held.digest: verified()
    if h.seq == held.seq and h.digest != held.digest: failed("fork"), tamper = true
 4  if h.seq > held.seq:
        L = hub.revocation.get (10 s, <= 1 MiB); Err -> failed("get: " + kind)
        require verify(pin, L) and L.digest == h.digest and L.seq == h.seq
                and (L.seq != held.seq + 1 or L.prev_digest == held.digest)   else failed("list"), tamper = true
        with store.lock: atomic_write(revocation, L); held = L; verified()
verified(): last_verified = mono(); state = VERIFIED
failed(why): if tamper: attention("revocation-tamper", why, coalesce = hour)
             if state != STALE and mono() - last_verified > PN-13 (1 h): state = STALE; attention("revocation stale")   # FCW-95
check(key) at receive (AL-23): refused iff key in held.entries            # keys on the last verified list stay refused
effects while state in {STALE, UNVERIFIED_SINCE_START}:
   interrupt allow-list = own project's verified roster; urgent from others -> delivered as normal, recorded downgrade  # FCW-95, FCW-96
status.revocation = "verified seq N at T" | "unverified since T (<why>)" | "stale since T"
```
2.6.d **INVARIANTS:** (1) `held.seq` never decreases; (2) a key on any list ever verified by this store stays refused
(lists only grow); (3) `last_verified` moves only in `verified()`; (4) narrowing never drops a message.
2.6.e **FAILS CLOSED WHEN:** 2.6.e.1 **FCW-93** [AFC-22; E-41] any failed-pull class of 06 4.7 → not verified;
`unverified_since` stays at the last verified pull. 2.6.e.2 **FCW-94** [AFC-22; E-40] the head or list has a lower
`seq`, or the same `seq` with another digest → refused, previous kept, tamper entry. 2.6.e.3 **FCW-95** [AFC-22; E-41]
PN-13 (1 h) without a verified pull → allow-list narrowed to the own project, urgent mail from others downgraded,
needs-attention. 2.6.e.4 **FCW-96** [new; E-40 (b), ZR-1] a started or restored sidecar has had no verified contact
yet → the same narrowing until its first verified pull (SI-11 "unreconciled until its first verified hub contact").
2.6.f **EDGE CASES:** (1) restore from backup with an old list: the first head shows a higher `seq`; the new list is
fetched before any urgent delivery from others (FCW-96); with the hub unreachable, normal delivery from a revoked key
is RR-20 (limit probe E-40 (c)); (2) clock: PN-13 on `mono()`; across a reboot `last_verified` is absent, so the state
is UNVERIFIED_SINCE_START; (3) concurrency: the receive path reads `held` under the store lock; (4) crash table:

| Crash between | State after restart | Recovery |
|---|---|---|
| get and install | old list | the next tick pulls again |
| install and `verified()` | new list, state from start | UNVERIFIED_SINCE_START until the next head matches (one tick) |

2.6.g **EXECUTION SEMANTICS:** read-only towards the hub. 2.6.h **CRYPTOGRAPHY:** AL-20 under the pin of AL-18.
**OBS:** OBS-41 status `revocation`; OBS-42 `revocation-tamper` and `revocation stale` entries; OBS-43 downgrade
records.
```

`interactive-agent-communication-06a-pseudo-code-ph2-hardening.md:77-77`

```text
### 2.2 AL-18 — Hub signing key and the sidecar's pin (K-9a; ARC4 6.1.4, 6.1.5; B-1; DR-2)
```

`interactive-agent-communication-06a-pseudo-code-ph2-hardening.md:110-113`

```text
2.2.g **EXECUTION SEMANTICS:** none beyond the pin write (idempotent). 2.2.h **CRYPTOGRAPHY:** ed25519 (RFC 8032), the
fingerprint is `ed25519:` + hex SHA-256 of the 32-byte public key **[P]** (ARC4 6.1.4 says `ed25519:<fingerprint>`
without fixing the hash; SHA-256 matches the TLS pin's form; ZR-5).
**OBS:** OBS-36 status `hub.signing_key {pinned, offered, state}`; OBS-37 admin rows `hub-key-pinned`.
```

`interactive-agent-communication-06a-pseudo-code-ph2-hardening.md:296-296`

```text
 5  if revocation.check(m.key): refuse "revoked", signed notice to the sender (AL-22)                          # FCW-99
```

`interactive-agent-communication-06a-pseudo-code-ph2-hardening.md:323-324`

```text
2.7.e.2 **FCW-98** [AFC-33; E-69, E-93] signature fails or the signer has no card → refused. 2.7.e.3 **FCW-99**
[AFC-22; E-39] the key is on the verified list → refused, signed notice. 2.7.e.4 **FCW-100** [AFC-3; E-3, ZR-1] the
```

### 3b. Pseudo-code 06: failed-pull definition (4.7), FCW register rows FCW-88..FCW-96 and FCW-99, FCW-76, FCW-200, AFC-22 map

`interactive-agent-communication-06-pseudo-code.md:199-210`

```text
4.7 **(g) Which revocation RPC failures count as a failed pull (AL-22).** Per tick the sidecar makes one pull
attempt. It is **verified** only in two cases: (1) `revocation.head` answers with a head whose signature verifies with
the pinned hub signing key and whose `(seq, digest)` equals the list held; or (2) the head shows a higher `seq`, and
`revocation.get` returns a list whose canonical form hashes to the head's digest, whose signature verifies with the
pinned key, whose `seq` equals the head's and whose `prev_digest` chains when `seq` = held + 1; the list is then
installed atomically. **Every other outcome is a failed pull**: transport error, timeout (10 s per RPC), a hub error or
refusal response on either RPC, `get` failing after a good head, a head or list with a bad or missing signature or a
signature by an unpinned key, a digest mismatch, a lower `seq` (rollback, TH-58), a broken `prev_digest` chain, a
malformed or oversized body (> 1 MiB). Timeout and refusal are treated alike for the stale rule (PN-13); they differ
only in what status says and in one thing: a signature failure, a wrong key or a rollback also raises an immediate
needs-attention entry (`revocation-tamper`, coalesced per hour), because they are evidence of interference, not of an
outage. `unverified_since` is the `mono()` time of the last verified pull, never of the first failure.
```

`interactive-agent-communication-06-pseudo-code.md:1559-1567`

```text
| FCW-88 | 06a | AL-20 | AFC-22 | E-42 | a number that is not an integer in range, a duplicate key, invalid UTF-8, depth over 8 or size over 1 MiB → refused before hashing. |
| FCW-89 | 06a | AL-20 | AFC-22 | E-42 | the digest does not recompute, or the signature is not over `<schema>\n<digest>` with the expected key → refused. |
| FCW-90 | 06a | AL-21 | new | E-43 | a self-revocation request is not signed by the key it revokes → refused and logged. |
| FCW-91 | 06a | AL-21 | new | ZR-1 | the admin-log row cannot be written → no revocation. |
| FCW-92 | 06a | AL-21 | new | ZR-1 | the new list cannot be persisted → the previous list stays, the call fails. |
| FCW-93 | 06a | AL-22 | AFC-22 | E-41 | any failed-pull class of 06 4.7 → not verified; `unverified_since` stays at the last verified pull. |
| FCW-94 | 06a | AL-22 | AFC-22 | E-40 | the head or list has a lower `seq`, or the same `seq` with another digest → refused, previous kept, tamper entry. |
| FCW-95 | 06a | AL-22 | AFC-22 | E-41 | PN-13 (1 h) without a verified pull → allow-list narrowed to the own project, urgent mail from others downgraded, needs-attention. |
| FCW-96 | 06a | AL-22 | new | E-40 (b), ZR-1 | a started or restored sidecar has had no verified contact yet → the same narrowing until its first verified pull (SI-11 "unreconciled until its first verified hub contact"). |
```

`interactive-agent-communication-06-pseudo-code.md:1570-1570`

```text
| FCW-99 | 06a | AL-23 | AFC-22 | E-39 | the key is on the verified list → refused, signed notice. |
```

`interactive-agent-communication-06-pseudo-code.md:893-895`

```text
        if Err(rerun receiver checks (policy, revocation AL-22, switch)):   # [B] the second run of ARCH 7.1
            refuse(m, reason); continue                     # CR-6.40: REFUSED, signed refusal notice to the sender,
                                                            # removed from the pending set of AL-5 and AL-8    (FCW-200)
```

`interactive-agent-communication-06-pseudo-code.md:943-945`

```text
5.9.e.8 **FCW-200** [AFC-22; E-39] (changed in v0.3 by CR-6.40) the second run of the receiver checks fails at pull
time (key revoked since STORED, a policy that no longer admits it; the switch is step 1, never a refusal) → the message is REFUSED with a signed notice to the sender and leaves
the pending set of AL-5 and AL-8; it is never left STORED and rung for.
```

`interactive-agent-communication-06-pseudo-code.md:1406-1407`

```text
5.15.e.5 **FCW-76** [AFC-22, AFC-25; E-39, E-50] the original sender's key is revoked, its sequence number is refused, or
its quota at X is full → refused or held exactly as a direct message would be.
```

`interactive-agent-communication-06-pseudo-code.md:1671-1671`

```text
| FCW-200 | 06 | AL-9 | AFC-22 | E-39 | (changed in v0.3 by CR-6.40) the second run of the receiver checks fails at pull time (key revoked since STORED, a policy that no longer admits it; the switch is step 1, never a refusal) → the message is REFUSED with a signed notice to the sender and leaves the pending set of AL-5 and AL-8; it is never left STORED and rung for. |
```

`interactive-agent-communication-06-pseudo-code.md:1709-1709`

```text
| AFC-22 | FCW-76, FCW-88, FCW-89, FCW-93, FCW-94, FCW-95, FCW-99, FCW-200 |
```

### 4. Evidence: every E-n tagged B-5 or citing K-9b / the revocation list

Levels as stated in the headers: E-39 L4 Tier 2; E-40 L4 + L6 Tier 2; E-41 L2 Tier 1; E-42 L3 Tier 1; E-43 L4 Tier 2;
E-167 L2 Tier 1; E-127 is an operational check row of 05 section 10 (no L-level stated in the row).

`interactive-agent-communication-05-evidence.md:520-553`

```text
**E-39 — A revoked key is refused within one tick.** Links: SI-11, R-46, RK-1, TH-26, TH-56, AFC-22, FCW-200. Category: adversarial. Level: L4. Tier: 2. Phase: PH-2 (B-5).
E-39.1 EVIDENCE: after `admin revoke --key F`, a message signed with F is refused at the receiver from the next tick with the hub reachable, recorded, and the sender sees a signed refusal notice; mail to the revoked agent waits (ARC4 6.2.5).
E-39.2 ARRANGE: throwaway hub; sidecars `fx-a` (sender, key F) and `fx-b`.
E-39.3 ACT: revoke F through the root-only admin socket; `fx-a` sends immediately and again after one tick; `fx-b` sends to `fx-a`.
E-39.4 ASSERT: time from revoke to the first refusal ≤ one tick + one pull; nothing STORED from F after that; the sender's record shows "refused-revoked"; `fx-b`'s message to `fx-a` waits (not dropped); admin-log row and needs-attention entry for the revocation (SI-24).
E-39.5 CONTROL: mutant that skips the revocation check stores the message.
E-39.6 (changed in v0.2, follows CR-6.40) CASE: key F revoked after a message from F is STORED and before the hook pulls it. ASSERT: the pull's second receiver check refuses it: REFUSED with a signed notice, removed from the pending set; no ring or watchdog line names it afterwards (FCW-200). CONTROL: mutant that skips a failing message but leaves it STORED keeps ringing for it.

**E-40 — Rollback of the list and restore from backup.** Links: SI-11, TH-58, RR-20, AFC-22. Category: adversarial. Level: L4 + L6. Tier: 2. Phase: PH-2 (B-5).
E-40.1 EVIDENCE: a list with a lower `seq` is refused and the previous kept and flagged; a sidecar restored from backup with an older list and its older stored `seq` detects the higher head on first contact and refuses the revoked key; with the hub unreachable after such a restore, normal delivery from the revoked key is the stated limit RR-20 (TM SI-11 probe).
E-40.2 ARRANGE: throwaway hub at `seq` 7 with F revoked at 6; a snapshot of the sidecar store taken at `seq` 5.
E-40.3 ACT: (a) serve `seq` 5 from a test proxy; (b) restore the snapshot and reconnect; (c) restore the snapshot with the hub unreachable and send from F.
E-40.4 ASSERT: (a) refused, status `ROLLBACK refused, kept seq 7`; (b) first head read shows 7, list fetched, F refused; (c) verdict `LIMIT` naming RR-20, with no circuit and no urgent delivery from F.
E-40.5 CONTROL: mutant that accepts any `seq` passes (a) and fails.

**E-41 — A failed pull is not an empty list; the stale rule narrows, never stops.** Links: SI-11, SI-18, R-63, DR-8, AFC-22. Category: error handling. Level: L2. Tier: 1. Phase: PH-2 (B-5).
E-41.1 EVIDENCE: when only the list fetch fails while posts succeed, status shows `unverified since T`; after PN-13 (1 h) the interrupt allow-list narrows to the own project's verified roster, urgent mail from others is downgraded (never dropped), a needs-attention entry appears, and keys on the last verified list stay refused (ARC4 6.2.4).
E-41.2 ARRANGE: simulated clock; a hub client stub that fails `revocation.head` and `revocation.get` only.
E-41.3 ACT: advance 59:59 and 60:01; send urgent from another project's allowed sender and from a key on the last list.
E-41.4 ASSERT: status values at both times; after 60:01 the urgent message is STORED and handed over as normal priority with a recorded downgrade; the revoked key's message refused; zero messages dropped.
E-41.5 CONTROL: mutant "failed pull treated as an empty list" accepts the revoked key (ARC4 6.2.6).

**E-42 — Forged or malformed revocation lists are refused (fuzz).** Links: SI-11, TH-26, TH-58, AFC-22. Category: adversarial. Level: L3. Tier: 1. Phase: PH-2 (B-5).
E-42.1 EVIDENCE: only a list whose canonical form (RFC 8785) hashes to `digest`, whose `signature` verifies over `termlink.revocation/1\n<digest>` with the pinned key, whose `prev_digest` chains and whose `seq` is not lower is accepted.
E-42.2 ARRANGE: a valid list; a mutator over JSON (key order, whitespace, number forms `1.0`/`1e0`, Unicode escapes, duplicate keys, missing fields, signature by another key, signature over the digest without the domain prefix, broken `prev_digest`).
E-42.3 ACT: feed at least 10^5 mutated lists.
E-42.4 ASSERT: every list that is not canonically equal to a validly signed one is refused; semantically equal re-encodings of the valid list verify the same; no crash.
E-42.5 CONTROL: mutant that verifies over the raw bytes instead of the canonical form fails the key-order cases.

**E-43 — Self-revocation and the audit of every revocation.** Links: SI-11, SI-24, TH-34, TH-56. Category: adversarial. Level: L4. Tier: 2. Phase: PH-2 (B-5).
E-43.1 EVIDENCE: a `revocation.request` signed by the old key is accepted without the operator; one signed by another key is refused; every revocation writes an admin-log row, a needs-attention entry and re-signs the list (ARC4 6.2.2).
E-43.2 ARRANGE: throwaway hub; agents `fx-a`, `fx-b`.
E-43.3 ACT: `fx-a` self-revokes; `fx-b` submits a request to revoke `fx-a`'s new key.
E-43.4 ASSERT: first accepted (`by: self`), `seq` +1; second refused and logged; each accepted revocation has its three traces.
```

`interactive-agent-communication-05-evidence.md:946-951`

```text
**E-167 — Revocation issue is durable or does not happen.** Links: SI-11, SI-24, RK-1, FCW-91, FCW-92. Category: error handling. Level: L2. Tier: 1. Phase: PH-2 (B-5)
E-167.1 EVIDENCE: a revocation whose admin-log row cannot be written is not issued; a new list that cannot be persisted leaves the previous list and the call fails.
E-167.2 ARRANGE: hub with the admin log read-only, then the list file's directory read-only.
E-167.3 ACT: `admin revoke --key F` in each state.
E-167.4 ASSERT: no new sequence number, previous list served, the call exits non-zero with the reason.
E-167.5 CONTROL: mutant persisting the list before the admin row serves a revocation with no audit row.
```

`interactive-agent-communication-05-evidence.md:1300-1300`

```text
| E | Links | EVIDENCE (property, ARC4 15) | ARRANGE | ACT | ASSERT (fires with exit 1 when) | CONTROL | Not proven |
```

`interactive-agent-communication-05-evidence.md:1304-1304`

```text
| E-127 | K-9b, SI-11 | A revoked key is refused within one tick; the list is fresh | status `revocation` | read each tick; count refusals after each revocation row | `unverified` older than PN-13 without the narrowing applied, or a message from a revoked key STORED | drill revocation (E-39) on the canary pair | anything while the hub is down longer than PN-13 beyond narrowing |
```

`interactive-agent-communication-05-evidence.md:1491-1491`

```text
| AFC-22 | revocation pull failed: not an empty list; lower seq: refused; stale past PN-13: narrow | 6.2.3, 6.2.4 | E-39, E-40, E-41, E-42 |
```

`interactive-agent-communication-05a-coverage.md:250-250`

```text
| AFC-22 | E-39, E-40, E-41, E-42 | covered |
```

`interactive-agent-communication-05a-coverage.md:269-269`

```text
| K-9b | E-127 | covered |
```

## Cited parameters/invariants

### DR-8 (stale rule; ruled A). DR-2 (pin) is quoted in section 2 above

`interactive-agent-communication-04-architecture.md:1410-1410`

```text
| DR-8 | Revocation stale rule effect in PH-2 | A narrow allow-list to own project, downgrade urgent; B stop all delivery | **A** | never stops all mail on a hub blip | proposed; operator rules it with the approval of this step (no separate O-n) | none yet |
```

### OQ-h ruling (arc-011-ph2-architecture.md:138, with the table header)

`arc-011-ph2-architecture.md:130-131`

```text
| id | Question | Options | Recommendation |
|---|---|---|---|
```

`arc-011-ph2-architecture.md:138-138`

```text
| OQ-h | Revocation list: where and who signs | A a hub topic signed by the hub signing key; requested by the agent's own project (signed by the old key) or by the operator through runme; B operator-only | **Ruled 2026-10-13: A** (operator revokes through the hub's root-only local socket on its host via a runme action, root on the hub host standing in for the operator key until K; the hub signs the list with its signing key (P10) and keeps it as signed, numbered hub state, not a topic; self-revocation by a statement signed with the old key stays; every revocation logged and a needs-attention entry. Open: replace root-on-host by the operator key in PH-3; whether a revocation also needs the operator's confirmation once a provider exists) |
```

### PN-1, PN-3, PN-13 (01 parameter table and 02 parameter table)

`interactive-agent-communication-01-requirements.md:1726-1728`

```text
| Number | Meaning | Starting value | Requirement |
|---|---|---|---|
| PN-1 | circuit credential lifetime | 1 hour, from establishment, never reset | R-46.a (4d), R-7.e |
```

`interactive-agent-communication-01-requirements.md:1730-1730`

```text
| PN-3 | revocation pull | every 30-second tick | R-46.a (4e) |
```

`interactive-agent-communication-01-requirements.md:1740-1740`

```text
| PN-13 | stale-list limit | equal to PN-1 (1 hour) | R-46.a (4g) |
```

`interactive-agent-communication-02-threat-model.md:1411-1411`

```text
| PN-1 | Circuit credential lifetime | 1 hour | 7.3: the aim is to survive a typical hub restart; a thief's window with the hub unreachable is one hour and needs the key too | Hypothesis H-1 (typical hub outages last minutes to an hour). Measure hub outage and restart durations from the hub logs and `fleet history` over 30 days | OQ-3 |
```

`interactive-agent-communication-02-threat-model.md:1413-1413`

```text
| PN-3 | Revocation pull | Every existing 30-second tick | Reuses the tick of R-17; no new timer | A ruled number reused, not new | step 4 |
```

`interactive-agent-communication-02-threat-model.md:1423-1423`

```text
| PN-13 | Stale-list limit: time without a verified revocation pull after which new circuits stop and urgent delivery is downgraded (11.2.d) | Equal to PN-1 | A sidecar cut off for longer than a credential lives has nothing it can verify | Follows PN-1; same status | OQ-3 |
```

### RK-1 and PR-16 (02 11.1, 11.2.a-e; RK-2..RK-7 rows and 11.2.f omitted)

`interactive-agent-communication-02-threat-model.md:394-398`

```text
11.1 Revocation is not one thing. Review round 1 (Codex 2b) found that version 0.1 mixed several different "take it away" actions into one table. They are kept apart here, because each has a different effect and a different bound. Two rules apply to all of them. **(a) Revocation never makes mail DEAD.** R-44.a and R-60.a reserve DEAD for the home hub's own determination that an instance has ended; a revoked key proves nothing about whether the instance is still running. **(b) Each scope has two bounds.** The *reachable* bound applies when the sidecar can reach its home hub. The *partitioned* bound applies when it cannot. The two are stated separately so that no sentence promises the reachable speed in a partition. All bounds are **[P]**.

| Scope | Who may revoke | Effect | Reachable bound | Partitioned bound |
|---|---|---|---|---|
| RK-1 An agent key (stolen, leaked, misfiled): the key stops being believed as a sender | The operator, or the agent's own project by a statement signed with the old key | The home hub marks the key revoked. Messages signed by it are refused by receivers and hubs. Mail **to** that agent is not dead-lettered: the sender sees WAITING FOR RECIPIENT ("recipient not able to receive", R-44.a) with the reason "key revoked, operator action needed", and a needs-attention entry appears. DEAD follows only if the home hub then determines the instance ended (11.3) | The next 30-second tick (PN-3) | The sidecar's last list holds until it is stale (11.2.d); circuits end at credential expiry |
```

`interactive-agent-communication-02-threat-model.md:406-411`

```text
11.2 **PR-16 [P] The revocation list, and what happens when it cannot be read.**
11.2.a Each hub publishes a signed revocation list. Every sidecar pulls it on its 30-second tick when the hub is reachable, and checks it at every set-up and every renewal.
11.2.b **Rollback protection, and why a stored counter is not enough (review round 2, Codex 3a).** Each list carries a hub-signed, strictly increasing sequence number, and a sidecar refuses a list with a lower number, an unsigned list, or one whose signature does not verify against the hub's signing key (C-9). **That alone does not defeat a restore.** A restored backup of a sidecar restores *both* the older signed list *and* the highest number it had seen, so the two agree with each other and the sidecar sees nothing wrong. Version 0.2 claimed the stored number "cannot forget" a revocation; it can. What detects a restore is an independent current authority: the hub, reached live. Rules: (i) every authenticated answer of the sidecar's home hub carries the hub's *current* revocation head (the latest sequence number); a sidecar holding a list below it treats the list as stale and pulls at once; (ii) a sidecar that has just started, or whose store was restored, is *unreconciled* until its first verified contact with its home hub; while unreconciled it sets up and renews no circuit (a restart already expires every circuit, 7.2.d.4), delivers no urgent turn mid-turn (downgrade, never drop, R-63.a) and treats the list as stale (11.2.d); ordinary delivery continues, because R-7.a requires the sidecar to work without the hub; (iii) while cut off from the hub, a restored sidecar cannot know what was revoked after its backup was taken, and during that time it may deliver normal-priority mail from a sender whose key was revoked: that exposure is RR-20, stated and not hidden; (iv) the same applies to the signed policy objects of SI-21 (11.2.f) and to the roster (10.5.d).
11.2.c **A failed pull is a failure, not an empty list.** If a hub answers posts and pulls but the revocation list cannot be fetched or verified, the sidecar treats the list as not refreshed. It never reads "no list" as "no revocations".
11.2.d **The stale rule.** When a sidecar has had no successful, verified revocation pull for longer than PN-13 (proposed equal to the credential lifetime, PN-1), it (i) refuses to set up or renew circuits, (ii) downgrades urgent mid-turn deliveries to normal, never dropping them (R-63.a), and (iii) shows "revocation list stale since T" in its status call and in needs-attention. Established circuits continue until their credential expires (R-7.e (1)). The rule fails closed for new trust and stays open for ordinary delivery. Its cost, stated honestly: during a long outage, urgency is lost and no new circuit can start.
11.2.e The bounds as sentences a test can check. With the hub reachable, a revoked key's next turn is refused no later than one tick plus one pull. With the hub unreachable, a revoked circuit works for at most the remaining life of its credential, counted from establishment, and never longer than PN-1. SI-23's "within one tick" applies only to sidecars that can reach the hub; on a partitioned host the operator uses the local admin scope of the sidecar (PR-8), which acts at once on that host.
```

### SI-11 (02 invariant table, 03 floor table)

`interactive-agent-communication-02-threat-model.md:1345-1345`

```text
| SI-11 | Each revocation scope RK-1 to RK-6 (11.1) takes effect within its stated bound, and the two bounds are never conflated: with the hub reachable, at the next tick; with the hub unreachable, at most the remaining life of the credential counted from its establishment. RK-7 (the alias end date, R-71) is a calendar date checked by R-71 itself and is outside this invariant by design. A sidecar refuses a revocation list with a lower sequence number or without a valid signature, a failed pull is never read as an empty list, the home hub's current head is compared on every contact so that a restored sidecar is detected, a started or restored sidecar is unreconciled until its first verified hub contact, and after PN-13 without a verified pull it refuses new circuits and downgrades urgent delivery | Revoke, then measure the time until the next turn is refused, hub reachable and unreachable; replay an older list; make only the list fetch fail while posts succeed; **restore a sidecar from backup so that both the older list and its stored highest sequence number come back, make the hub reachable, and require that the first contact detects the lower head and the revoked key is refused**; repeat with the hub unreachable and require no new circuit and no urgent delivery, and record that normal delivery from the revoked key is the known limit RR-20 | TH-26, TH-56, TH-58, PR-16 |
```

`interactive-agent-communication-03-floor-and-phasing.md:104-104`

```text
| SI-11 | Each revocation scope takes effect within its stated bound; reachable and partitioned bounds never conflated; failed pull is not an empty list | R-46, R-57 | PH-2 (RK-1, agent keys) | PH-4 (RK-6), PH-5 (RK-2, RK-3, RK-5) | none | R CR-7, OQ-3 |
```

### TH-58

`interactive-agent-communication-02-threat-model.md:857-860`

```text
TH-58 **Revocation rollback, suppression or a stale list.** TB-4, TB-5 · T, D, E · ADV-2a, ADV-11, ADV-13
15.3.31 Scenario: an attacker replays an older signed revocation list, blocks only the list fetch, or restores a sidecar from a backup, so that a revoked key or credential keeps working.
15.3.32 L medium (restores and re-vendors happen, REQ 2.4a; blocking one fetch needs only a network position). I high (a key known to be stolen keeps working).
15.3.33 Countermeasure: PR-16 and SI-11 (strictly rising sequence numbers, the home hub's current head compared on every contact so that a restore of both the list and its stored number is detected, an unreconciled start, a failed pull is not an empty list, the stale rule after PN-13). Residual: a restore while cut off from the hub, RR-20.
```

### R-63.a (R-46.a omitted for size: its revocation clauses (4e)-(4g) are restated in the PN rows and SI-11 above) and their phasing rows

`interactive-agent-communication-01-requirements.md:946-946`

```text
R-63.a [R] Ruled (OD-15 A refined, 3a): mid-turn urgent delivery MUST be accepted only from senders the receiver allows by verified key; the default allow-list is its own project plus the operator. Others are downgraded to normal and never dropped. Senders are verified by signing key (T-1427); per-agent keys exist (T-3346). [R] (changed in v0.5) (CR-10, ruling 2026-10-08 OQ-6 D) v0.4.1 text: "the default allow-list is its own project plus the operator". Until an operator key exists (R-76) the default allow-list is its own project only: nothing is operator-class, and a message is operator-class only if it verifies against an operator key kept off agent hosts. (CR-13, ruling 2026-10-10 D7) The allow-list is the one the receiver applies per message; what changes is who may write it: only operator-signed configuration, or a record that the hub or an operator-owned store holds and the sidecar verifies, with an epoch checked against the home hub's current policy head (R-78). (CR-19, ruling 2026-10-10 OQ-12) The allow-list is an instance of the rule model of R-73. (CR-11, ruling 2026-10-10 D7) Peers get no read of reachability beyond yes or no (R-53.a).
```

`interactive-agent-communication-03-floor-and-phasing.md:158-158`

```text
| R-46 | Floor | SI-9, SI-10, SI-11 | PH-5 |
```

`interactive-agent-communication-03-floor-and-phasing.md:166-166`

```text
| R-63 | Floor | SI-18, SI-21 | |
```

## Implementation surface

Named by the spec (06a AL-20..AL-22, 04 6.2, 05 E-39..E-43, E-127, E-167), mapped to what exists today.

1. **AL-20 canonicaliser (new, shared).** Strict-JSON gate (FCW-88: integers only in ±(2^53-1), no duplicate keys,
   depth ≤ 8, ≤ 1 MiB, valid UTF-8), RFC 8785 serialisation (UTF-16 key order, minimal escapes), `digest` = hex SHA-256,
   `sign`/`verify` over `<schema>\n<digest>` (FCW-89). Both hub and sidecar need it, and T-3704 slice B (T-3819: key
   replace, signed DEAD) needs it too, so it belongs in a crate both depend on (`termlink-protocol` or a new small
   crate), not in `termlink-sidecar/src/wire.rs`. No JCS implementation exists in `crates/` today (see 1d below).
   Fixtures: V-REV-1, V-REV-2, V-HEAD-1, V-FWD-1, V-NEG-1..4 (06a 2.4.c.1). Duplicate-key detection needs a parser
   that sees duplicates; `serde_json::from_str` into `Value` keeps the last one silently.
2. **Hub state and writer (AL-21).** `<runtime_dir>/revocations.json` (schema `termlink.revocation/1`), a single
   writer under `revocation.lock`, admin-log row BEFORE persist (FCW-91; `rpc_audit::admin_note` exists and is
   used by the key-card RPCs, `router.rs:947`), atomic write (FCW-92), idempotent re-revoke, signed by
   `hub_identity::HubIdentity::signing_key()` (`crates/termlink-hub/src/hub_identity.rs:43`). Start-up check: list
   `seq` newer than the last attention row → raise the entry (AL-21 crash table).
3. **Hub RPCs.** `revocation.head` → signed `termlink.revocation-head/1 {hub_id, seq, digest}`; `revocation.get` →
   the list; `admin revoke --key F --why W` (operator, local socket); `revocation.request` (self-revoke, signed
   `termlink.revocation-request/1 {key, project, agent, reason, at}` by the key being revoked or the card's current key,
   FCW-90). The empty list (seq 0, `prev_digest` 64 zeros, per V-REV-1's chain root) has to be defined as served state.
4. **Sidecar pull (AL-22).** Per 30 s tick (PN-3): head (10 s timeout) → verify under the pin
   (`hub_key_pin::verified_hub_key`, `crates/termlink-sidecar/src/hub_key_pin.rs:195`) → rollback / fork / chain
   checks (FCW-94) → get (≤ 1 MiB) → atomic install under `store.lock`. States UNVERIFIED_SINCE_START / VERIFIED /
   STALE (04 D-4 (c) also draws ROLLBACK). `last_verified` on `mono()`. Tamper attention coalesced per hour; stale
   attention at PN-13 (FCW-95); start/restore narrowing (FCW-96).
5. **Receive check (AL-23 step 5, FCW-99) and the pull-time re-check (06 AL-9, FCW-200).** Refuse a message whose
   signer key is on `held.entries`, record it, send a signed refusal notice; at hook pull, a STORED message whose key
   was revoked since is REFUSED and leaves the pending set (E-39.6). Forwarding (06 AL-15, FCW-76) applies the same.
6. **Stale-rule effects (DR-8 A).** Interrupt allow-list narrowed to own project's verified roster; urgent from
   others delivered as normal with a recorded downgrade (OBS-43); never drop.
7. **Status field** `revocation`: `"verified seq N at T" | "unverified since T (<why>)" | "stale since T"` (AL-22,
   OBS-41; 04 6.7.1 lists it among status fields). The sidecar status builder is `crates/termlink-sidecar/src/api.rs`
   (it already reports `hub_signing_key` at `api.rs:1252`).
8. **CLI / runme.** An operator `admin revoke` path run via a `/opt/termlink/runme.sh` action (idempotent, verifies
   on disk that the new `seq` is served, hermetic case in `tests/runme-fixtures.sh` via `RUNME_*` seams, per
   CLAUDE.md). Rollback per plan row: "pull off returns to PH-1 trust (status says so)" → a sidecar setting.
9. **Fixtures / harnesses.** E-41 and E-167 (L2, Tier 1): simulated clock and a hub-client stub failing only the
   revocation RPCs; seams must be estate-gated (06 4.8, AL-16). E-42 (L3): a JSON mutator, ≥ 10^5 cases (no proptest
   / fuzz dependency found in the workspace Cargo files). E-39, E-40, E-43 (L4 Tier 2): throwaway hub + `fx-a`/`fx-b`
   sidecars; E-40 (a) needs a test proxy serving an older list; E-40 (c) is a LIMIT verdict (RR-20). E-127: daily
   check on the canary pair (or carried by the status call when the guard cap, DP-3, forbids a new check).
10. **Mutants named.** Skip the revocation check (E-39.5); skip-but-leave-STORED (E-39.6); accept any `seq`
    (E-40.5); failed pull treated as an empty list (E-41.5, 04 6.2.6); verify over raw bytes instead of the
    canonical form (E-42.5); accept any signed request (E-43.5); persist the list before the admin row (E-167.5).

1d. **Existing code (paths only).** Canonical JSON today: `crates/termlink-sidecar/src/wire.rs:165`
(`canonical_json`, BTreeMap byte-order sort, no strict-JSON gate); `crates/termlink-sidecar/src/enckey.rs:223`
(comment: a "jcs" subset of strings and integers "which serde_json renders as RFC 8785 does");
`crates/termlink-protocol/src/control.rs:367` (`canonical_sign_bytes`, the transport signature, not JSON). Hub
identity and signing key: `crates/termlink-hub/src/hub_identity.rs`. Sidecar pin: `crates/termlink-sidecar/src/hub_key_pin.rs`.
K-15 key card (T-3704 slice A): `crates/termlink-hub/src/key_card.rs`. No file in `crates/` mentions "revocation"
except comments in those two identity/pin files. **V-REV vectors:** no tracked vector file. The generator is
`.context/working/t3355/vectors.py`, which is gitignored (`.gitignore:153`, `.context/working/*`); the values live
only in 06a 2.4.c.1 and copies in `docs/reports/T-3704-spec-extract.md`.

## Gaps and contradictions

1. **The pin cannot verify a signature.** AL-22 says `verify(pin, h)`, but the pin and `hub.version` hold only
   `ed25519:<sha256(pubkey)>` (AL-18 2.2.h; `hub_key_pin.rs` header). Verifying needs the 32-byte public key, and no
   RPC exposes it. The head and list schemas (V-HEAD-1, V-REV-1) carry no key. Either `hub.version` publishes the
   public key (the sidecar checks sha256(pk) == pin), or head and list carry it. This is the same gap as T-3704 gap 6,
   and P-15 is the first unit to hit it, so it should settle the form for P-16 too.
2. **Entry key format is unspecified.** 6.2.1 writes `key: "ed25519:<fpr>"`; the AL-18 fingerprint form is
   `ed25519:` + hex SHA-256 of the public key; key cards use `ed25519-pub:<raw hex>` (`key_card.rs:29`). V-REV-1's
   `ed25519:3b6a…da29` could be either. The receiver check (`key in held.entries`) must hash the signer's public key
   the same way. Pin one form; the vectors already commit to `ed25519:` + 64 hex.
3. **The PN-13 effect differs between documents.** 01 R-46.a (4g), 02 11.2.d and the PN-13 row say "no new circuits
   + urgent downgraded". 04 6.2.4 and AL-22 say "allow-list narrows to the own project's verified roster". Circuits
   do not exist until PH-5. R-63.a (CR-10) already makes the PH-2 default allow-list "its own project only", and 03
   puts a configurable allow-list at PH-4. So in PH-2 the narrowing changes nothing. E-41's "urgent from another
   project's allowed sender" has no such sender unless an allow-list widening exists. The fixture needs a test-only
   widened allow-list, or E-41.4 checks a no-op.
4. **"Head compared on every contact" is not in AL-22.** 02 11.2.b (i), R-46.a (4e) and SI-11 require every
   authenticated hub answer to carry the current head. AL-22 only pulls the head once per tick. For a restore, FCW-96
   (UNVERIFIED_SINCE_START) covers most of this, but the bound becomes "the next tick", not "every contact". Either
   accept that as the reading or add the head to the other answers (wire change).
5. **Status vocabulary.** E-40.4 (a) expects status `ROLLBACK refused, kept seq 7`, but AL-22's status values have no
   ROLLBACK form (rollback becomes `unverified since T (rollback)`). 04 D-4 (c) draws a ROLLBACK state. One must change.
6. **Head size is stated three ways.** 04 6.2.3 says "about 100 bytes", AL-21 says "~250 bytes", and V-HEAD-1's
   canonical form is 151 bytes (before the digest and signature fields). This is cosmetic, but a fixture should not
   assert a size.
7. **"Root-only local socket" has no counterpart in code.** The hub's unix socket admits any same-uid peer, and agents
   run as root (OQ-b A; AR-4: "revocation and dead letters are forgeable locally"). Operator ruling A (2026-11-18)
   for `hub.key_card_put` made the same path "a visible, logged convention, not a barrier". P-15 should state that
   it inherits that ruling, or ask for a new one. Any root agent can call `admin revoke` against any key.
8. **Self-revocation needs the public key and has no vector.** AL-21 step 1 verifies against `F.pub`, "or the key
   card's current key, AL-33". With fingerprints only, the public key has to come from the K-15 card (it exists:
   `key_card.rs`) or from the request itself. The `termlink.revocation-request/1` schema has no test vector and no
   nonce (replay is harmless only because re-revoke is idempotent). Using the card makes B-5 depend on B-6 slice A,
   which the plan does not record (P-15 depends only on P-10).
9. **Mail to a revoked agent.** RK-1 says the sender sees WAITING FOR RECIPIENT with "key revoked, operator action
   needed" and a needs-attention entry. 04 6.2.5 and E-39.4 say only that the mail "waits". No AL tells a sender to
   check the recipient's key against the list, and none adds the reason to the sender's vocabulary. E-39.4 also names
   a new sender record value, "refused-revoked".
10. **Hub-side refusal on post.** 04 6.2.5 says "and at the hub on post, once K-15's key card exists". K-15's
    post-time check now exists (`key_card.rs`, modes off/log/enforce), but no AL specifies a revocation check there.
    Is it in B-5 scope or deferred?
11. **The hub has no attention store.** AL-21 step 7 and its crash table ("a list seq newer than the last attention
    row raises the entry") assume persistent hub attention rows. This is the same gap as T-3704 gap 12; only the
    admin log (`rpc_audit::admin_note`) exists.
12. **Hub-side restore is unaddressed.** If the hub's own `revocations.json` is restored to an older `seq`, every
    sidecar holds a higher seq and raises `revocation-tamper` indefinitely (FCW-94). Nothing specifies recovery: a
    re-sign at `held.seq + 1`, or a runme re-pin. E-40 covers only a sidecar restore.
13. **The list only grows, but it is capped at 1 MiB.** AL-22 invariant (2) has no compaction, and FCW-88 refuses a
    body over 1 MiB, so the list eventually reaches a hard failure (very far off, but unbounded). There is also no
    un-revoke; re-keying goes through a new key card (AL-33).
14. **AL-22 does not check `L.hub_id`.** The pin is per hub, so this is mostly covered, but an explicit
    `hub_id == declared` check is cheap and matches FX-2.
15. **Roll-out ordering and NID.** A sidecar that talks to a hub without `revocation.head` (method not found) takes
    the failed-pull path. It sits UNVERIFIED and then STALE, with hourly attention. The hub must ship first, or
    "method not found" needs its own status ("hub predates P-15") that does not raise attention. The plan's rollback
    ("pull off → PH-1 trust, status says so") implies a sidecar switch. Its default should be stated.
16. **No tracked vectors.** `vectors.py` sits only in a gitignored scratch directory. P-15 should commit the vectors
    (and ideally the generator) as fixtures, because T-3819 (P-16 slice B) reuses AL-20.
17. **E-127 versus the guard cap.** A new daily canary needs one guard retired in the same commit (DP-3, 99 checks /
    34 crons). Otherwise E-127 is carried by the status call (05 section 10 note, YR-3).
18. **Dependencies not yet built.** The sidecar's 30 s tick exists (tick-driven modules: `beat.rs`, `deadline.rs`,
    `liveness.rs`). The signed refusal-notice path (FCW-99) and the hook's second receive check (FCW-200) need
    checking against current code. B-14 (P-24 status) reads `revocation`, so the status field format chosen here
    binds P-24.
