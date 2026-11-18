**Verdict: FAIL** — every round-2 MAJOR is closed in code or soundly deferred, but the OC 6 fix introduces a new MAJOR: one torn committed row makes the hub block the revocation list again on every later start.

## Findings marked A

- **CC 1** — CLOSED — `pass_locked` now calls `revocation_outage` → `Tracker::outage` → `failed()` on both `PinError` arms, and `e41_l4_a_hub_outage_makes_the_list_stale_through_the_receive_pass` drives it through `Receiver::pass` with the hub stopped (mutant c13).
- **CC 2** — CLOSED — `handle_msg` and `handle_stage` no longer skip the check when no fingerprint is derivable (`let Some(fp) = fp else { refuse }` / `None => ignore_stage`); the fail-closed branch has no test or mutant and `_signer` is still discarded (see N2).
- **CC 3, OC 2** — CLOSED — `policy::stale_gate` exists and `assess` calls it with `revocation::narrowing`; `t3703_stale_gate_keeps_urgent_only_for_the_own_project_while_narrowing` and c14 pin the pure function only (see N4).
- **CC 5** — CLOSED — `reissue` chains to the last committed digest when `file_top < logged_top`, and the hub test asserts seq 3 with `prev_digest == two.digest` (mutant b10); the sidecar-through-reissue L4 test the finding asked for was not added.
- **CC 6, OC 5** — CLOSED — `(None, _)` now returns `hub predates revocation list`, and `r2_an_old_hub_without_a_public_key_is_predates_and_no_pin_yet_is_not_tamper` asserts the status line and the `revocation-hub-predates` entry.
- **CC 7** — CLOSED — `held_unverified` sends `pull` straight to `fetch` and filters the held list out of the chain check; `r2_a_corrupted_stored_list_heals_on_the_first_verified_fetch` shows seq 1,000,000 with `fp(7)` replaced by seq 9 with `fp(1)` (mutant c17).
- **CC 8** — CLOSED — `name_ok` moved ahead of the revoked branch and a failed `report` is logged instead of returning `Retry`; no test or mutant covers this, and `from_agent` is still not validated before the notice (the disposition leaves that unchanged).
- **CC 9** — CLOSED — `refuse_stored` skips messages with an unclosed claim and `held_keys` falls back to the stored list, per `r2_the_sweep_leaves_an_open_attempt_to_its_finalize` and `r2_without_a_tracker_the_stored_list_still_refuses` (c15, c18); `deliver.rs` is unchanged, so "its finalize decides" is not evidenced (see N3).
- **CC 10, OC 7** — CLOSED — `raise_new_entries` filters against the persisted `revocation-attention-keys`, caps at 20, and returns without an entry on an empty delta; the test asserts the second entry names only `fp(2)` and the reissue raises nothing (c16). The start-time raise in `Tracker::new` is untested.
- **CC 11** — CLOSED — head/list mismatch and "not pinned yet" return tamper = false while a differing pin stays tamper, the status re-stamp is limited to once a minute, and `since_wall` replaces the recomputed time. Only the not-pinned case is asserted.
- **CC 12, OC 1** — CLOSED — `revoked_keys` adds the committed rows' keys while reconciling (test asserts `fp(2)` is still refused, b9), a missing identity yields `Err`, the key set is cached, `startup_reconcile_with` writes the lost row (tested), and refused rows are capped at 10 a minute. The identity, cache and rate-limit branches have no test.
- **OC 4** — CLOSED — `key_card_gate` maps `hub_identity::current() == None` to `Err`, which takes the refusing arm; untested.
- **OC 6** — CLOSED — `committed_rows_checked` counts unreadable lines and `startup_reconcile_with` blocks on `torn > 0`, with the test asserting `head` is `Err` (b11); the fix is the source of N1.
- **OC 8** — CLOSED — the per-message `store.attention` is replaced by one after the loop, and the sweep test asserts exactly one `refused-revoked` entry.
- **OC 9** — CLOSED — `KEY_CACHE` keyed by (len, mtime) skips `load` and its signature check on an unchanged file; no test covers invalidation.
- **OC 10** — CLOSED — c12-handle-before-head is in the slice C table and killed.

## Findings marked R or D

- **CC 4, OC 3 (D)** — SOUND — the gap is a missing sender-side status reason, not a fail-open (mail is stored at the recipient and the receiver still refuses the revoked key), and it is deferred to a named task for round-3 re-review as OC 3 asked. The T-3708 criterion itself is not in the bundle.
- **OC 11 (R)** — SOUND — the collision causes over-refusal only, needs about 2^64 work plus a roster entry, is stated in design 6.5, and the fix is a store-format change outside this unit.

