**Verdict: FAIL** — 1 BLOCKER, 4 MAJOR. I judged only from the bundle; where a defect depends on code not shown, I say so.

## Defects

1. **BLOCKER — `inbox_consumers.rs:record` (table key) and `consumers.rs:verdict` (`is_me`)** — A name-only reader can fold itself into the owner's own entry, so it is never flagged.
   - **Cause:** the table is keyed by (agent, instance) only and strength is last-writer-wins; the sidecar treats any (my agent, my instance or `-`) entry as itself without checking strength or `key_fp`.
   - **Sequence A:** on an unbound inbox, call `channel.subscribe {topic:"inbox:H/fx-a", cursor:0, limit:100, reader:{agent:"fx-a", instance:"-"}}`. It is recorded as (fx-a, `-`, claimed: unsigned) and content is returned. `verdict` puts it in `mine`, picks the `inst-a` entry, and shows `one (proven)` with no attention entry.
   - **Sequence B:** read the instance id from `inbox.consumers` (Observe, any caller), then read with `reader:{agent:"fx-a", instance:"inst-a"}` unsigned. The entry flips to claimed, and the sidecar's next signed long-poll flips it back to proven. The check sees `one (proven)`, or at worst `one (claimed: unsigned)`, never "two identities".
   - **Side effect:** each flip marks the table dirty and rewrites the whole file.
   - **Fix:** key entries by (agent, instance, strength, key_fp), or never merge a claimed action into a proven entry. In the sidecar, count as self only entries that are `proven` with its own key fingerprint. Reject `instance == "-"` in `parse_reader`, or record acks under a separate field.

2. **MAJOR — `consumers.rs:status` / `record`, `pull.rs:check_consumers`** — The status line has no age, so a stale `one (proven)` is shown when the list can no longer be read.
   - **Sequence:** a check succeeds with `one (proven)`; the sidecar restarts with the hub down, or `pass` returns before the `check_consumers` line (it sits after the hub-version `match`, whose early-return arms are not in the bundle), or `write_state` fails (only an `eprintln`). `status` still returns the stored `one (proven)`.
   - **Fix:** store `checked_ms`; report `unknown (stale since …)` when older than about 2×`CHECK_EVERY_MS`; clear the state at start-up; run the check before any early return.

3. **MAJOR — `channel.rs:consumer_gate` → `inbox_consumers.rs:record` (growth and amplification)** — The gate records before the bus confirms the topic exists, and topic length is not checked.
   - **Table size:** `MAX_TOPICS` bounds the count but not the bytes. 4096 calls with `topic = "inbox:" + <frame-max junk>` store 4096 arbitrary-length keys in memory and in `inbox-consumers.json`.
   - **Per-call cost:** each new topic or name is dirty, so the whole table is serialized and atomically written under the single global `STATES` mutex, blocking every inbox read hub-wide.
   - **Admin log:** the `inbox-consumer-evicted` and `inbox-consumer-topic-evicted` rows have no rate cap. Cycling 17 names on one topic, or topics past 4096, writes one admin row per call indefinitely.
   - **Evidence erasure:** topic eviction can drop an idle inbox's list, including an intruder's entry, with only an admin row.
   - **Fix:** record only for topics that exist; cap topic length; rate-cap eviction rows; never evict a topic with entries inside the 24 h window (refuse instead); write off the hot path.

4. **MAJOR — `inbox_consumers.rs:load_state` / `record`** — A corrupt or unknown-schema table is overwritten, and the error state is wrong on both sides of a restart.
   - **Sequence:** the file is unparseable or `schema: v2`. `load_error` is set and the table starts empty. The first inbox read saves over the file, destroying the history (or the newer-schema file). `load_error` is never cleared, so `inbox.consumers` errors until restart. After restart the truncated file loads cleanly and the sidecar can show `one (proven)` for a window whose readers were lost.
   - **Fix:** on a load error, move the file to `.corrupt-<ts>` with an admin row, never overwrite it silently, and persist a "history incomplete since T" marker that keeps `inbox.consumers` in error (or flagged) until the window has passed.

5. **MAJOR — tests and mutants (evidence plan §6.1, §6.4, §6.5)** — Several planned items are missing or unverifiable.
   - **Claim path:** no test sends a `channel.claim` through the hub, signed or refused. The `"offset"` position key and method string are exercised nowhere; the `proof` helper hard-codes `channel.subscribe`.
   - **Ack and claim refusal after bind:** tested only by calling `record` with a hand-built `Reader`, not through `channel.post` or `channel.claim`.
   - **Missing ruling-E cases:** replay on another hub, replay across methods, and stale `ts_ms` refused after a bind.
   - **Unpinned bounds:** nothing tests `MAX_TOPICS`, topic length, the refused-row cap, or defect 1's inputs. No mutant removes the claim gate or makes unreadable bindings fail open.
   - **Unverifiable results:** "Measured" shows only the 5 L4 tests, with no unit-test output. The mutant table gives no killing test per mutant, and the run's own finding admits L4 tests can run a stale binary.
   - **Fix:** add these as L4 cases; list the killing test per mutant; force a rebuild before every L4 run.

