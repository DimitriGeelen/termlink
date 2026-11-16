# T-3366 close review bundle — confirmation round (operator ruling A; code under review = deployed commit fe72c850c)

Rounds 1-3: one seat each (claude-code), FAIL each time; opencode never ran (usage limit). The operator ruled one confirmation round with BOTH seats on the final commit (Decisions). Round-3 findings are answered in section 2; round 3 confirmed every round-2 row except its item 1, now fixed. Rounds 1 and 2: r1/r2-dispositions.md.

## 1. Acceptance criteria and decisions (verbatim)
```
## Acceptance Criteria

### Agent
- [x] Canonical id persisted in the runtime dir and survives cert regeneration (unit e36_certificate_rotation…; L4 t3366_hub_identity rotation test; mutant m1 killed)
- [x] Existing hubs migrate to their current fingerprint-derived id (no inbox renamed) (runme-20261116T073957Z rc=0: local cacc73ea32b121dd, .122 22c19fedafd73da2, .121 1389a831016c4bf1, greenfield aef925c6d2f4ac54, each adopted before the restart and verified unchanged after; live hub.version serves hub_id + signing_key)
- [x] The B-1 specs pass, including the hub signing key and the `hub.version` fields (DR-1 A, DR-2 A) (docs/reports/T-3366-evidence.md; live `hub.version` serves `hub_id` + `signing_key`)
- [x] A re-mint beside existing state is refused (FCW-79 unit + L4 legacy test; mutant m2 killed)
- [x] The unit's fixture suite exits 0 with a PASS line per B-1 E-n and its mutant list is all killed (FP 3.1.b) (evidence table: E-36, E-37, E-126, E-166 PASS; E-38 is P-11's, E-163 not applicable, both stated; every mutant in docs/reports/T-3366-mutants.md killed, run on the deployed commit)

## Decisions`) and confirms this task's scope as plan unit **P-10** (build,
PH-2, T1 hardening): B-1, minted hub id + hub signing key + `hub.version` fields (DR-1 A, DR-2 A). Serves B-1, K-9a.
Re-scoped by PD-2: ending the `sidecar:` alias moves to the T4 decommission P-53 (T-3726, NID).

**Problem / hypothesis:** The hub's identity is its TLS fingerprint, which changes when the certificate is regenerated;
signed state (revocation list P-15, signed dead letters P-16, cards P-48) needs a stable id and a hub signing key first.

**Goal:** A canonical hub id minted once and persisted, a hub signing key, and the `hub.version` fields.

**Expected outcome:** The B-1 specs pass; existing hubs adopt the fingerprint-derived id; a re-mint beside existing
state is refused.

**Rollback:** the id file stays; clients ignore the new fields.

Depends on: gate G-0 (decision: plan approved 2026-11-12; satisfied)
Unblocks: P-11 (T-3363), P-15 (T-3703, security), P-16 (T-3704, security), P-31 (T-3713), P-48 (T-3722); part of gate G-2.

## Acceptance Criteria

### Agent
- [x] Canonical id persisted in the runtime dir and survives cert regeneration (unit e36_certificate_rotation…; L4 t3366_hub_identity rotation test; mutant m1 killed)
- [x] Existing hubs migrate to their current fingerprint-derived id (no inbox renamed) (runme-20261116T073957Z rc=0: local cacc73ea32b121dd, .122 22c19fedafd73da2, .121 1389a831016c4bf1, greenfield aef925c6d2f4ac54, each adopted before the restart and verified unchanged after; live hub.version serves hub_id + signing_key)
- [x] The B-1 specs pass, including the hub signing key and the `hub.version` fields (DR-1 A, DR-2 A) (docs/reports/T-3366-evidence.md; live `hub.version` serves `hub_id` + `signing_key`)
- [x] A re-mint beside existing state is refused (FCW-79 unit + L4 legacy test; mutant m2 killed)
- [x] The unit's fixture suite exits 0 with a PASS line per B-1 E-n and its mutant list is all killed (FP 3.1.b) (evidence table: E-36, E-37, E-126, E-166 PASS; E-38 is P-11's, E-163 not applicable, both stated; every mutant in docs/reports/T-3366-mutants.md killed, run on the deployed commit)

## Verification

env -u PROJECT_ROOT cargo test -p termlink-hub hub_identity > .context/working/.t3366-v1.out 2>&1 && grep -q 'test result: ok. 26 passed' .context/working/.t3366-v1.out
env -u PROJECT_ROOT cargo test -p termlink --test t3366_hub_identity > .context/working/.t3366-v2.out 2>&1 && grep -q 'test result: ok. 3 passed' .context/working/.t3366-v2.out
env -u PROJECT_ROOT cargo test -p termlink-sidecar --test t3366_hub_key_pin > .context/working/.t3366-v3.out 2>&1 && grep -q 'test result: ok. 9 passed' .context/working/.t3366-v3.out
grep -q 'PASS: all 26 mutants turned their test red' docs/reports/T-3366-mutants.md

## RCA




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

### 2026-11-16 — closing after three one-seat review rounds (operator ruling)
- **Chose:** A, "A, as recommended" (operator, 2026-11-16): one confirmation round on the final commit with BOTH seats once opencode's usage limit resets; close only on two PASS; a FAIL from either goes to the operator with the dispositions, not to another round.
- **Why:** the close ruling (T-3800, 2026-11-15) asks for a positive review from both seats; opencode never ran in rounds 1–3 (usage limit), and the 2–3 round cap was reached.
- **Rejected:** B close on evidence alone (departs from the ruling); C operator judges in Watchtower; D wait a day.

### 2026-11-16 — the `sidecar:` forward criterion (operator ruling)
- **Chose:** C, "C, split it into its own task" (operator, 2026-11-16). The criterion "Mail to sidecar: topics forwarded with a sender warning until the alias end release" moves verbatim in intent to **T-3809** (build, arc-011, horizon next), where it is built as a hub forward envelope carrying the original signed message.
- **Why:** one task, one deliverable: alias handling is not id minting, and PD-2 already moved the alias end to its own unit (P-53, T-3726). A plain copy would fail the receiver's signature check (the signature covers the topic), so the forward needs its own design. No `sidecar:` post in 42 days, so the split carries no delivery risk.
- **Rejected:** A build it here (keeps T-3366 and T-3704 waiting on a route idle for six weeks); B warn only (departs from R-71.a); D defer inside T-3366 (same wait as A).


