# Interactive agent communication: step 6, pseudo-code, part A: PH-2 hardening

**Task:** T-3355 · **Arc:** arc-011 · **Chain:** arc-011-design, step 6 · **Role:** pseudo-coder
**Status:** DRAFT v0.1, sub-document of `docs/design/interactive-agent-communication-06-pseudo-code.md` ("06"). Read
06 sections 2 and 3 first: identifiers, the algorithm form a..h, clocks, primitives, strict JSON and the seam gate
are defined there and not repeated. The FAILS CLOSED WHEN register for this file is 06 section 6.
**Review link:** none (profile P2.1).

## 0 Version history

| Version | Date | Change | Task |
|---|---|---|---|
| 0.1 | 2026-11-11 | First draft: AL-17..AL-40 (B-1..B-15, B-18, B-21..B-23, B-26 and the task rows P12, P14); test vectors V-REV-1, V-REV-2, V-HEAD-1, V-FWD-1; drawings D-1.8..D-1.11, D-2.5, D-2.6. | T-3355 |

## 1 Scope and inputs

1.1 Inputs of record: 06 section 1 (same bytes, same hashes); nothing else was read for this part.
1.2 Order follows the build table's dependency order (ARC4 22): B-1, B-5 (revocation), B-7 (caps), B-11..B-13 (keys and
blobs), B-14 (status), B-2..B-4, B-6 (hub changes), B-10 (reconciliation), B-8, B-9 (task rows), B-15 (injectors),
B-18, B-26 and the wave actions.

## 2 Algorithms

### 2.1 AL-17 — Hub id: mint once, keep, refuse a silent re-mint (K-9a; ARC4 6.1.1..6.1.3, 6.1.6; B-1; DR-1)

2.1.a **SERVES:** R-61, R-54, R-33; FX-2; TH-3; AFC-20.
2.1.b **PROBLEM:** the id is derived from the TLS fingerprint on every call **[B]** (`tls.rs:48`), absent without
`--tcp`, and changes when the certificate changes. **APPROACH:** read a stored id; mint only into an empty runtime
directory; derive from the certificate once for legacy hubs so no inbox is renamed.
2.1.c **ALGORITHM:**
```text
hub_start(runtime_dir):
 1  prior = exists(hub.secret) or exists(hub.cert.pem) or exists(hub.signing.key) or any topic log
 2  if exists(hub.id):
        s = read(hub.id) stripped of one trailing newline
        if not s matches ^[0-9a-f]{16}$: refuse("hub.id unreadable or malformed")                          # FCW-80
        HUB_ID = s
    elif prior and not exists(hub.id):
        refuse("hub.id missing beside existing state; restore it or re-home (runme)")                      # FCW-79
    else:                                   # an empty runtime directory: first start
        HUB_ID = first16(sha256 fingerprint of hub.cert.pem) if a certificate will exist, else hex(random 64 bits)
        atomic_write(hub.id, HUB_ID + "\n", mode 0644); Err -> refuse, remove the temp file                   # FCW-81
 3  hub.version returns hub_id = HUB_ID (never recomputed), signing_key = "ed25519:" + fpr(signing pub)    # AL-18
legacy migration (one runme action per hub, Δ): a hub with prior state and no hub.id writes
    hub.id = first16(current TLS fingerprint) once, logged, then restarts       # the only path past FCW-79
```
2.1.d **INVARIANTS:** (1) `hub_id` is constant for the life of a runtime directory; (2) a certificate rotation
changes neither `hub_id` nor the signing key; (3) a hub never answers with an id it did not read from `hub.id`.
2.1.e **FAILS CLOSED WHEN:** 2.1.e.1 **FCW-79** [AFC-20; E-36] `hub.id` is missing while the runtime directory holds
prior hub state → the hub refuses to start with the stated message. 2.1.e.2 **FCW-80** [AFC-20; E-36] `hub.id` is
unreadable or not 16 lowercase hex → refuse to start. 2.1.e.3 **FCW-81** [new; ZR-1] the mint cannot be written
atomically → refuse to start, no partial file left.
2.1.f **EDGE CASES:** (1) volatile `/tmp` (PL-021): the whole directory is gone, so the hub mints a new id; senders
refuse it (FX-2) and status names both ids: loud, not silent (E-80 control); (2) altered `hub.id` (edited by root):
accepted as the hub's id; clients pinned to the old id refuse it (FX-2), which is the detection; (3) concurrency: two
hubs started on one runtime directory: the built PID/socket lock refuses the second **[B]**; (4) crash table:

