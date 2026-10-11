# arc-011 PH-1 fail-closed matrix (FC-1..FC-17)

Task T-3413 (B12, X2). Each row of section 9.1 of `docs/design/arc-011-ph1-architecture.md`
has a test that proves the rule holds and a **negative control**: a one-line-class patch
(`crates/termlink-sidecar/tests/mutants/*.patch`) that breaks the rule. A control only
counts once it has been seen to turn its test red (R-69); `scripts/arc011-mutants.sh`
applies each patch in a scratch copy of the tree, rebuilds, runs the named test and
reverts. The run's table is at the end.

## The rules and their tests

| FC | Rule | Test (file :: name) | Negative control (patch) |
|---|---|---|---|
| FC-1 | A forged, wrong-key, altered or misaddressed message never reaches the store | hub half: `fc_matrix :: fc_1_hub_refuses_a_post_whose_signature_does_not_verify`; receiver half: `receive_hub :: receive_hub_forged_or_misaddressed_messages_are_refused_never_stored` | `fc-01-verify-accepts-any-signature` |
| FC-2 | A failed write (ENOSPC on a 256 KiB tmpfs inside the estate) never reports STORED; the message waits, the temp file is not leaked, the sender sees STUCK | `fc_matrix :: fc_2_a_full_disk_never_reports_stored_and_the_message_waits`; also `receive_hub :: receive_hub_store_failure_at_every_step_reports_nothing_then_restart_stores_once` | `fc-02-stored-reported-on-write-error` |
| FC-3 | A kill at any step of the write sequence leaves the message absent-and-unreported or present-and-reported once | `store_tests :: store_kill_at_every_step_never_reported_and_missing` | `fc-03-stored-line-before-the-record` |
| FC-4 | An open claim blocks a second frame, also across a restart | `deliver_tests :: deliver_an_open_claim_survives_a_restart_and_blocks_a_second_frame`; stress: `deliver_concurrent_pulls_never_hand_out_a_message_twice` | `fc-04-no-claim-file` |
| FC-5 | A missing, empty, unreadable, stale or future ready flag reads not ready | `hooks_tests :: hooks_ready_flag_reads_not_ready_unless_present_parseable_and_fresh` | `fc-05-missing-flag-reads-ready` |
| FC-6 | Nothing but harness evidence makes HANDED_OVER; a live, ready session is not evidence | `deliver_tests :: deliver_no_completeness_point_never_redelivers_on_a_timeout` | `fc-06-ready-flag-taken-as-evidence` |
| FC-7 | A stored message is WAITING FOR RECIPIENT, then STUCK; never delivered | `hooks_tests :: hooks_sender_sees_waiting_for_recipient_then_stuck_never_delivered` | `fc-07-storage-counted-as-delivery` |
| FC-8 | Hub down: WAITING; hub back, nothing heard: UNKNOWN; then delivered, nothing lost | `fc_matrix :: fc_8_hub_down_shows_waiting_then_unknown_then_goes_on_with_no_loss` | `fc-08-drop-on-hub-error` |
| FC-9 | A second sidecar on the same store is refused | `receive_hub :: receive_hub_second_sidecar_on_the_same_store_is_refused` | `fc-09-no-store-lock` |
| FC-10 | Same (sender, conv, msg_id) with another body: CONFLICT, first record kept | `receive_hub :: receive_hub_same_id_with_another_body_is_a_conflict_and_never_stored` | `fc-10-no-digest-comparison` |
| FC-11 | The nonce in an assistant, user, tool_result or hook_success record is not evidence | `deliver_tests :: deliver_handed_over_only_on_a_harness_record_never_on_echoes` | `fc-11-any-record-is-evidence` |
| FC-12 | Hostile bodies (forged header, boundary copies, approval text, control characters, fuzz) stay inside the boundary | `deliver_tests :: frame_hostile_bodies_stay_inside_the_boundary` | `fc-12-frame-without-boundary` |
| FC-13 | An unreadable transcript is UNCERTAIN and keeps the attempt open | `deliver_tests :: deliver_unreadable_transcript_is_uncertain_and_keeps_the_attempt_open` | `fc-13-unreadable-read-as-absent` |
| FC-14 | A hub with another id is not read (receive) and not posted to (send) | `receive_hub :: receive_hub_wrong_hub_id_reads_nothing_and_says_so`; send half in `send_hub` | `fc-14-no-hub-id-check` |
| FC-15 | An edited, inserted or deleted chain line is named by `status` | `store_tests :: store_chain_break_detected_for_edit_insert_delete` | `fc-15-no-chain-check` |
| FC-16 | Before HANDED_OVER the sender sees only accepted/stored words | `record_print :: record_print_every_pre_handover_state_uses_only_accepted_vocabulary` | `fc-16-acceptance-called-delivered` |
| FC-17 | The secret canary in the environment appears in no file, log or answer of a real two-sidecar run | `fc_matrix :: fc_17_the_canary_appears_in_nothing_the_estate_produced` | `fc-17-environment-logged` |

