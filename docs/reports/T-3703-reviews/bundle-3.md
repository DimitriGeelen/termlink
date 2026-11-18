# Bundle 3 — T-3703 confirmation round

## Part 1: round-2 findings (both seats, verbatim)

### Seat claude-code
**Verdict: FAIL**. No finding meets the BLOCKER definition as written, but the stale rule is not wired into the real receive path, two design-v2 commitments are unbuilt, and the hub's recovery path leaves up-to-date sidecars failing every pull. This is judged from the bundle only; where a finding depends on code not shown, it says so.

1. **MAJOR** — `termlink-sidecar/src/pull.rs: pass_locked` — The revocation tick only runs when `hub.version` succeeds. `PinError::Unreachable` returns `Waiting` (and `Mismatch` takes its own arm) without calling `Tracker::tick`, so `failed()` never runs during an outage. The stale rule (FCW-93/FCW-95) is dead in the most common failure class. — Sequence: verified pull, then the hub goes down for 2 hours. State stays `Verified`, status still reads `verified seq N at T`, no `revocation-stale` entry is raised, and `narrowing()` is false. The E-41 unit test and mutants c1/c6 drive `Tracker::tick` directly, so they do not see this. — Fix: call the tracker's failure path on every `check_pin` error, and add an L4 E-41 test through `Receiver::pass`. Also add the disposition-4 mutant "handle messages before the head", which is missing from the slice C table.

2. **MAJOR** — `termlink-sidecar/src/pull.rs: handle_msg / handle_stage` — The revocation check fails open. It re-derives the fingerprint from the unverified envelope field `env.signed.signer_pk` via `fingerprint_of_hex` and discards the `_signer` that `wire::verify` just returned. If `env.signed` is `None` or the helper returns `None`, the `if let … &&` is skipped and the message proceeds to STORED. — Input: an envelope from a revoked key whose `signer_pk` is in a form the wire verifier accepts but the lower-hex-strict helper rejects, such as upper-case hex. Whether the wire verifier accepts that is not visible in the bundle; the fail-open structure is. — Fix: compute the fingerprint from the verified `_signer` key, and treat "cannot derive fingerprint" as a refusal.

3. **MAJOR** — `termlink-sidecar/src/policy.rs` (absent) and `revocation.rs: narrowing()` — Design 5.3 and disposition 3 commit to an explicit stale gate in `policy.rs` and to running E-41.3 as written. Section 6.3 declares it not built, and `narrowing()` has no caller. Nothing binds a later widening of the urgent allow-list. — Sequence: with the sidecar STALE, an operator-admitted non-roster sender posts at priority 5. No code path consults the revocation state and no test asserts the downgrade at 59:59 and 60:01. — Fix: add the gate in `policy.rs` with the DOWNGRADED reason, the E-41.3 test and a mutant that removes the gate.

4. **MAJOR** — `termlink-sidecar: send path` (absent) — Design 5.11 first half and disposition 9 are not built: WAITING FOR RECIPIENT with `key revoked, operator action needed` plus one attention entry. Section 6.4's reason (no name-to-key binding on the sender) conflicts with section 1 item 6, which says the roster carries optional agent names since T-3704. — Sequence: fx-b sends to fx-a after fx-a's key is revoked. No reason and no attention appear; the E-39 test only asserts the message is STORED at fx-a. — Fix: implement it for named roster entries, or get the deferral re-reviewed rather than self-declared.

5. **MAJOR** — `termlink-hub/src/revocation.rs: reissue` — After a restore, the reissued list chains to the restored file's digest, not the last committed digest. An up-to-date sidecar then sees exactly held + 1 with a `prev_digest` that does not match. — Sequence (the one in `a_restored_list_is_refused_until_reissue`): the backup is at seq 1, the log at seq 2, and reissue issues seq 3 with `prev_digest` = D1. A sidecar holding seq 2 computes `lseq == held.seq + 1` and `prev != D2`, so it fails with "list does not chain" and tamper = true. It raises tamper hourly and goes STALE after 1 hour, and stays that way until the next revocation moves the seq past held + 1. No sidecar test covers reissue. — Fix: when the file is behind the log, chain to the last committed digest, or issue at last + 2. Add an L4 sidecar-through-reissue test.

6. **MAJOR** — `termlink-sidecar/src/revocation.rs: Tracker::pull` — The `hub predates revocation list` path (design 5.10, disposition 15) cannot fire against a real older hub. `signing_public_key` ships in the same slice as the RPCs, so an older hub lacks it. `pull` returns at `(None, _)` with "hub offers no signing public key" and tamper = false, before `hub.revocation_head` is ever called. — Sequence: a new sidecar against a pre-T-3703 hub gets status `unverified since T (hub offers no signing public key)` and no daily attention. — Fix: treat a missing public key as the predates case, or try the head first, and test against a version answer without the key.

7. **MAJOR** — `termlink-sidecar/src/revocation.rs: tamper_load + fetch` — A stored list that fails verification still has its keys enforced and its `seq` kept as the floor. Disposition M1 asks for that, but combined with the subset check and rollback check it can never heal. — Sequence A: one key's hex in the `revocation-list` state is corrupted. `held.keys` now contains a key the hub never listed, so every fetch fails with "list drops a key this sidecar holds as revoked", forever. Sequence B: the stored `seq` is corrupted to a large value, so every head is `ROLLBACK refused` forever. Sequence C: a local writer adds a legitimate sender's fingerprint, and that sender's mail is REFUSED without any hub signature. — Fix: on a load failure, keep the keys only until the first verified fetch, skip the subset and floor checks when `held.digest` is empty, and move the `held.digest.is_empty()` test ahead of `hdig == held.digest`.

8. **MAJOR** — `termlink-sidecar/src/pull.rs: handle_msg` revoked branch — The branch runs before `name_ok` and addressing validation, and returns `Outcome::Retry` when the REVOKED notice cannot be reported. — Sequence: a revoked key posts a message whose signed `from_agent` names a nonexistent or invalid inbox. If `report` errors, the envelope is retried every pass. If `Retry` holds the cursor, legitimate mail behind it stalls. `count_refusal` also runs again on each retry, inflating `refused_revoked`. The `report`, `refuse` and cursor code are not in the bundle, so the stall is conditional on them. Separately, any revoked key can make the receiver post signed notices to an inbox of its choosing. — Fix: refuse and count once, make the notice best-effort (never `Retry`), and validate `from_agent` first.

9. **MAJOR** — `termlink-sidecar/src/deliver.rs` and `revocation.rs: refuse_stored / SNAPS` — The hand-over re-check has two gaps. First, the doc comment says the sweep skips messages "held open by an attempt", but the code does not check for that. Second, `revoked()` and `revoked_from()` return false when no snapshot exists, while `narrowing()` defaults to true. — Sequence 1: the deliverer passes `revoked_from` (false), the pull thread installs the list and writes REFUSED, then the deliverer writes HANDED_OVER. The message then carries both lines; this assumes the hand-over append is not guarded against REFUSED, which the bundle does not show. Sequence 2: any process, or the window before `Tracker::new`, that has no snapshot reads "nothing revoked". — Fix: make the HANDED_OVER append conditional on no REFUSED under the store lock, and default the two lookups to the stored list or to fail-closed.

10. **MAJOR** — `termlink-sidecar/src/revocation.rs: raise_new_entries` — Disposition 11 says the entry names "the new entries since `attention_seq`". The code lists every entry of the held list on each seq change. — Sequence: with a list of about 3000 entries, one new revocation writes an attention entry of several hundred kilobytes. A `reissue` bumps the seq with nothing new and still raises `key-revoked` naming all keys. The start-time raise (`attention_seq < held.seq` at start) is also not done; it waits for a verified pull. — Fix: record the entry count or keys at `attention_seq`, name only the delta, and raise nothing for an empty delta.

11. **MINOR** — `termlink-sidecar/src/revocation.rs: pull / fetch / status` — Benign outcomes are flagged as tamper, which 06 4.7 reserves for signature failure, wrong key and rollback. A revoke landing between `head` and `get` gives "list does not match its head" with tamper = true. A sidecar that has not pinned yet (for example TCP-only) hits `(Some, None)` and raises `revocation-tamper` every hour indefinitely. Two smaller issues sit alongside: `verified()` puts `now_ms` in the status line, so the "write only on change" guard never holds and state is written every pass. `since_ms` is recomputed as wall-clock minus elapsed, so "since T" drifts. — Fix: retry the head once on a seq mismatch, separate "not pinned" from "pin mismatch", and store the verified time once.

