# Stale-waker-code detection canary (T-2405)

**Question it answers:** *Are any of my LIVE push-wakers running old code?*

## Why this exists (G-019)

T-2404 shipped `scripts/fleet-rearm-wakers.sh` — the **remediation** for a push-waker
process still executing pre-current code after the waker script was patched (e.g. a
T-2402 idle-gating fix). But a push-waker is a long-lived detached process: patching
`scripts/be-reachable-pushwaker.sh` does **not** touch the already-running waker, which
keeps executing the old code until it is re-armed. Nothing surfaced that drift — it was
found only by a manual `/proc/<pid>` mtime compare.

G-019 says: fix the symptom, then ask *why was the framework blind?* and close the
blindness. This canary is the **detection** sibling of T-2404's remediation.

## The three staleness canaries (distinct layers)

| Canary | Task | Layer | Fires when |
|---|---|---|---|
| fleet-binary-freshness | T-2359 | hub binary | a reachable hub serves a version below its floor |
| waker-liveness | T-2387 | waker *running-ness* | a LIVE listener has no waker, or a waker pid is dead |
| **stale-waker-code (this)** | **T-2405** | waker *code version* | a LIVE waker's process predates the current waker script |

They are complementary: T-2387 tells you a waker is **dead**; T-2405 tells you a waker is
**alive but old**.

## How it works

Walks `$HOME/.termlink/be-reachable-<id>.state` (the same per-agent state files
`fleet-rearm-wakers.sh` re-arms) and classifies each `pushwaker_pid`:

- **STALE** (firing) — pid alive **and** `/proc/<pid>` start-mtime `<` current
  `be-reachable-pushwaker.sh` mtime. The waker is running old code.
- **current** (healthy) — pid alive and not older than the script.
- **not-running** (informational, non-firing) — pid dead/absent. This is the
  *waker-liveness* class (T-2387), not the old-code class — a dead waker cannot be
  "running old code".

It reuses T-2404's exact staleness primitives (`code_mtime` / `proc_start_mtime` /
`is_stale`) so detection and remediation cannot drift apart.

## Usage

```bash
bash scripts/check-stale-waker-code-freshness.sh          # human summary
bash scripts/check-stale-waker-code-freshness.sh --json    # {ok, stale[], current[], not_running[], summary}
bash scripts/check-stale-waker-code-freshness.sh --quiet    # print only on firing (cron form)
```

Exit codes: **0** = healthy (no stale wakers), **1** = firing (≥1 stale), **2** = tooling
error (waker script not found).

## Operator action on firing

Re-arm the named agent(s) — a zero-outage waker-only respawn (heartbeat/presence never
drops, per T-2404):

```bash
bash scripts/fleet-rearm-wakers.sh <agent>     # one agent
bash scripts/fleet-rearm-wakers.sh --all       # roll the whole fleet
```

## Cron

Daily at 08:13 UTC (`.context/cron/stale-waker-code-canary.crontab`, installed to
`/etc/cron.d/termlink-stale-waker-code-canary`). Appends to
`.context/working/.stale-waker-code-canary.log` — **empty log = healthy**, the same
convention as the other nine canaries. `/canaries` auto-discovers it via the log +
`.heartbeat` companion.

## Testing

Hermetic test `tests/stale-waker-code-canary.sh` — a fixture state dir + a fake waker
script whose mtime is controlled, with `$$` (the test's own shell, a guaranteed-live pid)
standing in for a running waker. Env hooks: `STALE_WAKER_STATE_DIR`,
`STALE_WAKER_PW_SCRIPT`, `STALE_WAKER_LIB=1` (source pure helpers without running main).
