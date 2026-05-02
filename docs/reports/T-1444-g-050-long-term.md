# T-1444 — G-050 long-term: hub-side topic registry persistence

**Status:** Inception complete — recommendation **NO-GO**
**Date:** 2026-05-02
**Author:** claude (autonomous mandate, T-1438 field-rollout arc)

## Summary

T-1444 was captured as the long-term follow-up to T-1443's tactical
`--ensure-topic` mitigation, with the premise that hub-side topic
registration lives only in `RwLock<HashMap<String, Topic>>` with no
on-disk backing and therefore needs an option-1 persistence layer
analogous to T-1294's runtime_dir secret/cert persistence.

**This premise is refuted.** Topic registration is already persisted in
SQLite at `<runtime_dir>/bus/meta.db`. Topics survive every hub restart
where the bus root directory itself survives. The observed "topic loss"
incidents that motivated G-050/T-1443 actually trace to a different
class of failure: bus-root volatility, not in-memory registry loss.

T-1443 remains a useful client-side mitigation (callers no longer have
to remember which canon topics need to exist before posting), and the
operational lesson (PL-111: hub-side state needs runtime_dir
persistence parallel to T-1294) is correct in principle. But the
specific option-1 work this inception was commissioned to scope —
adding a hand-rolled JSON/RON-on-disk topic registry — would
**duplicate existing SQLite-backed persistence** and is therefore
unnecessary.

## Spike Findings

### S1 (1h target, 15min actual): map the in-memory registry

The relevant types live in `crates/termlink-bus/src/lib.rs`:

```rust
// lib.rs:38-46
pub struct Bus {
    root: PathBuf,
    meta: meta::Meta,                                            // ← SQLite!
    appenders: StdMutex<HashMap<String, Arc<log::LogAppender>>>, // runtime cache
    notifiers: StdMutex<HashMap<String, Arc<Notify>>>,           // runtime cache
}
```

The `appenders` and `notifiers` HashMaps are **runtime caches**, not
the registry. They're rebuilt lazily from `meta.db` on first access via
`appender_for(topic)` / `notifier_for(topic)`. The authoritative
registry is `meta::Meta`, which is a SQLite database at `<root>/meta.db`.

`create_topic` (`lib.rs:90-92`) immediately writes to SQLite:

```rust
pub fn create_topic(&self, name: &str, retention: Retention) -> Result<bool> {
    self.meta.create_topic(name, retention)
}
```

`list_topics`, `topic_exists`, `topic_retention`, `record_append`,
`records_from`, `count_records`, `trim_records` — all delegate to
`meta::Meta` and therefore all read/write SQLite.

**Conclusion:** there is no in-memory-only registry to persist.
`RwLock<HashMap<...>>` exists only as transient runtime caches that
are correctly rebuilt from durable storage.

### S2 (30min target, 5min actual): compare with T-1294 pattern

T-1294 persisted `hub.secret` and `hub.cert.pem` directly to
`<runtime_dir>/`. T-1294's contribution was **moving runtime_dir off
volatile /tmp**, not adding a new persistence layer — secret/cert
were already file-based, just on a volatile root.

The bus is the same situation. `meta.db` lives under
`<runtime_dir>/bus/`. When `runtime_dir` is `/var/lib/termlink`
(post-T-1294), the bus survives every restart. When it's
`/tmp/termlink-0` on a volatile-/tmp host (the situation T-1294
diagnosed), the bus is wiped on every reboot — same failure mode as
the secret/cert.

**Conclusion:** the G-050 incidents were T-1294 incidents in disguise.
The fix is finishing the runtime_dir migration on every hub, not a
new persistence layer.

### S3 (30min target, n/a): file format choice

Not reached — S1+S2 refuted the need.

If a future redesign were to replace SQLite (which would require
strong justification given that SQLite already does the job), the
choice would be JSON vs RON for ops-readability. Not relevant under
the current finding.

### S4 (30min target, n/a): reload-on-start path

Not reached. The reload happens implicitly via `meta::Meta::open`,
which opens the existing SQLite file. There is no separate "reload"
step to design.

### S5 (15min target, n/a): migration plan

Not reached — no migration is needed.

## Live Verification

On the local hub (`.107`, `runtime_dir=/var/lib/termlink`,
post-T-1294):

```
$ ls -la /var/lib/termlink/bus/
drwxr-xr-x 4 root root    4096 May  2 07:31 .
drwxr-xr-x 4 root root    4096 Apr 30 18:31 artifacts
-rw-r--r-- 1 root root 1306624 May  2 07:31 meta.db
drwxr-xr-x 2 root root  122880 May  2 07:01 topics

$ sqlite3 /var/lib/termlink/bus/meta.db \
    'SELECT name FROM topics WHERE name LIKE "%learning%"
     OR name LIKE "%pickup%" OR name LIKE "%chat-arc%" OR name LIKE "%broadcast%";'
agent-chat-arc
broadcast:global
channel:learnings
framework:pickup
```