## Found while building the matrix

FC-2 was the first test to put a store on a full disk, and it found two defects, both
fixed in T-3413:

1. **A failed write leaked its temp file.** `Store::put` left the partial `tmp/` file
   behind on an I/O error, and every retry added another, so a full disk stayed full
   until a restart swept `tmp/`. A real I/O error now removes it; a failpoint (which
   stands for a crash, where no cleanup runs) still leaves it for recovery.
2. **The stall was silent.** The receiver's needs-attention entry goes into the same
   store, so on a full disk it could not be written. Entries that cannot be written now
   go to the sidecar's stderr (its log), and the sender's view shows a message that was
   received but never stored as STUCK past the deadline.

The run also measured the throwaway hub taking over 30 s to answer `hub.version` at
load average 44; the test kit now waits up to 90 s for it.

## Mutation run

| FC | Mutant | What it breaks | Test that must go red | Result |
|---|---|---|---|---|
| FC-1 | `fc-01-verify-accepts-any-signature` | wire::verify returns the message without checking the signature: a forged message is stored | `receive_hub_forged_or_misaddressed_messages_are_refused_never_stored` | killed |
| FC-2 | `fc-02-stored-reported-on-write-error` | a store error is reported as STORED | `fc_2_a_full_disk_never_reports_stored_and_the_message_waits` | killed |
| FC-3 | `fc-03-stored-line-before-the-record` | the STORED line is written before the record is durable: a kill leaves a STORED line with no record | `store_kill_at_every_step_never_reported_and_missing` | killed |
| FC-4 | `fc-04-no-claim-file` | pull skips the O_EXCL claim: a restarted sidecar frames the message again | `deliver_an_open_claim_survives_a_restart_and_blocks_a_second_frame` | killed |
| FC-5 | `fc-05-missing-flag-reads-ready` | a missing or unreadable ready flag reads ready | `hooks_ready_flag_reads_not_ready_unless_present_parseable_and_fresh` | killed |
| FC-6 | `fc-06-ready-flag-taken-as-evidence` | a ready session is taken as evidence of hand-over | `deliver_no_completeness_point_never_redelivers_on_a_timeout` | killed |
| FC-7 | `fc-07-storage-counted-as-delivery` | a stored message is shown to the sender as delivered | `hooks_sender_sees_waiting_for_recipient_then_stuck_never_delivered` | killed |
| FC-8 | `fc-08-drop-on-hub-error` | the outbox drops the message when the hub is unreachable | `fc_8_hub_down_shows_waiting_then_unknown_then_goes_on_with_no_loss` | killed |
| FC-9 | `fc-09-no-store-lock` | the store lock is not enforced: a second sidecar opens the same store | `receive_hub_second_sidecar_on_the_same_store_is_refused` | killed |
| FC-10 | `fc-10-no-digest-comparison` | the same key with another body counts as a duplicate | `receive_hub_same_id_with_another_body_is_a_conflict_and_never_stored` | killed |
| FC-11 | `fc-11-any-record-is-evidence` | the nonce in any transcript record counts as evidence | `deliver_handed_over_only_on_a_harness_record_never_on_echoes` | killed |
| FC-12 | `fc-12-frame-without-boundary` | the frame is built without the random boundary | `frame_hostile_bodies_stay_inside_the_boundary` | killed |
| FC-13 | `fc-13-unreadable-read-as-absent` | an unreadable transcript line is skipped (read as absent) | `deliver_unreadable_transcript_is_uncertain_and_keeps_the_attempt_open` | killed |
| FC-14 | `fc-14-no-hub-id-check` | the receiver does not compare the hub id | `receive_hub_wrong_hub_id_reads_nothing_and_says_so` | killed |
| FC-15 | `fc-15-no-chain-check` | status does not check the chain links | `store_chain_break_detected_for_edit_insert_delete` | killed |
| FC-16 | `fc-16-acceptance-called-delivered` | a post the hub accepted is shown as delivered | `record_print_every_pre_handover_state_uses_only_accepted_vocabulary` | killed |
| FC-17 | `fc-17-environment-logged` | the sidecar logs its environment at start | `fc_17_the_canary_appears_in_nothing_the_estate_produced` | killed |

PASS: all 17 mutants turned their test red. Run 2026-10-11T23:27Z on 0facf8aac.

The run used the B12 working tree (these patches, the FC-2 fixes and the new tests) on top of commit 0facf8aac; reproduce with `bash scripts/arc011-mutants.sh`.
