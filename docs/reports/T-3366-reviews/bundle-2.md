# T-3366 close review bundle — round 2 (code under review = deployed commit adf4555e0)

Round 1: one seat FAIL (claude-code); the other seat could not run. Every finding is answered in section 2.

## 1. The task's acceptance criteria and decisions (verbatim)
```
## Acceptance Criteria

### Agent
- [x] Canonical id persisted in the runtime dir and survives cert regeneration (unit e36_certificate_rotation…; L4 t3366_hub_identity rotation test; mutant m1 killed)
- [x] Existing hubs migrate to their current fingerprint-derived id (no inbox renamed) (runme-20261116T073957Z rc=0: local cacc73ea32b121dd, .122 22c19fedafd73da2, .121 1389a831016c4bf1, greenfield aef925c6d2f4ac54, each adopted before the restart and verified unchanged after; live hub.version serves hub_id + signing_key)
- [x] The B-1 specs pass, including the hub signing key and the `hub.version` fields (DR-1 A, DR-2 A) (docs/reports/T-3366-evidence.md; live `hub.version` serves `hub_id` + `signing_key`)
- [x] A re-mint beside existing state is refused (FCW-79 unit + L4 legacy test; mutant m2 killed)
- [x] The unit's fixture suite exits 0 with a PASS line per B-1 E-n and its mutant list is all killed (FP 3.1.b) (evidence table: E-36, E-37, E-126, E-166 PASS; E-38 is P-11's, E-163 not applicable, both stated; 12/12 mutants killed)

## Verification
## Decisions

### 2026-11-12 — plan G-0
- **Chose:** re-scope as proposed in plan 08 v0.3 section 7.1 (approved by the operator 2026-11-12, ruling A with PD-2, recorded in T-3357): this task is unit P-10 (minting, B-1); ending the `sidecar:` alias moves to the T4 decommission P-53 (T-3726), which needs the operator's own approval (NID, G-8).
- **Why:** minting is T1 hardening that signed state depends on; ending an alias switches a working route off, so it cannot ride along in a build task.
- **Rejected:** keeping the alias end here (would decommission implicitly, against the NID rule).

### 2026-11-16 — when to build the auth chain (operator ruling)
- **Chose:** A, "A, start T-3366 after the hand-over" (operator, 2026-11-16): build in this session, right after the T-3807 runme hand-over.
- **Why:** unsigned mail stopped a week ago (measured: 204/204 inbox messages of the last 7 days signed and valid), but a valid signature binds only the key, not the project; T-3366 → T-3704 is the chain that binds them.
- **Rejected:** B next session; C T-3704 first on a temporary key (breaks the security order); D park.

### 2026-11-16 — is a certificate-only directory "existing state"? (operator ruling, review r1 #3a)
- **Chose:** B, "as recommended please" (operator, 2026-11-16): the strict AL-17 step 1 reading. A certificate counts as prior state; a certificate-only directory refuses to start (FCW-79) until `hub adopt-id` gives it the certificate-derived id E-36 expects.
- **Why:** the spec contradicts itself (step 1 lists the certificate; E-36.2/36.4 start a cert-only directory). B fails closed; its only cost is an `adopt-id` step runme already runs. Real legacy hubs always have a secret, and a new hub mints before its certificate exists, so neither is affected.
- **Rejected:** A (keep the E-36 reading: a directory that lost every other file would mint silently, possibly from a rotated certificate); C (change request first, A meanwhile).

### 2026-11-16 — the `sidecar:` forward criterion (operator ruling)
- **Chose:** C, "C, split it into its own task" (operator, 2026-11-16). The criterion "Mail to sidecar: topics forwarded with a sender warning until the alias end release" moves verbatim in intent to **T-3809** (build, arc-011, horizon next), where it is built as a hub forward envelope carrying the original signed message.
- **Why:** one task, one deliverable: alias handling is not id minting, and PD-2 already moved the alias end to its own unit (P-53, T-3726). A plain copy would fail the receiver's signature check (the signature covers the topic), so the forward needs its own design. No `sidecar:` post in 42 days, so the split carries no delivery risk.
- **Rejected:** A build it here (keeps T-3366 and T-3704 waiting on a route idle for six weeks); B warn only (departs from R-71.a); D defer inside T-3366 (same wait as A).


## Decision
```

## 2. Round-1 findings and their dispositions
# T-3366 review round 1 — dispositions

Round 1: claude-code FAIL (`review-1-claude-code.md`); opencode did not run (usage limit, `review-1-opencode.md`).
Every finding is fixed, accepted with a reason, or put to the operator. Nothing is sent round again unanswered.

| # | Finding | Disposition |
|---|---|---|
| 1a | `adopt_legacy` re-keys a migrated hub that lost its key (m5's behaviour through the deploy path) | **Fixed.** adopt-id acts only when BOTH files are absent; id-without-key and key-without-id are refused ("does not re-key" / "does not re-mint"). Tests `adopt_refuses_a_lost_key_*`, `adopt_refuses_a_lost_id_*` (the latter after a certificate rotation); mutants m13, m14. |
| 1b | adopt re-mints a lost `hub.id` from a NEW certificate | **Fixed** by 1a (a key on disk marks a migrated hub). |
| 1c | runme hides `key_generated`, prints only the first line | **Fixed.** Every adopt line is printed. |
| 1d | local "hub.id unchanged" compares the adopted id with itself | **Fixed.** Before adopt: the RUNNING hub's served id is read (`sidecar hub-id`) and passed as `--expect`; after the restart: the NEW hub's served id must equal it. Fixtures: served-id mismatch refused with nothing written; restart that changes the served id FAILED. |
| 1e | no test for id present + key missing | **Fixed** (1a tests). |
| 2a | `observe` turns a store read error into "no pin" and re-pins | **Fixed.** A read error → `Unverified`, needs-attention `hub-signing-key-pin-unreadable`, nothing written. Test `e37_unreadable_pin_is_not_replaced`; mutant m17. |
| 2b | OBS-37 `admin_row(hub-key-pinned)` not implemented | **Accepted, stated.** The sidecar has no admin-row log; the durable record is the needs-attention entry `hub-signing-key-pinned` (store `attention.jsonl`, timed). The evidence map says so. |
| 2c | pin written from `check_pin` at runtime, not only at sidecar start (2.2.f(4)) | **Accepted with reason.** Both writers are threads of the one sidecar process that owns the store (`store.lock` refuses a second sidecar); `write_state` is temp + fsync + rename; the only value either can write on an empty pin is the same offer. The spec's concern (two writers meeting) cannot produce two different pins. |
| 2d | a pinned hub that stops offering its key → silent `NotOffered` | **Fixed.** needs-attention `hub-signing-key-withdrawn` (coalesced per day); `verified_hub_key` refuses. Test `e37_withdrawn_key_is_named_and_refused`. |
| 3a | "prior state" reading resolved by the agent alone, toward the less fail-closed side | **To the operator** (one decision, after this round): the spec contradicts itself (AL-17 step 1 vs E-36.2/36.4). |
| 3b | topic-log test asserts only `is_err()` | **Fixed.** It asserts the FCW-79 message and that no file was written. |
| 3c | mint before the pidfile lock: two concurrent first starts could serve an id that lost | **Fixed.** Publish is no-clobber (`hard_link`, never `rename` over a file); the loser reads the winner's id and key. Stale-temp cleanup removes only temps whose writer pid is dead (the race test found the earlier version deleting a live writer's temp). Tests `e166_write_never_clobbers_*`, `e166_concurrent_first_starts_converge_*` (8 threads); mutant m16. |
| 4a | window between installing the new binary and step 5: a crash/auto-restart starts it without `hub.id` | **Fixed.** Step 4 runs adopt-id with the NEWLY BUILT binary BEFORE installing it; a refusal leaves the old binary in place (FAILED "not installing"). Fixtures: adopt line precedes the install line; refusal keeps the old binary. |
| 4b | key ownership vs the hub's euid not checked | **Fixed.** Local: key uid must equal the hub process's uid (`/proc/<MainPID>`); fleet: same check remotely (`RUNME-ADOPT-UID`). Fixtures for both. |
| 4c | fleet: empty served `id0` skips the comparison; a wrong id is written before the compare | **Fixed.** No served `hub_id` → FAILED, no swap. The remote adopt runs with `--expect <served id>`, so a differing id is refused before anything is written. Fixtures for both. |
| 5a | E-126 needs status `hub {id, declared, match}` and a fake-status control | **Fixed.** `status.hub {id, declared, match}` from the id each check observed. Test `e126_*` with the control (a sidecar declared for another hub reads `match: false`); mutant m18. |
| 5b | E-166 third leg cites unquoted tests | **Fixed in bundle 2** (the test names and their result lines are quoted). |
| 5c | E-37 AR-1 LIMIT run missing; SI-25 scan only of the sidecar store | **Fixed.** `e37_ar1_limit_rogue_key_at_first_contact_is_pinned` records the LIMIT; the real-hub test now runs the hub with `RUST_LOG=info`, keeps its stdout/stderr, and scans every file of the runtime dir (logs included, key file excluded) for the seed. |
| 5d | E-38 and E-163 dropped by the agent's statement | **Clarified.** E-38 is listed under plan row P-11 (T-3363) as that unit's evidence; this unit provides P-11's prerequisite. E-163 concerns `TERMLINK_*_TEST_*` seams; T-3366 adds none (its fault injector is `#[cfg(test)]`). Both stay stated, not claimed. |
| 5e | mutants ran on one commit, the deploy on another | **Fixed in bundle 2.** Mutants, fixtures and suites rerun on the round-2 commit; the redeploy runs that same commit, and the bundle quotes both hashes. |
| 5f | live `hub.version` only for the local hub | **Fixed.** `fleet doctor --json` now reports `hub_id` and `signing_key` per hub; bundle 2 quotes all four. |

## 3. Round-1 review (verbatim)
**T-3366: FAIL — `hub adopt-id`, which runme runs automatically before every hub restart, silently re-mints a lost `hub.id` or signing key beside existing state (the refusal FCW-79 and mutant m5 exist to guarantee), and several PASS rows in the evidence map rest on tests or output the bundle does not contain.**

1. **`hub_identity.rs` `adopt_legacy` plus `runme.sh` `adopt_hub_id`: the refusal is undone by the deploy path.**
   - **Key:** with `hub.id` present and `hub.signing.key` missing, `load_or_mint` refuses, but `adopt_legacy` takes the `None => generate_key(dir)` arm and writes a new key. This is mutant m5's behaviour, live in the migration verb. It also contradicts AL-18 2.2.f(4) (key generated only by the hub process at start) and the PH-2 replacement rule (new key signed by the old).
   - **Id:** a migrated hub that later loses `hub.id` after a certificate rotation gets `first16(new certificate)`, or a random id if the certificate is unreadable to the runme user.
   - **Runme hides it:** `adopt_hub_id` runs on every restart where `exe != HUB_BIN`, not "once". It never checks `id_source` or `key_generated`, and prints only `head -1`, so the "hub.signing.key generated" line is dropped.
   - **Local check is vacuous:** `HUB_ID0` is read after the adopt, so "hub.id unchanged" compares the new id with itself; only the fleet path compares against the served id.
   - **Untested:** no test covers adopt with an id present and the key missing; m7 covers only the id.

2. **`hub_key_pin.rs` `observe`: a store read error becomes a re-pin.**
   - The pin is read with `store.read_state(..).ok().flatten()`, so an I/O error looks like "no pin". On the trusted path the `None =>` arm then writes the offered key over the existing pin, breaking 2.2.d(3) and FCW-83. `verified_hub_key` fails closed on the same error; `observe` does not.
   - `admin_row(hub-key-pinned)` (OBS-37) is not implemented and the evidence map does not mention it.
   - The pin is written from `check_pin` in both `send.rs` and `pull.rs` at runtime, not "only by the sidecar's start, before its tick loop" (2.2.f(4)).
   - A hub that stops offering `signing_key` while a pin exists yields `NotOffered` with no attention entry.

3. **Spec readings under Evolution: one is not closed, the others hold.**
   - **"Prior state" (`has_prior_state`):** only half defensible. The spec is self-contradictory: step 1 makes the "certificate present now" branch unreachable, and E-36.3 starts a cert-only directory. But the agent resolved it alone, toward the less fail-closed side, with no change request or operator ruling; it is not under Decisions. A directory holding `hub.cert.pem` and `hub.key.pem` that has lost `hub.secret` and `hub.id` now mints silently.
   - **Topic logs:** the `bus/` location is assumed. The only test creates an empty directory `bus/x` and asserts `is_err()` without checking the message.
   - **Mint before the lock (`server.rs` `prepare_start`):** `load_or_mint` runs before `claim_runtime`, so 2.1.f(3) does not cover the mint. Two concurrent first starts can leave the lock winner serving an id that lost the rename (invariant 3), unless an unquoted earlier check prevents it.
   - **Defensible:** minting before the hub creates its own certificate (CR-6.42), and not holding mail on a key mismatch.

4. **Runme order: safe inside the three restart points, not outside them.**
   - **Covered:** `restart_hub`, `install_hub_unit` and the fleet swap all adopt first and abort on failure.
   - **Window:** the new binary is already at `$HUB_BIN` before step 5 (the log shows sidecars restarted on it first). A hub crash, systemd auto-restart, reboot or runme abort in that window starts the new binary without `hub.id`, and it refuses to start. Nothing in the bundle closes or tests this.
   - **Ownership:** adopt runs as the runme or remote-shell user, while `read_key` requires owner equal to the hub's euid. If the hub runs as a different user, it refuses after the swap (FCW-82). Runme checks only `-s hub.signing.key`.
   - **Fleet compare:** if `fleet_hub_field` returns an empty `id0`, the served-id comparison is skipped. On a mismatch the wrong `hub.id` has already been written and stays on disk.

