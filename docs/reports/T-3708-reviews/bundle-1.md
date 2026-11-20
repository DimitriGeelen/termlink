# T-3708 review bundle 1 (P-24 slice a)

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

## Code (git diff of the slice, committed content)

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
index d494e6782..a3f1726fa 100644
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
@@ -480,6 +481,13 @@ fn dispatch_inner(req: &Value, scope: Scope, store: &Store, agent: &str, hub_id:
                 let vl = vendor_log_view(ctx.hook_errlog, store);
                 v["vendor_log"] = vl["state"].clone();
                 v["vendor_log_detail"] = vl;
+                // T-3708 (P-24 slice a): stamped fields, CLEAR, host strength, skew (additive; design 3.0).
+                if crate::fields::enabled() {
+                    let ready = ctx.deliverer.and_then(|d| d.ready_since(crate::store::now_ms()));
+                    p24_fields(&mut v, crate::fields::global(), Some(store), ready, crate::store::now_ms());
+                    // E-55 / E-134 (design 3.9): one latency line per call, bounded; a failing log is shown
+                    v["latency_log"] = json!(latency_log(store.dir(), started.elapsed().as_millis() as u64, &v));
+                }
             }
             v
         }
@@ -498,6 +506,10 @@ fn dispatch_inner(req: &Value, scope: Scope, store: &Store, agent: &str, hub_id:
             if let (Some(m), Some(sess)) = (ctx.meter, obj.get("session").and_then(Value::as_str)) {
                 m.legacy_hook(sess, "post-tool-use", None, crate::meter::adapter_ord(obj));
             }
+            // T-3708 (V-8): a hook call ran while these messages were pending; the first such time counts.
+            if let Some(d) = ctx.deliverer {
+                crate::fields::global().hook_call(&d.pending_ids());
+            }
             pull(obj, ctx.deliverer, ctx.lease, agent)
         }
         "claim" => claim(obj, ctx.deliverer),
@@ -1295,6 +1307,86 @@ fn status_at(
     })
 }
 
+/// T-3708 (P-24 slice a; AL-29, arch 6.7.1; design 3.1-3.5): add `fields` (each stamped, or
+/// `unknown (last at T)` past its bound), `clear`, and `strength_host` to a status reply. Additive:
+/// no existing key is touched. CLEAR is judged over this agent's own fields; the main's liveness is
+/// another agent's state and keeps its own verdict under `main`.
+pub(crate) fn p24_fields(v: &mut Value, reg: &crate::fields::Registry, store: Option<&Store>, ready_since: Option<u64>, now_wall: u64) {
+    use crate::fields::{self as f, BOUND_DAILY_MS, BOUND_HEARTBEAT_MS, BOUND_TICKS_MS};
+    let mono = reg.clock().mono_ms();
+    let rows = f::rows_of(&v["rows"]);
+    let missed = |id: &str| reg.hook_missed_at(id);
+    let strength = crate::host::strength_now();
+    let mut m = serde_json::Map::new();
+    m.insert("mode".into(), f::computed(&v["mode"], now_wall));
+    m.insert("binary".into(), f::computed(&v["binary"], now_wall));
+    m.insert("hub".into(), f::render(&v["hub"], reg.last(f::PULL), BOUND_TICKS_MS, mono));
+    m.insert("revocation".into(), f::render(&v["revocation"], reg.last(f::PULL), BOUND_TICKS_MS, mono));
+    m.insert("consumers".into(), f::render(&v["consumers"], reg.last(f::CONSUMERS), 2 * crate::consumers::CHECK_EVERY_MS, mono));
+    m.insert("supervision".into(), f::render(&v["supervision"], reg.last(f::HEARTBEAT), BOUND_HEARTBEAT_MS, mono));
+    // T-3708 D-2 (operator ruling A): no receiver quotas yet (P-17), so nothing can refuse receipts.
+    m.insert("quota".into(), f::computed(&json!(f::QUOTA_NOT_BUILT), now_wall));
+    m.insert("strength".into(), f::computed(&strength, now_wall));
+    let skew = crate::host::skew_view().unwrap_or(Value::Null);
+    m.insert("skew".into(), f::render(&skew, reg.last(f::SKEW), BOUND_TICKS_MS, mono));
+    let ntp = crate::host::ntp_view().map(Value::from).unwrap_or(Value::Null);
+    m.insert("ntp".into(), f::render(&ntp, reg.last(f::NTP), BOUND_DAILY_MS, mono));
+    m.insert("last_surface".into(), f::computed(&f::last_surface(&rows, ready_since, &missed, now_wall), now_wall));
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
+        let broken = crate::revocation::e127_broken(line, crate::revocation::narrowing(store), &crate::revocation::held_since(store), &stored, now_wall);
+        if !broken.is_empty() {
+            let day = (now_wall / 86_400_000).to_string();
+            let _ = store.attention_coalesced("revocation-invariant", &day, &broken.join("; "));
+        }
+        m.insert("revocation_invariants".into(), f::computed(&json!({"broken": broken}), now_wall));
+    }
+    v["clear"] = f::clear(&m, &rows, now_wall);
+    v["fields"] = Value::Object(m);
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
diff --git a/crates/termlink-sidecar/src/daemon.rs b/crates/termlink-sidecar/src/daemon.rs
index 5ef419a09..c0657b87c 100644
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
@@ -406,7 +408,12 @@ impl Running {
                 }
             }
         }
-        crate::beat::write(self.store.dir(), now, &beats, after)
+        let r = crate::beat::write(self.store.dir(), now, &beats, after);
+        // T-3708 (FCW-120): `supervision` is fresh only while the heartbeat is being written
+        if r.is_ok() {
+            crate::fields::touch(crate::fields::HEARTBEAT);
+        }
+        r
     }
 }
 