6. **MINOR — `channel.rs:handle_channel_subscribe` (bind vs long-poll)** — The bind is checked once, before the read. A long-poll with `timeout_ms` opened by fx-z just before `hub.inbox_bind` still delivers messages posted after the bind. Fix: re-check the binding before returning messages, or cancel waiters on bind.

7. **MINOR — ack gate (`handle_channel_post_with_peer`) / `assess_ack`** — An ack proof has no freshness or domain of its own. fx-a's signed receipt sits in the topic; anyone re-posting it verbatim passes a bound inbox as `proven fx-a` and refreshes its `last_ms`, unless the post path rejects duplicates or stale `ts` (not in the bundle). Fix: enforce the ±120 s window and dedupe on receipt posts.

8. **MINOR — `assess` / design §2.1** — `limit`, `timeout_ms` and `conversation_id` are unsigned, and `position` is `as_u64().unwrap_or(0)` regardless of how the subscribe handler reads `cursor`. A captured `reader` object therefore opens any read at that cursor for 120 s. Fix: sign a hash of the full params, or keep a per-key highest-`ts_ms` seen.

9. **MINOR — `liveness.rs:read`** — Any `first_at` error, not just -32601, falls back to an unsigned content read of the main's inbox. A transient -32603 records `(unnamed)/local` on the main's inbox (false flag), and after a bind yields `Read::Unreadable`. Fix: fall back only on method-not-found.

10. **MINOR — design §4.1 vs diff** — The design says every sidecar subscribe and claim carries a signed reader (`pull.rs`, `liveness.rs`, `deliver.rs`). The diff signs only the pull read and peek; `deliver.rs` is untouched and the liveness fallback is unsigned. If the sidecar claims on its own inbox, it flags itself and is refused after a bind. Fix: sign there, or correct the design.

11. **MINOR — `inbox_consumers.rs` locking and I/O** — The design says per-topic lock; the code uses one global `std::sync::Mutex`. Bindings, key cards and the revocation list are read from disk on every inbox read inside async handlers. One corrupt `inbox-bindings.json` refuses every inbox on the hub, bound or not. Fix: cache with invalidation, per-topic locks, `spawn_blocking`.

12. **MINOR — `assess` timestamp check** — `(now_ms - p.ts_ms).abs()` overflows for `ts_ms = i64::MIN` signed by a carded key: a panic with overflow checks on, a wrap otherwise. Fix: `checked_sub` / `unsigned_abs`.

13. **MINOR — `record` admin rows** — `let _ = log(...)` means an eviction or refusal proceeds when the admin row fails; m8 only covers "no row emitted". The 10/min cap is global, so ten junk refusals on any bound topic silence the rows for a real probe that minute, and the suppressed count is never logged. Fix: per-topic cap plus a "N suppressed" row; surface log failure.

14. **MINOR — `assess` / `assess_ack` vs design §3.1** — A bad signature returns `CHANNEL_SIGNATURE_INVALID`, not the designed code. No refusal counter is incremented. A revoked ack key is downgraded to `claimed (key revoked)` rather than refused; `find` takes the first of duplicate-key cards; `bind` does not check the card key against the revocation list.

15. **MINOR — `channel.first_at`, `channel.receipts`** — Both are ungated and unrecorded on bound inboxes. Any Observe caller can walk offsets and timestamps of every record, and read receipts. The no-content ruling is met, but the traffic metadata of a bound inbox stays open and unlogged.

## One-line answers

- **(a) WEAK** — Name-only and unproven readers are refused at `record`, but a long-poll opened before the bind keeps delivering, a verbatim receipt replay passes as the bound agent, a captured `reader` works for 120 s with any `limit`, and the claim and ack gates are untested end-to-end.
- **(b) WEAK** — `assess` itself is sound (signature, card equality, revocation, window, unambiguous bytes), but the (agent, instance) merge lets an unsigned reader's actions sit inside an entry displayed `proven` (defect 1).
- **(c) WEAK** — `verdict` maps every error or malformed answer to `unknown`, but `status` serves an undated stored `one (proven)` after a restart, early return or write failure, and the hub reports a clean list after overwriting a corrupt one (defects 2, 4).
- **(d) WEAK** — Entry counts are bounded (16 × 4096), but topic key length is not, eviction rows are uncapped, every junk call forces a full-table rewrite under a global lock, and the sidecar's attention entries grow per distinct instance name.
- **(e) WEAK** — (a) is pinned for reads only, (b) only at `assess` level, (c) only for hub error and malformed answer, (d) not at all; the mutant table names no killing tests and the run admits stale-binary contamination.
