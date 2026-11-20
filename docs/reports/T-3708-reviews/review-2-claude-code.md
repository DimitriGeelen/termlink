**1. Verdict: FAIL** — 5 MAJOR, 9 MINOR, no BLOCKER. The round-1 BLOCKER itself is closed, but its replacement (`age_ms`) only holds for stamps aged every tick, and three of the MAJORs sit there.

I judged only from the bundle. Where a finding depends on code that is not shown, it says so.

## Round-1 dispositions

| # | State | Evidence |
|---|---|---|
| 1 | CLOSED as stated; replacement defective | `pub fn now_ms() -> u64 { wall_now_ms() }` restores wall stamps. The new `age_ms` has defects 1–3 below, and the "suspend" part of the finding got no disposition. |
| 2 | Fix CLOSED; rejection justified only by the letter | `let earliest = pending_now.iter().map(|(_, t)| *t).chain(handed_in_window).min();` closes the 1 h drop. The rejection matches design 3.2.2 word for word, but V-8 does not cover what it is said to cover (defect 4). "The line is the evidence" is a comment; no shown code or test supports it. |
| 3 | CLOSED | `("no (sidecars run under separate accounts; the agent sessions' accounts are not checked)".to_string(), Some(false))`; no arm produces "yes". |
| 4 | consumers CLOSED; revocation half-closed | `if crate::consumers::stamps_field(&v) { touch(CONSUMERS) }` is right. `failed()` also calls `touch(REVOCATION)`, which is not "producer last succeeded" (defect 6). |
| 5 | CLOSED | `} else { "bad" }` in `revocation_reading`; supervision reads `heartbeat_age_ms`, `component_age_ms`, `stale` and fails closed on a missing shape. |
| 6 | CLOSED | `ts_ms: r.get("ts_ms").and_then(Value::as_u64).unwrap_or(0)`. |
| 7 | CLOSED | `store.write_state(HOOK_MISSED_STATE, &text)`; writer and reader both key on `pending_ids()`. |
| 8 | Fix CLOSED; `mail_waiting` rejection not justified | `other.insert("main.liveness", …)` and `main.lease` are stamped. The design table says `mail_waiting.rs:65` reads `hub_now_ms` from subscribe replies, which contradicts "computed from local state". |
| 9 | m5/m5b/m5c CLOSED; blocking-hub part OPEN | The disposition's "m10" test is not in the bundle. The shown `t3708_status.rs` is "Hermetic: testkit estate, no hub" with one test, and the list's m10 is `m10-sent-handover-is-ours` (defect 5). |
| 10 | CLOSED for values | `"off" | "0" | "false" | "no"`. The claim "changes no existing behaviour" is wrong: the `age_ms` deadline change is outside the switch (defect 10). |
| 11 | OPEN | `pub fn test_shift_wall_ms` is still compiled into the production library, with no `cfg` or feature gate. |
| 12 | CLOSED | Follows from 1. |
| 13 | CLOSED in the shown code | `.filter(|id| deliverable.is_none_or(|d| d.iter().any(|x| x == id)))`. Whether it is correct depends on unseen `candidates()` (defect 11). |
| 14 | Acceptance justified | A local append; a lost rotation race is reported. |
| 15 | OPEN, as declared | `let _ = store.write_state(HELD_SINCE_STATE, &text);` |

## 2. Defects

1. **MAJOR — `fields.rs:Registry::age_ms`, used by `api.rs:status_at` and `p24_fields`** — A stamp is classified by the jump count when *first aged*. The sender labels (POSTED, SENT, HANDED_OVER) and the V-8 inputs (hook-miss time, the earlier of stored/ready) are first aged only when someone calls `status`, not every tick. The residual is therefore "any jump between the write and the first status call", not "inside one tick".
   - **Sequence:** post at T; wall +1 h at T+10 s (the heartbeat `look()` records jump 1); first `status` at T+20 s. `epochs.entry(stamp).or_insert(jumps.len())` gives epoch 1, correction 0, age 1 h 20 s, so the label flips to `UNKNOWN`/STUCK. With −1 h the age clamps to 0 and the label and V-8 are an hour late.
   - **Test gap:** `t3708_clock` ages its stamp before the jump, so it cannot see this.
   - **Fix:** register the stamp's epoch when it is written (event append, `mark_ready`, `record_hook_call`), or age every open stamp in the heartbeat tick.

