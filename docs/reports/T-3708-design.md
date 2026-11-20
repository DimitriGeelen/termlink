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

1. **Steady time is the sidecar's only clock (refines 3.5.3).** `store::now_ms()` now returns the wall time at the
   first call plus monotonic time since, so every stamp and deadline of the process (56 call sites) is immune to a
   wall jump without touching the callers. `store::wall_now_ms()` is the host clock, for display. A host clock step
   is not absorbed until the next start; the skew line against the hub shows it meanwhile.
2. **CLEAR judges this agent's own fields** (refines 3.2): mode, binary, hub, revocation, consumers, supervision,
   quota, strength, skew, ntp, last_surface. The main's liveness is another agent's state and keeps its own verdict
   under `main`. `revocation` reads unknown when `unverified …` or `hub predates revocation list` (no verified list:
   revoked keys are not known), bad when `stale …` or `ROLLBACK …`.
3. **V-8 counts from the later of storing and readiness** (as R-44.e and `deliver.rs` do), so a message that arrives
   after the agent became ready gets its 2 ticks.
4. **Carried T-3703 items:** the vocabulary (3.10.3) and the DOWNGRADED reason (3.10.4) are built. The reason applies
   only once the urgent allow-list is widened beyond the roster (`policy::urgent_allowed`); today the stale gate cannot
   change a decision. RK-1 (3.10.1) and E-127's reader (3.10.2) are open: E-127's "a message from a revoked key is
   STORED" needs the time each key was revoked, which the sidecar does not keep yet.
