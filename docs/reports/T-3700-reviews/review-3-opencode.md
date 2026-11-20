**PASS**

**A findings**

- N1 — CLOSED — `record` sets and persists `dropped_since_ms` on the full-table skip and `consumers` returns Err("consumer table full since …") for an absent inbox, asserted by the rewritten test and m27 killed (residual timing corner reported as MINOR below).
- N2 — CLOSED — `consumer_gate` and the receipt path now gate on `topic_exists || is_bound`, `is_bound` fails closed on unreadable bindings, `unreadable_bindings_count_as_bound` passes, m29/m17 killed.
- N3 — CLOSED — the fallback fires only on `code == SESSION_NOT_FOUND && m.contains("Missing 'target'")`, the test adds (-32603, "bus unavailable") → Unreadable, m21 killed.
- N4 / OC 3 — CLOSED — `load_state` saves the marker-carrying empty table at the move (persist_error on failure) and `pending_rows.retain(|row| log(row.clone()).is_err())` keeps the reset row until written; restart test passes, m30 killed.
- N5 — CLOSED — `w.2 = 0` now happens only inside `if log(row).is_ok()`, and `capped_rows_keep_the_suppressed_count_until_written` shows the failed row's suppressed=2 carried to the next write; m28 killed.
- CC 5 (`capped_row` untested) — CLOSED — the same unit test exercises `capped_row`'s rate cap and suppressed accounting end to end.
- N6 — CLOSED — the `why` diff no longer sets `dirty` (`e.why.clone_from(&reader.why)`), leaving persistence to the PERSIST_EVERY_MS cadence exactly as dispositioned.
- N7 / OC 1 — CLOSED — the `waited`/`timeout_ms` parse condition is removed, so every gated subscribe re-checks via `check_bound` before the response is returned; m20 killed.
- OC 2 — CLOSED — `verdict` returns `unknown("this sidecar's own key fingerprint is unavailable")` on empty `me_fp` before any self-matching, with a unit test using a matching empty-fp entry.

**R / D**

- N8 (R) — SOUND — the rejection rests on a measurement (no `"receipt"` post anywhere in the sidecar crate) plus posters signing at post time, so the stale rule can only reject replays, which m22 confirms killed.
- CC 5 unsigned-claim-before-bind (D) — SOUND — claims share the same `consumer_gate` as reads, whose unsigned-before-bind path is tested (E-66.6) and whose removal m16 kills, so the deferred combination is guarded indirectly.

**New defects**

1. MINOR — hub `inbox_consumers.rs` (`record` drop throttle × `consumers` check) — the drop marker refreshes only every `PERSIST_EVERY_MS`, so a drop inside that lag with a short window still gets a clean empty list — table full at t0 (inbox A dropped, marker=t0), inbox B first read at t0+10 s (throttle skips refresh), `inbox.consumers` for B at t0+11 s with `window_s=5` (since=t0+6 s > t0) returns `{"consumers":[]}` although B's reader is unrecorded (FCW-125) — fix: set `st.dropped_since_ms = Some(now_ms)` in memory on every drop (persist on the cadence), or test `t + PERSIST_EVERY_MS > since`.
2. MINOR — sidecar `consumers.rs` `verdict` — the empty-fp early return runs before `answer` is inspected, masking genuine hub faults — sidecar with `me_fp=""` and hub answering `(-32049, "consumer list unreadable")` shows "unknown (this sidecar's own key fingerprint is unavailable)" instead of the unreadable-hub reason — fix: apply the empty-fp guard only to `Ok` answers (after the Err match) so hub-error reasons still surface.