2. **MAJOR — `deliver.rs:check_overdue`** — `let start = stored_at.max(ready_at)` orders two wall stamps that may lie on opposite sides of a backward jump, then ages only the winner.
   - **Sequence:** stored at 12:00; wall −1 h at 12:01; agent ready at wall 11:30 (29 real minutes later). `max` picks 12:00; at wall 11:31 the age is 11:31 − 12:00 + 1 h = 31 min. `OVERDUE_HANDOVER` fires one minute after readiness.
   - **Variant:** if the stored stamp was never aged before the jump (agent not ready), its age clamps to 0 and the deadline is an hour late.
   - **Fix:** `age(stored_at).min(age(ready_at))`, as `last_surface` does, with both stamps aged every tick. Add a −1 h deliver test.

3. **MAJOR — `fields.rs:SystemClock::mono_ms`, `look`, `render`** — `Instant` is CLOCK_MONOTONIC on Linux and stops during suspend or a VM pause. Two consequences:
   - `look()` records a suspend as a forward wall jump, so every deadline age excludes real elapsed time. A sender suspended 8 h after POSTED shows no `UNKNOWN` on resume, and V-8 ages shrink the same way.
   - `render` compares monotonic time. A field stamped at 12:00, suspended until 15:00 and read at 15:00:05 reads `fresh` with `at_ms` three hours old, until the producer next succeeds or the bound runs out on post-resume time.
   - **Fix:** read CLOCK_BOOTTIME through `libc::clock_gettime` for `mono_ms`.

4. **MAJOR — design 3.2.2 / `fields.rs:clear`** — The surfacing term is vacuous whenever any hand-over exists.
   - **Why:** a message handed over in the window puts its own STORED into `earliest`, and its own hand-over then satisfies `t >= e`. `own_handovers` is also not limited to the window.
   - **Sequence:** M0 stored 10:00 and still pending; M1 stored 11:00, handed over 11:05; no `ready-*` file and no hook call (session gone). `last_surface.bad` is false, `surfaced` is true, and the state is CLEAR indefinitely with M0 never handed over.
   - **Status:** the code matches the approved sentence, so this needs a ruling rather than a silent change. But the reason given for the rejection ("V-8 is per message") fails exactly when V-8 has no anchor.
   - **Fix:** pending mail older than 2 ticks with no READY or hook anchor makes `last_surface` unknown; or require a hand-over at or after the *latest* pending STORED.

5. **MAJOR — mutant list / tests (round-1 finding 9)** — Nothing in the bundle pins "status never blocks on the hub". Design mutant 10 has no entry in the list, and the test named in the disposition is absent. The bundle also carries no runner output, so no kill is evidenced for any mutant.
   - **Fix:** add the mutant (a `self.rpc("hub.version")` call inside `p24_fields`), include the test, and attach the kill table.

6. **MINOR (MAJOR if the unseen `write_status` keeps the line) — `revocation.rs:Tracker::failed`** — `touch(REVOCATION)` runs on a failed pull. If the line stays `verified seq N` during the PN-13 grace, the field renders `fresh, at_ms = now` for a list last verified well past its 2-tick bound, and CLEAR reads it as ok.
   - **Fix:** stamp only in `verified` and `off`; or have `failed` rewrite the line before the stamp, with a test.