| Crash between | State after restart | Recovery |
|---|---|---|
| temp written and rename | no `hub.id`, a temp file, and (first start) no prior state | the temp is removed; mint again |
| rename and directory fsync | `hub.id` may be missing after power loss | first start: mint again (no prior state yet); otherwise FCW-79 asks the operator |

2.1.g **EXECUTION SEMANTICS:** once per runtime directory. 2.1.h **CRYPTOGRAPHY:** SHA-256 of the DER certificate for
legacy ids (as `tls.rs` today **[B]**); otherwise 64 bits from the OS CSPRNG.
**OBS:** OBS-34 `hub.version` reply; OBS-35 the refusal text in the hub journal.

### 2.2 AL-18 — Hub signing key and the sidecar's pin (K-9a; ARC4 6.1.4, 6.1.5; B-1; DR-2)

2.2.a **SERVES:** R-61, R-75; SI-11, SI-25; TH-26, TH-58; AFC-20; AR-1.
2.2.b **PROBLEM:** the revocation list, dead letters and (later) roster records need a signer that is not the TLS key
and a verifier that notices a swap. **APPROACH:** an ed25519 key persisted like the secret; each sidecar pins its
fingerprint at first verified contact (TOFU, DR-2) and refuses a different one.
2.2.c **ALGORITHM:**
```text
hub_start: if not exists(hub.signing.key): (first start only, same rule as AL-17 step 2) generate ed25519; atomic_write 0600
           k = read(hub.signing.key); require mode 0600, owner = hub user, 32-byte seed      else refuse            # FCW-82
sidecar, on each hub contact that returns hub-signed data or hub.version:
 1  f = hub.version.signing_key
 2  if store.pin(HUB_ID) is none:
        if contact is over the local hub socket or TLS with the pinned certificate (TOFU pin, built [B]):
            store.pin(HUB_ID) = f; admin_row(hub-key-pinned, f); attention("hub signing key pinned", f)
        else: do not pin; treat hub-signed data as unverified                                                       # FCW-84
 3  elif f != store.pin(HUB_ID):
        refuse every hub-signed object; status hub.signing_key = mismatch {pinned, offered};
        attention("hub signing key changed", both fingerprints, coalesce = day)                                     # FCW-83
replacement in PH-2: runme action on the hub host (new key, signed by the old key), then on each sidecar host a runme
action that re-pins with the operator's typed-back fingerprint suffix; both logged (SI-24)
```
2.2.d **INVARIANTS:** (1) the private key never leaves the hub's runtime directory and is never printed (SI-25);
(2) a sidecar accepts hub-signed data only under its pin; (3) a pin changes only by a runme action.
2.2.e **FAILS CLOSED WHEN:** 2.2.e.1 **FCW-82** [AFC-20; E-37] the key file is unreadable, has another mode or owner,
or is malformed → the hub refuses to start. 2.2.e.2 **FCW-83** [AFC-20; E-37] the offered fingerprint differs from the
pin → every hub-signed object is refused; needs-attention names both. 2.2.e.3 **FCW-84** [new; ZR-1] first contact is
neither the local socket nor TLS under the pinned certificate → no pin; hub-signed data stays unverified.
2.2.f **EDGE CASES:** (1) a rogue key at first contact is pinned (AR-1, limit probe E-37); (2) a certificate rotation
does not touch the key; (3) crash: generation and write are one atomic write; a crash leaves no key and the next start
generates (first start only).
2.2.g **EXECUTION SEMANTICS:** none beyond the pin write (idempotent). 2.2.h **CRYPTOGRAPHY:** ed25519 (RFC 8032), the
fingerprint is `ed25519:` + hex SHA-256 of the 32-byte public key **[P]** (ARC4 6.1.4 says `ed25519:<fingerprint>`
without fixing the hash; SHA-256 matches the TLS pin's form; ZR-5).
**OBS:** OBS-36 status `hub.signing_key {pinned, offered, state}`; OBS-37 admin rows `hub-key-pinned`.

### 2.3 AL-19 — Mail-hub declaration check (K-25; V-7; ARC4 6.7.1, 9.10; B-1; DR-7)

2.3.a **SERVES:** R-54, R-61; FX-2; TH-3; BP-8; AFC-21.
2.3.b **PROBLEM:** a sidecar must post only to the hub the project declared. **APPROACH:** compare the project file,
the sidecar's copy and the answering hub at start and every tick; no fallback.
2.3.c **ALGORITHM:**
```text
start:
 1  d1 = .framework.yaml mail_hub {id, address}; d2 = sidecar config mail_hub
    if d1 missing or d2 missing or d1 != d2 (both fields, exact): refuse start, reason names both          # FCW-85
 2  r = hub.version at d1.address (timeout 10 s)
    if Err: start in state HUB_UNVERIFIED: receive nothing, post nothing, status hub.match = unknown           # FCW-87
    elif r.hub_id != d1.id: refuse start; status hub {declared: d1.id, answering: r.hub_id, match: false}       # FCW-86
every tick: repeat 2; a mismatch stops posting and pulling at once and raises attention (fault injector 5)
```
2.3.d **INVARIANTS:** no message is posted to or pulled from a hub whose `hub_id` is not the declared one.
2.3.e **FAILS CLOSED WHEN:** 2.3.e.1 **FCW-85** [AFC-21; E-38] the declaration is missing in either place, or the two
differ → refuse to start. 2.3.e.2 **FCW-86** [AFC-21; E-38] the hub at the declared address answers another `hub_id` →
refuse (start) or stop (tick). 2.3.e.3 **FCW-87** [AFC-21; ZR-1] the hub cannot be reached at start → the sidecar
runs without posting or pulling until the id is verified.
2.3.f **EDGE CASES:** (1) the declaration names the wrong hub on purpose (who wrote the file): not detectable (ARC4 15
K-25); (2) DNS or address changes: the id check is what binds; (3) crash: stateless.
2.3.g **EXECUTION SEMANTICS:** none. 2.3.h **CRYPTOGRAPHY:** none beyond the TLS pin of the transport **[B]**.
**OBS:** OBS-38 status `hub {id, declared, match}` each tick.

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

### 2.7 AL-23 — Receive admission: the order of checks before STORED (K-11, K-9b, K-15; ARC4 6.3.1, 6.3.2, 6.3.4, 6.2.5, 6.4.4; B-5, B-6, B-7, B-12)

2.7.a **SERVES:** SI-3, SI-5, SI-11, SI-15, SI-19, SI-1 (ids), SI-27, R-56; TH-28, TH-29, TH-30, TH-36; AFC-3,
AFC-22, AFC-23, AFC-25, AFC-26, AFC-33.
2.7.b **PROBLEM:** a message is either STORED (a promise) or refused with a reason; floods wait at the sender; nothing
is dropped. **APPROACH:** a fixed order: cheap structural checks first, then authenticity, then limits, then the blob,
then durability; only the last step reports STORED.
2.7.c **ALGORITHM:**
```text
admit(raw, topic) at the receiver:                     # the pull loop [B]; every refusal is recorded and noticed
 1  version: unknown wire version -> refuse "unsupported version" [B]                                         # FCW-97
 2  strict_json; inline payload <= 8 KiB (PN-7) else QUOTA(inline)                                            # FCW-102
 3  m.id matches ^[0-9a-f]{32}$ and the conversation id is a plain name else refuse                            # FCW-100
 4  key = key card / roster entry for m.sender (AL-33); verify(m, topic) [B SI-3] else refuse                  # FCW-98
 5  if revocation.check(m.key): refuse "revoked", signed notice to the sender (AL-22)                          # FCW-99
 6  if m.hop > 3: refuse "hop limit" with a notice (V-4)                                                       # FCW-101
 7  sequence window and equivocation [B T-3362 / AL-36]
 8  budget = receipts (10% of the aggregate) if m is a receipt
             own_project (20%) if m.sender.project == MY_PROJECT
             general otherwise
    if m is not a receipt and open(m.sender) >= 32: QUOTA(per-sender)                                          # FCW-103
    if budget.count >= its share of 256 or budget.bytes >= its share of 64 MiB: QUOTA(aggregate or reserved)  # FCW-103, FCW-104
 9  if m.blob: fetch and check (AL-27); Err -> refuse                                                          # FCW-114..116
10  store.write(m) with checksum, fsync file and directory; Err (disk full, I/O) -> not STORED                 # FCW-105 [B]
11  chain_append(STORED); hub.stage(m, STORED) [B]; after_stored(m) (06 AL-14)
QUOTA(limit): not STORED; notice(m.sender, QUOTA, {limit}); attention("quota", coalesce = (sender, tick)); the
              hub copy stays unacked, so the sender's ladder keeps it                                           # nothing dropped
```
2.7.d **INVARIANTS:** (1) STORED only after step 10's fsyncs; (2) receipts never count against a sender quota and
general traffic never uses the reserved shares; (3) every refused message has a reason the sender can read; (4) no
message is deleted at the receiver because of a limit.
2.7.e **FAILS CLOSED WHEN:** 2.7.e.1 **FCW-97** [new; E-80, E-100] unknown wire version → refused with a reason.
2.7.e.2 **FCW-98** [AFC-33; E-69, E-93] signature fails or the signer has no card → refused. 2.7.e.3 **FCW-99**
[AFC-22; E-39] the key is on the verified list → refused, signed notice. 2.7.e.4 **FCW-100** [AFC-3; E-3, ZR-1] the
message id is not 32 lowercase hex → refused at the receiver, so a bad id can never reach a doorbell. 2.7.e.5
**FCW-101** [AFC-26; E-53] hop over 3 → refused with a notice. 2.7.e.6 **FCW-102** [AFC-25; E-50] inline over 8 KiB →
not STORED, QUOTA notice. 2.7.e.7 **FCW-103** [AFC-25; E-50, E-51] 32 open from one sender, or the general budget of
256 messages or 64 MiB is full → not STORED, QUOTA notice, coalesced entry. 2.7.e.8 **FCW-104** [AFC-25; E-51] a
reserved share (receipts 10%, own project 20%) is full → the same, for that class only. 2.7.e.9 **FCW-105** [new; E-122]
the store write or a fsync fails → not STORED.
2.7.f **EDGE CASES:** (1) forged sender: step 4; replayed message: identity triple **[B]** and step 7; altered: the
signature covers `hop` and every field (E-53); (2) a dishonest sender ignoring QUOTA: its messages stay on the hub
unacked; the receiver's budget is unaffected; (3) concurrency: steps 8..11 under the store lock, so two pulls cannot
both take the last quota slot; (4) crash table: step 10 is the built store write sequence with its kill points **[B]**;
a crash before the STORED line leaves no STORED claim and the hub copy unacked, so the message is pulled again.
2.7.g **EXECUTION SEMANTICS:** at-most-once STORED per identity triple; at-least-once pull from the hub.
2.7.h **CRYPTOGRAPHY:** the built message signature (ed25519, ARCH) **[B]**.
**OBS:** OBS-44 refusal records with the step; OBS-45 QUOTA notices; OBS-46 status `quota {per_sender, aggregate,
reserved}`.

### 2.8 AL-24 — Sender window, jitter and the early re-send ledger (K-11, K-12; ARC4 6.3.3, 6.3.5; V-4; B-7, B-9)

2.8.a **SERVES:** R-56, R-30; SI-19; TH-29; AFC-5.
2.8.b **PROBLEM:** a sender must back off on QUOTA and never chase faster than its ladder. **APPROACH:** AIMD per
receiver agent; a ledger of re-send times.
2.8.c **ALGORITHM:**
```text
w = 100 (start), per receiver agent
per tick: if receipts advanced and no verified QUOTA arrived: w = min(400, w + 5)
on a QUOTA notice: if verify(notice) fails: ignore, count("quota-notice-unverified")                            # FCW-108
                   else w = max(10, floor(w / 2))
send(m): if open_to(receiver) >= w: m waits in the outbox (durable, SENT local); attention("window full", coalesce = tick)  # FCW-106
resend(m, rung r): earliest = m.last_send + interval(r) x U(0.8, 1.2) drawn once per (m, r) and stored
                   if now < m.last_send + 0.8 x interval(r): refuse, record                                         # FCW-107
```
2.8.d **INVARIANTS:** `10 ≤ w ≤ 400`; no re-send earlier than 0.8 of its rung.
2.8.e **FAILS CLOSED WHEN:** 2.8.e.1 **FCW-106** [AFC-5; E-52] the window is full → the message waits in the outbox.
2.8.e.2 **FCW-107** [new; E-54] a re-send earlier than 0.8 of its rung → refused by the sender's own ledger.
2.8.e.3 **FCW-108** [new; ZR-1] a QUOTA notice that does not verify → ignored (a forged notice cannot throttle a sender).
2.8.f **EDGE CASES:** (1) clock: rungs on `mono()`; the stored jitter draw survives a restart, so a restart does not
re-draw an earlier time; (2) crash: `w` restarts at 100 after a restart (conservative versus 400, aggressive versus
10; the receiver's quota still bounds it); (3) concurrency: one window per receiver under the sender's lock.
2.8.g **EXECUTION SEMANTICS:** the outbox is the at-least-once queue **[B]**. 2.8.h **CRYPTOGRAPHY:** notices are signed
by the receiver **[B]**.
**OBS:** OBS-47 status `send.window` per receiver; OBS-48 refused early re-sends.

