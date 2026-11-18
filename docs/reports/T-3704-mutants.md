| FC | Mutant | What it breaks | Test that must go red | Result |
|---|---|---|---|---|
| FCW-133 / E-69.5 | `t3704-m1-fc-01-verify-accepts-any-signature` | the card key is not compared with the signer (plan control fc-01) | `key_card::tests::e69_fcw133_signed_by_a_sibling_key_is_refused_own_key_passes` | killed |
| FCW-128 | `t3704-m10-put-not-admin` | card writes are not admin RPCs (no intent row first) | `router::tests::t3704_key_card_writes_are_refused_off_the_local_socket` | killed |
| FCW-98 / E-69.4 | `t3704-m11-receiver-ignores-bound-name` | the receiver does not check a bound from_agent against the key | `wire::wire_tests::t3704_named_roster_binds_from_agent_to_its_key` | killed |
| FCW-98 | `t3704-m12-open-path-skips-names` | an unrostered key may claim a bound agent name | `wire::wire_tests::t3704_named_roster_binds_from_agent_to_its_key` | killed |
| design v2 C-12 | `t3704-m13-no-refusal-counts` | refusals are not counted for the impersonated agent | `e69_posts_are_checked_against_the_card_and_e169_no_card_is_refused` | killed |
| AL-33 step 2 (admin_log) | `t3704-m14-refusals-never-logged` | no admin row is ever written for a refusal | `key_card::tests::refusal_rows_are_coalesced_per_agent_key_minute` | killed |
| design v2 C-12 (sidecar) | `t3704-m15-impersonation-silent` | the impersonated agent's sidecar never raises the attention entry | `a_refused_post_claiming_to_be_me_raises_one_attention_entry` | killed |
| AL-33 2.17.f(2) | `t3704-m16-no-startup-reconcile` | a card ahead of the admin log is never reported at start | `key_card::tests::al33_startup_reconcile_flags_a_card_the_admin_log_does_not_show` | killed |
| r2 MAJOR 2 | `t3704-m17-outside-edit-accepted` | a card store edited outside hub.key_card_put is used | `key_card::tests::r2_an_outside_edit_with_the_same_seq_is_refused_until_restart` | killed |
| r2 BLOCKER (operator ruling A) | `t3704-m18-row-not-first` | the card is written without its admin row | `key_card::tests::r2_the_admin_row_is_written_first_and_a_failed_row_writes_nothing` | killed |
| r2 MAJOR 3 | `t3704-m19-encryption-key-silent` | the encryption key changes without replace | `key_card::tests::r2_every_field_change_needs_replace_and_a_key_has_one_card` | killed |
| FCW-135 / E-169 | `t3704-m2-no-card-passes` | a poster without a card passes | `key_card::tests::e169_fcw135_a_poster_without_a_card_is_refused` | killed |
| r2 MAJOR 2/5 (O-4/C-4) | `t3704-m20-receiver-admits-other-type` | the receiver handles an unlisted type as mail | `an_unlisted_type_on_the_inbox_is_skipped_never_handled_as_mail` | killed |
| r2 MINOR 10 | `t3704-m21-mode-not-reconciled` | a setting changed outside the RPC is not reported at start | `key_card::tests::r2_mode_changes_are_logged_first_and_reconciled_at_start` | killed |
| design v2 C-M1 | `t3704-m3-no-name-check` | from_agent is used without the plain-name check | `key_card::tests::a_traversal_name_is_refused_and_never_read_as_a_path` | killed |
| design v2 O-M3 | `t3704-m4-corrupt-store-as-empty` | a corrupt card store reads as no cards | `key_card::tests::an_unreadable_store_refuses_never_reads_as_empty` | killed |
| design v2 C-3 | `t3704-m5-replace-not-required` | a card key or project changes without replace | `key_card::tests::a_key_or_project_change_needs_replace_and_seq_rises` | killed |
| design v2 (fail closed) | `t3704-m6-bad-setting-is-off` | an unreadable or invalid setting reads as off | `key_card::tests::mode_defaults_off_and_a_bad_setting_enforces` | killed |
| FCW-133 at the hub | `t3704-m7-enforce-stores` | in enforce mode a refused post is stored anyway | `e69_posts_are_checked_against_the_card_and_e169_no_card_is_refused` | killed |
| NID | `t3704-m8-gate-every-type` | the check also gates other message types and topics | `key_card::tests::other_types_and_topics_are_never_gated` | killed |
| operator ruling A (local socket only) | `t3704-m9-put-over-tcp` | a card write is accepted from a TCP peer | `router::tests::t3704_key_card_writes_are_refused_off_the_local_socket` | killed |

PASS: all 21 mutants turned their test red. Run 2026-11-18T02:11Z on 36af5af05.