`meta.db` is 1.3 MB, has the four canon topics registered, and is
being live-modified (mtime 07:31 = current). Survives any hub
restart that doesn't wipe `/var/lib/termlink/`.

## Assumption Re-evaluation

| ID | Original | Outcome |
|----|----------|---------|
| A1 | State is small/bounded — topic name + retention + topic_metadata | **REFUTED — already persisted in SQLite, no new layer needed** |
| A2 | runtime_dir filesystem is durable across hub restarts | **VALIDATED — same as T-1294's invariant** |
| A3 | Atomicity per change (write-then-rename) acceptable | **N/A — SQLite WAL already provides ACID** |
| A4 | < 10K topics per hub → JSON or RON acceptable | **N/A — SQLite scales further than this anyway** |

## Recommendation

**Recommendation:** **NO-GO** on hand-rolled topic registry persistence.

**Rationale:** The premise (in-memory-only topic registry) is wrong.
Topics are persisted to SQLite at `<runtime_dir>/bus/meta.db` via
`termlink_bus::meta::Meta`. The observed "topic loss after restart"
incidents trace to runtime_dir volatility (T-1294's domain), not to a
missing persistence layer.

**Evidence:**
- `crates/termlink-bus/src/lib.rs:38-92` — `Bus` delegates to
  `meta::Meta` (SQLite-backed) for create/list/exists/retention.
- `/var/lib/termlink/bus/meta.db` exists, is 1.3MB, holds all 4 canon
  topics, and survives current hub restart.
- The `RwLock<HashMap<...>>` referenced in T-1444's premise is a
  runtime cache (`appenders`/`notifiers`) for log handles and notify
  primitives, not the registry.

**What this means for the original gap (G-050):**
- T-1443's `--ensure-topic` flag is a useful idempotent client-side
  shortcut (callers don't have to track which canon topics need
  pre-creation). Keep it. Already deployed via T-1445 in framework
  scripts.
- The actual ground-truth long-term fix for G-050 is the **same fix
  as T-1294**: ensure every hub uses a durable `runtime_dir`. This is
  already tracked by:
  - **T-1294** (.122 ring20-management, complete)
  - **T-1296** (.121/.143 ring20-dashboard, captured/next)
  - Any other hub still on `/tmp/termlink-0` (audit needed — likely
    none on the current fleet, but this should be a periodic sweep).
- Update G-050 in `concerns.yaml`: status `mitigated` is correct;
  remove the "long-term option-1" mitigation_progress hook since
  option-1 is now refuted. Replace with: "long-term fix subsumed by
  T-1294 runtime_dir migration on every hub."

**What to do with PL-111:** The learning text reads "Hub-side state
that needs to survive restart belongs in runtime_dir alongside
T-1294's secret/cert." This is **structurally correct** — bus IS in
runtime_dir, exactly as the learning prescribes. The only edit needed
is to drop the "needs additional persistence work" implication and
instead frame it as "bus already follows this pattern; T-1294
runtime_dir migration is what protects it."

**Re-trigger conditions for re-opening this inception:**
- A topic-loss incident is observed on a hub where `runtime_dir`
  points at durable storage (e.g., `/var/lib/termlink/bus/meta.db`
  exists, has the topic in SQLite, but `termlink topics` doesn't
  return it). That would mean the cache→SQLite reload path has a
  bug, which is a different scope than this inception.
- A future requirement to persist additional state beyond what
  SQLite already covers (e.g., subscriber cursors that currently
  live client-side per T-1318 — explicitly out-of-scope here, but
  could be a separate inception).

## Decision (for human reviewer)

Run, after reviewing the SQLite check below:

    fw inception decide T-1444 no-go --rationale \
      "Premise refuted: topic registry persists in SQLite at
      <runtime_dir>/bus/meta.db (crates/termlink-bus/src/lib.rs:38-92).
      Observed topic loss traces to runtime_dir volatility (T-1294
      territory), not in-memory-only registry. Long-term fix subsumed
      by T-1294 migration on every hub (T-1296 covers .121/.143).
      T-1443 --ensure-topic remains useful as idempotent client-side
      shortcut. See docs/reports/T-1444-g-050-long-term.md."

## Dialogue Log

This inception was conducted autonomously under the user's
"proceed and continue until context at 300k" mandate. No live
dialogue occurred during the spikes. The recommendation reflects
direct code reading and live SQLite inspection on the local hub
(.107). Human reviewer is invited to verify the SQLite finding via:

    sqlite3 /var/lib/termlink/bus/meta.db .schema topics
    sqlite3 /var/lib/termlink/bus/meta.db 'SELECT name, retention FROM topics ORDER BY name;'
