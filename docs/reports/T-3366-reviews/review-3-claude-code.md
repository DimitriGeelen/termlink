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
