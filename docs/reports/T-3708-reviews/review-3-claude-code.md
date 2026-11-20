**PASS** — no BLOCKER or MAJOR can be demonstrated from the bundle. The pass is conditional on three points the bundle does not show (defects 1, 2 and 3); each becomes MAJOR if the unseen code behaves the wrong way.

## Dispositions

The bundle carries the round-2 table; round-1 findings appear only as mutant labels, so both are judged from the code.

**Round-1 items visible in code**
- r1 BLOCKER (steady `now_ms` broke cross-process stamps): CLOSED — `pub fn now_ms() -> u64 { wall_now_ms() }`.
- r1 MAJOR 3 (per-agent accounts claimed): CLOSED — every branch of `strength()` yields a string starting `"no"`.
- r1 MAJOR 4 (consumers/revocation stamped by the wrong producer): CLOSED — `if crate::consumers::stamps_field(&v) { touch(CONSUMERS) }`; revocation is rendered from `reg.last(f::REVOCATION)`.
- r1 MAJOR 5 (unknown revocation line reads fine): CLOSED — `revocation_reading` falls through to `"bad"`.
- r1 MAJOR 6 (peer clock in rows): CLOSED — `ts_ms: r.get("ts_ms")…`; `peer_ts_ms` is never read.
- r1 MAJOR 7 (hook miss in memory): CLOSED — `store.write_state(HOOK_MISSED_STATE, &text)`.
- r1 MAJOR 8 (other agents' state in CLEAR): CLOSED — `v["fields_other_agents"]` is built after `f::clear(&m, …)`.
- r1 MAJOR 9 (wall-time deadlines): CLOSED — all five sites use `crate::fields::age_ms(t, now_ms)`.

**Round-2 dispositions**
1. CLOSED for stamps this process writes — `ts_ms: Some(crate::fields::stamp_now())`, `crate::fields::register(now_ms)` in `mark_ready`, `register(now_wall)` in `record_hook_call`. Residual in defects 5 and 6.
2. CLOSED in `check_overdue` and `last_surface` — `age_ms(stored_at, now_ms).min(age_ms(ready_at, now_ms))`. The same wall-ordering pattern survives in `clear` (defect 7).
3. CLOSED by reading only, untested — `libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut ts)`. It introduces defects 8 and 9.
4. CLOSED — `else if ready_since.is_none() && !hook_missed.contains_key(id) && age(*stored) > SURFACE_AFTER_MS { unanchored.push(..) }`, and `clear` pushes `last_surface (…)` to `unknown`.
5. CLOSED for a direct hub call on the status path (m10b plus the silent-hub test). Lock contention is not covered (defect 12).
6. Rejection NOT PROVEN — the body of `failed()` after `touch(REVOCATION)` is not in the bundle, and the added test covers only never-verified → failed (defect 1).
7. CLOSED — `let mut last = lock(&self.last_look); let (mono, wall) = (self.clock.mono_ms(), self.clock.wall_ms());`.
8. CLOSED while the process lives — `HELD_SINCE_UNWRITTEN.store(true, …)` and `broken.push("the held-since record could not be written…")`. The flag is in memory and process-wide; after a restart it is false and the key is re-recorded at a later time.
9. "Accepted as is" is NOT justified — the claim that a backward jump "can only miss one report, never invent one" is false (defect 4).
10. Ages outside the switch: justified, since `age_ms` equals `now - stamp` without a jump. The switch tests are not in the bundle. `record_hook_call` is also outside the switch (defect 11).
11. Unverifiable — `candidates()` is not in the bundle; the disposition itself says withheld mail is excluded (defect 2).
12. Partly unjustified — "lowest = oldest" is false after a backward jump (defect 5).
13. Justified — the shift affects only this process's view and has no production caller.
14. CLOSED — `"no (fewer than two sidecars visible: nothing to compare)"`. Unknown skew or NTP still holds CLEAR (defect 10).

## Defects

1. **MINOR (MAJOR if the unseen branch keeps the line)** — `revocation.rs:Tracker::failed`
   - Wrong: the field is stamped fresh before the line is rewritten, and the rewrite is asserted only in the disposition. `t3708_revoff` never drives a tracker that was verified.
   - Sequence: list verified ("verified seq 7"); hub stops serving the list; `failed()` runs each tick inside the grace period. If that branch leaves the line alone, `fields.revocation` reads fresh "verified seq 7" indefinitely and CLEAR holds.
   - Fix: add a test for verified → failed asserting `revocation_reading(line) != "ok"`, and move the `touch` after `write_status`.

2. **MINOR (MAJOR if `candidates()` drops veto-withheld mail or returns empty on a read error)** — `api.rs:p24_fields` / `fields.rs:pending`
   - Wrong: pending mail is intersected with `d.pending_ids()`, "what may be offered now".
   - Sequence: a message is STORED, the activity veto withholds it for six hours, there is no HANDED_OVER. `pending` is empty, so `earliest` is `None`, `surfaced` is true, `last_surface` is neither bad nor unknown, and status reads CLEAR.
   - Fix: narrow only by terminal states (refused, closed, operator-held). Treat withheld mail as pending, or get an explicit ruling and a test.

3. **MINOR (MAJOR if `status_at` filters or truncates rows by the request)** — `api.rs:p24_fields`
   - Wrong: `rows_of(&v["rows"])` is the only source for pending mail, hand-overs and E-127. `status_at` takes the request `obj`, and its row selection is not in the bundle.
   - Sequence: `{"op":"status","conv":"c1"}` while mail is pending in `c2`. If rows are filtered, CLEAR is computed over `c1` only.
   - Fix: compute CLEAR from an unfiltered chain summary, or assert in a test that rows are always complete.

4. **MINOR** — `revocation.rs:e127_broken`
   - Wrong: `*at > **t` compares two raw wall stamps, so a backward jump can invent a broken invariant.
   - Sequence: a message from key K is STORED lawfully at wall 12:00; the wall is stepped back to 11:00; the list naming K is verified at 11:05, so held-since is 11:05. Since 12:00 > 11:05, the reader reports "STORED after this sidecar held it as revoked".
   - Effect: CLEAR is permanently NOT CLEAR, and a false security attention is raised every day, with no way to clear it.
   - Fix: compare corrected ages (`age(at) < age(t)`) and register the held-since stamp.

5. **MINOR** — `fields.rs:Registry::register`
   - Wrong: eviction removes the lowest wall value, which after a backward jump is the newest stamp.
   - Sequence: the map is full (20,000 chain events, weeks of uptime); the wall steps back further than the map's span. Each new stamp evicts the previous new one, which falls to epoch 0 and is aged with the full negative correction. Every pending hand-over and answer deadline fires at once.
   - Fix: evict by insertion order, or store the epoch with the stamp.

6. **MINOR** — `fields.rs:Registry::age_ms`
   - Wrong: an unregistered stamp gets epoch 0 ("predates all jumps"), which is false for a stamp written by another process during this one's lifetime (a CLI or a second sidecar appending to the same store).
   - Sequence: after a +1 h jump, such a stamp ages 1 h too young for the rest of the process's life; after a −1 h jump it fires immediately.
   - Fix: confirm the daemon is the store's only writer and state it, or persist the epoch alongside the stamp.

7. **MINOR** — `fields.rs:clear`
   - Wrong: `own_handovers(rows).any(|t| t >= e)` and `r.ts_ms >= since` order raw wall stamps; this is the m5d pattern left unfixed.
   - Sequence: hand-over H at wall W−10 min; jump −1 h; new mail STORED at W−55 min. H ≥ stored, so the surfacing term is satisfied by a hand-over that came before the mail, and CLEAR holds until V-8 trips two ticks later. In the other direction, a real hand-over after a −1 h jump reads "no hand-over" for an hour.
   - Fix: compare ages (`age(handover) <= age(stored)`), and add a mutant.

8. **MINOR (new from r2 fix 3)** — `host.rs:start_ntp_sampler`
   - Wrong: `thread::sleep(NTP_EVERY)` runs on CLOCK_MONOTONIC, while the 1 h bound now runs on CLOCK_BOOTTIME.
   - Sequence: after a suspend longer than 30 min, `ntp` is unknown and status is NOT CLEAR for up to 30 min after resume.
   - Fix: sleep in short steps against `mono_ms`.

9. **MINOR (new from r2 fix 3)** — `fields.rs:SystemClock::mono_ms`
   - Wrong: the fallback mixes two time bases per call (ms since boot, ms since process start).
   - Sequence: one failed `clock_gettime` after earlier successes makes `now_mono.saturating_sub(s.mono_ms)` zero, so every field reads fresh, and a giant jump is recorded.
   - Fix: choose the source once at construction. Also, `libc::CLOCK_BOOTTIME` has no `cfg` guard and will not build on non-Linux targets.

10. **MINOR** — `pull.rs:check_pin` / `fields.rs:clear`
    - Wrong: unknown `skew` holds CLEAR, although section 8.6 says skew is not a CLEAR condition.
    - Sequence: an agent with an empty inbox (`CHANNEL_TOPIC_UNKNOWN`) against a hub that has not been upgraded gets no `hub_now_ms` at all. Skew is never stamped and the sidecar is NOT CLEAR for the whole rollout.
    - Fix: render skew as `absent: hub reports no time` (computed) when the hub answered without a clock.

11. **MINOR** — `api.rs` hook arm
    - Wrong: `record_hook_call(store, &d.pending_ids(), …)` runs on every hook call, outside the P24 switch. It adds a second `candidates()` scan and a state read/write to the hook path that rollback does not remove. Only this one arm records, so other hook-driven pulls never produce a V-8 hook anchor.
    - Fix: gate it on `enabled()`, reuse the pull's candidate list, and cover every hook arm.

12. **MINOR** — `tests/t3708_live`
    - Wrong: the silent-hub rig has no deliverer and no pass in flight, so `ready_since` and `pending_ids` on the status path, and any lock held across a 30 s RPC, are untested. The E-55 test returns green with `SKIP` when no hub binary exists.
    - Fix: run status while `rx.pass()` is blocked on the silent hub with a deliverer configured; make the skip a failure in the evidence run.

13. **MINOR** — `host.rs:strength`
    - Wrong: the "separate accounts" text can be false.
    - Sequence: agent `a` under uids {1,2} and agent `b` under uid {1} gives `uids.len() == by_agent.len()`, so the text reads "sidecars run under separate accounts" although uid 1 is shared. `Some(false)` also renders `same_user_separation` as "absent (could not check)" when it was checked.
    - Fix: require pairwise-disjoint uid sets; use distinct text for the checked case. Neither case claims strength.

14. **MINOR** — `api.rs:latency_log`
    - Wrong: two concurrent status calls at the 1 MiB mark both rename; the second moves the new small file over `.1` and loses the older megabyte.
    - Fix: put a mutex around rotate-and-append.

15. **MINOR** — `fields.rs:Registry::look`
    - Wrong: on the 65th jump every epoch is decremented with saturation. Epoch-0 stamps (including all stamps from before this process) silently lose the dropped jump's correction, so their ages shift by that delta.
    - Fix: fold the dropped delta into a base correction applied to epoch-0 stamps.

## Lines (a)–(g)

- **(a) SOUND, with one unproven branch.** `render` gives `unknown` when the stamp is absent or `now_mono - s.mono_ms > bound`, the registry is memory-only, and PULL and CONSUMERS are touched only on an answered call; the revocation verified → failed branch is asserted, not shown (defect 1).
- **(b) WEAK.** An unknown field cannot read CLEAR (`unknown.push` for every non-fresh field, and the unanchored case); but "pending" is whatever `candidates()` returns and whatever `v["rows"]` contains, neither of which is in the bundle (defects 2, 3), and the surfacing term is satisfiable by an earlier hand-over after a backward jump (defect 7).
- **(c) SOUND.** No branch of `strength()` emits a positive value; an unreadable `/proc` gives `"no (could not check)"` and `"absent (could not check)"`, and partial visibility still yields "no".
- **(d) SOUND for the five deadline sites, WEAK at the edges.** Stamps written in-process are classified at write time and wall time stays the cross-host base; the gaps are eviction and foreign writers (defects 5, 6), and the raw wall comparisons left in `clear`, `e127_broken` and `ready_since`'s 24 h limit.
- **(e) SOUND on reading, thinly tested.** `p24_fields` reads only the registry, the store and `/proc`; the epochs (20,000), jumps (64), skew samples (5) and latency log (two files of 1 MiB) are capped. Contention with a blocked pass is unpinned (defect 12), and the per-call chain read grows with the store (accepted risk 7.1).
- **(f) SOUND for the reply shape.** Only new keys are added (`latency_log` is new but missing from design 3.0's list), and the DOWNGRADED text is unchanged today because `stale_gate(true, OwnProject, _)` is true. The new hook-path work and status-path writes sit outside the switch (defect 11).
- **(g) WEAK.** The 40 mutants pin (a), (c), the five deadline sites and the direct hub call. There is no mutant or test for the items below:
  - the verified → failed revocation line;
  - withheld or filtered pending mail;
  - the wall ordering in `clear` and E-127;
  - eviction and the jump cap;
  - suspend;
  - "switch word ignored" (m9 covers only the default);
  - an end-to-end CLEAR over the socket (the rigs never start the NTP sampler, so no live test can read CLEAR).