12. **MINOR** — `termlink-hub: channel.rs key_card_gate, revocation.rs revoked_keys / issue, router.rs` — Several hub-side gaps:
    - `revoked_keys` ignores the reconcile block, so after a restore the post check accepts a key whose entry the restore lost, while `head` is refused.
    - `hub_identity::current()?` passes the post when the identity is not loaded (fail open).
    - Every carded post reads and signature-verifies the whole list under the global `LOCK`.
    - A failed committed-row write is only logged and never repaired, so the reconcile floor stays behind permanently and a later restore to before that entry is not detected.
    - Each refused `hub.revocation_request` writes an uncoalesced `attention: true` admin row, which any Interact-scope caller can trigger.
    
    — Fix: honour reconcile in `revoked_keys` and fail closed there, cache the verified key set by file mtime or seq, rewrite the missing committed row at start-up, and coalesce the refusal rows.

### Seat opencode
**Verdict: FAIL** — the pull/verify/refuse core is sound (no BLOCKER: one-pass refusal, rollback/fork/pin checks, failed-pull-never-empty and load-time verify are all implemented and mutation-tested), but a hub-side fail-open window exists and two section-5/dispositions commitments are not implemented.

1. **MAJOR** — crates/termlink-hub/src/revocation.rs:`revoked_keys` + channel.rs:`key_card_gate` — during REVOCATION_RECONCILE the post check serves the stale behind-the-log file, so a card key the admin log shows as committed-revoked passes the hub gate while head/get are blocked. — Reproduce the restore test (revoke k1, backup, revoke k2, restore the seq-1 file, restart → head refuses -32048); before reissue, post with k2 in enforce mode: `check_post` = Pass, `revoked_keys` loads the seq-1 list {k1}, k2 ∉ {k1} → post accepted; only the sidecar (which keeps its held seq-2 list because the pull fails) still refuses. — Make `revoked_keys` return `Err` when `reconcile_reason(dir)` is `Some` (the existing Err arm refuses with KEY_REVOKED), or union the file's keys with `committed_rows`' `added` keys.

2. **MAJOR** — termlink-sidecar policy.rs / revocation.rs:`narrowing` — v2 5.3 / disposition 3 commit an explicit stale gate in `policy.rs` that "binds any later widening"; policy.rs is unchanged, `narrowing()` has no caller, and no test binds it (6.3's "equivalent mutant" argument is PH-2-true but the committed gate is neither implemented nor tested). — Any future widening of the urgent allow-list compiles without consulting the stale state. — Add the gate (`if revocation::narrowing(store) && trust != OwnProject → downgrade, reason "revocation list stale"`) plus a unit test pinning policy×narrowing composition.

3. **MAJOR** — sender side (v2 5.11 first half, disposition 9) — the WAITING FOR RECIPIENT reason "key revoked, operator action needed" and its attention entry are not built (6.4 re-defers to P-24 post-hoc; the disposition promised slice C). — In the E-39 test itself: `b.sender.send("fx-a", …)` is stored at fx-a; b's status carries no revoked-recipient reason at all. — Re-run the disposition with the operator (formally re-dispose as D→T-3708) or build it when the name→key binding lands; do not leave it as "as built".

4. **MINOR** — channel.rs:`key_card_gate` — `crate::hub_identity::current()?` skips the whole revocation arm when identity is unloaded while posts are still served. — Hub serving posts with identity absent → revoked card key passes. — Refuse (or log-and-refuse in enforce) when identity is unavailable.

5. **MINOR** — sidecar revocation.rs:`pull` — the `hub predates revocation list` status and daily attention (disposition 15) are unreachable for a real pre-T-3703 hub: the pull fails earlier because `hub.version` lacks `signing_public_key`. — Sidecar → old hub → status forever `unverified since T (hub offers no signing public key)`, no `revocation-hub-predates` entry. — Map a version without `signing_public_key` (pin present) to the same predates class as -32601.

6. **MINOR** — revocation.rs:`committed_rows` — unparsable admin lines are skipped silently; a torn last committed row plus a restore to that row's seq dodges the block. — Torn seq-2 row + restore to seq-1 file → last parsed row = seq 1, matches file → served without RECONCILE, post check missing k2. — Treat a committed-row parse failure as reconcile-worthy (attention or block).

7. **MINOR** — revocation.rs:`raise_new_entries` — the entry names every key in the list, not "the new entries since attention_seq" (disposition 11); at the ~3000-entry cap each revocation writes a ~100s-of-KB attention row. — Filter names to entries above the stored `attention_seq`.

8. **MINOR** — revocation.rs:`refuse_stored` — one `refused-revoked` attention row and one `post_stage` notice per swept message, uncoalesced: a revoked key that mass-mailed before revocation makes the first verified pass emit O(stored) rows and hub posts in one sweep. — Coalesce the sweep's attention (kind + list seq) and batch the notices.

9. **MINOR** — hub revocation.rs:`revoked_keys` — the full signed list is re-read and ed25519-verified under `LOCK` on every carded post (up to 1 MiB), serializing posts against revokes. — Cache the verified key set in-process, invalidated on issue/reissue.

10. **MINOR** — run-t3703c.py — disposition 4's ordering mutant "handle messages before the head" is absent from the slice-C table (c1–c11); the ordering is only incidentally pinned by E-39's no-STORED assertion. — Add the mutant to the runner.

11. **MINOR** — revocation.rs:`revoked_from`/`refuse_stored` — the 16-hex prefix match can over-refuse a legitimate sender on a 64-bit sha256-prefix collision with a revoked key (documented 6.5); not attacker-forceable below ~2^64 grinding plus a card, but it is dropped legitimate mail. — Persist the full fingerprint (or signer pk) on the STORED line so the sweep matches exactly.

## Part 2: dispositions

# T-3703 code review round 2 — dispositions

Seats (isolated, `review-2-independence.json`): claude-code (`review-2-claude-code.md`) FAIL, 0 BLOCKER, 10 MAJOR,
2 MINOR; opencode (`review-2-opencode.md`) FAIL, 0 BLOCKER, 3 MAJOR, 8 MINOR. Both confirm the core: one-pass
refusal, rollback / fork / pin checks, failed pull never empty, load-time verify. A = accepted with a change (test and
mutant named), R = rejected with a reason, D = deferred as a criterion of a named task.

| # | Finding | Disp. | Change / where it lands |
|---|---|---|---|
| CC 1 | stale rule dead during an outage (tick only after `hub.version` succeeds) | A | every `check_pin` failure (unreachable, hub-id mismatch) calls `Tracker::outage` → the failed-pull path. L4 test `e41_l4_a_hub_outage_makes_the_list_stale_through_the_receive_pass` (real hub stopped). Mutant c13. |
| CC 2 | receive check derives the fingerprint from the unverified field and fails open | A | fingerprint of the roster key that verified (equal to the signer's), else of the signer field; none derivable → REFUSED (receive) / ignored (stage). Existing E-39 test; mutant c3. |
| CC 3, OC 2 | the committed `policy.rs` stale gate not built | A | `policy::stale_gate(urgent, trust, narrowing)` called in `assess`; urgent only for `OwnProject` while narrowing. Test `t3703_stale_gate_keeps_urgent_only_for_the_own_project_while_narrowing`; mutant c14. (Today urgent already needs the roster; the gate binds any widening, which is what was committed.) |
| CC 4, OC 3 | sender-side WAITING reason (design 5.11 first half) self-deferred | D | formally deferred to P-24 (T-3708) for re-review in round 3: the sending sidecar's `Sender` has no recipient-name → key binding (the named roster is the RECEIVER's allow-list, `ReceiveConfig.roster`, consulted only on receive); building it here means a new sender-side directory of recipient keys, which is P-24's status/reason vocabulary unit (it also needs the recipient's card key, B-6). Until then: mail to a revoked agent is stored at the recipient, never dropped (E-39 test), and the recipient's sidecar refuses nothing it should accept. Criterion added to T-3708. |
| CC 5 | reissue chains to the restored file; an up-to-date sidecar fails every pull | A | when the file is behind the log, the reissue chains to the last committed digest (seq = last + 1). Hub test `r2_a_key_lost_by_a_restore_is_still_refused_and_reissue_chains_to_the_committed_digest`; mutant b10. |
| CC 6, OC 5 | `hub predates` unreachable against a real old hub | A | a `hub.version` without `signing_public_key` is the predates class (status `hub predates revocation list`, daily entry), like `method not found`. Test `r2_an_old_hub_without_a_public_key_is_predates_and_no_pin_yet_is_not_tamper`. |
| CC 7 | a corrupted stored list can never heal | A | a list that fails its load check is held as `unverified`: its keys stay refused (fail closed) until the first verified fetch, which REPLACES it with no floor, chain or subset check against the unverified copy. Test `r2_a_corrupted_stored_list_heals_on_the_first_verified_fetch`; mutant c17. |
| CC 8 | revoked branch before name checks; `Retry` on a failed notice can hold the inbox and re-count | A | names checked first; the REVOKED notice is best effort (logged, never `Retry`), so the envelope is handled once and counted once. The notice goes to the `from_agent` the signed message names, the same one-notice-per-message bound as every other notice (unchanged, stated in `handle_msg`). |
| CC 9 | sweep vs open attempts; no-snapshot lookups fail open | A | the sweep skips a message whose hand-over attempt is open (its finalize decides); `revoked` / `revoked_from` read the stored list when no tracker has published (fail closed). Tests `r2_the_sweep_leaves_an_open_attempt_to_its_finalize`, `r2_without_a_tracker_the_stored_list_still_refuses`; mutants c15, c18. |
| CC 10, OC 7 | `key-revoked` names the whole list; no start-time raise | A | names only keys not named before (persisted set), at most 20 plus "and N more"; a reissue with nothing new raises nothing; `Tracker::new` raises for a verified stored list above the raised seq. Test `r2_the_entry_names_only_new_keys_and_a_reissue_raises_nothing`; mutant c16. |
| CC 11 | benign outcomes flagged as tamper; status rewritten every pass; drifting `since` | A | head/list mismatch (a revoke between the two calls) and "not pinned yet" are not tamper (a pin that DIFFERS still is); the verified status is re-stamped at most once a minute or on change; `since T` is the wall time of the last verified pull, stored once. |
| CC 12, OC 1 | post check during reconcile misses keys the restore lost; identity-less fail open; per-post list verify; lost commit row never repaired; uncoalesced refused-request rows | A | `revoked_keys` adds the committed rows' keys while a reconcile is pending (test in the hub r2 test; mutant b9); no hub identity → the post check fails closed; the verified key set is cached per (file length, mtime); a file ahead of the log gets its committed row written at start (`startup_reconcile_with`, test `r2_a_torn_committed_row_blocks_and_a_lost_row_is_repaired`); refused-request rows at most 10 a minute. |
| OC 4 | identity unloaded → revocation arm skipped | A | as CC 12. |
| OC 6 | torn committed row lets a restore dodge the block | A | any `revocation-committed` line that cannot be read blocks (`REVOCATION_RECONCILE`) until reissue. Same hub test; mutant b11. |
| OC 8 | one attention row per swept message | A | one entry per sweep; the per-message REVOKED notices stay (each sender is told), bounded by what was stored. |
| OC 9 | per-post re-verify under the lock | A | as CC 12 (cache). |
| OC 10 | ordering mutant missing | A | c12 "messages handled before the head" (the pull call moved off the start of the pass). |
| OC 11 | 16-hex prefix can over-refuse on a 64-bit collision | R | stated in design 6.5; over-refusal only, not attacker-forceable without ~2^64 work AND a roster entry at the receiver; persisting the full key on the STORED line is a store-format change outside this unit. |

