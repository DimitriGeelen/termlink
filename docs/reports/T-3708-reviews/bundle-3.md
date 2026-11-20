# T-3708 review bundle 3 (P-24 slice a, after round 2; FINAL round)

## Round 2 dispositions

# T-3708 review round 2 — dispositions

Reviewer: claude-code (FAIL, 5 MAJOR, 9 MINOR, no BLOCKER; round-1 BLOCKER confirmed closed). opencode returned no
text (second round in a row; its seat started and produced nothing). Fixes in the commit "review round 2 fixes".

| # | Finding | Disposition | Change |
|---|---|---|---|
| 1 | MAJOR: stamps classified by jump count when first AGED; status-path stamps are first aged late | **Accepted, fixed** | Stamps are registered with the jump count when WRITTEN: every chain event (`Store` stamps through `fields::stamp_now`), readiness (`mark_ready`), hook misses. A stamp this process did not write predates all its jumps (epoch 0). Test `e59_a_stamp_written_before_a_jump_but_first_aged_after_it_is_still_corrected`; mutant m5e. |
| 2 | MAJOR: `check_overdue` orders two wall stamps with `max` | **Accepted, fixed** | Age = the younger of the two jump-corrected ages. `t3708_clock` runs the reviewer's sequence (stored, wall −1 h, ready 29 min later, checked 30 s after): nothing fires. Mutant m5d. |
| 3 | MAJOR: `Instant` stops during suspend | **Accepted, fixed** | `mono_ms` reads CLOCK_BOOTTIME (`libc::clock_gettime`), which counts through suspend; falls back to `Instant`. A suspend is then neither a jump nor frozen freshness. Not testable without suspending the host; by review. |
| 4 | MAJOR: surfacing term vacuous when any hand-over exists and the pending message has no V-8 anchor | **Accepted, fixed** (within the approved rule) | Pending mail older than 2 ticks with no READY and no hook call makes `last_surface` unknown, so CLEAR reads NOT CLEAR. The approved sentence (FCW-121) stays; this closes the case where V-8 had nothing to measure from. Test `e57_no_ready_no_hook_call_is_not_bad_but_unknown_and_holds_clear`; mutant m34. |
| 5 | MAJOR: blocking-hub mutant and kill evidence missing from the bundle | **Accepted, fixed** | Mutant m10b inserts a hub call on the status path; the silent-hub test (`t3708_live::t3708_status_never_waits_on_a_hub_that_accepts_and_never_replies`) is in the round-3 bundle with the full kill table. |
| 6 | MINOR: `failed()` stamps revocation | **Rejected with evidence** | `failed()` always rewrites the line (`unverified since…`, `stale since…` or the ROLLBACK/predates line) before returning, so the stamped value is the failure, never "verified". Test added in `t3708_revoff`: a failed pass leaves a line `revocation_reading` does not read as ok. |
| 7 | MINOR: clocks read outside the lock | **Accepted, fixed** | Both clocks read under `last_look`'s lock. The caller's `now` is still taken before the look; a jump inside that window is one call's error, gone at the next tick. |
| 8 | MINOR: failed held-since write fails open | **Accepted, fixed** | A failed write sets a flag; E-127's output then says invariant (b) cannot be checked (a bad field). No mutant: the tests run as root, so a store write cannot be made to fail. |
| 9 | MINOR: E-127 sender key; status writes attention; wall compare | **Accepted as is** | `status_at` builds rows with `"sender": l.sender` (same file); attention is coalesced per day; E-127's (b) compares two stamps this host wrote, and a backward jump between them can only miss one report, never invent one. |
| 10 | MINOR: switch does not undo ages; `fields_other_agents` unlisted; old keys only checked for presence | **Partly accepted** | `fields_other_agents` added to the test's key list and design 3.0; old keys are now compared for equality with the switch on and off; every switch word tested. The jump-corrected ages stay outside the switch: without a jump they equal the old `now - stamp`, so they change nothing a reader sees. |
| 11 | MINOR: `candidates()` scope unverifiable | **Accepted as is** | `pending_ids()` is the deliver path's own list of what it may still offer (stored, not handed over, not refused, held or withheld); mail it cannot offer is not "pending surfacing". |
| 12 | MINOR: epochs keyed by stamp value; eviction | **Accepted as is** | Same-millisecond collisions across a backward jump are classified by the first writer; eviction drops the lowest (oldest) stamps, which are past every deadline. |
| 13 | MINOR: `test_shift_wall_ms` public in production | **Accepted as is** | `#[doc(hidden)]`, no caller outside the test binaries; it shifts only this process's view. A feature gate would need the integration tests to build with a feature; not worth the build split. |
| 14 | MINOR: "one agent on this host" claims; NTP/skew never hold CLEAR | **Partly accepted** | Text is now `no (fewer than two sidecars visible: nothing to compare)`. NTP and skew are shown, not CLEAR conditions: AL-29's bad-field list does not include them; said in the design. |

## Mutant kill table

