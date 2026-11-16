| FC | Mutant | What it breaks | Test that must go red | Result |
|---|---|---|---|---|
| AL-17 inv.3 | `t3366-m1-derive-id-per-call` | hub.version derives hub_id from the certificate on each call (ARC4 6.1.7) | `e36_e37_restart_and_certificate_rotation_keep_id_and_signing_key` | killed |
| FCW-84 | `t3366-m10-pin-untrusted` | a first contact off the local socket pins the key | `e166_fcw84_untrusted_first_contact_pins_nothing_and_no_offer_changes_nothing` | killed |
| FCW-83 | `t3366-m11-gate-ignores-pin` | verified_hub_key accepts any offered key | `e37_first_contact_pins_restart_keeps_and_a_swapped_key_is_refused_and_named` | killed |
| FCW-83 / OBS-36 | `t3366-m12-mismatch-silent` | a changed hub key raises no needs-attention entry | `e37_first_contact_pins_restart_keeps_and_a_swapped_key_is_refused_and_named` | killed |
| FCW-79 | `t3366-m2-no-fcw79` | a missing hub.id beside prior state is minted instead of refused | `hub_identity::tests::e36_fcw79_missing_id_beside_state_refuses_and_writes_nothing` | killed |
| FCW-80 | `t3366-m3-lenient-id` | any 16-character hub.id is accepted | `hub_identity::tests::e36_fcw80_malformed_id_refuses` | killed |
| FCW-82 | `t3366-m4-no-key-mode-check` | a signing key file with any mode is accepted | `hub_identity::tests::e37_fcw82_bad_key_file_refuses` | killed |
| AL-18 | `t3366-m5-key-regenerated` | a missing signing key beside an existing id is regenerated silently | `hub_identity::tests::e37_key_missing_beside_id_refuses` | killed |
| FCW-81 | `t3366-m6-write-in-place` | hub.id is written in place, not via temp + rename | `hub_identity::tests::e166_failure_before_rename_leaves_no_id_and_no_temp` | killed |
| AL-17 adopt | `t3366-m7-adopt-overwrites` | adopt-id replaces an id already on disk | `hub_identity::tests::adopt_is_idempotent_and_never_changes_an_existing_id` | killed |
| SI-25 | `t3366-m8-debug-prints-key` | the debug form prints the private seed | `hub_identity::tests::e37_private_key_is_never_printed` | killed |
| FCW-83 / E-37 control | `t3366-m9-silent-repin` | a changed hub key is re-pinned silently | `e37_first_contact_pins_restart_keeps_and_a_swapped_key_is_refused_and_named` | killed |

PASS: all 12 mutants turned their test red. Run 2026-11-16T08:04Z on 7ddb462c6.