Re-run after the fixes: hub 11 unit + 3 real-hub tests (full hub suite green), sidecar 12 unit + 2 real-hub + policy tests, full sidecar
suite; mutants: slice B 11, slice C 18 (table in `T-3703-mutants.md`).

## Part 3: mutant table (measured after the fixes)

# T-3703 mutants (FP 3.1.b)

Runners: `crates/termlink-session/tests/mutants/run-t3703a.py` (slice A), `run-t3703b.py [id ...]` (slice B), `run-t3703c.py [id ...]` (slice C). Each applies one change, runs the slice's tests, restores the file; a compile error is not a kill.

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

## Slice B (hub revocation list) — 11/11 killed, 2026-11-18 (after review round 2; b9–b11 added for the r2 fixes)

In the first run b5 as first written did not compile (not a kill); it was rewritten (`Ok(_) => return None`) and is killed here.

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
| b9-post-check-ignores-reconcile | while reconciling, the post check misses keys the restore lost (r2 OC 1) | killed |
| b10-reissue-chains-to-file | a reissue chains to the restored file, not the committed head (r2 CC 5) | killed |
| b11-torn-row-ignored | a torn committed row does not block (r2 OC 6) | killed |

11/11 killed

## Slice C (sidecar revocation pull) — 18/18 killed, 2026-11-18 (after review round 2; c12–c18 added, c3 retargeted)

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
| c12-handle-before-head | messages are handled before the head is read (design 5.4, r2 OC 10) | killed |
| c13-outage-not-counted | a pass that cannot reach the hub is not a failed pull (r2 CC 1) | killed |
| c14-no-stale-gate | the stale state does not narrow urgent (DR-8 A, r2 CC 3) | killed |
| c15-no-fallback | with no snapshot, nothing is revoked (r2 CC 9) | killed |
| c16-raise-all | the entry names every key, not the new ones (r2 CC 10) | killed |
| c17-no-heal | an unverifiable stored list keeps its floor and keys (r2 CC 7) | killed |
| c18-sweep-open-attempt | the sweep refuses a message whose attempt is open (r2 CC 9) | killed |

18/18 killed

## Part 4: the code and test changes made for the dispositions (diff)

