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