```

## 2. Round-3 findings and their dispositions
# T-3366 review round 3 — dispositions (last round)

Round 3: claude-code FAIL (`review-3-claude-code.md`); it confirmed round-2 rows 1, 2a, 2b, 3a, 3b, 3c, 4a–4e, 5a
(apart from its item 1) and 5c. opencode did not run (usage limit, `review-3-opencode.md`). The round cap (2–3 per
step) is reached: every finding below is fixed, or accepted with a reason; none is sent round again.

| # | Finding | Disposition |
|---|---|---|
| 1 | BLOCKING: `finish_pending` generates a new key beside an existing `hub.id` when a stale marker is present | **Fixed.** `finish_pending` refuses when `hub.id` exists and the key does not ("not re-keying"); the marker unlink failure is logged instead of ignored. Test `e37_stale_marker_beside_an_id_without_key_is_refused_not_rekeyed` (load_or_mint AND adopt refuse, no key written); mutant m24. |
| 2a | BLOCKING (evidence): bodies of `e166_write_never_clobbers_*`, `e36_key_without_id_*`, `e166_concurrent_*` and the L4 bodies not shown | **Accepted with reason: size.** The isolated-run carries the bundle on argv and refuses over 120,000 bytes; bundle 3 was at the limit. The tests are in the repository at the reviewed commit, each is named by a mutant that the runner shows killed on that commit, and their result lines are quoted. |
| 2b | "help says pass `--expect`" not checkable | **Accepted (shown here):** `crates/termlink-cli/src/cli.rs`, `AdoptId.expect`: "The id the hub serves today: refuse, writing nothing, if the id would differ. Always pass it when the hub is running; without it a hub that lost both files after a certificate rotation would take the NEW certificate's id". |
| 2c | a manual `adopt-id` on a stopped hub, without `--expect`, can take a rotated certificate's id: a residual, not "fixed" | **Accepted as a residual, reason:** it needs a migrated hub that lost BOTH its id and its key, plus a certificate rotation, plus an operator running the verb by hand on a stopped hub while ignoring its help. runme never does that (it refuses an unmigrated stopped hub and always passes the served id). Peers then refuse the new id loudly (FX-2), so it cannot pass silently. |
| 3a | admin row not shown live; the live pin predates the deploy; no backfill | **Fixed.** The admin row is now idempotent and backfilled once for a pin made before it existed (state flag avoids rereading the admin chain each check). Test `e37_admin_row_is_backfilled_once_for_an_old_pin` (3 observations → 1 row); mutant m25. The live W1 pins get their row on the first check after the next deploy. |
| 3b | only one of the two local sidecars shown; node1-pool and laptop-141 unmigrated | **Stated.** Both W1 sidecars are on the same hub and code. node1-pool is not one of runme's fleet hubs (`FLEET_HUBS`: ring20-management, ring20-dashboard, greenfield) and runs another project's binary; laptop-141 is a customer machine excluded by operator ruling and is unreachable (T-3801). Neither serves the T-3366 binary, so neither has anything to migrate yet; adopt-id runs when one is upgraded. |
| 4 | false "pinned" records from losers / failed writes; admin row assertion too weak | **Fixed.** Only the thread that published the pin writes the record; if the record cannot be written the fresh pin is rolled back (no record, no pin). The race test asserts exactly one "pinned" record naming the pinned key; the real-hub test asserts exactly one admin row whose digest is sha256 of the pinned fingerprint. Mutant m22 (no rollback). |
| 5a | the injector now fires on the marker, so m6's test no longer fails a `hub.id` write; failures at key/id writes untested | **Fixed.** The injector targets a named file. `e166_failure_before_rename_leaves_no_id_and_no_temp` fails the `hub.id` publish (no `hub.id`, no temp; the next start finishes the journalled mint); `e166_failure_at_the_key_write_is_finished_by_the_next_start` fails the key publish. m6 targets the first. |
| 5b | races before the pidfile lock: spurious FCW-79 / "mint vanished" | **Fixed.** Before FCW-79 refuses, the journal and the files are read once more; `mint_both` takes a result a concurrent start finished. Both stay fail-closed if neither is found. |
| 5c | `hub.id` 0644 only under a permissive umask | **Fixed.** The temp's mode is set exactly (`set_permissions`). Test `e36_id_mode_is_0644_under_a_strict_umask` (umask 077 → hub.id 0644, key 0600); mutant m26. |

## 3. Round-3 review (verbatim)
**T-3366: FAIL — `finish_pending` silently generates a new signing key beside an existing `hub.id` when a stale journal marker is present, and two round-2 "Fixed" rows (3d, 5b) are not fully borne out by what the bundle contains.**

I could not run anything; this is from reading the bundle only.

1. **BLOCKING — `hub_identity.rs` `finish_pending` (reached from `load_or_mint` and the pending branch of `adopt_legacy`): re-keys a hub that lost its key.**
   - Input: `hub.id` present, `hub.signing.key` absent, `.hub-identity.pending` present and well-formed.
   - `read_key` returns `None`, `generate_key` succeeds, `write_id` hits `AlreadyExists` and keeps the old id, the marker is removed, and the hub starts with a new key.
   - This bypasses the `(Some(_), None)` refusal in both callers, the rule m5 and m13 guard, and AL-18's "first start only".
   - The marker can outlive a finished mint: a kill between the id publish and the marker unlink, or an unlink error (it is ignored with `let _ =`).
   - It needs a second fault (key lost before the next start), so it is narrow.
   - No test covers it: `e36_key_without_id_and_no_journal_is_refused` is the opposite case and its body is not shown.
   - Fix: in `finish_pending`, refuse when `hub.id` exists and the key does not.

2. **BLOCKING (evidence) — dispositions 3d and 5b are marked "Fixed" but only partly shown.**
   - **3d:** round 2 named m13, m14, m15, m16 and m19 as resting on test names alone. The body for m16 (`e166_write_never_clobbers_an_existing_id`) is still not quoted.
   - **5a:** the bodies of `e36_key_without_id_and_no_journal_is_refused` and `e166_concurrent_first_starts_converge_on_one_id` are also absent. The L4 bodies (m1's kill) are omitted "for size".
   - **5b:** the claim "the verb's help says to pass `--expect`" cannot be checked because the CLI plumbing is omitted.
   - **5b** also admits that a manual `adopt-id` on a stopped hub still takes the new certificate's id. That is a residual to be accepted with a reason or put to the operator, not a "Fixed".

3. **MINOR — section 20: the heading promises an admin row and none is shown.**
   - The pin's needs-attention record has `ts_ms` 1794821868014, which is 2026-11-16T09:37:48Z, before the 10:31Z deploy of 0043aff40.
   - So the reviewed first-pin path (record first, no-clobber publish, admin row) has not been shown running live.
   - `admin_row_pinned` runs only on `NewlyPinned`, so this existing pin will never get its OBS-37 row; there is no backfill.
   - Only one of the two local sidecars is shown.
   - `node1-pool` (0.12.1084, no `signing_key`) and `laptop-141` are in the fleet output but unmigrated, with no stated reason.

4. **MINOR — `hub_key_pin.rs` `observe`, the `None =>` pin arm: false "pinned" records.**
   - The "hub signing key pinned: <off>" entry is written before `write_state_new`.
   - A thread that loses the race (`Ok(false)`) or whose write fails (`Err`) leaves a durable entry naming a key that was never pinned.
   - Two threads with the same offer leave duplicates.
   - `e37_concurrent_first_contacts_with_different_offers_pin_once` does not count these entries (8 offers would leave 8 "pinned" records for 1 pin).
   - The admin-row assertion checks only `event` and `msg_id`, not that the row carries the fingerprint.

5. **MINOR — `hub_identity.rs` `mint_both` / `load_or_mint`: the journal changed what the FCW-81 evidence exercises, and opened races.**
   - `FAIL_BEFORE_RENAME` now fires on the first `atomic_write`, which is the marker. So `e166_failure_before_rename_leaves_no_id_and_no_temp` (m6's test) no longer tests a failed `hub.id` write.
   - A real failure at the key or id write leaves the marker (and possibly the key) on disk, a state that test's "directory empty" assertion would not hold for. It is untested.
   - `load_or_mint` runs before `claim_runtime` in `prepare_start`, so the PID/socket lock does not serialise it.
   - A second start that sees no marker and then the first start's fresh key gets a spurious FCW-79.
   - A start whose marker is removed by the winner gets "mint vanished".
   - Both fail closed.
   - `atomic_write` applies the mode through `open`, so `hub.id` is 0644 only under a permissive umask (E-36.4 asks for 0644).

The other "Fixed" rows are borne out by the code, tests or output shown: 1 (hash 0043aff40 quoted at each place, 23/23), 2a, 2b (code and test), 3a, 3b, 3c, 4a–4e, 5a (apart from item 1) and 5c.

## 4. Spec: AL-17 and AL-18
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

## 5. Spec: E-36, E-37, E-126, E-166
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
| E-37 hub signing key: persisted, never printed, pinned on first contact | PASS | Hub half: unit `e37_*` (0600, FCW-82 mode/contents refused, key missing beside an id refused, debug form carries no seed). Sidecar half: `crates/termlink-sidecar/tests/t3366_hub_key_pin.rs` on a real hub: first contact pins + needs-attention; restart keeps; swapped key file → `verified_hub_key` refuses, pin unchanged, one entry naming both fingerprints (coalesced per day); an unreadable pin is never replaced; a withdrawn key is named and refused; the hub seed appears in no file of the sidecar store. SI-25 hub side: the real-hub test runs the hub with `RUST_LOG=info` and scans every file of its runtime dir, its stdout/stderr logs included, for the seed. LIMIT AR-1: `e37_ar1_limit_rogue_key_at_first_contact_is_pinned` records that a rogue key at the first trusted contact is pinned (by design). OBS-37: a new pin writes the admin row `hub-key-pinned` on the store admin chain, after a needs-attention record (no record, no pin). Controls: m4, m5, m8, m9, m11, m12, m17, m21, m22, m23 killed. |
| E-38 mail-hub declaration | NOT THIS UNIT | The `mail_hub` declaration reader and check is plan row P-11 (T-3363, "AEF owns the key", DR-7), which lists E-38 as its own evidence and depends on this unit's minted id, which now exists. |
| E-126 hub id unchanged across restart and certificate rotation | PASS | Sidecar status `hub {id, declared, match}` from the id each check observed, with the fake-status control (`e126_*`: a sidecar declared for another hub reads `match: false`; m18 killed). The rotation case of E-36 (L4). runme passes the running hub's served id as `--expect` and compares the NEW hub's served id after every restart, local and fleet (fixtures: "an id changed by the restart => FAILED"). |
| E-163 test seams refused outside an estate | NOT APPLICABLE | T-3366 adds no `TERMLINK_*_TEST_*` variable. Its only fault injector (`FAIL_BEFORE_RENAME`) is `#[cfg(test)]` and does not exist in a release build. |
| E-166 hub identity and pin at start | PASS | A mint that cannot be written atomically refuses and leaves no file or temp (`e166_fcw81_*`, `e166_failure_before_rename_*` with a fault injector between write and publish; m6 killed); the publish never clobbers (`e166_write_never_clobbers_*`, m16); 8 concurrent first starts converge on one id and key (`e166_concurrent_first_starts_converge_*`); a stale temp is removed only when its writer is dead. FCW-84: a first contact off the local socket pins nothing (`e166_fcw84_*`, m10 killed). A sidecar whose hub is down neither posts nor pulls until the id verifies: `fc_matrix::fc_8_hub_down_shows_waiting_then_unknown_then_goes_on_with_no_loss`, `send_hub::send_hub_hub_down_waits_then_flows_on_without_loss_or_duplicate` (both green in the full sidecar suite). |

