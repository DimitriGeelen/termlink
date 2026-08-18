---
id: T-2691
name: "Canary heartbeat proves scheduling, not completion — a hung or killed canary still writes a fresh heartbeat (24/24 scripts)"
description: >
  Every canary script touches its `.heartbeat` in the first 30-50% of the file,
  before doing any work. The meta-canary (T-1723) reads heartbeat freshness to
  answer "is this canary still completing runs?" — but the signal only ever
  answered "did cron fire?". T-2690 closed the ERRORING half of this class by
  making the stderr sink a read surface; the remaining hole is a canary that
  HANGS or is KILLED: fresh heartbeat, no log, no stderr, invisible everywhere.

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: [governance, canary, observability]
components: [scripts/check-canary-aliveness.sh]
related_tasks: [T-2690, T-1723, T-2172]
created: 2026-08-18T21:45:00Z
last_update: 2026-08-18T22:02:48Z
date_finished: null
---

# T-2691: Canary heartbeat proves scheduling, not completion

## Context

Split out of T-2690 (one bug = one task). T-2690 established that three choices
composed into a blind spot in the canary layer and closed two of them — the
unread stderr sink (now the `ERRORING` class) and the four phantom `STALE`
static checks (now `NOT_SCHEDULED`). The **third** is untouched and is the
deepest of the three:

> The heartbeat is written *before the work*, so its freshness proves the script
> **started**, never that it **finished**.

Measured across the tree — every heartbeat-writing script, write position as a
fraction of file length:

| Script | heartbeat write | file length |
|---|---|---|
| `check-mirror-freshness.sh` | line 50 | 175 |
| `check-dead-letter-freshness.sh` | line 60 | 117 |
| `check-framework-pickup-freshness.sh` | line 61 | 188 |
| `check-preflight-doc-set-drift.sh` | line 63 | 209 |
| `check-stuck-claims-freshness.sh` | line 65 | 133 |
| `check-session-control-freshness.sh` | line 65 | 106 |
| `check-task-finalization-freshness.sh` | line 66 | 188 |
| `check-unconfirmed-delivery-freshness.sh` | line 67 | 128 |
| `check-charter-sentence-drift.sh` | line 68 | 149 |
| `check-charter-drift-freshness.sh` | line 70 | 119 |
| `check-canary-aliveness.sh` | line 77 | 130 |
| `check-topic-growth-freshness.sh` | line 79 | 206 |
| `check-waker-liveness-freshness.sh` | line 79 | 259 |
| `check-fleet-binary-freshness.sh` | line 81 | 198 |
| `check-fleet-doorbell-mail-health.sh` | line 82 | 263 |
| `check-forever-archival-freshness.sh` | line 85 | 200 |
| `check-drain-sink-caps.sh` | line 86 | 198 |
| `check-alloc-sink-clamps.sh` | line 88 | 224 |
| `check-frozen-husk-freshness.sh` | line 88 | 282 |
| `check-busy-spin.sh` | line 98 | 229 |
| `check-silent-exit.sh` | line 100 | 220 |
| `check-fleet-capability-freshness.sh` | line 103 | 242 |
| `check-stale-waker-code-freshness.sh` | line 129 | 210 |
| `substrate-preflight.sh` | line 153 | 613 |

24 of 24. In every case the write sits just after argument parsing and before
the probe. So a canary that **hangs** (a hub RPC with no wall-clock bound) or is
**killed** (cron timeout, OOM) leaves: heartbeat FRESH → meta-canary says ALIVE;
firing log EMPTY → `/canaries` says HEALTHY; stderr sink EMPTY → `ERRORING` does
not fire. Invisible on all three surfaces, which is the same shape T-2690 fixed
for the error case.

## Acceptance Criteria

### Agent
- [x] Heartbeat is written on **completion**, not on start, so its freshness means
      "this canary finished a run" — the property the meta-canary (T-1723) already
      assumes it has. 23 of 24 heartbeat-WRITING scripts migrated; the 24th
      (`check-canary-aliveness.sh`) only READS a heartbeat, so it is correctly untouched.
- [x] The change composes with each script's existing cleanup. The migration preserves
      each script's heartbeat body **verbatim** (three variants exist in-tree: a
      `date`-write, a `touch --`, and a bare `touch`) and reuses its guard line, so
      `-eq 1` vs `= 1` spellings survive too. The 4 scripts that already own an
      `EXIT` trap were migrated by hand: their helper is self-guarding and is CHAINED
      into the existing trap (`trap 'rm -f -- "$LIST_TMP"; _canary_hb' EXIT`) rather
      than replacing it.
- [x] Those 4 also arm an **early** `trap _canary_hb EXIT` at the helper definition,
      because their combined trap is installed late (next to the tmpfile) — without it
      a tooling-error `exit 2` on an earlier line would skip the heartbeat entirely,
      inconsistent with the other 19.
- [x] `--no-heartbeat` continues to suppress the write entirely (fixture Case 2).
- [x] A fixture proves the property is load-bearing: `tests/canary-heartbeat-fixtures.sh`
      Case 3 hangs a real canary on a FIFO and SIGKILLs it (EXIT traps do not run on
      SIGKILL) — heartbeat stays stale. Verified load-bearing by re-introducing the eager
      write, which makes Case 3 FAIL, and restoring, which makes it pass.
- [x] Case 5 guards the clobber hazard directly: the pre-existing tmpfile cleanup still
      fires after chaining.
- [x] Regression-checked: `tests/canary-status-fixtures.sh` 8/8, all four static checks
      clean, `check-cron-install-drift` 0 drift, charter-sentence + task-finalization
      canaries unchanged.

Scope note: the fixture's FIRST draft silently lied — it injected `HEARTBEAT_FILE` into
`check-dead-letter-freshness.sh`, which **hard-assigns** that variable, so every case
measured an untouched temp file and Case 3 "passed" for the wrong reason. The suite now
opens with Case 0 asserting the canary under test honours an injected `HEARTBEAT_FILE`,
so that failure mode cannot recur silently.

## Verification

bash tests/canary-heartbeat-fixtures.sh
bash tests/canary-status-fixtures.sh
bash scripts/check-alloc-sink-clamps.sh
bash scripts/check-drain-sink-caps.sh
bash scripts/check-busy-spin.sh
bash scripts/check-silent-exit.sh

## Decisions

### 2026-08-18 — Filed rather than built inside T-2690
- **Chose:** Split this out as its own task instead of folding it into the T-2690 fix.
- **Why:** It is a distinct root cause with a distinct blast radius — 24 scripts, several
  of which already own an `EXIT` trap for tmpdir cleanup. A blanket `trap` rewrite would
  silently drop that cleanup, and a bug introduced into a canary's own cleanup path is
  worse than the gap being closed: the detection layer would start leaking while still
  reporting green. T-2690's stderr reader already covers the common case (a canary that
  runs and errors); what remains here is the rarer hang/kill case, so the correct
  sequencing is "ship the safe fix, file the risky one with its evidence".
- **Rejected:** Doing the 24-file trap migration opportunistically inside T-2690 — that
  would have made one commit carry both a verified fix and an unverified sweep.

### 2026-08-18T22:02:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