```diff
diff --git a/crates/termlink-hub/src/channel.rs b/crates/termlink-hub/src/channel.rs
index 966df68b1..f7c387e8d 100644
--- a/crates/termlink-hub/src/channel.rs
+++ b/crates/termlink-hub/src/channel.rs
@@ -2912,9 +2912,13 @@ pub(crate) fn key_card_gate(
         Check::NotGated => return None,
         // T-3703 (design v2 5.8): the card matches; its key must not be on the hub's own revocation list.
         Check::Pass { agent } => {
-            let ident = crate::hub_identity::current()?;
             let fp = termlink_session::signed_state::key_fingerprint(signer);
-            match crate::revocation::revoked_keys(dir, &ident.hub_id, ident.signing_key()) {
+            // no identity loaded = the list cannot be read: fail closed like an unreadable list (r2 CC 12, OC 4)
+            let keys = match crate::hub_identity::current() {
+                Some(ident) => crate::revocation::revoked_keys(dir, &ident.hub_id, ident.signing_key()),
+                None => Err("the hub identity is not loaded".to_string()),
+            };
+            match keys {
                 Ok(keys) if !keys.contains(&fp) => return None,
                 Ok(_) => (
                     error_code::KEY_REVOKED,
diff --git a/crates/termlink-hub/src/revocation.rs b/crates/termlink-hub/src/revocation.rs
index 1c074e178..a847250a4 100644
--- a/crates/termlink-hub/src/revocation.rs
+++ b/crates/termlink-hub/src/revocation.rs
@@ -285,40 +285,112 @@ pub fn get(dir: &Path, hub_id: &str, key: &SigningKey) -> Result<J, String> {
     load(dir, hub_id, key)
 }
 
-/// The fingerprints on the current list, for the hub's own post check. An unreadable list is an `Err`.
+/// Verified key sets per directory, keyed by the list file's (len, mtime): the post check does not re-verify an
+/// unchanged list on every post (r2 OC 9 / CC 12).
+type Stamp = (u64, u128);
+static KEY_CACHE: Mutex<BTreeMap<PathBuf, (Stamp, BTreeSet<String>)>> = Mutex::new(BTreeMap::new());
+
+fn file_stamp(dir: &Path) -> Option<Stamp> {
+    let m = std::fs::metadata(list_path(dir)).ok()?;
+    let t = m.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_nanos();
+    Some((m.len(), t))
+}
+
+/// The fingerprints the hub's own post check refuses. An unreadable list is an `Err` (the caller fails closed).
+/// While a reconcile is pending (the file is behind the admin log), the keys of the committed rows are added, so
+/// a key whose entry the restore lost is still refused (r2 OC 1 / CC 12).
 pub fn revoked_keys(dir: &Path, hub_id: &str, key: &SigningKey) -> Result<BTreeSet<String>, String> {
     let _g = lock();
-    let l = load(dir, hub_id, key)?;
-    Ok(entries_of(&l).iter().filter_map(|e| entry_key(e).map(str::to_string)).collect())
+    let stamp = file_stamp(dir);
+    let cached = stamp.and_then(|st| {
+        KEY_CACHE.lock().unwrap_or_else(|p| p.into_inner()).get(dir).filter(|(s, _)| *s == st).map(|(_, k)| k.clone())
+    });
+    let mut keys = match cached {
+        Some(k) => k,
+        None => {
+            let l = load(dir, hub_id, key)?;
+            let k: BTreeSet<String> = entries_of(&l).iter().filter_map(|e| entry_key(e).map(str::to_string)).collect();
+            if let Some(st) = stamp {
+                KEY_CACHE.lock().unwrap_or_else(|p| p.into_inner()).insert(dir.to_path_buf(), (st, k.clone()));
+            }
+            k
+        }
+    };
+    if reconcile_reason(dir).is_some() {
+        let log = dir.join(crate::rpc_audit::ADMIN_FILE_NAME);
+        for (_, _, added) in committed_rows(&log) {
+            keys.extend(added.iter().filter_map(|e| e.get("key").and_then(|k| k.as_str()).map(str::to_string)));
+        }
+    }
+    Ok(keys)
 }
 
-/// Committed rows in the admin log, in order: (seq, digest, entries added).
-fn committed_rows(admin_log: &Path) -> Vec<(i64, String, Vec<serde_json::Value>)> {
-    let Ok(text) = std::fs::read_to_string(admin_log) else { return vec![] };
-    text.lines()
-        .filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
-        .filter(|v| v.get("event").and_then(|e| e.as_str()) == Some(COMMITTED_EVENT))
-        .filter_map(|v| {
+type Committed = (i64, String, Vec<serde_json::Value>);
+
+/// Committed rows in the admin log, in order: (seq, digest, entries added), and how many lines named the event
+/// but could not be read (a torn row; reconcile treats any as a block, r2 OC 6).
+fn committed_rows_checked(admin_log: &Path) -> (Vec<Committed>, usize) {
+    let Ok(text) = std::fs::read_to_string(admin_log) else { return (vec![], 0) };
+    let mut rows = Vec::new();
+    let mut torn = 0;
+    for l in text.lines().filter(|l| l.contains(COMMITTED_EVENT)) {
+        let parsed = serde_json::from_str::<serde_json::Value>(l).ok().and_then(|v| {
+            if v.get("event").and_then(|e| e.as_str()) != Some(COMMITTED_EVENT) {
+                return None;
+            }
             Some((
                 v.get("seq")?.as_i64()?,
                 v.get("digest")?.as_str()?.to_string(),
                 v.get("added").and_then(|a| a.as_array()).cloned().unwrap_or_default(),
             ))
-        })
-        .collect()
+        });
+        match parsed {
+            Some(r) => rows.push(r),
+            None => torn += 1,
+        }
+    }
+    (rows, torn)
+}
+
+fn committed_rows(admin_log: &Path) -> Vec<Committed> {
+    committed_rows_checked(admin_log).0
 }
 
 /// Design v2 5.7: compare the file with the last committed row. Returns messages for attention rows; a file
 /// behind the log (or unreadable while the log shows commits) blocks `head`/`get`/`revoke` until [`reissue`].
 pub fn startup_reconcile(dir: &Path, admin_log: &Path, hub_id: &str, key: &SigningKey) -> Vec<String> {
+    startup_reconcile_with(dir, admin_log, hub_id, key, |_| Ok(()))
+}
+
+/// [`startup_reconcile`] with `repair`: when the file is ahead of the log (a committed row was lost), the missing
+/// committed row is written through `repair`, so a later restore to before it is still detected (r2 CC 12).
+pub fn startup_reconcile_with(
+    dir: &Path,
+    admin_log: &Path,
+    hub_id: &str,
+    key: &SigningKey,
+    repair: impl Fn(serde_json::Value) -> Result<(), String>,
+) -> Vec<String> {
     let mut out = Vec::new();
-    let rows = committed_rows(admin_log);
+    let (rows, torn) = committed_rows_checked(admin_log);
     let last = rows.last().map(|(s, d, _)| (*s, d.clone()));
     let _g = lock();
     let block = |why: String, out: &mut Vec<String>| {
         RECONCILE.lock().unwrap_or_else(|p| p.into_inner()).insert(dir.to_path_buf(), why.clone());
         out.push(why);
     };
+    if torn > 0 {
+        block(format!("{torn} revocation-committed row(s) in the admin log cannot be read; the floor is unknown"), &mut out);
+        return out;
+    }
+    let repair_row = |l: &J| {
+        let known: BTreeSet<String> = rows
+            .iter()
+            .flat_map(|(_, _, a)| a.iter().filter_map(|e| e.get("key").and_then(|k| k.as_str()).map(str::to_string)))
+            .collect();
+        let added: Vec<J> = entries_of(l).into_iter().filter(|e| entry_key(e).is_some_and(|k| !known.contains(k))).collect();
+        repair(committed_row(l, &added, "startup-repair"))
+    };
     match (load(dir, hub_id, key), last) {
         (Err(e), Some((s, _))) => block(format!("{e}; the admin log shows seq {s} committed"), &mut out),
         (Err(e), None) => block(e, &mut out),
@@ -329,11 +401,13 @@ pub fn startup_reconcile(dir: &Path, admin_log: &Path, hub_id: &str, key: &Signi
             } else if fs == s && digest_of(&l) != d {
                 block(format!("revocation list seq {fs} has another digest than the committed one (a fork)"), &mut out);
             } else if fs > s {
-                out.push(format!("revocation list seq {fs} is ahead of the last committed row (seq {s}): a commit row was lost"));
+                let r = repair_row(&l).err().map(|e| format!(" (repair row not written: {e})")).unwrap_or_default();
+                out.push(format!("revocation list seq {fs} is ahead of the last committed row (seq {s}): a commit row was lost; repaired{r}"));
             }
         }
         (Ok(l), None) if seq_of(&l) > 0 => {
-            out.push(format!("revocation list seq {} has no committed row in the admin log", seq_of(&l)))
+            let r = repair_row(&l).err().map(|e| format!(" (repair row not written: {e})")).unwrap_or_default();
+            out.push(format!("revocation list seq {} has no committed row in the admin log; repaired{r}", seq_of(&l)))
         }
         (Ok(_), None) => {}
     }
@@ -364,8 +438,15 @@ pub fn reissue(
             entries.push(e);
         }
     }
-    let top = rows.last().map(|(s, _, _)| *s).unwrap_or(0).max(file.as_ref().map(seq_of).unwrap_or(0));
-    let prev = file.as_ref().map(digest_of).or_else(|| rows.last().map(|(_, d, _)| d.clone())).unwrap_or(ZERO.into());
+    let logged_top = rows.last().map(|(s, _, _)| *s).unwrap_or(0);
+    let file_top = file.as_ref().map(seq_of).unwrap_or(-1);
+    let top = logged_top.max(file_top).max(0);
+    // chain to the highest version any sidecar may hold, so one holding it sees seq + 1 with a matching prev
+    let prev = if file_top >= logged_top {
+        file.as_ref().map(digest_of).unwrap_or(ZERO.into())
+    } else {
+        rows.last().map(|(_, d, _)| d.clone()).unwrap_or(ZERO.into())
+    };
     let base = J::Obj(vec![("seq".into(), J::Int(top)), ("digest".into(), J::Str(prev))]);
     let sealed = issue(dir, hub_id, key, &base, entries, now_ms, log, &[])?;
     RECONCILE.lock().unwrap_or_else(|p| p.into_inner()).remove(dir);
@@ -577,4 +658,53 @@ mod tests {
         assert!(e.contains("1 MiB"), "{e}");
         assert!(!list_path(d.path()).exists());
     }
+
+    #[test]
+    fn r2_a_key_lost_by_a_restore_is_still_refused_and_reissue_chains_to_the_committed_digest() {
+        let d = tmp();
+        let k = hub_key();
+        let log = d.path().join(crate::rpc_audit::ADMIN_FILE_NAME);
+        revoke(d.path(), HUB, &k, &op(&fp(1)), 1, logger(&log)).unwrap();
+        let backup = std::fs::read(list_path(d.path())).unwrap();
+        let two = revoke(d.path(), HUB, &k, &op(&fp(2)), 2, logger(&log)).unwrap();
+        std::fs::write(list_path(d.path()), &backup).unwrap();
+        startup_reconcile(d.path(), &log, HUB, &k);
+        let keys = revoked_keys(d.path(), HUB, &k).unwrap();
+        assert!(keys.contains(&fp(2)), "the post check still refuses the key the restore lost (r2 OC 1)");
+        reissue(d.path(), &log, HUB, &k, 3, logger(&log)).unwrap();
+        let l = get(d.path(), HUB, &k).unwrap();
+        assert_eq!(l.get("seq"), Some(&J::Int(3)));
+        assert_eq!(l.get("prev_digest").and_then(J::as_str), Some(two.digest.as_str()), "a sidecar holding seq 2 chains (r2 CC 5)");
+    }
+
+    #[test]
+    fn r2_a_torn_committed_row_blocks_and_a_lost_row_is_repaired() {
+        let d = tmp();
+        let k = hub_key();
+        let log = d.path().join("admin.jsonl");
+        revoke(d.path(), HUB, &k, &op(&fp(1)), 1, logger(&log)).unwrap();
+        {
+            use std::io::Write;
+            let mut f = std::fs::OpenOptions::new().append(true).open(&log).unwrap();
+            writeln!(f, "{{\"event\":\"{COMMITTED_EVENT}\",\"seq\":2,\"dig").unwrap();
+        }
+        let msgs = startup_reconcile(d.path(), &log, HUB, &k);
+        assert!(msgs.iter().any(|m| m.contains("cannot be read")), "{msgs:?}");
+        assert!(head(d.path(), HUB, &k).is_err());
+        RECONCILE.lock().unwrap().remove(d.path());
+        // a list written with no committed row: the start-up check writes the missing row
+        let d2 = tmp();
+        let log2 = d2.path().join("admin.jsonl");
+        revoke(d2.path(), HUB, &k, &op(&fp(1)), 1, |_: &J, _: &[J]| Ok(())).unwrap();
+        let rows = std::cell::RefCell::new(vec![]);
+        let msgs = startup_reconcile_with(d2.path(), &log2, HUB, &k, |r| {
+            rows.borrow_mut().push(r);
+            Ok(())
+        });
+        assert!(msgs.iter().any(|m| m.contains("repaired")), "{msgs:?}");
+        let rows = rows.into_inner();
+        assert_eq!(rows.len(), 1);
+        assert_eq!(rows[0]["seq"], 1);
+        assert_eq!(rows[0]["added"][0]["key"], fp(1));
+    }
 }
diff --git a/crates/termlink-hub/src/router.rs b/crates/termlink-hub/src/router.rs
index 242934d96..c8a97a315 100644
--- a/crates/termlink-hub/src/router.rs
+++ b/crates/termlink-hub/src/router.rs
@@ -1064,8 +1064,11 @@ fn handle_revocation_request(id: serde_json::Value, params: &serde_json::Value)
     let r = match crate::revocation::check_request(&req, now_ms(), &cards) {
         Ok(r) => r,
         Err(e) => {
-            let row = serde_json::json!({"event": "revocation-request-refused", "attention": true, "reason": e});
-            let _ = crate::rpc_audit::admin_note(row);
+            // at most REFUSED_ROWS_PER_MINUTE rows a minute: any Interact caller can send these (r2 CC 12)
+            if refused_request_row_allowed(now_ms()) {
+                let row = serde_json::json!({"event": "revocation-request-refused", "attention": true, "reason": e});
+                let _ = crate::rpc_audit::admin_note(row);
+            }
             return revocation_error(id, &e);
         }
     };
@@ -1079,6 +1082,19 @@ fn handle_revocation_request(id: serde_json::Value, params: &serde_json::Value)
     }
 }
 
+const REFUSED_ROWS_PER_MINUTE: u64 = 10;
+static REFUSED_ROWS: std::sync::Mutex<(u64, u64)> = std::sync::Mutex::new((0, 0));
+
+fn refused_request_row_allowed(now_ms: u64) -> bool {
+    let mut g = REFUSED_ROWS.lock().unwrap_or_else(|p| p.into_inner());
+    let minute = now_ms / 60_000;
+    if g.0 != minute {
+        *g = (minute, 0);
+    }
+    g.1 += 1;
+    g.1 <= REFUSED_ROWS_PER_MINUTE
+}
+
 fn handle_revocation_reissue(id: serde_json::Value, peer_addr: Option<&str>) -> RpcResponse {
     if let Some(r) = key_card_local_only(&id, "hub.revocation_reissue", peer_addr) {
         return r;
diff --git a/crates/termlink-hub/src/server.rs b/crates/termlink-hub/src/server.rs
index b2d1d6ef2..13db66397 100644
--- a/crates/termlink-hub/src/server.rs
+++ b/crates/termlink-hub/src/server.rs
@@ -621,7 +621,8 @@ pub async fn run_with_tcp(
     // blocks head/get until hub.revocation_reissue; a lost commit row or unreadable list raises attention.
     if let Some(ident) = crate::hub_identity::current() {
         let log = runtime_dir.join(crate::rpc_audit::ADMIN_FILE_NAME);
-        for m in crate::revocation::startup_reconcile(&runtime_dir, &log, &ident.hub_id, ident.signing_key()) {
+        let repair = |row: serde_json::Value| crate::rpc_audit::admin_note(row).map_err(|e| e.to_string());
+        for m in crate::revocation::startup_reconcile_with(&runtime_dir, &log, &ident.hub_id, ident.signing_key(), repair) {
             tracing::warn!(detail = %m, "T-3703: revocation list reconcile");
             let row = serde_json::json!({"event": "revocation-reconcile", "attention": true, "detail": m});
             if let Err(e) = crate::rpc_audit::admin_note(row) {
diff --git a/crates/termlink-sidecar/src/policy.rs b/crates/termlink-sidecar/src/policy.rs
index 543190ffc..9fa227193 100644
--- a/crates/termlink-sidecar/src/policy.rs
+++ b/crates/termlink-sidecar/src/policy.rs
@@ -143,6 +143,10 @@ impl Policy for BuiltIn {
         };
         let asks_urgent = priority >= URGENT_AT;
         let urgent = asks_urgent && trust == Trust::OwnProject;
+        // T-3703 (DR-8 A, FCW-95/96): while the revocation list is stale or unverified since start, urgent is
+        // allowed only from the own project's roster. Today urgent already needs the roster; this gate binds any
+        // later widening of the urgent allow-list.
+        let urgent = stale_gate(urgent, trust, self.store.as_deref().is_some_and(crate::revocation::narrowing));
         Assessment {
             trust,
             priority,
@@ -157,6 +161,12 @@ impl Policy for BuiltIn {
     }
 }
 
+/// T-3703 (DR-8 A): while `narrowing` (the revocation list is stale or not verified since start), urgent stays
+/// urgent only for the own project's roster; anything else is downgraded to normal (never dropped).
+pub fn stale_gate(urgent: bool, trust: Trust, narrowing: bool) -> bool {
+    urgent && (!narrowing || trust == Trust::OwnProject)
+}
+
 /// SI-23: read the switch now. Absent flag means on; anything else, including
 /// a flag that cannot be checked, means off (fail closed).
 /// Flag file in the store dir; present means the sidecar is drained (PH-2 row
diff --git a/crates/termlink-sidecar/src/pull.rs b/crates/termlink-sidecar/src/pull.rs
index 53f4e1d93..b152646e6 100644
--- a/crates/termlink-sidecar/src/pull.rs
+++ b/crates/termlink-sidecar/src/pull.rs
@@ -320,6 +320,12 @@ impl Inner {
         }
     }
 
+    /// T-3703 (r2 CC 1): the hub could not be asked this pass; the stale rule still counts it.
+    fn revocation_outage(&self, why: &str) {
+        let mut t = self.revocation.lock().unwrap_or_else(|p| p.into_inner());
+        t.outage(&self.store, std::time::Instant::now(), why);
+    }
+
     /// T-3703 (AL-22): one revocation pull. A failure never holds mail (the stale rule narrows, never stops).
     fn pull_revocations(&self, version: &Value) {
         let rpc = |m: &str| -> Result<Value, (i64, String)> {
@@ -367,15 +373,18 @@ impl Inner {
         if !policy::receiving_enabled(&self.store) {
             return PassResult::Drained;
         }
-        match self.check_pin() {
+        let version = match self.check_pin() {
             Ok(version) => {
                 mem.held = None;
                 self.observe_impersonation();
-                // T-3703 (design 5.4): the revocation head is compared before any message is handled.
-                self.pull_revocations(&version);
+                version
+            }
+            Err(PinError::Unreachable(e)) => {
+                self.revocation_outage(&format!("hub unreachable: {e}"));
+                return PassResult::Waiting;
             }
-            Err(PinError::Unreachable(_)) => return PassResult::Waiting,
             Err(PinError::Mismatch { declared, found }) => {
+                self.revocation_outage("hub id mismatch");
                 let why = format!(
                     "hub id mismatch: declared {declared}, hub says {}; inbox not read",
                     found.as_deref().unwrap_or("nothing")
@@ -386,7 +395,9 @@ impl Inner {
                 mem.held = Some(why.clone());
                 return PassResult::Held(why);
             }
-        }
+        };
+        // T-3703 (design 5.4): the revocation head is compared before any message is handled.
+        self.pull_revocations(&version);
         let topic = inbox_topic(&self.cfg.declared_hub_id, &self.cfg.agent);
         let mut handled = 0;
         loop {
@@ -471,23 +482,29 @@ impl Inner {
             Ok((m, _signer)) => m,
             Err(r) => return self.refuse(env, None, wire::refusal_code(&r), &r.to_string()),
         };
+        if !name_ok(&msg.conv) || !name_ok(&msg.msg_id) {
+            return self.refuse(env, None, "bad-name", "conv or msg_id is not a valid name");
+        }
         // T-3703 (FCW-99, E-39): a message signed by a key on the held revocation list is refused, not stored,
-        // and the sender gets a signed REVOKED notice.
-        if let Some(fp) = env.signed.as_ref().and_then(|s| termlink_session::signed_state::fingerprint_of_hex(&s.signer_pk))
-            && crate::revocation::revoked(&self.store, &fp)
-        {
+        // and the sender gets a signed REVOKED notice. The fingerprint is of the key that verified (the roster's
+        // copy of it, equal to the signer's); one that cannot be derived is refused, never passed (r2 CC 2).
+        let fp = self.cfg.roster.get(&msg.from).map(termlink_session::signed_state::key_fingerprint).or_else(|| {
+            env.signed.as_ref().and_then(|s| termlink_session::signed_state::fingerprint_of_hex(&s.signer_pk))
+        });
+        let Some(fp) = fp else {
+            return self.refuse(env, Some(&msg), crate::revocation::REASON_REVOKED, "signer key fingerprint cannot be derived");
+        };
+        if crate::revocation::revoked(&self.store, &fp) {
             let out = self.refuse(env, Some(&msg), crate::revocation::REASON_REVOKED, &format!("signed by revoked key {fp}"));
             if matches!(out, Outcome::Done) {
                 crate::revocation::count_refusal(&self.store);
+                // best effort: a notice that cannot be queued never holds the inbox (r2 CC 8)
                 if let Err(e) = self.report(&msg, crate::revocation::REVOKED) {
-                    return Outcome::Retry(e);
+                    eprintln!("termlink-sidecar: REVOKED notice for {}/{} not queued: {e}", msg.conv, msg.msg_id);
                 }
             }
             return out;
         }
-        if !name_ok(&msg.conv) || !name_ok(&msg.msg_id) {
-            return self.refuse(env, None, "bad-name", "conv or msg_id is not a valid name");
-        }
         // Receiver checks (C-7). Their inputs are the verified sender and the
         // signed priority only.
         let a = self.policy.assess(&msg);
@@ -662,10 +679,12 @@ impl Inner {
             Ok(s) => s,
             Err(r) => return self.ignore_stage(env, &r.to_string()),
         };
-        if let Some(fp) = env.signed.as_ref().and_then(|s| termlink_session::signed_state::fingerprint_of_hex(&s.signer_pk))
-            && crate::revocation::revoked(&self.store, &fp)
-        {
-            return self.ignore_stage(env, &format!("signed by revoked key {fp} (FCW-99)"));
+        match env.signed.as_ref().and_then(|s| termlink_session::signed_state::fingerprint_of_hex(&s.signer_pk)) {
+            None => return self.ignore_stage(env, "signer key fingerprint cannot be derived (FCW-99)"),
+            Some(fp) if crate::revocation::revoked(&self.store, &fp) => {
+                return self.ignore_stage(env, &format!("signed by revoked key {fp} (FCW-99)"));
+            }
+            Some(_) => {}
         }
         if st.to_agent != self.cfg.agent {
             return self.ignore_stage(env, &format!("addressed to {}, this is {}", st.to_agent, self.cfg.agent));
diff --git a/crates/termlink-sidecar/src/revocation.rs b/crates/termlink-sidecar/src/revocation.rs
index 89bbcba40..767a19525 100644
--- a/crates/termlink-sidecar/src/revocation.rs
+++ b/crates/termlink-sidecar/src/revocation.rs
@@ -44,6 +44,7 @@ const STATUS_STATE: &str = "revocation-status";
 const SWITCH_STATE: &str = "revocation-pull";
 const SWITCH_SEEN_STATE: &str = "revocation-pull-seen";
 const REFUSED_COUNT_STATE: &str = "revocation-refused-count";
+const RAISED_KEYS_STATE: &str = "revocation-attention-keys";
 
 /// What every other part of the sidecar reads: the refused keys and whether the stale rule narrows.
 #[derive(Debug, Clone, Default)]
@@ -61,9 +62,25 @@ fn publish(store: &Store, keys: &BTreeSet<String>, narrowing: bool) {
         .insert(store.dir().to_path_buf(), Snap { keys: keys.clone(), narrowing });
 }
 
+/// The held keys: the published snapshot, or, before any tracker has published for this store (another process,
+/// the window before `Tracker::new`), the keys of the stored list read without verification: fail closed, never
+/// "nothing revoked" (r2 CC 9).
+fn held_keys(store: &Store) -> BTreeSet<String> {
+    if let Some(s) = SNAPS.lock().unwrap_or_else(|p| p.into_inner()).get(store.dir()) {
+        return s.keys.clone();
+    }
+    store
+        .read_state(LIST_STATE)
+        .ok()
+        .flatten()
+        .and_then(|t| ss::parse_strict(t.as_bytes()).ok())
+        .map(|l| keys_of(&entries_of(&l)))
+        .unwrap_or_default()
+}
+
 /// FCW-99: the full fingerprint `ed25519:<64 hex>` of a signer is on the held list.
 pub fn revoked(store: &Store, fingerprint: &str) -> bool {
-    SNAPS.lock().unwrap_or_else(|p| p.into_inner()).get(store.dir()).is_some_and(|s| s.keys.contains(fingerprint))
+    held_keys(store).contains(fingerprint)
 }
 
 /// The short sender id of a stored message (`from`, the first 16 hex of sha256(key)) matches a held key. Used
@@ -72,11 +89,7 @@ pub fn revoked_from(store: &Store, from: &str) -> bool {
     if from.len() < 16 {
         return false;
     }
-    SNAPS
-        .lock()
-        .unwrap_or_else(|p| p.into_inner())
-        .get(store.dir())
-        .is_some_and(|s| s.keys.iter().any(|k| k.strip_prefix("ed25519:").is_some_and(|h| h.starts_with(from))))
+    held_keys(store).iter().any(|k| k.strip_prefix("ed25519:").is_some_and(|h| h.starts_with(from)))
 }
 
 /// DR-8 A: true while the list is STALE or not yet verified since start.
@@ -133,6 +146,13 @@ pub struct Tracker {
     state: State,
     held: Option<Held>,
     stale_raised: bool,
+    /// Wall-clock ms of the last verified pull (or of the start): the stable `since T` (r2 CC 11).
+    since_wall: u64,
+    /// The last `verified` status write: (seq, when), so an unchanged list is re-stamped at most once a minute.
+    status_written: Option<(i64, Instant)>,
+    /// The held list failed its load check: its keys are refused until the first verified fetch replaces them,
+    /// and no floor, chain or subset check is made against it (r2 CC 7).
+    held_unverified: bool,
 }
 
 /// One RPC to the home hub: Ok(result) or Err((code, message)). Code 0 = transport.
@@ -149,6 +169,9 @@ impl Tracker {
             state: State::UnverifiedSinceStart,
             held: None,
             stale_raised: false,
+            since_wall: crate::store::now_ms(),
+            status_written: None,
+            held_unverified: false,
         };
         if let Ok(Some(text)) = store.read_state(LIST_STATE) {
             let pk = store.read_state(HUB_PK_STATE).ok().flatten().and_then(|h| ss::verifying_key_of_hex(h.trim()));
@@ -166,6 +189,9 @@ impl Tracker {
         }
         t.publish(store);
         t.write_status(store, "unverified since start (no pull yet)");
+        if !t.held_unverified {
+            t.raise_new_entries(store); // a crash between install and raise (design 5.9)
+        }
         t
     }
 
@@ -177,9 +203,19 @@ impl Tracker {
             let mut h = held_of(&l);
             h.digest = String::new(); // never matches a head: the next pull fetches the full list
             self.held = Some(h);
+            self.held_unverified = true;
         }
     }
 
+    /// The hub could not be asked at all this pass (`hub.version` failed, or names another hub): a failed pull
+    /// for the stale rule (r2 CC 1).
+    pub fn outage(&mut self, store: &Store, now: Instant, why: &str) {
+        if self.state == State::Off {
+            return;
+        }
+        self.failed(store, now, why, false);
+    }
+
     pub fn set_stale_after(&mut self, d: Duration) {
         self.stale_after = d;
     }
@@ -246,11 +282,15 @@ impl Tracker {
     fn pull(&mut self, store: &Store, version: &Value, rpc: Rpc<'_>) -> Result<(), (String, bool)> {
         let pk_hex = version.get("signing_public_key").and_then(Value::as_str);
         let pinned = crate::hub_key_pin::verified_hub_key(store, &self.hub_id, version);
+        let pin_stored = store.read_state(&crate::hub_key_pin::pin_state_name(&self.hub_id)).ok().flatten().is_some();
         let pk = match (pk_hex.and_then(ss::verifying_key_of_hex), pinned) {
             (Some(k), Some(fp)) if ss::key_fingerprint(&k) == fp => k,
             (Some(_), Some(_)) => return Err(("hub public key does not hash to the pinned fingerprint".into(), true)),
-            (None, _) => return Err(("hub offers no signing public key".into(), false)),
-            (Some(_), None) => return Err(("hub signing key not pinned (or pin mismatch)".into(), true)),
+            // a hub older than T-3703 offers no public key: the same class as `method not found` (r2 CC 6, OC 5)
+            (None, _) => return Err(("hub predates revocation list".into(), false)),
+            // a pin that differs is interference; no pin yet (first contact not over a trusted path) is not
+            (Some(_), None) if pin_stored => return Err(("hub signing key differs from the pin".into(), true)),
+            (Some(_), None) => return Err(("hub signing key not pinned yet".into(), false)),
         };
         let h = match rpc("hub.revocation_head") {
             Ok(v) => v,
@@ -265,6 +305,9 @@ impl Tracker {
         }
         let hseq = head.get("seq").and_then(J::as_int).ok_or(("head without seq".to_string(), true))?;
         let hdig = head.get("digest").and_then(J::as_str).unwrap_or("").to_string();
+        if self.held_unverified {
+            return self.fetch(store, &pk, hseq, &hdig, rpc); // replaces an unverifiable stored list (r2 CC 7)
+        }
         if let Some(held) = &self.held {
             if hseq < held.seq {
                 return Err((format!("ROLLBACK refused, kept seq {}", held.seq), true));
@@ -272,8 +315,6 @@ impl Tracker {
             if hseq == held.seq {
                 return if hdig == held.digest {
                     Ok(())
-                } else if held.digest.is_empty() {
-                    self.fetch(store, &pk, hseq, &hdig, rpc) // a held list that failed its load check
                 } else {
                     Err((format!("fork: seq {hseq} with another digest"), true))
                 };
@@ -292,14 +333,14 @@ impl Tracker {
         let lseq = l.get("seq").and_then(J::as_int).unwrap_or(-1);
         let ldig = l.get("digest").and_then(J::as_str).unwrap_or("");
         if lseq != hseq || ldig != hdig {
-            return Err(("list does not match its head".into(), true));
+            // a revocation between head and get: benign, the next pass reads both again (r2 CC 11)
+            return Err(("list does not match its head (changed between head and get)".into(), false));
         }
         if l.get("hub_id").and_then(J::as_str) != Some(self.hub_id.as_str()) {
             return Err(("list names another hub".into(), true));
         }
-        if let Some(held) = &self.held {
-            if !held.digest.is_empty()
-                && lseq == held.seq + 1
+        if let Some(held) = self.held.as_ref().filter(|_| !self.held_unverified) {
+            if lseq == held.seq + 1
                 && l.get("prev_digest").and_then(J::as_str) != Some(held.digest.as_str())
             {
                 return Err(("list does not chain to the held one".into(), true));
@@ -313,16 +354,22 @@ impl Tracker {
         store.write_state(LIST_STATE, text).map_err(|e| (format!("list not stored: {e}"), false))?;
         let _ = store.write_state(HUB_PK_STATE, &ss::hex(pk.as_bytes()));
         self.held = Some(held_of(&l));
+        self.held_unverified = false;
         Ok(())
     }
 
     fn verified(&mut self, store: &Store, now: Instant, sender: Option<&crate::send::Sender>) {
         self.last_verified = Some(now);
-        self.state = State::Verified;
+        self.since_wall = crate::store::now_ms();
+        let was = std::mem::replace(&mut self.state, State::Verified);
         self.stale_raised = false;
         self.publish(store);
         let seq = self.held.as_ref().map(|h| h.seq).unwrap_or(0);
-        self.write_status(store, &format!("verified seq {seq} at {}", crate::store::now_ms()));
+        let fresh = matches!(self.status_written, Some((s, t)) if s == seq && now.duration_since(t) < Duration::from_secs(60));
+        if !fresh || was != State::Verified {
+            self.write_status(store, &format!("verified seq {seq} at {}", self.since_wall));
+            self.status_written = Some((seq, now));
+        }
         self.raise_new_entries(store);
         if let Some(h) = &self.held {
             refuse_stored(store, &h.keys, sender);
@@ -336,22 +383,34 @@ impl Tracker {
         if h.seq <= raised {
             return;
         }
-        let names: Vec<String> = h
-            .entries
+        // only the keys not named before (r2 CC 10, OC 7); a reissue with nothing new raises nothing
+        let named: BTreeSet<String> =
+            store.read_state(RAISED_KEYS_STATE).ok().flatten().unwrap_or_default().lines().map(str::to_string).collect();
+        let new: Vec<&J> = h.entries.iter().filter(|e| e.get("key").and_then(J::as_str).is_some_and(|k| !named.contains(k))).collect();
+        let all_keys: Vec<String> = h.keys.iter().cloned().collect();
+        if new.is_empty() {
+            let _ = store.write_state(ATTENTION_STATE, &h.seq.to_string());
+            return;
+        }
+        let total = new.len();
+        let names: Vec<String> = new
             .iter()
+            .take(20)
             .map(|e| {
                 let s = |n: &str| e.get(n).and_then(J::as_str).unwrap_or("?").to_string();
                 format!("{}/{} ({}, by {}: {})", s("project"), s("agent"), s("key"), s("by"), s("reason"))
             })
             .collect();
+        let more = if total > names.len() { format!(" and {} more", total - names.len()) } else { String::new() };
         let detail = format!(
-            "hub {} revocation list is at seq {}: {} key(s) revoked: {}; messages signed by them are refused",
+            "hub {} revocation list is at seq {}: {} key(s) newly revoked: {}{more}; messages signed by them are refused",
             self.hub_id,
             h.seq,
-            names.len(),
+            total,
             names.join("; ")
         );
         if store.attention_coalesced("key-revoked", &h.seq.to_string(), &detail).is_ok() {
+            let _ = store.write_state(RAISED_KEYS_STATE, &all_keys.join("\n"));
             let _ = store.write_state(ATTENTION_STATE, &h.seq.to_string());
         }
     }
@@ -369,7 +428,7 @@ impl Tracker {
                 &format!("hub {} serves no revocation list (it predates T-3703); revoked keys are not known here", self.hub_id),
             );
         }
-        let since_ms = crate::store::now_ms().saturating_sub(now.duration_since(self.since()).as_millis() as u64);
+        let since_ms = self.since_wall;
         if self.state != State::Stale && now.duration_since(self.since()) > self.stale_after {
             self.state = State::Stale;
             if !self.stale_raised {
@@ -383,6 +442,7 @@ impl Tracker {
                 );
             }
         }
+        self.status_written = None;
         self.publish(store);
         let line = if why.starts_with("ROLLBACK") || why == "hub predates revocation list" {
             why.to_string()
@@ -407,8 +467,7 @@ fn held_of(l: &J) -> Held {
 
 /// [`refuse_stored`] with the keys currently held (the hand-over re-check calls this).
 pub fn refuse_stored_now(store: &Store, sender: Option<&crate::send::Sender>) -> usize {
-    let keys = SNAPS.lock().unwrap_or_else(|p| p.into_inner()).get(store.dir()).map(|s| s.keys.clone()).unwrap_or_default();
-    refuse_stored(store, &keys, sender)
+    refuse_stored(store, &held_keys(store), sender)
 }
 
 /// FCW-200 sweep: every STORED message from a revoked key that is not handed over, refused or held open by an
@@ -431,6 +490,19 @@ pub fn refuse_stored(store: &Store, keys: &BTreeSet<String>, sender: Option<&cra
             if done("HANDED_OVER") || done("REFUSED") {
                 continue;
             }
+            // an open hand-over attempt is already in the agent's hands; its finalize decides (r2 CC 9)
+            let closed: BTreeSet<String> = lines
+                .iter()
+                .filter(|l| l.event == "ABSENT" && l.msg_id == st.msg_id)
+                .filter_map(|l| l.reason.as_deref().and_then(|r| r.split_whitespace().next()).map(str::to_string))
+                .collect();
+            let open = store
+                .claims_for(&st.msg_id)
+                .map(|a| a.iter().any(|n| !closed.contains(&format!("attempt={n}"))))
+                .unwrap_or(true);
+            if open {
+                continue;
+            }
             let extra = LineExtra { reason: Some(REASON_REVOKED.to_owned()), ..Default::default() };
             let (id, sd, dg) = (st.msg_id.clone(), st.sender.clone(), st.digest.clone());
             let wrote = store.append_event_unless(&conv, "REFUSED", &st.msg_id, &st.sender, &st.digest, extra, move |ls| {
@@ -439,10 +511,6 @@ pub fn refuse_stored(store: &Store, keys: &BTreeSet<String>, sender: Option<&cra
             if let Ok(Some(_)) = wrote {
                 n += 1;
                 count_refusal(store);
-                let _ = store.attention(
-                    "refused-revoked",
-                    &format!("{conv}/{} from {from}: stored before its key was revoked; refused, not handed over (FCW-200)", st.msg_id),
-                );
                 if let (Some(s), Ok((_, payload))) = (sender, store.read_record(&conv, st.seq, &st.msg_id))
                     && let Ok(msg) = crate::wire::parse_payload(&payload)
                     && let Ok(stage) = crate::pull::build_stage(store, s.fingerprint(), s.agent().0, &msg, REVOKED)
@@ -452,6 +520,13 @@ pub fn refuse_stored(store: &Store, keys: &BTreeSet<String>, sender: Option<&cra
             }
         }
     }
+    if n > 0 {
+        // one entry per sweep, not one per message (r2 OC 8)
+        let _ = store.attention(
+            "refused-revoked",
+            &format!("{n} stored message(s) from revoked key(s) refused before hand-over (FCW-200); see the chains' REFUSED lines (reason revoked)"),
+        );
+    }
     n
 }
 
@@ -777,4 +852,98 @@ mod tests {
         assert!(revoked(&f.store, &fp(1)));
         assert_eq!(kinds(&f.store).iter().filter(|k| *k == "revocation-pull-switched").count(), 2);
     }
+
+    // ---- round 2 (docs/reports/T-3703-reviews/r2-dispositions.md) ----
+
+    #[test]
+    fn r2_the_entry_names_only_new_keys_and_a_reissue_raises_nothing() {
+        let f = fx();
+        let hub = Hub::new();
+        let mut t = Tracker::new(&f.store, HUB, PN13);
+        let now = Instant::now();
+        let l1 = list(&f.k, 1, Z, &[fp(1)]);
+        hub.serve(&f.k, &l1);
+        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
+        let l2 = list(&f.k, 2, l1.get("digest").unwrap().as_str().unwrap(), &[fp(1), fp(2)]);
+        hub.serve(&f.k, &l2);
+        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
+        let details: Vec<String> = f
+            .store
+            .attention_entries()
+            .iter()
+            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
+            .filter(|v| v["kind"] == "key-revoked")
+            .map(|v| v["detail"].as_str().unwrap_or("").to_string())
+            .collect();
+        assert_eq!(details.len(), 2);
+        assert!(details[1].contains(&fp(2)) && !details[1].contains(&fp(1)), "{}", details[1]);
+        // a reissue: seq 3, same keys -> nothing new, no entry
+        hub.serve(&f.k, &list(&f.k, 3, l2.get("digest").unwrap().as_str().unwrap(), &[fp(1), fp(2)]));
+        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
+        assert_eq!(kinds(&f.store).iter().filter(|k| *k == "key-revoked").count(), 2);
+    }
+
+    #[test]
+    fn r2_without_a_tracker_the_stored_list_still_refuses() {
+        let f = fx();
+        let l = list(&f.k, 1, Z, &[fp(1)]);
+        f.store.write_state(LIST_STATE, &serde_json::to_string(&l.to_serde()).unwrap()).unwrap();
+        assert!(revoked(&f.store, &fp(1)), "no snapshot published for this store: fail closed from the stored list");
+        assert!(revoked_from(&f.store, &fp(1)["ed25519:".len()..][..16]));
+    }
+
+    #[test]
+    fn r2_a_corrupted_stored_list_heals_on_the_first_verified_fetch() {
+        let f = fx();
+        let hub = Hub::new();
+        let l = list(&f.k, 9, Z, &[fp(1)]);
+        hub.serve(&f.k, &l);
+        let now = Instant::now();
+        Tracker::new(&f.store, HUB, PN13).tick(&f.store, &f.version, &|m| hub.call(m), now, None);
+        // corrupt the stored copy: another key and a huge seq (neither signed)
+        let mut bad = l.clone();
+        bad.set("seq", J::Int(1_000_000));
+        bad.set("entries", J::Arr(vec![J::Obj(vec![("key".into(), J::Str(fp(7)))])]));
+        f.store.write_state(LIST_STATE, &serde_json::to_string(&bad.to_serde()).unwrap()).unwrap();
+        let mut t = Tracker::new(&f.store, HUB, PN13);
+        assert!(revoked(&f.store, &fp(7)), "until verified, the stored keys are refused (fail closed)");
+        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
+        assert_eq!(t.state(), &State::Verified, "{}", line(&f.store));
+        assert_eq!(t.held_seq(), Some(9), "no floor kept from an unverified list");
+        assert!(revoked(&f.store, &fp(1)) && !revoked(&f.store, &fp(7)), "the hub's list replaces it");
+    }
+
+    #[test]
+    fn r2_an_old_hub_without_a_public_key_is_predates_and_no_pin_yet_is_not_tamper() {
+        let f = fx();
+        let hub = Hub::new();
+        let mut t = Tracker::new(&f.store, HUB, PN13);
+        let now = Instant::now();
+        let mut old = f.version.clone();
+        old.as_object_mut().unwrap().remove("signing_public_key");
+        t.tick(&f.store, &old, &|m| hub.call(m), now, None);
+        assert_eq!(line(&f.store), "hub predates revocation list");
+        assert!(kinds(&f.store).contains(&"revocation-hub-predates".to_string()));
+        f.store.remove_state(&crate::hub_key_pin::pin_state_name(HUB)).unwrap();
+        hub.serve(&f.k, &list(&f.k, 1, Z, &[]));
+        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
+        assert!(line(&f.store).contains("not pinned yet"), "{}", line(&f.store));
+        assert!(!kinds(&f.store).contains(&"revocation-tamper".to_string()), "no pin yet is not interference");
+    }
+
+    #[test]
+    fn r2_the_sweep_leaves_an_open_attempt_to_its_finalize() {
+        let f = fx();
+        let key = fp(1);
+        let from = &key["ed25519:".len()..][..16];
+        let keys: BTreeSet<String> = [key.clone()].into();
+        f.store.put(from, "c1", "m1", b"payload").unwrap();
+        f.store.claim("m1", 1, "n1", 1).unwrap();
+        assert_eq!(refuse_stored(&f.store, &keys, None), 0, "attempt 1 is open");
+        f.store
+            .append_event_unless("c1", "ABSENT", "m1", from, "d", LineExtra { reason: Some("attempt=1 nonce=n1".into()), ..Default::default() }, |_| false)
+            .unwrap();
+        assert_eq!(refuse_stored(&f.store, &keys, None), 1, "closed attempt: refused");
+        assert_eq!(kinds(&f.store).iter().filter(|k| *k == "refused-revoked").count(), 1, "one entry per sweep");
+    }
 }
diff --git a/crates/termlink-sidecar/tests/policy_tests.rs b/crates/termlink-sidecar/tests/policy_tests.rs
index 0f8535417..b095cb25c 100644
--- a/crates/termlink-sidecar/tests/policy_tests.rs
+++ b/crates/termlink-sidecar/tests/policy_tests.rs
@@ -366,3 +366,14 @@ fn policy_urgent_from_an_allow_listed_sender_is_kept_and_answered() {
         .collect();
     assert_eq!(al, ["SENT", "POSTED", "RECEIVED", "STORED"]);
 }
+
+// ---- T-3703 (DR-8 A): the stale gate binds any widening of the urgent allow-list -------------------
+
+#[test]
+fn t3703_stale_gate_keeps_urgent_only_for_the_own_project_while_narrowing() {
+    use termlink_sidecar::policy::{Trust, stale_gate};
+    assert!(stale_gate(true, Trust::OwnProject, true), "own project stays urgent when stale");
+    assert!(!stale_gate(true, Trust::Admitted, true), "another project's allowed sender is downgraded when stale");
+    assert!(stale_gate(true, Trust::Admitted, false), "not stale: the allow-list decides");
+    assert!(!stale_gate(false, Trust::OwnProject, true), "the gate never raises urgency");
+}
diff --git a/crates/termlink-sidecar/tests/t3703_revocation_sidecar.rs b/crates/termlink-sidecar/tests/t3703_revocation_sidecar.rs
index 3599e2e09..6e24fea51 100644
--- a/crates/termlink-sidecar/tests/t3703_revocation_sidecar.rs
+++ b/crates/termlink-sidecar/tests/t3703_revocation_sidecar.rs
@@ -137,3 +137,25 @@ fn e39_a_revoked_key_is_refused_within_one_pass_and_stored_mail_from_it_is_refus
     let _ = a.rx.pass();
     assert!(lines(&a.store, "c3").iter().any(|(ev, _, _)| ev == "STORED"), "mail to fx-a kept");
 }
+
+/// r2 CC 1: the stale rule runs when the hub cannot be asked at all (`hub.version` fails), through the real pass.
+#[test]
+fn e41_l4_a_hub_outage_makes_the_list_stale_through_the_receive_pass() {
+    let mut e = Estate::create().unwrap();
+    let id = e.hub_id().unwrap();
+    let _ = rpc(&e, "channel.create", json!({"name": format!("inbox:{id}/fx-b")}));
+    let b = side(&e, &id, "b", "a");
+    b.rx.set_revocation_stale_after(Duration::from_millis(300));
+    let _ = b.rx.pass();
+    assert_eq!(b.rx.revocation_state(), termlink_sidecar::revocation::State::Verified);
+    e.stop_hub();
+    let _ = b.rx.pass();
+    assert_eq!(b.rx.revocation_state(), termlink_sidecar::revocation::State::Verified, "not yet past PN-13");
+    std::thread::sleep(Duration::from_millis(400));
+    let _ = b.rx.pass();
+    assert_eq!(b.rx.revocation_state(), termlink_sidecar::revocation::State::Stale);
+    assert!(termlink_sidecar::revocation::narrowing(&b.store));
+    assert!(kinds(&b.store).iter().any(|k| k == "revocation-stale"));
+    let st = termlink_sidecar::revocation::status(&b.store);
+    assert!(st["revocation"].as_str().unwrap().starts_with("stale since"), "{st}");
+}
```

## Part 5: measured results

- hub: full suite 638 + integration binaries, 0 failed; revocation unit 11/11; real-hub t3703_revocation 3/3
- sidecar: full suite 43 binaries, 0 failed; revocation unit 12/12; real-hub t3703_revocation_sidecar 2/2 (E-39, E-41 L4 outage); policy_tests 6/6
- mutants: slice B 11/11, slice C 18/18 killed
