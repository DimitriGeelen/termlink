VERDICT: PASS

## Round-1 findings marked A

- F1 self-spoof (CC 1 / OC 1 / OC 4) — CLOSED — `record` now keys entries by (agent, instance, strength, key_fp) so a claimed reader never merges, `parse_reader` rejects empty/`-` instances, and sidecar `verdict`'s `is_me` requires `proven` + own `key_fp`; proven by unit tests `a_claimed_reader_never_merges_into_a_proven_entry`, `the_owners_name_without_its_key_is_not_me`, L4 `e66_an_unsigned_reader_using_the_owners_name_and_instance_is_flagged`, and mutants m18/m19 killed.
- F2 junk topics / topic length / row caps / rewrite-per-junk (CC 3 / OC 2) — CLOSED — `consumer_gate` and the ack gate require `topic_exists`, `MAX_TOPIC_LEN=256` enforced in both `bind` and `record`, `capped_row` gives 10/min per kind with a carried `suppressed` count, and the full-table path never evicts an in-window reader (`a_full_table_never_evicts_an_inbox_read_inside_the_window`, `a_long_topic_is_not_recorded`, `a_read_of_a_topic_that_does_not_exist_records_nothing`; m25/m26 killed).
- F3 liveness fallback (CC 9 / OC 3) — CLOSED — fallback only on `Err((Some(_), m)) if !m.contains("channel.first_at")`, transport errors Unreadable; `read_falls_back_only_for_an_old_hub` covers -32001 / no-code / -32603, mutant m21 killed.
- F4 corrupt table (CC 4 / OC 5) — CLOSED — `load_state` renames the file to `.corrupt-<ms>`, persists `incomplete_since_ms`, `consumers()` errors until the window passes the reset (correct: any lost entry is then out of window), and `load_error` is cleared by the move; `a_corrupt_table_is_moved_aside_and_reported_incomplete`, m23 killed.
- F5 claim path / design 4.1 (CC 10 / OC 7) — CLOSED — `handle_channel_claim` runs the gate, L4 `e66_claims_and_acks_through_the_hub_before_and_after_a_bind` exercises signed/unsigned claims and receipts before and after a bind, mutant m16 killed, design corrected.
- CC 2 — CLOSED — `checked_ms` persisted, `status` shows `unknown (no check since …)` past `STALE_AFTER_MS`, `reset` called at receiver start in `pull.rs`; m24 killed.
- CC 5 — CLOSED — every promised test exists and passes (measured: 9 integration tests ok, mutants m16–m26 all killed, rebuild-after-mutants remediated and disclosed).
- CC 6 — CLOSED — subscribe re-checks the binding after a long-poll returns via `check_bound` and discards the response on refusal; `a_long_poll_opened_before_a_bind_does_not_deliver_after_it`, m20 killed.
- CC 7 — CLOSED — `assess_ack` proves only when `abs_diff(post_ts, now) ≤ 120 s`, else `claimed (stale ack)`; unit `ack_reader_from_cards` and the L4 replay of a 300 s-old receipt (INBOX_BOUND), m22 killed.
- CC 12 — CLOSED — `abs_diff` in `assess`/`assess_ack`; `extreme_timestamp_does_not_overflow` with `i64::MIN`.
- CC 13 (accepted part) — CLOSED — per-kind `capped_row` with suppressed count carried to the next written row; a failed row no longer weakens the refusal or blocks eviction.
- CC 14 (accepted part) — CLOSED — design 3.1 corrected and `bind` refuses a revoked key (`bind_refuses_a_revoked_key_and_a_long_topic`; router passes the revocation closure).
- OC 6 — CLOSED — uncarded acks list under a validated payload `from_agent` else `(unnamed)`, key prefix moved into `why`; tested in `ack_reader_from_cards`.

## Round-1 findings marked R or D

- CC 8 (D) — SOUND — ruling E fixes the 120 s proof window and the residual risk (root re-reading a captured page) grants nothing root lacks on disk.
- CC 11 (D perf / R fail-closed) — SOUND — a global lock only strengthens the invariants, and refusing all reads on unreadable bindings is the only safe judgment under unknown binds (FCW-124); the bindings-per-read cost is deferred with a stated measurement plan.
- CC 13 per-topic caps (D) — SOUND — callers choose topic names, so per-topic caps are unbounded in aggregate while the per-kind global cap bounds the log.
- CC 14 refusal counter (D) — SOUND — counters are status P-24's; the security-relevant halves (error codes, bind-vs-revocation) were fixed here.
- CC 15 (D) — SOUND — ack metadata is content-free per ruling D and the question is escalated to the operator rather than decided unilaterally.

## New defects introduced by the changes

1. MINOR — channel.rs `handle_channel_subscribe` — the recheck fires only when `timeout_ms` parses via `as_u64() > 0`, independently of whatever parsing the inner handler uses to wait, so any divergence reopens CC 6 — input: a reader subscribes with `timeout_ms: 4000.5` (or `"4000"`) before a bind; if the inner handler honors it as a wait, a post landing after the bind is delivered with no recheck — fix: parse the timeout once and share the value, or recheck whenever `reader` is `Some`.
2. MINOR — send.rs `key_fp` — `unwrap_or_default()` yields `""` on a fingerprint error, so `verdict`'s exact-match `is_me` never matches and the sidecar permanently reports its own proven entry as a second copy ("two identities") — fix: surface a fingerprint failure as an `unknown` consumers line (or a startup error) instead of comparing against `""`.
3. MINOR — inbox_consumers.rs `load_state` — the `inbox-consumers-reset` admin row waits in `pending_rows` for the next `record`, so a corrupt table moved aside with no subsequent inbox read is never reported to the operator — fix: best-effort write the row at move time in addition to queueing it.