5. **Evidence map claims the bundle does not back.**
   - **E-126:** the spec asks for status `hub {id, declared, match}` per tick and a fake-status control. Neither is in the diff or tests; the cited runme fixtures and "375 passed" are not quoted.
   - **E-166 third leg:** `fc_8_hub_down_*` and `send_hub_hub_down_*` are named but not quoted.
   - **E-37:** the AR-1 LIMIT run that E-37.5 requires is "not separately run". The SI-25 scan only searches the sidecar store for a seed the sidecar never receives; no hub log, journal or status is scanned. "Every hub-signed object refused" is one gate function with no caller.
   - **AC 5:** no test-run output is quoted, and the mutant "killed" results are a table of assertions. E-38 and E-163 were dropped from plan row P-10 by the agent's own statement, not a ruling.
   - **Commits:** mutants ran on `7ddb462c6`, the fielded binary is `g33e6cd61d`, and nothing ties either to "the close commit", so the deployed binary is not shown to be the reviewed code.
   - **Live check:** `hub.version` is quoted for the local hub only, not the three fleet hubs.

## 4. Spec: AL-17 and AL-18 (06a pseudo-code)
```
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
        HUB_ID = first16(sha256 fingerprint of hub.cert.pem) if hub.cert.pem is present now, else hex(random 64 bits)
                                                     # CR-6.42: decided on what exists at first start, never on a
                                                     # certificate that "will exist"
        atomic_write(hub.id, HUB_ID + "\n", mode 0644); Err -> refuse, remove the temp file                   # FCW-81
 3  hub.version returns hub_id = HUB_ID (never recomputed), signing_key = "ed25519:" + fpr(signing pub)    # AL-18
legacy migration (one runme action per hub, Δ): a hub with prior state and no hub.id writes
    hub.id = first16(current TLS fingerprint) once, logged, then restarts       # the only path past FCW-79
    CR-6.42: a hub with prior state and no certificate (socket-only) writes hub.id = hex(random 64 bits) instead,
    logged with "minted at random: no certificate"; peers see a new id and refuse it loudly (FX-2) until re-pinned
```
2.1.d **INVARIANTS:** (1) `hub_id` is constant for the life of a runtime directory; (2) a certificate rotation
changes neither `hub_id` nor the signing key; (3) a hub never answers with an id it did not read from `hub.id`.
2.1.e **FAILS CLOSED WHEN:** 2.1.e.1 **FCW-79** [AFC-20; E-36] `hub.id` is missing while the runtime directory holds
prior hub state → the hub refuses to start with the stated message. 2.1.e.2 **FCW-80** [AFC-20; E-36] `hub.id` is
unreadable or not 16 lowercase hex → refuse to start. 2.1.e.3 **FCW-81** [new; ZR-1] the mint cannot be written
atomically → refuse to start, no partial file left.
2.1.f **EDGE CASES:** (0) (changed in v0.3 by CR-6.42) the mint at first start uses the certificate only if it is
present at that moment; a socket-only hub, or a legacy hub without a certificate, gets a random id, so no step depends
on a file that may never exist; (1) volatile `/tmp` (PL-021): the whole directory is gone, so the hub mints a new id; senders
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
generates (first start only); (4) concurrency: single writer per object: the key is generated only by the hub
process at its start, before it accepts connections; the pin is written only by the sidecar's start, before its tick
loop runs; a second sidecar on the same store is refused by `store.lock` (06 3.2.a) **[B]**, so no two writers meet.
2.2.g **EXECUTION SEMANTICS:** none beyond the pin write (idempotent). 2.2.h **CRYPTOGRAPHY:** ed25519 (RFC 8032), the
fingerprint is `ed25519:` + hex SHA-256 of the 32-byte public key **[P]** (ARC4 6.1.4 says `ed25519:<fingerprint>`
without fixing the hash; SHA-256 matches the TLS pin's form; ZR-5).
**OBS:** OBS-36 status `hub.signing_key {pinned, offered, state}`; OBS-37 admin rows `hub-key-pinned`.
```

## 5. Spec: evidence E-36, E-37, E-126, E-166
```
**E-36 — Hub id: mint once, keep, refuse a silent re-mint.** Links: R-61, R-54, R-33, FX-2, TH-3, AFC-20. Category: core function. Level: L2 + L4. Tier: 1 and 2. Phase: PH-2 (B-1).
E-36.1 EVIDENCE: a fresh runtime directory mints `hub.id` (fingerprint-derived when a certificate exists, else 64 random bits); an existing id is kept across restarts and certificate rotation; `hub.id` missing beside existing state refuses start with the stated message; a socket-only hub has an id; `hub.version` reads the file, never the fingerprint (ARC4 6.1).
E-36.2 ARRANGE: throwaway hub runtime directories: empty; with cert only (legacy hub); with `hub.secret` and cert but no `hub.id`; with `hub.id` and a regenerated cert; socket-only.
E-36.3 ACT: start the hub in each; call `hub.version`; restart; rotate the certificate; restart.
E-36.4 ASSERT: ids as expected (legacy hub keeps `first16(fingerprint)`, so no inbox renames); the partial-state case exits non-zero with "hub.id missing beside existing state" and writes no file; after rotation id and signing key unchanged; `hub.id` is 16 lowercase hex, mode 0644, written via temp + fsync + rename.
E-36.5 CONTROL: mutant "derive from the fingerprint on each call" fails the rotation case (ARC4 6.1.7).

**E-37 — Hub signing key: persisted, never printed, pinned on first contact.** Links: R-61, R-75, SI-25, SI-11, AR-1, AFC-20. Category: adversarial. Level: L2 + L4. Tier: 1 and 2. Phase: PH-2 (B-1).
E-37.1 EVIDENCE: `hub.signing.key` (0600) persists like the secret; `hub.version` returns only `ed25519:<fingerprint>`; each sidecar pins the fingerprint on first verified contact and refuses a different key with a needs-attention entry naming both (ARC4 6.1.4, 6.1.5).
E-37.2 ARRANGE: throwaway hub; a sidecar with an empty pin; a second hub key prepared.
E-37.3 ACT: first contact; restart hub; swap the key file; contact again.
E-37.4 ASSERT: pin recorded; same key after restart; after the swap every signed hub object is refused and the entry names old and new fingerprints; the private key bytes appear in no log, status or record (SI-25 scan, E-101).
E-37.5 CONTROL: mutant that re-pins silently on mismatch passes the swapped key and fails. Limit (AR-1): a rogue key present at first contact is pinned; recorded as `LIMIT` in a separate run.

| E-126 | K-9a, R-61, FX-2 | Hub id unchanged across restart and certificate rotation | status `hub {id, declared, match}` each tick; last id per hub kept | compare each tick and after each hub restart | `match: false` or id changed | fake status with another id fires | the right hub after a full restore (RR-20) |
**E-166 — Hub identity and pin at start: the mint is atomic, a first contact off the trusted paths pins nothing, an unreachable hub blocks posting.** Links: R-61, R-54, FX-2, AFC-21, FCW-81, FCW-84, FCW-87. Category: error handling. Level: L2. Tier: 1. Phase: PH-2 (B-1)
E-166.1 EVIDENCE: a mint that cannot be written atomically refuses start and leaves no partial file (a socket-only first start mints at random, CR-6.42); a first contact that is neither the local socket nor TLS under the pinned certificate pins nothing; a sidecar that cannot reach its hub at start neither posts nor pulls until the id is verified.
E-166.2 ARRANGE: runtime directory on a full disk; a first contact over an unpinned TLS endpoint; a hub that is down at sidecar start.
E-166.3 ACT: start the hub; start the sidecar against each.
E-166.4 ASSERT: hub refuses, no `hub.id` and no temp left; no pin written, hub-signed data unverified; the sidecar posts nothing and pulls nothing until verification succeeds.
E-166.5 CONTROL: mutant writing `hub.id` in place leaves a truncated id after a kill.
```

## 6. Evidence map
# T-3366 (arc-011 P-10, B-1) — evidence per E-n

The plan row P-10 (`docs/design/interactive-agent-communication-08-plan.md`) lists E-36, E-37, E-38, E-126, E-163 and
E-166. One line per E-n: what proves it, and where. Round 2, after review round 1
(`T-3366-reviews/r1-dispositions.md`), 2026-11-16.

| E-n | Verdict | Evidence |
|---|---|---|
| E-36 hub id: mint once, keep, refuse a silent re-mint | PASS | Unit `hub_identity::tests::e36_*`: an empty dir mints random, 0644, 16 hex; a cert-only dir is prior state (operator ruling B) and refuses until `adopt-id` gives it `first16(fingerprint)`; rotation keeps id and key; FCW-79 refuses and writes nothing, also on a topic log alone (message asserted); FCW-80 malformed ids refused. adopt-id refuses a migrated hub that lost one file (`adopt_refuses_a_lost_*`) and, with `--expect`, a differing id before writing anything. L4 `crates/termlink-cli/tests/t3366_hub_identity.rs` on a real hub: socket-only hub has an id, `hub.version` reads the file; restart + certificate rotation keep `hub_id` and `signing_key` while `hub_instance_id` changes; a legacy dir refuses to start with "hub.id missing beside existing state" until `hub adopt-id`. Controls: m1 (derive per call), m2, m3, m7, m13, m14, m15, m19 killed. Field: 4 hubs migrated, ids unchanged (runme-20261116T073957Z). |
| E-37 hub signing key: persisted, never printed, pinned on first contact | PASS | Hub half: unit `e37_*` (0600, FCW-82 mode/contents refused, key missing beside an id refused, debug form carries no seed). Sidecar half: `crates/termlink-sidecar/tests/t3366_hub_key_pin.rs` on a real hub: first contact pins + needs-attention; restart keeps; swapped key file → `verified_hub_key` refuses, pin unchanged, one entry naming both fingerprints (coalesced per day); an unreadable pin is never replaced; a withdrawn key is named and refused; the hub seed appears in no file of the sidecar store. SI-25 hub side: the real-hub test runs the hub with `RUST_LOG=info` and scans every file of its runtime dir, its stdout/stderr logs included, for the seed. LIMIT AR-1: `e37_ar1_limit_rogue_key_at_first_contact_is_pinned` records that a rogue key at the first trusted contact is pinned (by design). OBS-37: the sidecar has no admin-row log; the durable record is the timed needs-attention entry. Controls: m4, m5, m8, m9, m11, m12, m17 killed. |
| E-38 mail-hub declaration | NOT THIS UNIT | The `mail_hub` declaration reader and check is plan row P-11 (T-3363, "AEF owns the key", DR-7), which lists E-38 as its own evidence and depends on this unit's minted id, which now exists. |
| E-126 hub id unchanged across restart and certificate rotation | PASS | Sidecar status `hub {id, declared, match}` from the id each check observed, with the fake-status control (`e126_*`: a sidecar declared for another hub reads `match: false`; m18 killed). The rotation case of E-36 (L4). runme passes the running hub's served id as `--expect` and compares the NEW hub's served id after every restart, local and fleet (fixtures: "an id changed by the restart => FAILED"). |
| E-163 test seams refused outside an estate | NOT APPLICABLE | T-3366 adds no `TERMLINK_*_TEST_*` variable. Its only fault injector (`FAIL_BEFORE_RENAME`) is `#[cfg(test)]` and does not exist in a release build. |
| E-166 hub identity and pin at start | PASS | A mint that cannot be written atomically refuses and leaves no file or temp (`e166_fcw81_*`, `e166_failure_before_rename_*` with a fault injector between write and publish; m6 killed); the publish never clobbers (`e166_write_never_clobbers_*`, m16); 8 concurrent first starts converge on one id and key (`e166_concurrent_first_starts_converge_*`); a stale temp is removed only when its writer is dead. FCW-84: a first contact off the local socket pins nothing (`e166_fcw84_*`, m10 killed). A sidecar whose hub is down neither posts nor pulls until the id verifies: `fc_matrix::fc_8_hub_down_shows_waiting_then_unknown_then_goes_on_with_no_loss`, `send_hub::send_hub_hub_down_waits_then_flows_on_without_loss_or_duplicate` (both green in the full sidecar suite). |

Mutants: `docs/reports/T-3366-mutants.md`, 19 of 19 killed. Runme fixtures: 383 passed, 0 failed. Unit `hub_identity`:
20 passed; L4 hub: 3 passed; sidecar `t3366_hub_key_pin`: 6 passed; full hub, sidecar and CLI suites green.

## 7. Mutant report (run on the round-2 source)
| FC | Mutant | What it breaks | Test that must go red | Result |
|---|---|---|---|---|
| AL-17 inv.3 | `t3366-m1-derive-id-per-call` | hub.version derives hub_id from the certificate on each call (ARC4 6.1.7) | `e36_e37_restart_and_certificate_rotation_keep_id_and_signing_key` | killed |
| FCW-84 | `t3366-m10-pin-untrusted` | a first contact off the local socket pins the key | `e166_fcw84_untrusted_first_contact_pins_nothing_and_no_offer_changes_nothing` | killed |
| FCW-83 | `t3366-m11-gate-ignores-pin` | verified_hub_key accepts any offered key | `e37_first_contact_pins_restart_keeps_and_a_swapped_key_is_refused_and_named` | killed |
| FCW-83 / OBS-36 | `t3366-m12-mismatch-silent` | a changed hub key raises no needs-attention entry | `e37_first_contact_pins_restart_keeps_and_a_swapped_key_is_refused_and_named` | killed |
| AL-18 / r1 #1 | `t3366-m13-adopt-rekeys` | adopt-id generates a new key for a migrated hub that lost its key | `hub_identity::tests::adopt_refuses_a_lost_key_beside_an_id_and_writes_no_key` | killed |
| FCW-79 / r1 #1 | `t3366-m14-adopt-remints` | adopt-id re-mints a migrated hub that lost its id | `hub_identity::tests::adopt_refuses_a_lost_id_beside_a_key_and_writes_no_id` | killed |
| r1 #4 | `t3366-m15-expect-ignored` | adopt-id --expect is ignored | `hub_identity::tests::adopt_expect_mismatch_writes_nothing` | killed |
| AL-17 inv.1 / r1 #3 | `t3366-m16-publish-clobbers` | the publish replaces an existing hub.id | `hub_identity::tests::e166_write_never_clobbers_an_existing_id` | killed |
| FCW-83 / r1 #2 | `t3366-m17-read-error-repins` | an unreadable pin is taken as no pin and re-pinned | `e37_unreadable_pin_is_not_replaced` | killed |
| E-126 | `t3366-m18-status-always-match` | status says the hub id matches whatever answered | `e126_status_states_the_answering_hub_id_and_whether_it_matches` | killed |
| FCW-79 / ruling B | `t3366-m19-cert-not-prior` | a certificate-only directory mints silently instead of refusing | `hub_identity::tests::e36_first_start_with_cert_takes_the_certificate_id` | killed |
| FCW-79 | `t3366-m2-no-fcw79` | a missing hub.id beside prior state is minted instead of refused | `hub_identity::tests::e36_fcw79_missing_id_beside_state_refuses_and_writes_nothing` | killed |
| FCW-80 | `t3366-m3-lenient-id` | any 16-character hub.id is accepted | `hub_identity::tests::e36_fcw80_malformed_id_refuses` | killed |
| FCW-82 | `t3366-m4-no-key-mode-check` | a signing key file with any mode is accepted | `hub_identity::tests::e37_fcw82_bad_key_file_refuses` | killed |
| AL-18 | `t3366-m5-key-regenerated` | a missing signing key beside an existing id is regenerated at start | `hub_identity::tests::e37_key_missing_beside_id_refuses` | killed |
| FCW-81 | `t3366-m6-write-in-place` | hub.id is written in place, not via temp + publish | `hub_identity::tests::e166_failure_before_rename_leaves_no_id_and_no_temp` | killed |
| AL-17 adopt | `t3366-m7-adopt-ignores-existing-id` | adopt-id does not see an id already on disk | `hub_identity::tests::adopt_is_idempotent_and_never_changes_an_existing_id` | killed |
| SI-25 | `t3366-m8-debug-prints-key` | the debug form prints the private seed | `hub_identity::tests::e37_private_key_is_never_printed` | killed |
| FCW-83 / E-37 control | `t3366-m9-silent-repin` | a changed hub key is re-pinned silently | `e37_first_contact_pins_restart_keeps_and_a_swapped_key_is_refused_and_named` | killed |

PASS: all 19 mutants turned their test red. Run 2026-11-16T09:09Z on d04bfe9f7.

## 8. Evolution notes
```
## Evolution





### 2026-11-16 — build start: deployment order is part of the unit
- **What changed:** every runme run restarts the local hub onto the installed binary (step 5, `restart_hub`) and step 6 upgrades the fleet hubs. AL-17 FCW-79 makes a hub with prior state and no `hub.id` refuse to start. Shipping the binary without the migration in front of the restart would take every hub down on the next run.
- **Plan impact:** the legacy migration (AL-17 "one runme action per hub") is built here, not left to P-31: a CLI verb `termlink hub adopt-id` (writes `first16(current fingerprint)`, or random with "no certificate", only when `hub.id` is absent; idempotent; refuses a differing existing id), and runme steps 5 and 6 run it with the NEW binary before each restart. P-31 (T-3713) keeps the .107 encryption keys and key cards.
- **Slices:** S1 hub side: `hub.id` + `hub.signing.key` (FCW-79..82), `hub.version` reads the file and adds `signing_key`, `adopt-id` verb, E-36/E-37-hub/E-166-hub fixtures + mutants. S2 runme steps 5/6 migration-before-restart + fixtures. S3 sidecar signing-key pin (FCW-83/84, E-37 sidecar half). S4 `sidecar:` topic forward-with-warning (AC 3). E-38 is P-11's (T-3363) and is not built here.
- **Triggered:** none yet.

### 2026-11-16 — S1 + S2 built: spec reading on "prior state"
- **What changed:** AL-17 step 1 lists `hub.cert.pem` as prior state, but E-36.2/E-36.4 start a "cert only (legacy hub)" directory and expect it to keep `first16(fingerprint)`, which only AL-17's first-start branch gives. Built to E-36 (the tested reading): prior state = `hub.secret`, `hub.signing.key` or any topic log under `bus/` (what makes an id visible to peers); a certificate alone mints from the certificate. A real legacy hub always has a secret, so it still meets FCW-79 and goes through `adopt-id`. **Superseded the same day:** review r1 #3a objected that the agent settled the contradiction alone, toward the less fail-closed side; the operator ruled B (see Decisions): the certificate IS prior state, as AL-17 step 1 says.
- **What changed (2):** the hub mints before it writes its own secret/certificate in `prepare_start`, so a brand-new TCP hub gets a random id (CR-6.42), not one derived from the certificate it is about to create.
- **S3 design (AL-18 sidecar half):** one module `hub_key_pin` in termlink-sidecar, called from the existing `check_pin` in send.rs and pull.rs after the hub-id match, with the `hub.version` reply already in hand. Pin stored per hub id in store state `hub-key-pin-<hub_id>`. No `signing_key` offered (pre-T-3366 hub): unchanged behaviour, status says "not offered". No pin yet: pin only when the hub address is the local socket or TLS under the TOFU-pinned certificate, write the pin, needs-attention "hub signing key pinned" (FCW-84 otherwise: no pin, hub-signed data unverified). Pin differs: needs-attention naming both fingerprints, coalesced per day; `verified_hub_key()` returns None, so every hub-signed object P-15/P-16 will check is refused (FCW-83). Mail posting and pulling are NOT held on a key mismatch: there are no hub-signed objects yet, and holding mail would switch off a working route (NID rule); the spec refuses hub-signed objects, not mail. Status gains `hub_signing_key {pinned, offered, state}` (OBS-36). Pin changes only by a runme re-pin action (later, with P-15).
- **Plan impact:** S2 covers both local restart points (step 5 `restart_hub`, step 5a `install_hub_unit`) and the fleet (step 6), where the runtime dir is read from the running hub's own environment; an unconfirmable dir or an adopted id that differs from the served id fails the hub without swapping.
- **Triggered:** none.

## Recommendation
```

## 9. Code: crates/termlink-hub/src/hub_identity.rs (at adf4555e0) — production code in full; its 20 unit tests by name (bodies omitted for size; result line in section 14, mutants in section 7)
```rust
//! T-3366 (arc-011 P-10, B-1; AL-17 + AL-18 hub half): the hub's canonical id and signing key.
//!
//! `hub.id` holds 16 lowercase hex, minted ONCE per runtime directory and never recomputed: a
//! certificate rotation no longer renames `inbox:<hub-id>/…`. `hub.signing.key` holds an ed25519
//! seed (64 hex, mode 0600) for hub-signed state (revocation list, dead letters, cards).
//!
//! Rules (docs/design/interactive-agent-communication-06a-pseudo-code-ph2-hardening.md 2.1, 2.2):
//! 1. `hub.id` present: it must be 16 lowercase hex, else refuse (FCW-80).
//! 2. `hub.id` absent beside prior hub state: refuse (FCW-79). The only way past is
//!    [`adopt_legacy`] (`termlink hub adopt-id`), which runme runs before a restart.
//! 3. Empty runtime directory: mint from the certificate if one exists NOW, else 64 random bits
//!    (CR-6.42). Written via temp + fsync + rename; a failed write leaves no file (FCW-81).
//! 4. `hub.signing.key`: generated only on a first start (same rule); otherwise it must exist,
//!    be mode 0600, owned by this user, and hold a 32-byte seed (FCW-82). Never printed.
//!
//! Every function takes the runtime directory explicitly, so tests use temp dirs and never the
//! live hub's directory.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use ed25519_dalek::SigningKey;
use sha2::{Digest, Sha256};

pub const ID_FILE: &str = "hub.id";
pub const KEY_FILE: &str = "hub.signing.key";

/// The identity a started hub answers with. Set once per start, read by `hub.version`.
#[derive(Clone)]
pub struct HubIdentity {
    pub hub_id: String,
    signing: SigningKey,
}

impl HubIdentity {
    /// `ed25519:` + hex SHA-256 of the 32-byte public key (AL-18 2.2.h).
    pub fn signing_key_fingerprint(&self) -> String {
        signing_fingerprint(&self.signing)
    }

    /// The signing key, for hub-signed objects (P-15, P-16). Never serialise it.
    pub fn signing_key(&self) -> &SigningKey {
        &self.signing
    }
}

impl std::fmt::Debug for HubIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // SI-25: the private key never appears in any log or debug print.
        f.debug_struct("HubIdentity")
            .field("hub_id", &self.hub_id)
            .field("signing_key", &self.signing_key_fingerprint())
            .finish()
    }
}

pub fn signing_fingerprint(k: &SigningKey) -> String {
    let digest = Sha256::digest(k.verifying_key().as_bytes());
    format!("ed25519:{}", hex(&digest))
}

static CURRENT: RwLock<Option<HubIdentity>> = RwLock::new(None);

/// Record the identity the running hub answers with (called by the server at start).
pub fn set_current(id: HubIdentity) {
    if let Ok(mut g) = CURRENT.write() {
        *g = Some(id);
    }
}

/// The running hub's identity, if it has started. `None` means `hub.version` states null:
/// the hub never falls back to the certificate (AL-17 invariant 3).
pub fn current() -> Option<HubIdentity> {
    CURRENT.read().ok().and_then(|g| g.clone())
}

fn refuse(msg: impl Into<String>) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, msg.into())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn is_id(s: &str) -> bool {
    s.len() == 16 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Prior hub state in `dir`: a secret, a certificate, a signing key, or any topic log
/// (AL-17 step 1). Operator ruling B, 2026-11-16 (T-3366): a certificate alone IS prior state,
/// so a certificate-only directory refuses (FCW-79) until `hub adopt-id` gives it the same
/// certificate-derived id E-36 expects. Fail-closed over E-36's unreachable "start" reading.
pub fn has_prior_state(dir: &Path) -> bool {
    ["hub.secret", "hub.cert.pem", KEY_FILE].iter().any(|f| dir.join(f).exists())
        || std::fs::read_dir(dir.join("bus")).map(|mut r| r.next().is_some()).unwrap_or(false)
}

/// Read `hub.id`: `Ok(None)` when absent, `Err` when unreadable or malformed (FCW-80).
pub fn read_id(dir: &Path) -> std::io::Result<Option<String>> {
    let path = dir.join(ID_FILE);
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(refuse(format!("hub.id unreadable or malformed ({}: {e})", path.display()))),
    };
    let s = raw.strip_suffix('\n').unwrap_or(&raw);
    if !is_id(s) {
        return Err(refuse(format!(
            "hub.id unreadable or malformed ({}: not 16 lowercase hex)",
            path.display()
        )));
    }
    Ok(Some(s.to_string()))
}

/// Write `contents` to `dir/name` atomically: temp (with `mode`) + fsync + rename + dir fsync.
/// On any error the temp is removed and no partial file is left (FCW-81).
#[cfg(test)]
thread_local! {
    /// E-166 fault injector (tests only): fail after the temp is written, before the rename.
    static FAIL_BEFORE_RENAME: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn atomic_write(dir: &Path, name: &str, contents: &[u8], mode: u32) -> std::io::Result<()> {
    let path = dir.join(name);
    // Crash table (AL-17 2.1.f): a temp left by a start that died before its publish is removed.
    // Only a DEAD writer's temp: a concurrent start may be writing its own right now.
    if let Ok(rd) = std::fs::read_dir(dir) {
        let prefix = format!(".{name}.tmp-");
        for e in rd.flatten() {
            let fname = e.file_name().to_string_lossy().into_owned();
            let Some(rest) = fname.strip_prefix(&prefix) else { continue };
            let pid: Option<i32> = rest.split('-').next().and_then(|p| p.parse().ok());
            // SAFETY: kill(pid, 0) only probes for existence.
            let dead = pid.is_none_or(|p| p <= 0 || (unsafe { libc::kill(p, 0) } != 0
                && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)));
            if dead {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let tmp: PathBuf = dir.join(format!(
        ".{name}.tmp-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let res = (|| {
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new().create_new(true).write(true).mode(mode).open(&tmp)?;
        f.write_all(contents)?;
        f.sync_all()?;
        drop(f);
        #[cfg(test)]
        if FAIL_BEFORE_RENAME.with(|c| c.get()) {
            return Err(std::io::Error::other("injected: crash before rename"));
        }
        // No-clobber publish (review T-3366 r1 #3): link the complete temp to the final name;
        // `AlreadyExists` when another start got there first, and that file is never replaced.
        std::fs::hard_link(&tmp, &path)?;
        let _ = std::fs::remove_file(&tmp);
        if let Ok(d) = std::fs::File::open(dir) {
            let _ = d.sync_all();
        }
        Ok(())
    })();
    if res.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    res
}

/// The id a fresh mint takes: the certificate's when one is present now, else random.
/// Returns (id, from_certificate).
fn mint_value(dir: &Path) -> (String, bool) {
    if let Some(id) = crate::tls::own_cert_fingerprint_at(&dir.join("hub.cert.pem"))
        .as_deref()
        .and_then(crate::tls::hub_id_from_fingerprint)
    {
        return (id, true);
    }
    let r = termlink_session::auth::generate_secret();
    (hex(&r[..8]), false)
}

fn write_id(dir: &Path, id: &str) -> std::io::Result<()> {
    atomic_write(dir, ID_FILE, format!("{id}\n").as_bytes(), 0o644)
        .map_err(|e| std::io::Error::new(e.kind(), format!("hub.id could not be written atomically: {e}")))
}

fn generate_key(dir: &Path) -> std::io::Result<SigningKey> {
    let seed = termlink_session::auth::generate_secret();
    atomic_write(dir, KEY_FILE, format!("{}\n", hex(&seed)).as_bytes(), 0o600)
        .map_err(|e| std::io::Error::new(e.kind(), format!("hub.signing.key could not be written atomically: {e}")))?;
    Ok(SigningKey::from_bytes(&seed))
}

/// Read `hub.signing.key`: `Ok(None)` when absent; `Err` on wrong mode, owner or contents (FCW-82).
pub fn read_key(dir: &Path) -> std::io::Result<Option<SigningKey>> {
    use std::os::unix::fs::MetadataExt;
    let path = dir.join(KEY_FILE);
    let meta = match std::fs::metadata(&path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(refuse(format!("hub.signing.key unreadable ({}: {e})", path.display()))),
    };
    let mode = meta.mode() & 0o777;
    if mode != 0o600 {
        return Err(refuse(format!("hub.signing.key has mode {mode:o}, must be 600 ({})", path.display())));
    }
    // SAFETY: geteuid has no preconditions.
    let euid = unsafe { libc::geteuid() };
    if meta.uid() != euid {
        return Err(refuse(format!(
            "hub.signing.key is owned by uid {}, the hub runs as uid {euid} ({})",
            meta.uid(),
            path.display()
        )));
    }
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| refuse(format!("hub.signing.key unreadable ({}: {e})", path.display())))?;
    let s = raw.trim_end_matches('\n');
    let bytes: Option<Vec<u8>> = (s.len() == 64)
        .then(|| (0..32).map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).ok()).collect())
        .flatten();
    let Some(bytes) = bytes else {
        // The contents are never echoed (SI-25).
        return Err(refuse(format!("hub.signing.key is malformed: not a 32-byte hex seed ({})", path.display())));
    };
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&bytes);
    Ok(Some(SigningKey::from_bytes(&seed)))
}

/// Hub start (AL-17 step 2, AL-18): read or mint the id, read or generate the key.
/// Must run BEFORE the start writes its own secret or certificate, so a first start is
/// recognised as one.
pub fn load_or_mint(dir: &Path) -> std::io::Result<HubIdentity> {
    let prior = has_prior_state(dir);
    let (hub_id, first_start) = match read_id(dir)? {
        Some(id) => (id, false),
        None if prior => {
            return Err(refuse(format!(
                "hub.id missing beside existing state in {}; restore it, or run `termlink hub adopt-id` (runme does this before a restart)",
                dir.display()
            )));
        }
        None => {
            std::fs::create_dir_all(dir)?;
            let (id, from_cert) = mint_value(dir);
            match write_id(dir, &id) {
                Ok(()) => {
                    tracing::info!(hub_id = %id, from_certificate = from_cert, "hub.id minted (first start, T-3366)");
                    (id, true)
                }
                // A concurrent first start minted first: take ITS id, never serve one that lost.
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => match read_id(dir)? {
                    Some(won) => (won, true),
                    None => return Err(e),
                },
                Err(e) => return Err(e),
            }
        }
    };
    let signing = match read_key(dir)? {
        Some(k) => k,
        None if first_start => match generate_key(dir) {
            Ok(k) => k,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => read_key(dir)?.ok_or(e)?,
            Err(e) => return Err(e),
        },
        None => {
            return Err(refuse(format!(
                "hub.signing.key missing beside existing state in {}; run `termlink hub adopt-id` (runme does this before a restart)",
                dir.display()
            )));
        }
    };
    Ok(HubIdentity { hub_id, signing })
}

/// What [`adopt_legacy`] did.
#[derive(Debug, PartialEq, Eq)]
pub struct Adopted {
    pub hub_id: String,
    /// "kept" (already present), "certificate" (first16 of the current fingerprint) or
    /// "random" (no certificate: peers will see a new id and refuse it until re-pinned).
    pub id_source: &'static str,
    /// true when this call generated the signing key.
    pub key_generated: bool,
    pub signing_key: String,
}

/// The legacy migration (AL-17, "the only path past FCW-79"): give a hub that predates
/// T-3366 its id and signing key, without renaming any inbox. Idempotent. It acts ONLY on a
/// pre-T-3366 directory (neither `hub.id` nor `hub.signing.key`); a directory that has one
/// without the other was already migrated and lost a file, and is refused, never re-minted or
/// re-keyed (review T-3366 r1 #1: that would undo FCW-79 / AL-18 through the deploy path).
/// `expect`: the id the hub serves today; the call refuses, writing nothing, when the id it
/// would write (or finds) differs, so a wrong id never lands on disk.
pub fn adopt_legacy(dir: &Path, dry_run: bool, expect: Option<&str>) -> std::io::Result<Adopted> {
    let id = read_id(dir)?;
    let key = read_key(dir)?;
    let check = |found: &str| -> std::io::Result<()> {
        match expect {
            Some(e) if e != found => Err(refuse(format!(
                "hub.id would be {found} but the hub serves {e}; nothing written (an inbox would be renamed)"
            ))),
            _ => Ok(()),
        }
    };
    match (id, key) {
        (Some(id), Some(k)) => {
            check(&id)?;
            Ok(Adopted { hub_id: id, id_source: "kept", key_generated: false, signing_key: signing_fingerprint(&k) })
        }
        (Some(_), None) => Err(refuse(format!(
            "hub.signing.key missing beside hub.id in {}: this hub was already migrated and lost its key; adopt-id does not re-key (a new key is an operator action)",
            dir.display()
        ))),
        (None, Some(_)) => Err(refuse(format!(
            "hub.id missing beside hub.signing.key in {}: this hub was already migrated and lost its id; restore hub.id, adopt-id does not re-mint",
            dir.display()
        ))),
        (None, None) => {
            let (id, from_cert) = mint_value(dir);
            check(&id)?;
            let id_source = if from_cert { "certificate" } else { "random" };
            if dry_run {
                return Ok(Adopted { hub_id: id, id_source, key_generated: true, signing_key: "(would be generated)".into() });
            }
            std::fs::create_dir_all(dir)?;
            write_id(dir, &id)?;
            let k = generate_key(dir)?;
            Ok(Adopted { hub_id: id, id_source, key_generated: true, signing_key: signing_fingerprint(&k) })
        }
    }
}

```
Unit tests:
```
// E-36: empty dir mints random, file 0644, 16 hex; restart keeps it.
    fn e36_empty_dir_mints_once_and_restart_keeps_it
// E-36: a cert present at first start gives first16(fingerprint).
    fn e36_first_start_with_cert_takes_the_certificate_id
// E-36 control (ARC4 6.1.7): rotation changes neither id nor key — kills "derive each call".
    fn e36_certificate_rotation_changes_neither_id_nor_key
// E-36 / FCW-79: secret + cert, no hub.id → refuse, and write nothing.
    fn e36_fcw79_missing_id_beside_state_refuses_and_writes_nothing
// FCW-79 also on a topic log alone.
    fn e36_fcw79_topic_log_counts_as_prior_state
// FCW-80: malformed id refuses (uppercase, short, junk).
    fn e36_fcw80_malformed_id_refuses
// E-166 / FCW-81: an unwritable directory refuses and leaves no file or temp.
    fn e166_fcw81_mint_that_cannot_be_written_leaves_nothing
// E-166 control (mutant "write hub.id in place"): a failure after the bytes are written but // before the rename leaves NO hub.id and no temp; an in-place writer leaves the file.
    fn e166_failure_before_rename_leaves_no_id_and_no_temp
// Crash table: a temp left by a dead start is removed by the next write.
    fn e166_stale_temp_from_a_dead_start_is_removed

    fn e166_atomic_write_never_leaves_a_temp
// E-37 hub half / FCW-82: key mode, contents.
    fn e37_fcw82_bad_key_file_refuses
// AL-18: key missing beside an existing id refuses (not regenerated silently).
    fn e37_key_missing_beside_id_refuses
// E-37 / SI-25: the debug form and fingerprint carry no seed bytes.
    fn e37_private_key_is_never_printed
// adopt: idempotent, keeps an existing id, socket-only legacy gets random, dry-run writes nothing.
    fn adopt_is_idempotent_and_never_changes_an_existing_id
// Review r1 #1: a migrated hub that lost ONE file is refused, never re-keyed or re-minted.
    fn adopt_refuses_a_lost_key_beside_an_id_and_writes_no_key

    fn adopt_refuses_a_lost_id_beside_a_key_and_writes_no_id
// `--expect`: a differing id is refused before anything is written; a matching one passes.
    fn adopt_expect_mismatch_writes_nothing
// Review r1 #3: the publish never replaces an existing file.
    fn e166_write_never_clobbers_an_existing_id
// Review r1 #3: concurrent first starts on one directory all end with the SAME id and key.
    fn e166_concurrent_first_starts_converge_on_one_id

    fn adopt_refuses_a_malformed_id_rather_than_replacing_it
```

## 10. Code: crates/termlink-sidecar/src/hub_key_pin.rs (at adf4555e0)
```rust
//! T-3366 (arc-011 P-10, B-1; AL-18 sidecar half, E-37): pin the hub's signing key.
//!
//! The hub states `signing_key = "ed25519:<sha256 of the public key>"` in `hub.version`. The
//! sidecar pins it on its first VERIFIED contact and from then on accepts hub-signed data only
//! under that pin. Hub-signed objects arrive with P-15 (revocation list) and P-16 (dead letters);
//! until then the pin is recorded, compared and reported, and [`verified_hub_key`] is the gate
//! those units call.
//!
//! Rules (06a 2.2):
//! 1. No `signing_key` offered (a hub that predates T-3366): nothing pinned, nothing refused.
//! 2. No pin yet: pin only when the contact is over the local hub socket (FCW-84: anything else
//!    pins nothing and hub-signed data stays unverified). TCP sidecar transport has no runtime
//!    implementation today, so "TLS under the pinned certificate" is not reachable here yet.
//! 3. Pin differs from the offer: every hub-signed object is refused, and a needs-attention entry
//!    names both fingerprints, once per day per offered key (FCW-83). A pin is never replaced
//!    here; only an operator re-pin (runme) changes it.
//!
//! Mail is NOT held on a key mismatch: there are no hub-signed objects yet, and stopping mail
//! would switch off a working route (NID rule). The hub-id check (FX-2) still holds mail.

use serde_json::{Value, json};

use crate::store::Store;

/// What one observation of `hub.version` found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyState {
    /// The hub offers no signing key (pre-T-3366).
    NotOffered,
    /// The offer matches the pin.
    Pinned(String),
    /// First verified contact: the offer was pinned now.
    NewlyPinned(String),
    /// No pin, and this contact was not over a trusted path (FCW-84).
    Unverified(String),
    /// The offer differs from the pin (FCW-83).
    Mismatch { pinned: String, offered: String },
}

impl KeyState {
    pub fn as_str(&self) -> &'static str {
        match self {
            KeyState::NotOffered => "not offered",
            KeyState::Pinned(_) | KeyState::NewlyPinned(_) => "pinned",
            KeyState::Unverified(_) => "unverified",
            KeyState::Mismatch { .. } => "mismatch",
        }
    }
}

pub fn pin_state_name(hub_id: &str) -> String {
    format!("hub-key-pin-{hub_id}")
}

fn seen_state_name(hub_id: &str) -> String {
    format!("hub-key-seen-{hub_id}")
}

fn valid_fp(s: &str) -> bool {
    s.strip_prefix("ed25519:")
        .is_some_and(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
}

fn offered(version: &Value) -> Option<String> {
    version.get("signing_key").and_then(Value::as_str).map(str::to_owned)
}

fn alert(store: &Store, kind: &str, detail: &str) {
    if let Err(e) = store.attention(kind, detail) {
        eprintln!("termlink-sidecar: needs attention ({kind}): {detail} [could not record it: {e}]");
    }
}

/// Record the last observation so `status` can show it without a hub call; written only on change.
fn remember(store: &Store, hub_id: &str, st: &KeyState) {
    let off = match st {
        KeyState::NotOffered => String::new(),
        KeyState::Pinned(f) | KeyState::NewlyPinned(f) | KeyState::Unverified(f) => f.clone(),
        KeyState::Mismatch { offered, .. } => offered.clone(),
    };
    let line = format!("{} {}", st.as_str().replace(' ', "-"), off);
    let name = seen_state_name(hub_id);
    if store.read_state(&name).ok().flatten().as_deref().map(str::trim) != Some(line.trim()) {
        let _ = store.write_state(&name, &line);
    }
}

/// Observe one `hub.version` reply from the hub `hub_id` (already matched against the declared
/// id). `trusted_path` is true for the local hub socket. Never fails: a problem becomes a state
/// and, where the spec asks for one, a needs-attention entry.
pub fn observe(store: &Store, hub_id: &str, version: &Value, trusted_path: bool) -> KeyState {
    // Review T-3366 r1 #2: a store read ERROR is not "no pin" — it must never lead to a re-pin.
    let pin = match store.read_state(&pin_state_name(hub_id)) {
        Ok(p) => p.map(|s| s.trim().to_string()),
        Err(e) => {
            let off = offered(version).unwrap_or_default();
            alert(store, "hub-signing-key-pin-unreadable", &format!("hub {hub_id}: the stored pin cannot be read ({e}); hub-signed data unverified, nothing re-pinned"));
            let st = KeyState::Unverified(off);
            remember(store, hub_id, &st);
            return st;
        }
    };
    let st = match offered(version) {
        None => {
            if let Some(p) = &pin {
                // A pinned hub that stops offering its key: refused like a mismatch, and said so.
                let day = crate::store::now_ms() / 86_400_000;
                let detail = format!("hub {hub_id} no longer offers its signing key (pinned {p}); every hub-signed object is refused");
                if let Err(e) = store.attention_coalesced("hub-signing-key-withdrawn", &format!("{day}"), &detail) {
                    eprintln!("termlink-sidecar: needs attention (hub-signing-key-withdrawn): {detail} [could not record it: {e}]");
                }
            }
            KeyState::NotOffered
        }
        Some(off) => {
            match pin {
                Some(p) if p == off => KeyState::Pinned(off),
                Some(p) => {
                    let detail = format!(
                        "hub {hub_id} signing key changed: pinned {p}, offered {off}; every hub-signed object is refused until an operator re-pin"
                    );
                    let day = crate::store::now_ms() / 86_400_000;
                    if let Err(e) = store.attention_coalesced("hub-signing-key-changed", &format!("{day}:{off}"), &detail) {
                        eprintln!("termlink-sidecar: needs attention (hub-signing-key-changed): {detail} [could not record it: {e}]");
                    }
                    KeyState::Mismatch { pinned: p, offered: off }
                }
                None if !trusted_path || !valid_fp(&off) => KeyState::Unverified(off),
                None => match store.write_state(&pin_state_name(hub_id), &off) {
                    Ok(()) => {
                        alert(store, "hub-signing-key-pinned", &format!("hub {hub_id} signing key pinned: {off}"));
                        KeyState::NewlyPinned(off)
                    }
                    Err(e) => {
                        alert(store, "hub-signing-key-pin-failed", &format!("hub {hub_id}: could not record the pin {off}: {e}"));
                        KeyState::Unverified(off)
                    }
                },
            }
        }
    };
    remember(store, hub_id, &st);
    st
}

/// The gate for hub-signed objects (P-15, P-16): the hub's key fingerprint when, and only when,
/// the offer in `version` equals the stored pin. `None` means refuse.
pub fn verified_hub_key(store: &Store, hub_id: &str, version: &Value) -> Option<String> {
    let off = offered(version)?;
    let pin = store.read_state(&pin_state_name(hub_id)).ok().flatten()?; // error or absent: refuse
    (pin.trim() == off).then_some(off)
}

fn hub_seen_name(declared: &str) -> String {
    format!("hub-id-seen-{declared}")
}

/// E-126: record the id the hub answered with on this check (written only on change), so
/// `status` shows `hub {id, declared, match}` without a hub call.
pub fn observe_hub_id(store: &Store, declared: &str, found: Option<&str>) {
    let line = found.unwrap_or("").to_string();
    let name = hub_seen_name(declared);
    if store.read_state(&name).ok().flatten().as_deref().map(str::trim) != Some(line.as_str()) {
        let _ = store.write_state(&name, &line);
    }
}

/// `status` view (E-126): `{id, declared, match}`; `id` null when never observed.
pub fn hub_status(store: &Store, declared: Option<&str>) -> Value {
    let Some(d) = declared else {
        return json!({"id": null, "declared": null, "match": false});
    };
    let seen = store.read_state(&hub_seen_name(d)).ok().flatten().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let m = seen.as_deref() == Some(d);
    json!({"id": seen, "declared": d, "match": m})
}

/// `status` view (OBS-36): `{pinned, offered, state}` from the stored pin and last observation.
pub fn status(store: &Store, hub_id: Option<&str>) -> Value {
    let Some(h) = hub_id else {
        return json!({"pinned": null, "offered": null, "state": "no hub id configured"});
    };
    let pinned = store.read_state(&pin_state_name(h)).ok().flatten().map(|s| s.trim().to_string());
    let seen = store.read_state(&seen_state_name(h)).ok().flatten().unwrap_or_default();
    let mut it = seen.trim().splitn(2, ' ');
    let state = it.next().filter(|s| !s.is_empty()).unwrap_or("not observed").replace('-', " ");
    let offered = it.next().filter(|s| !s.is_empty()).map(str::to_owned);
    json!({"pinned": pinned, "offered": offered, "state": state})
}
```

## 11. Diff of the other T-3366 changes (hub start, hub.version, CLI, fleet doctor, sidecar wiring, runme), 8bba3733c..adf4555e0
```diff
diff --git a/crates/termlink-cli/src/cli.rs b/crates/termlink-cli/src/cli.rs
index 1e4b839e8..27a810229 100644
--- a/crates/termlink-cli/src/cli.rs
+++ b/crates/termlink-cli/src/cli.rs
@@ -1606,6 +1606,25 @@ pub(crate) enum HubAction {
         #[arg(long)]
         json: bool,
     },
+    /// Give a hub that predates T-3366 its canonical id and signing key (legacy migration)
+    ///
+    /// Writes <runtime_dir>/hub.id = first 16 hex of the current TLS fingerprint (the id every
+    /// inbox already uses, so nothing is renamed; random when there is no certificate) and
+    /// generates hub.signing.key (0600), each only when absent. Idempotent: an existing id is
+    /// never changed; a directory with only one of the two files (a migrated hub that lost one) is
+    /// refused, never re-minted or re-keyed. Safe while the hub runs (the running binary ignores
+    /// both files). runme runs it right after installing a new binary and before every restart.
+    AdoptId {
+        /// Report what would be written; write nothing
+        #[arg(long)]
+        dry_run: bool,
+        /// The id the hub serves today: refuse, writing nothing, if the id would differ
+        #[arg(long, value_name = "HUB_ID")]
+        expect: Option<String>,
+        /// Output as JSON: {"runtime_dir","hub_id","id_source","key_generated","signing_key","dry_run"}
+        #[arg(long)]
+        json: bool,
+    },
     /// Probe a remote hub via TLS handshake and print its leaf cert sha256
     ///
     /// G-011 rotation-protocol companion to `hub fingerprint` (which reads
diff --git a/crates/termlink-cli/src/commands/infrastructure.rs b/crates/termlink-cli/src/commands/infrastructure.rs
index f0563a26d..3a9617393 100644
--- a/crates/termlink-cli/src/commands/infrastructure.rs
+++ b/crates/termlink-cli/src/commands/infrastructure.rs
@@ -983,6 +983,44 @@ pub(crate) fn cmd_hub_fingerprint(json_output: bool) -> Result<()> {
     Ok(())
 }
 
+/// T-3366: `termlink hub adopt-id` — the AL-17 legacy migration, the only path past FCW-79.
+/// Exit 0 with what it did (or would do); an unreadable/malformed id or key file is an error.
+pub(crate) fn cmd_hub_adopt_id(dry_run: bool, expect: Option<&str>, json_output: bool) -> Result<()> {
+    let dir = termlink_session::discovery::runtime_dir();
+    let a = termlink_hub::hub_identity::adopt_legacy(&dir, dry_run, expect)
+        .map_err(|e| anyhow::anyhow!("hub adopt-id: {e}"))?;
+    if json_output {
+        println!(
+            "{}",
+            json!({
+                "runtime_dir": dir.display().to_string(),
+                "hub_id": a.hub_id,
+                "id_source": a.id_source,
+                "key_generated": a.key_generated,
+                "signing_key": a.signing_key,
+                "dry_run": dry_run,
+            })
+        );
+    } else {
+        let verb = if dry_run { "would write" } else { "wrote" };
+        match a.id_source {
+            "kept" => println!("hub.id {} kept (already present) in {}", a.hub_id, dir.display()),
+            "certificate" => println!("hub.id {} {verb} from the current certificate in {}", a.hub_id, dir.display()),
+            _ => println!(
+                "hub.id {} {verb} at random: no certificate in {} (peers will see a new id)",
+                a.hub_id,
+                dir.display()
+            ),
+        }
+        if a.key_generated {
+            println!("hub.signing.key {} ({})", if dry_run { "would be generated" } else { "generated" }, a.signing_key);
+        } else {
+            println!("hub.signing.key kept ({})", a.signing_key);
+        }
+    }
+    Ok(())
+}
+
 /// T-3503 (T7d): resolve an `unknown` worker-contract attempt on this host. No RPC exists
 /// for it; this opens the local bus store directly. Errors are loud (non-zero exit).
 /// T-3527: `CLAUDECODE=1` marks an agent session (the AEF convention, `lib/arc.sh`), which
diff --git a/crates/termlink-cli/src/commands/remote.rs b/crates/termlink-cli/src/commands/remote.rs
index fd41b7879..56fe805de 100644
--- a/crates/termlink-cli/src/commands/remote.rs
+++ b/crates/termlink-cli/src/commands/remote.rs
@@ -5330,7 +5330,9 @@ pub(crate) async fn cmd_fleet_doctor(
                 total_pass += 1;
 
                 // T-1132: probe hub.version; fall back to "unknown" for pre-T-1132 hubs.
-                let hub_version = match client
+                // T-3366: keep the hub's canonical `hub_id` from the same reply (null when the hub
+                // states none), so a deploy can check the id survived the restart (runme step 6).
+                let (hub_version, hub_id, signing_key) = match client
                     .call(
                         "hub.version",
                         serde_json::json!("fleet-doctor-version"),
@@ -5338,13 +5340,12 @@ pub(crate) async fn cmd_fleet_doctor(
                     )
                     .await
                 {
-                    Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => r
-                        .result
-                        .get("hub_version")
-                        .and_then(|v| v.as_str())
-                        .unwrap_or("unknown")
-                        .to_string(),
-                    _ => "unknown".to_string(),
+                    Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => (
+                        r.result.get("hub_version").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
+                        r.result.get("hub_id").cloned().unwrap_or(serde_json::Value::Null),
+                        r.result.get("signing_key").cloned().unwrap_or(serde_json::Value::Null),
+                    ),
+                    _ => ("unknown".to_string(), serde_json::Value::Null, serde_json::Value::Null),
                 };
                 *fleet_versions.entry(hub_version.clone()).or_insert(0) += 1;
 
@@ -5401,6 +5402,8 @@ pub(crate) async fn cmd_fleet_doctor(
                     "latency_ms": latency,
                     "secret_source": &secret_source,
                     "hub_version": &hub_version,
+                    "hub_id": hub_id,
+                    "signing_key": signing_key,
                 });
                 // T-1652: surface the perms warning even on a healthy hub —
                 // the leak risk is structural to the file, not to the probe.
diff --git a/crates/termlink-hub/src/router.rs b/crates/termlink-hub/src/router.rs
index d7112ce0e..cfc920062 100644
--- a/crates/termlink-hub/src/router.rs
+++ b/crates/termlink-hub/src/router.rs
@@ -884,13 +884,18 @@ fn handle_register_remote(id: serde_json::Value, params: &serde_json::Value) ->
 /// across the fleet before a Tier-B RPC fails on a skewed hub.
 ///
 /// T-3345 (OD-12): also states the hub's own identity — `hub_id` (canonical
-/// id, today the 16-hex fingerprint prefix) and `hub_instance_id` (the full
-/// TLS fingerprint). Both are `null` when the hub has no cert on disk; the
-/// hub never guesses. Clients read these instead of deriving the id from an
-/// unauthenticated `hub probe`.
+/// id) and `hub_instance_id` (the full TLS fingerprint, `null` without a cert;
+/// it changes on rotation, the canonical id does not). Clients read these
+/// instead of deriving the id from an unauthenticated `hub probe`.
+///
+/// T-3366 (AL-17 invariant 3): `hub_id` is the minted `hub.id` the hub read at start, never
+/// recomputed from the certificate (so a rotation renames nothing); `signing_key` is
+/// `ed25519:<sha256 of the public key>` (AL-18). Both null only before the hub has started.
 fn handle_hub_version(id: serde_json::Value) -> RpcResponse {
     let instance = crate::tls::own_cert_fingerprint();
-    let hub_id = instance.as_deref().and_then(crate::tls::hub_id_from_fingerprint);
+    let ident = crate::hub_identity::current();
+    let hub_id = ident.as_ref().map(|i| i.hub_id.clone());
+    let signing_key = ident.as_ref().map(|i| i.signing_key_fingerprint());
     Response::success(
         id,
         json!({
@@ -899,6 +904,7 @@ fn handle_hub_version(id: serde_json::Value) -> RpcResponse {
             "control_plane_version": termlink_protocol::CONTROL_PLANE_VERSION,
             "hub_id": hub_id,
             "hub_instance_id": instance,
+            "signing_key": signing_key,
         }),
     )
     .into()
diff --git a/crates/termlink-hub/src/server.rs b/crates/termlink-hub/src/server.rs
index f3ff039bd..3dc58dae7 100644
--- a/crates/termlink-hub/src/server.rs
+++ b/crates/termlink-hub/src/server.rs
@@ -489,6 +489,12 @@ async fn prepare_start(
         std::fs::create_dir_all(parent)?;
     }
 
+    // T-3366 (AL-17/AL-18): read or mint hub.id and hub.signing.key BEFORE this start writes
+    // its own secret or certificate, so a first start is recognised as one. A refusal here
+    // (FCW-79..82) leaves a running hub's files untouched, like every check above.
+    let identity = crate::hub_identity::load_or_mint(&termlink_session::discovery::runtime_dir())?;
+    tracing::info!(hub_id = %identity.hub_id, signing_key = %identity.signing_key_fingerprint(), "Hub identity loaded (T-3366)");
+
     let (tcp_listener, token_secret, tls_acceptor) = if let Some(addr) = tcp_addr {
         let listener = TcpListener::bind(addr).await.map_err(|e| {
             std::io::Error::new(e.kind(), format!("Failed to bind TCP {}: {}", addr, e))
@@ -501,6 +507,7 @@ async fn prepare_start(
     };
 
     let (unix_listener, socket_id) = claim_runtime(socket_path, pidfile_path)?;
+    crate::hub_identity::set_current(identity);
     tracing::info!(path = %socket_path.display(), "Hub listening on Unix");
 
     let tcp_flag = termlink_session::discovery::runtime_dir().join("hub.tcp");
diff --git a/crates/termlink-sidecar/src/api.rs b/crates/termlink-sidecar/src/api.rs
index 96262c93e..4c12824a9 100644
--- a/crates/termlink-sidecar/src/api.rs
+++ b/crates/termlink-sidecar/src/api.rs
@@ -1248,6 +1248,8 @@ fn status_at(
         "ok": true,
         "agent": agent,
         "hub_id": hub_id.map(Value::from).unwrap_or_else(|| json!("not configured")),
+        "hub": crate::hub_key_pin::hub_status(store, hub_id),
+        "hub_signing_key": crate::hub_key_pin::status(store, hub_id),
         "stages": STAGES,
         "rows": rows,
         "messages": msgs,
diff --git a/crates/termlink-sidecar/src/pull.rs b/crates/termlink-sidecar/src/pull.rs
index 93a14f625..f2364da41 100644
--- a/crates/termlink-sidecar/src/pull.rs
+++ b/crates/termlink-sidecar/src/pull.rs
@@ -242,7 +242,11 @@ impl Inner {
         let resp = self.rpc("hub.version", json!({})).map_err(PinError::Unreachable)?;
         let v = unwrap_result(resp).map_err(PinError::Unreachable)?;
         let found = v.get("hub_id").and_then(Value::as_str).map(str::to_owned);
+        crate::hub_key_pin::observe_hub_id(&self.store, &self.cfg.declared_hub_id, found.as_deref());
         if found.as_deref() == Some(self.cfg.declared_hub_id.as_str()) {
+            // T-3366 (AL-18): pin / compare the hub's signing key; never holds mail.
+            let trusted = matches!(self.cfg.hub, termlink_protocol::TransportAddr::Unix { .. });
+            crate::hub_key_pin::observe(&self.store, &self.cfg.declared_hub_id, &v, trusted);
             Ok(())
         } else {
             Err(PinError::Mismatch { declared: self.cfg.declared_hub_id.clone(), found })
diff --git a/crates/termlink-sidecar/src/send.rs b/crates/termlink-sidecar/src/send.rs
index 46372a1ef..7473be176 100644
--- a/crates/termlink-sidecar/src/send.rs
+++ b/crates/termlink-sidecar/src/send.rs
@@ -383,7 +383,11 @@ impl Inner {
         let resp = self.rpc("hub.version", json!({})).map_err(PinError::Unreachable)?;
         let v = unwrap_result(resp).map_err(PinError::Unreachable)?;
         let found = v.get("hub_id").and_then(Value::as_str).map(str::to_owned);
+        crate::hub_key_pin::observe_hub_id(&self.store, &self.cfg.declared_hub_id, found.as_deref());
         if found.as_deref() == Some(self.cfg.declared_hub_id.as_str()) {
+            // T-3366 (AL-18): pin / compare the hub's signing key; never holds mail.
+            let trusted = matches!(self.cfg.hub, termlink_protocol::TransportAddr::Unix { .. });
+            crate::hub_key_pin::observe(&self.store, &self.cfg.declared_hub_id, &v, trusted);
             Ok(())
         } else {
             Err(PinError::Mismatch { declared: self.cfg.declared_hub_id.clone(), found })
diff --git a/runme.sh b/runme.sh
index ef165f9ed..e2b9927ab 100755
--- a/runme.sh
+++ b/runme.sh
@@ -701,6 +701,8 @@ install_termlink() {
         if [ "$DRY_RUN" = "1" ]; then say "  [DRY]   would build and install $TL_SRC -> $TL_DESTS"; DONE=$((DONE+1)); return; fi
         say "  FAILED  built binary missing: $TL_SRC"; FAILED=$((FAILED+1)); return
     fi
+    # T-3366: migrate hub.id with the NEW binary before it replaces the hub's (review r1 #4).
+    adopt_hub_id "$TL_SRC" || { say "  FAILED  not installing: the new binary would refuse to start this hub"; FAILED=$((FAILED+1)); return; }
     local want dest first=1
     want="$("$TL_SRC" --version 2>/dev/null)"
     for dest in $TL_DESTS; do
@@ -714,6 +716,57 @@ install_termlink() {
     done
 }
 
+# T-3366 (AL-17 legacy migration): a hub binary with T-3366 refuses to start beside existing
+# state without hub.id (FCW-79). `adopt_hub_id [binary]` makes <binary> write hub.id (the
+# current certificate's first 16 hex, the id every inbox already uses) and hub.signing.key,
+# only on a pre-T-3366 directory; a directory with one file but not the other is refused by
+# adopt-id itself (review T-3366 r1 #1). It runs (a) in step 4 with the NEWLY BUILT binary
+# BEFORE it is installed at the hub's path, so a crash/auto-restart between install and step 5
+# cannot start a binary that would refuse (review r1 #4), and (b) again before every restart.
+# The running hub's served id is passed as --expect, so a differing id is never written; the
+# key must be owned by the hub process's user (FCW-82 would refuse it otherwise). Sets HUB_ID0.
+# Returns 1 (caller must not install/restart) when anything cannot be verified.
+# A binary that predates T-3366 (clap: "unrecognized subcommand") has nothing to migrate.
+seam HUB_RT RUNME_HUB_RUNTIME_DIR "/var/lib/termlink"
+seam HUB_PROC RUNME_HUB_PROC "/proc"
+seam HUB_SYSTEMCTL RUNME_HUB_SYSTEMCTL "systemctl"
+adopt_hub_id() {
+    local bin="${1:-$HUB_BIN}" help served out line pid kuid puid
+    HUB_ID0=""
+    [ -e "$HUB_RT/hub.secret" ] || [ -e "$HUB_RT/hub.id" ] || { say "  note    no hub state in $HUB_RT — nothing to migrate"; return 0; }
+    if ! help="$("$bin" hub adopt-id --help 2>&1)"; then
+        case "$help" in
+          *"unrecognized subcommand"*)
+            say "  note    $bin has no 'hub adopt-id' (pre-T-3366): no hub.id needed"; return 0 ;;
+        esac
+        say "  FAILED  cannot tell whether $bin supports 'hub adopt-id': $(printf '%s' "$help" | tail -1)"; return 1
+    fi
+    served=""
+    if [ -S "$HUB_RT/hub.sock" ]; then
+        if ! served="$("$bin" sidecar hub-id --hub-socket "$HUB_RT/hub.sock" 2>/dev/null)" || ! [[ "$served" =~ ^[0-9a-f]{16}$ ]]; then
+            say "  FAILED  the running hub's id cannot be read from $HUB_RT/hub.sock — not migrating blind"; return 1
+        fi
+    fi
+    if [ "$DRY_RUN" = "1" ]; then
+        say "  [DRY]   $(TERMLINK_RUNTIME_DIR="$HUB_RT" "$bin" hub adopt-id --dry-run ${served:+--expect "$served"} 2>&1 | head -1)"; return 0
+    fi
+    if ! out="$(TERMLINK_RUNTIME_DIR="$HUB_RT" "$bin" hub adopt-id ${served:+--expect "$served"} 2>&1)"; then
+        say "  FAILED  hub adopt-id refused in $HUB_RT: $(printf '%s' "$out" | tail -1)"; return 1
+    fi
+    HUB_ID0="$(tr -d '\n' < "$HUB_RT/hub.id" 2>/dev/null)"
+    if ! [[ "$HUB_ID0" =~ ^[0-9a-f]{16}$ ]] || [ ! -s "$HUB_RT/hub.signing.key" ]; then
+        say "  FAILED  hub adopt-id ran but $HUB_RT/hub.id or hub.signing.key is not on disk"; HUB_ID0=""; return 1
+    fi
+    pid="$("$HUB_SYSTEMCTL" show -p MainPID --value termlink-hub 2>/dev/null)"
+    if [ -n "$pid" ] && [ "$pid" != "0" ] && [ -d "$HUB_PROC/$pid" ]; then
+        kuid="$(stat -c %u "$HUB_RT/hub.signing.key")"; puid="$(stat -c %u "$HUB_PROC/$pid")"
+        if [ "$kuid" != "$puid" ]; then
+            say "  FAILED  hub.signing.key is owned by uid $kuid but the hub runs as uid $puid (it would refuse it, FCW-82)"; return 1
+        fi
+    fi
+    while IFS= read -r line; do say "  adopt   $line"; done <<< "$out"
+}
+
 head2 "4. termlink binary — every copy on this host (T-3287, T-3289)"
 install_termlink
 
@@ -1229,6 +1282,7 @@ restart_hub() {
     if [ "$exe" = "$HUB_BIN" ]; then
         say "  skip    hub (pid $pid) already runs the installed binary: $exe"; SKIPPED=$((SKIPPED+1)); return
     fi
+    adopt_hub_id || { FAILED=$((FAILED+1)); return; }
     if [ "$DRY_RUN" = "1" ]; then
         say "  [DRY]   would restart $HUB_UNIT (pid $pid runs '${exe:-unknown}', installed is $HUB_BIN)"; DONE=$((DONE+1)); return
     fi
@@ -1253,6 +1307,15 @@ restart_hub() {
     [ "$nexe" = "$HUB_BIN" ] || problems="$problems exe='${nexe:-none}';"
     [ "$(hub_hash hub.secret)" = "$sec0" ] || problems="$problems hub.secret-CHANGED(peers-need-reauth);"
     [ "$(hub_hash hub.cert.pem)" = "$crt0" ] || problems="$problems hub.cert-CHANGED(tofu-drift);"
+    # T-3366: the id the NEW hub serves must be the adopted one (a changed id renames every inbox).
+    if [ -n "$HUB_ID0" ]; then
+        local served1=""
+        for i in $(seq 1 "$HUB_WAIT"); do
+            served1="$("$HUB_BIN" sidecar hub-id --hub-socket "$HUB_RT/hub.sock" 2>/dev/null)" && [ -n "$served1" ] && break
+            nap 1
+        done
+        [ "$served1" = "$HUB_ID0" ] || problems="$problems hub.id-CHANGED(was $HUB_ID0, serves '${served1:-unreadable}');"
+    fi
     # T-3450: systemd reports the unit active before the new hub has written its pidfile and
     # socket, so one read of `hub status` raced the startup (a false FAILED on 2026-10-17 at
     # load ~30). Re-read within the same bounded window; fail only if it never reads running.
@@ -1264,7 +1327,7 @@ restart_hub() {
     done
     [ "$ok_status" = 1 ] || problems="$problems hub-status-not-running-as-$npid;"
     if [ -z "$problems" ]; then
-        say "  OK      restarted and verified: pid $pid -> $npid, exe $nexe ($("$HUB_BIN" --version 2>/dev/null)); secret + cert unchanged"; DONE=$((DONE+1))
+        say "  OK      restarted and verified: pid $pid -> $npid, exe $nexe ($("$HUB_BIN" --version 2>/dev/null)); secret + cert${HUB_ID0:+ + hub.id $HUB_ID0} unchanged"; DONE=$((DONE+1))
     else
         say "  FAILED  hub restart verification:$problems"; FAILED=$((FAILED+1))
     fi
@@ -1299,6 +1362,7 @@ install_hub_unit() {
         say "  [DRY]   would install $HUB_UNIT_SRC -> $HUB_UNIT_DEST, daemon-reload, restart $HUB_UNIT"; DONE=$((DONE+1)); return
     fi
     local old_pid; old_pid="$(hub_mainpid)"
+    adopt_hub_id || { FAILED=$((FAILED+1)); return; }   # T-3366: before any restart
     if ! install -m 644 "$HUB_UNIT_SRC" "$HUB_UNIT_DEST"; then
         say "  FAILED  could not install $HUB_UNIT_DEST"; FAILED=$((FAILED+1)); return
     fi
@@ -1480,6 +1544,35 @@ upgrade_fleet_hubs() {
             say "  FAILED  staging on $hub failed (see /tmp/.runme-deploy-$hub.log)"; FAILED=$((FAILED+1)); continue
         fi
         unitbin="$(fleet_remote "$hub" 'systemctl cat termlink-hub 2>/dev/null | sed -n "s/^ExecStart=\([^ ]*\).*/\1/p" | head -1' 2>/dev/null | tr -d '\r' | tail -1)"
+        # T-3366: the staged binary refuses to start beside existing state without hub.id
+        # (FCW-79). Migrate first, against the RUNNING hub's own runtime dir (from its
+        # environment); the adopted id must be the one fleet doctor already reports
+        # (no inbox renamed). Not confirmed => no swap.
+        local id0 adopt
+        id0="$(fleet_hub_field "$hub" hub_id)"
+        if ! [[ "$id0" =~ ^[0-9a-f]{16}$ ]]; then
+            say "  FAILED  $hub: fleet doctor states no hub_id for it — cannot migrate against the served id, not swapping"; FAILED=$((FAILED+1)); continue
+        fi
+        # --expect: adopt-id refuses, writing nothing, if the id it would write differs from the
+        # served one (review r1 #4); the key must belong to the hub process's user (FCW-82).
+        adopt="$(fleet_remote "$hub" "if ! h=\$($STAGED hub adopt-id --help 2>&1); then case \"\$h\" in *'unrecognized subcommand'*) echo RUNME-ADOPT-NA ;; *) echo RUNME-ADOPT-PROBEFAIL ;; esac; exit 0; fi; p=\$(pgrep -o -f '^[^ ]*termlink hub start'); rt=\$(tr '\\0' '\\n' </proc/\$p/environ 2>/dev/null | sed -n 's/^TERMLINK_RUNTIME_DIR=//p'); if [ -z \"\$rt\" ] || [ ! -d \"\$rt\" ]; then echo RUNME-ADOPT-NORT; exit 0; fi; TERMLINK_RUNTIME_DIR=\$rt $STAGED hub adopt-id --expect $id0 >/dev/null 2>&1 || { echo RUNME-ADOPT-REFUSED; exit 0; }; [ \"\$(stat -c %u \$rt/hub.signing.key)\" = \"\$(stat -c %u /proc/\$p)\" ] || { echo RUNME-ADOPT-UID; exit 0; }; echo RUNME-ADOPT-OK \$(tr -d '\\n' < \$rt/hub.id) \$rt" 2>&1 | tr -d '\r')"
+        case "$adopt" in
+          *RUNME-ADOPT-NA*) id0="" ;;
+          *RUNME-ADOPT-OK*)
+            adopt="$(printf '%s\n' "$adopt" | sed -n 's/.*RUNME-ADOPT-OK \([0-9a-f]\{16\}\) .*/\1/p' | tail -1)"
+            if [ "$adopt" != "$id0" ]; then
+                say "  FAILED  $hub: hub.id on disk '${adopt:-unreadable}' is not the id it serves ('$id0') — not swapping"; FAILED=$((FAILED+1)); continue
+            fi
+            say "  adopt   $hub hub.id $adopt (unchanged)" ;;
+          *RUNME-ADOPT-NORT*)
+            say "  FAILED  $hub: cannot confirm the running hub's TERMLINK_RUNTIME_DIR, so hub.id cannot be migrated — not swapping (the new hub would refuse to start)"; FAILED=$((FAILED+1)); continue ;;
+          *RUNME-ADOPT-REFUSED*)
+            say "  FAILED  $hub: hub adopt-id refused (id would differ from the served '$id0', or a migrated hub lost a file) — nothing written, not swapping"; FAILED=$((FAILED+1)); continue ;;
+          *RUNME-ADOPT-UID*)
+            say "  FAILED  $hub: hub.signing.key is not owned by the hub process's user (it would refuse it) — not swapping"; FAILED=$((FAILED+1)); continue ;;
+          *)
+            say "  FAILED  $hub: hub adopt-id did not confirm: $(printf '%s' "$adopt" | tail -1) — not swapping"; FAILED=$((FAILED+1)); continue ;;
+        esac
         if [ -n "$unitbin" ]; then
             # G-070: a systemd-supervised hub is restarted THROUGH its unit. The
             # restart is delayed and detached so this exec (which rides on that
@@ -1498,8 +1591,11 @@ upgrade_fleet_hubs() {
             [ "$(fleet_hub_field "$hub" hub_version)" = "$want" ] && break; nap 1
         done
         have="$(fleet_hub_field "$hub" hub_version)"; st="$(fleet_hub_field "$hub" status)"
-        if [ "$have" = "$want" ] && [ "$st" = "ok" ] && $TOFU "$addr" >/dev/null 2>&1; then
-            say "  OK      upgraded and verified: $hub serves $have; fleet doctor ok (secret still authenticates); tofu verify ok (cert unchanged)"; DONE=$((DONE+1))
+        local id1=""; [ -n "$id0" ] && id1="$(fleet_hub_field "$hub" hub_id)"
+        if [ -n "$id0" ] && [ "$id1" != "$id0" ]; then
+            say "  FAILED  $hub after deploy: hub_id '${id1:-none}' is not '$id0' (T-3366: the id must survive the restart)"; FAILED=$((FAILED+1))
+        elif [ "$have" = "$want" ] && [ "$st" = "ok" ] && $TOFU "$addr" >/dev/null 2>&1; then
+            say "  OK      upgraded and verified: $hub serves $have; fleet doctor ok (secret still authenticates); tofu verify ok (cert unchanged)${id0:+; hub_id $id0 unchanged}"; DONE=$((DONE+1))
         else
             say "  FAILED  $hub after deploy: version='${have:-none}' status='${st:-none}' (want $want/ok), or tofu verify failed — see /tmp/.runme-deploy-$hub.log"; FAILED=$((FAILED+1))
         fi
```

## 12. Tests: real-hub and sidecar tests (at adf4555e0)
```rust
//! T-3366 (arc-011 P-10, B-1): E-36 / E-37 hub half / E-166 hub half at L4 — a real hub binary
//! in a throwaway runtime dir. `hub.version` must answer the minted `hub.id`, unchanged across
//! a restart and a certificate rotation; a legacy dir (secret, no id) must refuse to start until
//! `termlink hub adopt-id` gives it the certificate-derived id.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use assert_cmd::cargo;
use termlink_test_utils::{ProcessGuard, TestDir};

fn cmd(dir: &TestDir, args: &[&str]) -> Command {
    let mut c = termlink_test_utils::termlink_cmd(cargo::cargo_bin!("termlink"), &dir.path);
    c.env("HOME", &dir.path).env("TERMLINK_IDENTITY_DIR", dir.path.join("id")).args(args).stdin(Stdio::null());
    c
}

fn hub_version(sock: &Path) -> serde_json::Value {
    let mut s = UnixStream::connect(sock).expect("connect hub");
    s.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    s.write_all(b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"hub.version\",\"params\":{}}\n").unwrap();
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).unwrap();
    let v: serde_json::Value = serde_json::from_str(&line).expect("json reply");
    v["result"].clone()
}

/// Start a hub (optionally with TCP), wait for its socket, return the guard.
fn start(dir: &TestDir, tag: &str, tcp: bool) -> ProcessGuard {
    let port;
    let mut args = vec!["hub", "start"];
    if tcp {
        let l = std::net::TcpListener::bind("<ip>:0").unwrap();
        port = format!("<ip>:{}", l.local_addr().unwrap().port());
        drop(l);
        args.push("--tcp");
        args.push(&port);
    }
    // SI-25 scan input: the hub's own output is kept in the runtime dir.
    let log = |n: &str| std::fs::OpenOptions::new().create(true).append(true).open(dir.path.join(n)).unwrap();
    let child = cmd(dir, &args).env("RUST_LOG", "info").stdout(log("hub.out.log")).stderr(log("hub.err.log")).spawn().expect("spawn hub");
    let g = ProcessGuard::new(child, tag);
    let deadline = Instant::now() + Duration::from_secs(20);
    while !(dir.path.join("hub.sock").exists() && dir.path.join("hub.pid").exists()) {
        assert!(Instant::now() < deadline, "{tag}: hub socket never appeared");
        std::thread::sleep(Duration::from_millis(50));
    }
    // the socket file may exist a moment before accept; retry one connect
    let deadline = Instant::now() + Duration::from_secs(10);
    while UnixStream::connect(dir.path.join("hub.sock")).is_err() {
        assert!(Instant::now() < deadline, "{tag}: hub never accepted");
        std::thread::sleep(Duration::from_millis(50));
    }
    g
}

fn stop(g: ProcessGuard, dir: &TestDir) {
    drop(g);
    let deadline = Instant::now() + Duration::from_secs(10);
    while dir.path.join("hub.pid").exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = std::fs::remove_file(dir.path.join("hub.pid"));
    let _ = std::fs::remove_file(dir.path.join("hub.sock"));
}

fn read_id(dir: &TestDir) -> String {
    std::fs::read_to_string(dir.path.join("hub.id")).unwrap().trim_end().to_string()
}

#[test]
fn e36_socket_only_hub_has_a_minted_id_and_hub_version_reads_the_file() {
    let dir = TestDir::new("t3366-sock");
    let g = start(&dir, "t3366-sock-hub", false);
    let v = hub_version(&dir.path.join("hub.sock"));
    let id = read_id(&dir);
    assert_eq!(v["hub_id"], id.as_str(), "{v}");
    assert!(v["signing_key"].as_str().is_some_and(|k| k.starts_with("ed25519:") && k.len() == 8 + 64), "{v}");
    stop(g, &dir);
}

#[test]
fn e36_e37_restart_and_certificate_rotation_keep_id_and_signing_key() {
    let dir = TestDir::new("t3366-rot");
    let g = start(&dir, "t3366-rot-1", true);
    let v1 = hub_version(&dir.path.join("hub.sock"));
    stop(g, &dir);
    // rotate the certificate: remove it; the next TCP start generates a new one
    let fp_before = std::fs::read(dir.path.join("hub.cert.pem")).unwrap();
    std::fs::remove_file(dir.path.join("hub.cert.pem")).unwrap();
    std::fs::remove_file(dir.path.join("hub.key.pem")).unwrap();
    let g = start(&dir, "t3366-rot-2", true);
    let v2 = hub_version(&dir.path.join("hub.sock"));
    assert_ne!(std::fs::read(dir.path.join("hub.cert.pem")).unwrap(), fp_before, "the certificate must really rotate");
    assert_ne!(v1["hub_instance_id"], v2["hub_instance_id"], "instance id follows the certificate");
    assert_eq!(v1["hub_id"], v2["hub_id"], "hub_id survives rotation: {v1} vs {v2}");
    assert_eq!(v1["signing_key"], v2["signing_key"], "signing key survives rotation");
    stop(g, &dir);
    // SI-25 (E-37.4): the private seed appears in no file of the runtime dir but its own,
    // the hub's stdout/stderr logs included.
    let seed = std::fs::read_to_string(dir.path.join("hub.signing.key")).unwrap().trim().to_string();
    assert!(std::fs::metadata(dir.path.join("hub.err.log")).unwrap().len() > 0, "the scan must see real hub output");
    let mut stack = vec![dir.path.clone()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() { stack.push(p); continue; }
            if p.file_name().is_some_and(|n| n == "hub.signing.key") { continue; }
            let body = std::fs::read(&p).unwrap_or_default();
            assert!(!String::from_utf8_lossy(&body).contains(&seed), "seed found in {}", p.display());
        }
    }
}

#[test]
fn e36_fcw79_legacy_dir_refuses_until_adopt_id_then_keeps_the_certificate_id() {
    let dir = TestDir::new("t3366-legacy");
    // build a legacy dir: a TCP start makes secret + cert, then remove the T-3366 files
    let g = start(&dir, "t3366-legacy-1", true);
    let legacy_id = hub_version(&dir.path.join("hub.sock"))["hub_instance_id"]
        .as_str()
        .unwrap()
        .trim_start_matches("sha256:")[..16]
        .to_string();
    stop(g, &dir);
    std::fs::remove_file(dir.path.join("hub.id")).unwrap();
    std::fs::remove_file(dir.path.join("hub.signing.key")).unwrap();

    let out = cmd(&dir, &["hub", "start"]).output().unwrap();
    let se = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "must refuse: {se}");
    assert!(se.contains("hub.id missing beside existing state"), "{se}");
    assert!(!dir.path.join("hub.id").exists(), "the refusal writes no file");

    let out = cmd(&dir, &["hub", "adopt-id", "--dry-run", "--json"]).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!((v["hub_id"].as_str(), v["id_source"].as_str()), (Some(legacy_id.as_str()), Some("certificate")));
    assert!(!dir.path.join("hub.id").exists(), "dry-run writes nothing");

    let out = cmd(&dir, &["hub", "adopt-id", "--json"]).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(read_id(&dir), legacy_id, "no inbox renamed");
    let again: serde_json::Value =
        serde_json::from_slice(&cmd(&dir, &["hub", "adopt-id", "--json"]).output().unwrap().stdout).unwrap();
    assert_eq!((again["id_source"].as_str(), again["key_generated"].as_bool()), (Some("kept"), Some(false)));

    let g = start(&dir, "t3366-legacy-2", true);
    assert_eq!(hub_version(&dir.path.join("hub.sock"))["hub_id"], legacy_id.as_str());
    stop(g, &dir);
}
//! T-3366 (arc-011 P-10, B-1): E-37 sidecar half and E-166 (FCW-84) against a throwaway hub.
//! The sidecar pins the hub's signing key on first verified contact, keeps it across a hub
//! restart, and after a key swap refuses every hub-signed object with a needs-attention entry
//! naming both fingerprints. It never re-pins silently and never holds mail for it.

use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use termlink_sidecar::hub_key_pin::{self, KeyState};
use termlink_sidecar::send::*;
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;

fn sender_cfg(e: &Estate, hub_id: &str) -> SendConfig {
    SendConfig {
        agent: "agent-a".into(),
        instance: "inst-a".into(),
        identity: e.identity("a").unwrap(),
        hub: e.hub_addr(),
        declared_hub_id: hub_id.into(),
        queue_path: e.root().join("outbox-a.sqlite"),
        timeout: Duration::from_secs(5),
        pump_interval: None,
    }
}

fn hub_version(e: &Estate) -> Value {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let resp = rt
        .block_on(termlink_session::client::rpc_call_addr_with_timeout(
            &e.hub_addr(),
            "hub.version",
            json!({}),
            Duration::from_secs(10),
        ))
        .unwrap();
    termlink_session::client::unwrap_result(resp).unwrap()
}

fn attention_kinds(store: &Store) -> Vec<(String, String)> {
    store
        .attention_entries()
        .iter()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .map(|v| (v["kind"].as_str().unwrap_or("").to_string(), v["detail"].as_str().unwrap_or("").to_string()))
        .collect()
}

#[test]
fn e37_first_contact_pins_restart_keeps_and_a_swapped_key_is_refused_and_named() {
    let mut e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), "agent-a").unwrap());
    let sender = Sender::new(sender_cfg(&e, &id), store.clone()).unwrap();

    // first verified contact (local socket): pinned, and said so
    assert!(sender.check_pin().is_ok());
    let k1 = hub_version(&e)["signing_key"].as_str().unwrap().to_string();
    assert_eq!(store.read_state(&hub_key_pin::pin_state_name(&id)).unwrap().as_deref(), Some(k1.as_str()));
    assert!(attention_kinds(&store).iter().any(|(k, d)| k == "hub-signing-key-pinned" && d.contains(&k1)));
    assert_eq!(hub_key_pin::verified_hub_key(&store, &id, &hub_version(&e)).as_deref(), Some(k1.as_str()));

    // restart: same key, still pinned, nothing changed
    e.restart_hub().unwrap();
    assert!(sender.check_pin().is_ok());
    assert_eq!(hub_key_pin::status(&store, Some(&id))["state"], "pinned");
    assert!(!attention_kinds(&store).iter().any(|(k, _)| k == "hub-signing-key-changed"));

    // swap the hub's key file (a second valid key), restart
    e.stop_hub();
    let key = e.runtime_dir().join("hub.signing.key");
    std::fs::write(&key, format!("{}\n", "7".repeat(64))).unwrap();
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&key, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    e.restart_hub().unwrap();
    let k2 = hub_version(&e)["signing_key"].as_str().unwrap().to_string();
    assert_ne!(k1, k2, "the fixture must really swap the key");

    // mail is not held (the hub id still matches) ...
    assert!(sender.check_pin().is_ok());
    // ... but every hub-signed object is refused, the pin is unchanged, and the entry names both
    assert_eq!(hub_key_pin::verified_hub_key(&store, &id, &hub_version(&e)), None);
    assert_eq!(store.read_state(&hub_key_pin::pin_state_name(&id)).unwrap().as_deref(), Some(k1.as_str()));
    let changed: Vec<_> = attention_kinds(&store).into_iter().filter(|(k, _)| k == "hub-signing-key-changed").collect();
    assert_eq!(changed.len(), 1, "{changed:?}");
    assert!(changed[0].1.contains(&k1) && changed[0].1.contains(&k2), "{changed:?}");
    let st = hub_key_pin::status(&store, Some(&id));
    assert_eq!((st["state"].as_str(), st["pinned"].as_str(), st["offered"].as_str()), (Some("mismatch"), Some(k1.as_str()), Some(k2.as_str())));

    // coalesced: a second contact the same day adds no second entry
    assert!(sender.check_pin().is_ok());
    assert_eq!(attention_kinds(&store).iter().filter(|(k, _)| k == "hub-signing-key-changed").count(), 1);

    // SI-25: the hub's private seed appears nowhere in the sidecar's store
    let seed = "7".repeat(64);
    for entry in walk(&e.store_root()) {
        let body = std::fs::read(&entry).unwrap_or_default();
        assert!(!String::from_utf8_lossy(&body).contains(&seed), "seed found in {}", entry.display());
    }
}

fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = vec![];
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() { out.extend(walk(&p)); } else { out.push(p); }
        }
    }
    out
}

#[test]
fn e166_fcw84_untrusted_first_contact_pins_nothing_and_no_offer_changes_nothing() {
    let e = Estate::create_without_hub().unwrap();
    let store = Store::open(&e.store_root(), "agent-a").unwrap();
    let fp = format!("ed25519:{}", "a".repeat(64));
    let v = json!({"hub_id": "0123456789abcdef", "signing_key": fp});

    // not over the local socket: no pin, unverified, hub-signed data refused
    assert_eq!(hub_key_pin::observe(&store, "0123456789abcdef", &v, false), KeyState::Unverified(fp.clone()));
    assert_eq!(store.read_state(&hub_key_pin::pin_state_name("0123456789abcdef")).unwrap(), None);
    assert_eq!(hub_key_pin::verified_hub_key(&store, "0123456789abcdef", &v), None);
    assert_eq!(hub_key_pin::status(&store, Some("0123456789abcdef"))["state"], "unverified");

    // a hub that predates T-3366 offers no key: nothing pinned, nothing refused as a mismatch
    let old = json!({"hub_id": "0123456789abcdef"});
    assert_eq!(hub_key_pin::observe(&store, "0123456789abcdef", &old, true), KeyState::NotOffered);
    assert_eq!(store.read_state(&hub_key_pin::pin_state_name("0123456789abcdef")).unwrap(), None);

    // a malformed offer is never pinned, even over the trusted path
    let bad = json!({"hub_id": "0123456789abcdef", "signing_key": "ed25519:nothex"});
    assert!(matches!(hub_key_pin::observe(&store, "0123456789abcdef", &bad, true), KeyState::Unverified(_)));
    assert_eq!(store.read_state(&hub_key_pin::pin_state_name("0123456789abcdef")).unwrap(), None);

    // and the trusted path does pin a well-formed offer
    assert_eq!(hub_key_pin::observe(&store, "0123456789abcdef", &v, true), KeyState::NewlyPinned(fp.clone()));
    assert_eq!(hub_key_pin::observe(&store, "0123456789abcdef", &v, true), KeyState::Pinned(fp));
}

// Review r1 #2: an UNREADABLE pin is never treated as "no pin" (no re-pin over it).
#[test]
fn e37_unreadable_pin_is_not_replaced() {
    let e = Estate::create_without_hub().unwrap();
    let store = Store::open(&e.store_root(), "agent-a").unwrap();
    let h = "0123456789abcdef";
    // the pin's state file is a directory: reading it errors
    std::fs::create_dir_all(store.dir().join(format!("state.{}", hub_key_pin::pin_state_name(h)))).unwrap();
    let v = json!({"hub_id": h, "signing_key": format!("ed25519:{}", "b".repeat(64))});
    assert!(matches!(hub_key_pin::observe(&store, h, &v, true), KeyState::Unverified(_)));
    assert!(store.dir().join(format!("state.{}", hub_key_pin::pin_state_name(h))).is_dir(), "not replaced");
    assert_eq!(hub_key_pin::verified_hub_key(&store, h, &v), None);
    assert!(attention_kinds(&store).iter().any(|(k, _)| k == "hub-signing-key-pin-unreadable"));
}

// Review r1 #2: a pinned hub that stops offering its key is refused and named.
#[test]
fn e37_withdrawn_key_is_named_and_refused() {
    let e = Estate::create_without_hub().unwrap();
    let store = Store::open(&e.store_root(), "agent-a").unwrap();
    let h = "0123456789abcdef";
    let fp = format!("ed25519:{}", "c".repeat(64));
    hub_key_pin::observe(&store, h, &json!({"hub_id": h, "signing_key": fp}), true);
    let old = json!({"hub_id": h});
    assert_eq!(hub_key_pin::observe(&store, h, &old, true), KeyState::NotOffered);
    assert_eq!(hub_key_pin::verified_hub_key(&store, h, &old), None);
    assert!(attention_kinds(&store).iter().any(|(k, d)| k == "hub-signing-key-withdrawn" && d.contains(&fp)));
}

// E-126: status `hub {id, declared, match}`; control: another id reads match=false.
#[test]
fn e126_status_states_the_answering_hub_id_and_whether_it_matches() {
    let mut e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), "agent-a").unwrap());
    assert_eq!(hub_key_pin::hub_status(&store, Some(&id))["id"], Value::Null, "not observed yet");
    let sender = Sender::new(sender_cfg(&e, &id), store.clone()).unwrap();
    assert!(sender.check_pin().is_ok());
    let st = hub_key_pin::hub_status(&store, Some(&id));
    assert_eq!((st["id"].as_str(), st["declared"].as_str(), st["match"].as_bool()), (Some(id.as_str()), Some(id.as_str()), Some(true)));
    e.restart_hub().unwrap();
    assert!(sender.check_pin().is_ok());
    assert_eq!(hub_key_pin::hub_status(&store, Some(&id))["match"], true, "same id after restart");
    // control: a sidecar declared for another hub sees this hub's id, match false
    let other = Sender::new(sender_cfg(&e, "ffffffffffffffff"), store.clone()).unwrap();
    assert!(other.check_pin().is_err());
    let st = hub_key_pin::hub_status(&store, Some("ffffffffffffffff"));
    assert_eq!((st["id"].as_str(), st["match"].as_bool()), (Some(id.as_str()), Some(false)));
}

// AR-1 LIMIT (E-37.5): a rogue key present at the FIRST trusted contact is pinned. Recorded as
// the design's known limit, not a pass: the pin defends against a later swap only.
#[test]
fn e37_ar1_limit_rogue_key_at_first_contact_is_pinned() {
    let e = Estate::create_without_hub().unwrap();
    let store = Store::open(&e.store_root(), "agent-a").unwrap();
    let rogue = format!("ed25519:{}", "d".repeat(64));
    let st = hub_key_pin::observe(&store, "0123456789abcdef", &json!({"hub_id": "0123456789abcdef", "signing_key": rogue}), true);
    assert_eq!(st, KeyState::NewlyPinned(rogue));
    eprintln!("LIMIT AR-1: a rogue key present at first trusted contact is pinned (by design)");
}
```

## 13. T-3366 runme fixture cases (names, at adf4555e0) and the run summary
```
1336:    ok "hub (T-3366): legacy hub gets hub.id from adopt-id BEFORE the restart; id verified unchanged after"
1341:    ok "hub (T-3366): second migration keeps the existing id (idempotent)"
1346:    ok "hub (T-3366): adopt-id refused => FAILED and NO restart (the new hub would refuse to start)"
1351:    ok "hub (T-3366): adopt-id that claims success but writes nothing => FAILED by the on-disk read, no restart"
1356:    ok "hub (T-3366): a pre-T-3366 binary restarts as before (nothing to migrate)"
1361:    ok "hub (T-3366): an id changed by the restart => FAILED naming it"
1366:    ok "hub (T-3366): a binary whose adopt-id probe fails for another reason => FAILED, no restart (never taken as pre-T-3366)"
1371:    ok "hub (T-3366): --dry-run previews the migration and writes nothing"
1379:    ok "hub (T-3366 r1): step 4 migrates hub.id with the NEW binary before installing it (no crash window)"
1385:    ok "hub (T-3366 r1): step 4 migration refused => FAILED, the new binary is NOT installed"
1393:    ok "hub (T-3366 r1): adopted id would differ from the RUNNING hub's served id => refused, nothing written, no restart"
1399:    ok "hub (T-3366 r1): a migrated hub that lost its key => FAILED, no re-key, no restart"
1407:        ok "hub (T-3366 r1): key not owned by the hub's user => FAILED, no restart (FCW-82 would refuse it)"
1409:else ok "hub (T-3366 r1): key-owner case needs root to chown; skipped as non-root"; fi
1473:    ok "fleet (T-3366): adopt-id runs remotely BEFORE the restart; the served id is verified unchanged after"
1478:    ok "fleet (T-3366): running hub's runtime dir unconfirmable => FAILED, no swap"
1483:    ok "fleet (T-3366): adopted id differs from the served id (an inbox would be renamed) => FAILED, no swap"
1488:    ok "fleet (T-3366): adopt-id refused => FAILED, no --swap-restart"
1493:    ok "fleet (T-3366): an id changed by the restart => FAILED naming both"
1498:    ok "fleet (T-3366 r1): no served hub_id to check against => FAILED, no swap (never skipped)"
1503:    ok "fleet (T-3366 r1): remote adopt-id refused (--expect mismatch / lost file) => FAILED, no swap"
1508:    ok "fleet (T-3366 r1): remote key owned by another user => FAILED, no swap"
T-3052 fixtures: 383 passed, 0 failed
```

## 14. Test run result lines (round-2 source)
```
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 582 filtered out; finished in 0.10s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.83s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.56s
full hub+sidecar+CLI suites: all-rc=0
```

## 15. Field deploy of adf4555e0 (runme log excerpt, 2026-11-16 09:35Z, rc=0)
```
=== runme.sh start 2026-11-16T09:35:10Z  args=[]  uid=0  head=adf4555e0 ===
  adopt   hub.id cacc73ea32b121dd kept (already present) in /var/lib/termlink
  adopt   hub.signing.key kept (ed25519:ef7bed9659f22b825bba654fe2f6f3933deef08db6ec8d5c59c192386f23e030)
  OK      termlink-sidecar@010-termlink--claude-termlink.service restarted and verified: active, pid 187191 runs the installed binary
  OK      termlink-sidecar@010-termlink--claude-termlink-alt.service restarted and verified: active, pid 187502 runs the installed binary
  adopt   hub.id cacc73ea32b121dd kept (already present) in /var/lib/termlink
  adopt   hub.signing.key kept (ed25519:ef7bed9659f22b825bba654fe2f6f3933deef08db6ec8d5c59c192386f23e030)
  OK      restarted and verified: pid 3282051 -> 189778, exe /root/.cargo/bin/termlink (termlink 0.12.13+114.gadf4555e0); secret + cert + hub.id cacc73ea32b121dd unchanged
  adopt   ring20-management hub.id 22c19fedafd73da2 (unchanged)
  OK      upgraded and verified: ring20-management serves 0.12.13+114.gadf4555e0; fleet doctor ok (secret still authenticates); tofu verify ok (cert unchanged); hub_id 22c19fedafd73da2 unchanged
  adopt   ring20-dashboard hub.id 1389a831016c4bf1 (unchanged)
  OK      upgraded and verified: ring20-dashboard serves 0.12.13+114.gadf4555e0; fleet doctor ok (secret still authenticates); tofu verify ok (cert unchanged); hub_id 1389a831016c4bf1 unchanged
  adopt   greenfield hub.id aef925c6d2f4ac54 (unchanged)
  OK      upgraded and verified: greenfield serves 0.12.13+114.gadf4555e0; fleet doctor ok (secret still authenticates); tofu verify ok (cert unchanged); hub_id aef925c6d2f4ac54 unchanged
=== runme.sh finished rc=0 at 2026-11-16T09:46:58Z — log: /opt/termlink/.context/working/runme-logs/runme-20261116T093510Z-55908.log ===
```

## 16. Live after the deploy: fleet doctor --json (hub, version, hub_id, signing_key)
```
greenfield 0.12.13+114.gadf4555e0 aef925c6d2f4ac54 ed25519:cf4618885fb9fd26da0246a693672c3f5b128a07cdf15984fb63f7decd0f5f6d
laptop-141 None None None
local-hub 0.12.13+114.gadf4555e0 cacc73ea32b121dd ed25519:ef7bed9659f22b825bba654fe2f6f3933deef08db6ec8d5c59c192386f23e030
local-test 0.12.13+114.gadf4555e0 cacc73ea32b121dd ed25519:ef7bed9659f22b825bba654fe2f6f3933deef08db6ec8d5c59c192386f23e030
node1-pool 0.12.1084 925d947ce6e94b74 None
ring20-dashboard 0.12.13+114.gadf4555e0 1389a831016c4bf1 ed25519:1edf80fefdcca76ce63efd144f916a8656ef479b4b2b0e66bf314ef087d85b15
ring20-management 0.12.13+114.gadf4555e0 22c19fedafd73da2 ed25519:f2cad3d44b5f485ee988f1168f6d3e8eaa5fdb97243a337128b63daf51719c83
workstation-107-public 0.12.13+114.gadf4555e0 cacc73ea32b121dd ed25519:ef7bed9659f22b825bba654fe2f6f3933deef08db6ec8d5c59c192386f23e030
```

## 17. Live W1 sidecar pin state on the local hub (claude-termlink)
```
state.hub-id-seen-cacc73ea32b121dd: cacc73ea32b121dd
state.hub-key-pin-cacc73ea32b121dd: ed25519:ef7bed9659f22b825bba654fe2f6f3933deef08db6ec8d5c59c192386f23e030
state.hub-key-seen-cacc73ea32b121dd: pinned ed25519:ef7bed9659f22b825bba654fe2f6f3933deef08db6ec8d5c59c192386f23e030
{"detail":"hub cacc73ea32b121dd signing key pinned: ed25519:ef7bed9659f22b825bba654fe2f6f3933deef08db6ec8d5c59c192386f23e030","kind":"hub-signing-key-pinned","ts_ms":1794821868014}
```