Round 3 (after review round 2, `T-3366-reviews/r2-dispositions.md`), all on commit 0043aff40, which runme deployed:
mutants `docs/reports/T-3366-mutants.md`, 23 of 23 killed ("Run 2026-11-16T10:42Z on 0043aff40"); runme fixtures 387
passed, 0 failed; unit `hub_identity` 23 passed; L4 hub 3 passed; sidecar `t3366_hub_key_pin` 8 passed; full hub,
sidecar and CLI suites green. Added in round 3: the two-file mint is journalled (`.hub-identity.pending`; tests
`e166_interrupted_*`, `e36_key_without_id_and_no_journal_is_refused`, m20); the sidecar pin is published no-clobber
after its record (`e37_concurrent_first_contacts_*`, `e37_no_record_means_no_pin`, m21, m22) and writes the OBS-37
admin row `hub-key-pinned` (asserted in the real-hub test, m23).

## 7. Mutant report (run on 2b3759e02, whose T-3366 sources are identical to fe72c850c: section 8)
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
| FCW-79 / ruling B | `t3366-m19-cert-not-prior` | a certificate-only directory mints silently instead of refusing | `hub_identity::tests::e36_cert_only_dir_refuses_until_adopt_gives_the_certificate_id` | killed |
| FCW-79 | `t3366-m2-no-fcw79` | a missing hub.id beside prior state is minted instead of refused | `hub_identity::tests::e36_fcw79_missing_id_beside_state_refuses_and_writes_nothing` | killed |
| E-166 crash table / r2 #5 | `t3366-m20-journal-ignored` | the journal is never read: an interrupted mint is not finished | `hub_identity::tests::e166_interrupted_first_start_is_finished_by_the_next_start` | killed |
| AL-18 inv.3 / r2 #2 | `t3366-m21-pin-clobbers` | the pin publish replaces another thread's pin | `e37_concurrent_first_contacts_with_different_offers_pin_once` | killed |
| OBS-37 / r2 #2, r3 #4 | `t3366-m22-pin-without-record` | a pin whose record could not be written is kept | `e37_no_record_means_no_pin` | killed |
| OBS-37 / r2 #2b | `t3366-m23-no-admin-row` | a new pin writes no admin row | `e37_first_contact_pins_restart_keeps_and_a_swapped_key_is_refused_and_named` | killed |
| AL-18 / r3 #1 | `t3366-m24-stale-marker-rekeys` | a stale journal marker re-keys a hub whose key was lost | `hub_identity::tests::e37_stale_marker_beside_an_id_without_key_is_refused_not_rekeyed` | killed |
| OBS-37 / r3 #3 | `t3366-m25-no-backfill` | a pin made before OBS-37 never gets its admin row | `e37_admin_row_is_backfilled_once_for_an_old_pin` | killed |
| E-36.4 / r3 #5 | `t3366-m26-mode-from-umask` | the file mode is left to the umask | `hub_identity::tests::e36_id_mode_is_0644_under_a_strict_umask` | killed |
| FCW-80 | `t3366-m3-lenient-id` | any 16-character hub.id is accepted | `hub_identity::tests::e36_fcw80_malformed_id_refuses` | killed |
| FCW-82 | `t3366-m4-no-key-mode-check` | a signing key file with any mode is accepted | `hub_identity::tests::e37_fcw82_bad_key_file_refuses` | killed |
| AL-18 | `t3366-m5-key-regenerated` | a missing signing key beside an existing id is regenerated at start | `hub_identity::tests::e37_key_missing_beside_id_refuses` | killed |
| FCW-81 | `t3366-m6-write-in-place` | hub.id is written in place, not via temp + publish | `hub_identity::tests::e166_failure_before_rename_leaves_no_id_and_no_temp` | killed |
| AL-17 adopt | `t3366-m7-adopt-overwrites` | adopt-id replaces an id already on disk | `hub_identity::tests::adopt_is_idempotent_and_never_changes_an_existing_id` | killed |
| SI-25 | `t3366-m8-debug-prints-key` | the debug form prints the private seed | `hub_identity::tests::e37_private_key_is_never_printed` | killed |
| FCW-83 / E-37 control | `t3366-m9-silent-repin` | a changed hub key is re-pinned silently | `e37_first_contact_pins_restart_keeps_and_a_swapped_key_is_refused_and_named` | killed |

PASS: all 26 mutants turned their test red. Run 2026-11-16T11:36Z on 2b3759e02.

## 8. Commits from the round-3 fix commit (995ac840a) to the deployed fe72c850c, and the T-3366 files they touch (only mutant patches and docs; the T-3810/T-3811 commits touch channel.rs and runme action 29)
```
9 commits: T-3366 mutant patches/docs, T-3810 (channel.rs), T-3811 (runme 29), T-3806 notes, a handover
--- T-3366 source files changed in that range (empty = none):
(none)
```

## 9. crates/termlink-hub/src/hub_identity.rs at fe72c850c: production code in full
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

#[cfg(test)]
thread_local! {
    /// E-166 fault injector (tests only): fail after the temp is written, before the rename.
    static FAIL_BEFORE_RENAME: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) };
}

/// Write `contents` to `dir/name` atomically: temp (exact `mode`) + fsync + no-clobber publish +
/// dir fsync. On any error the temp is removed and no partial file is left (FCW-81).
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
        {
            // the mode argument is masked by the umask; set it exactly (review r3 #5: E-36.4 asks 0644)
            use std::os::unix::fs::PermissionsExt;
            f.set_permissions(std::fs::Permissions::from_mode(mode))?;
        }
        f.write_all(contents)?;
        f.sync_all()?;
        drop(f);
        #[cfg(test)]
        if FAIL_BEFORE_RENAME.with(|c| c.get()) == Some(name) {
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

pub const PENDING_FILE: &str = ".hub-identity.pending";

/// Finish a mint the journal says was started (review T-3366 r2 #5): the key and the id are two
/// files, so a crash between their writes would otherwise leave a state every later start refuses.
/// The marker holds the chosen id; the key is written first, then the id, then the marker goes.
/// A complete identity already on disk wins over the marker (a racing start finished first).
/// `Ok(None)` when no mint is pending.
fn finish_pending(dir: &Path) -> std::io::Result<Option<HubIdentity>> {
    let marker = dir.join(PENDING_FILE);
    let raw = match std::fs::read_to_string(&marker) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(refuse(format!("{PENDING_FILE} unreadable ({e})"))),
    };
    let want = raw.trim_end_matches('\n');
    if !is_id(want) {
        return Err(refuse(format!("{PENDING_FILE} is malformed; remove it only after checking hub.id by hand")));
    }
    // Review r3 #1: a marker that outlived a finished mint must not re-key a hub that later lost
    // its key. The key is always published before the id, so an id on disk with no key is a loss.
    let id_now = read_id(dir)?;
    let key_now = read_key(dir)?;
    if id_now.is_some() && key_now.is_none() {
        return Err(refuse(format!(
            "hub.signing.key missing beside hub.id in {} ({PENDING_FILE} is stale); not re-keying (a new key is an operator action)",
            dir.display()
        )));
    }
    let signing = match key_now {
        Some(k) => k,
        None => match generate_key(dir) {
            Ok(k) => k,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => read_key(dir)?.ok_or(e)?,
            Err(e) => return Err(e),
        },
    };
    let hub_id = match write_id(dir, want) {
        Ok(()) => want.to_string(),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => read_id(dir)?.ok_or(e)?,
        Err(e) => return Err(e),
    };
    if let Err(e) = std::fs::remove_file(&marker)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(error = %e, "{PENDING_FILE} could not be removed after the mint finished");
    }
    if let Ok(d) = std::fs::File::open(dir) {
        let _ = d.sync_all();
    }
    Ok(Some(HubIdentity { hub_id, signing }))
}

/// Journal a mint of `id`, then finish it. A marker already there (a concurrent mint) is used as is.
fn mint_both(dir: &Path, id: &str) -> std::io::Result<HubIdentity> {
    std::fs::create_dir_all(dir)?;
    match atomic_write(dir, PENDING_FILE, format!("{id}\n").as_bytes(), 0o644) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(std::io::Error::new(e.kind(), format!("hub.id could not be written atomically: {e}"))),
    }
    if let Some(done) = finish_pending(dir)? {
        return Ok(done);
    }
    // A concurrent start finished the mint and removed the marker first: take its result.
    match (read_id(dir)?, read_key(dir)?) {
        (Some(hub_id), Some(signing)) => Ok(HubIdentity { hub_id, signing }),
        _ => Err(refuse("hub identity mint vanished while being finished")),
    }
}

/// Hub start (AL-17 step 2, AL-18): read the id and key, finish an interrupted mint, or mint on a
/// first start. Must run BEFORE the start writes its own secret or certificate, so a first start
/// is recognised as one.
/// Review r3 #5: before FCW-79 refuses, look once more — a concurrent first start may have written
/// its marker and key between our reads. A finished identity found now is used, not refused.
fn finish_pending_again(dir: &Path) -> std::io::Result<Option<HubIdentity>> {
    if let Some(done) = finish_pending(dir)? {
        return Ok(Some(done));
    }
    match (read_id(dir)?, read_key(dir)?) {
        (Some(hub_id), Some(signing)) => Ok(Some(HubIdentity { hub_id, signing })),
        _ => Ok(None),
    }
}

