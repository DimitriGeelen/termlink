**1. Verdict: FAIL.** Every round-1 BLOCKER and MAJOR is closed as its disposition says, but the fix for finding 2 introduces one new MAJOR (N1): a full table now answers a clean empty list for an unrecorded inbox.

I judged only from the bundle. The corrected design text for 3.1 and 4.1 and the runner's "rebuild first" change are not in it, so I could not check them.

**2. Accepted findings**

- **1 (self-spoof)** — CLOSED — `record` matches on agent, instance, strength and `key_fp`; `parse_reader` refuses an empty or `-` instance; the sidecar's `is_me` needs `proven` and its own `key_fp`. Covered by `a_claimed_reader_never_merges_into_a_proven_entry`, `the_owners_name_without_its_key_is_not_me`, the L4 owner-name test, and m18/m19.
- **2 (unbounded recording)** — CLOSED as stated — `consumer_gate` and the receipt path skip topics that do not exist (m25, L4 ghost-topic test); long topics are not recorded or bound; eviction rows go through `capped_row`; in-window inboxes are not evicted (m26). The cap has no test, and the fix opens N1.
- **3 (liveness fallback)** — CLOSED — a transport error and a `channel.first_at`-named error are `Unreadable` with no subscribe call (`read_falls_back_only_for_an_old_hub`, m21). The match is looser than the disposition says (N3).
- **4 (corrupt table)** — CLOSED — `load_state` renames the file to `.corrupt-<ms>`, queues the admin row and sets `incomplete_since_ms`; `consumers` errors while the window reaches before it (`a_corrupt_table_is_moved_aside_and_reported_incomplete`, m23). `load_error` now persists only when the rename fails.
- **5 (claim path)** — CLOSED — `e66_claims_and_acks_through_the_hub_before_and_after_a_bind` shows the bound agent's signed claim passing and the other agent's signed claim and an unsigned claim refused with INBOX_BOUND (m16). The design correction is not visible.
- **CC 2 (stale status)** — CLOSED — `checked_ms` is written, `status(store, now_ms)` returns `unknown (no check since …)` past `STALE_AFTER_MS`, and `reset` runs in the receiver start (test asserts, m24).
- **CC 5 (tests)** — CLOSED — the listed tests and m16–m19 exist and are killed. Gaps: nothing tests `capped_row` or the `suppressed` count, an unsigned claim before a bind is not exercised, and the rebuild-first change is not shown.
- **CC 6 (long-poll)** — CLOSED — `handle_channel_subscribe` calls `check_bound` after a waited poll and returns the refusal (`a_long_poll_opened_before_a_bind_does_not_deliver_after_it`, m20).
- **CC 7 (ack replay)** — CLOSED — `assess_ack` returns `claimed (stale ack)` outside ±120 s; the L4 test posts the owner's receipt at `now − 300 s` and gets INBOX_BOUND (m22).
- **CC 12 (overflow)** — CLOSED — `abs_diff` in `assess` and `assess_ack`; `extreme_timestamp_does_not_overflow`.
- **CC 13 (partly)** — CLOSED — per-kind window with a carried `suppressed` count in `capped_row`, by code only; there is no test.
- **CC 14 (partly)** — CLOSED — `bind` refuses a revoked card key and fails closed when the list or hub identity is unavailable (`bind_refuses_a_revoked_key_and_a_long_topic`). The design 3.1 text is not visible.
- **OC 6 (uncarded ack name)** — CLOSED — it is listed under a valid `from_agent`, else `(unnamed)`, with the key prefix in `why` (`ack_reader_from_cards`).

**3. Rejected and deferred findings**

- **CC 8 (D)** — SOUND — it stays inside ruling E's 120 s window and the stated risk matches the code, provided reader objects are only capturable on-host as claimed.
- **CC 11 (D perf / R fail-closed)** — SOUND — a global lock is correct if slow, and with unreadable bindings no read can be judged, so refusing is the only safe answer (m17 holds it).
- **CC 13 per-topic caps (D)** — SOUND — callers pick topics, so a per-topic cap bounds nothing in total.
- **CC 14 revoked ack stays claimed / refusal counter (D)** — SOUND — a revoked ack is refused after a bind via `passes`, and the counter is status-only.
- **CC 15 (D)** — SOUND — it is within ruling D and escalated to the operator rather than decided by the agent.