diff --git a/crates/termlink-sidecar/src/deliver.rs b/crates/termlink-sidecar/src/deliver.rs
index 8374e2787..2b3dd1ff7 100644
--- a/crates/termlink-sidecar/src/deliver.rs
+++ b/crates/termlink-sidecar/src/deliver.rs
@@ -360,6 +360,11 @@ impl Deliverer {
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
diff --git a/crates/termlink-sidecar/src/fields.rs b/crates/termlink-sidecar/src/fields.rs
new file mode 100644
index 000000000..84c1a7106
--- /dev/null
+++ b/crates/termlink-sidecar/src/fields.rs
@@ -0,0 +1,313 @@
+//! T-3708 (P-24 slice a; 06a AL-29, arch 6.7; design `docs/reports/T-3708-design.md` 3.1-3.3).
+//!
+//! Honest status: every field carries the time its producer last succeeded and a bound; past the
+//! bound it reads `unknown (last at T)`, never its last value (FCW-120). CLEAR is computed from the
+//! fields and from real hand-overs, never from a heartbeat (FCW-121). Freshness is measured on the
+//! monotonic clock; the wall time is shown only. The registry lives in memory, so after a restart
+//! every measured field is unknown until its producer has run again (06a 2.13.f).
+//!
+//! Additive only (design 3.0): this module adds `fields` and `clear` to `status`; it changes no
+//! existing key. `TERMLINK_SIDECAR_P24=off` hides both (rollback).
+
+use serde_json::{Value, json};
+use std::collections::BTreeMap;
+use std::sync::{Arc, Mutex, OnceLock};
+use std::time::Instant;
+
+use crate::deadline::TICK_MS;
+
+/// Producer names the registry stamps.
+pub const PULL: &str = "pull";
+pub const CONSUMERS: &str = "consumers";
+pub const HEARTBEAT: &str = "heartbeat";
+pub const LIVENESS: &str = "liveness";
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
+/// CLEAR looks back this far for pending mail.
+pub const CLEAR_WINDOW_MS: u64 = 3_600_000;
+
+/// The value `quota` reads until receiver quotas exist (T-3708 D-2, operator ruling A).
+pub const QUOTA_NOT_BUILT: &str = "absent: receiver quotas not built (P-17)";
+
+/// The switch for the new keys (design 3.0 rollback).
+pub const SWITCH_ENV: &str = "TERMLINK_SIDECAR_P24";
+
+/// Whether the P-24 keys are shown (default on; `off` hides them).
+pub fn enabled() -> bool {
+    std::env::var(SWITCH_ENV).map(|v| v.trim() != "off").unwrap_or(true)
+}
+
+/// Time source: monotonic for freshness and deadlines, wall for display.
+pub trait Clock: Send + Sync {
+    fn mono_ms(&self) -> u64;
+    fn wall_ms(&self) -> u64;
+    /// Wall time at start plus monotonic time since: a wall-shaped time that a wall jump does not move
+    /// (design 3.5.3). Deadlines compare against this.
+    fn steady_ms(&self) -> u64;
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
+impl Clock for SystemClock {
+    fn mono_ms(&self) -> u64 {
+        self.start.elapsed().as_millis() as u64
+    }
+    fn wall_ms(&self) -> u64 {
+        crate::store::wall_now_ms()
+    }
+    fn steady_ms(&self) -> u64 {
+        // the same timeline as every stamp and deadline of this process
+        crate::store::now_ms()
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
+/// The in-memory registry of producer successes and first missed hook calls.
+pub struct Registry {
+    clock: Arc<dyn Clock>,
+    seen: Mutex<BTreeMap<String, Seen>>,
+    /// msg_id -> steady time of the first hook call that ran while it was pending (V-8).
+    hook_missed: Mutex<BTreeMap<String, u64>>,
+}
+
+impl Registry {
+    pub fn new(clock: Arc<dyn Clock>) -> Self {
+        Registry { clock, seen: Mutex::new(BTreeMap::new()), hook_missed: Mutex::new(BTreeMap::new()) }
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
+    /// A hook call ran while these messages were pending: keep the first such time per message.
+    pub fn hook_call(&self, pending: &[String]) {
+        let now = self.clock.steady_ms();
+        let mut m = lock(&self.hook_missed);
+        m.retain(|k, _| pending.contains(k));
+        for id in pending {
+            m.entry(id.clone()).or_insert(now);
+        }
+    }
+
+    pub fn hook_missed_at(&self, msg_id: &str) -> Option<u64> {
+        lock(&self.hook_missed).get(msg_id).copied()
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
+/// Steady time for deadline checks (design 3.5.3).
+pub fn steady_ms() -> u64 {
+    global().clock().steady_ms()
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
+/// One chain row as `status` lists it (the subset V-8 and CLEAR read).
+#[derive(Debug, Clone)]
+pub struct Row {
+    pub msg_id: String,
+    pub stage: String,
+    pub ts_ms: u64,
+}
+
+/// Rows of `status.rows` as [`Row`]s.
+pub fn rows_of(status_rows: &Value) -> Vec<Row> {
+    status_rows
+        .as_array()
+        .into_iter()
+        .flatten()
+        .filter_map(|r| {
+            Some(Row {
+                msg_id: r.get("msg_id")?.as_str()?.to_string(),
+                stage: r.get("stage")?.as_str()?.to_string(),
+                ts_ms: r.get("peer_ts_ms").and_then(Value::as_u64).or_else(|| r.get("ts_ms").and_then(Value::as_u64))?,
+            })
+        })
+        .collect()
+}
+
+/// Messages this agent received and stored but has not handed over: (msg_id, stored at).
+/// A message this side SENT is never pending here.
+pub fn pending(rows: &[Row]) -> Vec<(String, u64)> {
+    let mut ids: Vec<&str> = rows.iter().filter(|r| r.stage == "STORED").map(|r| r.msg_id.as_str()).collect();
+    ids.sort();
+    ids.dedup();
+    ids.into_iter()
+        .filter(|id| !rows.iter().any(|r| r.msg_id == *id && (r.stage == "HANDED_OVER" || r.stage == "SENT")))
+        .filter_map(|id| rows.iter().filter(|r| r.msg_id == id && r.stage == "STORED").map(|r| r.ts_ms).min().map(|t| (id.to_string(), t)))
+        .collect()
+}
+
+/// Times of hand-overs this side made: HANDED_OVER lines of messages it received (a HANDED_OVER of a
+/// message it SENT is the recipient's, reported back, and is not this agent's surfacing).
+fn own_handovers(rows: &[Row]) -> impl Iterator<Item = u64> + '_ {
+    rows.iter()
+        .filter(|r| r.stage == "HANDED_OVER" && !rows.iter().any(|s| s.msg_id == r.msg_id && s.stage == "SENT"))
+        .map(|r| r.ts_ms)
+}
+
+/// V-8 (06a 2.13.c, arch 6.7.3): the last real surfacing and whether it is bad.
+/// Bad iff a pending message was not handed over 2 ticks after the agent became ready (counted from
+/// the later of storing and readiness, as R-44.e) or 2 ticks after a hook call that missed it.
+/// An idle agent with nothing pending is never bad; its age is shown. Only a HANDED_OVER line counts
+/// as surfacing; a heartbeat or READY never does.
+pub fn last_surface(rows: &[Row], ready_since: Option<u64>, hook_missed: &dyn Fn(&str) -> Option<u64>, now: u64) -> Value {
+    let last = own_handovers(rows).max();
+    let mut overdue = vec![];
+    for (id, stored) in pending(rows) {
+        let by_ready = ready_since.map(|t| t.max(stored)).filter(|t| now.saturating_sub(*t) > SURFACE_AFTER_MS);
+        let by_hook = hook_missed(&id).filter(|t| now.saturating_sub(*t) > SURFACE_AFTER_MS);
+        if let Some(t) = by_ready.or(by_hook) {
+            overdue.push(json!({"msg_id": id, "since_ms": t, "by": if by_ready.is_some() { "ready" } else { "hook call" }}));
+        }
+    }
+    json!({
+        "last_surface_ms": last,
+        "age_ms": last.map(|t| now.saturating_sub(t)),
+        "bad": !overdue.is_empty(),
+        "overdue": overdue,
+    })
+}
+
+/// CLEAR (FCW-121): no field bad, no field unknown, and if anything was pending in the last hour, a
+/// HANDED_OVER after the earliest such STORED. Returns `{state, bad, unknown, why}`.
+pub fn clear(fields: &serde_json::Map<String, Value>, rows: &[Row], now: u64) -> Value {
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
+                let s = v["revocation"].as_str().unwrap_or("");
+                if s.starts_with("stale") || s.starts_with("ROLLBACK") {
+                    bad.push(format!("revocation: {s}"));
+                } else if s.starts_with("unverified") || s == "hub predates revocation list" {
+                    // no verified list: revoked keys are not known here
+                    unknown.push("revocation".into());
+                }
+            }
+            "consumers" => {
+                let s = v["consumers"].as_str().unwrap_or("");
+                if s.starts_with("unknown") {
+                    unknown.push("consumers".into());
+                } else if s != "one (proven)" {
+                    bad.push(format!("consumers: {s}"));
+                }
+            }
+            "supervision" => {
+                if v.get("heartbeat").is_some() {
+                    unknown.push("supervision".into());
+                } else {
+                    if v["stale"].as_array().is_some_and(|a| !a.is_empty()) {
+                        bad.push("supervision: a component is dead".into());
+                    }
+                    let old: Vec<&String> = v["component_age_ms"]
+                        .as_object()
+                        .map(|m| m.iter().filter(|(_, a)| a.as_u64().unwrap_or(u64::MAX) > COMPONENT_BAD_MS).map(|(k, _)| k).collect())
+                        .unwrap_or_default();
+                    if !old.is_empty() {
+                        bad.push(format!("supervision: heartbeat older than 3 ticks: {}", old.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")));
+                    }
+                }
+            }
+            "last_surface" if v["bad"] == Value::Bool(true) => bad.push("last_surface: pending mail not handed over".into()),
+            "revocation_invariants" if v["broken"].as_array().is_some_and(|a| !a.is_empty()) => {
+                bad.push(format!("revocation_invariants: {}", v["broken"]))
+            }
+            _ => {}
+        }
+    }
+    // Pending in the last hour needs a real surfacing after the earliest such STORED.
+    let since = now.saturating_sub(CLEAR_WINDOW_MS);
+    let earliest = rows
+        .iter()
+        .filter(|r| r.stage == "STORED" && r.ts_ms >= since && !rows.iter().any(|s| s.msg_id == r.msg_id && s.stage == "SENT"))
+        .map(|r| r.ts_ms)
+        .min();
+    let surfaced = earliest.is_none_or(|e| own_handovers(rows).any(|t| t >= e));
+    if !surfaced {
+        bad.push("no hand-over after mail pending in the last hour".into());
+    }
+    let state = if bad.is_empty() && unknown.is_empty() { "CLEAR" } else { "NOT CLEAR" };
+    json!({"state": state, "bad": bad, "unknown": unknown})
+}
+
+#[cfg(test)]
+#[path = "fields_tests.rs"]
+mod tests;
diff --git a/crates/termlink-sidecar/src/fields_tests.rs b/crates/termlink-sidecar/src/fields_tests.rs
new file mode 100644
index 000000000..b2c38360d
--- /dev/null
+++ b/crates/termlink-sidecar/src/fields_tests.rs
@@ -0,0 +1,320 @@
+//! T-3708 unit tests for `fields` on a simulated clock (E-19, E-55 freshness; E-56 CLEAR; E-57 V-8).
+
+use super::*;
+use std::sync::atomic::{AtomicU64, Ordering};
+
+/// A clock the test moves by hand; `wall_jump` moves the wall time only.
+struct FakeClock {
+    mono: AtomicU64,
+    wall_offset: AtomicU64,
+}
+
+impl FakeClock {
+    fn new() -> Arc<Self> {
+        Arc::new(FakeClock { mono: AtomicU64::new(0), wall_offset: AtomicU64::new(1_000_000_000) })
+    }
+    fn advance(&self, ms: u64) {
+        self.mono.fetch_add(ms, Ordering::SeqCst);
+    }
+    fn wall_jump(&self, ms: i64) {
+        let w = self.wall_offset.load(Ordering::SeqCst) as i64 + ms;
+        self.wall_offset.store(w as u64, Ordering::SeqCst);
+    }
+}
+
+impl Clock for FakeClock {
+    fn mono_ms(&self) -> u64 {
+        self.mono.load(Ordering::SeqCst)
+    }
+    fn wall_ms(&self) -> u64 {
+        self.wall_offset.load(Ordering::SeqCst) + self.mono_ms()
+    }
+    fn steady_ms(&self) -> u64 {
+        1_000_000_000 + self.mono_ms()
+    }
+}
+
+fn row(id: &str, stage: &str, ts: u64) -> Row {
+    Row { msg_id: id.into(), stage: stage.into(), ts_ms: ts }
+}
+
+fn fresh(v: Value) -> Value {
+    json!({"state": "fresh", "value": v, "at_ms": 1, "bound_ms": 0})
+}
+
+fn healthy() -> serde_json::Map<String, Value> {
+    let mut m = serde_json::Map::new();
+    m.insert("hub".into(), fresh(json!({"id": "h", "declared": "h", "match": true})));
+    m.insert("revocation".into(), fresh(json!({"revocation": "verified seq 3 at 100", "refused_revoked": 0})));
+    m.insert("consumers".into(), fresh(json!({"consumers": "one (proven)"})));
+    m.insert("supervision".into(), fresh(json!({"stale": [], "component_age_ms": {"send": 100, "receive": 200}})));
+    m.insert("quota".into(), fresh(json!(QUOTA_NOT_BUILT)));
+    m.insert("last_surface".into(), fresh(json!({"bad": false})));
+    m
+}
+
+// E-19 / E-55 (FCW-120): a field past its bound reads unknown, never its last value.
+#[test]
+fn e19_frozen_field_turns_unknown_after_its_bound() {
+    let c = FakeClock::new();
+    let r = Registry::new(c.clone());
+    r.touch(PULL);
+    let v = json!({"match": true});
+    let at = render(&v, r.last(PULL), BOUND_TICKS_MS, c.mono_ms());
+    assert_eq!(at["state"], "fresh");
+    assert_eq!(at["value"], v);
+    assert!(at["at_ms"].is_u64() && at["bound_ms"] == BOUND_TICKS_MS);
+    c.advance(BOUND_TICKS_MS);
+    assert_eq!(render(&v, r.last(PULL), BOUND_TICKS_MS, c.mono_ms())["state"], "fresh", "at the bound it is still fresh");
+    c.advance(1);
+    let late = render(&v, r.last(PULL), BOUND_TICKS_MS, c.mono_ms());
+    assert_eq!(late["state"], "unknown");
+    assert!(late.get("value").is_none(), "the last value is never served past the bound: {late}");
+    assert!(late["shown"].as_str().unwrap().starts_with("unknown (last at "));
+}
+
+#[test]
+fn e55_never_measured_reads_unknown_not_measured_since_start() {
+    let c = FakeClock::new();
+    let r = Registry::new(c.clone());
+    let v = render(&json!(1), r.last(PULL), BOUND_TICKS_MS, c.mono_ms());
+    assert_eq!(v["state"], "unknown");
+    assert_eq!(v["shown"], "unknown (not measured since start)");
+    assert!(v["last_at_ms"].is_null());
+}
+
+// E-59: a wall jump does not change freshness (measured on the monotonic clock) or steady time.
+#[test]
+fn e59_wall_jump_moves_neither_freshness_nor_steady_time() {
+    let c = FakeClock::new();
+    let r = Registry::new(c.clone());
+    r.touch(PULL);
+    let steady = c.steady_ms();
+    c.wall_jump(3_600_000);
+    assert_eq!(render(&json!(1), r.last(PULL), BOUND_TICKS_MS, c.mono_ms())["state"], "fresh");
+    c.wall_jump(-7_200_000);
+    assert_eq!(render(&json!(1), r.last(PULL), BOUND_TICKS_MS, c.mono_ms())["state"], "fresh");
+    assert_eq!(c.steady_ms(), steady);
+}
+
+// E-56 (FCW-121): CLEAR needs real surfacing; a fresh heartbeat alone is never enough.
+#[test]
+fn e56_refused_confirmations_with_fresh_heartbeat_read_not_clear() {
+    let now = 10_000_000;
+    // mail stored 10 min ago, never handed over (every confirmation refused); heartbeat is fresh
+    let rows = vec![row("m1", "RECEIVED", now - 600_000), row("m1", "STORED", now - 600_000)];
+    let ls = last_surface(&rows, Some(now - 600_000), &|_| None, now);
+    assert_eq!(ls["bad"], true);
+    let mut f = healthy();
+    f.insert("last_surface".into(), fresh(ls));
+    let c = clear(&f, &rows, now);
+    assert_eq!(c["state"], "NOT CLEAR");
+    assert!(c["bad"].as_array().unwrap().iter().any(|b| b.as_str().unwrap().starts_with("last_surface")), "{c}");
+}
+
+#[test]
+fn e56_healthy_receiver_reads_clear_after_the_hand_over() {
+    let now = 10_000_000;
+    let rows = vec![row("m1", "STORED", now - 600_000), row("m1", "HANDED_OVER", now - 590_000)];
+    let mut f = healthy();
+    f.insert("last_surface".into(), fresh(last_surface(&rows, Some(now - 600_000), &|_| None, now)));
+    assert_eq!(clear(&f, &rows, now)["state"], "CLEAR");
+}
+
+#[test]
+fn e56_a_sent_message_handed_over_by_the_recipient_is_not_our_surfacing() {
+    let now = 10_000_000;
+    let rows = vec![
+        row("in", "STORED", now - 600_000),
+        row("out", "SENT", now - 500_000),
+        row("out", "HANDED_OVER", now - 400_000),
+    ];
+    let f = healthy();
+    assert_eq!(clear(&f, &rows, now)["state"], "NOT CLEAR");
+}
+
+#[test]
+fn clear_counts_unknown_and_each_bad_field() {
+    let now = 10_000_000;
+    let mut f = healthy();
+    f.insert("consumers".into(), json!({"state": "unknown", "shown": "unknown (last at 1)"}));
+    let c = clear(&f, &[], now);
+    assert_eq!(c["state"], "NOT CLEAR");
+    assert_eq!(c["unknown"], json!(["consumers"]));
+    for (k, v, needle) in [
+        ("hub", json!({"match": false}), "hub"),
+        ("revocation", json!({"revocation": "stale since 5 (no pull)"}), "revocation"),
+        ("consumers", json!({"consumers": "inbox consumed by 2 identities: a, b"}), "consumers"),
+        ("supervision", json!({"stale": ["send"], "component_age_ms": {}}), "supervision"),
+        ("supervision", json!({"stale": [], "component_age_ms": {"tick": 3 * TICK_MS + 1}}), "older than 3 ticks"),
+    ] {
+        let mut f = healthy();
+        f.insert(k.into(), fresh(v));
+        let c = clear(&f, &[], now);
+        assert_eq!(c["state"], "NOT CLEAR", "{k}");
+        assert!(c["bad"].to_string().contains(needle), "{k}: {c}");
+    }
+    let mut f = healthy();
+    f.insert("revocation".into(), fresh(json!({"revocation": "unverified since start (no pull yet)"})));
+    assert_eq!(clear(&f, &[], now)["unknown"], json!(["revocation"]));
+}
+
+// T-3703 r1 finding 5 (carried): the six `revocation` status forms bind, and CLEAR reads each one.
+#[test]
+fn revocation_vocabulary_binds_and_clear_reads_each_form() {
+    for (line, expect) in [
+        ("verified seq 7 at 1795150000000", "CLEAR"),
+        ("off (PH-1 trust)", "CLEAR"),
+        ("unverified since 1795150000000 (hub unreachable: refused)", "unknown"),
+        ("unverified since start (no pull yet)", "unknown"),
+        ("hub predates revocation list", "unknown"),
+        ("stale since 1795150000000 (hub unreachable: refused)", "bad"),
+        ("ROLLBACK refused, kept seq 7", "bad"),
+    ] {
+        let mut f = healthy();
+        f.insert("revocation".into(), fresh(json!({"revocation": line, "refused_revoked": 0})));
+        let c = clear(&f, &[], 10_000_000);
+        let got = if c["state"] == "CLEAR" {
+            "CLEAR"
+        } else if c["unknown"] == json!(["revocation"]) {
+            "unknown"
+        } else if c["bad"].to_string().contains("revocation") {
+            "bad"
+        } else {
+            "other"
+        };
+        assert_eq!(got, expect, "{line}: {c}");
+    }
+}
+
+// E-127's reader (carried from T-3703 r1 finding 17): both invariants fire, and only when broken.
+#[test]
+fn e127_reader_fires_on_each_broken_invariant_only() {
+    use crate::revocation::{PN13, e127_broken};
+    let pn13 = PN13.as_millis() as u64;
+    let key = "ed25519:abcdef0123456789aaaa".to_string();
+    let held: std::collections::BTreeMap<String, u64> = [(key.clone(), 5_000)].into_iter().collect();
+    let now = 10 * pn13;
+    // (a) unverified longer than PN-13: fires only without the narrowing
+    let line = format!("unverified since {} (hub unreachable)", now - pn13 - 1);
+    assert_eq!(e127_broken(&line, true, &held, &[], now), Vec::<String>::new());
+    assert_eq!(e127_broken(&line, false, &held, &[], now).len(), 1);
+    let stale = format!("stale since {} (x)", now - pn13 - 1);
+    assert_eq!(e127_broken(&stale, false, &held, &[], now).len(), 1);
+    let young = format!("unverified since {} (x)", now - pn13 + 1);
+    assert!(e127_broken(&young, false, &held, &[], now).is_empty());
+    assert!(e127_broken("verified seq 3 at 1", false, &held, &[], now).is_empty());
+    // (b) STORED from a held key: before it was held is lawful, after is broken
+    let before = [("m1".to_string(), "abcdef0123456789".to_string(), 4_999)];
+    assert!(e127_broken("verified seq 3 at 1", true, &held, &before, now).is_empty());
+    let after = [("m2".to_string(), "abcdef0123456789".to_string(), 5_001)];
+    let b = e127_broken("verified seq 3 at 1", true, &held, &after, now);
+    assert_eq!(b.len(), 1);
+    assert!(b[0].contains("m2"), "{b:?}");
+    // another key's message is not flagged
+    let other = [("m3".to_string(), "0000000000000000".to_string(), 9_999)];
+    assert!(e127_broken("verified seq 3 at 1", true, &held, &other, now).is_empty());
+    // CLEAR reads a broken invariant as bad
+    let mut f = healthy();
+    f.insert("revocation_invariants".into(), fresh(json!({"broken": b})));
+    assert!(clear(&f, &[], now)["bad"].to_string().contains("revocation_invariants"));
+}
+
+// D-2 (operator ruling A): the not-built quota value does not hold CLEAR back.
+#[test]
+fn d2_quota_not_built_does_not_hold_clear() {
+    let f = healthy();
+    assert_eq!(f["quota"]["value"], QUOTA_NOT_BUILT);
+    assert_eq!(clear(&f, &[], 10_000_000)["state"], "CLEAR");
+}
+
+// E-57 (V-8): idle 48 h with nothing pending is never bad, its age is shown.
+#[test]
+fn e57_idle_48h_with_nothing_pending_is_not_bad() {
+    let now = 200 * 3_600_000;
+    let rows = vec![row("m0", "STORED", now - 48 * 3_600_000), row("m0", "HANDED_OVER", now - 48 * 3_600_000)];
+    let v = last_surface(&rows, Some(now - 48 * 3_600_000), &|_| None, now);
+    assert_eq!(v["bad"], false);
+    assert_eq!(v["age_ms"], 48 * 3_600_000);
+}
+
+#[test]
+fn e57_bad_exactly_after_ready_plus_two_ticks() {
+    let ready = 1_000_000;
+    let rows = vec![row("m1", "STORED", ready - 5_000)];
+    assert_eq!(last_surface(&rows, Some(ready), &|_| None, ready + SURFACE_AFTER_MS)["bad"], false);
+    let v = last_surface(&rows, Some(ready), &|_| None, ready + SURFACE_AFTER_MS + 1);
+    assert_eq!(v["bad"], true);
+    assert_eq!(v["overdue"][0]["by"], "ready");
+}
+
+#[test]
+fn e57_stored_after_ready_counts_from_storing() {
+    let ready = 1_000_000;
+    let stored = ready + 100_000;
+    let rows = vec![row("m1", "STORED", stored)];
+    assert_eq!(last_surface(&rows, Some(ready), &|_| None, stored + SURFACE_AFTER_MS)["bad"], false);
+    assert_eq!(last_surface(&rows, Some(ready), &|_| None, stored + SURFACE_AFTER_MS + 1)["bad"], true);
+}
+
+#[test]
+fn e57_bad_two_ticks_after_a_hook_call_that_missed_it() {
+    let c = FakeClock::new();
+    let r = Registry::new(c.clone());
+    let rows = vec![row("m1", "STORED", 0)];
+    r.hook_call(&["m1".to_string()]);
+    let t = c.steady_ms();
+    c.advance(10_000);
+    r.hook_call(&["m1".to_string()]); // a later call keeps the first time
+    assert_eq!(r.hook_missed_at("m1"), Some(t));
+    let missed = |id: &str| r.hook_missed_at(id);
+    assert_eq!(last_surface(&rows, None, &missed, t + SURFACE_AFTER_MS)["bad"], false);
+    let v = last_surface(&rows, None, &missed, t + SURFACE_AFTER_MS + 1);
+    assert_eq!(v["bad"], true);
+    assert_eq!(v["overdue"][0]["by"], "hook call");
+    // handed over: no longer pending, the record is dropped at the next call
+    r.hook_call(&[]);
+    assert_eq!(r.hook_missed_at("m1"), None);
+}
+
+#[test]
+fn e57_no_ready_no_hook_call_is_not_bad_however_old() {
+    let rows = vec![row("m1", "STORED", 0)];
+    assert_eq!(last_surface(&rows, None, &|_| None, 100 * 3_600_000)["bad"], false);
+}
+
+#[test]
+fn heartbeat_or_ready_never_counts_as_surfacing() {
+    // only HANDED_OVER rows count; READY and heartbeats are not rows at all, and other stages do not count
+    let rows = vec![row("m1", "STORED", 0), row("m1", "CLAIMED", 10), row("m1", "HANDOVER_UNCONFIRMED", 20)];
+    let v = last_surface(&rows, Some(0), &|_| None, 1_000_000);
+    assert!(v["last_surface_ms"].is_null());
+    assert_eq!(v["bad"], true);
+}
+
+// E-59 control (design mutant 5): the process time every stamp and deadline uses does not follow a
+// wall-clock jump of ±1 h; the wall clock (display only) does.
+#[test]
+fn e59_process_time_ignores_a_wall_jump() {
+    let before = crate::store::now_ms();
+    let wall_before = crate::store::wall_now_ms();
+    crate::store::test_shift_wall_ms(3_600_000);
+    let after = crate::store::now_ms();
+    let wall_after = crate::store::wall_now_ms();
+    crate::store::test_shift_wall_ms(-7_200_000);
+    let back = crate::store::now_ms();
+    crate::store::test_shift_wall_ms(3_600_000);
+    assert!(wall_after >= wall_before + 3_600_000, "the seam moves the wall clock");
+    assert!(after >= before && after - before < 60_000, "+1 h jump moved process time by {} ms", after - before);
+    assert!(back >= after && back - after < 60_000, "-1 h jump moved process time");
+}
+
+#[test]
+fn switch_off_hides_the_keys() {
+    // The env switch is read per call; default on.
+    // SAFETY (test): single-threaded use of this variable in this test binary.
+    unsafe { std::env::set_var(SWITCH_ENV, "off") };
+    assert!(!enabled());
+    unsafe { std::env::remove_var(SWITCH_ENV) };
+    assert!(enabled());
+}
diff --git a/crates/termlink-sidecar/src/host.rs b/crates/termlink-sidecar/src/host.rs
new file mode 100644
index 000000000..4edf65129
--- /dev/null
+++ b/crates/termlink-sidecar/src/host.rs
@@ -0,0 +1,330 @@
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
+    let per_agent = match procs {
+        None => "no (could not check)".to_string(),
+        Some(ps) => {
+            let mut by_agent: BTreeMap<&str, BTreeSet<u32>> = BTreeMap::new();
+            for p in ps {
+                by_agent.entry(p.agent.as_str()).or_default().insert(p.uid);
+            }
+            let uids: BTreeSet<u32> = ps.iter().map(|p| p.uid).collect();
+            if by_agent.len() < 2 {
+                "no (one agent on this host: nothing to separate)".to_string()
+            } else if uids.len() == by_agent.len() && by_agent.values().all(|u| u.len() == 1) && !uids.contains(&0) {
+                "yes".to_string()
+            } else {
+                format!("no: {} agents run under {} account(s)", by_agent.len(), uids.len())
+            }
+        }
+    };
+    let same_user = if own_uid == 0 {
+        "absent: agents run as root".to_string()
+    } else if per_agent == "yes" {
+        "present: one account per agent".to_string()
+    } else {
+        format!("absent: agents share uid {own_uid}")
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
+    fn one_account_per_agent_is_present_only_off_root() {
+        assert_eq!(strength(Some(&[p("a", 1001), p("b", 1002)]), 1001)["per_agent_accounts"], "yes");
+        assert_eq!(strength(Some(&[p("a", 1001), p("b", 1002)]), 1001)["same_user_separation"], "present: one account per agent");
+        // two processes of one agent under two uids is not per-agent separation
+        assert_ne!(strength(Some(&[p("a", 1001), p("a", 1003), p("b", 1002)]), 1001)["per_agent_accounts"], "yes");
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
index 081e63da0..4de26dcab 100644
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
@@ -414,6 +418,8 @@ impl Inner {
             Err(e) => crate::consumers::verdict(me, inst, &fp, Err((0, &format!("hub unreachable: {e}")))),
         };
         crate::consumers::record(&self.store, &topic, &v, now);
+        // T-3708 (FCW-120): the check ran; its value says when it could not see (`unknown (...)`)
+        crate::fields::touch(crate::fields::CONSUMERS);
     }
 
     fn cursor(&self) -> u64 {
@@ -453,6 +459,8 @@ impl Inner {
         let version = match self.check_pin() {
             Ok(version) => {
                 mem.held = None;
+                // T-3708 (FCW-120): the hub was read; `hub` and `revocation` are fresh from this pass.
+                crate::fields::touch(crate::fields::PULL);
                 self.observe_impersonation();
                 version
             }
@@ -481,6 +489,7 @@ impl Inner {
         let mut handled = 0;
         loop {
             let cursor = self.cursor();
+            let before = crate::store::now_ms();
             let resp = match self.rpc(
                 method::CHANNEL_SUBSCRIBE,
                 // T-3700 (AL-30, ruling E): a named, signed read of our own inbox (proven consumer)
@@ -490,6 +499,10 @@ impl Inner {
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
@@ -607,7 +620,7 @@ impl Inner {
         }
         if a.downgraded {
             // Asked for urgent without the allow-list: normal, and said so (SI-18).
-            let why = format!("priority {} from a {} sender: urgent needs the allow-list", a.priority, a.trust.as_str());
+            let why = policy::downgrade_why(a.priority, a.trust, crate::revocation::narrowing(&self.store));
             if let Err(e) = once(policy::DOWNGRADED, Some(&why)) {
                 return Outcome::Retry(format!("DOWNGRADED not recorded: {e}"));
             }
diff --git a/crates/termlink-sidecar/src/revocation.rs b/crates/termlink-sidecar/src/revocation.rs
index 12586cab3..c7985e963 100644
--- a/crates/termlink-sidecar/src/revocation.rs
+++ b/crates/termlink-sidecar/src/revocation.rs
@@ -111,6 +111,67 @@ pub fn revoked_from(store: &Store, from: &str) -> bool {
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
+        let _ = store.write_state(HELD_SINCE_STATE, &text);
+    }
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
@@ -377,6 +438,9 @@ impl Tracker {
         let _ = store.write_state(HUB_PK_STATE, &ss::hex(pk.as_bytes()));
         self.held = Some(held_of(&l));
         self.held_unverified = false;
+        if let Some(h) = &self.held {
+            record_held_since(store, &h.keys, crate::store::now_ms());
+        }
         Ok(())
     }
 
diff --git a/crates/termlink-sidecar/src/store.rs b/crates/termlink-sidecar/src/store.rs
index c3067735c..cf0f72d08 100644
--- a/crates/termlink-sidecar/src/store.rs
+++ b/crates/termlink-sidecar/src/store.rs
@@ -174,11 +174,31 @@ struct Body<'a> {
 }
 
 /// Wall-clock now in ms since the epoch.
+/// The sidecar's time: the wall time at the first call plus monotonic time since (T-3708, E-59,
+/// design 3.5.3). Every stamp and deadline in this process uses it, so a wall-clock jump moves no
+/// deadline. A host clock step is not absorbed until the next start; the skew estimate against the
+/// hub shows it meanwhile (`status.fields.skew`).
 pub fn now_ms() -> u64 {
-    std::time::SystemTime::now()
+    static ANCHOR: std::sync::OnceLock<(std::time::Instant, u64)> = std::sync::OnceLock::new();
+    let (at, wall) = ANCHOR.get_or_init(|| (std::time::Instant::now(), wall_now_ms()));
+    wall + at.elapsed().as_millis() as u64
+}
+
+static WALL_SHIFT_MS: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
+
+/// The host's wall clock, for display only (plus a test shift, see [`test_shift_wall_ms`]).
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
diff --git a/crates/termlink-sidecar/tests/t3708_live.rs b/crates/termlink-sidecar/tests/t3708_live.rs
new file mode 100644
index 000000000..e61996a3e
--- /dev/null
+++ b/crates/termlink-sidecar/tests/t3708_live.rs
@@ -0,0 +1,148 @@
+//! T-3708 (P-24 slice a) on a real hub (testkit estate): E-55 (fields stamped by their producers; p95 ≤ 5 s over
+//! 100 calls with the hub up and with the hub stopped, the fault seam that exists before P-19), FCW-120 (a stopped
+//! hub leaves `hub` fresh only within its bound), E-58 at L4 on this host (strength flags against `ps -o user=`).
+
+use std::io::{BufRead, BufReader, Write};
+use std::os::unix::net::UnixStream;
+use std::sync::Arc;
+use std::time::{Duration, Instant};
+
+use serde_json::{Value, json};
+use termlink_sidecar::api::{Api, ApiConfig, Scope};
+use termlink_sidecar::policy::BuiltIn;
+use termlink_sidecar::pull::{ReceiveConfig, Receiver};
+use termlink_sidecar::send::{SendConfig, Sender};
+use termlink_sidecar::store::Store;
+use termlink_sidecar::testkit::Estate;
+use termlink_sidecar::wire::Roster;
+
+struct Rig {
+    store: Arc<Store>,
+    rx: Receiver,
+    api: Api,
+}
+
+fn rig(e: &Estate) -> Rig {
+    let id = e.hub_id().unwrap();
+    let agent = "fx-b".to_string();
+    let store = Arc::new(Store::open(&e.store_root(), &agent).unwrap());
+    let sender = Arc::new(
+        Sender::new(
+            SendConfig {
+                agent: agent.clone(),
+                instance: "b1".into(),
+                identity: e.identity("b").unwrap(),
+                hub: e.hub_addr(),
+                declared_hub_id: id.clone(),
+                queue_path: e.root().join("outbox-b1.sqlite"),
+                timeout: Duration::from_secs(5),
+                pump_interval: None,
+            },
+            store.clone(),
+        )
+        .unwrap(),
+    );
+    let roster = Roster::new();
+    let rx = Receiver::new(
+        ReceiveConfig {
+            agent: agent.clone(),
+            instance: "b1".into(),
+            hub: e.hub_addr(),
+            declared_hub_id: id.clone(),
+            roster: roster.clone(),
+            timeout: Duration::from_secs(5),
+            page: 50,
+            poll_interval: None,
+        },
+        store.clone(),
+        sender.clone(),
+        Box::new(BuiltIn::new(roster)),
+    )
+    .unwrap();
+    let mut cfg = ApiConfig::new(&agent, &e.runtime_dir());
+    cfg.hub_id = Some(id);
+    cfg.sender = Some(sender);
+    let api = Api::start(cfg, store.clone()).unwrap();
+    Rig { store, rx, api }
+}
+
+fn status(r: &Rig) -> (Value, Duration) {
+    let t = Instant::now();
+    let mut s = UnixStream::connect(r.api.path(Scope::Status)).unwrap();
+    writeln!(s, "{}", json!({"op": "status"})).unwrap();
+    let mut line = String::new();
+    BufReader::new(s).read_line(&mut line).unwrap();
+    (serde_json::from_str(&line).unwrap(), t.elapsed())
+}
+
+fn p95(mut d: Vec<Duration>) -> Duration {
+    d.sort();
+    d[(d.len() * 95).div_ceil(100) - 1]
+}
+
+#[test]
+fn t3708_e55_fields_stamped_by_producers_and_fast_with_the_hub_up_or_down() {
+    let Ok(mut e) = Estate::create() else {
+        eprintln!("SKIP: no termlink binary for a test hub");
+        return;
+    };
+    let r = rig(&e);
+    let _ = r.rx.pass();
+    let (v, _) = status(&r);
+    // the pull pass read the hub: hub and revocation are fresh, with the producer's time
+    for n in ["hub", "revocation", "consumers", "skew"] {
+        assert_eq!(v["fields"][n]["state"], "fresh", "{n} after a pass: {}", v["fields"][n]);
+    }
+    assert_eq!(v["fields"]["hub"]["value"]["match"], true);
+    let skew = &v["fields"]["skew"]["value"];
+    assert_eq!(skew["clock"], "ok", "same host, no skew: {skew}");
+
+    let mut up = vec![];
+    for _ in 0..100 {
+        up.push(status(&r).1);
+    }
+    e.stop_hub();
+    let mut down = vec![];
+    for _ in 0..100 {
+        down.push(status(&r).1);
+    }
+    let (pu, pd) = (p95(up), p95(down));
+    println!("E-55 p95 status latency: hub up {pu:?}, hub stopped {pd:?}");
+    assert!(pu <= Duration::from_secs(5) && pd <= Duration::from_secs(5), "p95 up {pu:?} down {pd:?}");
+    // status makes no hub call: a stopped hub costs nothing (design mutant 10)
+    assert!(pd < Duration::from_millis(1500), "status waited on the stopped hub: p95 {pd:?}");
+    // a pass against the stopped hub does not refresh `hub`
+    let before = status(&r).0["fields"]["hub"]["at_ms"].clone();
+    let _ = r.rx.pass();
+    assert_eq!(status(&r).0["fields"]["hub"]["at_ms"], before, "a failed pass must not stamp the hub field");
+    let lines = std::fs::read_to_string(r.store.dir().join(termlink_sidecar::api::LATENCY_LOG)).unwrap();
+    assert!(lines.lines().count() >= 200, "one latency line per call");
+    let _ = e.teardown();
+}
+
+/// E-58 at L4: every flag matches what the host shows (`ps -o user=` over the sidecar processes).
+#[test]
+fn t3708_e58_strength_flags_match_the_host() {
+    let v = termlink_sidecar::host::strength_now();
+    let out = std::process::Command::new("ps").args(["-eo", "user=,args="]).output().unwrap();
+    let ps = String::from_utf8_lossy(&out.stdout);
+    let users: std::collections::BTreeSet<&str> = ps
+        .lines()
+        .filter(|l| l.contains("sidecar start --config") || (l.contains("http_server.py") && l.contains("--agent")))
+        .filter_map(|l| l.split_whitespace().next())
+        .collect();
+    let agents = ps.lines().filter(|l| l.contains("sidecar start --config") || (l.contains("http_server.py") && l.contains("--agent"))).count();
+    // SAFETY: geteuid has no preconditions.
+    let root = unsafe { libc::geteuid() } == 0;
+    println!("E-58 host: {agents} sidecar processes under users {users:?}; flags {v}");
+    let pa = v["per_agent_accounts"].as_str().unwrap();
+    if users.len() < agents || users.contains("root") {
+        assert!(pa.starts_with("no"), "per_agent_accounts claims strength the host lacks: {pa}");
+    }
+    if root {
+        assert_eq!(v["same_user_separation"], "absent: agents run as root");
+    }
+    assert_eq!(v["operator_key"], "no");
+    assert_eq!(v["confirmation_provider"], "no");
+    assert_eq!(v["policy_records"], "none (fixed defaults)");
+}
diff --git a/crates/termlink-sidecar/tests/t3708_status.rs b/crates/termlink-sidecar/tests/t3708_status.rs
new file mode 100644
index 000000000..8f3461234
--- /dev/null
+++ b/crates/termlink-sidecar/tests/t3708_status.rs
@@ -0,0 +1,89 @@
+//! T-3708 (P-24 slice a): status through a real API socket. Hermetic: testkit estate, no hub.
+//! Design 3.0 (additive only), 3.1 (E-55 schema, restart = unknown), 3.9 (latency log), D-2 (quota).
+
+use serde_json::{Value, json};
+use std::io::{BufRead, BufReader, Write};
+use std::os::unix::net::UnixStream;
+use std::sync::Arc;
+use termlink_sidecar::api::*;
+use termlink_sidecar::store::Store;
+use termlink_sidecar::testkit::Estate;
+
+/// Keys `status` returned before P-24 (live path): they must stay, with the same shape.
+const PRE_P24_KEYS: &[&str] = &[
+    "ok", "agent", "hub_id", "hub", "hub_signing_key", "revocation", "consumers", "stages", "rows", "messages", "chains",
+    "delivery", "mode", "receiving", "strength", "missing_strength", "notes", "outbox", "supervision", "activity_veto",
+    "binary", "main", "keys", "vendor_log", "vendor_log_detail",
+];
+const P24_KEYS: &[&str] = &["fields", "clear", "strength_host", "latency_log"];
+const FIELD_NAMES: &[&str] =
+    &["mode", "binary", "hub", "revocation", "consumers", "supervision", "quota", "strength", "skew", "ntp", "last_surface"];
+
+fn status(e: &Estate, store: &Arc<Store>) -> Value {
+    let cfg = ApiConfig::new("agent-b", &e.runtime_dir());
+    let api = Api::start(cfg, store.clone()).unwrap();
+    let mut s = UnixStream::connect(api.path(Scope::Status)).unwrap();
+    writeln!(s, "{}", json!({"op": "status"})).unwrap();
+    let mut line = String::new();
+    BufReader::new(s).read_line(&mut line).unwrap();
+    serde_json::from_str(&line).unwrap()
+}
+
+// One test, sequential: the switch is a process environment variable.
+#[test]
+fn t3708_status_is_additive_stamped_and_switchable() {
+    let e = Estate::create_without_hub().unwrap();
+    let store = Arc::new(Store::open(&e.store_root(), "agent-b").unwrap());
+    let v = status(&e, &store);
+    assert_eq!(v["ok"], true);
+
+    // design 3.0: every pre-P-24 key is still there; the old strength map keeps its shape
+    for k in PRE_P24_KEYS {
+        assert!(v.get(*k).is_some(), "pre-P-24 key {k} missing");
+    }
+    assert_eq!(v["strength"]["per_agent_accounts"], false);
+    assert!(v["missing_strength"].is_array());
+
+    // E-55 schema: each field stamped fresh (value, at_ms, bound_ms) or unknown (shown, last_at_ms)
+    for k in P24_KEYS {
+        assert!(v.get(*k).is_some(), "P-24 key {k} missing");
+    }
+    for n in FIELD_NAMES {
+        let f = &v["fields"][*n];
+        match f["state"].as_str() {
+            Some("fresh") => assert!(f.get("value").is_some() && f["at_ms"].is_u64() && f["bound_ms"].is_u64(), "{n}: {f}"),
+            Some("unknown") => assert!(f["shown"].as_str().is_some_and(|s| s.starts_with("unknown (")) && f.get("value").is_none(), "{n}: {f}"),
+            _ => panic!("field {n} has no state: {f}"),
+        }
+    }
+
+    // 06a 2.13.f: nothing has been measured in this process (no pull, no heartbeat): unknown, never CLEAR
+    for n in ["hub", "revocation", "consumers", "supervision", "skew"] {
+        assert_eq!(v["fields"][n]["state"], "unknown", "{n} must read unknown before its producer ran");
+    }
+    assert_eq!(v["clear"]["state"], "NOT CLEAR");
+    assert!(v["clear"]["unknown"].as_array().unwrap().len() >= 5, "{}", v["clear"]);
+
+    // D-2: quota names the absence; computed fields are fresh
+    assert_eq!(v["fields"]["quota"]["value"], "absent: receiver quotas not built (P-17)");
+    assert_eq!(v["fields"]["mode"]["state"], "fresh");
+    assert_eq!(v["strength_host"]["operator_key"], "no");
+
+    // design 3.9: the call left one latency line
+    assert_eq!(v["latency_log"], "ok");
+    let log = std::fs::read_to_string(store.dir().join(LATENCY_LOG)).unwrap();
+    let last: Value = serde_json::from_str(log.lines().last().unwrap()).unwrap();
+    assert!(last["ms"].is_u64() && last["clear"] == "NOT CLEAR", "{last}");
+
+    // design 3.0 rollback: the switch hides every new key and leaves the old ones
+    // SAFETY (test): this binary has one test; nothing else reads the variable concurrently.
+    unsafe { std::env::set_var("TERMLINK_SIDECAR_P24", "off") };
+    let off = status(&e, &store);
+    unsafe { std::env::remove_var("TERMLINK_SIDECAR_P24") };
+    for k in P24_KEYS {
+        assert!(off.get(*k).is_none(), "{k} shown with the switch off");
+    }
+    for k in PRE_P24_KEYS {
+        assert!(off.get(*k).is_some(), "pre-P-24 key {k} missing with the switch off");
+    }
+}
diff --git a/crates/termlink-sidecar/tests/policy_tests.rs b/crates/termlink-sidecar/tests/policy_tests.rs
index b095cb25c..e11a4ed3d 100644
--- a/crates/termlink-sidecar/tests/policy_tests.rs
+++ b/crates/termlink-sidecar/tests/policy_tests.rs
@@ -377,3 +377,23 @@ fn t3703_stale_gate_keeps_urgent_only_for_the_own_project_while_narrowing() {
     assert!(stale_gate(true, Trust::Admitted, false), "not stale: the allow-list decides");
     assert!(!stale_gate(false, Trust::OwnProject, true), "the gate never raises urgency");
 }
+
+// ---- T-3708 (carried from T-3703 r3 CC N4): the DOWNGRADED reason names a stale-gate downgrade -------
+
+#[test]
+fn t3708_downgrade_reason_names_the_stale_list_only_when_the_gate_caused_it() {
+    use termlink_sidecar::policy::{Trust, downgrade_why, downgrade_why_with};
+    // today the urgent allow-list is the own project only: every downgrade names the allow-list
+    for narrowing in [false, true] {
+        let w = downgrade_why(7, Trust::Admitted, narrowing);
+        assert!(w.contains("urgent needs the allow-list") && !w.contains("revocation list stale"), "{w}");
+    }
+    // widened to admitted senders: while narrowing, the downgrade is the stale gate's and says so
+    fn widened(t: Trust) -> bool {
+        matches!(t, Trust::OwnProject | Trust::Admitted)
+    }
+    let w = downgrade_why_with(7, Trust::Admitted, true, widened);
+    assert!(w.contains("revocation list stale"), "{w}");
+    // not narrowing, a widened sender is not downgraded by the gate; an untrusted one names the allow-list
+    assert!(downgrade_why_with(7, Trust::Untrusted, true, widened).contains("urgent needs the allow-list"));
+}
```
