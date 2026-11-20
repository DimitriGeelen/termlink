**Verdict: FAIL** — 1 BLOCKER, 8 MAJOR, 6 MINOR.

Where a finding depends on code outside the bundle (`status_at`, `deliver.rs`, `liveness.rs`, the revocation `Tracker`), I say so. I count the missing evidence against the unit, per your "no credit for intent" rule.

## Defects

1. **BLOCKER — `store.rs:now_ms` (build note 8.1)** — The time base of all 56 call sites is swapped with no audit in the bundle, and the change is not behind the switch.
   - **What breaks:** `now_ms()` is now "wall at first call + `Instant`". Every stamp that leaves the process (chain `ts_ms`, liveness `sample_ms`, the heartbeat file, `checked_ms`, `ready-*` and `revocation-held-since` state) stops being wall time after any clock step. It is still compared with clocks that are wall time: other hosts, the hub's `hub_now_ms` (`lease.rs`, `mail_waiting.rs`), and any process with a newer anchor.
   - **Sequence 1 (forward step):** the sidecar starts at boot before NTP sync and the wall is then corrected by +1 h (E-59's own scenario). Its liveness notes carry `sample_ms` 1 h in the past, so every reader's "stale after 5 × 60 s" rule reads a live main as dead.
   - **Sequence 2 (backward step):** corrected by −1 h, its last note reads fresh for an hour after the process dies, which is exactly the stale-as-fresh that P-24 forbids.
   - **Sequence 3 (suspend):** Linux `Instant` is CLOCK_MONOTONIC and stops during a suspend or VM pause, so a 2 h pause leaves the sidecar 2 h behind until restart.
   - **Sequence 4 (restart):** stored `ready-*`, STORED and held-since times from the old anchor are compared with the new anchor, so the step lands on V-8 and E-127 at restart.
   - **Rollback:** `TERMLINK_SIDECAR_P24=off` does not restore the old clock, so "status as before" is false.
   - **Fix:** keep `now_ms()` as wall time for everything persisted or sent. Use `steady_ms()` only at the named in-process deadline comparisons (design 3.5.3 as written), with stored stamps converted at load. Otherwise, audit and list all 56 sites and put the clock behind the switch.

2. **MAJOR — `fields.rs:clear` and `last_surface`** — CLEAR can read CLEAR with mail pending and no hand-over of it.
   - **Window:** the last-hour check filters on `STORED.ts_ms >= now − 1 h`, so mail still pending after an hour drops out. Take m1 STORED at T with no READY file (or one aged out at 24 h) and no hook call because the agent is hung. `last_surface.bad` is false (pinned by `e57_no_ready_no_hook_call_is_not_bad_however_old`), and at T + 1 h + 1 ms `clear` returns CLEAR.
   - **Wrong message:** `own_handovers(rows).any(|t| t >= e)` accepts a hand-over of any message. With m1 STORED at t0 and m2 STORED then HANDED_OVER at t0+2, m1 counts as surfaced.
   - **Evidence:** "HANDED_OVER with its evidence" is never checked; `Row` has no evidence field.
   - **Fix:** treat a message as pending if it is unhanded now, regardless of age; require a HANDED_OVER per pending msg_id; check the evidence.

3. **MAJOR — `host.rs:strength` and `sidecars`** — `per_agent_accounts` can read "yes" on a host that lacks it.
   - **Agent processes ignored:** only sidecar processes are counted; design 3.4 requires the registered sessions' pids. Two sidecars under uids 1001 and 1002 with both agent sessions under uid 1000 read "yes" and "present: one account per agent".
   - **Own process not required:** if this sidecar was started as `--config=/x/me.json` (unparsed), it is skipped. With a at 1001, b at 1002 and me at 1001, the result is "yes".
   - **Partial read failure:** a per-pid `cmdline` or `status` read failure is a silent `continue`. With a at 1001, b at 1002 and c at 1002, one failed read of c gives "yes".
   - **Fix:** include session pids, require the caller's own pid in the set, and make any per-pid failure on a matching process yield "no (could not check)".

4. **MAJOR — `pull.rs` stamps and `api.rs:p24_fields`** — Stamps are not the time the field's own producer succeeded.
   - **Revocation:** it is rendered with `PULL`, which is touched when `hub.version` succeeds, before any revocation fetch or verify. If the hub answers `hub.version` but the list call fails, the field reads `fresh, at_ms = now` over an old "verified seq 7 at …" (assuming the Tracker keeps that line until DR-8; not shown).
   - **Consumers:** `CONSUMERS` is touched on the `Err` path too, so with the hub down the field is `state: fresh` with value "unknown (hub unreachable…)". The live test only checks that `hub.at_ms` does not move.
   - **Fix:** add a `REVOCATION` stamp touched in the Tracker's verified path, and touch `CONSUMERS` only on `Ok`.

5. **MAJOR — `fields.rs:clear`, revocation and supervision arms** — Both arms fail open and the supervision shape is unpinned.
   - **Revocation:** any line that is not one of four known prefixes, or a non-string value (`unwrap_or("")`), is neither bad nor unknown. `{"revocation": "error: bad signature"}` reads CLEAR.
   - **Supervision:** the arm reads `heartbeat`, `stale` and `component_age_ms`. Design section 2 says `supervision` carries no time and the diff adds none, so the "older than 3 ticks" rule is probably dead against real output. No test feeds a real `status_at` supervision value into `clear`.
   - **Fix:** allow-list the good revocation forms ("verified seq", "off…") and treat everything else as bad or unknown; add an integration test with a real dead component.

6. **MAJOR — `fields.rs:rows_of`** — It prefers `peer_ts_ms`, another host's clock, and compares it with local `now` in V-8 and CLEAR, with no skew guard. If a STORED row carries a sender stamp 1 h ahead, `now.saturating_sub(t)` is 0 and V-8 is delayed an hour. If the stamp is behind, V-8 fires at once and the row leaves the CLEAR window early. Fix: use only local `ts_ms`.

7. **MAJOR — `fields.rs:Registry::hook_call` and the `api.rs` pull op** — The hook-miss record does not survive a restart and is keyed differently from the reader.
   - **Restart:** the record is in memory; design 3.3.1 specifies a state file. A hook misses m1, the sidecar restarts, the agent makes no further calls and has no READY: V-8 never fires, and defect 2 then gives CLEAR.
   - **Keying:** it is keyed by `Deliverer::candidates()` while `last_surface` uses `fields::pending(rows)`, so a STORED message that is not a candidate never gets a miss time.
   - **Fix:** persist `hook-missed-<msg>` and use one definition of pending.

8. **MAJOR — `api.rs:p24_fields` (design 3.1.1)** — `main.liveness`, `lease` and `mail_waiting` are hub-derived and are not stamped; `activity` is not wrapped; `fields::LIVENESS` is never touched. With the hub down for 10 minutes, the `main` block still shows its last values and nothing says unknown. Build note 8.2 excludes the main's liveness from CLEAR, not from freshness. Fix: stamp and render them, or blank them to unknown past 2 ticks.

9. **MAJOR — tests and mutants** — The exit criterion "all mutants killed" is not shown; the bundle contains no mutant runner.
   - **Mutant 5 survives:** restore the original `now_ms` body (`SystemTime::now()`). The seam shifts only `wall_now_ms`, so `e59_process_time_ignores_a_wall_jump` still passes. No test drives a `deliver.rs` or `status_at` deadline across a jump.
   - **Mutant 10 survives:** a stopped local hub refuses the connection instantly, so a blocking hub call in status still meets the 1500 ms p95. The live test also returns success on "SKIP".
   - **Mutant 2 is inexpressible:** heartbeats are not rows, so `heartbeat_or_ready_never_counts_as_surfacing` is tautological.
   - **Fix:** a deadline-level jump test against the real clock path, a hub that accepts but never replies, and SKIP reported as not-PASS.

10. **MINOR — the rollback switch** — `enabled()` hides only the keys. Hook recording, held-since writes and the clock swap remain. Only the exact string `off` disables; `OFF` and `0` do not.

11. **MINOR — the test seam and test hygiene** — `store::test_shift_wall_ms` is `pub` in production builds. The ±1 h global shift and `set_var` in `fields_tests` race other threads of the unit-test binary; the "single-threaded" comment there is false.

12. **MINOR — time bases in `fields`** — `p24_fields(now_wall)` receives steady time, so `computed.at_ms` is steady while `render.at_ms` is wall, in the same map. Skew is sampled with steady time (design 3.5.1 says wall), so after a corrected step it reports "clock skew" while NTP says synchronised. Skew, `ntp: "unchecked"`, binary and strength "could not check" are always fresh and never bad.

13. **MINOR — `fields.rs:pending`** — A STORED message that ends in any other terminal stage stays pending forever, so NOT CLEAR never recovers. Rows without a timestamp are dropped silently, which fails open.

14. **MINOR — `api.rs:latency_log` and the status path** — "Never blocks status" is only a comment: the call does a synchronous append and rename, a full `/proc` scan and an attention write. Two concurrent rotations make one rename fail, which counts as a "failing" append.

15. **MINOR — design and code drift** — Build note 8.4 says E-127's reader is open, but it is built. Its rule (a) cannot parse "unverified since start". `record_held_since` swallows write errors. `clear` returns no `why`. The consumers bound is `2 × CHECK_EVERY_MS`, not Tick(2). `same_user_separation` asserts "share uid" when it could not check. RK-1 is listed in slice (a) and is not built.

## Questions

- **(a) WEAK** — Restart and a failed `check_pin` do give unknown, but revocation is stamped by another producer, consumers is stamped on failure, and `main.liveness`, `lease` and `mail_waiting` are not stamped at all (defects 4, 8).
- **(b) WEAK** — Mail pending for more than an hour with no READY or hook reads CLEAR, any other message's hand-over satisfies the check, and the revocation and supervision arms fail open (defects 2, 5, 7). Unknown-by-stamp is handled correctly.
- **(c) WEAK** — An unreadable `/proc` and a root host read absent, but agent session uids are ignored, the caller's own process need not be in the set, and per-pid read failures are skipped, each of which can yield "yes" (defect 3).
- **(d) WEAK** — In-process deadlines on stamps the same run wrote are immune to a jump; any stamp from a peer, the hub, another process or a previous run now differs by the step or the suspend time, which hits liveness readers, V-8 and E-127 (defects 1, 6).
- **(e) SOUND** — `p24_fields` and `latency_log` make no hub call, and the log, the hook map and the skew samples are bounded; the test does not prove the no-hub-call property (defect 9).
- **(f) WEAK** — Key shapes are kept and the DOWNGRADED text is unchanged today, but every existing time field changes meaning through `now_ms`, and the switch does not undo it (defects 1, 10).
- **(g) WEAK** — `render`, V-8 timing and unknown-in-CLEAR are pinned at unit level; mutants 5 and 10 survive, mutant 2 cannot be expressed, the status wiring is never tested on a simulated clock, and the old keys are checked for presence only.