7. **MINOR — `fields.rs:look` / `age_ms`** — The clocks are read before `last_look` is locked. Thread A reads pre-jump, thread B reads post-jump and locks first, then A locks and records a false −1 h jump, and the next look records another +1 h. Stamps first aged between those are off by an hour. Separately, `now_wall` is read by the caller before the inner `look()`, so a jump in between is subtracted from a `now` that does not contain it.
   - **Fix:** read both clocks under the lock, and have `age_ms` take its own `now`.

8. **MINOR — `revocation.rs:record_held_since` (open item 15)** — A failed write leaves `held_since` empty, so invariant (b) can never fire and `revocation_invariants` reads `broken: []`, fresh. That is fail-open. Entries are also never pruned.
   - **Fix:** surface the error as an unknown field.

9. **MINOR — `api.rs:p24_fields` (E-127 input)** — `r["sender"].as_str()?` silently drops every row if rows name the sender differently; no shown test pins the key. A `status` call also writes attention records, and `*at > **t` compares wall stamps, so it misses after a backward jump.

10. **MINOR — the switch / additive rule** — `TERMLINK_SIDECAR_P24=off` does not undo the `age_ms` deadlines, `record_hook_call` writes, the NTP sampler or the skew sampling. `fields_other_agents` is missing from design 3.0 and from `P24_KEYS`. Old keys are checked for presence only, not for equality between switch on and off.

11. **MINOR (unverifiable) — `fields.rs:pending` with `Deliverer::pending_ids`** — `candidates()` is "everything that may be offered *now*". If it excludes stored mail that is merely not offerable at this moment, or if `status_at(obj, …)` filters `rows` by request, that mail vanishes from V-8 and CLEAR. No shown test covers either.

12. **MINOR — `fields.rs` epochs map** — It is keyed by stamp value, so two stamps with the same millisecond on either side of a backward jump collide. Eviction is lowest-first, so stamp 0 and the oldest (most jump-exposed) stamps are re-classified at the current epoch. `unwrap_or(now_ms)` in `deliver.rs` adds a new key every tick.

13. **MINOR — `store.rs:test_shift_wall_ms`** — A public function in the production build that shifts every stored and sent stamp. Gate it behind a feature.

14. **MINOR — `host.rs:strength`, `clear`** — With hidepid or an unmatched cmdline the text says "one agent on this host", which is a claim about the host that was not checked (the flag still reads `no`). NTP `not synchronised` and skew `unusable` never hold CLEAR; if that is intended, the design should say so.

## 3. Lines (a)–(g)

- **(a) WEAK** — Restart and failed hub passes are sound (`None => unknown`, `touch(PULL)` only on `Ok`), but suspend freezes the bound clock (defect 3) and `failed()` stamps revocation (defect 6).
- **(b) WEAK** — Unknown fields do block CLEAR (`f["state"] != "fresh"`), but any hand-over satisfies the surfacing term while other mail stays pending with no V-8 anchor (defect 4).
- **(c) SOUND** — Every arm of `strength` yields `no…`/`absent…`/`none…`; `procs == None` gives `no (could not check)` and `absent (could not check)`.
- **(d) WEAK** — Tick-aged single stamps are jump-immune, but status-path stamps (defect 1), the `max` of two wall stamps (defect 2) and the wall comparisons in `clear` and `e127_broken` are not. No cross-host comparison remains, since `rows_of` uses own `ts_ms`.
- **(e) SOUND on the shown code** — There is no RPC on the status path, NTP is read from a cell, and jumps (64), epochs (20 000), samples (5) and the log (1 MiB + `.1`) are capped. Whether `pending_ids()` shares a lock with a blocked pass is not visible.
- **(f) WEAK** — Keys are additive and `strength`/`missing_strength` are untouched, but existing labels and overdue notices now follow `age_ms` and cannot be switched off (defect 10).
- **(g) WEAK** — There is no blocking-hub mutant or test, no test for a jump before first aging or for −1 h on deliver, no shape-equality check on old keys, and no kill evidence in the bundle.