pub fn load_or_mint(dir: &Path) -> std::io::Result<HubIdentity> {
    if let Some(done) = finish_pending(dir)? {
        tracing::info!(hub_id = %done.hub_id, "hub identity: an interrupted mint was finished (T-3366)");
        return Ok(done);
    }
    match (read_id(dir)?, read_key(dir)?) {
        (Some(hub_id), Some(signing)) => Ok(HubIdentity { hub_id, signing }),
        // The key is always written before the id, so an id without a key is a loss, not a crash.
        (Some(_), None) => Err(refuse(format!(
            "hub.signing.key missing beside existing state in {}; restore it (a new key is an operator action)",
            dir.display()
        ))),
        (None, _) if has_prior_state(dir) => match finish_pending_again(dir)? {
            Some(done) => Ok(done),
            None => Err(refuse(format!(
                "hub.id missing beside existing state in {}; restore it, or run `termlink hub adopt-id` on a hub that predates T-3366 (runme does this before a restart)",
                dir.display()
            ))),
        },
        (None, _) => {
            let (id, from_cert) = mint_value(dir);
            let done = mint_both(dir, &id)?;
            tracing::info!(hub_id = %done.hub_id, from_certificate = from_cert, "hub.id minted (first start, T-3366)");
            Ok(done)
        }
    }
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
    let check = |found: &str| -> std::io::Result<()> {
        match expect {
            Some(e) if e != found => Err(refuse(format!(
                "hub.id would be {found} but the hub serves {e}; nothing written (an inbox would be renamed)"
            ))),
            _ => Ok(()),
        }
    };
    // An interrupted migration (or first start) is finished, not refused (review r2 #5).
    if let Ok(raw) = std::fs::read_to_string(dir.join(PENDING_FILE)) {
        let id = raw.trim_end_matches('\n').to_string();
        check(&id)?;
        if dry_run {
            return Ok(Adopted { hub_id: id, id_source: "pending", key_generated: false, signing_key: "(an interrupted migration would be finished)".into() });
        }
        let had_key = read_key(dir)?.is_some();
        let done = finish_pending(dir)?.ok_or_else(|| refuse("pending migration vanished"))?;
        return Ok(Adopted { hub_id: done.hub_id.clone(), id_source: "pending", key_generated: !had_key, signing_key: done.signing_key_fingerprint() });
    }
    match (read_id(dir)?, read_key(dir)?) {
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
            let done = mint_both(dir, &id)?;
            check(&done.hub_id)?;
            Ok(Adopted { hub_id: done.hub_id.clone(), id_source, key_generated: true, signing_key: done.signing_key_fingerprint() })
        }
    }
}

```

## 10. Its unit tests: all 26 names, and the bodies of the 7 the mutants and findings rely on
```
e36_empty_dir_mints_once_and_restart_keeps_it
e36_cert_only_dir_refuses_until_adopt_gives_the_certificate_id
e36_certificate_rotation_changes_neither_id_nor_key
e36_fcw79_missing_id_beside_state_refuses_and_writes_nothing
e36_fcw79_topic_log_counts_as_prior_state
e36_fcw80_malformed_id_refuses
e166_fcw81_mint_that_cannot_be_written_leaves_nothing
e166_failure_before_rename_leaves_no_id_and_no_temp
e166_failure_at_the_key_write_is_finished_by_the_next_start
e37_stale_marker_beside_an_id_without_key_is_refused_not_rekeyed
e36_id_mode_is_0644_under_a_strict_umask
e166_stale_temp_from_a_dead_start_is_removed
e166_atomic_write_never_leaves_a_temp
e37_fcw82_bad_key_file_refuses
e37_key_missing_beside_id_refuses
e37_private_key_is_never_printed
adopt_is_idempotent_and_never_changes_an_existing_id
adopt_refuses_a_lost_key_beside_an_id_and_writes_no_key
adopt_refuses_a_lost_id_beside_a_key_and_writes_no_id
adopt_expect_mismatch_writes_nothing
e166_write_never_clobbers_an_existing_id
e166_concurrent_first_starts_converge_on_one_id
e166_interrupted_first_start_is_finished_by_the_next_start
e166_interrupted_migration_is_finished_by_adopt_and_checked_against_expect
e36_key_without_id_and_no_journal_is_refused
adopt_refuses_a_malformed_id_rather_than_replacing_it
```
```rust

    // E-36 under ruling B: a cert-only dir is prior state; adopt-id gives it first16(fingerprint).
    #[test]
    fn e36_cert_only_dir_refuses_until_adopt_gives_the_certificate_id() {
        let d = tmp("cert");
        let want = write_cert(&d);
        // ruling B: a certificate-only directory is prior state — refused, nothing written ...
        let e = load_or_mint(&d).unwrap_err().to_string();
        assert!(e.contains("hub.id missing beside existing state"), "{e}");
        assert!(!d.join(ID_FILE).exists() && !d.join(KEY_FILE).exists());
        // ... until adopt-id gives it first16(fingerprint) (E-36: no inbox renamed)
        let ad = adopt_legacy(&d, false, None).unwrap();
        assert_eq!((ad.hub_id.as_str(), ad.id_source), (want.as_str(), "certificate"));
        assert_eq!(load_or_mint(&d).unwrap().hub_id, want);
        // and adopt on a secret+cert legacy hub gives the same certificate id
        let d2 = tmp("cert2");
        let want2 = write_cert(&d2);
        std::fs::write(d2.join("hub.secret"), "ab".repeat(32)).unwrap();
        let ad = adopt_legacy(&d2, false, None).unwrap();
        assert_eq!((ad.hub_id.as_str(), ad.id_source, ad.key_generated), (want2.as_str(), "certificate", true));
        assert_eq!(load_or_mint(&d2).unwrap().hub_id, want2);
        let _ = std::fs::remove_dir_all(&d2);
        let _ = std::fs::remove_dir_all(&d);
    }


    // E-166 control (mutant "write hub.id in place"): a failure after hub.id's bytes are written but
    // before its publish leaves NO hub.id and no temp; the journal (marker + key) stays, and the next
    // start finishes the mint with the journalled id. An in-place writer leaves hub.id behind.
    #[test]
    fn e166_failure_before_rename_leaves_no_id_and_no_temp() {
        let d = tmp("inject");
        FAIL_BEFORE_RENAME.with(|c| c.set(Some(ID_FILE)));
        let r = load_or_mint(&d);
        FAIL_BEFORE_RENAME.with(|c| c.set(None));
        assert!(r.is_err());
        assert!(!d.join(ID_FILE).exists(), "no hub.id");
        let temps = std::fs::read_dir(&d).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().contains(".tmp-")).count();
        assert_eq!(temps, 0, "no temp");
        let journalled = std::fs::read_to_string(d.join(PENDING_FILE)).unwrap().trim().to_string();
        let i = load_or_mint(&d).unwrap();
        assert_eq!(i.hub_id, journalled, "the next start finishes the journalled mint");
        assert!(!d.join(PENDING_FILE).exists());
        let _ = std::fs::remove_dir_all(&d);
    }


    // Review r1 #1: a migrated hub that lost ONE file is refused, never re-keyed or re-minted.
    #[test]
    fn adopt_refuses_a_lost_key_beside_an_id_and_writes_no_key() {
        let d = tmp("lostkey");
        std::fs::write(d.join("hub.secret"), "ab".repeat(32)).unwrap();
        adopt_legacy(&d, false, None).unwrap();
        std::fs::remove_file(d.join(KEY_FILE)).unwrap();
        let e = adopt_legacy(&d, false, None).unwrap_err().to_string();
        assert!(e.contains("does not re-key"), "{e}");
        assert!(!d.join(KEY_FILE).exists());
        let _ = std::fs::remove_dir_all(&d);
    }


    #[test]
    fn adopt_refuses_a_lost_id_beside_a_key_and_writes_no_id() {
        let d = tmp("lostid");
        write_cert(&d);
        std::fs::write(d.join("hub.secret"), "ab".repeat(32)).unwrap();
        adopt_legacy(&d, false, None).unwrap();
        std::fs::remove_file(d.join(ID_FILE)).unwrap();
        write_cert(&d); // a rotation since: a re-mint would take the NEW certificate's id
        let e = adopt_legacy(&d, false, None).unwrap_err().to_string();
        assert!(e.contains("does not re-mint"), "{e}");
        assert!(!d.join(ID_FILE).exists());
        let _ = std::fs::remove_dir_all(&d);
    }


    // `--expect`: a differing id is refused before anything is written; a matching one passes.
    #[test]
    fn adopt_expect_mismatch_writes_nothing() {
        let d = tmp("expect");
        let want = write_cert(&d);
        std::fs::write(d.join("hub.secret"), "ab".repeat(32)).unwrap();
        let e = adopt_legacy(&d, false, Some("0123456789abcdef")).unwrap_err().to_string();
        assert!(e.contains("nothing written"), "{e}");
        assert!(!d.join(ID_FILE).exists() && !d.join(KEY_FILE).exists());
        assert_eq!(adopt_legacy(&d, false, Some(&want)).unwrap().hub_id, want);
        assert!(adopt_legacy(&d, false, Some("0123456789abcdef")).is_err(), "kept id checked too");
        let _ = std::fs::remove_dir_all(&d);
    }


    // Review r2 #5: a crash between the key and the id leaves a journal; the next start finishes it.
    #[test]
    fn e166_interrupted_first_start_is_finished_by_the_next_start() {
        let d = tmp("pend1");
        std::fs::write(d.join(PENDING_FILE), "aaaaaaaaaaaaaaaa\n").unwrap();
        let k = generate_key(&d).unwrap(); // the crash happened after the key, before the id
        let i = load_or_mint(&d).unwrap();
        assert_eq!(i.hub_id, "aaaaaaaaaaaaaaaa");
        assert_eq!(i.signing_key_fingerprint(), signing_fingerprint(&k), "the key already written is kept");
        assert!(!d.join(PENDING_FILE).exists());
        assert_eq!(load_or_mint(&d).unwrap().hub_id, "aaaaaaaaaaaaaaaa");
        let _ = std::fs::remove_dir_all(&d);
    }


    #[test]
    fn e166_interrupted_migration_is_finished_by_adopt_and_checked_against_expect() {
        let d = tmp("pend2");
        let want = write_cert(&d);
        std::fs::write(d.join("hub.secret"), "ab".repeat(32)).unwrap();
        std::fs::write(d.join(PENDING_FILE), format!("{want}\n")).unwrap(); // crashed before the key
        assert!(adopt_legacy(&d, false, Some("0123456789abcdef")).is_err(), "expect still checked");
        assert!(!d.join(ID_FILE).exists() && !d.join(KEY_FILE).exists());
        let a = adopt_legacy(&d, false, Some(&want)).unwrap();
        assert_eq!((a.hub_id.as_str(), a.id_source, a.key_generated), (want.as_str(), "pending", true));
        assert!(!d.join(PENDING_FILE).exists());
        assert_eq!(load_or_mint(&d).unwrap().hub_id, want);
        let _ = std::fs::remove_dir_all(&d);
    }

