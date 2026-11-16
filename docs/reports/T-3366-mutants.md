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
