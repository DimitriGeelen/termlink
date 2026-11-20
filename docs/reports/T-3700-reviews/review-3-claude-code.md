**Verdict: FAIL** — N1 is only partly closed: a read dropped by a full table is still served as a clean list once that inbox later enters the table. Everything else is closed or sound.

## Round-2 findings marked A

- **N1 — NOT CLOSED** — the error is conditioned on `!st.topics.contains_key(topic)`, so it disappears as soon as the inbox is recorded, while the dropped read is still inside the window (see defect 1); the test and m27 cover only the absent-inbox case.
- **N2 — CLOSED** — `consumer_gate` and the receipt branch now proceed when `is_bound(...)` is true, and `is_bound` returns `true` on unreadable bindings; only `is_bound` itself is tested (see defect 4).
- **N3 — CLOSED** — the fallback arm now requires `code == SESSION_NOT_FOUND && m.contains("Missing 'target'")`, and the test adds `(-32603, "bus unavailable")` → `Unreadable`.
- **N4 / OC 3 — CLOSED** — `load_state` calls `save_consumers` right after the rename (failure sets `persist_error`), `pending_rows.retain(.. is_err())` keeps unwritten rows, and the restart test plus m30 cover the marker; the row itself is still lost on a restart (see defect 3).
- **N5 / CC 5 gap (`capped_row`) — CLOSED** — `w.2 = 0` runs only inside `if log(row).is_ok()`, and `capped_rows_keep_the_suppressed_count_until_written` asserts `suppressed == 2` after a failed write; m28 killed.
- **N6 — CLOSED** — the `dirty = true` on a `why` change is gone (`e.why.clone_from(..)` only), so a changed reason no longer forces a rewrite; the eventual 60 s persist is not shown or tested in the bundle.
- **N7 / OC 1 — CLOSED** — the `waited` condition is removed and `check_bound` runs for every read where the gate returned a reader; m20 killed.
- **OC 2 — CLOSED** — `verdict` returns `unknown (this sidecar's own key fingerprint is unavailable)` when `me_fp.is_empty()`, with a unit assertion on an empty-fingerprint own entry.

## Findings marked R or D

- **N8 (R) — SOUND** — the bundle's measured grep shows the sidecar posts no `receipt`, and receipts signed at post time are fresh by construction, so the stale-ack rule refuses no honest sidecar path.
- **CC 5 gap, unsigned claim before a bind (D) — SOUND** — claims go through the same `consumer_gate` whose unsigned-before-bind path is tested (E-66.6), and m16 kills removal of the claim gate, so the residual risk is only a future divergence of the two paths.

## New defects

1. **MAJOR — `inbox_consumers.rs` `consumers()`, the `dropped_since_ms` check — a dropped read is served as a clean list once the inbox is in the table.**
   - Sequence (24 h window, table full): inbox `t0` was last read at n−23h and the rest at n. At n+1s claimed reader X reads `inbox:h/new`, which is skipped and sets `dropped_since_ms`. At n+2h `t0` is out of the window, the owner reads `inbox:h/new`, `t0` is evicted and the inbox is recorded. `inbox.consumers("inbox:h/new")` at n+3h returns a clean list with the owner only.
   - This is the normal aftermath of any table-full event, not a corner case, and it is the same FCW-125 breach as N1.
   - Fix: store a per-topic `recorded_since_ms` and return the error when `dropped_since_ms > since && (topic absent || recorded_since_ms > since)`; add a test and a mutant for this sequence.

2. **MINOR — `record()`, `is_none_or(|t| now_ms >= t + PERSIST_EVERY_MS)` — the marker can be up to 60 s older than the last drop, so the window check under-reports.**
   - Sequence: drops at T and T+59s (the second is not stamped); a query at T+W+30s has `since = T+30s`, so `t > since` is false and the list is clean although the T+59s drop is inside the window.
   - Fix: compare `t + PERSIST_EVERY_MS > since`, or update the in-memory value on every drop and rate-limit only the save.

3. **MINOR — `load_state` / `record()` — the `inbox-consumers-reset` attention row lives only in `pending_rows` and is flushed only by `record()`.**
   - Sequence: corrupt file, hub start, an `inbox.consumers` call triggers the move, hub restarts before any read. The reloaded file is valid, so the row is never written.
   - Related: if the post-rename save fails and the hub restarts, the marker is lost too, because no file reads as a clean state.
   - Fix: write the row at load time, or persist a `reset_row_pending` flag in the file; flush pending rows from `consumers()` and `check_bound` as well.

4. **MINOR — `channel.rs` `consumer_gate` and the receipt branch — the N2 wiring is not tested.**
   - m29 mutates only the `unwrap_or(true)` in `is_bound`, and the new test calls only `is_bound`. Deleting `&& !ic::is_bound(..)` from `consumer_gate`, or `|| is_bound(..)` from the receipt branch, would survive everything shown.
   - Fix: add a handler-level test (non-bound reader subscribes, claims and posts a receipt on a bound, not-yet-created inbox and gets `INBOX_BOUND`) and a mutant for each call site.

5. **MINOR — `termlink-sidecar/src/consumers.rs` `verdict()` — the early return on an empty fingerprint discards `others` and `bound` before reading the answer.**
   - Input: `me_fp = ""` and a hub answer listing agent `fx-evil` yields `others: []`, so no second-consumer attention entry is raised for an entry that is certainly not this sidecar. It also masks "hub predates the consumer list".
   - Fix: parse the answer first, keep entries with a different agent or instance in `others`, and return unknown only for the self-identification part.

6. **MINOR — `handle_channel_subscribe` — the re-check runs only when the gate returned a reader.**
   - Sequence: a read issued while the inbox topic is missing and unbound gets `Ok(None)`. If such a subscribe can wait until the topic is created and bound (not determinable from the bundle), it delivers post-bind messages with no check and no record.
   - Fix: when `ic::applies(topic)` and the reader is `None`, re-run `consumer_gate` after the wait before delivering, or refuse long-polls on missing inbox topics.
