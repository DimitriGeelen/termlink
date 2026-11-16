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
| E-166 hub identity and pin at start | PASS | A mint that cannot be written atomically refuses and leaves no partial `hub.id` and no temp; the journal (marker, and the key once published) stays, and the next start finishes the mint (`e166_fcw81_*`, `e166_failure_before_rename_*` and `e166_failure_at_the_key_write_*` with a fault injector before the publish; m6 killed); the publish never clobbers (`e166_write_never_clobbers_*`, m16); 8 concurrent first starts converge on one id and key (`e166_concurrent_first_starts_converge_*`); a stale temp is removed only when its writer is dead. FCW-84: a first contact off the local socket pins nothing (`e166_fcw84_*`, m10 killed). A sidecar whose hub is down neither posts nor pulls until the id verifies: `fc_matrix::fc_8_hub_down_shows_waiting_then_unknown_then_goes_on_with_no_loss`, `send_hub::send_hub_hub_down_waits_then_flows_on_without_loss_or_duplicate` (both green in the full sidecar suite). |

Round 3 (after review round 2, `T-3366-reviews/r2-dispositions.md`), all on commit 0043aff40, which runme deployed:
[superseded by the final state below]

**Final state (after the confirmation round, `T-3366-reviews/r4-dispositions.md`):** deployed commit fe72c850c, whose
T-3366 sources equal 2b3759e02's (`git diff --stat 2b3759e02 fe72c850c` over the unit's Rust paths is empty); mutants
26 of 26 killed on 2b3759e02 (m24 stale-marker re-key, m25 admin-row backfill, m26 mode from umask added in round 3);
runme fixtures 392/0 on fe72c850c; unit `hub_identity` 27 passed (incl. the malformed-key variants fixed after the
confirmation round); L4 hub 3 passed; sidecar `t3366_hub_key_pin` 9 passed; full suites green. Two unit tests have no
mutant of their own (`e36_key_without_id_and_no_journal_is_refused`, `e166_concurrent_first_starts_converge_on_one_id`):
their evidence is the green result line and the code.

Round 3 detail: Added in round 3: the two-file mint is journalled (`.hub-identity.pending`; tests
`e166_interrupted_*`, `e36_key_without_id_and_no_journal_is_refused`, m20); the sidecar pin is published no-clobber
after its record (`e37_concurrent_first_contacts_*`, `e37_no_record_means_no_pin`, m21, m22) and writes the OBS-37
admin row `hub-key-pinned` (asserted in the real-hub test, m23).
