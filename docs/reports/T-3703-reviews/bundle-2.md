# Bundle 2 — T-3703 code review (slices B and C)

## Part 1: operator ruling (2026-11-18, verbatim answer "A")

### 2026-11-18 — Who may call `hub.revoke` while agents run as root on the hub host (operator ruling)
- **Operator words:** "A"
- **Chose:** A, the same stated limit as the 2026-11-18 card-write ruling: `hub.revoke` is unix-socket-only, Execute scope, an intent admin row before and a committed row after, a `key-revoked` needs-attention entry from every sidecar; the operator's route is a runme action. Any same-uid (root) process can call it; that is the AR-4 limit, visible and logged, not a barrier.
- **Why:** matches the accepted card-write limit; a rogue revocation is loud and recoverable by re-keying the agent (new key card); the list keeps its only-grows guarantee.
- **Rejected:** B (an un-revoke would let a stolen key come back and breaks AL-22 invariant 2); C (no operator revocation in PH-2 defeats the purpose).
- **Left open:** replacing root-on-host by the operator key (PH-3, OQ-h); the hub under its own account (Track A).

## Part 2: design note (v1 sections 1-4, v2 section 5, as built section 6)

# T-3703 (arc-011 P-15, B-5) — design note: hub-signed revocation list

Inputs: `T-3703-spec-extract.md` (verbatim spec, 7 E-n, 18 gaps). Prepared 2026-11-18, v1 (before review round 1).

## 1. What exists today (measured)

1. Hub signing key: `crates/termlink-hub/src/hub_identity.rs` (`HubIdentity::signing_key()`, ed25519, 0600 file,
   fingerprint `ed25519:` + hex SHA-256 of the 32-byte public key). `hub.version` offers only the fingerprint.
2. Sidecar pin: `crates/termlink-sidecar/src/hub_key_pin.rs` pins that fingerprint on first verified local-socket
   contact; `verified_hub_key` is the gate hub-signed data must pass. No public key is ever transported.
3. Canonical JSON: `termlink-sidecar/src/wire.rs:165` (`canonical_json`, BTreeMap byte sort, no strict gate) and
   `enckey.rs:223` (a string/integer subset). No RFC 8785 implementation with a strict-JSON gate; no duplicate-key
   detection (serde_json keeps the last duplicate silently).
4. Admin log: `rpc_audit::admin_note` (used by the T-3704 key-card RPCs, row written before the store write).
5. Sidecar receive path: `pull.rs` `pass_locked` → `handle_msg` → `refuse(env, msg, code, why)` (existing refusal
   with notice); the impersonation observer (`observe_impersonation`) already runs once per pass.
6. Urgency: `policy.rs` — urgent means priority ≥ 5 AND the sender on the roster (`Trust::OwnProject`); every other
   would-be-urgent message is downgraded with a DOWNGRADED chain line, never dropped. The roster carries keys and
   (T-3704) optional agent names, but **no project label**.
7. Sidecar tick: `daemon.rs:459` heartbeat interval; the pull pass runs on `poll_ms`.

## 2. Slices (one task, three commits-sized slices, each green on its own)

| Slice | Delivers | E-n |
|---|---|---|
| A | AL-20 in `termlink-protocol` (new module `signed_state`): strict-JSON parse (FCW-88), RFC 8785 serialiser, digest, sign, verify (FCW-89); the 06a vectors committed as a fixture file; a deterministic JSON mutator for ≥ 10^5 cases (no new dependency) | E-42 |
| B | Hub: `revocation.rs` (state file, one writer, admin row first, atomic persist, idempotent re-revoke, start-up reconcile), RPCs `hub.revocation_head`, `hub.revocation_get`, `hub.revoke` (admin, local socket), `hub.revocation_request` (self-revoke); `hub.version` gains `signing_public_key`; CLI `termlink hub revoke` / `revocation show` | E-43, E-167 |
| C | Sidecar: per-30 s pull in the pass, pin check with the public key, states UNVERIFIED_SINCE_START / VERIFIED / STALE, tamper and stale attention, status field `revocation`, receive refusal `revoked` (FCW-99), the pull-time re-check (FCW-200), the stale downgrade, the switch `revocation-pull` | E-39, E-40, E-41, E-127 (via status) |