```

## 11. crates/termlink-sidecar/src/hub_key_pin.rs at fe72c850c
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

/// OBS-37 (review r2 #2b): the admin row `hub-key-pinned` on the store's admin chain: msg id =
/// hub id, digest = sha256 of the fingerprint, reason = the fingerprint. Exactly one per hub id:
/// written on a new pin, and backfilled once for a pin made before this row existed (r3 #3). A
/// state flag keeps the admin chain from being read on every check.
fn admin_row_pinned(store: &Store, hub_id: &str, fp: &str) {
    let flag = format!("hub-key-admin-{hub_id}");
    if store.read_state(&flag).ok().flatten().as_deref().map(str::trim) == Some(fp) {
        return;
    }
    use sha2::{Digest, Sha256};
    let digest: String = Sha256::digest(fp.as_bytes()).iter().map(|b| format!("{b:02x}")).collect();
    let extra = crate::store::LineExtra { reason: Some(fp.to_string()), ..Default::default() };
    let r = store.append_event_unless(crate::api::ADMIN_CONV, "hub-key-pinned", hub_id, "hub", &digest, extra, |lines| {
        lines.iter().any(|l| l.event == "hub-key-pinned" && l.msg_id == hub_id && l.digest == digest)
    });
    match r {
        Ok(_) => {
            let _ = store.write_state(&flag, fp);
        }
        Err(e) => alert(store, "hub-signing-key-admin-row-failed", &format!("hub {hub_id}: pinned {fp}, but the admin row could not be written: {e}")),
    }
}

/// FCW-83: the offer differs from the pin. Named once per day per offered key; never re-pinned.
fn mismatch(store: &Store, hub_id: &str, p: String, off: String) -> KeyState {
    let detail = format!(
        "hub {hub_id} signing key changed: pinned {p}, offered {off}; every hub-signed object is refused until an operator re-pin"
    );
    let day = crate::store::now_ms() / 86_400_000;
    if let Err(e) = store.attention_coalesced("hub-signing-key-changed", &format!("{day}:{off}"), &detail) {
        eprintln!("termlink-sidecar: needs attention (hub-signing-key-changed): {detail} [could not record it: {e}]");
    }
    KeyState::Mismatch { pinned: p, offered: off }
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
                Some(p) if p == off => {
                    admin_row_pinned(store, hub_id, &off); // r3 #3: backfills a pin made before OBS-37
                    KeyState::Pinned(off)
                }
                Some(p) => mismatch(store, hub_id, p, off),
                None if !trusted_path || !valid_fp(&off) => KeyState::Unverified(off),
                None => {
                    // Review r2 #2 / r3 #4: a no-clobber publish, so threads with different offers
                    // cannot replace each other's pin (the loser compares against the winner's);
                    // only the thread that published writes the "pinned" record, and if that record
                    // cannot be written the fresh pin is rolled back: no record, no pin.
                    match store.write_state_new(&pin_state_name(hub_id), &off) {
                        Ok(true) => match store.attention("hub-signing-key-pinned", &format!("hub {hub_id} signing key pinned: {off}")) {
                            Ok(()) => {
                                admin_row_pinned(store, hub_id, &off);
                                KeyState::NewlyPinned(off)
                            }
                            Err(e) => {
                                let _ = store.remove_state(&pin_state_name(hub_id));
                                eprintln!("termlink-sidecar: hub {hub_id}: pin {off} rolled back: its record could not be written ({e})");
                                KeyState::Unverified(off)
                            }
                        },
                        Ok(false) => match store.read_state(&pin_state_name(hub_id)) {
                            Ok(Some(p)) if p.trim() == off => KeyState::Pinned(off),
                            Ok(Some(p)) => mismatch(store, hub_id, p.trim().to_string(), off),
                            _ => KeyState::Unverified(off),
                        },
                        Err(e) => {
                            alert(store, "hub-signing-key-pin-failed", &format!("hub {hub_id}: could not record the pin {off}: {e}"));
                            KeyState::Unverified(off)
                        }
                    }
                }
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

## 12. Store::write_state_new (crates/termlink-sidecar/src/store.rs)
```rust
    /// Write a named state only if none exists yet: a no-clobber publish (T-3366 review r2 #2).
    /// `Ok(true)` when written, `Ok(false)` when the state was already there (left untouched).
    pub fn write_state_new(&self, name: &str, value: &str) -> Result<bool> {
        check_name(name)?;
        let tmp = self.dir.join("tmp").join(format!(
            "state-{}-{}",
            std::process::id(),
            TMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let mut f = create0600(&tmp, true)?;
        f.write_all(value.as_bytes())?;
        f.sync_all()?;
        drop(f);
        let res = fs::hard_link(&tmp, self.dir.join(format!("state.{name}")));
        let _ = fs::remove_file(&tmp);
        match res {
            Ok(()) => {
                fsync_dir(&self.dir)?;
                Ok(true)
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

```

## 13. Other Rust changes of the unit (hub start, hub.version, sidecar wiring), 8bba3733c..fe72c850c
```diff
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
```
CLI plumbing omitted for size: `hub adopt-id [--dry-run] [--expect ID] [--json]` calls `adopt_legacy(runtime_dir, dry_run, expect)` and prints each line; `fleet doctor --json` adds `hub_id` and `signing_key` from the same `hub.version` reply.

## 14. runme.sh at fe72c850c: adopt_hub_id, the step-4 call, the restart check, and the fleet migration block
```bash
# T-3366 (AL-17 legacy migration): a hub binary with T-3366 refuses to start beside existing
# state without hub.id (FCW-79). `adopt_hub_id [binary]` makes <binary> write hub.id (the
# current certificate's first 16 hex, the id every inbox already uses) and hub.signing.key,
# only on a pre-T-3366 directory; a directory with one file but not the other is refused by
# adopt-id itself (review T-3366 r1 #1). It runs (a) in step 4 with the NEWLY BUILT binary
# BEFORE it is installed at the hub's path, so a crash/auto-restart between install and step 5
# cannot start a binary that would refuse (review r1 #4), and (b) again before every restart.
# The running hub's served id is passed as --expect, so a differing id is never written; the
# key must be owned by the hub process's user (FCW-82 would refuse it otherwise). Sets HUB_ID0.
# Returns 1 (caller must not install/restart) when anything cannot be verified.
# A binary that predates T-3366 (clap: "unrecognized subcommand") has nothing to migrate.
seam HUB_RT RUNME_HUB_RUNTIME_DIR "/var/lib/termlink"
seam HUB_PROC RUNME_HUB_PROC "/proc"
seam HUB_SYSTEMCTL RUNME_HUB_SYSTEMCTL "systemctl"
HUB_UNIT=termlink-hub
adopt_hub_id() {
    local bin="${1:-$HUB_BIN}" help served sid_err out line pid kuid puid user
    HUB_ID0=""
    [ -d "$HUB_RT" ] || { say "  note    no hub runtime dir $HUB_RT — nothing to migrate"; return 0; }
    if ! help="$("$bin" hub adopt-id --help 2>&1)"; then
        case "$help" in
          *"unrecognized subcommand"*)
            say "  note    $bin has no 'hub adopt-id' (pre-T-3366): no hub.id needed"; return 0 ;;
        esac
        say "  FAILED  cannot tell whether $bin supports 'hub adopt-id': $(printf '%s' "$help" | tail -1)"; return 1
    fi
    # The id the RUNNING hub serves (review r1 #1d / r2 #4): --expect for the migration.
    served=""
    if [ -S "$HUB_RT/hub.sock" ]; then
        if served="$("$bin" sidecar hub-id --hub-socket "$HUB_RT/hub.sock" 2>"$HUB_RT/.runme-hubid.err")" && [[ "$served" =~ ^[0-9a-f]{16}$ ]]; then
            :
        elif grep -q "states no hub_id" "$HUB_RT/.runme-hubid.err" 2>/dev/null && [ ! -e "$HUB_RT/hub.cert.pem" ]; then
            served=""   # a socket-only legacy hub (no certificate): CR-6.42, the id is minted at random
            say "  note    the running hub states no id and has no certificate: a migration mints a RANDOM id (CR-6.42); peers will see a new id"
        else
            rm -f "$HUB_RT/.runme-hubid.err"
            say "  FAILED  the running hub's id cannot be read from $HUB_RT/hub.sock — not migrating blind"; return 1
        fi
        rm -f "$HUB_RT/.runme-hubid.err"
    elif [ ! -e "$HUB_RT/hub.id" ] && [ ! -e "$HUB_RT/.hub-identity.pending" ]; then
        say "  FAILED  the hub is not running, so the id it serves cannot be read; an unmigrated hub is not migrated blind (start it, then re-run)"; return 1
    fi
    if [ "$DRY_RUN" = "1" ]; then
        say "  [DRY]   $(TERMLINK_RUNTIME_DIR="$HUB_RT" "$bin" hub adopt-id --dry-run ${served:+--expect "$served"} 2>&1 | head -1)"; return 0
    fi
    if ! out="$(TERMLINK_RUNTIME_DIR="$HUB_RT" "$bin" hub adopt-id ${served:+--expect "$served"} 2>&1)"; then
        say "  FAILED  hub adopt-id refused in $HUB_RT: $(printf '%s' "$out" | tail -1)"; return 1
    fi
    HUB_ID0="$(tr -d '\n' < "$HUB_RT/hub.id" 2>/dev/null)"
    if ! [[ "$HUB_ID0" =~ ^[0-9a-f]{16}$ ]] || [ ! -s "$HUB_RT/hub.signing.key" ]; then
        say "  FAILED  hub adopt-id ran but $HUB_RT/hub.id or hub.signing.key is not on disk"; HUB_ID0=""; return 1
    fi
    # The key must belong to the user the hub runs as (FCW-82): the running process, else the unit's User.
    kuid="$(stat -c %u "$HUB_RT/hub.signing.key")"
    pid="$("$HUB_SYSTEMCTL" show -p MainPID --value "$HUB_UNIT" 2>/dev/null)"
    if [ -n "$pid" ] && [ "$pid" != "0" ] && [ -d "$HUB_PROC/$pid" ]; then
        puid="$(stat -c %u "$HUB_PROC/$pid")"
    else
        user="$("$HUB_SYSTEMCTL" show -p User --value "$HUB_UNIT" 2>/dev/null)"
        puid="$(id -u "${user:-root}" 2>/dev/null)"
    fi
    if [ -z "$puid" ] || [ "$kuid" != "$puid" ]; then
        say "  FAILED  hub.signing.key is owned by uid $kuid but the hub runs as uid ${puid:-unknown} (it would refuse it, FCW-82)"; return 1
    fi
    while IFS= read -r line; do say "  adopt   $line"; done <<< "$out"
}

...
    # T-3366: migrate hub.id with the NEW binary before it replaces the hub's (review r1 #4).
    adopt_hub_id "$TL_SRC" || { say "  FAILED  not installing: the new binary would refuse to start this hub"; FAILED=$((FAILED+1)); return; }
...
    # T-3366: the id the NEW hub serves must be the adopted one (a changed id renames every inbox).
    if [ -n "$HUB_ID0" ]; then
        local served1=""
        for i in $(seq 1 "$HUB_WAIT"); do
            served1="$("$HUB_BIN" sidecar hub-id --hub-socket "$HUB_RT/hub.sock" 2>/dev/null)" && [ -n "$served1" ] && break
            nap 1
        done
        [ "$served1" = "$HUB_ID0" ] || problems="$problems hub.id-CHANGED(was $HUB_ID0, serves '${served1:-unreadable}');"
    fi
    # T-3450: systemd reports the unit active before the new hub has written its pidfile and
    # socket, so one read of `hub status` raced the startup (a false FAILED on 2026-10-17 at
    # load ~30). Re-read within the same bounded window; fail only if it never reads running.
    local st="" ok_status=0
    for i in $(seq 1 "$HUB_WAIT"); do
        st="$("$HUB_BIN" hub status --json 2>/dev/null)"
        case "$st" in *"\"pid\":$npid"*'"status":"running"'*|*'"status":"running"'*"\"pid\":$npid"*) ok_status=1; break ;; esac
        nap 1
    done
...
        # T-3366: the staged binary refuses to start beside existing state without hub.id
        # (FCW-79). Migrate first, against the RUNNING hub's own runtime dir (from its
        # environment); the adopted id must be the one fleet doctor already reports
        # (no inbox renamed). Not confirmed => no swap.
        local id0 adopt adopt_all
        id0="$(fleet_hub_field "$hub" hub_id)"
        if ! [[ "$id0" =~ ^[0-9a-f]{16}$ ]]; then
            say "  FAILED  $hub: fleet doctor states no hub_id for it — cannot migrate against the served id, not swapping"; FAILED=$((FAILED+1)); continue
        fi
        # --expect: adopt-id refuses, writing nothing, if the id it would write differs from the
        # served one (review r1 #4); the key must belong to the hub process's user (FCW-82).
        adopt="$(fleet_remote "$hub" "if ! h=\$($STAGED hub adopt-id --help 2>&1); then case \"\$h\" in *'unrecognized subcommand'*) echo RUNME-ADOPT-NA ;; *) echo RUNME-ADOPT-PROBEFAIL ;; esac; exit 0; fi; p=\$(pgrep -o -f '^[^ ]*termlink hub start'); rt=\$(tr '\\0' '\\n' </proc/\$p/environ 2>/dev/null | sed -n 's/^TERMLINK_RUNTIME_DIR=//p'); if [ -z \"\$rt\" ] || [ ! -d \"\$rt\" ]; then echo RUNME-ADOPT-NORT; exit 0; fi; o=\$(TERMLINK_RUNTIME_DIR=\$rt $STAGED hub adopt-id --expect $id0 2>&1) || { echo RUNME-ADOPT-REFUSED \$o; exit 0; }; printf '%s\\n' \"\$o\" | sed 's/^/RUNME-ADOPT-LINE /'; [ \"\$(stat -c %u \$rt/hub.signing.key)\" = \"\$(stat -c %u /proc/\$p)\" ] || { echo RUNME-ADOPT-UID; exit 0; }; echo RUNME-ADOPT-OK \$(tr -d '\\n' < \$rt/hub.id) \$rt" 2>&1 | tr -d '\r')"
        case "$adopt" in
          *RUNME-ADOPT-NA*) id0="" ;;
          *RUNME-ADOPT-OK*)
            adopt_all="$adopt"
            adopt="$(printf '%s\n' "$adopt" | sed -n 's/.*RUNME-ADOPT-OK \([0-9a-f]\{16\}\) .*/\1/p' | tail -1)"
            if [ "$adopt" != "$id0" ]; then
                say "  FAILED  $hub: hub.id on disk '${adopt:-unreadable}' is not the id it serves ('$id0') — not swapping"; FAILED=$((FAILED+1)); continue
            fi
            printf '%s\n' "$adopt_all" | sed -n 's/.*RUNME-ADOPT-LINE //p' | while IFS= read -r l; do say "  adopt   $hub: $l"; done; say "  adopt   $hub hub.id $adopt (unchanged)" ;;
          *RUNME-ADOPT-NORT*)
            say "  FAILED  $hub: cannot confirm the running hub's TERMLINK_RUNTIME_DIR, so hub.id cannot be migrated — not swapping (the new hub would refuse to start)"; FAILED=$((FAILED+1)); continue ;;
          *RUNME-ADOPT-REFUSED*)
            say "  FAILED  $hub: hub adopt-id refused: $(printf '%s' "$adopt" | sed -n 's/.*RUNME-ADOPT-REFUSED //p' | tail -1) — nothing written, not swapping"; FAILED=$((FAILED+1)); continue ;;
          *RUNME-ADOPT-UID*)
            say "  FAILED  $hub: hub.signing.key is not owned by the hub process's user (it would refuse it) — not swapping"; FAILED=$((FAILED+1)); continue ;;
          *)
            say "  FAILED  $hub: hub adopt-id did not confirm: $(printf '%s' "$adopt" | tail -1) — not swapping"; FAILED=$((FAILED+1)); continue ;;
        esac
