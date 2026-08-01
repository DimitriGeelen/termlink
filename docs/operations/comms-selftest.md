# Comms round-trip self-test (T-2482)

## What it answers

**"Why is there still no response?"** — attributed to the exact link that broke.

TermLink's charter promise is that a fleet of agents can *discover each other* and
*exchange durable messages*. The codebase has 11 failure-detecting canaries and
four observability arcs — but they all detect the substrate breaking **after the
fact**, on a daily cron. There was no single on-demand command to **affirmatively
prove the full round-trip works right now** and, when it doesn't, say whether the
break is in **discover**, **send**, or **consume**.

`agent-send.sh --to <id>` already composes those three internally, but it
collapses to one exit code, is negative-framed (fires a real turn, fails loud),
and can't say `DISCOVER=PASS / SEND=PASS / CONSUME=FAIL`. `comms-selftest.sh` is
the thin staged prover that turns that collapse into a per-stage breakdown.

## Usage

```bash
# Side-effect-free health check: is the peer reachable + armed? (no turn fired)
bash scripts/comms-selftest.sh --peer <agent_id> --discover-only

# Full round-trip proof (fires ONE synthetic proof-ping; the peer's receipt is the proof)
bash scripts/comms-selftest.sh --peer <agent_id>

# Machine-readable staged breakdown
bash scripts/comms-selftest.sh --peer <agent_id> --json
```

## The three stages

| Stage | Proves | Reuses |
|-------|--------|--------|
| **DISCOVER** | peer is LIVE **and armed** (`pty_session` present) — a send would be consumable | `scripts/diagnose-unconsumed.sh` (T-2479) |
| **SEND** | a durable turn was written to the hub | `scripts/agent-send.sh` (arc-003) |
| **CONSUME** | the peer actually posted a receipt (consumed it) | `agent-send.sh`'s receipt wait |

A DISCOVER failure **stops before a pointless send** and is attributed as `dead`
(no live presence) or `unwakeable` (LIVE but nothing can ring its PTY). A CONSUME
failure carries the `diagnose-unconsumed.sh` class (`busy-or-manual` = LIVE+armed
but the session isn't consuming — G-083).

## Exit codes

| Exit | Meaning |
|------|---------|
| 0 | round-trip proven (every run stage PASSed) |
| 1 | round-trip broken — the report names the stage |
| 2 | tooling error (could not read a required signal). **Fail-closed** — an un-probeable peer is never reported "proven" |

The SEND stage is bounded by `COMMS_SELFTEST_SEND_TIMEOUT` (default 120s) so the
prover **never hangs**: no receipt within the bound renders as
`CONSUME=FAIL(timeout)`, not a wedged terminal.

## Example (live, against a busy peer)

```json
{"peer":"aef","verdict":"broken","broken_stage":"CONSUME",
 "stages":[
   {"stage":"DISCOVER","status":"PASS","detail":"peer is LIVE + armed"},
   {"stage":"SEND","status":"PASS","detail":"durable turn written to the hub"},
   {"stage":"CONSUME","status":"FAIL","detail":"no receipt within 8s (timed out; peer busy/manual: busy-or-manual, G-083)"}]}
```

The message was durably written; the peer is armed; it just isn't consuming.
That is a precise, actionable answer — not silence.

## Test hooks (PL-213 — host-independent)

- `TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON=<file>` — inherited by the DISCOVER stage.
- `COMMS_SELFTEST_TEST_SEND_RC=<int>` — canned `agent-send.sh` exit code (skips the
  real send) for the SEND+CONSUME stages.

`bash scripts/test-comms-selftest.sh` runs the full matrix with no live hub.

## Related

- The **affirmative complement** to the 11 failure-detecting canaries: "prove it
  works" vs "detect when it broke".
- T-2479 `diagnose-unconsumed.sh` (P2, G-083) — the DISCOVER + CONSUME attribution.
- T-2480 `arc-live-probe.sh` (P3a, G-069) — proves a *capability* is served; this
  proves a *message round-trip* is served.
- `agent-conversation-selftest.sh` (T-1829) — loopback-only self-check; this is the
  live-peer counterpart it could never be.