Order A → B → C (B and C both call A; C needs B's RPCs for the real-hub tests). Mutants per slice, listed in section 4.

## 3. The 18 gaps, settled (numbers as in the extract)

1. **Pin cannot verify.** `hub.version` (and the head) gain `signing_public_key: <64 hex>`. The sidecar requires
   `"ed25519:" + hex(sha256(pk)) == pin` before using `pk`; a mismatch is the existing FCW-83 path. The public key
   travels; trust stays in the pin. P-16 slice B (T-3819) reuses this.
2. **Entry key form.** `key` = `ed25519:` + 64 hex of SHA-256(public key), the AL-18 fingerprint form (the vectors
   already commit to `ed25519:` + 64 hex). The receiver check hashes the signer's public key the same way
   (`wire::fingerprint_of`). Key cards keep `ed25519-pub:<raw>`; conversion is one function, tested both ways.
3. **What the stale rule narrows (DR-8 A).** In PH-2 the roster has no project label, and urgent already needs the
   roster. Settlement: while STALE or UNVERIFIED_SINCE_START, **every** would-be-urgent message is downgraded to
   normal with the existing DOWNGRADED line and reason `revocation list stale`; nothing is dropped and normal
   delivery continues. This is the narrowest allow-list (empty) and stays inside DR-8 A ("narrow, never stop"); it
   is stricter than "own project" only because own project cannot be told apart yet. E-41's "urgent from another
   project's allowed sender" becomes "urgent from a rostered sender" (downgraded after 60:01, urgent at 59:59).
   Re-open when the roster carries projects (P-24 / allow-list unit).
4. **Head on every contact.** Read as: the bound is the next pass with the hub reachable (head pulled each 30 s),
   plus FCW-96 (a started or restored sidecar is unreconciled until its first verified head). No wire change to other
   answers in PH-2. Stated as a limit in status docs; criterion for P-24 if it wants per-contact heads.
5. **Status vocabulary.** `revocation` = `verified seq N at T` | `unverified since T (<why>)` | `stale since T
   (<why>)` | `off (PH-1 trust)` | `hub predates revocation list`. A rollback sets `why` = `rollback refused, kept seq
   N`, which is the E-40.4 (a) text inside the unverified form. D-4's ROLLBACK is that transition, not a separate
   lasting state.
6. **Head size.** Not asserted by any fixture.
7. **"Root-only" admin socket.** Inherits the 2026-11-18 ruling A made for `hub.key_card_put` (T-3704): the write
   RPC is unix-socket-only, needs Execute scope, writes an admin-log row first and raises attention; any same-uid
   (root) agent can call it, which is the architecture's stated AR-4 limit, not a barrier. The operator path is a
   runme action naming the key and reason (added when a revocation is decided; the verb and its fixture ship now).
8. **Self-revocation key and replay.** The request carries `public_key` (64 hex); the hub requires
   `fingerprint(public_key) == key` and the signature by that key over `termlink.revocation-request/1\n<digest>`
   (AL-20). No key-card dependency (B-5 depends only on B-1, as planned). Replay is harmless (idempotent, no
   un-revoke); `at` must be within ±10 min of the hub clock to bound reuse of a captured request against a key
   later re-carded. A vector V-REQ-1 is added to the fixture file (RFC 8032 test key 1).
9. **Mail to a revoked agent.** In scope only as "waits" (E-39.4): the sender's existing WAITING path is unchanged.
   The reason text "key revoked, operator action needed" needs the sender to read the recipient's key against the
   list: deferred as a criterion of P-24 (status/reason vocabulary).
10. **Hub-side refusal on post.** Deferred to P-31 (T-3713) / T-3819, where the key-card check moves to enforce: the
    post check refuses a carded agent whose card key is on the hub's own list. Recorded on T-3713.
11. **No hub attention store.** As in T-3704: the admin-log row is the hub's durable trace; the needs-attention
    entry is raised by every sidecar that installs a list with new entries (`key-revoked`, one per (key, seq)),
    exactly as `key-card-written` works. The AL-21 crash row "persist and attention" is then covered by the sidecars.
12. **Hub-side restore.** Start-up reconcile compares the file's `seq`/`digest` with the last `hub.revoke` admin row
    that recorded them; a file older than the log → the hub refuses `hub.revocation_head` with `REVOCATION_RECONCILE`
    (sidecars see a failed pull, not an empty list) and logs an admin row; recovery is a runme action that re-issues
    the logged entries at `seq = logged + 1` (planned with P-37's drill; until then stated). A missing file with no
    revoke rows is the empty list (seq 0).
13. **Growth.** 1 MiB holds about 3000 entries; stated limit, needs-attention at 50 % of the cap. No un-revoke in
    PH-2 (re-keying goes through a new card).
14. **hub_id.** The sidecar requires `L.hub_id == h.hub_id == cfg.hub_id` (FX-2).
15. **Roll-out.** `method not found` on `hub.revocation_head` → status `hub predates revocation list`, no attention,
    no narrowing (PH-1 trust, said so). Switch `revocation-pull` in the store dir (flag file like delivery):
    default ON; OFF = PH-1 trust, status `off (PH-1 trust)` (the plan's rollback). The hub ships before sidecars.
16. **Vectors tracked.** `crates/termlink-protocol/tests/fixtures/signed-state-vectors.json` (values from 06a
    2.4.c.1 + V-REQ-1) and the generator copied to `scripts/fixtures/t3703-vectors.py`.
17. **E-127.** Carried by the status field (no new canary; guard cap DP-3).
18. **Second receive check (FCW-200).** The hook's pull path re-runs `revocation.check` on each STORED message
    before hand-over; a hit is REFUSED with the notice and removed from the pending set. Located in slice C after
    reading the hand-over code.

## 4. Mutants (FP 3.1.b), all must be killed

| Slice | Mutant | Killed by |
|---|---|---|
| A | verify over raw bytes, not canonical (E-42.5) | V-REV-2, key-order cases |
| A | accept a fraction/exponent number | V-NEG-1 |
| A | keep last duplicate key | V-NEG-2 |
| A | sign/verify over the bare digest | V-NEG-3 |
| A | trust `obj.digest` without recomputing | tampered-entry case |
| B | persist before the admin row (E-167.5) | E-167 log-read-only case |
| B | accept a request signed by another key (E-43) | E-43 second request |
| B | non-idempotent re-revoke (seq +1 again) | replay case |
| B | `seq` not chained (`prev_digest` not set) | chain case |
| C | failed pull treated as empty list (E-41.5) | E-41 revoked key at 60:01 |
| C | accept any `seq` (E-40.5) | E-40 (a) |
| C | skip the receive check (E-39.5) | E-39 |
| C | skip-but-leave-STORED at hand-over (E-39.6) | E-39.6 |
| C | no pin check on the head | wrong-key head case |
| C | stale timer reset on a failed pull | E-41 59:59/60:01 |

## 5. v2 (after review round 1: `T-3703-reviews/r1-dispositions.md`)

Section 3 items 1, 2, 3, 4, 5, 7, 8, 9, 10, 11, 12, 13, 15, 18 are superseded by the dispositions table; in short:

5.1 **Public key** only in `hub.version` (`signing_public_key`); head and list keep the vector shapes.
5.2 **Gap 19 (found building slice A).** The head's `digest` names the list (V-HEAD-1, 151 bytes, includes it), so
AL-20's "drop top-level digest" cannot apply to it. `signed_state::CONTENT_DIGEST_SCHEMAS` lists such schemas; they
carry no self-digest and their signature covers `<schema>\n<recomputed digest>`. Change request to 06a 2.4.
5.3 **DR-8 A literally**: an explicit stale gate in `policy.rs` (urgent needs `Trust::OwnProject` while STALE or
UNVERIFIED_SINCE_START); no extra downgrade of rostered senders.
5.4 **Head every pass, before handling**; `get` only on change; the 30 s PN-3 is met a fortiori.
5.5 **Status vocabulary**: `verified seq N at T` | `unverified since T (<why>)` | `stale since T (<why>)` |
`ROLLBACK refused, kept seq N` | `hub predates revocation list` | `off (PH-1 trust)`; counter `refused_revoked`.
5.6 **Self-revocation** only for a carded key; project and agent from the card.
5.7 **Admin rows**: intent before, committed after (seq, digest, entry); reconcile on committed rows; `reissue`.
5.8 **Hub post check** refuses a revoked card key (`-32046 KEY_REVOKED`) under the card-check mode.
5.9 **Sidecar attention**: one coalesced `key-revoked` per new list, `attention_seq` persisted.
5.10 **Load-time verify** of the held list; the empty list seq 0 is served; method-not-found is a failed pull.
5.11 **Sender side**: WAITING FOR RECIPIENT `key revoked, operator action needed`; `refused-revoked` on a notice.
5.12 **Open for the operator before slice B**: does the same-uid stated limit (ruling A of 2026-11-18 on card writes)
extend to `hub.revoke`, which cannot be undone?

## 6. As built (slices B and C), and where it departs from v2

6.1 **Hub (slice B)**: `crates/termlink-hub/src/revocation.rs`; RPCs `hub.revocation_head`, `hub.revocation_get`
(Observe), `hub.revoke`, `hub.revocation_reissue` (Execute, local socket), `hub.revocation_request` (Interact, signed);
`hub.version` offers `signing_public_key`; the key-card post check refuses a revoked card key (`-32046`); CLI
`termlink hub revocation revoke|show|reissue`; runme action 33 (rows only for ruled revocations; none yet).
6.2 **Sidecar (slice C)**: `crates/termlink-sidecar/src/revocation.rs` (`Tracker`); the pull runs at the start of every
receive pass after `hub.version`, before any message is handled; the receive check refuses a revoked signer (REFUSED
reason `revoked`, signed `REVOKED` notice to the sender, which labels the message `refused-revoked`); stage records
signed by a revoked key are ignored; every verified pull sweeps STORED-not-handed-over mail from revoked keys to
REFUSED (FCW-200), and the deliverer's candidate scan does the same before offering a message.
6.3 **Departure: DR-8 A gate (5.3).** No code change in `policy.rs`: urgent already requires `Trust::OwnProject`
(rostered sender), which is exactly the narrowed allow-list, so a gate would be an equivalent mutant. `revocation::
narrowing()` is exported for the unit that widens the urgent allow-list (it must consult it). The stale state, its
attention entry and status are built and tested (E-41).
6.4 **Departure: sender-side WAITING reason (5.11 first half).** Not built: the sender's sidecar has no binding of a
recipient name to its key (the roster is receiver-side), so it cannot tell that a recipient's key is revoked. Mail to a
revoked agent is not dropped (E-39 test: stored at the recipient). Deferred as a criterion of P-24 (T-3708): the
status/reason vocabulary unit gives the sender the recipient's card key. The second half (`refused-revoked` on a
notice) is built.
6.5 **Hand-over re-check form.** A stored message carries only the 16-hex short sender id, so the hand-over re-check
matches it as a prefix of the held fingerprints: it can only over-refuse (a 64-bit prefix collision), never pass a
revoked key. The receive-time check uses the full fingerprint.
6.6 **Switch.** `revocation-pull` is a store state (`off`); its flip is observed each pass and raises attention.

## Part 3: round-1 design-review dispositions

# T-3703 design review round 1 — dispositions

Seats (isolated, `review-1-independence.json`): claude-code (`review-1-claude-code.md`): 3 WRONG (items 3, 8, 12, 15
counted as 4 with 15), 11 WEAK, 3 SOUND, slicing WEAK; opencode (`review-1-opencode.md`): 0 WRONG, 8 WEAK, 10 SOUND,
slicing SOUND. A = accepted with a change, R = rejected with a reason, D = deferred as a criterion of a named unit.
Every A lands in the design note's section 5 (v2).

| # | Finding | Disp. | Change |
|---|---|---|---|
| 1 | public key in the head breaks V-HEAD-1; unpinned key must be a failed pull with tamper | A | only `hub.version` carries `signing_public_key`; the head keeps the vector shape. A head or list that does not verify under the pinned key is a failed pull with `revocation-tamper` (06 4.7). Found independently while building slice A: the head's `digest` is content, so AL-20 gets a per-schema rule (gap 19, below). |
| 2 | "tested both ways" impossible; V-REV-1's key is a raw public key (all-zero seed), not a hash | A | confirmed (`3b6a27…da29` is the public key of the zero seed). The entry `key` stays the AL-18 fingerprint (`ed25519:` + hex SHA-256 of the key; 6.2.1 says `<fpr>`); the vector's value is opaque example data and verifies the canonical form only. Conversion is one way (key → fingerprint). Change request to 06a: note that V-REV-1's key value is example data. |
| 3 | "own project cannot be told apart" is false: rostered = `Trust::OwnProject` | A (WRONG fixed) | v1 withdrawn. DR-8 A is implemented literally: while STALE or UNVERIFIED_SINCE_START, urgent requires `Trust::OwnProject` as an explicit gate in `policy.rs` (today urgent already requires it, so in PH-2 the effect is the status, the attention entry and the gate itself, which binds any later widening). E-41.3 is run as written with the operator-admitted sender as "another project's allowed sender": urgent from it is downgraded at 59:59 and at 60:01 (no widening exists yet, recorded), the rostered sender stays urgent, the revoked key is refused. Circuit clauses of 11.2.d (no circuits before PH-5): D to the circuit unit. |
| 4 | per-contact head waived without a ruling | A | the head is read at the start of every pull pass, before any message is handled (passes run on `poll_ms`), so every contact compares the current head; `get` only on change. PN-3's 30 s is met a fortiori. Mutant: handle messages before the head. |
| 5 | E-40.4 (a) asserts the literal `ROLLBACK refused, kept seq 7` | A | status value on rollback is exactly `ROLLBACK refused, kept seq N`. P-24 is bound by the vocabulary of section 5.5; recorded on T-3708. |
| 7 | ruling for card writes stretched to an irreversible act; hub "raises attention" it cannot | A + operator | the hub raises nothing itself (item 11); the runme operator action ships with slice B (idempotent, takes key and reason from a decision file, verifies the served seq). Whether the 2026-11-18 ruling A (same-uid callers are a stated limit) extends to `hub.revoke`, which cannot be undone, is put to the operator as one decision before slice B. |
| 8 | any caller can mint keys and self-revoke, filling the list | A (WRONG fixed) | `hub.revocation_request` is accepted only for a key that is the current key of a hub key card (T-3704 slice A, built), project and agent taken from the card, not from the request; the request's `public_key` is inside the signed object (V-REQ-1). The list is then bounded by the carded keys. Dependency B-6 slice A recorded (built). |
| 9 | "waits" vacuous; RK-1 reason and `refused-revoked` unsettled | A | slice C: a sender whose named roster binds the recipient to a revoked key holds the message as WAITING FOR RECIPIENT with reason `key revoked, operator action needed` and one attention entry; a sender receiving a `revoked` refusal notice records `refused-revoked`. |
| 10 | hub-post check deferred although cards exist | A | slice B: the key-card post check refuses (enforce) or logs (log) a carded agent whose card key is on the hub's list, code `-32046 KEY_REVOKED`. Off by default with the card check (P-31 turns it on). |
| 11 | sidecar-raised attention lost, floods, drops start-up check | A | the sidecar keeps `attention_seq`; on install (and at start when `attention_seq < held.seq`) it raises ONE coalesced `key-revoked` entry naming the new entries since `attention_seq`; a fresh sidecar raises one entry for the whole list. With the pull off, status says `off (PH-1 trust)` and the hub's admin rows remain the trace. |
| 12 | admin row before persist leaves the log ahead of the file → hub refuses heads | A (WRONG fixed) | two rows: an intent row before (FCW-91) and a `committed` row after the atomic write carrying `seq`, `digest` and the full entry. Reconcile compares the file with the last committed row only: an intent without commit is a failed call (crash-table row 1 holds). File behind the last commit → `REVOCATION_RECONCILE` on head, plus `termlink hub revocation reissue` (admin) that rebuilds the list from committed rows at `seq = last + 1`; tested. |
| 13 | behaviour at the cap undefined | A | the hub refuses to issue a list whose canonical form would exceed 1 MiB (call fails, intent row says why); with self-revoke bounded by cards the cap is out of attacker reach. |
| 15 | `method not found` → silent PH-1 trust; OFF flag writable without notice | A (WRONG fixed) | method-not-found is a failed pull (counts toward PN-13) with status `hub predates revocation list` and one attention per day; keys on the held list stay refused. A flip of `revocation-pull` (either way) raises an attention entry, observed each pass. |
| 16 | V-NEG-3 full signature | A | the fixture carries every full signature (`signature_without_domain`). |
| 17 | status string is not E-127's reader | D | E-127 is the P-37 drill's / P-24's reader; slice C adds the counter `refused_revoked` to status so a reader has something to count. Criterion on T-3708. |
| 18 | FCW-200 is a placeholder; full receiver checks | A | slice C re-runs the full receiver check (policy, revocation, switch) at hand-over and clears both pending sets; settled after reading the hand-over code, reviewed in round 2 with the code. |
| M1 | stored list trusted on start | A | on load the held list is re-verified under the pin; a failure is tamper, the stored `seq` stays the rollback floor, its keys stay refused, and the next pull fetches the full list. Mutant: skip load verification. |
| M2 | empty list and first contact | A | the hub serves seq 0 (`entries: []`, `prev_digest` 64 zeros), signed; a sidecar with nothing held accepts any verified list (no chain check), then chains. |
| M3 | forwarding (FCW-76) | D | no forward envelope exists in the sidecar yet (AL-14/15 forward-once is not built); criterion of the forward-once unit: check the original sender's key against the held list. |
| M4 | test seams not gated | A | the clock and RPC-failure seams use the existing estate gate (`beat::test_overrides(store.mode())` pattern): inert outside test mode. |
| M5 | ordering and missing mutants | A | head before handling (item 4); mutants added: start VERIFIED from the store, accept a fork, skip `hub_id`. |
| OC 4 | roster restore (11.2.b iv) | R | the roster is static sidecar configuration, not hub state; a restored config is the operator's file. Revisited when the roster becomes hub-held (R-78). |
| 6, 14 | sizes; hub_id | — | SOUND in both seats; hub_id mismatch is a failed pull. |

## Part 4: spec excerpt (verbatim, from the spec extract): AL-21, AL-22, 06 4.7

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

## Part 5: mutant table

# T-3703 mutants (FP 3.1.b)

Runners: `crates/termlink-session/tests/mutants/run-t3703a.py` (slice A), `run-t3703b.py [id ...]` (slice B). Each applies one change, runs the slice's tests, restores the file; a compile error is not a kill.

## Slice A (AL-20, `termlink-session::signed_state`) — 6/6 killed, 2026-11-18

| mutant | what | verdict |
|---|---|---|
| m1-unsorted-members | verify over the parse order, not the canonical order (E-42.5) | killed |
| m2-accept-fraction | a fraction or exponent is not refused as a number (V-NEG-1) | killed |
| m3-keep-duplicate | a duplicate key is kept instead of refused (V-NEG-2) | killed |
| m4-bare-digest | sign and verify over the bare digest, no domain prefix (V-NEG-3) | killed |
| m5-trust-written-digest | the written digest is trusted, not recomputed (FCW-89) | killed |
| m6-range | an integer above 2^53-1 is accepted (V-NEG-4) | killed |

6/6 killed

## Slice B (hub revocation list) — 8/8 killed, 2026-11-18

First run 7/8: b5 as first written did not compile (not a kill); rewritten to `Ok(_) => return None` and run alone: killed.

| mutant | what | verdict |
|---|---|---|
| b1-no-row-before | hub.revoke runs without its intent row (E-167.5 / FCW-91) | killed |
| b2-unsigned-request | a self-request is not verified under the key it revokes (E-43 / FCW-90) | killed |
| b3-not-idempotent | re-revoking a listed key issues a new seq | killed |
| b4-unchained | prev_digest does not chain the versions | killed |
| b5-no-post-check | the hub post check ignores the revocation list (design 5.8) | killed |
| b6-restore-not-blocked | a list behind the committed log is served (design 5.7) | killed |
| b7-no-card-needed | a self-request without a key card is accepted with the request's names (r1 finding 8) | killed |
| b8-write-failure-ignored | a failed list write is reported as success (FCW-92) | killed |

## Slice C (sidecar revocation pull) — 11/11 killed, 2026-11-18

Runner: `crates/termlink-session/tests/mutants/run-t3703c.py` (sidecar lib revocation tests + `t3703_revocation_sidecar`).

| mutant | what | verdict |
|---|---|---|
| c1-failed-pull-empty | a failed pull is treated as an empty list (E-41.5, 04 6.2.6) | killed |
| c2-any-seq | a lower seq is accepted (E-40.5) | killed |
| c3-no-receive-check | the receiver does not check the held list (E-39.5) | killed |
| c4-leave-stored | a message stored before the revocation is left STORED (E-39.6) | killed |
| c5-no-pin-check | the hub key is used without checking it against the pin | killed |
| c6-stale-reset | a failed pull resets the stale timer (E-41 59:59 / 60:01) | killed |
| c7-start-verified | a started sidecar is VERIFIED from its store (FCW-96) | killed |
| c8-accept-fork | the same seq with another digest is accepted | killed |
| c9-no-hub-id | a head naming another hub is accepted | killed |
| c10-no-load-verify | the stored list is trusted on load (r1 M1) | killed |
| c11-no-chain | a list that does not chain at held + 1 is accepted | killed |


## Part 6: new file crates/termlink-hub/src/revocation.rs (test module omitted; its tests listed)

```rust
//! T-3703 (arc-011 P-15, B-5; AL-21): the hub's signed revocation list.
//!
//! `<runtime_dir>/revocations.json` holds ONE signed object, schema `termlink.revocation/1` (06a 2.4 AL-20 via
//! `termlink_session::signed_state`): `{schema, hub_id, seq, issued_at, prev_digest, entries, digest, signature}`.
//! Absent file = the empty list (seq 0, `prev_digest` 64 zeros), served signed but not written.
//!
//! Rules (design note v2, `docs/reports/T-3703-design.md` section 5; operator ruling A 2026-11-18):
//! 1. One writer under [`LOCK`]. The intent admin row is written by the RPC layer BEFORE the call runs
//!    (`rpc_audit::ADMIN_METHODS`, FCW-91); here the list is built, sealed with the hub signing key, written
//!    atomically (FCW-92: a failed write keeps the previous list and fails the call), and only then is the
//!    `revocation-committed` row written with seq, digest and the full entry.
//! 2. `seq` rises by exactly one per change; `prev_digest` chains every version; re-revoking a listed key is
//!    idempotent (no new seq). There is no un-revoke (lists only grow, AL-22 invariant 2).
//! 3. A self-revocation (`termlink.revocation-request/1`) is accepted only for a key that is the current
//!    signing key of a hub key card (T-3704); project and agent come from the card, never from the request
//!    (r1 finding 8). The request carries its `public_key` inside the signed object, must hash to `key`, must
//!    verify under it, and its `at` must be within [`REQUEST_WINDOW_MS`] of the hub clock.
//! 4. Start-up reconcile compares the file with the last committed row: a file behind it (a restore) makes
//!    `head` refuse with REVOCATION_RECONCILE until [`reissue`]; a file ahead of it (a lost commit row) or an
//!    unreadable file is reported for attention.
//! 5. A list whose canonical form would exceed 1 MiB is not issued (06a FCW-88 would refuse it downstream).

use ed25519_dalek::{SigningKey, VerifyingKey};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use termlink_session::signed_state::{self as ss, J};

pub const LIST_FILE: &str = "revocations.json";
pub const SCHEMA: &str = "termlink.revocation/1";
pub const HEAD_SCHEMA: &str = "termlink.revocation-head/1";
pub const REQUEST_SCHEMA: &str = "termlink.revocation-request/1";
/// A self-revocation request's `at` must lie within this many ms of the hub's clock (design gap 8).
pub const REQUEST_WINDOW_MS: i64 = 10 * 60 * 1000;
/// Longest accepted reason text.
pub const MAX_REASON: usize = 200;
/// Admin-log event written after a list is persisted.
pub const COMMITTED_EVENT: &str = "revocation-committed";

static LOCK: Mutex<()> = Mutex::new(());
/// Directories whose file is behind the admin log (set by [`startup_reconcile`], cleared by [`reissue`]).
static RECONCILE: Mutex<BTreeMap<PathBuf, String>> = Mutex::new(BTreeMap::new());

const ZERO: &str = "0000000000000000000000000000000000000000000000000000000000000000";

pub fn list_path(dir: &Path) -> PathBuf {
    dir.join(LIST_FILE)
}

/// `ed25519:` + 64 lower hex: the AL-18 fingerprint form of a revoked key (design 5.2 / r1 finding 2).
pub fn valid_key(s: &str) -> bool {
    s.strip_prefix("ed25519:").is_some_and(|h| h.len() == 64 && ss::unhex_lower(h).is_some())
}

fn lock() -> std::sync::MutexGuard<'static, ()> {
    LOCK.lock().unwrap_or_else(|p| p.into_inner())
}

fn reconcile_reason(dir: &Path) -> Option<String> {
    RECONCILE.lock().unwrap_or_else(|p| p.into_inner()).get(dir).cloned()
}

fn empty(hub_id: &str) -> J {
    J::Obj(vec![
        ("schema".into(), J::Str(SCHEMA.into())),
        ("hub_id".into(), J::Str(hub_id.into())),
        ("seq".into(), J::Int(0)),
        ("issued_at".into(), J::Int(0)),
        ("prev_digest".into(), J::Str(ZERO.into())),
        ("entries".into(), J::Arr(vec![])),
    ])
}

/// The current list, verified under the hub's own key. Absent file = the empty list, sealed now.
fn load(dir: &Path, hub_id: &str, key: &SigningKey) -> Result<J, String> {
    let bytes = match std::fs::read(list_path(dir)) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return ss::seal(key, empty(hub_id)).map_err(|e| e.to_string());
        }
        Err(e) => return Err(format!("revocation list unreadable: {e}")),
    };
    let list = ss::parse_and_verify(&bytes, &key.verifying_key(), SCHEMA)
        .map_err(|e| format!("revocation list does not verify under the hub key: {e}"))?;
    if list.get("hub_id").and_then(J::as_str) != Some(hub_id) {
        return Err("revocation list names another hub".into());
    }
    Ok(list)
}

fn seq_of(l: &J) -> i64 {
    l.get("seq").and_then(J::as_int).unwrap_or(-1)
}

fn digest_of(l: &J) -> String {
    l.get("digest").and_then(J::as_str).unwrap_or("").to_string()
}

fn entries_of(l: &J) -> Vec<J> {
    match l.get("entries") {
        Some(J::Arr(a)) => a.clone(),
        _ => vec![],
    }
}

fn entry_key(e: &J) -> Option<&str> {
    e.get("key").and_then(J::as_str)
}

/// The text written to disk and served: the sealed object, members in canonical order.
pub fn text_of(l: &J) -> String {
    serde_json::to_string(&l.to_serde()).unwrap_or_default()
}

fn atomic_write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let tmp = dir.join(format!(".{LIST_FILE}.tmp.{}", std::process::id()));
    {
        let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o644).open(&tmp)?;
        f.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o644))?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    if let Ok(d) = std::fs::File::open(dir) {
        let _ = d.sync_all();
    }
    Ok(())
}

/// Build, seal, size-check and persist the next version. `log` writes the committed row afterwards.
#[allow(clippy::too_many_arguments)]
fn issue(
    dir: &Path,
    hub_id: &str,
    key: &SigningKey,
    cur: &J,
    entries: Vec<J>,
    now_ms: u64,
    log: impl FnOnce(&J, &[J]) -> Result<(), String>,
    added: &[J],
) -> Result<J, String> {
    let next = J::Obj(vec![
        ("schema".into(), J::Str(SCHEMA.into())),
        ("hub_id".into(), J::Str(hub_id.into())),
        ("seq".into(), J::Int(seq_of(cur) + 1)),
        ("issued_at".into(), J::Int(now_ms as i64)),
        ("prev_digest".into(), J::Str(digest_of(cur))),
        ("entries".into(), J::Arr(entries)),
    ]);
    let sealed = ss::seal(key, next).map_err(|e| e.to_string())?;
    let text = text_of(&sealed);
    if text.len() > ss::MAX_BYTES {
        return Err(format!("the new list would be {} bytes, over the 1 MiB limit; nothing issued", text.len()));
    }
    atomic_write(&list_path(dir), text.as_bytes())
        .map_err(|e| format!("revocation list not written ({e}); the previous list stays (FCW-92)"))?;
    // The list is in force. A lost committed row is reported at the next start (file ahead of the log).
    if let Err(e) = log(&sealed, added) {
        tracing::error!(error = %e, "T-3703: revocation-committed row not written");
    }
    Ok(sealed)
}

/// One revocation to add.
#[derive(Debug, Clone)]
pub struct Revoke {
    pub key: String,
    pub project: String,
    pub agent: String,
    pub reason: String,
    /// `operator` (hub.revoke) or `self` (a signed request).
    pub by: &'static str,
    pub event: &'static str,
}

/// The outcome of a revocation call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issued {
    pub seq: i64,
    pub digest: String,
    pub already: bool,
}

fn check_names(r: &Revoke) -> Result<(), String> {
    if !valid_key(&r.key) {
        return Err(format!("key {:?} is not ed25519:<64 lower hex> (the key fingerprint)", r.key));
    }
    if !crate::key_card::valid_name(&r.project) {
        return Err(format!("project {:?} is not a plain name", r.project));
    }
    if !crate::key_card::valid_name(&r.agent) {
        return Err(format!("agent {:?} is not a plain name", r.agent));
    }
    if r.reason.trim().is_empty() || r.reason.chars().count() > MAX_REASON || r.reason.chars().any(char::is_control) {
        return Err(format!("reason must be 1..{MAX_REASON} characters without control characters"));
    }
    Ok(())
}

/// AL-21: add `r` to the list (idempotent). `log(new_list, added)` writes the committed admin row.
pub fn revoke(
    dir: &Path,
    hub_id: &str,
    key: &SigningKey,
    r: &Revoke,
    now_ms: u64,
    log: impl FnOnce(&J, &[J]) -> Result<(), String>,
) -> Result<Issued, String> {
    check_names(r)?;
    let _g = lock();
    if let Some(why) = reconcile_reason(dir) {
        return Err(format!("revocation list needs reconcile first ({why}); run hub.revocation_reissue"));
    }
    let cur = load(dir, hub_id, key)?;
    let mut entries = entries_of(&cur);
    if entries.iter().any(|e| entry_key(e) == Some(r.key.as_str())) {
        return Ok(Issued { seq: seq_of(&cur), digest: digest_of(&cur), already: true });
    }
    let entry = J::Obj(vec![
        ("scope".into(), J::Str("agent_key".into())),
        ("key".into(), J::Str(r.key.clone())),
        ("project".into(), J::Str(r.project.clone())),
        ("agent".into(), J::Str(r.agent.clone())),
        ("reason".into(), J::Str(r.reason.clone())),
        ("revoked_at".into(), J::Int(now_ms as i64)),
        ("by".into(), J::Str(r.by.into())),
        ("event".into(), J::Str(r.event.into())),
    ]);
    entries.push(entry.clone());
    let sealed = issue(dir, hub_id, key, &cur, entries, now_ms, log, &[entry])?;
    Ok(Issued { seq: seq_of(&sealed), digest: digest_of(&sealed), already: false })
}

/// The card lookup a self-revocation needs: `(agent, project)` of the card whose current signing key is `pk`.
pub type CardOf<'a> = &'a dyn Fn(&[u8; 32]) -> Result<Option<(String, String)>, String>;

/// FCW-90: verify a `termlink.revocation-request/1` and turn it into a [`Revoke`] by the card that holds the key.
pub fn check_request(req: &J, now_ms: u64, card_of: CardOf<'_>) -> Result<Revoke, String> {
    let pk_hex = req.get("public_key").and_then(J::as_str).ok_or("request has no public_key")?;
    let pk_bytes: [u8; 32] = ss::unhex_lower(pk_hex)
        .and_then(|b| b.try_into().ok())
        .ok_or("public_key is not 64 lower hex")?;
    let pk = VerifyingKey::from_bytes(&pk_bytes).map_err(|_| "public_key is not an ed25519 key")?;
    let key = req.get("key").and_then(J::as_str).ok_or("request has no key")?;
    if ss::key_fingerprint(&pk) != key {
        return Err("key is not the fingerprint of public_key: a request revokes only the key that signs it".into());
    }
    ss::verify(&pk, req, Some(REQUEST_SCHEMA)).map_err(|e| format!("request not signed by the key it revokes: {e} (FCW-90)"))?;
    let at = req.get("at").and_then(J::as_int).ok_or("request has no integer at")?;
    if (at - now_ms as i64).abs() > REQUEST_WINDOW_MS {
        return Err(format!("request time {at} is more than 10 minutes from the hub clock {now_ms}"));
    }
    let reason = req.get("reason").and_then(J::as_str).unwrap_or("self-revocation").to_string();
    let Some((agent, project)) = card_of(&pk_bytes)? else {
        return Err("no key card holds this key; only a carded key may revoke itself (the operator revokes others)".into());
    };
    Ok(Revoke { key: key.to_string(), project, agent, reason, by: "self", event: "self-revoke" })
}

/// `revocation.head`: `{schema: termlink.revocation-head/1, hub_id, seq, digest}` signed.
pub fn head(dir: &Path, hub_id: &str, key: &SigningKey) -> Result<J, String> {
    let _g = lock();
    if let Some(why) = reconcile_reason(dir) {
        return Err(format!("RECONCILE: {why}"));
    }
    let l = load(dir, hub_id, key)?;
    let h = J::Obj(vec![
        ("schema".into(), J::Str(HEAD_SCHEMA.into())),
        ("hub_id".into(), J::Str(hub_id.into())),
        ("seq".into(), J::Int(seq_of(&l))),
        ("digest".into(), J::Str(digest_of(&l))),
    ]);
    ss::seal(key, h).map_err(|e| e.to_string())
}

/// `revocation.get`: the current list.
pub fn get(dir: &Path, hub_id: &str, key: &SigningKey) -> Result<J, String> {
    let _g = lock();
    if let Some(why) = reconcile_reason(dir) {
        return Err(format!("RECONCILE: {why}"));
    }
    load(dir, hub_id, key)
}

/// The fingerprints on the current list, for the hub's own post check. An unreadable list is an `Err`.
pub fn revoked_keys(dir: &Path, hub_id: &str, key: &SigningKey) -> Result<BTreeSet<String>, String> {
    let _g = lock();
    let l = load(dir, hub_id, key)?;
    Ok(entries_of(&l).iter().filter_map(|e| entry_key(e).map(str::to_string)).collect())
}

/// Committed rows in the admin log, in order: (seq, digest, entries added).
fn committed_rows(admin_log: &Path) -> Vec<(i64, String, Vec<serde_json::Value>)> {
    let Ok(text) = std::fs::read_to_string(admin_log) else { return vec![] };
    text.lines()
        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .filter(|v| v.get("event").and_then(|e| e.as_str()) == Some(COMMITTED_EVENT))
        .filter_map(|v| {
            Some((
                v.get("seq")?.as_i64()?,
                v.get("digest")?.as_str()?.to_string(),
                v.get("added").and_then(|a| a.as_array()).cloned().unwrap_or_default(),
            ))
        })
        .collect()
}

/// Design v2 5.7: compare the file with the last committed row. Returns messages for attention rows; a file
/// behind the log (or unreadable while the log shows commits) blocks `head`/`get`/`revoke` until [`reissue`].
pub fn startup_reconcile(dir: &Path, admin_log: &Path, hub_id: &str, key: &SigningKey) -> Vec<String> {
    let mut out = Vec::new();
    let rows = committed_rows(admin_log);
    let last = rows.last().map(|(s, d, _)| (*s, d.clone()));
    let _g = lock();
    let block = |why: String, out: &mut Vec<String>| {
        RECONCILE.lock().unwrap_or_else(|p| p.into_inner()).insert(dir.to_path_buf(), why.clone());
        out.push(why);
    };
    match (load(dir, hub_id, key), last) {
        (Err(e), Some((s, _))) => block(format!("{e}; the admin log shows seq {s} committed"), &mut out),
        (Err(e), None) => block(e, &mut out),
        (Ok(l), Some((s, d))) => {
            let fs = seq_of(&l);
            if fs < s {
                block(format!("revocation list is at seq {fs}, behind the committed seq {s} (a restore?)"), &mut out);
            } else if fs == s && digest_of(&l) != d {
                block(format!("revocation list seq {fs} has another digest than the committed one (a fork)"), &mut out);
            } else if fs > s {
                out.push(format!("revocation list seq {fs} is ahead of the last committed row (seq {s}): a commit row was lost"));
            }
        }
        (Ok(l), None) if seq_of(&l) > 0 => {
            out.push(format!("revocation list seq {} has no committed row in the admin log", seq_of(&l)))
        }
        (Ok(_), None) => {}
    }
    out
}

/// Rebuild after a restore: every entry in the committed rows and in a readable file, at
/// `seq = max(file, committed) + 1`, chained to the file's digest when readable, else the last committed one.
pub fn reissue(
    dir: &Path,
    admin_log: &Path,
    hub_id: &str,
    key: &SigningKey,
    now_ms: u64,
    log: impl FnOnce(&J, &[J]) -> Result<(), String>,
) -> Result<Issued, String> {
    let _g = lock();
    let rows = committed_rows(admin_log);
    let file = load(dir, hub_id, key).ok();
    let mut seen = BTreeSet::new();
    let mut entries = Vec::new();
    let logged = rows.iter().flat_map(|(_, _, a)| a.iter()).filter_map(|v| J::from_serde(v).ok());
    for e in file.iter().flat_map(entries_of).chain(logged) {
        if let Some(k) = entry_key(&e)
            && valid_key(k)
            && seen.insert(k.to_string())
        {
            entries.push(e);
        }
    }
    let top = rows.last().map(|(s, _, _)| *s).unwrap_or(0).max(file.as_ref().map(seq_of).unwrap_or(0));
    let prev = file.as_ref().map(digest_of).or_else(|| rows.last().map(|(_, d, _)| d.clone())).unwrap_or(ZERO.into());
    let base = J::Obj(vec![("seq".into(), J::Int(top)), ("digest".into(), J::Str(prev))]);
    let sealed = issue(dir, hub_id, key, &base, entries, now_ms, log, &[])?;
    RECONCILE.lock().unwrap_or_else(|p| p.into_inner()).remove(dir);
    Ok(Issued { seq: seq_of(&sealed), digest: digest_of(&sealed), already: false })
}

/// The committed row for a new list (written by the RPC layer through `rpc_audit::admin_note`).
pub fn committed_row(list: &J, added: &[J], by: &str) -> serde_json::Value {
    serde_json::json!({
        "event": COMMITTED_EVENT, "attention": true, "by": by,
        "seq": seq_of(list), "digest": digest_of(list),
        "added": added.iter().map(J::to_serde).collect::<Vec<_>>(),
    })
}

```
Unit tests:
- empty_list_is_served_signed_at_seq_0
- revoke_rises_by_one_chains_and_is_idempotent
- bad_names_and_keys_are_refused
- e167_a_list_that_cannot_be_written_leaves_the_previous_one
- a_list_signed_by_another_key_or_hub_is_refused_on_load
- e43_self_request_needs_its_own_signature_a_card_and_a_fresh_time
- restore_behind_the_log_blocks_head_until_reissue
- a_lost_committed_row_is_reported_not_blocking
- a_list_over_one_mib_is_not_issued

## Part 7: new file crates/termlink-sidecar/src/revocation.rs (test module omitted; its tests listed)

```rust
//! T-3703 (arc-011 P-15, B-5; AL-22): the revocation pull, its verification and the stale rule.
//!
//! Rules (06a 2.6; 06 4.7; design note v2 `docs/reports/T-3703-design.md` section 5, r1 dispositions):
//! 1. At the start of every receive pass (before any message is handled) the hub's head is read and verified;
//!    the list is fetched only when the head moved (design 5.4). PN-3's 30 s tick is met a fortiori.
//! 2. Verified means one of the two cases of 06 4.7: the head verifies under the pinned key and equals the held
//!    list, or a higher head whose list verifies, hashes to the head's digest, has the head's seq, chains when
//!    seq = held + 1, and names this hub. **Everything else is a failed pull**, never an empty list: a transport
//!    error, a hub refusal (REVOCATION_RECONCILE included), `method not found` (status `hub predates revocation
//!    list`, design 5.10), a bad signature, an unpinned key, a lower seq (`ROLLBACK refused, kept seq N`), a fork.
//! 3. The hub's public key comes from `hub.version` (`signing_public_key`, design 5.1) and is used only when
//!    `ed25519:` + sha256(key) equals the stored pin (`hub_key_pin::verified_hub_key`).
//! 4. Keys on the held list stay refused whatever happens later (lists only grow); the held list is re-verified
//!    on load (r1 M1): a stored list that fails is tamper, its keys stay refused, its seq stays the floor.
//! 5. A started sidecar is UNVERIFIED_SINCE_START until its first verified pull (FCW-96); PN-13 without one makes
//!    it STALE (FCW-95) with one needs-attention entry. While either holds, `narrowing()` is true (DR-8 A: urgent
//!    from outside the own project's roster is downgraded; in PH-2 urgent already needs the roster, so this is
//!    the gate any later widening must pass).
//! 6. Every list with new entries raises ONE coalesced `key-revoked` entry (design 5.9); STORED mail from a newly
//!    revoked key that is not handed over is REFUSED at once (FCW-200), so no ring names it afterwards.
//! 7. The switch `revocation-pull` (state `off`) returns to PH-1 trust; a flip raises attention (design 5.10).

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use termlink_session::signed_state::{self as ss, J};

use crate::store::{LineExtra, Store};

pub const SCHEMA: &str = "termlink.revocation/1";
pub const HEAD_SCHEMA: &str = "termlink.revocation-head/1";
/// PN-13: the stale limit, equal to PN-1 (1 hour).
pub const PN13: Duration = Duration::from_secs(3600);
/// Chain reason (and notice) for a message refused because its signer is revoked (FCW-99, FCW-200).
pub const REVOKED: &str = "REVOKED";
pub const REASON_REVOKED: &str = "revoked";

const LIST_STATE: &str = "revocation-list";
const HUB_PK_STATE: &str = "revocation-hub-pk";
const ATTENTION_STATE: &str = "revocation-attention-seq";
const STATUS_STATE: &str = "revocation-status";
const SWITCH_STATE: &str = "revocation-pull";
const SWITCH_SEEN_STATE: &str = "revocation-pull-seen";
const REFUSED_COUNT_STATE: &str = "revocation-refused-count";

/// What every other part of the sidecar reads: the refused keys and whether the stale rule narrows.
#[derive(Debug, Clone, Default)]
struct Snap {
    keys: BTreeSet<String>,
    narrowing: bool,
}

static SNAPS: Mutex<BTreeMap<PathBuf, Snap>> = Mutex::new(BTreeMap::new());

fn publish(store: &Store, keys: &BTreeSet<String>, narrowing: bool) {
    SNAPS
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .insert(store.dir().to_path_buf(), Snap { keys: keys.clone(), narrowing });
}

/// FCW-99: the full fingerprint `ed25519:<64 hex>` of a signer is on the held list.
pub fn revoked(store: &Store, fingerprint: &str) -> bool {
    SNAPS.lock().unwrap_or_else(|p| p.into_inner()).get(store.dir()).is_some_and(|s| s.keys.contains(fingerprint))
}

/// The short sender id of a stored message (`from`, the first 16 hex of sha256(key)) matches a held key. Used
/// where only the stored message is at hand (the hand-over re-check); a prefix match can only over-refuse.
pub fn revoked_from(store: &Store, from: &str) -> bool {
    if from.len() < 16 {
        return false;
    }
    SNAPS
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .get(store.dir())
        .is_some_and(|s| s.keys.iter().any(|k| k.strip_prefix("ed25519:").is_some_and(|h| h.starts_with(from))))
}

/// DR-8 A: true while the list is STALE or not yet verified since start.
pub fn narrowing(store: &Store) -> bool {
    SNAPS.lock().unwrap_or_else(|p| p.into_inner()).get(store.dir()).is_none_or(|s| s.narrowing)
}

/// Status (OBS-41): the line the pull last wrote, and the count of messages refused as revoked.
pub fn status(store: &Store) -> Value {
    let line = store.read_state(STATUS_STATE).ok().flatten().unwrap_or_else(|| "unverified since start (no pull yet)".into());
    let n: u64 = store.read_state(REFUSED_COUNT_STATE).ok().flatten().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    serde_json::json!({"revocation": line.trim(), "refused_revoked": n})
}

/// Count one refusal of a revoked key (status `refused_revoked`, for E-127's reader).
pub fn count_refusal(store: &Store) {
    let n: u64 = store.read_state(REFUSED_COUNT_STATE).ok().flatten().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    let _ = store.write_state(REFUSED_COUNT_STATE, &(n + 1).to_string());
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    UnverifiedSinceStart,
    Verified,
    Stale,
    Off,
}

#[derive(Debug, Clone)]
struct Held {
    seq: i64,
    digest: String,
    keys: BTreeSet<String>,
    entries: Vec<J>,
}

fn keys_of(entries: &[J]) -> BTreeSet<String> {
    entries.iter().filter_map(|e| e.get("key").and_then(J::as_str).map(str::to_string)).collect()
}

fn entries_of(l: &J) -> Vec<J> {
    match l.get("entries") {
        Some(J::Arr(a)) => a.clone(),
        _ => vec![],
    }
}

/// The per-hub pull state machine (one per receiver).
pub struct Tracker {
    hub_id: String,
    stale_after: Duration,
    started: Instant,
    last_verified: Option<Instant>,
    state: State,
    held: Option<Held>,
    stale_raised: bool,
}

/// One RPC to the home hub: Ok(result) or Err((code, message)). Code 0 = transport.
pub type Rpc<'a> = &'a dyn Fn(&str) -> Result<Value, (i64, String)>;

impl Tracker {
    /// On start: the last verified list from the store, re-verified (r1 M1); state UNVERIFIED_SINCE_START.
    pub fn new(store: &Store, hub_id: &str, stale_after: Duration) -> Tracker {
        let mut t = Tracker {
            hub_id: hub_id.to_string(),
            stale_after,
            started: Instant::now(),
            last_verified: None,
            state: State::UnverifiedSinceStart,
            held: None,
            stale_raised: false,
        };
        if let Ok(Some(text)) = store.read_state(LIST_STATE) {
            let pk = store.read_state(HUB_PK_STATE).ok().flatten().and_then(|h| ss::verifying_key_of_hex(h.trim()));
            let pin_ok = pk.as_ref().is_some_and(|k| {
                store.read_state(&crate::hub_key_pin::pin_state_name(hub_id)).ok().flatten().map(|p| p.trim().to_string())
                    == Some(ss::key_fingerprint(k))
            });
            match (pk, pin_ok) {
                (Some(k), true) => match ss::parse_and_verify(text.as_bytes(), &k, SCHEMA) {
                    Ok(l) => t.held = Some(held_of(&l)),
                    Err(e) => t.tamper_load(store, &text, &format!("stored list does not verify: {e}")),
                },
                _ => t.tamper_load(store, &text, "stored list cannot be checked against the pinned hub key"),
            }
        }
        t.publish(store);
        t.write_status(store, "unverified since start (no pull yet)");
        t
    }

    /// A stored list that fails verification: tamper; its keys stay refused and its seq stays the floor.
    fn tamper_load(&mut self, store: &Store, text: &str, why: &str) {
        let hour = crate::store::now_ms() / 3_600_000;
        let _ = store.attention_coalesced("revocation-tamper", &hour.to_string(), &format!("revocation list in the store: {why}"));
        if let Ok(l) = ss::parse_strict(text.as_bytes()) {
            let mut h = held_of(&l);
            h.digest = String::new(); // never matches a head: the next pull fetches the full list
            self.held = Some(h);
        }
    }

    pub fn set_stale_after(&mut self, d: Duration) {
        self.stale_after = d;
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn held_seq(&self) -> Option<i64> {
        self.held.as_ref().map(|h| h.seq)
    }

    fn narrowing(&self) -> bool {
        matches!(self.state, State::Stale | State::UnverifiedSinceStart)
    }

    fn publish(&self, store: &Store) {
        let keys = self.held.as_ref().map(|h| h.keys.clone()).unwrap_or_default();
        publish(store, &keys, self.narrowing());
    }

    fn write_status(&self, store: &Store, line: &str) {
        if store.read_state(STATUS_STATE).ok().flatten().as_deref().map(str::trim) != Some(line) {
            let _ = store.write_state(STATUS_STATE, line);
        }
    }

    fn since(&self) -> Instant {
        self.last_verified.unwrap_or(self.started)
    }

    /// One pull. `version` is this pass's `hub.version` answer (already hub-id checked).
    pub fn tick(&mut self, store: &Store, version: &Value, rpc: Rpc<'_>, now: Instant, sender: Option<&crate::send::Sender>) {
        // the switch (design 5.10): off = PH-1 trust, said so; a flip either way raises attention
        let off = store.read_state(SWITCH_STATE).ok().flatten().is_some_and(|s| s.trim() == "off");
        let seen = store.read_state(SWITCH_SEEN_STATE).ok().flatten().map(|s| s.trim().to_string());
        let now_s = if off { "off" } else { "on" };
        if seen.as_deref() != Some(now_s) {
            if seen.is_some() || off {
                let _ = store.attention(
                    "revocation-pull-switched",
                    &format!("revocation pull switched {now_s}{}", if off { ": PH-1 trust, revoked keys are no longer refused" } else { "" }),
                );
            }
            let _ = store.write_state(SWITCH_SEEN_STATE, now_s);
        }
        if off {
            self.state = State::Off;
            publish(store, &BTreeSet::new(), false);
            self.write_status(store, "off (PH-1 trust)");
            return;
        }
        if self.state == State::Off {
            self.state = State::UnverifiedSinceStart;
            self.last_verified = None;
            self.started = now;
        }
        match self.pull(store, version, rpc) {
            Ok(()) => self.verified(store, now, sender),
            Err((why, tamper)) => self.failed(store, now, &why, tamper),
        }
    }

    fn pull(&mut self, store: &Store, version: &Value, rpc: Rpc<'_>) -> Result<(), (String, bool)> {
        let pk_hex = version.get("signing_public_key").and_then(Value::as_str);
        let pinned = crate::hub_key_pin::verified_hub_key(store, &self.hub_id, version);
        let pk = match (pk_hex.and_then(ss::verifying_key_of_hex), pinned) {
            (Some(k), Some(fp)) if ss::key_fingerprint(&k) == fp => k,
            (Some(_), Some(_)) => return Err(("hub public key does not hash to the pinned fingerprint".into(), true)),
            (None, _) => return Err(("hub offers no signing public key".into(), false)),
            (Some(_), None) => return Err(("hub signing key not pinned (or pin mismatch)".into(), true)),
        };
        let h = match rpc("hub.revocation_head") {
            Ok(v) => v,
            Err((-32601, _)) => return Err(("hub predates revocation list".into(), false)),
            Err((0, e)) => return Err((format!("head: {e}"), false)),
            Err((c, e)) => return Err((format!("head: hub refused ({c}): {e}"), false)),
        };
        let text = h.get("text").and_then(Value::as_str).ok_or(("head: no text".to_string(), true))?;
        let head = ss::parse_and_verify(text.as_bytes(), &pk, HEAD_SCHEMA).map_err(|e| (format!("head-signature: {e}"), true))?;
        if head.get("hub_id").and_then(J::as_str) != Some(self.hub_id.as_str()) {
            return Err(("head names another hub".into(), true));
        }
        let hseq = head.get("seq").and_then(J::as_int).ok_or(("head without seq".to_string(), true))?;
        let hdig = head.get("digest").and_then(J::as_str).unwrap_or("").to_string();
        if let Some(held) = &self.held {
            if hseq < held.seq {
                return Err((format!("ROLLBACK refused, kept seq {}", held.seq), true));
            }
            if hseq == held.seq {
                return if hdig == held.digest {
                    Ok(())
                } else if held.digest.is_empty() {
                    self.fetch(store, &pk, hseq, &hdig, rpc) // a held list that failed its load check
                } else {
                    Err((format!("fork: seq {hseq} with another digest"), true))
                };
            }
        }
        self.fetch(store, &pk, hseq, &hdig, rpc)
    }

    fn fetch(&mut self, store: &Store, pk: &ss::VerifyingKey, hseq: i64, hdig: &str, rpc: Rpc<'_>) -> Result<(), (String, bool)> {
        let g = match rpc("hub.revocation_get") {
            Ok(v) => v,
            Err((c, e)) => return Err((format!("get: ({c}) {e}"), false)),
        };
        let text = g.get("text").and_then(Value::as_str).ok_or(("get: no text".to_string(), true))?;
        let l = ss::parse_and_verify(text.as_bytes(), pk, SCHEMA).map_err(|e| (format!("list: {e}"), true))?;
        let lseq = l.get("seq").and_then(J::as_int).unwrap_or(-1);
        let ldig = l.get("digest").and_then(J::as_str).unwrap_or("");
        if lseq != hseq || ldig != hdig {
            return Err(("list does not match its head".into(), true));
        }
        if l.get("hub_id").and_then(J::as_str) != Some(self.hub_id.as_str()) {
            return Err(("list names another hub".into(), true));
        }
        if let Some(held) = &self.held {
            if !held.digest.is_empty()
                && lseq == held.seq + 1
                && l.get("prev_digest").and_then(J::as_str) != Some(held.digest.as_str())
            {
                return Err(("list does not chain to the held one".into(), true));
            }
            // lists only grow: a list that drops a held key is tamper
            let new_keys = keys_of(&entries_of(&l));
            if !held.keys.is_subset(&new_keys) {
                return Err(("list drops a key this sidecar holds as revoked".into(), true));
            }
        }
        store.write_state(LIST_STATE, text).map_err(|e| (format!("list not stored: {e}"), false))?;
        let _ = store.write_state(HUB_PK_STATE, &ss::hex(pk.as_bytes()));
        self.held = Some(held_of(&l));
        Ok(())
    }

    fn verified(&mut self, store: &Store, now: Instant, sender: Option<&crate::send::Sender>) {
        self.last_verified = Some(now);
        self.state = State::Verified;
        self.stale_raised = false;
        self.publish(store);
        let seq = self.held.as_ref().map(|h| h.seq).unwrap_or(0);
        self.write_status(store, &format!("verified seq {seq} at {}", crate::store::now_ms()));
        self.raise_new_entries(store);
        if let Some(h) = &self.held {
            refuse_stored(store, &h.keys, sender);
        }
    }

    /// Design 5.9: one coalesced `key-revoked` entry for the entries above the last raised seq.
    fn raise_new_entries(&self, store: &Store) {
        let Some(h) = &self.held else { return };
        let raised: i64 = store.read_state(ATTENTION_STATE).ok().flatten().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
        if h.seq <= raised {
            return;
        }
        let names: Vec<String> = h
            .entries
            .iter()
            .map(|e| {
                let s = |n: &str| e.get(n).and_then(J::as_str).unwrap_or("?").to_string();
                format!("{}/{} ({}, by {}: {})", s("project"), s("agent"), s("key"), s("by"), s("reason"))
            })
            .collect();
        let detail = format!(
            "hub {} revocation list is at seq {}: {} key(s) revoked: {}; messages signed by them are refused",
            self.hub_id,
            h.seq,
            names.len(),
            names.join("; ")
        );
        if store.attention_coalesced("key-revoked", &h.seq.to_string(), &detail).is_ok() {
            let _ = store.write_state(ATTENTION_STATE, &h.seq.to_string());
        }
    }

    fn failed(&mut self, store: &Store, now: Instant, why: &str, tamper: bool) {
        let hour = crate::store::now_ms() / 3_600_000;
        if tamper {
            let _ = store.attention_coalesced("revocation-tamper", &hour.to_string(), &format!("hub {}: {why}", self.hub_id));
        }
        if why == "hub predates revocation list" {
            let day = hour / 24;
            let _ = store.attention_coalesced(
                "revocation-hub-predates",
                &day.to_string(),
                &format!("hub {} serves no revocation list (it predates T-3703); revoked keys are not known here", self.hub_id),
            );
        }
        let since_ms = crate::store::now_ms().saturating_sub(now.duration_since(self.since()).as_millis() as u64);
        if self.state != State::Stale && now.duration_since(self.since()) > self.stale_after {
            self.state = State::Stale;
            if !self.stale_raised {
                self.stale_raised = true;
                let _ = store.attention(
                    "revocation-stale",
                    &format!(
                        "hub {}: no verified revocation pull since {since_ms} ({why}); urgent mail from outside the own project is downgraded, never dropped",
                        self.hub_id
                    ),
                );
            }
        }
        self.publish(store);
        let line = if why.starts_with("ROLLBACK") || why == "hub predates revocation list" {
            why.to_string()
        } else if self.state == State::Stale {
            format!("stale since {since_ms} ({why})")
        } else {
            format!("unverified since {since_ms} ({why})")
        };
        self.write_status(store, &line);
    }
}

fn held_of(l: &J) -> Held {
    let entries = entries_of(l);
    Held {
        seq: l.get("seq").and_then(J::as_int).unwrap_or(0),
        digest: l.get("digest").and_then(J::as_str).unwrap_or("").to_string(),
        keys: keys_of(&entries),
        entries,
    }
}

/// [`refuse_stored`] with the keys currently held (the hand-over re-check calls this).
pub fn refuse_stored_now(store: &Store, sender: Option<&crate::send::Sender>) -> usize {
    let keys = SNAPS.lock().unwrap_or_else(|p| p.into_inner()).get(store.dir()).map(|s| s.keys.clone()).unwrap_or_default();
    refuse_stored(store, &keys, sender)
}

/// FCW-200 sweep: every STORED message from a revoked key that is not handed over, refused or held open by an
/// attempt is REFUSED (reason `revoked`) at once, with one attention entry and a signed notice to its sender
/// when a sender is at hand. Returns how many were refused.
pub fn refuse_stored(store: &Store, keys: &BTreeSet<String>, sender: Option<&crate::send::Sender>) -> usize {
    if keys.is_empty() {
        return 0;
    }
    let mut n = 0;
    for conv in store.conversations() {
        let Ok(lines) = store.chain_lines(&conv) else { continue };
        for st in lines.iter().filter(|l| l.event == "STORED") {
            let from = st.sender.as_str();
            let is_revoked = from.len() >= 16 && keys.iter().any(|k| k.strip_prefix("ed25519:").is_some_and(|h| h.starts_with(from)));
            if !is_revoked {
                continue;
            }
            let done = |ev: &str| lines.iter().any(|l| l.event == ev && l.msg_id == st.msg_id && l.sender == st.sender);
            if done("HANDED_OVER") || done("REFUSED") {
                continue;
            }
            let extra = LineExtra { reason: Some(REASON_REVOKED.to_owned()), ..Default::default() };
            let (id, sd, dg) = (st.msg_id.clone(), st.sender.clone(), st.digest.clone());
            let wrote = store.append_event_unless(&conv, "REFUSED", &st.msg_id, &st.sender, &st.digest, extra, move |ls| {
                ls.iter().any(|l| l.event == "REFUSED" && l.msg_id == id && l.sender == sd && l.digest == dg)
            });
            if let Ok(Some(_)) = wrote {
                n += 1;
                count_refusal(store);
                let _ = store.attention(
                    "refused-revoked",
                    &format!("{conv}/{} from {from}: stored before its key was revoked; refused, not handed over (FCW-200)", st.msg_id),
                );
                if let (Some(s), Ok((_, payload))) = (sender, store.read_record(&conv, st.seq, &st.msg_id))
                    && let Ok(msg) = crate::wire::parse_payload(&payload)
                    && let Ok(stage) = crate::pull::build_stage(store, s.fingerprint(), s.agent().0, &msg, REVOKED)
                {
                    let _ = s.post_stage(&msg.from_agent, &stage);
                }
            }
        }
    }
    n
}


```
Unit tests:
- empty_list_then_a_revocation_is_verified_installed_and_raised_once
- e40a_rollback_and_fork_are_refused_and_the_list_kept
- e40b_a_restored_store_detects_the_higher_head_and_refuses_the_key
- e41_a_failed_pull_is_not_an_empty_list_and_goes_stale_after_pn13
- the_06_4_7_failure_classes_never_verify
- m1_a_tampered_stored_list_is_tamper_and_refetched
- the_switch_off_is_ph1_trust_said_so_and_a_flip_is_raised

## Part 8: diffs of changed files (from slice A's commit to slice C)

```diff
diff --git a/crates/termlink-hub/src/channel.rs b/crates/termlink-hub/src/channel.rs
index a34b7ea85..966df68b1 100644
--- a/crates/termlink-hub/src/channel.rs
+++ b/crates/termlink-hub/src/channel.rs
@@ -2907,9 +2907,26 @@ pub(crate) fn key_card_gate(
     if mode == Mode::Off {
         return None;
     }
-    let Check::Refuse { code, fcw, agent, reason, carded } = key_card::check_post(dir, topic, msg_type, payload, signer.as_bytes())
-    else {
-        return None;
+    let (code, fcw, agent, reason, carded) = match key_card::check_post(dir, topic, msg_type, payload, signer.as_bytes()) {
+        Check::Refuse { code, fcw, agent, reason, carded } => (code, fcw, agent, reason, carded),
+        Check::NotGated => return None,
+        // T-3703 (design v2 5.8): the card matches; its key must not be on the hub's own revocation list.
+        Check::Pass { agent } => {
+            let ident = crate::hub_identity::current()?;
+            let fp = termlink_session::signed_state::key_fingerprint(signer);
+            match crate::revocation::revoked_keys(dir, &ident.hub_id, ident.signing_key()) {
+                Ok(keys) if !keys.contains(&fp) => return None,
+                Ok(_) => (
+                    error_code::KEY_REVOKED,
+                    "FCW-99",
+                    agent.clone(),
+                    format!("agent {agent:?}'s card key {fp} is on the hub's revocation list"),
+                    true,
+                ),
+                // fail closed like an unreadable card store (T-3704 design v2)
+                Err(e) => (error_code::KEY_REVOKED, "FCW-99", agent, format!("revocation list unreadable: {e}"), true),
+            }
+        }
     };
     let enforced = mode == Mode::Enforce;
     key_card::note_refusal(&agent, signer_fp, enforced, carded);
diff --git a/crates/termlink-hub/src/hub_identity.rs b/crates/termlink-hub/src/hub_identity.rs
index dced03b87..1f5753303 100644
--- a/crates/termlink-hub/src/hub_identity.rs
+++ b/crates/termlink-hub/src/hub_identity.rs
@@ -39,6 +39,12 @@ impl HubIdentity {
         signing_fingerprint(&self.signing)
     }
 
+    /// T-3703 (design v2 5.1): the 32-byte public key as 64 lower hex, offered in `hub.version` so a
+    /// sidecar can verify hub-signed state after checking `sha256(key)` against its pin.
+    pub fn signing_public_key_hex(&self) -> String {
+        hex(self.signing.verifying_key().as_bytes())
+    }
+
     /// The signing key, for hub-signed objects (P-15, P-16). Never serialise it.
     pub fn signing_key(&self) -> &SigningKey {
         &self.signing
diff --git a/crates/termlink-hub/src/router.rs b/crates/termlink-hub/src/router.rs
index 86d8da887..242934d96 100644
--- a/crates/termlink-hub/src/router.rs
+++ b/crates/termlink-hub/src/router.rs
@@ -172,6 +172,11 @@ pub async fn route(req: &Request, peer_addr: Option<&str>) -> Option<RpcResponse
         "hub.key_card_mode" => handle_key_card_mode(id, &req.params, peer_addr),
         "hub.key_card_get" => handle_key_card_get(id, &req.params),
         "hub.key_card_list" => handle_key_card_list(id),
+        "hub.revocation_head" => handle_revocation_read(id, true),
+        "hub.revocation_get" => handle_revocation_read(id, false),
+        "hub.revoke" => handle_revoke(id, &req.params, peer_addr),
+        "hub.revocation_request" => handle_revocation_request(id, &req.params),
+        "hub.revocation_reissue" => handle_revocation_reissue(id, peer_addr),
         "hub.legacy_usage" => handle_hub_legacy_usage(id, &req.params),
         "hub.bus_state" => handle_hub_bus_state(id),
         "hub.governor_status" => handle_hub_governor_status(id),
@@ -974,6 +979,121 @@ fn handle_key_card_mode(id: serde_json::Value, params: &serde_json::Value, peer_
     }
 }
 
+// ---------------------------------------------------------------------------------------------------------------
+// T-3703 (arc-011 P-15, B-5; AL-21): the signed revocation list. Writes are admin RPCs (the intent row is
+// written before dispatch, FCW-91); the committed row follows the persisted list (design v2 5.7).
+
+fn revocation_error(id: serde_json::Value, reason: &str) -> RpcResponse {
+    let code = if reason.starts_with("RECONCILE") {
+        control::error_code::REVOCATION_RECONCILE
+    } else {
+        control::error_code::REVOCATION_REFUSED
+    };
+    ErrorResponse::with_data(id, code, &format!("refused: {reason}"), serde_json::json!({})).into()
+}
+
+fn now_ms() -> u64 {
+    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
+}
+
+fn revocation_commit_log(by: &'static str) -> impl FnOnce(&termlink_session::signed_state::J, &[termlink_session::signed_state::J]) -> Result<(), String> {
+    move |list, added| {
+        crate::rpc_audit::admin_note(crate::revocation::committed_row(list, added, by)).map_err(|e| e.to_string())
+    }
+}
+
+fn handle_revocation_read(id: serde_json::Value, head_only: bool) -> RpcResponse {
+    let Some(ident) = crate::hub_identity::current() else {
+        return revocation_error(id, "the hub identity is not loaded");
+    };
+    let dir = termlink_session::discovery::runtime_dir();
+    let r = if head_only {
+        crate::revocation::head(&dir, &ident.hub_id, ident.signing_key())
+    } else {
+        crate::revocation::get(&dir, &ident.hub_id, ident.signing_key())
+    };
+    match r {
+        // `text` is the exact signed object; a verifier parses it with the strict gate (FCW-88)
+        Ok(o) => Response::success(id, serde_json::json!({"text": crate::revocation::text_of(&o)})).into(),
+        Err(e) => revocation_error(id, &e),
+    }
+}
+
+fn handle_revoke(id: serde_json::Value, params: &serde_json::Value, peer_addr: Option<&str>) -> RpcResponse {
+    if let Some(r) = key_card_local_only(&id, "hub.revoke", peer_addr) {
+        return r;
+    }
+    let Some(ident) = crate::hub_identity::current() else {
+        return revocation_error(id, "the hub identity is not loaded");
+    };
+    let field = |n: &str| params.get(n).and_then(|v| v.as_str()).unwrap_or("").to_string();
+    let r = crate::revocation::Revoke {
+        key: field("key"),
+        project: field("project"),
+        agent: field("agent"),
+        reason: field("reason"),
+        by: "operator",
+        event: "admin-revoke",
+    };
+    let dir = termlink_session::discovery::runtime_dir();
+    match crate::revocation::revoke(&dir, &ident.hub_id, ident.signing_key(), &r, now_ms(), revocation_commit_log("hub.revoke")) {
+        Ok(i) => Response::success(id, serde_json::json!({"ok": true, "seq": i.seq, "digest": i.digest, "already": i.already})).into(),
+        Err(e) => revocation_error(id, &e),
+    }
+}
+
+fn handle_revocation_request(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
+    let Some(ident) = crate::hub_identity::current() else {
+        return revocation_error(id, "the hub identity is not loaded");
+    };
+    // the request travels as text so the strict gate sees duplicates and number forms (FCW-88)
+    let Some(text) = params.get("request").and_then(|v| v.as_str()) else {
+        return revocation_error(id, "missing 'request' (the signed request as JSON text)");
+    };
+    let req = match termlink_session::signed_state::parse_strict(text.as_bytes()) {
+        Ok(r) => r,
+        Err(e) => return revocation_error(id, &format!("request: {e}")),
+    };
+    let dir = termlink_session::discovery::runtime_dir();
+    let cards = |pk: &[u8; 32]| -> Result<Option<(String, String)>, String> {
+        Ok(crate::key_card::list(&dir)?
+            .into_iter()
+            .find(|c| crate::key_card::parse_signing_key(&c.signing_key).as_ref() == Some(pk))
+            .map(|c| (c.agent, c.project)))
+    };
+    let r = match crate::revocation::check_request(&req, now_ms(), &cards) {
+        Ok(r) => r,
+        Err(e) => {
+            let row = serde_json::json!({"event": "revocation-request-refused", "attention": true, "reason": e});
+            let _ = crate::rpc_audit::admin_note(row);
+            return revocation_error(id, &e);
+        }
+    };
+    match crate::revocation::revoke(&dir, &ident.hub_id, ident.signing_key(), &r, now_ms(), revocation_commit_log("hub.revocation_request")) {
+        Ok(i) => Response::success(
+            id,
+            serde_json::json!({"ok": true, "seq": i.seq, "digest": i.digest, "already": i.already, "agent": r.agent, "by": "self"}),
+        )
+        .into(),
+        Err(e) => revocation_error(id, &e),
+    }
+}
+
+fn handle_revocation_reissue(id: serde_json::Value, peer_addr: Option<&str>) -> RpcResponse {
+    if let Some(r) = key_card_local_only(&id, "hub.revocation_reissue", peer_addr) {
+        return r;
+    }
+    let Some(ident) = crate::hub_identity::current() else {
+        return revocation_error(id, "the hub identity is not loaded");
+    };
+    let dir = termlink_session::discovery::runtime_dir();
+    let log = dir.join(crate::rpc_audit::ADMIN_FILE_NAME);
+    match crate::revocation::reissue(&dir, &log, &ident.hub_id, ident.signing_key(), now_ms(), revocation_commit_log("hub.revocation_reissue")) {
+        Ok(i) => Response::success(id, serde_json::json!({"ok": true, "seq": i.seq, "digest": i.digest})).into(),
+        Err(e) => revocation_error(id, &e),
+    }
+}
+
 static HUB_START_MS: std::sync::LazyLock<u64> = std::sync::LazyLock::new(|| {
     std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
 });
@@ -1013,6 +1133,7 @@ fn handle_hub_version(id: serde_json::Value) -> RpcResponse {
     let ident = crate::hub_identity::current();
     let hub_id = ident.as_ref().map(|i| i.hub_id.clone());
     let signing_key = ident.as_ref().map(|i| i.signing_key_fingerprint());
+    let signing_public_key = ident.as_ref().map(|i| i.signing_public_key_hex());
     Response::success(
         id,
         json!({
@@ -1022,6 +1143,7 @@ fn handle_hub_version(id: serde_json::Value) -> RpcResponse {
             "hub_id": hub_id,
             "hub_instance_id": instance,
             "signing_key": signing_key,
+            "signing_public_key": signing_public_key,
         }),
     )
     .into()
diff --git a/crates/termlink-hub/src/rpc_audit.rs b/crates/termlink-hub/src/rpc_audit.rs
index 65632d23d..d2b98bac5 100644
--- a/crates/termlink-hub/src/rpc_audit.rs
+++ b/crates/termlink-hub/src/rpc_audit.rs
@@ -620,6 +620,10 @@ pub const ADMIN_METHODS: &[&str] = &[
     // T-3704 (B-6): card writes and the check setting (admin row before the act, FCW-128).
     "hub.key_card_put",
     "hub.key_card_mode",
+    // T-3703 (B-5, FCW-91): every revocation has its row before it takes effect.
+    "hub.revoke",
+    "hub.revocation_request",
+    "hub.revocation_reissue",
 ];
 
 pub fn is_admin_method(method: &str) -> bool {
diff --git a/crates/termlink-hub/src/server.rs b/crates/termlink-hub/src/server.rs
index bd075e4ec..b2d1d6ef2 100644
--- a/crates/termlink-hub/src/server.rs
+++ b/crates/termlink-hub/src/server.rs
@@ -617,6 +617,19 @@ pub async fn run_with_tcp(
         }
     }
 
+    // T-3703 (design v2 5.7): a revocation list behind the admin log's last committed row (a restore)
+    // blocks head/get until hub.revocation_reissue; a lost commit row or unreadable list raises attention.
+    if let Some(ident) = crate::hub_identity::current() {
+        let log = runtime_dir.join(crate::rpc_audit::ADMIN_FILE_NAME);
+        for m in crate::revocation::startup_reconcile(&runtime_dir, &log, &ident.hub_id, ident.signing_key()) {
+            tracing::warn!(detail = %m, "T-3703: revocation list reconcile");
+            let row = serde_json::json!({"event": "revocation-reconcile", "attention": true, "detail": m});
+            if let Err(e) = crate::rpc_audit::admin_note(row) {
+                tracing::error!(error = %e, "T-3703: revocation reconcile row not written");
+            }
+        }
+    }
+
     // T-2048: Install connection-cap + per-sender rate-limit governors.
     // Reads TERMLINK_MAX_CONNECTIONS / TERMLINK_RATE_LIMIT_PER_SEC from env,
     // falls back to DEFAULT_MAX_CONNECTIONS / DEFAULT_RATE_LIMIT_PER_SEC.
@@ -782,10 +795,15 @@ fn hub_method_scope(method: &str) -> PermissionScope {
         | "hub.bus_state"
         | "hub.governor_status"
         | "hub.key_card_get"
-        | "hub.key_card_list" => PermissionScope::Observe,
+        | "hub.key_card_list"
+        | "hub.revocation_head"
+        | "hub.revocation_get" => PermissionScope::Observe,
 
         // T-3704 (B-6): card writes and the check setting; also local-socket only (router).
-        "hub.key_card_put" | "hub.key_card_mode" => PermissionScope::Execute,
+        // T-3703 (B-5): operator revocation and reissue, same rule (operator ruling A 2026-11-18).
+        "hub.key_card_put" | "hub.key_card_mode" | "hub.revoke" | "hub.revocation_reissue" => PermissionScope::Execute,
+        // T-3703: a self-revocation is authenticated by its own signature and a key card.
+        "hub.revocation_request" => PermissionScope::Interact,
 
         // Observe: read-only channel/bus + agent introspection (T-2267).
         // Before this arm existed the entire channel.* surface fell through
diff --git a/crates/termlink-protocol/src/control.rs b/crates/termlink-protocol/src/control.rs
index 6c5585b7b..56a86a54b 100644
--- a/crates/termlink-protocol/src/control.rs
+++ b/crates/termlink-protocol/src/control.rs
@@ -572,6 +572,17 @@ pub mod error_code {
     /// T-3704: a key-card write was refused (not over the local socket, bad name or key, or a
     /// change of an existing card's key or project without `replace`).
     pub const KEY_CARD_REFUSED: i64 = -32045;
+    /// T-3703 (arc-011 B-5, FCW-99 at the hub): an arc-011 post is signed by a carded agent's key
+    /// that is on the hub's revocation list. `data` carries `{fcw, agent}`.
+    pub const KEY_REVOKED: i64 = -32046;
+    /// T-3703: a revocation call was refused (bad key or names, request not signed by the key it
+    /// revokes, no key card for a self-revocation, request too old, list would exceed 1 MiB, or the
+    /// list could not be written). Nothing changed.
+    pub const REVOCATION_REFUSED: i64 = -32047;
+    /// T-3703 (design v2 5.7): the revocation file is behind the admin log's last committed
+    /// revocation (a restore); the head is not served until `hub.revocation_reissue`. A sidecar
+    /// counts this as a failed pull, never as an empty list.
+    pub const REVOCATION_RECONCILE: i64 = -32048;
 }
 
 /// Default `protocol_version` when the field is missing on the wire.
diff --git a/crates/termlink-sidecar/src/api.rs b/crates/termlink-sidecar/src/api.rs
index 4c12824a9..1d95eca91 100644
--- a/crates/termlink-sidecar/src/api.rs
+++ b/crates/termlink-sidecar/src/api.rs
@@ -1225,7 +1225,10 @@ fn status_at(
             // OQ-2 E: held for an operator, on either side.
             let ev = |e: &str| rows.iter().any(|r| r["msg_id"] == *m && r["stage"] == e);
             if sent_here {
-                if ev(crate::pull::DENIED) {
+                if ev(crate::revocation::REVOKED) {
+                    // T-3703 (E-39.4): the recipient refused it because this sender's key is revoked
+                    v["label"] = json!("refused-revoked");
+                } else if ev(crate::pull::DENIED) {
                     v["label"] = json!(REFUSED_BY_RECIPIENTS_OPERATOR);
                 } else if ev(crate::pull::HELD) && !ev(crate::pull::ADMITTED) && !handed {
                     v["label"] = json!(HELD_BY_RECIPIENT);
@@ -1250,6 +1253,8 @@ fn status_at(
         "hub_id": hub_id.map(Value::from).unwrap_or_else(|| json!("not configured")),
         "hub": crate::hub_key_pin::hub_status(store, hub_id),
         "hub_signing_key": crate::hub_key_pin::status(store, hub_id),
+        // T-3703 (OBS-41, design 5.5): `revocation` and the `refused_revoked` count
+        "revocation": crate::revocation::status(store),
         "stages": STAGES,
         "rows": rows,
         "messages": msgs,
diff --git a/crates/termlink-sidecar/src/deliver.rs b/crates/termlink-sidecar/src/deliver.rs
index a0d41faf3..8374e2787 100644
--- a/crates/termlink-sidecar/src/deliver.rs
+++ b/crates/termlink-sidecar/src/deliver.rs
@@ -393,6 +393,12 @@ impl Deliverer {
                     );
                     continue;
                 }
+                // T-3703 (FCW-200, E-39.6): a key revoked since STORED: REFUSED now, with its notice, and out of
+                // the pending set, so no ring or watchdog line names it afterwards.
+                if crate::revocation::revoked_from(&self.store, &msg.from) {
+                    crate::revocation::refuse_stored_now(&self.store, self.sender.as_deref());
+                    continue;
+                }
                 // Second run of the receiver checks (7.1).
                 let assessment = self.policy.assess(&msg);
                 if assessment.withheld.is_some() {
diff --git a/crates/termlink-sidecar/src/pull.rs b/crates/termlink-sidecar/src/pull.rs
index 411440d85..53f4e1d93 100644
--- a/crates/termlink-sidecar/src/pull.rs
+++ b/crates/termlink-sidecar/src/pull.rs
@@ -102,6 +102,8 @@ struct Inner {
     /// Last time the background loop came round (ms since the epoch); the
     /// component heartbeat (K-3, FP-1 failure 4).
     beat: AtomicU64,
+    /// T-3703 (AL-22): the revocation pull, run at the start of every pass.
+    revocation: Mutex<crate::revocation::Tracker>,
 }
 
 #[derive(Default)]
@@ -176,6 +178,7 @@ impl Receiver {
     ) -> std::io::Result<Receiver> {
         let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build()?;
         let interval = cfg.poll_interval;
+        let revocation = Mutex::new(crate::revocation::Tracker::new(&store, &cfg.declared_hub_id, crate::revocation::PN13));
         let inner = Arc::new(Inner {
             cfg,
             store,
@@ -185,6 +188,7 @@ impl Receiver {
             pass: Mutex::new(PassMemory::default()),
             stop: AtomicBool::new(false),
             beat: AtomicU64::new(crate::store::now_ms()),
+            revocation,
         });
         let thread = interval.map(|iv| {
             let i = inner.clone();
@@ -222,6 +226,16 @@ impl Receiver {
     pub fn topic(&self) -> String {
         inbox_topic(&self.inner.cfg.declared_hub_id, &self.inner.cfg.agent)
     }
+
+    /// T-3703: PN-13 for this receiver (tests shorten it; the daemon keeps 1 hour).
+    pub fn set_revocation_stale_after(&self, d: std::time::Duration) {
+        self.inner.revocation.lock().unwrap_or_else(|p| p.into_inner()).set_stale_after(d);
+    }
+
+    /// T-3703: the revocation pull's state (tests and status).
+    pub fn revocation_state(&self) -> crate::revocation::State {
+        self.inner.revocation.lock().unwrap_or_else(|p| p.into_inner()).state().clone()
+    }
 }
 
 impl Drop for Receiver {
@@ -240,7 +254,7 @@ impl Inner {
             .map_err(|e| e.to_string())
     }
 
-    fn check_pin(&self) -> Result<(), PinError> {
+    fn check_pin(&self) -> Result<Value, PinError> {
         let resp = self.rpc("hub.version", json!({})).map_err(PinError::Unreachable)?;
         let v = unwrap_result(resp).map_err(PinError::Unreachable)?;
         let found = v.get("hub_id").and_then(Value::as_str).map(str::to_owned);
@@ -249,7 +263,7 @@ impl Inner {
             // T-3366 (AL-18): pin / compare the hub's signing key; never holds mail.
             let trusted = matches!(self.cfg.hub, termlink_protocol::TransportAddr::Unix { .. });
             crate::hub_key_pin::observe(&self.store, &self.cfg.declared_hub_id, &v, trusted);
-            Ok(())
+            Ok(v)
         } else {
             Err(PinError::Mismatch { declared: self.cfg.declared_hub_id.clone(), found })
         }
@@ -306,6 +320,19 @@ impl Inner {
         }
     }
 
+    /// T-3703 (AL-22): one revocation pull. A failure never holds mail (the stale rule narrows, never stops).
+    fn pull_revocations(&self, version: &Value) {
+        let rpc = |m: &str| -> Result<Value, (i64, String)> {
+            match self.rpc(m, json!({})) {
+                Err(e) => Err((0, e)),
+                Ok(RpcResponse::Success(s)) => Ok(s.result),
+                Ok(RpcResponse::Error(e)) => Err((e.error.code, e.error.message)),
+            }
+        };
+        let mut t = self.revocation.lock().unwrap_or_else(|p| p.into_inner());
+        t.tick(&self.store, version, &rpc, std::time::Instant::now(), Some(&self.sender));
+    }
+
     fn cursor(&self) -> u64 {
         self.store
             .read_state(CURSOR_STATE)
@@ -341,9 +368,11 @@ impl Inner {
             return PassResult::Drained;
         }
         match self.check_pin() {
-            Ok(()) => {
+            Ok(version) => {
                 mem.held = None;
                 self.observe_impersonation();
+                // T-3703 (design 5.4): the revocation head is compared before any message is handled.
+                self.pull_revocations(&version);
             }
             Err(PinError::Unreachable(_)) => return PassResult::Waiting,
             Err(PinError::Mismatch { declared, found }) => {
@@ -442,6 +471,20 @@ impl Inner {
             Ok((m, _signer)) => m,
             Err(r) => return self.refuse(env, None, wire::refusal_code(&r), &r.to_string()),
         };
+        // T-3703 (FCW-99, E-39): a message signed by a key on the held revocation list is refused, not stored,
+        // and the sender gets a signed REVOKED notice.
+        if let Some(fp) = env.signed.as_ref().and_then(|s| termlink_session::signed_state::fingerprint_of_hex(&s.signer_pk))
+            && crate::revocation::revoked(&self.store, &fp)
+        {
+            let out = self.refuse(env, Some(&msg), crate::revocation::REASON_REVOKED, &format!("signed by revoked key {fp}"));
+            if matches!(out, Outcome::Done) {
+                crate::revocation::count_refusal(&self.store);
+                if let Err(e) = self.report(&msg, crate::revocation::REVOKED) {
+                    return Outcome::Retry(e);
+                }
+            }
+            return out;
+        }
         if !name_ok(&msg.conv) || !name_ok(&msg.msg_id) {
             return self.refuse(env, None, "bad-name", "conv or msg_id is not a valid name");
         }
@@ -619,6 +662,11 @@ impl Inner {
             Ok(s) => s,
             Err(r) => return self.ignore_stage(env, &r.to_string()),
         };
+        if let Some(fp) = env.signed.as_ref().and_then(|s| termlink_session::signed_state::fingerprint_of_hex(&s.signer_pk))
+            && crate::revocation::revoked(&self.store, &fp)
+        {
+            return self.ignore_stage(env, &format!("signed by revoked key {fp} (FCW-99)"));
+        }
         if st.to_agent != self.cfg.agent {
             return self.ignore_stage(env, &format!("addressed to {}, this is {}", st.to_agent, self.cfg.agent));
         }
diff --git a/crates/termlink-sidecar/src/wire.rs b/crates/termlink-sidecar/src/wire.rs
index d18bb6797..63252803d 100644
--- a/crates/termlink-sidecar/src/wire.rs
+++ b/crates/termlink-sidecar/src/wire.rs
@@ -33,7 +33,15 @@ pub const STAGE_ORDER: [&str; 4] = ["RECEIVED", "STORED", "HANDED_OVER", "REPLIE
 /// stored message not handed over, or a handed-over one not answered, while the
 /// recipient's agent is ready. They are not stages of the order above; each is
 /// recorded once.
-pub const NOTICES: [&str; 5] = ["HELD", "ADMITTED", "DENIED", crate::deadline::OVERDUE_HANDOVER, crate::deadline::OVERDUE_ANSWER];
+pub const NOTICES: [&str; 6] = [
+    "HELD",
+    "ADMITTED",
+    "DENIED",
+    crate::deadline::OVERDUE_HANDOVER,
+    crate::deadline::OVERDUE_ANSWER,
+    // T-3703 (FCW-99, FCW-200): the message was refused because its signing key is revoked.
+    crate::revocation::REVOKED,
+];
 
 /// The signed message body (section 5.1).
 #[derive(Debug, Clone, PartialEq, Eq)]
```

## Part 9: real-hub tests

```rust
//! T-3703 (arc-011 P-15, B-5) slice B at L4: a real hub binary in a throwaway runtime dir.
//! E-43 (self-revocation by the old key, a request by another key refused, the traces of every revocation),
//! E-167 (no admin row → no revocation; a list that cannot be written → the previous list stays), the hub-side
//! post check (design v2 5.8, KEY_REVOKED) and the restore → RECONCILE → reissue path (design v2 5.7).

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use assert_cmd::cargo;
use base64::Engine;
use termlink_protocol::control::channel::canonical_sign_bytes;
use termlink_protocol::control::error_code;
use termlink_session::agent_identity::Identity;
use termlink_session::signed_state::{self as ss, J};
use termlink_test_utils::{ProcessGuard, TestDir};

fn cmd(dir: &TestDir, args: &[&str]) -> Command {
    let mut c = termlink_test_utils::termlink_cmd(cargo::cargo_bin!("termlink"), &dir.path);
    c.env("HOME", &dir.path).env("TERMLINK_IDENTITY_DIR", dir.path.join("id")).args(args).stdin(Stdio::null());
    c
}

fn rpc(sock: &Path, method: &str, params: serde_json::Value) -> serde_json::Value {
    let mut s = UnixStream::connect(sock).expect("connect hub");
    s.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let req = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params});
    s.write_all(format!("{req}\n").as_bytes()).unwrap();
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).unwrap();
    serde_json::from_str(&line).expect("json reply")
}

fn start(dir: &TestDir, tag: &str) -> ProcessGuard {
    let log = |n: &str| std::fs::OpenOptions::new().create(true).append(true).open(dir.path.join(n)).unwrap();
    let _ = std::fs::remove_file(dir.path.join("hub.sock"));
    let child = cmd(dir, &["hub", "start"]).stdout(log("hub.out.log")).stderr(log("hub.err.log")).spawn().expect("spawn hub");
    let g = ProcessGuard::new(child, tag);
    let deadline = Instant::now() + Duration::from_secs(20);
    while UnixStream::connect(dir.path.join("hub.sock")).is_err() {
        assert!(Instant::now() < deadline, "{tag}: hub never accepted");
        std::thread::sleep(Duration::from_millis(50));
    }
    g
}

fn code(v: &serde_json::Value) -> Option<i64> {
    v.get("error").and_then(|e| e.get("code")).and_then(|c| c.as_i64())
}

fn hub_key(sock: &Path) -> ss::VerifyingKey {
    let v = rpc(sock, "hub.version", serde_json::json!({}));
    let r = &v["result"];
    let pk = r["signing_public_key"].as_str().expect("hub.version offers signing_public_key (design v2 5.1)");
    let fp = ss::fingerprint_of_hex(pk).unwrap();
    assert_eq!(Some(fp.as_str()), r["signing_key"].as_str(), "sha256(public key) equals the offered fingerprint");
    ss::verifying_key_of_hex(pk).unwrap()
}

/// The signed head and list, verified under the hub key (AL-20).
fn head_seq(sock: &Path, pk: &ss::VerifyingKey) -> i64 {
    let h = rpc(sock, "hub.revocation_head", serde_json::json!({}));
    let text = h["result"]["text"].as_str().unwrap_or_else(|| panic!("head: {h}"));
    let h = ss::parse_and_verify(text.as_bytes(), pk, "termlink.revocation-head/1").unwrap();
    let l = rpc(sock, "hub.revocation_get", serde_json::json!({}));
    let l = ss::parse_and_verify(l["result"]["text"].as_str().unwrap().as_bytes(), pk, "termlink.revocation/1").unwrap();
    assert_eq!(h.get("digest"), l.get("digest"), "the head names the list it heads");
    h.get("seq").and_then(J::as_int).unwrap()
}

fn list_keys(sock: &Path, pk: &ss::VerifyingKey) -> Vec<(String, String, String)> {
    let l = rpc(sock, "hub.revocation_get", serde_json::json!({}));
    let l = ss::parse_and_verify(l["result"]["text"].as_str().unwrap().as_bytes(), pk, "termlink.revocation/1").unwrap();
    let Some(J::Arr(es)) = l.get("entries") else { return vec![] };
    es.iter()
        .map(|e| {
            let s = |n: &str| e.get(n).and_then(J::as_str).unwrap_or("").to_string();
            (s("key"), s("agent"), s("by"))
        })
        .collect()
}

/// A `termlink.revocation-request/1` for `key_of`'s key, signed by `signer`.
fn request(key_of: &Identity, signer: &Identity, at_ms: i64) -> String {
    let pk_hex = key_of.public_key_hex();
    let mut o = J::Obj(vec![
        ("schema".into(), J::Str("termlink.revocation-request/1".into())),
        ("key".into(), J::Str(ss::fingerprint_of_hex(&pk_hex).unwrap())),
        ("public_key".into(), J::Str(pk_hex)),
        ("project".into(), J::Str("claimed".into())),
        ("agent".into(), J::Str("claimed".into())),
        ("reason".into(), J::Str("retired".into())),
        ("at".into(), J::Int(at_ms)),
    ]);
    let d = ss::digest(&o).unwrap();
    o.set("digest", J::Str(d.clone()));
    o.set("signature", J::Str(signer.sign_hex(&ss::signing_input("termlink.revocation-request/1", &d))));
    serde_json::to_string(&o.to_serde()).unwrap()
}

fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64
}

fn post(sock: &Path, key: &Identity, from_agent: &str) -> serde_json::Value {
    let topic = "inbox:0000000000000000/fx-target";
    let msg_type = "sidecar.msg.v1";
    let payload = serde_json::json!({"v": 1, "kind": "msg", "from_agent": from_agent, "body": "hello"}).to_string();
    let ts = 1_800_000_000_000i64;
    let bytes = canonical_sign_bytes(topic, msg_type, payload.as_bytes(), None, ts);
    rpc(
        sock,
        "channel.post",
        serde_json::json!({
            "topic": topic, "msg_type": msg_type,
            "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload.as_bytes()),
            "ts": ts, "sender_id": key.fingerprint(), "sender_pubkey_hex": key.public_key_hex(),
            "signature_hex": key.sign_hex(&bytes),
        }),
    )
}

fn put(dir: &TestDir, agent: &str, key: &Identity) {
    let k = format!("ed25519-pub:{}", key.public_key_hex());
    let o = cmd(dir, &["hub", "key-card", "put", "--agent", agent, "--project", "010-termlink", "--signing-key", &k, "--json"])
        .output()
        .unwrap();
    assert!(o.status.success(), "card put: {}", String::from_utf8_lossy(&o.stderr));
}

fn admin(dir: &TestDir) -> String {
    std::fs::read_to_string(dir.path.join("rpc-admin.jsonl")).unwrap_or_default()
}

#[test]
fn e43_self_revocation_forged_request_operator_revoke_and_the_post_check() {
    let dir = TestDir::new("t3703-rev");
    let _g = start(&dir, "t3703-rev-hub");
    let sock = dir.path.join("hub.sock");
    let pk = hub_key(&sock);
    assert_eq!(head_seq(&sock, &pk), 0, "the empty list is served signed at seq 0");

    let created = rpc(&sock, "channel.create", serde_json::json!({"name": "inbox:0000000000000000/fx-target"}));
    assert!(created.get("result").is_some(), "{created}");
    let a = Identity::generate();
    let b = Identity::generate();
    put(&dir, "fx-a", &a);
    put(&dir, "fx-b", &b);
    assert!(cmd(&dir, &["hub", "key-card", "mode", "enforce"]).output().unwrap().status.success());
    let p0 = post(&sock, &a, "fx-a");
    assert!(p0.get("error").is_none(), "carded, not revoked: stored: {p0}");

    // fx-a revokes its own key (by: self), names from the card, not from the request
    let r = rpc(&sock, "hub.revocation_request", serde_json::json!({"request": request(&a, &a, now_ms())}));
    assert_eq!(r["result"]["seq"], 1, "{r}");
    assert_eq!(r["result"]["agent"], "fx-a");
    let keys = list_keys(&sock, &pk);
    assert_eq!(keys, vec![(ss::fingerprint_of_hex(&a.public_key_hex()).unwrap(), "fx-a".into(), "self".into())]);
    // the three traces: the admin rows (intent before, committed after), the re-signed list, the attention flag
    let log = admin(&dir);
    let intent = log.lines().position(|l| l.contains("\"hub.revocation_request\"") && l.contains("\"intent\""));
    let commit = log.lines().position(|l| l.contains("\"revocation-committed\"") && l.contains("\"seq\":1"));
    assert!(matches!((intent, commit), (Some(i), Some(c)) if i < c), "intent row before committed row:\n{log}");
    assert!(log.lines().any(|l| l.contains("\"revocation-committed\"") && l.contains("\"attention\":true")));
    assert_eq!(head_seq(&sock, &pk), 1);

    // the hub's post check now refuses fx-a's key (design v2 5.8)
    assert_eq!(code(&post(&sock, &a, "fx-a")), Some(error_code::KEY_REVOKED));
    assert!(post(&sock, &b, "fx-b").get("error").is_none(), "fx-b unaffected");

    // E-43 second case: a request naming fx-b's key but signed by another key (fx-a's) is refused
    let forged = rpc(&sock, "hub.revocation_request", serde_json::json!({"request": request(&b, &a, now_ms())}));
    assert_eq!(code(&forged), Some(error_code::REVOCATION_REFUSED), "{forged}");
    assert!(admin(&dir).contains("\"revocation-request-refused\""), "the refused request is logged");
    assert_eq!(head_seq(&sock, &pk), 1, "nothing issued");
    // a stale request by the right key is refused too
    let stale = rpc(&sock, "hub.revocation_request", serde_json::json!({"request": request(&b, &b, now_ms() - 3_600_000)}));
    assert_eq!(code(&stale), Some(error_code::REVOCATION_REFUSED));

    // the operator revokes fx-b through the CLI (runme path); a repeat is idempotent
    let pkb = b.public_key_hex();
    let args = ["hub", "revocation", "revoke", "--public-key", &pkb, "--project", "010-termlink", "--agent", "fx-b", "--reason", "drill", "--json"];
    let o = cmd(&dir, &args).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!((v["seq"].as_i64(), v["already"].as_bool()), (Some(2), Some(false)));
    let v: serde_json::Value = serde_json::from_slice(&cmd(&dir, &args).output().unwrap().stdout).unwrap();
    assert_eq!((v["seq"].as_i64(), v["already"].as_bool()), (Some(2), Some(true)), "re-revoke: no new seq");
    assert_eq!(list_keys(&sock, &pk)[1].2, "operator");
}

#[test]
fn e167_no_admin_row_no_revocation_and_an_unwritable_list_keeps_the_previous_one() {
    let dir = TestDir::new("t3703-e167");
    let mut g = start(&dir, "t3703-e167-hub");
    let sock = dir.path.join("hub.sock");
    let pk = hub_key(&sock);
    let a = Identity::generate();
    let pka = a.public_key_hex();
    let revoke = |agent: &str, pk_hex: &str| {
        cmd(&dir, &["hub", "revocation", "revoke", "--public-key", pk_hex, "--project", "p", "--agent", agent, "--reason", "e167"])
            .output()
            .unwrap()
    };
    // (1) the admin log cannot be written: a directory where the file goes
    let log = dir.path.join("rpc-admin.jsonl");
    let _ = std::fs::remove_file(&log);
    std::fs::create_dir(&log).unwrap();
    std::fs::write(log.join("x"), b"x").unwrap();
    let o = revoke("fx-a", &pka);
    assert!(!o.status.success(), "no admin row, no revocation");
    assert_eq!(head_seq(&sock, &pk), 0, "no new sequence number");
    std::fs::remove_dir_all(&log).unwrap();
    assert!(revoke("fx-a", &pka).status.success());
    assert_eq!(head_seq(&sock, &pk), 1);
    // (2) the new list cannot be written: a directory where the hub's temporary file goes
    let pid = g.child().id();
    let blocker = dir.path.join(format!(".revocations.json.tmp.{pid}"));
    std::fs::create_dir(&blocker).unwrap();
    std::fs::write(blocker.join("x"), b"x").unwrap();
    let b = Identity::generate();
    let o = revoke("fx-b", &b.public_key_hex());
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("FCW-92"), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(head_seq(&sock, &pk), 1, "previous list served");
}

#[test]
fn a_restored_list_is_refused_until_reissue() {
    let dir = TestDir::new("t3703-restore");
    let sock = dir.path.join("hub.sock");
    let (k1, k2) = (Identity::generate(), Identity::generate());
    let backup;
    {
        let _g = start(&dir, "t3703-restore-hub-1");
        let pk = hub_key(&sock);
        let rv = |i: &Identity, agent: &str| {
            let h = i.public_key_hex();
            assert!(cmd(&dir, &["hub", "revocation", "revoke", "--public-key", &h, "--project", "p", "--agent", agent, "--reason", "r"])
                .output()
                .unwrap()
                .status
                .success());
        };
        rv(&k1, "fx-a");
        backup = std::fs::read(dir.path.join("revocations.json")).unwrap();
        rv(&k2, "fx-b");
        assert_eq!(head_seq(&sock, &pk), 2);
    }
    std::fs::write(dir.path.join("revocations.json"), &backup).unwrap(); // the restore
    let _g = start(&dir, "t3703-restore-hub-2");
    let pk = hub_key(&sock);
    let h = rpc(&sock, "hub.revocation_head", serde_json::json!({}));
    assert_eq!(code(&h), Some(error_code::REVOCATION_RECONCILE), "a list behind the log is not served: {h}");
    assert!(admin(&dir).contains("\"revocation-reconcile\""));
    let o = cmd(&dir, &["hub", "revocation", "reissue", "--json"]).output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(head_seq(&sock, &pk), 3, "reissued above every seq a sidecar may hold");
    assert_eq!(list_keys(&sock, &pk).len(), 2, "the entry the restore lost is back");
}

//! T-3703 (arc-011 P-15, B-5) slice C at L4: E-39 on a real hub. After `hub.revoke` of fx-a's key, fx-b's
//! sidecar refuses fx-a's next message within one pass (nothing STORED, a signed REVOKED notice to fx-a, which
//! records it), refuses a message from fx-a stored before the revocation and not yet handed over (FCW-200,
//! E-39.6), raises one `key-revoked` entry, and fx-b's own mail to fx-a is not dropped.

use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::pull::{ReceiveConfig, Receiver};
use termlink_sidecar::send::{SendConfig, Sender};
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::Roster;

fn rpc(e: &Estate, method: &str, params: Value) -> Value {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let resp = rt
        .block_on(termlink_session::client::rpc_call_addr_with_timeout(&e.hub_addr(), method, params, Duration::from_secs(10)))
        .unwrap();
    serde_json::to_value(&resp).unwrap()
}

fn kinds(store: &Store) -> Vec<String> {
    store
        .attention_entries()
        .iter()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .map(|v| v["kind"].as_str().unwrap_or("").to_string())
        .collect()
}

struct Side {
    store: Arc<Store>,
    sender: Arc<Sender>,
    rx: Receiver,
}

fn side(e: &Estate, id: &str, who: &str, peer: &str) -> Side {
    let agent = format!("fx-{who}");
    let store = Arc::new(Store::open(&e.store_root(), &agent).unwrap());
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: agent.clone(),
                instance: format!("inst-{who}"),
                identity: e.identity(who).unwrap(),
                hub: e.hub_addr(),
                declared_hub_id: id.to_string(),
                queue_path: e.root().join(format!("outbox-{who}.sqlite")),
                timeout: Duration::from_secs(10),
                pump_interval: None,
            },
            store.clone(),
        )
        .unwrap(),
    );
    let mut roster = Roster::new();
    roster.add(e.identity(peer).unwrap().verifying_key());
    let rx = Receiver::new(
        ReceiveConfig {
            agent: agent.clone(),
            instance: format!("inst-{who}"),
            hub: e.hub_addr(),
            declared_hub_id: id.to_string(),
            roster: roster.clone(),
            timeout: Duration::from_secs(10),
            page: 50,
            poll_interval: None,
        },
        store.clone(),
        sender.clone(),
        Box::new(BuiltIn::new(roster)),
    )
    .unwrap();
    Side { store, sender, rx }
}

/// Every chain line of `conv` as (event, msg_id, reason).
fn lines(store: &Store, conv: &str) -> Vec<(String, String, Option<String>)> {
    store.chain_lines(conv).unwrap_or_default().into_iter().map(|l| (l.event, l.msg_id, l.reason)).collect()
}

#[test]
fn e39_a_revoked_key_is_refused_within_one_pass_and_stored_mail_from_it_is_refused() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    for a in ["fx-a", "fx-b"] {
        let _ = rpc(&e, "channel.create", json!({"name": format!("inbox:{id}/{a}")}));
    }
    let a = side(&e, &id, "a", "b");
    let b = side(&e, &id, "b", "a");

    // m1 from fx-a is stored at fx-b before any revocation; not handed over
    let m1 = a.sender.send("fx-b", "inst-b", Some("c1"), "before", None).unwrap();
    let _ = b.rx.pass();
    let c1 = lines(&b.store, "c1");
    assert!(c1.iter().any(|(ev, _, _)| ev == "STORED"), "m1 stored: {c1:?}");
    let st = b.rx.revocation_state();
    assert_eq!(st, termlink_sidecar::revocation::State::Verified, "the empty list verified on the first pass");

    // the operator revokes fx-a's key
    let fp = termlink_session::signed_state::key_fingerprint(&e.identity("a").unwrap().verifying_key());
    let r = rpc(&e, "hub.revoke", json!({"key": fp, "project": "p", "agent": "fx-a", "reason": "key exposed"}));
    assert_eq!(r["result"]["seq"], 1, "{r}");

    // fx-a sends again at once; ONE pass of fx-b refuses it (head read before any message is handled)
    let _m2 = a.sender.send("fx-b", "inst-b", Some("c2"), "after", None).unwrap();
    let _ = b.rx.pass();
    let c2 = lines(&b.store, "c2");
    assert!(c2.iter().any(|(ev, _, r)| ev == "REFUSED" && r.as_deref() == Some("revoked")), "m2 refused: {c2:?}");
    assert!(!c2.iter().any(|(ev, _, _)| ev == "STORED"), "nothing STORED from the revoked key: {c2:?}");
    // E-39.6 / FCW-200: m1, stored before the revocation, is refused and leaves the pending set
    let c1 = lines(&b.store, "c1");
    assert!(c1.iter().any(|(ev, _, r)| ev == "REFUSED" && r.as_deref() == Some("revoked")), "m1 refused: {c1:?}");
    let status = termlink_sidecar::revocation::status(&b.store);
    assert!(status["revocation"].as_str().unwrap().starts_with("verified seq 1 at"), "{status}");
    assert_eq!(status["refused_revoked"], 2, "{status}");
    assert_eq!(kinds(&b.store).iter().filter(|k| *k == "key-revoked").count(), 1);

    // fx-a's side: the signed REVOKED notices arrive and are recorded on its chain
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let _ = a.rx.pass();
        let c2a = lines(&a.store, "c2");
        if c2a.iter().any(|(ev, _, _)| ev == "REVOKED") {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "fx-a never recorded the REVOKED notice: {c2a:?}");
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(lines(&a.store, "c1").iter().any(|(ev, id, _)| ev == "REVOKED" && *id == m1.msg_id), "m1's notice too");

    // mail to the revoked agent waits (is not dropped): fx-b -> fx-a is stored at fx-a
    b.sender.send("fx-a", "inst-a", Some("c3"), "to the revoked agent", None).unwrap();
    let _ = a.rx.pass();
    assert!(lines(&a.store, "c3").iter().any(|(ev, _, _)| ev == "STORED"), "mail to fx-a kept");
}
```