## New defects

- **N1 — MAJOR** — `termlink-hub/src/revocation.rs: committed_rows_checked` / `startup_reconcile_with`, with `server.rs`.
  - **What:** the torn-row block recurs on every start. `reissue` clears only the in-memory `RECONCILE`; the torn line stays in the append-only admin log. Lines are selected by substring (`l.contains(COMMITTED_EVENT)`), and a line that parses with another event is also counted as torn. The block's own attention row (`"event":"revocation-reconcile"`, detail "N revocation-committed row(s)… cannot be read") matches that substring.
  - **Sequence:** crash mid-write of one committed row → start blocks → operator runs `hub.revocation_reissue` → hub restarts → `torn ≥ 1` again → `head`/`get`/`revoke` are refused. Every sidecar goes STALE after an hour and no revocation can be issued until another local reissue, at every restart. Any admin row whose text contains the event name (an operator's revocation reason, for example) triggers the same block.
  - **Assumption:** `COMMITTED_EVENT` is `revocation-committed` and `admin_note` writes to the same file; the bundle implies both but shows neither literally.
  - **Fix:** count as torn only lines that fail to parse, ignore lines that parse with another event, and let a later reissue row supersede earlier torn lines. Add a reissue-then-restart test.

- **N2 — MINOR** — `termlink-sidecar/src/pull.rs: handle_msg` / `handle_stage`.
  - **What:** the fingerprint still does not come from the verified `_signer`. It comes from `cfg.roster.get(&msg.from)`, whose equality with the verifying key is asserted only in a comment, or from the unverified `signer_pk` field for non-roster senders and for all stages. The "cannot be derived" refusal uses `REASON_REVOKED`, is neither counted nor notified, and has no test (c3 only removes the list check).
  - **Sequence:** if `wire::verify` ever accepts a signer other than the roster's key for that `from`, a message signed by the revoked old key is checked against the unrevoked roster key and stored.
  - **Fix:** use `key_fingerprint(&_signer)` in both paths, give the underivable case its own reason, and add a test and mutant for it.

- **N3 — MINOR** — `termlink-sidecar/src/revocation.rs: refuse_stored`.
  - **What:** "open" means any claim without an ABSENT line whose reason starts `attempt=N`, and a `claims_for` error also counts as open. The test writes that reason by hand, not through the real deliverer.
  - **Sequence:** the deliverer crashes after `claim("m1", 1, …)` and never writes ABSENT (or writes another reason format). The message from the revoked key is then skipped by every sweep, including `refuse_stored_now`, so it stays STORED and the sender gets no REVOKED notice. An attempt open at install time is also finalized with no visible list check.
  - **Fix:** expire or close orphaned claims, make the HANDED_OVER append conditional on a revocation check under the store lock, and test through the real deliver path.

- **N4 — MINOR** — `termlink-sidecar/src/policy.rs: assess`.
  - **What:** `self.store.as_deref().is_some_and(narrowing)` turns the gate off when the policy has no store, while `narrowing()` itself defaults to true. The call in `assess` is equivalent today, so no test or mutant binds it. No downgrade reason is recorded, and the E-41.3 59:59/60:01 case is not run.
  - **Sequence:** a `BuiltIn` policy built without a store, plus a later widening of the urgent allow-list, lets an admitted sender stay urgent while the list is STALE.
  - **Fix:** treat a missing store as narrowing, record the reason `revocation list stale`, and test `assess` with a narrowing store.

- **N5 — MINOR** — `termlink-hub/src/revocation.rs: revoked_keys`.
  - **What:** while a reconcile is pending, every carded post re-reads and re-parses the whole admin log under `LOCK`, which is the OC 9 cost again in a state that lasts until an operator acts. The cache stamp is (len, mtime) with no content check.
  - **Sequence:** restore, then a busy hub before reissue: each post costs a full log read. Separately, a same-length rewrite within the filesystem's mtime granularity serves the old key set.
  - **Fix:** cache the committed keys by the log's stamp and add the list digest to the cache key.

- **N6 — MINOR** — `termlink-sidecar/src/revocation.rs: pull`, `(None, _)` arm.
  - **What:** a hub that already has a stored pin and then stops offering `signing_public_key`, or offers unparsable hex, is classed as "predates" with tamper = false.
  - **Sequence:** an interposed or downgraded hub strips the key from `hub.version`. The sidecar raises only the daily `revocation-hub-predates` entry and no `revocation-tamper`; the stale rule still narrows after an hour.
  - **Fix:** when `pin_stored` is true and the key is missing or malformed, return tamper = true.