Runner: `python3 crates/termlink-session/tests/mutants/run-t3708.py`. Full run after review round 2 fixes: 38/40 killed; m5e survived (test gap: the clock test's earlier jumps summed to zero) and m34 did not compile (an invalid mutant). Fixed (a direct registration check in t3708_clock; m34 rewritten) and rerun: 2/2 killed. Total 40/40. Not in the list: m35 (a failed held-since write is reported), which cannot be triggered while the tests run as root.

| mutant | what | verdict |
|---|---|---|
| m1-cached-without-stamp | E-19/E-55 control: a field past its bound is served as fresh | killed |
| m2-heartbeat-is-surfacing | E-56 control (arch 6.7.5): pending mail without a hand-over still reads CLEAR | killed |
| m3-idle-age-threshold | E-57 control: an idle agent turns bad by age alone | killed |
| m4-per-agent-yes | E-58 control (r1 MAJOR 3): per_agent_accounts claims yes | killed |
| m4b-root-host-separated | E-58 control: a root host reads as separated | killed |
| m5-wall-time-deadlines | E-59 control (r1 BLOCKER/MAJOR 9): deadline ages follow the wall clock | killed |
| m5b-jumps-by-wall-range | E-59: a stamp's side of a jump is guessed from wall values (fails after a backward jump) | killed |
| m5c-deliver-wall-deadline | E-59: the receiver's hand-over deadline uses plain wall age | killed |
| m5d-deliver-orders-wall-stamps | r2 MAJOR 2: the later of storing and readiness is picked by wall value | killed |
| m6-unknown-counts-fresh | FCW-121: an unknown field does not hold CLEAR | killed |
| m7-failed-pass-stamps-hub | FCW-120: a pass that could not reach the hub stamps `hub` fresh | killed |
| m8-restart-is-fresh | 06a 2.13.f: a field never measured since start reads fresh | killed |
| m9-switch-default-off | design 3.0: the new keys are hidden by default | killed |
| m10-sent-handover-is-ours | a recipient's hand-over of a message we SENT counts as our surfacing | killed |
| m11-v8-from-ready-only | V-8: a message stored after READY is overdue counted from READY | killed |
| m12-hook-time-overwritten | V-8: a later hook call resets the first missed time | killed |
| m27-pending-window-one-hour | r1 MAJOR 2: mail pending for more than an hour drops out of CLEAR | killed |
| m28-unknown-revocation-ok | r1 MAJOR 5: an unknown revocation line reads fine | killed |
| m29-heartbeat-age-ignored | r1 MAJOR 5: an old heartbeat file with fresh component ages reads fine | killed |
| m30-consumers-stamped-on-error | r1 MAJOR 4: a consumers check the hub did not answer stamps the field | killed |
| m31-revocation-off-unstamped | FCW-120: a switched-off revocation pass leaves the field unstamped (reads unknown, never CLEAR) | killed |
| m32-peer-time-in-rows | r1 MAJOR 6: V-8 and CLEAR read the peer's clock | killed |
| m33-hook-miss-in-memory | r1 MAJOR 7: the hook-miss record is not kept | killed |
| m13-e127-ignores-narrowing | E-127 reader: an unverified list past PN-13 fires even while narrowing | killed |
| m14-e127-ignores-held-time | E-127 reader: a message stored before the key was held is flagged | killed |
| m15-skew-last-not-median | V-5: one slow round trip flags clock skew | killed |
| m16-skew-threshold-30s | V-5: clock skew flagged only past 30 s | killed |
| m17-ntp-guess | V-5: with no tools the NTP state is claimed synchronised | killed |
| m18-stale-revocation-ok | AL-29: a stale revocation list is not a bad field | killed |
| m19-second-consumer-ok | AL-29: more than one consumer is not a bad field | killed |
| m20-downgrade-reason-never-stale | carried T-3703 CC N4: a stale-gate downgrade names the allow-list | killed |
| m21-no-skew-from-version | V-5: an agent with an empty inbox is never measured for skew | killed |
| m22-hub-version-no-time | V-5: the hub's version reply carries no clock | killed |
| m23-latency-not-logged | E-55/E-134: status calls leave no latency line | killed |
| m24-pass-not-stamped | FCW-120: a successful pass does not stamp `hub` | killed |
| m25-hub-predates-is-fine | no verified list (`hub predates`) does not hold CLEAR | killed |
| m26-invariant-ignored | a broken revocation invariant does not hold CLEAR | killed |
| m10b-status-calls-the-hub | design mutant 10 (r2 MAJOR 5): status makes a hub call (blocks on a silent hub) | killed |
| m5e-stamps-not-registered | r2 MAJOR 1: chain stamps are not registered when written | killed |
| m34-unanchored-pending-ignored | r2 MAJOR 4: pending mail with nothing to measure from does not hold CLEAR | killed |

## Design

# T-3708 design — P-24 / B-14: honest status (freshness, CLEAR, V-8, strength, skew) and the reachability card

Draft 2026-11-20. Inputs: `docs/reports/T-3708-spec-extract.md` (spec sources, evidence list), the code as of
`9f2eff1e9`. Paths are relative to the repo root. Every "today" claim was read this session; line numbers are for
`9f2eff1e9`.

## 1. What P-24 must deliver (one paragraph)

`termlink sidecar status` must never show a stale value as current, never count a heartbeat as surfacing, and must name
the strength the host lacks (AL-29, 06a 2.13). Each agent publishes a small reachability card on its home hub so a
sender can see whether a recipient will actually be woken (arch 6.7.2). The sender sees honest GP-4 lines when a hub or
recipient is offline (arch 8.2, E-60). Plan exit: E-19, E-55..E-61 and E-134 pass at their levels, status answers in 5 s
(p95), fixture suite with a PASS line per E-n, all mutants killed. Rollback: the new fields are hidden and status is as
before.

## 2. What exists today (verified)

| Piece | Where | State |
|---|---|---|
| Status view | `crates/termlink-sidecar/src/api.rs:1112` `status_at` (pure) + `api.rs:449-484` live additions | Fields `mode`, `hub`, `hub_signing_key`, `revocation`, `consumers` (P-12, with `checked_ms`), `strength` (bool map) + `missing_strength`, `supervision`, `outbox`, `activity_veto`; live: `binary`, `main` (+`liveness`, `lease`, `mail_waiting`), `activity`, `doorbell`, `keys`, `vendor_log`. Only `consumers`, `revocation` and `activity` carry a time. |
| Strength flags | `api.rs:1299` `strength_flags` | Static: four `false` constants and `hub_id_pinned`. Nothing is read from the host. |
| Liveness note | `liveness.rs:76` `Note {agent, instance, sample_ms, drained_up_to, receiving}`; posted every `NOTE_EVERY_MS` = 60 s (`:39`) | **Only the main posts it** (`liveness.rs:380`, `is_self`). Readers judge stale after 5 × 60 s. |
| READY time | `deliver.rs:450` `ready_since` (state files `ready-<session>`, max age 24 h) | Exists. |
| Hook calls that did not hand over | — | **Not recorded.** Needed for V-8. |
| Deadlines | `deliver.rs:509-517` (receiver), `api.rs:1196, 1225, 1243` (sender) | **Wall-clock** `now_ms` against stored wall `ts_ms`. A ±1 h wall jump fires them early or late, against E-59. |
| Hub time | `hub_now_ms` in `channel.subscribe` replies (CR-6.21; read by `lease.rs:108`, `mail_waiting.rs:65`) | Exists; usable for skew. |
| NTP check | `scripts/substrate-preflight.sh` (P-23, T-3364, closed) | Bash only; not in the sidecar. |
| Quota | P-17 / T-3705 (B-7 receiver quotas), **captured, not built** | The `quota` field has nothing to read yet. |
| WAITING (sender) | `send.rs:50`, label at `api.rs:1186` | Exists; no "home hub unreachable since T" text. |
| `strength` readers | `client.rs:151` (`missing_strength`), `scripts/lib/arc011-acceptance-report.py:256` (`missing_strength`), AEF `fw sidecar status` | The shape of `strength` / `missing_strength` must not change (see 3.0). |

## 3. Design

### 3.0 Compatibility rule (learned today, e15e4c380)

Everything is **additive**. No existing status key changes shape or meaning; no existing note field changes. New
content goes under new keys (`fields`, `clear`, `strength_host`, `skew`, `reachability`) and new optional note fields.
A reader that predates P-24 sees exactly what it sees today. Rollback = stop emitting the new keys (one switch,
`TERMLINK_SIDECAR_P24=off`, default on), which is the plan's "fields hidden; status as before".

### 3.1 Field freshness (FCW-120; E-19, E-55)

A new module `fields.rs` holds an in-memory registry, written by producers and read by `status`:

```text
Stamp { value: Value, at_mono: Instant, at_wall_ms: u64, bound: Bound }    Bound = Tick(2) | Heartbeat(60 s) | Daily(1 h)
render(stamp, now_mono) = if absent or now_mono - at_mono > bound: {"state":"unknown","last_at_ms":at_wall_ms|null}
                          else {"state":"fresh","value":…,"at_ms":at_wall_ms,"bound_ms":…}
```

1. Producers and bounds: pull pass (`hub`, `revocation`, `consumers`: Tick(2)); `beat` (`supervision`: Heartbeat);
   liveness cycle (`main.liveness`, `reachability`: Tick(2)); skew sampler (Tick(2)); NTP sampler and adapter report
   (Daily). `activity` keeps its own stamps (B-27) and is wrapped, not rewritten.
2. Freshness uses `Instant` (monotonic); the wall time is display only ("last at T").
3. The registry lives in memory only. After a restart every field is absent, so it reads `unknown` and the sidecar
   is never CLEAR until it has re-measured (06a 2.13.f).
4. Status reads only the registry and the local store; it makes **no blocking hub call** (2.13.f.1). The existing
   chain read in `status_at` stays (local disk), and its cost is measured by E-55/E-134 (3.9).
5. Test seam: a `Clock` trait (mono + wall) so E-19/E-57/E-59 run on a simulated clock without sleeping.

Output: `status.fields = {name: rendered stamp}` next to the existing keys.

### 3.2 Bad fields and CLEAR (FCW-121; E-56)

A pure function `clear(fields, surfacing, now) -> {state: "CLEAR"|"NOT CLEAR", bad: [..], unknown: [..], why}`:

1. Bad: `hub.match` false or unknown; revocation stale (T-3703's DR-8 rule); consumers > 1 identity (P-12 verdict);
   supervision dead; quota refusing receipts (3.7); `last_surface` bad (3.3); any component heartbeat older than 3 ticks.
2. CLEAR iff nothing bad, nothing unknown, and either nothing was pending in the last hour or a HANDED_OVER with
   evidence exists after the earliest pending STORED of the last hour.
3. A heartbeat, a READY report or a pull without evidence never counts as surfacing. Only a HANDED_OVER chain line
   with its evidence counts. Mutant m-heartbeat (count a heartbeat as surfacing) must read CLEAR in the injected
   case, which kills it.

### 3.3 V-8 last surface (E-57)

1. Inputs: STORED and HANDED_OVER lines from the chain (exists), `ready_since` (exists), and **new**: the time of the
   first hook call that ran while a STORED message was pending and did not hand it over (recorded by `hook.rs` in a
   state file `hook-missed-<msg>`, written once per message).
2. Bad iff a STORED message exists, not handed over, and (READY at t_r and now − t_r > 2 ticks) or (hook miss at t_h
   and now − t_h > 2 ticks), with times compared on the steady clock (3.5).
3. An idle agent with nothing pending is never bad; status shows the age of the last surface.
4. `last_surface` = the latest HANDED_OVER-with-evidence time, never a heartbeat or READY time.

### 3.4 Strength flags from the host (FCW-123; E-58)

New key `strength_host`, computed at each status call by reading `/proc` (no subprocess):

| Flag | Computed as | Absent text |
|---|---|---|
| `per_agent_accounts` | distinct uids across this agent's sidecar and agent processes (the sidecar's own uid + the uids of registered sessions' pids) | `no` / `no (could not check)` |
| `same_user_separation` | uid 0 → `absent: agents run as root`; else `absent: same user` unless per-agent accounts | `absent (could not check)` |
| `operator_key` | an operator key configured and loadable | `no` |
| `confirmation_provider` | a configured provider | `no` (none exists in PH-2) |
| `policy_records` | operator policy records present | `none (fixed defaults)` |

A flag that cannot be computed reads as absent, never present. The old `strength` and `missing_strength` stay as they
are. E-58 compares each flag with `ps -o user=` on the W1 pair host; its control fakes `per_agent_accounts: yes` on a
root host and must FAIL.

### 3.5 Skew, NTP and steady time (V-5; E-59)

1. **Skew sample.** Each pull pass already calls `channel.subscribe`; it records (wall before, wall after, mono rtt,
   `hub_now_ms`). skew = `hub_now_ms` − midpoint. Keep the median of the last 5 samples. `> 2 s` → "clock skew";
   `> 30 s` → cross-host delay figures shown as "unusable".
2. **NTP.** A background sampler, hourly, runs `timedatectl show -p NTPSynchronized --value`, else `chronyc tracking`,
   else `unchecked`. It reuses P-23's commands, with a 2 s timeout. Status never shells out.
3. **Steady time for deadlines (gap found in this design).** Today deadlines compare wall `now_ms` with stored wall
   `ts_ms`, so a wall jump fires them. Fix: one process clock `steady_ms() = wall_at_start + (Instant::now() −
   start)`. All deadline checks (`deliver.rs` receiver side, `status_at` sender labels) use `steady_ms()`, and
   events written in this process are stamped with it. A jump while the sidecar was down cannot be seen; the first
   skew sample after start reports it. E-59's ±1 h jump control (a mutant using wall time) must fire at the jump.

### 3.6 Reachability card (FCW-122; E-61) — slice (b)

1. Every sidecar posts its own note every tick, not only the main. The note gains optional fields:
   `card: {receiver_up, right_hub, adapter_present, last_surface_ms}`. Old readers ignore unknown fields; this is
   checked by a test that parses a P-24 note with the pre-P-24 struct.
2. `receiver_up` = heartbeat fresh; `right_hub` = `hub.match`; `adapter_present` = a valid adapter report within
   24 h (the meter's adapter record); `last_surface_ms` = 3.3's value, never a heartbeat time.
3. Reader rule: a card older than 2 × PN-9 (2 × 600 s) reads UNKNOWN, never DEAD. Sender status shows the
   recipient's card when the address book or directory names its hub.
4. Readers of other projects see only reachable yes/no and the version class. That needs the PH-3 read scope, so in
   PH-2 the card carries no secret content and the restriction is recorded as a PH-3 item (E-61 PH-3 part).
5. Hub load: one note per agent per minute, retention `NOTE_KEEP` (as today for mains).

### 3.7 Quota

P-17 (receiver quotas) is not built, so `quota` has no producer. See decision D-2 in section 6.

### 3.8 GP-4 offline lines (E-60) — slice (b)

1. Sender without its hub: `send` returns an id, the message is durable in the outbox (exists), and the label reads
   `WAITING: home hub unreachable since T`, where T is the first failed post after the last success. Nothing claims
   RECEIVED from the local outbox (an existing rule; E-60's mutant checks it).
2. Recipient offline: after its card expires (PN-9, 600 s) the sender shows WAITING FOR RECIPIENT, not STUCK.
3. On restore, the pump reads the hub record first and posts only what is missing (no duplicates by the identity
   triple). The existing pump dedupe must be verified at L4 under a real network cut (netns or firewall rule on a
   throwaway hub).

### 3.9 Latency and cause logging (E-55, E-134)

1. Status appends one line per call to `status-latency.jsonl` in the store (ms, fields unknown, CLEAR state), keeping
   a bounded size (rotate at 1 MiB, keep 2). A failed append is counted in status (`latency_log: failing`), never
   silent.
2. E-55: 100 calls per injected fault (the E-63 set; injectors are B-15/P-19, so P-24 uses the fault seams that
   exist, and E-55's tier-3 run waits for P-19 where an injector is missing; this is said in the E-55 PASS line).
3. E-134 needs the day-long latency log; P-24 builds the log, and the daily evaluation is a canary candidate subject
   to the guard cap (DP-3).

### 3.10 Carried from T-3703 reviews

1. WAITING reason `key revoked, operator action needed` (RK-1) on the sender's label when the recipient refused
   because the sender's key is revoked (exists as `refused-revoked`; add the operator wording).
2. E-127's reader: the `refused_revoked` counts in `revocation` status (verify; add if missing).
3. The `revocation` status vocabulary: fresh / stale (DR-8) / unknown.
4. The DOWNGRADED line naming `revocation list stale`.

## 4. Proposed slices

| Slice | Content | Evidence | Levels |
|---|---|---|---|
| (a) honest status | 3.1 freshness, 3.2 CLEAR, 3.3 V-8, 3.4 host strength, 3.5 skew/NTP/steady deadlines, 3.7 quota, 3.9 latency log, 3.10 | E-19, E-55, E-56, E-57, E-58, E-59, E-134 | L2 (simulated clock) + L4 on .107 for E-58, E-55 tier 3 |
| (b) card + offline | 3.6 card, 3.8 GP-4 lines | E-60, E-61 | L4 with a throwaway hub and a network cut |

(b) depends on (a): the card's `last_surface` and `right_hub` are (a)'s fields. Each slice has its own review rounds
(cap 3), its own mutant runner, and its own runme action if a live check is needed.

## 5. Mutants (first list)

1. Serve a cached field without a stamp (E-19/E-55 control).
2. Count a heartbeat as surfacing (E-56 control, arch 6.7.5).
3. Age threshold on an idle agent (E-57 control).
4. Report `per_agent_accounts: yes` on a root host (E-58 control).
5. Use wall time for a deadline (E-59 control).
6. Report RECEIVED from the local outbox (E-60 control).
7. Write the heartbeat time into `last_surface` (E-61 control).
8. Treat an unknown field as fresh in CLEAR.
9. Read a card older than 2 × PN-9 as DEAD instead of UNKNOWN.
10. Status makes a blocking hub call (latency under the hub-down fault).

## 6. Decisions for the operator (one at a time)

1. **D-1 slicing**: build as the two slices of section 4, or as one unit.
2. **D-2 quota before P-17**: how the `quota` field behaves while receiver quotas do not exist (e.g. `not built
   (P-17)` and excluded from CLEAR until P-17, versus `unknown`, which keeps every sidecar NOT CLEAR until P-17).

The steady-time fix (3.5.3) is inside the plan's scope (E-59 requires it), so it is proposed rather than asked.

## 7. Risks

1. Status cost: `status_at` reads every conversation's chain from disk on each call. Under a slow-disk fault this
   may exceed 5 s. E-55 measures it; if it fails, the chain summary moves into the registry (produced per tick).
2. Every sidecar posting a note multiplies liveness topics by the agent count (about 20 on .107). Retention bounds it.
3. The steady clock changes the time base of new chain stamps. Old lines keep wall times. Mixing is safe only while
   the process runs; a restart re-anchors.

## 8. Build notes (slice a, 2026-11-20)

1. **Time (replaces 3.5.3 after review round 1).** Every stamp stays wall time. Deadlines are ages: `fields::age_ms`
   subtracts every wall jump the process saw after it first aged a stamp (a jump is wall and monotonic time
   disagreeing by more than 2 s between two looks; the heartbeat looks every tick). A stamp's side of a jump is
   decided by how many jumps had been seen when it was first aged, because after a backward jump wall values alone
   cannot tell. Residual: a jump inside the tick a stamp is first seen in, or while the process is down, is not
   corrected; the skew line shows the latter. (The first build made `now_ms` itself steady; review round 1 showed that
   breaks every stamp compared across processes and hosts.)
2. **CLEAR judges this agent's own fields** (refines 3.2): mode, binary, hub, revocation, consumers, supervision,
   quota, strength, skew, ntp, last_surface. The main's liveness is another agent's state and keeps its own verdict
   under `main`. `revocation` reads unknown when `unverified …` or `hub predates revocation list` (no verified list:
   revoked keys are not known), bad when `stale …` or `ROLLBACK …`.
3. **V-8 counts from the later of storing and readiness** (as R-44.e and `deliver.rs` do), so a message that arrives
   after the agent became ready gets its 2 ticks.
4. **Carried T-3703 items:** the vocabulary (3.10.3), the DOWNGRADED reason (3.10.4) and E-127's reader (3.10.2) are
   built. The reason applies only once the urgent allow-list is widened beyond the roster (`policy::urgent_allowed`).
   E-127's reader keys "a message from a revoked key is STORED" on the time this sidecar first held the key as
   revoked (`revocation-held-since`), because a message stored before the sidecar knew of the revocation was stored
   lawfully. RK-1 (3.10.1) moved to T-3834 (a sender-side label that needs a recipient record outside the hashed
   chain).
5. **Strength (review round 1):** the agent sessions' accounts cannot be read from the sidecar, so
   `per_agent_accounts` never reads "yes" in PH-2; it names why. `fields_other_agents` carries `main.liveness` and
   `main.lease` with their own stamps, outside CLEAR. The consumers bound is 2 x its 30 s check.
6. **Review round 2.** Stamps are registered with the jump count when written, not when first aged; the receiver deadline
   uses the younger of two corrected ages; the monotonic clock is CLOCK_BOOTTIME (counts through suspend). Pending
   mail with no READY and no hook call makes `last_surface` unknown, so CLEAR waits. NTP state and clock skew are
   shown but are not CLEAR conditions (AL-29's bad-field list does not include them). `fields_other_agents` is part
   of the additive key set (3.0).

## Code (git diff of the slice, committed; unit tests in src/fields_tests.rs omitted for size)

```diff
diff --git a/crates/termlink-hub/src/router.rs b/crates/termlink-hub/src/router.rs
index 4ac8e37fc..5eff04e76 100644
--- a/crates/termlink-hub/src/router.rs
+++ b/crates/termlink-hub/src/router.rs
@@ -1221,6 +1221,8 @@ fn handle_hub_version(id: serde_json::Value) -> RpcResponse {
             "hub_instance_id": instance,
             "signing_key": signing_key,
             "signing_public_key": signing_public_key,
+            // T-3708 (V-5): the hub's clock, so every client pass can estimate skew (additive field)
+            "hub_now_ms": crate::channel::hub_now_ms(),
         }),
     )
     .into()
diff --git a/crates/termlink-sidecar/src/api.rs b/crates/termlink-sidecar/src/api.rs
index d494e6782..51291ee04 100644
--- a/crates/termlink-sidecar/src/api.rs
+++ b/crates/termlink-sidecar/src/api.rs
@@ -447,6 +447,7 @@ fn dispatch_inner(req: &Value, scope: Scope, store: &Store, agent: &str, hub_id:
     }
     match op {
         "status" => {
+            let started = std::time::Instant::now();
             let mut v = status_at(obj, store, agent, hub_id, sender, crate::store::now_ms(), &crate::deadline::Deadlines::R44E);
             // SI-27 (P9): the running binary against the release checksum record.
             // Added here, on the live socket path, not in the pure view.
@@ -480,6 +481,14 @@ fn dispatch_inner(req: &Value, scope: Scope, store: &Store, agent: &str, hub_id:
                 let vl = vendor_log_view(ctx.hook_errlog, store);
                 v["vendor_log"] = vl["state"].clone();
                 v["vendor_log_detail"] = vl;
+                // T-3708 (P-24 slice a): stamped fields, CLEAR, host strength, skew (additive; design 3.0).
+                if crate::fields::enabled() {
+                    let ready = ctx.deliverer.and_then(|d| d.ready_since(crate::store::now_ms()));
+                    let deliverable = ctx.deliverer.map(|d| d.pending_ids());
+                    p24_fields(&mut v, crate::fields::global(), Some(store), ready, deliverable.as_deref(), crate::store::now_ms());
+                    // E-55 / E-134 (design 3.9): one latency line per call, bounded; a failing log is shown
+                    v["latency_log"] = json!(latency_log(store.dir(), started.elapsed().as_millis() as u64, &v));
+                }
             }
             v
         }
@@ -498,6 +507,10 @@ fn dispatch_inner(req: &Value, scope: Scope, store: &Store, agent: &str, hub_id:
             if let (Some(m), Some(sess)) = (ctx.meter, obj.get("session").and_then(Value::as_str)) {
                 m.legacy_hook(sess, "post-tool-use", None, crate::meter::adapter_ord(obj));
             }
+            // T-3708 (V-8): a hook call ran while these messages were pending; the first such time counts.
+            if let Some(d) = ctx.deliverer {
+                crate::fields::record_hook_call(store, &d.pending_ids(), crate::store::now_ms());
+            }
             pull(obj, ctx.deliverer, ctx.lease, agent)
         }
         "claim" => claim(obj, ctx.deliverer),
@@ -1193,7 +1206,7 @@ fn status_at(
                 // signal (row P3) the recipient may simply not be running: UNKNOWN,
                 // never STUCK (R-44.a).
                 let posted = first_ts(crate::send::POSTED);
-                if posted.is_some_and(|t| now_ms.saturating_sub(t) > dl.accept(urgent)) {
+                if posted.is_some_and(|t| crate::fields::age_ms(t, now_ms) > dl.accept(urgent)) {
                     v["label"] = json!("UNKNOWN");
                 }
             }
@@ -1222,7 +1235,7 @@ fn status_at(
             let sent_here = rows.iter().any(|r| r["msg_id"] == *m && r["stage"] == "SENT");
             // R-44.e (PH-2 row P2). Received but never stored: the recipient is
             // demonstrably alive, so past the accept deadline it is STUCK.
-            if sent_here && reached == "RECEIVED" && sent_at.is_some_and(|t| now_ms.saturating_sub(t) > dl.accept(urgent)) {
+            if sent_here && reached == "RECEIVED" && sent_at.is_some_and(|t| crate::fields::age_ms(t, now_ms) > dl.accept(urgent)) {
                 stuck(&mut v, crate::deadline::STEP_NOT_ACCEPTED);
             }
             // Stored, not handed over: only the recipient knows when its agent became
@@ -1240,7 +1253,7 @@ fn status_at(
             if sent_here && reached == "HANDED_OVER" {
                 if has(crate::deadline::OVERDUE_ANSWER) {
                     stuck(&mut v, crate::deadline::STEP_NOT_ANSWERED);
-                } else if first_ts("HANDED_OVER").is_some_and(|t| now_ms.saturating_sub(t) > dl.answer(urgent)) {
+                } else if first_ts("HANDED_OVER").is_some_and(|t| crate::fields::age_ms(t, now_ms) > dl.answer(urgent)) {
                     v["label"] = json!("UNKNOWN");
                 }
             }
@@ -1295,6 +1308,108 @@ fn status_at(
     })
 }
 
+/// T-3708 (P-24 slice a; AL-29, arch 6.7.1; design 3.1-3.5): add `fields` (each stamped, or
+/// `unknown (last at T)` past its bound), `clear`, and `strength_host` to a status reply. Additive:
+/// no existing key is touched. CLEAR is judged over this agent's own fields; the main's liveness is
+/// another agent's state and keeps its own verdict under `main`.
+pub(crate) fn p24_fields(
+    v: &mut Value,
+    reg: &crate::fields::Registry,
+    store: Option<&Store>,
+    ready_since: Option<u64>,
+    deliverable: Option<&[String]>,
+    now_wall: u64,
+) {
+    use crate::fields::{self as f, BOUND_DAILY_MS, BOUND_HEARTBEAT_MS, BOUND_TICKS_MS};
+    let mono = reg.clock().mono_ms();
+    let rows = f::rows_of(&v["rows"]);
+    let pending = f::pending(&rows, deliverable);
+    let missed = store.map(f::hook_missed).unwrap_or_default();
+    let age = |t: u64| reg.age_ms(t, now_wall);
+    let strength = crate::host::strength_now();
+    let mut m = serde_json::Map::new();
+    m.insert("mode".into(), f::computed(&v["mode"], now_wall));
+    m.insert("binary".into(), f::computed(&v["binary"], now_wall));
+    m.insert("hub".into(), f::render(&v["hub"], reg.last(f::PULL), BOUND_TICKS_MS, mono));
+    // r1 MAJOR 4: stamped by the revocation tracker's own evaluation, not by the hub.version check
+    m.insert("revocation".into(), f::render(&v["revocation"], reg.last(f::REVOCATION), BOUND_TICKS_MS, mono));
+    m.insert("consumers".into(), f::render(&v["consumers"], reg.last(f::CONSUMERS), 2 * crate::consumers::CHECK_EVERY_MS, mono));
+    m.insert("supervision".into(), f::render(&v["supervision"], reg.last(f::HEARTBEAT), BOUND_HEARTBEAT_MS, mono));
+    // T-3708 D-2 (operator ruling A): no receiver quotas yet (P-17), so nothing can refuse receipts.
+    m.insert("quota".into(), f::computed(&json!(f::QUOTA_NOT_BUILT), now_wall));
+    m.insert("strength".into(), f::computed(&strength, now_wall));
+    let skew = crate::host::skew_view().unwrap_or(Value::Null);
+    m.insert("skew".into(), f::render(&skew, reg.last(f::SKEW), BOUND_TICKS_MS, mono));
+    let ntp = crate::host::ntp_view().map(Value::from).unwrap_or(Value::Null);
+    m.insert("ntp".into(), f::render(&ntp, reg.last(f::NTP), BOUND_DAILY_MS, mono));
+    m.insert("last_surface".into(), f::computed(&f::last_surface(&rows, &pending, ready_since, &missed, &age), now_wall));
+    // E-127's reader (carried from T-3703 r1 finding 17): the revocation invariants, computed from local state
+    if let Some(store) = store {
+        let stored: Vec<(String, String, u64)> = v["rows"]
+            .as_array()
+            .into_iter()
+            .flatten()
+            .filter(|r| r["stage"] == "STORED")
+            .filter_map(|r| Some((r["msg_id"].as_str()?.to_string(), r["sender"].as_str()?.to_string(), r["ts_ms"].as_u64()?)))
+            .collect();
+        let line = v["revocation"]["revocation"].as_str().unwrap_or("");
+        let mut broken = crate::revocation::e127_broken(line, crate::revocation::narrowing(store), &crate::revocation::held_since(store), &stored, now_wall);
+        if crate::revocation::held_since_unwritten() {
+            broken.push("the held-since record could not be written: a STORED message from a revoked key cannot be checked".into());
+        }
+        if !broken.is_empty() {
+            let day = (now_wall / 86_400_000).to_string();
+            let _ = store.attention_coalesced("revocation-invariant", &day, &broken.join("; "));
+        }
+        m.insert("revocation_invariants".into(), f::computed(&json!({"broken": broken}), now_wall));
+    }
+    v["clear"] = f::clear(&m, &rows, &pending, now_wall);
+    // r1 MAJOR 8: the hub-derived views of another agent's state, stamped too, but outside CLEAR
+    let mut other = serde_json::Map::new();
+    if v["main"].get("liveness").is_some() {
+        other.insert("main.liveness".into(), f::render(&v["main"]["liveness"], reg.last(f::LIVENESS), BOUND_TICKS_MS, mono));
+    }
+    if v["main"].get("lease").is_some() {
+        other.insert("main.lease".into(), f::render(&v["main"]["lease"], reg.last(f::LEASE), BOUND_TICKS_MS, mono));
+    }
+    v["fields"] = Value::Object(m);
+    v["fields_other_agents"] = Value::Object(other);
+    v["strength_host"] = strength;
+}
+
+/// The status latency log (E-55, E-134): one JSON line per status call in the store.
+pub const LATENCY_LOG: &str = "status-latency.jsonl";
+/// Rotate the latency log at this size, keeping one older file (`.1`).
+pub const LATENCY_LOG_MAX: u64 = 1 << 20;
+
+/// Append one latency line; returns `ok` or `failing (N appends failed)`. Never blocks status.
+pub(crate) fn latency_log(dir: &Path, ms: u64, v: &Value) -> String {
+    use std::sync::atomic::{AtomicU64, Ordering};
+    static FAILED: AtomicU64 = AtomicU64::new(0);
+    let path = dir.join(LATENCY_LOG);
+    let line = json!({
+        "ts_ms": crate::store::now_ms(),
+        "ms": ms,
+        "clear": v["clear"]["state"],
+        "unknown": v["clear"]["unknown"],
+        "bad": v["clear"]["bad"],
+    });
+    let r = (|| -> std::io::Result<()> {
+        if std::fs::metadata(&path).map(|m| m.len() >= LATENCY_LOG_MAX).unwrap_or(false) {
+            std::fs::rename(&path, dir.join(format!("{LATENCY_LOG}.1")))?;
+        }
+        use std::io::Write as _;
+        use std::os::unix::fs::OpenOptionsExt as _;
+        let mut f = std::fs::OpenOptions::new().create(true).append(true).mode(0o600).open(&path)?;
+        writeln!(f, "{line}")
+    })();
+    match r {
+        Ok(()) if FAILED.load(Ordering::SeqCst) == 0 => "ok".into(),
+        Ok(()) => format!("ok now ({} appends failed earlier)", FAILED.load(Ordering::SeqCst)),
+        Err(e) => format!("failing ({} appends failed: {e})", FAILED.fetch_add(1, Ordering::SeqCst) + 1),
+    }
+}
+
 /// Own strength flags: (name, present, text shown when absent).
 fn strength_flags(hub_id: Option<&str>) -> Vec<(&'static str, bool, &'static str)> {
     vec![
diff --git a/crates/termlink-sidecar/src/consumers.rs b/crates/termlink-sidecar/src/consumers.rs
index ea6102712..cf771eb81 100644
--- a/crates/termlink-sidecar/src/consumers.rs
+++ b/crates/termlink-sidecar/src/consumers.rs
@@ -158,8 +158,29 @@ pub fn status(store: &Store, now_ms: u64) -> Value {
     }
 }
 
+/// T-3708 (FCW-120, r1 MAJOR 4): whether a check's verdict stamps the `consumers` field fresh. Only a
+/// verdict from a list the hub answered does; an `unknown (…)` verdict (no answer, an error, a malformed
+/// list) leaves the field to go unknown at its bound.
+pub fn stamps_field(v: &Verdict) -> bool {
+    !v.line.starts_with("unknown")
+}
+
 #[cfg(test)]
 mod tests {
+    #[test]
+    fn t3708_only_an_answered_list_stamps_the_field() {
+        let me = "ed25519:mine";
+        let unanswered = super::verdict("fx", "i1", me, Err((0, "hub unreachable: refused")));
+        assert!(!super::stamps_field(&unanswered), "{}", unanswered.line);
+        let odd = super::verdict("fx", "i1", me, Ok(&serde_json::json!({"nothing": 1})));
+        assert!(!super::stamps_field(&odd), "{}", odd.line);
+        let own = serde_json::json!({"bound": null, "consumers": [
+            {"agent": "fx", "instance": "i1", "strength": "proven", "key_fp": me, "actions": ["read"], "first_ms": 1, "last_ms": 2}]});
+        let one = super::verdict("fx", "i1", me, Ok(&own));
+        assert_eq!(one.line, "one (proven)");
+        assert!(super::stamps_field(&one));
+    }
+
     use super::*;
 
     const FP: &str = "ed25519:mine";
diff --git a/crates/termlink-sidecar/src/daemon.rs b/crates/termlink-sidecar/src/daemon.rs
index 5ef419a09..caf473b78 100644
--- a/crates/termlink-sidecar/src/daemon.rs
+++ b/crates/termlink-sidecar/src/daemon.rs
@@ -286,6 +286,8 @@ pub fn start(cfg: &Config, store_root: &Path, runtime: &Path, home: &Path) -> Re
     // B-16 (T-3697): the doorbell and its watchdog, OFF until `<store>/doorbell-on` exists.
     let doorbell = Arc::new(crate::doorbell::Runner::new(&cfg.agent, store.clone(), deliverer.clone(), meter.clone(), rpc.clone(), Some(lease_shared.clone())));
     api_cfg.doorbell = Some(doorbell.clone());
+    // T-3708 (V-5): the NTP state, sampled in the background so `status` never runs a command.
+    crate::host::start_ntp_sampler();
     let api = Api::start(api_cfg, store.clone()).map_err(|e| format!("api: {e}"))?;
     let liveness = match liveness_monitor {
         None => None,
@@ -406,7 +408,14 @@ impl Running {
                 }
             }
         }
-        crate::beat::write(self.store.dir(), now, &beats, after)
+        // T-3708 (E-59): look at both clocks every tick, so a wall jump is seen even when no deadline runs
+        crate::fields::look();
+        let r = crate::beat::write(self.store.dir(), now, &beats, after);
+        // T-3708 (FCW-120): `supervision` is fresh only while the heartbeat is being written
+        if r.is_ok() {
+            crate::fields::touch(crate::fields::HEARTBEAT);
+        }
+        r
     }
 }
 
diff --git a/crates/termlink-sidecar/src/deliver.rs b/crates/termlink-sidecar/src/deliver.rs
index 8374e2787..35a44aa90 100644
--- a/crates/termlink-sidecar/src/deliver.rs
+++ b/crates/termlink-sidecar/src/deliver.rs
@@ -325,6 +325,7 @@ impl Deliverer {
     /// Record that `session` is live and can take mail (SessionStart, Stop).
     /// PH-1 only records it; the PH-2 doorbell reads it.
     pub fn mark_ready(&self, session: &str, now_ms: u64) -> Result<(), String> {
+        crate::fields::register(now_ms);
         self.store.write_state(&format!("ready-{session}"), &now_ms.to_string()).map_err(|e| e.to_string())
     }
 
@@ -360,6 +361,11 @@ impl Deliverer {
             .find(|p| p.is_file())
     }
 
+    /// T-3708 (V-8): ids of the messages that may be offered now (stored, not handed over).
+    pub fn pending_ids(&self) -> Vec<String> {
+        self.candidates().into_iter().map(|c| c.msg_id).collect()
+    }
+
     /// Everything that may be offered now, in delivery order (urgent first, then
     /// oldest first).
     fn candidates(&self) -> Vec<Candidate> {
@@ -513,8 +519,10 @@ impl Deliverer {
                 .and_then(|ls| ls.into_iter().find(|l| l.event == "STORED" && own(l) && l.msg_id == c.msg_id))
                 .and_then(|l| l.ts_ms)
                 .unwrap_or(now_ms);
-            let start = stored_at.max(ready_at);
-            if now_ms.saturating_sub(start) > d.handover(c.assessment.urgent) {
+            // T-3708 (E-59, r2 MAJOR 2): the later of storing and readiness is the younger of the two jump-corrected
+            // ages; ordering the two wall stamps directly is wrong across a backward jump
+            let age = crate::fields::age_ms(stored_at, now_ms).min(crate::fields::age_ms(ready_at, now_ms));
+            if age > d.handover(c.assessment.urgent) {
                 notify(&c.conv, &c.msg, &c.digest, OVERDUE_HANDOVER, c.assessment.urgent);
             }
         }
@@ -533,7 +541,7 @@ impl Deliverer {
                 let Ok(msg) = wire::parse_payload(&payload) else { continue };
                 let urgent = self.policy.assess(&msg).urgent;
                 let handed_at = h.ts_ms.unwrap_or(now_ms);
-                if now_ms.saturating_sub(handed_at) > d.answer(urgent) {
+                if crate::fields::age_ms(handed_at, now_ms) > d.answer(urgent) {
                     notify(&conv, &msg, &st.digest, OVERDUE_ANSWER, urgent);
                 }
             }
diff --git a/crates/termlink-sidecar/src/fields.rs b/crates/termlink-sidecar/src/fields.rs
new file mode 100644
index 000000000..3f446fc54
--- /dev/null
+++ b/crates/termlink-sidecar/src/fields.rs
@@ -0,0 +1,468 @@
+//! T-3708 (P-24 slice a; 06a AL-29, arch 6.7; design `docs/reports/T-3708-design.md` 3.1-3.3).
+//!
+//! Honest status: every field carries the time its producer last succeeded and a bound; past the
+//! bound it reads `unknown (last at T)`, never its last value (FCW-120). CLEAR is computed from the
+//! fields and from real hand-overs, never from a heartbeat (FCW-121). Freshness is measured on the
+//! monotonic clock; the wall time is shown only. The registry lives in memory, so after a restart
+//! every measured field is unknown until its producer has run again (06a 2.13.f).
+//!
+//! Time (r1 BLOCKER): every stored or sent stamp stays wall time (`store::now_ms`). A deadline is an
+//! AGE, and [`Registry::age_ms`] corrects the wall age of a stamp by every wall jump this process saw
+//! after the stamp (a jump = wall and monotonic time disagreeing by more than 2 s between two looks,
+//! taken at least every heartbeat tick), so a wall jump moves no deadline (E-59). Without a jump the
+//! age is plain `now - stamp`, as before.
+//!
+//! Additive only (design 3.0): this module adds `fields` and `clear` to `status`; it changes no
+//! existing key. `TERMLINK_SIDECAR_P24=off|0|false|no` hides them (rollback).
+
+use serde_json::{Value, json};
+use std::collections::BTreeMap;
+use std::sync::{Arc, Mutex, OnceLock};
+use std::time::Instant;
+
+use crate::deadline::TICK_MS;
+use crate::store::Store;
+
+/// Producer names the registry stamps.
+pub const PULL: &str = "pull";
+pub const REVOCATION: &str = "revocation";
+pub const CONSUMERS: &str = "consumers";
+pub const HEARTBEAT: &str = "heartbeat";
+pub const LIVENESS: &str = "liveness";
+pub const LEASE: &str = "lease";
+pub const SKEW: &str = "skew";
+pub const NTP: &str = "ntp";
+
+/// Bounds (06a 2.13.c): per-tick fields 2 ticks, heartbeat 60 s, daily-derived 1 h.
+pub const BOUND_TICKS_MS: u64 = 2 * TICK_MS;
+pub const BOUND_HEARTBEAT_MS: u64 = 60_000;
+pub const BOUND_DAILY_MS: u64 = 3_600_000;
+/// V-8: a pending message is overdue for surfacing 2 ticks after READY or a hook call that missed it.
+pub const SURFACE_AFTER_MS: u64 = 2 * TICK_MS;
+/// A component heartbeat older than this is a bad field.
+pub const COMPONENT_BAD_MS: u64 = 3 * TICK_MS;
+/// CLEAR looks back this far for mail that was pending.
+pub const CLEAR_WINDOW_MS: u64 = 3_600_000;
+/// Wall and monotonic time disagreeing by more than this between two looks is a wall jump.
+pub const JUMP_MS: i64 = 2_000;
+/// At most this many jumps are kept (oldest first out).
+pub const JUMPS_CAP: usize = 64;
+/// At most this many stamps are remembered (lowest first out).
+pub const AGES_CAP: usize = 20_000;
+
+/// The value `quota` reads until receiver quotas exist (T-3708 D-2, carried provisionally: see the task).
+pub const QUOTA_NOT_BUILT: &str = "absent: receiver quotas not built (P-17)";
+
+/// The switch for the new keys (design 3.0 rollback).
+pub const SWITCH_ENV: &str = "TERMLINK_SIDECAR_P24";
+
+/// Whether the P-24 keys are shown (default on; `off`, `0`, `false`, `no` hide them, any case).
+pub fn enabled() -> bool {
+    match std::env::var(SWITCH_ENV) {
+        Ok(v) => !matches!(v.trim().to_ascii_lowercase().as_str(), "off" | "0" | "false" | "no"),
+        Err(_) => true,
+    }
+}
+
+/// Time source: monotonic for freshness and ages, wall for stamps and display.
+pub trait Clock: Send + Sync {
+    fn mono_ms(&self) -> u64;
+    fn wall_ms(&self) -> u64;
+}
+
+/// The process clock.
+pub struct SystemClock {
+    start: Instant,
+}
+
+impl SystemClock {
+    pub fn new() -> Self {
+        SystemClock { start: Instant::now() }
+    }
+}
+
+impl Default for SystemClock {
+    fn default() -> Self {
+        Self::new()
+    }
+}
+
+/// CLOCK_BOOTTIME in ms: monotonic, and it keeps counting through a suspend or VM pause (r2 MAJOR 3), which
+/// `Instant` (CLOCK_MONOTONIC on Linux) does not. Falls back to `Instant` if the call fails.
+fn boottime_ms() -> Option<u64> {
+    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
+    // SAFETY: clock_gettime writes into the timespec we own; CLOCK_BOOTTIME is a valid clock id on Linux.
+    let rc = unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut ts) };
+    (rc == 0).then(|| ts.tv_sec as u64 * 1000 + ts.tv_nsec as u64 / 1_000_000)
+}
+
+impl Clock for SystemClock {
+    fn mono_ms(&self) -> u64 {
+        boottime_ms().unwrap_or_else(|| self.start.elapsed().as_millis() as u64)
+    }
+    fn wall_ms(&self) -> u64 {
+        crate::store::wall_now_ms()
+    }
+}
+
+/// When a producer last succeeded: monotonic and wall time.
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub struct Seen {
+    pub mono_ms: u64,
+    pub wall_ms: u64,
+}
+
+/// The in-memory registry of producer successes and of the ages of stamps this process has seen.
+pub struct Registry {
+    clock: Arc<dyn Clock>,
+    seen: Mutex<BTreeMap<String, Seen>>,
+    /// The last look at both clocks (mono, wall), and the wall jumps seen: (wall just before, delta).
+    last_look: Mutex<Option<(u64, u64)>>,
+    jumps: Mutex<Vec<(u64, i64)>>,
+    /// stamp -> how many jumps had been seen when this process WROTE it ([`Registry::register`], r2 MAJOR 1).
+    /// A stamp this process did not write (from before its start) predates all its jumps: epoch 0. Wall
+    /// values alone cannot say which side of a backward jump a stamp is on.
+    epochs: Mutex<BTreeMap<u64, usize>>,
+}
+
+impl Registry {
+    pub fn new(clock: Arc<dyn Clock>) -> Self {
+        Registry { clock, seen: Mutex::new(BTreeMap::new()), last_look: Mutex::new(None), jumps: Mutex::new(vec![]), epochs: Mutex::new(BTreeMap::new()) }
+    }
+
+    pub fn clock(&self) -> &dyn Clock {
+        &*self.clock
+    }
+
+    /// A producer succeeded now.
+    pub fn touch(&self, name: &str) {
+        let s = Seen { mono_ms: self.clock.mono_ms(), wall_ms: self.clock.wall_ms() };
+        lock(&self.seen).insert(name.to_string(), s);
+    }
+
+    pub fn last(&self, name: &str) -> Option<Seen> {
+        lock(&self.seen).get(name).copied()
+    }
+
+    /// Look at both clocks; a disagreement past [`JUMP_MS`] since the last look is a wall jump. Both clocks
+    /// are read under the lock, so two threads cannot record a false jump between them (r2 MINOR 7).
+    pub fn look(&self) {
+        let mut last = lock(&self.last_look);
+        let (mono, wall) = (self.clock.mono_ms(), self.clock.wall_ms());
+        if let Some((m0, w0)) = *last {
+            let delta = (wall as i64 - w0 as i64) - (mono as i64 - m0 as i64);
+            if delta.abs() > JUMP_MS {
+                let mut j = lock(&self.jumps);
+                if j.len() >= JUMPS_CAP {
+                    // dropping the oldest jump shifts every index down by one
+                    j.remove(0);
+                    for e in lock(&self.epochs).values_mut() {
+                        *e = e.saturating_sub(1);
+                    }
+                }
+                j.push((w0 + mono.saturating_sub(m0), delta));
+            }
+        }
+        *last = Some((mono, wall));
+    }
+
+    /// This process wrote `stamp` now: remember how many jumps came before it.
+    pub fn register(&self, stamp_wall: u64) {
+        self.look();
+        let n = lock(&self.jumps).len();
+        let mut e = lock(&self.epochs);
+        if !e.contains_key(&stamp_wall) && e.len() >= AGES_CAP {
+            if let Some(k) = e.keys().next().copied() {
+                e.remove(&k);
+            }
+        }
+        e.entry(stamp_wall).or_insert(n);
+    }
+
+    /// The age of a wall stamp at `now_wall`, less every wall jump that came after the stamp was written
+    /// (E-59). A jump while the process was down is not seen here; the skew line shows it.
+    pub fn age_ms(&self, stamp_wall: u64, now_wall: u64) -> u64 {
+        self.look();
+        let jumps = lock(&self.jumps);
+        let epoch = lock(&self.epochs).get(&stamp_wall).copied().unwrap_or(0);
+        let correction: i64 = jumps.iter().skip(epoch).map(|(_, d)| *d).sum();
+        (now_wall as i64 - stamp_wall as i64 - correction).max(0) as u64
+    }
+
+    #[cfg(test)]
+    pub(crate) fn jumps_len(&self) -> usize {
+        lock(&self.jumps).len()
+    }
+}
+
+fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
+    match m.lock() {
+        Ok(g) => g,
+        Err(p) => p.into_inner(),
+    }
+}
+
+/// The process registry (system clock).
+pub fn global() -> &'static Registry {
+    static R: OnceLock<Registry> = OnceLock::new();
+    R.get_or_init(|| Registry::new(Arc::new(SystemClock::new())))
+}
+
+/// Stamp a producer success in the process registry.
+pub fn touch(name: &str) {
+    global().touch(name);
+}
+
+/// Age of a wall stamp in the process registry (see [`Registry::age_ms`]).
+pub fn age_ms(stamp_wall: u64, now_wall: u64) -> u64 {
+    global().age_ms(stamp_wall, now_wall)
+}
+
+/// Register a stamp this process writes (chain events, readiness, hook misses).
+pub fn register(stamp_wall: u64) {
+    global().register(stamp_wall);
+}
+
+/// The wall time now, registered as a stamp this process writes.
+pub fn stamp_now() -> u64 {
+    let t = crate::store::now_ms();
+    register(t);
+    t
+}
+
+/// Look at the clocks (called every heartbeat tick, so a jump is seen even when no age is asked for).
+pub fn look() {
+    global().look();
+}
+
+/// Render one measured field: fresh with its times, or `unknown (last at T)` past the bound (FCW-120).
+pub fn render(value: &Value, seen: Option<Seen>, bound_ms: u64, now_mono: u64) -> Value {
+    match seen {
+        Some(s) if now_mono.saturating_sub(s.mono_ms) <= bound_ms => {
+            json!({"state": "fresh", "value": value, "at_ms": s.wall_ms, "bound_ms": bound_ms})
+        }
+        Some(s) => json!({"state": "unknown", "shown": format!("unknown (last at {})", s.wall_ms), "last_at_ms": s.wall_ms, "bound_ms": bound_ms}),
+        None => json!({"state": "unknown", "shown": "unknown (not measured since start)", "last_at_ms": null, "bound_ms": bound_ms}),
+    }
+}
+
+/// A field computed at this call from local state: fresh by construction.
+pub fn computed(value: &Value, now_wall: u64) -> Value {
+    json!({"state": "fresh", "value": value, "at_ms": now_wall, "bound_ms": 0})
+}
+
+/// One chain row as `status` lists it (the subset V-8 and CLEAR read). Times are this side's own
+/// stamps (`ts_ms`), never a peer's (r1 MAJOR 6).
+#[derive(Debug, Clone)]
+pub struct Row {
+    pub msg_id: String,
+    pub stage: String,
+    pub ts_ms: u64,
+}
+
+/// Rows of `status.rows` as [`Row`]s. A row without its own time is aged as the oldest possible (0),
+/// which can only make it look more overdue, never less (r1 MINOR 13).
+pub fn rows_of(status_rows: &Value) -> Vec<Row> {
+    status_rows
+        .as_array()
+        .into_iter()
+        .flatten()
+        .filter_map(|r| {
+            Some(Row {
+                msg_id: r.get("msg_id")?.as_str()?.to_string(),
+                stage: r.get("stage")?.as_str()?.to_string(),
+                ts_ms: r.get("ts_ms").and_then(Value::as_u64).unwrap_or(0),
+            })
+        })
+        .collect()
+}
+
+/// Messages this agent received and stored but has not handed over: (msg_id, stored at). A message
+/// this side SENT is never pending here. `deliverable`, when given, narrows to the messages the
+/// deliver path may still offer (a held, refused or otherwise closed message is not pending).
+pub fn pending(rows: &[Row], deliverable: Option<&[String]>) -> Vec<(String, u64)> {
+    let mut ids: Vec<&str> = rows.iter().filter(|r| r.stage == "STORED").map(|r| r.msg_id.as_str()).collect();
+    ids.sort();
+    ids.dedup();
+    ids.into_iter()
+        .filter(|id| !rows.iter().any(|r| r.msg_id == *id && (r.stage == "HANDED_OVER" || r.stage == "SENT")))
+        .filter(|id| deliverable.is_none_or(|d| d.iter().any(|x| x == id)))
+        .filter_map(|id| rows.iter().filter(|r| r.msg_id == id && r.stage == "STORED").map(|r| r.ts_ms).min().map(|t| (id.to_string(), t)))
+        .collect()
+}
+
+/// Times of hand-overs this side made: HANDED_OVER lines of messages it received (a HANDED_OVER of a
+/// message it SENT is the recipient's, reported back, and is not this agent's surfacing). A
+/// HANDED_OVER line is written only with hand-over evidence (an unconfirmed attempt is a different
+/// stage), so the line is the evidence.
+fn own_handovers(rows: &[Row]) -> impl Iterator<Item = u64> + '_ {
+    rows.iter()
+        .filter(|r| r.stage == "HANDED_OVER" && !rows.iter().any(|s| s.msg_id == r.msg_id && s.stage == "SENT"))
+        .map(|r| r.ts_ms)
+}
+
+/// V-8 hook misses, persisted (r1 MAJOR 7): the first hook call that ran while a message was pending.
+const HOOK_MISSED_STATE: &str = "v8-hook-missed";
+
+/// A hook call ran now while `pending` was pending: keep the first time per message, drop the rest.
+pub fn record_hook_call(store: &Store, pending: &[String], now_wall: u64) {
+    let old = hook_missed(store);
+    let mut m: BTreeMap<String, u64> = old.iter().filter(|(k, _)| pending.contains(k)).map(|(k, v)| (k.clone(), *v)).collect();
+    for id in pending {
+        m.entry(id.clone()).or_insert_with(|| {
+            register(now_wall);
+            now_wall
+        });
+    }
+    if m != old {
+        let text: String = m.iter().map(|(k, t)| format!("{k} {t}\n")).collect();
+        if let Err(e) = store.write_state(HOOK_MISSED_STATE, &text) {
+            eprintln!("termlink-sidecar: V-8 hook-miss record not written: {e}");
+        }
+    }
+}
+
+/// msg_id -> wall time of the first hook call that missed it.
+pub fn hook_missed(store: &Store) -> BTreeMap<String, u64> {
+    store
+        .read_state(HOOK_MISSED_STATE)
+        .ok()
+        .flatten()
+        .unwrap_or_default()
+        .lines()
+        .filter_map(|l| {
+            let (k, t) = l.split_once(' ')?;
+            Some((k.to_string(), t.trim().parse().ok()?))
+        })
+        .collect()
+}
+
+/// V-8 (06a 2.13.c, arch 6.7.3): the last real surfacing and whether it is bad. Bad iff a pending
+/// message was not handed over 2 ticks after the agent became ready (counted from the later of storing
+/// and readiness, as R-44.e) or 2 ticks after a hook call that missed it. `age(stamp)` gives the
+/// jump-immune age of a wall stamp. An idle agent with nothing pending is never bad; its age is shown.
+pub fn last_surface(
+    rows: &[Row],
+    pending: &[(String, u64)],
+    ready_since: Option<u64>,
+    hook_missed: &BTreeMap<String, u64>,
+    age: &dyn Fn(u64) -> u64,
+) -> Value {
+    let last = own_handovers(rows).max();
+    let mut overdue = vec![];
+    let mut unanchored = vec![];
+    for (id, stored) in pending {
+        // the later of storing and readiness is the younger of the two ages
+        let by_ready = ready_since.map(|r| age(*stored).min(age(r))).filter(|a| *a > SURFACE_AFTER_MS);
+        let by_hook = hook_missed.get(id).map(|t| age(*t)).filter(|a| *a > SURFACE_AFTER_MS);
+        if let Some(a) = by_ready.or(by_hook) {
+            overdue.push(json!({"msg_id": id, "overdue_by_ms": a - SURFACE_AFTER_MS, "by": if by_ready.is_some() { "ready" } else { "hook call" }}));
+        } else if ready_since.is_none() && !hook_missed.contains_key(id) && age(*stored) > SURFACE_AFTER_MS {
+            // r2 MAJOR 4: pending mail with no READY and no hook call to measure from: surfacing cannot be judged
+            unanchored.push(json!(id));
+        }
+    }
+    json!({
+        "last_surface_ms": last,
+        "age_ms": last.map(age),
+        "bad": !overdue.is_empty(),
+        "overdue": overdue,
+        "unknown": !unanchored.is_empty(),
+        "unanchored": unanchored,
+    })
+}
+
+/// The revocation status forms (T-3703 design 5.5), as CLEAR reads them: ok, unknown or bad. Any other
+/// line is bad: a form this code does not know is never read as fine (r1 MAJOR 5).
+pub fn revocation_reading(line: &str) -> &'static str {
+    if line.starts_with("verified seq ") || line == "off (PH-1 trust)" {
+        "ok"
+    } else if line.starts_with("unverified since ") || line == "hub predates revocation list" {
+        "unknown"
+    } else {
+        "bad"
+    }
+}
+
+/// CLEAR (FCW-121): no field bad, no field unknown, and if any mail was pending in the last hour (still
+/// pending now at any age, or handed over within the hour) a hand-over this side made after the
+/// earliest such STORED. Returns `{state, bad, unknown, why}`.
+pub fn clear(fields: &serde_json::Map<String, Value>, rows: &[Row], pending_now: &[(String, u64)], now_wall: u64) -> Value {
+    let mut bad: Vec<String> = vec![];
+    let mut unknown: Vec<String> = vec![];
+    for (name, f) in fields {
+        if f["state"] != "fresh" {
+            unknown.push(name.clone());
+            continue;
+        }
+        let v = &f["value"];
+        match name.as_str() {
+            "hub" if v["match"] != Value::Bool(true) => bad.push("hub: does not match the declared hub".into()),
+            "revocation" => {
+                let s = v["revocation"].as_str().unwrap_or("(no line)");
+                match revocation_reading(s) {
+                    "ok" => {}
+                    "unknown" => unknown.push("revocation".into()),
+                    _ => bad.push(format!("revocation: {s}")),
+                }
+            }
+            "consumers" => {
+                let s = v["consumers"].as_str().unwrap_or("(no line)");
+                if s.starts_with("unknown") {
+                    unknown.push("consumers".into());
+                } else if s != "one (proven)" {
+                    bad.push(format!("consumers: {s}"));
+                }
+            }
+            "supervision" => {
+                if v.get("heartbeat").is_some() || !v["heartbeat_age_ms"].is_u64() {
+                    unknown.push("supervision".into());
+                } else {
+                    if v["stale"].as_array().is_some_and(|a| !a.is_empty()) || !v["stale"].is_array() {
+                        bad.push(format!("supervision: dead or unreadable component list: {}", v["stale"]));
+                    }
+                    if v["heartbeat_age_ms"].as_u64().unwrap_or(u64::MAX) > COMPONENT_BAD_MS {
+                        bad.push("supervision: heartbeat older than 3 ticks".into());
+                    }
+                    let old: Vec<String> = v["component_age_ms"]
+                        .as_object()
+                        .map(|m| m.iter().filter(|(_, a)| a.as_u64().unwrap_or(u64::MAX) > COMPONENT_BAD_MS).map(|(k, _)| k.clone()).collect())
+                        .unwrap_or_else(|| vec!["(no component ages)".into()]);
+                    if !old.is_empty() {
+                        bad.push(format!("supervision: component heartbeat older than 3 ticks: {}", old.join(", ")));
+                    }
+                }
+            }
+            "last_surface" if v["bad"] == Value::Bool(true) => bad.push("last_surface: pending mail not handed over".into()),
+            "last_surface" if v["unknown"] == Value::Bool(true) => {
+                unknown.push("last_surface (pending mail with no READY or hook call to measure from)".into())
+            }
+            "revocation_invariants" if v["broken"].as_array().is_some_and(|a| !a.is_empty()) => {
+                bad.push(format!("revocation_invariants: {}", v["broken"]))
+            }
+            _ => {}
+        }
+    }
+    let since = now_wall.saturating_sub(CLEAR_WINDOW_MS);
+    let handed_in_window: Vec<u64> = rows
+        .iter()
+        .filter(|r| r.stage == "HANDED_OVER" && r.ts_ms >= since && !rows.iter().any(|s| s.msg_id == r.msg_id && s.stage == "SENT"))
+        .filter_map(|h| rows.iter().filter(|r| r.msg_id == h.msg_id && r.stage == "STORED").map(|r| r.ts_ms).min())
+        .collect();
+    let earliest = pending_now.iter().map(|(_, t)| *t).chain(handed_in_window).min();
+    let surfaced = earliest.is_none_or(|e| own_handovers(rows).any(|t| t >= e));
+    if !surfaced {
+        bad.push("no hand-over after mail that was pending in the last hour".into());
+    }
+    let state = if bad.is_empty() && unknown.is_empty() { "CLEAR" } else { "NOT CLEAR" };
+    let why = match (bad.first(), unknown.first()) {
+        (Some(b), _) => b.clone(),
+        (None, Some(u)) => format!("{u}: unknown"),
+        (None, None) => "every field fresh and good; mail surfaced".into(),
+    };
+    json!({"state": state, "bad": bad, "unknown": unknown, "why": why})
+}
+
+#[cfg(test)]
+#[path = "fields_tests.rs"]
+mod tests;
diff --git a/crates/termlink-sidecar/src/host.rs b/crates/termlink-sidecar/src/host.rs
new file mode 100644
index 000000000..aed5e0fb5
--- /dev/null
+++ b/crates/termlink-sidecar/src/host.rs
@@ -0,0 +1,338 @@
+//! T-3708 (P-24 slice a; 06a AL-29; design 3.4-3.5): what the host really provides.
+//!
+//! * `strength_host` (FCW-123): computed from `/proc` at each status call; a flag that cannot be
+//!   computed reads as absent, never present.
+//! * Clock skew (V-5): estimated from hub round trips (`hub_now_ms` in `channel.subscribe` replies);
+//!   "clock skew" above 2 s, cross-host delay figures "unusable" above 30 s.
+//! * NTP state: sampled in the background (timedatectl, else chronyc, else `unchecked`); status never
+//!   runs a command.
+
+use serde_json::{Value, json};
+use std::collections::{BTreeMap, BTreeSet, VecDeque};
+use std::path::Path;
+use std::sync::{Mutex, OnceLock};
+use std::time::Duration;
+
+pub const SKEW_FLAG_MS: i64 = 2_000;
+pub const SKEW_UNUSABLE_MS: i64 = 30_000;
+const SKEW_SAMPLES: usize = 5;
+/// How often the NTP state is sampled (its field bound is 1 h).
+pub const NTP_EVERY: Duration = Duration::from_secs(1800);
+
+/// One running sidecar found in `/proc`: its agent name and owner uid.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Proc {
+    pub agent: String,
+    pub uid: u32,
+}
+
+/// The agent a sidecar process serves, from its command line: the Rust sidecar
+/// (`termlink sidecar start --config …/<agent>.json`) or the framework one (`--agent <agent>`).
+pub fn sidecar_agent(argv: &[String]) -> Option<String> {
+    let s: Vec<&str> = argv.iter().map(String::as_str).collect();
+    if let Some(i) = s.iter().position(|a| *a == "--agent") {
+        if s.iter().any(|a| a.ends_with("http_server.py")) {
+            return s.get(i + 1).map(|a| a.to_string());
+        }
+    }
+    if s.windows(2).any(|w| w == ["sidecar", "start"]) {
+        let i = s.iter().position(|a| *a == "--config")?;
+        let p = Path::new(s.get(i + 1)?);
+        return p.file_stem().and_then(|f| f.to_str()).map(str::to_string);
+    }
+    None
+}
+
+/// Every sidecar process under `proc_root`; `None` when `/proc` cannot be read at all.
+pub fn sidecars(proc_root: &Path) -> Option<Vec<Proc>> {
+    let rd = std::fs::read_dir(proc_root).ok()?;
+    let mut out = vec![];
+    for e in rd.flatten() {
+        let name = e.file_name();
+        let Some(pid) = name.to_str().filter(|n| n.bytes().all(|b| b.is_ascii_digit())) else { continue };
+        let dir = proc_root.join(pid);
+        let Ok(raw) = std::fs::read(dir.join("cmdline")) else { continue };
+        let argv: Vec<String> = raw.split(|b| *b == 0).filter(|a| !a.is_empty()).map(|a| String::from_utf8_lossy(a).into_owned()).collect();
+        let Some(agent) = sidecar_agent(&argv) else { continue };
+        let Some(uid) = std::fs::read_to_string(dir.join("status")).ok().and_then(|s| {
+            s.lines().find_map(|l| l.strip_prefix("Uid:").and_then(|r| r.split_whitespace().nth(1)).and_then(|u| u.parse().ok()))
+        }) else {
+            continue;
+        };
+        out.push(Proc { agent, uid });
+    }
+    Some(out)
+}
+
+/// The strength flags as the host shows them (arch 6.7.1; E-58). `procs` is `None` when the process
+/// table could not be read; `own_uid` is this sidecar's effective uid.
+pub fn strength(procs: Option<&[Proc]>, own_uid: u32) -> Value {
+    // r1 MAJOR 3: the agent sessions' accounts cannot be read from here (only the sidecars'), so
+    // per-agent accounts are never claimed in PH-2; the flag names why it is absent (FCW-123).
+    let (per_agent, shared) = match procs {
+        None => ("no (could not check)".to_string(), None),
+        Some(ps) => {
+            let mut by_agent: BTreeMap<&str, BTreeSet<u32>> = BTreeMap::new();
+            for p in ps {
+                by_agent.entry(p.agent.as_str()).or_default().insert(p.uid);
+            }
+            let uids: BTreeSet<u32> = ps.iter().map(|p| p.uid).collect();
+            if by_agent.len() < 2 {
+                ("no (fewer than two sidecars visible: nothing to compare)".to_string(), None)
+            } else if uids.len() < by_agent.len() || uids.contains(&0) {
+                (format!("no: {} agents run under {} account(s)", by_agent.len(), uids.len()), Some(true))
+            } else {
+                ("no (sidecars run under separate accounts; the agent sessions' accounts are not checked)".to_string(), Some(false))
+            }
+        }
+    };
+    let same_user = if own_uid == 0 {
+        "absent: agents run as root".to_string()
+    } else {
+        match shared {
+            Some(true) => format!("absent: agents share accounts (this sidecar runs as uid {own_uid})"),
+            _ => "absent (could not check)".to_string(),
+        }
+    };
+    json!({
+        "per_agent_accounts": per_agent,
+        "same_user_separation": same_user,
+        // Not built in PH-2: no operator key, no confirmation provider, no policy records exist to find.
+        "operator_key": "no",
+        "confirmation_provider": "no",
+        "policy_records": "none (fixed defaults)",
+    })
+}
+
+/// `strength_host` for this process.
+pub fn strength_now() -> Value {
+    // SAFETY: geteuid has no preconditions.
+    let uid = unsafe { libc::geteuid() };
+    let procs = sidecars(Path::new("/proc"));
+    strength(procs.as_deref(), uid)
+}
+
+/// The skew estimator: the median offset (hub − local midpoint) of the last few round trips.
+#[derive(Default)]
+pub struct Skew {
+    samples: VecDeque<(i64, u64)>,
+}
+
+impl Skew {
+    /// One round trip: local wall before and after the call, and the hub's `hub_now_ms`.
+    pub fn sample(&mut self, before_ms: u64, after_ms: u64, hub_now_ms: u64) {
+        let mid = before_ms / 2 + after_ms / 2;
+        let offset = hub_now_ms as i64 - mid as i64;
+        self.samples.push_back((offset, after_ms.saturating_sub(before_ms)));
+        while self.samples.len() > SKEW_SAMPLES {
+            self.samples.pop_front();
+        }
+    }
+
+    /// `{offset_ms, rtt_ms, clock, cross_host_delays}`, or `None` before any sample.
+    pub fn view(&self) -> Option<Value> {
+        if self.samples.is_empty() {
+            return None;
+        }
+        let mut o: Vec<i64> = self.samples.iter().map(|s| s.0).collect();
+        o.sort();
+        let offset = o[o.len() / 2];
+        let rtt = self.samples.back().map(|s| s.1).unwrap_or(0);
+        Some(judge(offset, rtt))
+    }
+}
+
+/// The skew verdict for a median offset (V-5).
+pub fn judge(offset_ms: i64, rtt_ms: u64) -> Value {
+    let a = offset_ms.abs();
+    json!({
+        "offset_ms": offset_ms,
+        "rtt_ms": rtt_ms,
+        "clock": if a > SKEW_FLAG_MS { "clock skew" } else { "ok" },
+        "cross_host_delays": if a > SKEW_UNUSABLE_MS { "unusable" } else { "usable" },
+    })
+}
+
+fn skew_cell() -> &'static Mutex<Skew> {
+    static S: OnceLock<Mutex<Skew>> = OnceLock::new();
+    S.get_or_init(|| Mutex::new(Skew::default()))
+}
+
+/// Record a round trip in the process estimator and stamp the `skew` field.
+pub fn record_round_trip(before_ms: u64, after_ms: u64, hub_now_ms: Option<u64>) {
+    let Some(h) = hub_now_ms else { return };
+    match skew_cell().lock() {
+        Ok(mut s) => s.sample(before_ms, after_ms, h),
+        Err(p) => p.into_inner().sample(before_ms, after_ms, h),
+    }
+    crate::fields::touch(crate::fields::SKEW);
+}
+
+/// The process estimator's view.
+pub fn skew_view() -> Option<Value> {
+    match skew_cell().lock() {
+        Ok(s) => s.view(),
+        Err(p) => p.into_inner().view(),
+    }
+}
+
+/// Classify the NTP tools' output: timedatectl's `NTPSynchronized` value, else chronyc's tracking.
+pub fn ntp_from(timedatectl: Option<&str>, chronyc: Option<&str>) -> &'static str {
+    match timedatectl.map(str::trim) {
+        Some("yes") => return "synchronised (timedatectl)",
+        Some("no") => return "not synchronised (timedatectl)",
+        _ => {}
+    }
+    if let Some(c) = chronyc {
+        let leap = c.lines().find_map(|l| l.strip_prefix("Leap status").map(|r| r.trim_start_matches([' ', ':']).trim().to_string()));
+        return match leap.as_deref() {
+            Some("Normal") => "synchronised (chronyc)",
+            Some(_) => "not synchronised (chronyc)",
+            None => "unchecked",
+        };
+    }
+    "unchecked"
+}
+
+fn run_bounded(cmd: &str, args: &[&str]) -> Option<String> {
+    let mut child = std::process::Command::new(cmd)
+        .args(args)
+        .stdin(std::process::Stdio::null())
+        .stdout(std::process::Stdio::piped())
+        .stderr(std::process::Stdio::null())
+        .spawn()
+        .ok()?;
+    let deadline = std::time::Instant::now() + Duration::from_secs(2);
+    loop {
+        match child.try_wait() {
+            Ok(Some(st)) if st.success() => {
+                let mut s = String::new();
+                use std::io::Read as _;
+                child.stdout.take()?.read_to_string(&mut s).ok()?;
+                return Some(s);
+            }
+            Ok(Some(_)) => return None,
+            Ok(None) if std::time::Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
+            _ => {
+                let _ = child.kill();
+                let _ = child.wait();
+                return None;
+            }
+        }
+    }
+}
+
+fn ntp_cell() -> &'static Mutex<Option<&'static str>> {
+    static N: OnceLock<Mutex<Option<&'static str>>> = OnceLock::new();
+    N.get_or_init(|| Mutex::new(None))
+}
+
+/// Sample the NTP state now (P-23's commands, 2 s bound each) and stamp the `ntp` field.
+pub fn sample_ntp() {
+    let t = run_bounded("timedatectl", &["show", "-p", "NTPSynchronized", "--value"]);
+    let c = if t.as_deref().map(str::trim).is_some_and(|v| v == "yes" || v == "no") { None } else { run_bounded("chronyc", &["tracking"]) };
+    let state = ntp_from(t.as_deref(), c.as_deref());
+    match ntp_cell().lock() {
+        Ok(mut n) => *n = Some(state),
+        Err(p) => *p.into_inner() = Some(state),
+    }
+    crate::fields::touch(crate::fields::NTP);
+}
+
+/// The last sampled NTP state.
+pub fn ntp_view() -> Option<&'static str> {
+    match ntp_cell().lock() {
+        Ok(n) => *n,
+        Err(p) => *p.into_inner(),
+    }
+}
+
+/// Start the background NTP sampler (once per process).
+pub fn start_ntp_sampler() {
+    static STARTED: OnceLock<()> = OnceLock::new();
+    if STARTED.set(()).is_err() {
+        return;
+    }
+    let _ = std::thread::Builder::new().name("ntp-sampler".into()).spawn(|| loop {
+        sample_ntp();
+        std::thread::sleep(NTP_EVERY);
+    });
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn p(agent: &str, uid: u32) -> Proc {
+        Proc { agent: agent.into(), uid }
+    }
+
+    #[test]
+    fn root_host_reads_absent_never_present() {
+        let ps = [p("a", 0), p("b", 0)];
+        let v = strength(Some(&ps), 0);
+        assert_eq!(v["per_agent_accounts"], "no: 2 agents run under 1 account(s)");
+        assert_eq!(v["same_user_separation"], "absent: agents run as root");
+        assert_eq!(v["operator_key"], "no");
+        assert_eq!(v["confirmation_provider"], "no");
+        assert_eq!(v["policy_records"], "none (fixed defaults)");
+    }
+
+    #[test]
+    fn unreadable_process_table_reads_absent() {
+        assert_eq!(strength(None, 1000)["per_agent_accounts"], "no (could not check)");
+    }
+
+    #[test]
+    fn separate_sidecar_accounts_are_never_claimed_as_per_agent_accounts() {
+        // r1 MAJOR 3: the sessions may still share one account; nothing here can see them
+        for ps in [vec![p("a", 1001), p("b", 1002)], vec![p("a", 1001), p("a", 1003), p("b", 1002)]] {
+            let v = strength(Some(&ps), 1001);
+            assert!(v["per_agent_accounts"].as_str().unwrap().starts_with("no"), "{v}");
+            assert!(v["same_user_separation"].as_str().unwrap().starts_with("absent"), "{v}");
+        }
+        assert_eq!(strength(None, 1001)["same_user_separation"], "absent (could not check)");
+        // a root sidecar among separated ones is shared, not separated
+        assert_eq!(strength(Some(&[p("a", 0), p("b", 1001)]), 1001)["per_agent_accounts"], "no: 2 agents run under 2 account(s)");
+    }
+
+    #[test]
+    fn sidecar_agent_from_both_kinds_of_command_line() {
+        let a = |s: &str| s.split(' ').map(str::to_string).collect::<Vec<_>>();
+        assert_eq!(sidecar_agent(&a("/root/.cargo/bin/termlink sidecar start --config /etc/termlink-sidecar/claude-termlink.json")).as_deref(), Some("claude-termlink"));
+        assert_eq!(sidecar_agent(&a("/usr/bin/python3 /opt/x/lib/sidecar/http_server.py --port 0 --agent 010-termlink")).as_deref(), Some("010-termlink"));
+        assert_eq!(sidecar_agent(&a("termlink register --name x --agent y")), None);
+    }
+
+    #[test]
+    fn skew_flags_at_the_stated_thresholds() {
+        for (off, clock, delays) in [(0, "ok", "usable"), (3_000, "clock skew", "usable"), (45_000, "clock skew", "unusable"), (-45_000, "clock skew", "unusable")] {
+            let mut s = Skew::default();
+            s.sample(1_000_000, 1_000_010, (1_000_005i64 + off) as u64);
+            let v = s.view().unwrap();
+            assert_eq!(v["offset_ms"], off, "offset {off}");
+            assert_eq!(v["clock"], clock, "offset {off}");
+            assert_eq!(v["cross_host_delays"], delays, "offset {off}");
+        }
+    }
+
+    #[test]
+    fn skew_is_the_median_so_one_slow_round_trip_does_not_flag() {
+        let mut s = Skew::default();
+        for _ in 0..4 {
+            s.sample(1_000, 1_010, 1_005);
+        }
+        s.sample(1_000, 9_000, 9_000); // one outlier
+        assert_eq!(s.view().unwrap()["clock"], "ok");
+    }
+
+    #[test]
+    fn ntp_reads_unchecked_without_tools() {
+        assert_eq!(ntp_from(None, None), "unchecked");
+        assert_eq!(ntp_from(Some("yes\n"), None), "synchronised (timedatectl)");
+        assert_eq!(ntp_from(Some("no"), None), "not synchronised (timedatectl)");
+        assert_eq!(ntp_from(Some(""), Some("Reference ID : X\nLeap status     : Normal\n")), "synchronised (chronyc)");
+        assert_eq!(ntp_from(None, Some("Leap status     : Not synchronised\n")), "not synchronised (chronyc)");
+        assert_eq!(ntp_from(None, Some("garbage")), "unchecked");
+    }
+}
diff --git a/crates/termlink-sidecar/src/lease.rs b/crates/termlink-sidecar/src/lease.rs
index 359909406..ebc6056fd 100644
--- a/crates/termlink-sidecar/src/lease.rs
+++ b/crates/termlink-sidecar/src/lease.rs
@@ -163,7 +163,11 @@ pub fn spawn(client: LeaseClient, shared: Arc<Shared>, stop: Arc<AtomicBool>) ->
                 let o = rt.block_on(client.status(&project, None));
                 let at = crate::store::now_ms();
                 match o.view {
-                    Some(v) => shared.set(at, Ok(v)),
+                    Some(v) => {
+                        // T-3708 (FCW-120): the lease view is fresh from this read
+                        crate::fields::touch(crate::fields::LEASE);
+                        shared.set(at, Ok(v))
+                    }
                     None => shared.set(at, Err(o.message)),
                 }
             }
diff --git a/crates/termlink-sidecar/src/lib.rs b/crates/termlink-sidecar/src/lib.rs
index b38dece4a..f499b32ac 100644
--- a/crates/termlink-sidecar/src/lib.rs
+++ b/crates/termlink-sidecar/src/lib.rs
@@ -13,6 +13,8 @@ pub mod pull;
 pub mod hub_key_pin;
 pub mod revocation;
 pub mod consumers;
+pub mod fields;
+pub mod host;
 pub mod policy;
 pub mod frame;
 pub mod adapter;
diff --git a/crates/termlink-sidecar/src/liveness.rs b/crates/termlink-sidecar/src/liveness.rs
index 6f5ac4f5c..64a96b750 100644
--- a/crates/termlink-sidecar/src/liveness.rs
+++ b/crates/termlink-sidecar/src/liveness.rs
@@ -390,6 +390,10 @@ impl Monitor {
         let signers = |fp: &str| fp == self.own_fingerprint || self.roster.get(fp).is_some();
         let expect = Expect { agent: main, instance, signers: &signers };
         let r = read(hub, &liveness_topic(&self.hub_id, main), &inbox_topic(&self.hub_id, main));
+        // T-3708 (FCW-120): the main's liveness verdict is fresh only from a read the hub answered
+        if !matches!(r, Read::Unreadable(_)) {
+            crate::fields::touch(crate::fields::LIVENESS);
+        }
         let v = judge(&expect, &r, now, self.stale_after_ms);
         alarm(&self.store, main, &v);
         self.shared.set(now, v.clone());
diff --git a/crates/termlink-sidecar/src/policy.rs b/crates/termlink-sidecar/src/policy.rs
index 9fa227193..90dfd4586 100644
--- a/crates/termlink-sidecar/src/policy.rs
+++ b/crates/termlink-sidecar/src/policy.rs
@@ -142,7 +142,7 @@ impl Policy for BuiltIn {
             Trust::Untrusted
         };
         let asks_urgent = priority >= URGENT_AT;
-        let urgent = asks_urgent && trust == Trust::OwnProject;
+        let urgent = asks_urgent && urgent_allowed(trust);
         // T-3703 (DR-8 A, FCW-95/96): while the revocation list is stale or unverified since start, urgent is
         // allowed only from the own project's roster. Today urgent already needs the roster; this gate binds any
         // later widening of the urgent allow-list.
@@ -161,6 +161,27 @@ impl Policy for BuiltIn {
     }
 }
 
+/// The urgent allow-list: who may send urgent at all. Today the own project's roster only; widening it
+/// here also widens what the stale gate below narrows, and what [`downgrade_why`] names.
+pub fn urgent_allowed(trust: Trust) -> bool {
+    trust == Trust::OwnProject
+}
+
+/// The DOWNGRADED chain reason (SI-18). A downgrade the stale gate caused names `revocation list stale`
+/// (T-3703 r3 CC N4, carried into T-3708); any other names the allow-list.
+pub fn downgrade_why(priority: i64, trust: Trust, narrowing: bool) -> String {
+    downgrade_why_with(priority, trust, narrowing, urgent_allowed)
+}
+
+/// As [`downgrade_why`] for a given urgent allow-list (a test can widen it).
+pub fn downgrade_why_with(priority: i64, trust: Trust, narrowing: bool, allowed: fn(Trust) -> bool) -> String {
+    if allowed(trust) && !stale_gate(true, trust, narrowing) {
+        format!("priority {priority} from a {} sender: revocation list stale (urgent only from the own project until a verified pull)", trust.as_str())
+    } else {
+        format!("priority {priority} from a {} sender: urgent needs the allow-list", trust.as_str())
+    }
+}
+
 /// T-3703 (DR-8 A): while `narrowing` (the revocation list is stale or not verified since start), urgent stays
 /// urgent only for the own project's roster; anything else is downgraded to normal (never dropped).
 pub fn stale_gate(urgent: bool, trust: Trust, narrowing: bool) -> bool {
diff --git a/crates/termlink-sidecar/src/pull.rs b/crates/termlink-sidecar/src/pull.rs
index 081e63da0..441a1d34c 100644
--- a/crates/termlink-sidecar/src/pull.rs
+++ b/crates/termlink-sidecar/src/pull.rs
@@ -271,8 +271,12 @@ impl Inner {
     }
 
     fn check_pin(&self) -> Result<Value, PinError> {
+        let before = crate::store::now_ms();
         let resp = self.rpc("hub.version", json!({})).map_err(PinError::Unreachable)?;
+        let after = crate::store::now_ms();
         let v = unwrap_result(resp).map_err(PinError::Unreachable)?;
+        // T-3708 (V-5): every pass is a skew sample, also for an agent whose inbox has never had mail
+        crate::host::record_round_trip(before, after, v.get("hub_now_ms").and_then(Value::as_u64));
         let found = v.get("hub_id").and_then(Value::as_str).map(str::to_owned);
         crate::hub_key_pin::observe_hub_id(&self.store, &self.cfg.declared_hub_id, found.as_deref());
         if found.as_deref() == Some(self.cfg.declared_hub_id.as_str()) {
@@ -414,6 +418,10 @@ impl Inner {
             Err(e) => crate::consumers::verdict(me, inst, &fp, Err((0, &format!("hub unreachable: {e}")))),
         };
         crate::consumers::record(&self.store, &topic, &v, now);
+        // T-3708 (FCW-120, r1 MAJOR 4): fresh only from a check the hub answered
+        if crate::consumers::stamps_field(&v) {
+            crate::fields::touch(crate::fields::CONSUMERS);
+        }
     }
 
     fn cursor(&self) -> u64 {
@@ -453,6 +461,8 @@ impl Inner {
         let version = match self.check_pin() {
             Ok(version) => {
                 mem.held = None;
+                // T-3708 (FCW-120): the hub was read; `hub` and `revocation` are fresh from this pass.
+                crate::fields::touch(crate::fields::PULL);
                 self.observe_impersonation();
                 version
             }
@@ -481,6 +491,7 @@ impl Inner {
         let mut handled = 0;
         loop {
             let cursor = self.cursor();
+            let before = crate::store::now_ms();
             let resp = match self.rpc(
                 method::CHANNEL_SUBSCRIBE,
                 // T-3700 (AL-30, ruling E): a named, signed read of our own inbox (proven consumer)
@@ -490,6 +501,10 @@ impl Inner {
                 Ok(r) => r,
                 Err(_) => return PassResult::Waiting,
             };
+            // T-3708 (V-5): this round trip is a skew sample when the hub reports its time
+            if let RpcResponse::Success(s) = &resp {
+                crate::host::record_round_trip(before, crate::store::now_ms(), s.result.get("hub_now_ms").and_then(Value::as_u64));
+            }
             let page = match resp {
                 RpcResponse::Success(s) => s.result,
                 RpcResponse::Error(e) if e.error.code == error_code::CHANNEL_TOPIC_UNKNOWN => {
@@ -607,7 +622,7 @@ impl Inner {
         }
         if a.downgraded {
             // Asked for urgent without the allow-list: normal, and said so (SI-18).
-            let why = format!("priority {} from a {} sender: urgent needs the allow-list", a.priority, a.trust.as_str());
+            let why = policy::downgrade_why(a.priority, a.trust, crate::revocation::narrowing(&self.store));
             if let Err(e) = once(policy::DOWNGRADED, Some(&why)) {
                 return Outcome::Retry(format!("DOWNGRADED not recorded: {e}"));
             }
diff --git a/crates/termlink-sidecar/src/revocation.rs b/crates/termlink-sidecar/src/revocation.rs
index 12586cab3..5694fea51 100644
--- a/crates/termlink-sidecar/src/revocation.rs
+++ b/crates/termlink-sidecar/src/revocation.rs
@@ -111,6 +111,80 @@ pub fn revoked_from(store: &Store, from: &str) -> bool {
     held_keys(store).iter().any(|k| k.strip_prefix("ed25519:").is_some_and(|h| h.starts_with(from)))
 }
 
+/// T-3708 (E-127's reader): when this sidecar first held each key as revoked (its own time), one
+/// `<key> <ms>` line per key. A message stored before then was stored lawfully (the list was not known
+/// here yet); one stored after is a broken invariant.
+const HELD_SINCE_STATE: &str = "revocation-held-since";
+
+/// Add the keys not recorded yet, at `now`.
+pub fn record_held_since(store: &Store, keys: &BTreeSet<String>, now: u64) {
+    let mut m = held_since(store);
+    let before = m.len();
+    for k in keys {
+        m.entry(k.clone()).or_insert(now);
+    }
+    if m.len() != before {
+        let text: String = m.iter().map(|(k, t)| format!("{k} {t}\n")).collect();
+        if let Err(e) = store.write_state(HELD_SINCE_STATE, &text) {
+            // r2 MINOR 8: never fail open: E-127 then says its invariant (b) cannot be checked
+            HELD_SINCE_UNWRITTEN.store(true, std::sync::atomic::Ordering::SeqCst);
+            eprintln!("termlink-sidecar: revocation held-since record not written: {e}");
+        } else {
+            HELD_SINCE_UNWRITTEN.store(false, std::sync::atomic::Ordering::SeqCst);
+        }
+    }
+}
+
+static HELD_SINCE_UNWRITTEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
+
+/// Whether the last held-since write failed (E-127's invariant (b) then cannot be checked).
+pub fn held_since_unwritten() -> bool {
+    HELD_SINCE_UNWRITTEN.load(std::sync::atomic::Ordering::SeqCst)
+}
+
+/// Key -> the time this sidecar first held it as revoked.
+pub fn held_since(store: &Store) -> std::collections::BTreeMap<String, u64> {
+    store
+        .read_state(HELD_SINCE_STATE)
+        .ok()
+        .flatten()
+        .unwrap_or_default()
+        .lines()
+        .filter_map(|l| {
+            let (k, t) = l.split_once(' ')?;
+            Some((k.to_string(), t.trim().parse().ok()?))
+        })
+        .collect()
+}
+
+/// E-127's reader (T-3703 r1 finding 17, carried into T-3708): the invariants that must never break.
+/// (a) the list unverified or stale for longer than PN-13 while the narrowing is not applied;
+/// (b) a message from a revoked key STORED after this sidecar held that key as revoked.
+/// `stored` is (msg_id, short sender id, stored at). Returns one line per broken invariant.
+pub fn e127_broken(status_line: &str, narrowing_now: bool, held: &std::collections::BTreeMap<String, u64>, stored: &[(String, String, u64)], now: u64) -> Vec<String> {
+    let mut out = vec![];
+    let since = ["unverified since ", "stale since "]
+        .iter()
+        .find_map(|p| status_line.strip_prefix(p))
+        .and_then(|r| r.split_whitespace().next())
+        .and_then(|n| n.parse::<u64>().ok());
+    if let Some(t) = since {
+        if now.saturating_sub(t) > PN13.as_millis() as u64 && !narrowing_now {
+            out.push(format!("revocation list unverified since {t}, longer than PN-13, and the narrowing is not applied"));
+        }
+    }
+    for (id, from, at) in stored {
+        if from.len() < 16 {
+            continue;
+        }
+        let hit = held.iter().find(|(k, t)| k.strip_prefix("ed25519:").is_some_and(|h| h.starts_with(from.as_str())) && *at > **t);
+        if let Some((k, t)) = hit {
+            out.push(format!("message {id} from revoked key {k} was STORED at {at}, after this sidecar held it as revoked at {t}"));
+        }
+    }
+    out
+}
+
 /// DR-8 A: true while the list is STALE or not yet verified since start.
 pub fn narrowing(store: &Store) -> bool {
     SNAPS.lock().unwrap_or_else(|p| p.into_inner()).get(store.dir()).is_none_or(|s| s.narrowing)
@@ -282,6 +356,8 @@ impl Tracker {
             let _ = store.write_state(SWITCH_SEEN_STATE, now_s);
         }
         if off {
+            // T-3708 (FCW-120): "off (PH-1 trust)" is this pass's evaluation too; unstamped it would read unknown
+            crate::fields::touch(crate::fields::REVOCATION);
             self.state = State::Off;
             publish(store, &BTreeSet::new(), false);
             self.write_status(store, "off (PH-1 trust)");
@@ -377,10 +453,15 @@ impl Tracker {
         let _ = store.write_state(HUB_PK_STATE, &ss::hex(pk.as_bytes()));
         self.held = Some(held_of(&l));
         self.held_unverified = false;
+        if let Some(h) = &self.held {
+            record_held_since(store, &h.keys, crate::store::now_ms());
+        }
         Ok(())
     }
 
     fn verified(&mut self, store: &Store, now: Instant, sender: Option<&crate::send::Sender>) {
+        // T-3708 (FCW-120, r1 MAJOR 4): the revocation field is fresh from this tracker's own evaluation
+        crate::fields::touch(crate::fields::REVOCATION);
         self.last_verified = Some(now);
         self.since_wall = crate::store::now_ms();
         let was = std::mem::replace(&mut self.state, State::Verified);
@@ -438,6 +519,8 @@ impl Tracker {
     }
 
     fn failed(&mut self, store: &Store, now: Instant, why: &str, tamper: bool) {
+        // the status line below says why; the evaluation itself ran now
+        crate::fields::touch(crate::fields::REVOCATION);
         let hour = crate::store::now_ms() / 3_600_000;
         if tamper {
             let _ = store.attention_coalesced("revocation-tamper", &hour.to_string(), &format!("hub {}: {why}", self.hub_id));
diff --git a/crates/termlink-sidecar/src/store.rs b/crates/termlink-sidecar/src/store.rs
index c3067735c..e7124bed3 100644
--- a/crates/termlink-sidecar/src/store.rs
+++ b/crates/termlink-sidecar/src/store.rs
@@ -174,11 +174,27 @@ struct Body<'a> {
 }
 
 /// Wall-clock now in ms since the epoch.
+/// Wall time: what every stamp that is stored or sent carries (T-3708 r1 BLOCKER: it stays wall time;
+/// deadlines measure ages with [`crate::fields::age_ms`] instead, so a wall jump moves none of them).
 pub fn now_ms() -> u64 {
-    std::time::SystemTime::now()
+    wall_now_ms()
+}
+
+static WALL_SHIFT_MS: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
+
+/// The host's wall clock (plus a test shift, see [`test_shift_wall_ms`]).
+pub fn wall_now_ms() -> u64 {
+    let w = std::time::SystemTime::now()
         .duration_since(std::time::UNIX_EPOCH)
-        .map(|d| d.as_millis() as u64)
-        .unwrap_or(0)
+        .map(|d| d.as_millis() as i64)
+        .unwrap_or(0);
+    (w + WALL_SHIFT_MS.load(std::sync::atomic::Ordering::SeqCst)).max(0) as u64
+}
+
+/// Test seam (E-59): jump this process's view of the wall clock by `delta_ms`.
+#[doc(hidden)]
+pub fn test_shift_wall_ms(delta_ms: i64) {
+    WALL_SHIFT_MS.fetch_add(delta_ms, std::sync::atomic::Ordering::SeqCst);
 }
 
 fn hex(b: &[u8]) -> String {
@@ -503,7 +519,8 @@ impl Store {
             digest: digest.into(),
             prev,
             hash: String::new(),
-            ts_ms: Some(now_ms()),
+            // T-3708 (E-59, r2 MAJOR 1): registered, so a later wall jump is known to come after it
+            ts_ms: Some(crate::fields::stamp_now()),
             peer_ts_ms: extra.peer_ts_ms,
             reason: extra.reason.map(|r| {
                 let mut r = r;
diff --git a/crates/termlink-sidecar/tests/t3708_clock.rs b/crates/termlink-sidecar/tests/t3708_clock.rs
new file mode 100644
index 000000000..2a9ec5f5d
--- /dev/null
+++ b/crates/termlink-sidecar/tests/t3708_clock.rs
@@ -0,0 +1,117 @@
+//! T-3708 (E-59, r1 MAJOR 9 / BLOCKER): the deadline ages the sidecar uses do not move with a wall jump, through the
+//! real clock path (`store::test_shift_wall_ms` shifts what `store::now_ms` returns). Own binary: the shift is
+//! process-wide, so nothing else may run in this process.
+
+use termlink_sidecar::fields;
+use termlink_sidecar::store;
+
+#[test]
+fn t3708_e59_deadline_ages_ignore_wall_jumps_on_the_real_clock_path() {
+    fields::look();
+    let stamp = store::now_ms() - 10_000; // stored 10 s ago, before any jump
+    assert!((10_000..12_000).contains(&fields::age_ms(stamp, store::now_ms())));
+
+    store::test_shift_wall_ms(3_600_000); // +1 h
+    fields::look();
+    let a = fields::age_ms(stamp, store::now_ms());
+    assert!(a < 15_000, "a +1 h jump aged the stamp to {a} ms: a deadline would fire early");
+
+    store::test_shift_wall_ms(-7_200_000); // then -2 h
+    fields::look();
+    let b = fields::age_ms(stamp, store::now_ms());
+    assert!((10_000..15_000).contains(&b), "after -2 h the stamp's age is {b} ms: a deadline would fire late");
+
+    // a stamp written (registered) after the jumps is aged plainly
+    let late = fields::stamp_now();
+    assert!(fields::age_ms(late, store::now_ms()) < 2_000);
+    store::test_shift_wall_ms(3_600_000);
+    fields::look();
+
+    // the receiver's hand-over deadline (deliver.rs) across a +1 h jump: no notice fires early
+    receiver_deadline_ignores_a_forward_jump();
+}
+
+fn receiver_deadline_ignores_a_forward_jump() {
+    use std::sync::Arc;
+    use std::time::Duration;
+    use termlink_sidecar::deadline::Deadlines;
+    use termlink_sidecar::deliver::Deliverer;
+    use termlink_sidecar::policy::BuiltIn;
+    use termlink_sidecar::send::{SendConfig, Sender};
+    use termlink_sidecar::store::Store;
+    use termlink_sidecar::testkit::Estate;
+    use termlink_sidecar::wire::{Msg, Roster};
+    let Ok(e) = Estate::create() else { panic!("no test hub: E-59 deliver part not run (not a PASS)") };
+    let ka = e.identity("a").unwrap();
+    let mut roster = Roster::new();
+    roster.add(ka.verifying_key());
+    let st = Arc::new(Store::open(&e.store_root(), "agent-b").unwrap());
+    let sender = Arc::new(
+        Sender::new(
+            SendConfig {
+                agent: "agent-b".into(),
+                instance: "inst-b".into(),
+                identity: e.identity("b").unwrap(),
+                hub: e.hub_addr(),
+                declared_hub_id: e.hub_id().unwrap(),
+                queue_path: e.root().join("outbox-b.sqlite"),
+                timeout: Duration::from_secs(15),
+                pump_interval: None,
+            },
+            st.clone(),
+        )
+        .unwrap(),
+    );
+    let d = Deliverer::new("agent-b", st.clone(), Box::new(BuiltIn::new(roster)), Some(sender));
+    let m = Msg {
+        from: ka.fingerprint().to_owned(),
+        from_agent: "agent-a".into(),
+        from_instance: "inst-a".into(),
+        to_agent: "agent-b".into(),
+        to_instance: "inst-b".into(),
+        conv: "c".into(),
+        msg_id: "h1".into(),
+        seq: 0,
+        priority: "0".into(),
+        ts_ms: 1,
+        body: "b".into(),
+    };
+    st.append_event(&m.conv, "RECEIVED", &m.msg_id, &m.from, &m.digest()).unwrap();
+    st.put(&m.from, &m.conv, &m.msg_id, &m.payload_bytes()).unwrap();
+    fields::look();
+    let now = store::now_ms();
+    d.mark_ready("s1", now).unwrap();
+    assert_eq!(d.check_overdue(store::now_ms(), &Deadlines::R44E), 0);
+    store::test_shift_wall_ms(3_600_000);
+    fields::look();
+    let fired = d.check_overdue(store::now_ms(), &Deadlines::R44E);
+    store::test_shift_wall_ms(-3_600_000);
+    fields::look();
+    assert_eq!(fired, 0, "a +1 h wall jump fired the hand-over deadline early");
+
+    // r2 MAJOR 2 (the reviewer's sequence): stored, the wall goes back 1 h, the agent becomes ready 29 minutes
+    // later (on the new, earlier wall), and is checked 30 s after that. Readiness is the later event, 30 s ago:
+    // nothing may fire. Ordering the two wall stamps instead picks the storing stamp and reads ~29.5 min.
+    let m2 = Msg { msg_id: "h2".into(), ..m };
+    st.append_event(&m2.conv, "RECEIVED", &m2.msg_id, &m2.from, &m2.digest()).unwrap();
+    st.put(&m2.from, &m2.conv, &m2.msg_id, &m2.payload_bytes()).unwrap();
+    store::test_shift_wall_ms(-3_600_000);
+    fields::look();
+    let ready = store::now_ms() + 29 * 60_000;
+    d.mark_ready("s2", ready).unwrap();
+    let fired2 = d.check_overdue(ready + 30_000, &Deadlines::R44E);
+    store::test_shift_wall_ms(3_600_000);
+    fields::look();
+    assert_eq!(fired2, 0, "after a -1 h jump the hand-over deadline fired 30 s after readiness");
+
+    // r2 MAJOR 1 (mutant m5e): a chain event written AFTER a backward jump is registered with that jump, so its
+    // age is plain; an unregistered stamp would be "corrected" by the jump and read an hour old
+    store::test_shift_wall_ms(-3_600_000);
+    fields::look();
+    st.append_event("c", "NOTE", "h3", &ka.fingerprint().to_owned(), "d").unwrap();
+    let ts = st.chain_lines("c").unwrap().into_iter().rev().find(|l| l.msg_id == "h3").and_then(|l| l.ts_ms).unwrap();
+    let a = fields::age_ms(ts, store::now_ms());
+    store::test_shift_wall_ms(3_600_000);
+    fields::look();
+    assert!(a < 5_000, "a stamp written after a jump was aged as if before it: {a} ms");
+}
diff --git a/crates/termlink-sidecar/tests/t3708_revoff.rs b/crates/termlink-sidecar/tests/t3708_revoff.rs
new file mode 100644
index 000000000..c567e3b50
--- /dev/null
+++ b/crates/termlink-sidecar/tests/t3708_revoff.rs
@@ -0,0 +1,27 @@
+//! T-3708 (FCW-120): with the revocation pull switched off, the tracker's pass still stamps the `revocation`
+//! field ("off (PH-1 trust)" is a measured value). Own binary: the field registry is process-wide and must start empty.
+
+use std::time::{Duration, Instant};
+use termlink_sidecar::fields;
+use termlink_sidecar::revocation::{PN13, Tracker};
+use termlink_sidecar::store::Store;
+use termlink_sidecar::testkit::Estate;
+
+#[test]
+fn t3708_switched_off_revocation_is_still_stamped() {
+    let e = Estate::create_without_hub().unwrap();
+    let store = Store::open(&e.store_root(), "fx-b").unwrap();
+    store.write_state("revocation-pull", "off").unwrap();
+    assert!(fields::global().last(fields::REVOCATION).is_none(), "the registry starts empty in this process");
+    let mut t = Tracker::new(&store, "0123456789abcdef", PN13);
+    let rpc = |_: &str| -> Result<serde_json::Value, (i64, String)> { Err((0, "no hub in this test".into())) };
+    t.tick(&store, &serde_json::json!({}), &rpc, Instant::now(), None);
+    assert!(fields::global().last(fields::REVOCATION).is_some(), "an off pass left the revocation field unstamped");
+    let _ = Duration::ZERO;
+    // r2 MINOR 6: a failed pass stamps the field and rewrites the line, so it never reads "verified" fresh
+    store.write_state("revocation-pull", "on").unwrap();
+    t.tick(&store, &serde_json::json!({}), &rpc, Instant::now(), None);
+    let line = termlink_sidecar::revocation::status(&store)["revocation"].as_str().unwrap().to_string();
+    assert!(!line.starts_with("verified"), "a failed pull left the line as {line}");
+    assert!(termlink_sidecar::fields::revocation_reading(&line) != "ok", "{line}");
+}
```

## Live test (silent hub, E-55, E-58)

```rust
//! T-3708 (P-24 slice a) on a real hub (testkit estate): E-55 (fields stamped by their producers; p95 ≤ 5 s over
//! 100 calls with the hub up and with the hub stopped, the fault seam that exists before P-19), FCW-120 (a stopped
//! hub leaves `hub` fresh only within its bound), E-58 at L4 on this host (strength flags against `ps -o user=`).

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use termlink_sidecar::api::{Api, ApiConfig, Scope};
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::pull::{ReceiveConfig, Receiver};
use termlink_sidecar::send::{SendConfig, Sender};
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::Roster;

struct Rig {
    store: Arc<Store>,
    rx: Receiver,
    api: Api,
}

fn rig(e: &Estate) -> Rig {
    let id = e.hub_id().unwrap();
    let agent = "fx-b".to_string();
    let store = Arc::new(Store::open(&e.store_root(), &agent).unwrap());
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: agent.clone(),
                instance: "b1".into(),
                identity: e.identity("b").unwrap(),
                hub: e.hub_addr(),
                declared_hub_id: id.clone(),
                queue_path: e.root().join("outbox-b1.sqlite"),
                timeout: Duration::from_secs(5),
                pump_interval: None,
            },
            store.clone(),
        )
        .unwrap(),
    );
    let roster = Roster::new();
    let rx = Receiver::new(
        ReceiveConfig {
            agent: agent.clone(),
            instance: "b1".into(),
            hub: e.hub_addr(),
            declared_hub_id: id.clone(),
            roster: roster.clone(),
            timeout: Duration::from_secs(5),
            page: 50,
            poll_interval: None,
        },
        store.clone(),
        sender.clone(),
        Box::new(BuiltIn::new(roster)),
    )
    .unwrap();
    let mut cfg = ApiConfig::new(&agent, &e.runtime_dir());
    cfg.hub_id = Some(id);
    cfg.sender = Some(sender);
    let api = Api::start(cfg, store.clone()).unwrap();
    Rig { store, rx, api }
}

fn status(r: &Rig) -> (Value, Duration) {
    let t = Instant::now();
    let mut s = UnixStream::connect(r.api.path(Scope::Status)).unwrap();
    writeln!(s, "{}", json!({"op": "status"})).unwrap();
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).unwrap();
    (serde_json::from_str(&line).unwrap(), t.elapsed())
}

fn p95(mut d: Vec<Duration>) -> Duration {
    d.sort();
    d[(d.len() * 95).div_ceil(100) - 1]
}

#[test]
fn t3708_e55_fields_stamped_by_producers_and_fast_with_the_hub_up_or_down() {
    let Ok(mut e) = Estate::create() else {
        eprintln!("SKIP: no termlink binary for a test hub");
        return;
    };
    let r = rig(&e);
    let _ = r.rx.pass();
    let (v, _) = status(&r);
    // the pull pass read the hub: hub and revocation are fresh, with the producer's time
    for n in ["hub", "revocation", "consumers", "skew"] {
        assert_eq!(v["fields"][n]["state"], "fresh", "{n} after a pass: {}", v["fields"][n]);
    }
    assert_eq!(v["fields"]["hub"]["value"]["match"], true);
    let skew = &v["fields"]["skew"]["value"];
    assert_eq!(skew["clock"], "ok", "same host, no skew: {skew}");

    let mut up = vec![];
    for _ in 0..100 {
        up.push(status(&r).1);
    }
    e.stop_hub();
    let mut down = vec![];
    for _ in 0..100 {
        down.push(status(&r).1);
    }
    let (pu, pd) = (p95(up), p95(down));
    println!("E-55 p95 status latency: hub up {pu:?}, hub stopped {pd:?}");
    assert!(pu <= Duration::from_secs(5) && pd <= Duration::from_secs(5), "p95 up {pu:?} down {pd:?}");
    // status makes no hub call: a stopped hub costs nothing (design mutant 10)
    assert!(pd < Duration::from_millis(1500), "status waited on the stopped hub: p95 {pd:?}");
    // a pass against the stopped hub does not refresh `hub`
    let before = status(&r).0["fields"]["hub"]["at_ms"].clone();
    let _ = r.rx.pass();
    assert_eq!(status(&r).0["fields"]["hub"]["at_ms"], before, "a failed pass must not stamp the hub field");
    let lines = std::fs::read_to_string(r.store.dir().join(termlink_sidecar::api::LATENCY_LOG)).unwrap();
    assert!(lines.lines().count() >= 200, "one latency line per call");
    let _ = e.teardown();
}

/// E-58 at L4: every flag matches what the host shows (`ps -o user=` over the sidecar processes).
#[test]
fn t3708_e58_strength_flags_match_the_host() {
    let v = termlink_sidecar::host::strength_now();
    let out = std::process::Command::new("ps").args(["-eo", "user=,args="]).output().unwrap();
    let ps = String::from_utf8_lossy(&out.stdout);
    let users: std::collections::BTreeSet<&str> = ps
        .lines()
        .filter(|l| l.contains("sidecar start --config") || (l.contains("http_server.py") && l.contains("--agent")))
        .filter_map(|l| l.split_whitespace().next())
        .collect();
    let agents = ps.lines().filter(|l| l.contains("sidecar start --config") || (l.contains("http_server.py") && l.contains("--agent"))).count();
    // SAFETY: geteuid has no preconditions.
    let root = unsafe { libc::geteuid() } == 0;
    println!("E-58 host: {agents} sidecar processes under users {users:?}; flags {v}");
    let pa = v["per_agent_accounts"].as_str().unwrap();
    if users.len() < agents || users.contains("root") {
        assert!(pa.starts_with("no"), "per_agent_accounts claims strength the host lacks: {pa}");
    }
    if root {
        assert_eq!(v["same_user_separation"], "absent: agents run as root");
    }
    assert_eq!(v["operator_key"], "no");
    assert_eq!(v["confirmation_provider"], "no");
    assert_eq!(v["policy_records"], "none (fixed defaults)");
}

/// r1 MAJOR 9 (design mutant 10): a hub that accepts and never answers. Status must not wait on it.
#[test]
fn t3708_status_never_waits_on_a_hub_that_accepts_and_never_replies() {
    let e = Estate::create_without_hub().unwrap();
    let l = std::net::TcpListener::bind("<loopback>:0").unwrap();
    let addr = l.local_addr().unwrap();
    // hold every connection open without a byte back
    std::thread::spawn(move || {
        let mut held = vec![];
        for c in l.incoming().flatten() {
            held.push(c);
        }
    });
    let agent = "fx-b".to_string();
    let store = Arc::new(Store::open(&e.store_root(), &agent).unwrap());
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: agent.clone(),
                instance: "b1".into(),
                identity: e.identity("b").unwrap(),
                hub: termlink_protocol::TransportAddr::Tcp { host: addr.ip().to_string(), port: addr.port() },
                declared_hub_id: "0123456789abcdef".into(),
                queue_path: e.root().join("outbox-b1.sqlite"),
                timeout: Duration::from_secs(30),
                pump_interval: None,
            },
            store.clone(),
        )
        .unwrap(),
    );
    let mut cfg = ApiConfig::new(&agent, &e.runtime_dir());
    cfg.hub_id = Some("0123456789abcdef".into());
    cfg.sender = Some(sender);
    let api = Api::start(cfg, store.clone()).unwrap();
    let r = Rig { store, rx: dummy_rx(&e), api };
    let mut d = vec![];
    for _ in 0..20 {
        d.push(status(&r).1);
    }
    let p = p95(d);
    println!("status p95 against a silent hub: {p:?}");
    assert!(p < Duration::from_millis(1500), "status waited on a hub that never answers: p95 {p:?}");
}

fn dummy_rx(e: &Estate) -> Receiver {
    std::fs::create_dir_all(e.store_root().join("rx")).unwrap();
    let store = Arc::new(Store::open(&e.store_root().join("rx"), "fx-z").unwrap());
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: "fx-z".into(),
                instance: "z1".into(),
                identity: e.identity("z").unwrap(),
                hub: termlink_protocol::TransportAddr::Tcp { host: "<loopback>".into(), port: 9 },
                declared_hub_id: "0123456789abcdef".into(),
                queue_path: e.root().join("outbox-z1.sqlite"),
                timeout: Duration::from_secs(1),
                pump_interval: None,
            },
            store.clone(),
        )
        .unwrap(),
    );
    Receiver::new(
        ReceiveConfig {
            agent: "fx-z".into(),
            instance: "z1".into(),
            hub: termlink_protocol::TransportAddr::Tcp { host: "<loopback>".into(), port: 9 },
            declared_hub_id: "0123456789abcdef".into(),
            roster: Roster::new(),
            timeout: Duration::from_secs(1),
            page: 50,
            poll_interval: None,
        },
        store,
        sender,
        Box::new(BuiltIn::new(Roster::new())),
    )
    .unwrap()
}
```