...
        local id1=""; [ -n "$id0" ] && id1="$(fleet_hub_field "$hub" hub_id)"
        if [ -n "$id0" ] && [ "$id1" != "$id0" ]; then
            say "  FAILED  $hub after deploy: hub_id '${id1:-none}' is not '$id0' (T-3366: the id must survive the restart)"; FAILED=$((FAILED+1))
        elif [ "$have" = "$want" ] && [ "$st" = "ok" ] && $TOFU "$addr" >/dev/null 2>&1; then
            say "  OK      upgraded and verified: $hub serves $have; fleet doctor ok (secret still authenticates); tofu verify ok (cert unchanged)${id0:+; hub_id $id0 unchanged}"; DONE=$((DONE+1))
        else
            say "  FAILED  $hub after deploy: version='${have:-none}' status='${st:-none}' (want $want/ok), or tofu verify failed — see /tmp/.runme-deploy-$hub.log"; FAILED=$((FAILED+1))
        fi
```

## 15. Real-hub tests at fe72c850c. crates/termlink-cli/tests/t3366_hub_identity.rs (real hub binary): test names and doc lines only, bodies omitted for size (result line in section 17):
```
//! T-3366 (arc-011 P-10, B-1): E-36 / E-37 hub half / E-166 hub half at L4 — a real hub binary
//! in a throwaway runtime dir. `hub.version` must answer the minted `hub.id`, unchanged across
//! a restart and a certificate rotation; a legacy dir (secret, no id) must refuse to start until
//! `termlink hub adopt-id` gives it the certificate-derived id.
fn cmd(dir: &TestDir, args: &[&str]) -> Command {
fn hub_version(sock: &Path) -> serde_json::Value {
fn start(dir: &TestDir, tag: &str, tcp: bool) -> ProcessGuard {
    // SI-25 scan input: the hub's own output is kept in the runtime dir.
fn stop(g: ProcessGuard, dir: &TestDir) {
fn read_id(dir: &TestDir) -> String {
fn e36_socket_only_hub_has_a_minted_id_and_hub_version_reads_the_file() {
fn e36_e37_restart_and_certificate_rotation_keep_id_and_signing_key() {
    // SI-25 (E-37.4): the private seed appears in no file of the runtime dir but its own,
fn e36_fcw79_legacy_dir_refuses_until_adopt_id_then_keeps_the_certificate_id() {
```
crates/termlink-sidecar/tests/t3366_hub_key_pin.rs: all 9 test names; the helpers and the bodies of the 4 tests round 3 relies on (the rest were confirmed in round 2; result line in section 17):
```
e37_first_contact_pins_restart_keeps_and_a_swapped_key_is_refused_and_named
e166_fcw84_untrusted_first_contact_pins_nothing_and_no_offer_changes_nothing
e37_unreadable_pin_is_not_replaced
e37_withdrawn_key_is_named_and_refused
e126_status_states_the_answering_hub_id_and_whether_it_matches
e37_ar1_limit_rogue_key_at_first_contact_is_pinned
e37_concurrent_first_contacts_with_different_offers_pin_once
e37_admin_row_is_backfilled_once_for_an_old_pin
e37_no_record_means_no_pin
```
```rust
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
    // OBS-37: the admin row on the store's admin chain names the pinned key
    let rows = store.chain_lines(termlink_sidecar::api::ADMIN_CONV).unwrap();
    let want: String = {
        use sha2::{Digest, Sha256};
        Sha256::digest(k1.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
    };
    let pinned_rows: Vec<_> = rows.iter().filter(|l| l.event == "hub-key-pinned" && l.msg_id == id).collect();
    assert_eq!(pinned_rows.len(), 1, "exactly one admin row");
    assert_eq!(pinned_rows[0].digest, want, "the admin row carries the pinned fingerprint's digest");
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

// Review r2 #2: threads observing DIFFERENT offers on an empty pin end with ONE pin, never replaced;
// the losers see a mismatch, not a silent re-pin.
#[test]
fn e37_concurrent_first_contacts_with_different_offers_pin_once() {
    let e = Estate::create_without_hub().unwrap();
    let store = Arc::new(Store::open(&e.store_root(), "agent-a").unwrap());
    let h = "0123456789abcdef";
    let offers: Vec<String> = (0..8).map(|i| format!("ed25519:{}", format!("{i:x}").repeat(64))).collect();
    let hs: Vec<_> = offers
        .iter()
        .cloned()
        .map(|off| {
            let st = store.clone();
            std::thread::spawn(move || hub_key_pin::observe(&st, h, &json!({"hub_id": h, "signing_key": off}), true))
        })
        .collect();
    let rs: Vec<KeyState> = hs.into_iter().map(|t| t.join().unwrap()).collect();
    let pinned = store.read_state(&hub_key_pin::pin_state_name(h)).unwrap().unwrap();
    assert_eq!(rs.iter().filter(|r| matches!(r, KeyState::NewlyPinned(_))).count(), 1, "{rs:?}");
    for r in &rs {
        match r {
            KeyState::NewlyPinned(f) | KeyState::Pinned(f) => assert_eq!(f, &pinned),
            KeyState::Mismatch { pinned: p, .. } => assert_eq!(p, &pinned),
            other => panic!("unexpected {other:?}"),
        }
    }
    // r3 #4: only the publisher writes a "pinned" record, and it names the key actually pinned
    let recs: Vec<_> = attention_kinds(&store).into_iter().filter(|(k, _)| k == "hub-signing-key-pinned").collect();
    assert_eq!(recs.len(), 1, "{recs:?}");
    assert!(recs[0].1.contains(&pinned));
}

// Review r2 #2: no durable record, no pin.
#[test]
fn e37_no_record_means_no_pin() {
    let e = Estate::create_without_hub().unwrap();
    let store = Store::open(&e.store_root(), "agent-a").unwrap();
    let h = "0123456789abcdef";
    std::fs::create_dir_all(store.dir().join("attention.jsonl")).unwrap(); // the record cannot be written
    let v = json!({"hub_id": h, "signing_key": format!("ed25519:{}", "e".repeat(64))});
    assert!(matches!(hub_key_pin::observe(&store, h, &v, true), KeyState::Unverified(_)));
    assert_eq!(store.read_state(&hub_key_pin::pin_state_name(h)).unwrap(), None);
}

// r3 #3: a pin made before OBS-37 gets its admin row once, however often it is observed.
#[test]
fn e37_admin_row_is_backfilled_once_for_an_old_pin() {
    let e = Estate::create_without_hub().unwrap();
    let store = Store::open(&e.store_root(), "agent-a").unwrap();
    let h = "0123456789abcdef";
    let fp = format!("ed25519:{}", "f".repeat(64));
    store.write_state(&hub_key_pin::pin_state_name(h), &fp).unwrap(); // pinned by older code
    let v = json!({"hub_id": h, "signing_key": fp});
    for _ in 0..3 {
        assert_eq!(hub_key_pin::observe(&store, h, &v, true), KeyState::Pinned(fp.clone()));
    }
    let rows = store.chain_lines(termlink_sidecar::api::ADMIN_CONV).unwrap();
    assert_eq!(rows.iter().filter(|l| l.event == "hub-key-pinned" && l.msg_id == h).count(), 1);
}

```

## 16. Runme fixture run on fe72c850c sources: every T-3366 PASS line as printed (shows which branch ran), and the total
```
  PASS  hub (T-3366): legacy hub gets hub.id from adopt-id BEFORE the restart; id verified unchanged after
  PASS  hub (T-3366): second migration keeps the existing id (idempotent)
  PASS  hub (T-3366): adopt-id refused => FAILED and NO restart (the new hub would refuse to start)
  PASS  hub (T-3366): adopt-id that claims success but writes nothing => FAILED by the on-disk read, no restart
  PASS  hub (T-3366): a pre-T-3366 binary restarts as before (nothing to migrate)
  PASS  hub (T-3366): an id changed by the restart => FAILED naming it
  PASS  hub (T-3366): a binary whose adopt-id probe fails for another reason => FAILED, no restart (never taken as pre-T-3366)
  PASS  hub (T-3366): --dry-run previews the migration and writes nothing
  PASS  hub (T-3366 r1): step 4 migrates hub.id with the NEW binary before installing it (no crash window)
  PASS  hub (T-3366 r1): step 4 migration refused => FAILED, the new binary is NOT installed
  PASS  hub (T-3366 r1): adopted id would differ from the RUNNING hub's served id => refused, nothing written, no restart
  PASS  hub (T-3366 r1): a migrated hub that lost its key => FAILED, no re-key, no restart
  PASS  hub (T-3366 r1): key not owned by the hub's user => FAILED, no restart (FCW-82 would refuse it)
  PASS  hub (T-3366 r2): no hub process => key owner compared with the unit's User, mismatch FAILED
  PASS  hub (T-3366 r2): hub not running and no hub.id => FAILED, nothing written (no blind migration)
  PASS  hub (T-3366 r2): socket-only legacy hub (no id served, no certificate) => random-id migration, named
  PASS  hub (T-3366 r2): no id served but a certificate present => FAILED, nothing written
  PASS  fleet (T-3366): adopt-id runs remotely BEFORE the restart; the served id is verified unchanged after
  PASS  fleet (T-3366): running hub's runtime dir unconfirmable => FAILED, no swap
  PASS  fleet (T-3366): adopted id differs from the served id (an inbox would be renamed) => FAILED, no swap
  PASS  fleet (T-3366): adopt-id refused => FAILED, no --swap-restart
  PASS  fleet (T-3366): an id changed by the restart => FAILED naming both
  PASS  fleet (T-3366 r1): no served hub_id to check against => FAILED, no swap (never skipped)
  PASS  fleet (T-3366 r1): remote adopt-id refused (--expect mismatch / lost file) => FAILED, no swap
  PASS  fleet (T-3366 r1): remote key owned by another user => FAILED, no swap
T-3052 fixtures: 392 passed, 0 failed
```

## 17. Test result lines, run on fe72c850c (hub_identity unit; L4 hub; sidecar pin; the two hub-down tests E-166 cites)
```
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 582 filtered out; finished in 0.16s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.90s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.38s
test fc_8_hub_down_shows_waiting_then_unknown_then_goes_on_with_no_loss ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 1.80s
test send_hub_hub_down_waits_then_flows_on_without_loss_or_duplicate ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 2.39s
full hub+sidecar+CLI suites: rc=0
```

## 18. Pre-migration served ids, recorded BEFORE the first deploy (commit 12c550540, committed 2026-11-16T08:18:23+01:00; first deploy started 2026-11-16T07:39:57Z)
```
T-3366: fleet doctor --json reports each hub's hub_id (from the same hub.version reply)

runme step 6 compares the adopted id with the served one and checks it after the restart; without this
field every successful fleet deploy would read as FAILED (the fixture's fake doctor had the field, the
real one did not). Live: local cacc73ea32b121dd, .122 22c19fedafd73da2, .121 1389a831016c4bf1,
greenfield aef925c6d2f4ac54.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>

```

First deploy (round-1 code, 07:39Z): the lines that wrote or compared the ids
```
=== runme.sh start 2026-11-16T07:39:57Z  args=[]  uid=0  head=33e6cd61d ===
  OK      termlink-sidecar@010-termlink--claude-termlink.service restarted and verified: active, pid 3279318 runs the installed binary
  OK      termlink-sidecar@010-termlink--claude-termlink-alt.service restarted and verified: active, pid 3279984 runs the installed binary
  adopt   hub.id cacc73ea32b121dd wrote from the current certificate in /var/lib/termlink
  OK      restarted and verified: pid 3027258 -> 3282051, exe /root/.cargo/bin/termlink (termlink 0.12.13+110.g33e6cd61d); secret + cert + hub.id cacc73ea32b121dd unchanged
  adopt   ring20-management hub.id 22c19fedafd73da2 (unchanged)
  OK      upgraded and verified: ring20-management serves 0.12.13+110.g33e6cd61d; fleet doctor ok (secret still authenticates); tofu verify ok (cert unchanged); hub_id 22c19fedafd73da2 unchanged
  adopt   ring20-dashboard hub.id 1389a831016c4bf1 (unchanged)
  OK      upgraded and verified: ring20-dashboard serves 0.12.13+110.g33e6cd61d; fleet doctor ok (secret still authenticates); tofu verify ok (cert unchanged); hub_id 1389a831016c4bf1 unchanged
  adopt   greenfield hub.id aef925c6d2f4ac54 (unchanged)
  OK      upgraded and verified: greenfield serves 0.12.13+110.g33e6cd61d; fleet doctor ok (secret still authenticates); tofu verify ok (cert unchanged); hub_id aef925c6d2f4ac54 unchanged
=== runme.sh finished rc=0 at 2026-11-16T07:52:01Z — log: /opt/termlink/.context/working/runme-logs/runme-20261116T073957Z-3024735.log ===
```

## 19. Deploy of fe72c850c (runme log, 2026-11-16 19:34Z)
```
=== runme.sh start 2026-11-16T19:34:48Z  args=[]  uid=0  head=fe72c850c ===
  adopt   hub.id cacc73ea32b121dd kept (already present) in /var/lib/termlink
  adopt   hub.signing.key kept (ed25519:ef7bed9659f22b825bba654fe2f6f3933deef08db6ec8d5c59c192386f23e030)
  OK      termlink-sidecar@010-termlink--claude-termlink.service restarted and verified: active, pid 1556550 runs the installed binary
  OK      termlink-sidecar@010-termlink--claude-termlink-alt.service restarted and verified: active, pid 1557278 runs the installed binary
  adopt   hub.id cacc73ea32b121dd kept (already present) in /var/lib/termlink
  adopt   hub.signing.key kept (ed25519:ef7bed9659f22b825bba654fe2f6f3933deef08db6ec8d5c59c192386f23e030)
  OK      restarted and verified: pid 2023806 -> 1559574, exe /root/.cargo/bin/termlink (termlink 0.12.13+126.gfe72c850c); secret + cert + hub.id cacc73ea32b121dd unchanged
  adopt   ring20-management: hub.id 22c19fedafd73da2 kept (already present) in /var/lib/termlink
  adopt   ring20-management: hub.signing.key kept (ed25519:f2cad3d44b5f485ee988f1168f6d3e8eaa5fdb97243a337128b63daf51719c83)
  adopt   ring20-management hub.id 22c19fedafd73da2 (unchanged)
  OK      upgraded and verified: ring20-management serves 0.12.13+126.gfe72c850c; fleet doctor ok (secret still authenticates); tofu verify ok (cert unchanged); hub_id 22c19fedafd73da2 unchanged
  adopt   ring20-dashboard: hub.id 1389a831016c4bf1 kept (already present) in /var/lib/termlink
  adopt   ring20-dashboard: hub.signing.key kept (ed25519:1edf80fefdcca76ce63efd144f916a8656ef479b4b2b0e66bf314ef087d85b15)
  adopt   ring20-dashboard hub.id 1389a831016c4bf1 (unchanged)
  OK      upgraded and verified: ring20-dashboard serves 0.12.13+126.gfe72c850c; fleet doctor ok (secret still authenticates); tofu verify ok (cert unchanged); hub_id 1389a831016c4bf1 unchanged
  adopt   greenfield: hub.id aef925c6d2f4ac54 kept (already present) in /root/.termlink/hub-persist
  adopt   greenfield: hub.signing.key kept (ed25519:cf4618885fb9fd26da0246a693672c3f5b128a07cdf15984fb63f7decd0f5f6d)
  adopt   greenfield hub.id aef925c6d2f4ac54 (unchanged)
  OK      upgraded and verified: greenfield serves 0.12.13+126.gfe72c850c; fleet doctor ok (secret still authenticates); tofu verify ok (cert unchanged); hub_id aef925c6d2f4ac54 unchanged
=== runme.sh finished rc=0 at 2026-11-16T19:47:55Z — log: /opt/termlink/.context/working/runme-logs/runme-20261116T193448Z-1347309.log ===
```

## 20. Live after the deploy
fleet doctor --json (hub, version, hub_id, signing_key):
```
greenfield 0.12.13+126.gfe72c850c aef925c6d2f4ac54 ed25519:cf4618885fb9fd26da0246a693672c3f5b128a07cdf15984fb63f7decd0f5f6d
laptop-141 None None None
local-hub 0.12.13+126.gfe72c850c cacc73ea32b121dd ed25519:ef7bed9659f22b825bba654fe2f6f3933deef08db6ec8d5c59c192386f23e030
local-test 0.12.13+126.gfe72c850c cacc73ea32b121dd ed25519:ef7bed9659f22b825bba654fe2f6f3933deef08db6ec8d5c59c192386f23e030
node1-pool 0.12.1084 925d947ce6e94b74 None
ring20-dashboard 0.12.13+126.gfe72c850c 1389a831016c4bf1 ed25519:1edf80fefdcca76ce63efd144f916a8656ef479b4b2b0e66bf314ef087d85b15
ring20-management 0.12.13+126.gfe72c850c 22c19fedafd73da2 ed25519:f2cad3d44b5f485ee988f1168f6d3e8eaa5fdb97243a337128b63daf51719c83
workstation-107-public 0.12.13+126.gfe72c850c cacc73ea32b121dd ed25519:ef7bed9659f22b825bba654fe2f6f3933deef08db6ec8d5c59c192386f23e030
```

W1 sidecar (claude-termlink) pin state, needs-attention record and admin row:
```
state.hub-id-seen-cacc73ea32b121dd: cacc73ea32b121dd
state.hub-key-pin-cacc73ea32b121dd: ed25519:ef7bed9659f22b825bba654fe2f6f3933deef08db6ec8d5c59c192386f23e030
state.hub-key-seen-cacc73ea32b121dd: pinned ed25519:ef7bed9659f22b825bba654fe2f6f3933deef08db6ec8d5c59c192386f23e030
{"detail":"hub cacc73ea32b121dd signing key pinned: ed25519:ef7bed9659f22b825bba654fe2f6f3933deef08db6ec8d5c59c192386f23e030","kind":"hub-signing-key-pinned","ts_ms":1794821868014}
{"conv":"_admin","seq":4,"event":"hub-key-pinned","msg_id":"cacc73ea32b121dd","sender":"hub","digest":"871cf9e7b1ec65258350ad4f436855248f912f9d53804ae73b3e532b7af11c28","prev":"eab4b22b6223274a97a1a019c0d5088467c34c42d40b32f7337bfbd682305948","hash":"d4fc642327bb08502eda0521b11cfedf95e7ef655e13a9777c8f2baac2473394","ts_ms":1794835810046,"reason":"ed25519:ef7bed9659f22b825bba654fe2f6f3933deef08db6ec8d5c59c192386f23e030"}
```