**4. New defects**

- **N1 — MAJOR** — `inbox_consumers.rs::record` table-full branch with `consumers()` — When the table is full of in-window topics, a new inbox is never recorded, yet `inbox.consumers` for it answers `Ok` with an empty list. The second-consumer check is silently blind for that inbox, against FCW-125; the only signal is a capped admin row. The unit test asserts this empty list as correct.
  - **Sequence:** create 4096 `inbox:<hub>/jN` topics with `channel.create` (the L4 tests do this over bare rpc), read each once, and repeat within 24 h. Then read a new agent's inbox unsigned. Neither that read nor the owner's own reads are listed. This assumes topic creation is open to the same callers as reads.
  - **Fix:** keep a `full_since_ms` marker and answer an error for a topic absent from the table while the window reaches a full-table drop. Optionally reserve slots for topics with a proven reader or a binding.
- **N2 — MINOR** — `channel.rs::consumer_gate` and the receipt gate — A topic that does not exist returns `Ok(None)` and the call proceeds ungated, so there is a check-then-use gap on a bound inbox whose topic is not yet created. A long-poll that passes the gate just before creation is never re-checked because `reader` is `None`.
  - **Sequence:** bind `inbox:h/fx-a` before the topic exists, loop `channel.subscribe` with `timeout_ms`, and one call straddles `channel.create`. If `channel.post` auto-creates topics, the first receipt on such an inbox also skips the bind.
  - **Fix:** return the unknown-topic error directly from the gate, or still enforce an existing binding when the topic is missing.
- **N3 — MINOR** — `liveness.rs::read` — The fallback fires on any coded error whose message lacks `channel.first_at`, not only on -32001 as the disposition and design 7.5 say.
  - **Sequence:** a T-3700 hub answers `first_at` with a generic coded error (bus unavailable, auth, rate limit). The sidecar does a content read of the main's inbox, which is recorded as a second consumer or refused after a bind.
  - **Fix:** match `code == -32001` and the "Missing 'target'" text, and add a test for another coded error.
- **N4 — MINOR** — `load_state` / `pending_rows` — The incomplete marker and the reset row live only in memory until the first dirty `record`. The row is dropped if `log` fails, and only `record` flushes it.
  - **Sequence:** the corrupt file is loaded by an `inbox.consumers` call and the hub restarts before any recorded read. The next load sees no file, so the list is clean with no marker and no admin row.
  - **Fix:** save the empty table with `incomplete_since_ms` right after the rename, and keep the row queued until `log` succeeds.
- **N5 — MINOR** — `capped_row` — The `suppressed` count is zeroed before `log` and lost if that write fails, and the function has no test or mutant.
  - **Sequence:** 30 refusals in a minute, then the next row's write fails; the count of 20 is gone.
  - **Fix:** clear the count only on a successful write; add a unit test and a mutant.
- **N6 — MINOR** — `assess_ack` `why` — The key prefix is now in `why`, and `record` marks the entry dirty when `why` changes.
  - **Sequence:** two uncarded keys alternate receipts naming the same agent. Each post rewrites the whole table, which is the per-junk-call rewrite of finding 2 on existing topics.
  - **Fix:** leave `why` out of the dirty test, or persist `why` changes on the 60 s cadence.
- **N7 — MINOR (to verify)** — `handle_channel_subscribe` — The re-check runs only when `timeout_ms` parses as a u64 above zero.
  - **Sequence:** if the inner handler has a non-zero default or accepts a float, a read sent without `timeout_ms` or with `4000.0` waits across a bind and still delivers.
  - **Fix:** always re-check when a reader was gated, or share the handler's parsed timeout.
- **N8 — MINOR (to verify)** — stale-ack rule — The rule also catches honest late receipts.
  - **Sequence:** if the sidecar retries a receipt signed more than 120 s earlier (hub down for 3 minutes), a bound inbox refuses it on every retry.
  - **Fix:** confirm the sidecar re-signs with a fresh `ts` on retry, and add a test.
