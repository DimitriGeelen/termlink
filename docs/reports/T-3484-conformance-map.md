# Worker contract v0.3.3 §9 — TermLink hub backend coverage map

Case ids and titles from AEF (msg e05fa392, 2026-10-21; their `tests/conformance/worker-contract/vectors.yaml`,
d33d2dc6e). Test files: S1 `crates/termlink-bus/src/worker_contract/tests.rs`, S2
`crates/termlink-bus/src/worker_contract/sweep_tests.rs`, RPC layer `crates/termlink-hub/src/dispatch_contract.rs`.

Status: **covered** = a test asserts the case's outcome; **partial** = the behaviour is tested but the case's exact
given/steps have not been checked against the vector text yet; **open** = not built (slice named).

| Case | Title | Status | Test(s) |
|---|---|---|---|
| CC-01 | Completion racing session end | partial | `done_evidence_inside_grace_prevents_lost` |
| CC-02 | Same race, opposite order | partial | `reappearing_binding_after_lost_is_noticed_not_resurrected_and_t5_still_upgrades` (late T5 upgrades `lost`) |
| CC-03 | Supervisor crash before commit | partial | `absent_twice_then_lost_only_after_grace`, `one_absence_is_never_enough_for_lost` |
| CC-04 | Supervisor crash after commit | covered | `cc04_supervisor_crash_after_commit_is_idempotent` |
| CC-05 | Duplicate finalisation | covered | `cc05_duplicate_finalisation` |
| CC-06 | Retry while the old worker survives | covered | `cc06_retry_while_old_worker_survives` |
| CC-07 | Stale-attempt write rejected | covered | `cc07_stale_attempt_write_rejected`, `stale_attempt_on_running_path` |
| CC-08 | Hub restart | covered | `hub_restart_between_sweeps_never_produces_lost`, `restart_keeps_state_and_a_live_binding_changes_nothing` |
| CC-09 | Partition | partial | `unreadable_listing_never_produces_lost` (the hub cannot see → never `lost`) |
| CC-10 | Parent-session loss | open | S3 (T-3503) |
| CC-11 | Answer delivered and acked | open | S3 (T-3503) |
| CC-12 | Answer TTL expiry bounces | open | S3 (T-3503) |
| CC-13 | Local backend parity | n/a here | AEF runs it against both backends; we expose a test hub (S4) |
| CC-14 | Stale supervisor generation | covered | `cc14_stale_supervisor_generation` |
| CC-15 | Unknown resolution by operator | open | S3 (T-3503, `dispatch.resolve` / T7d) |
| CC-16 | Outcome derivation | covered | `cc16_outcome_derivation_and_cc19_paused` |
| CC-17 | Absence across a registry epoch | covered | `hub_restart_between_sweeps_never_produces_lost`, `epoch_is_bumped_at_hub_start_and_persists` |
| CC-18 | Caller-chosen dispatch id | covered | `cc18_caller_chosen_dispatch_id`, `id_taken_and_not_found_have_their_own_codes` |
| CC-19 | Paused is terminal, not success | covered | `cc16_outcome_derivation_and_cc19_paused` |
| CC-20 | Lost attempt seen alive | covered | `reappearing_binding_after_lost_is_noticed_not_resurrected_and_t5_still_upgrades` |
| CC-21 | Dispatched never seen | covered | `dispatched_never_seen_goes_unknown_at_its_deadline_and_no_further` |
| CC-22 | Attested done is not finished work | open | not yet checked; needs the vector text |
| CC-23 | The nonce alone cannot complete | covered | `nonce_alone_cannot_complete` |
| CC-24 | Late T5 whose evidence postdates the lost entry | covered | `cc24_late_t5_postdating_lost_is_forbidden` |
| CC-25 | Backend-initiated stop | open | not yet checked; needs the vector text |

Totals: 15 covered, 4 partial, 6 open or n/a (CC-10/11/12/15 wait for S3; CC-22/25 need the vector text; CC-13 is AEF's).

Next: when the vector file is on the mirror, (1) read each case's given/steps/then; (2) turn the four partials into
covered or open; (3) add `cc<NN>_` aliases or rename so each test names its case; (4) send AEF the patch with the
hub-side `applicability` and `backend_notes` for every case we cover.
