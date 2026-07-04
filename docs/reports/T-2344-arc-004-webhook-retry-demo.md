# T-2344 — arc-004 webhook retry-loop E2E regression demo (flaky sink)

**Task:** T-2344 · **Arc:** arc-004 push-transport (Candidate B) · **Date:** 2026-07-04
**Artifact type:** regression-coverage add (reusable E2E reproducer, resilience path)

## What this closes

The webhook retry/backoff chain (T-2334: `classify_outcome` → `schedule_retry` →
`spawn_retry_loop` drain → re-dispatch) had 18 unit tests on the pure mechanics
but **no live E2E evidence** — every prior smoke/demo (T-2336 manual smoke, T-2343
fan-out demo) exercised only the direct-success dispatch. This is the exact
T-2341/PL-240 defect class: a background resilience loop can be unit-green yet
**unreachable in the shipped wiring** — if `spawn_retry_loop` were unwired, every
transient 5xx would silently dead-letter and nothing would surface it.
`scripts/demo-webhook-retry.sh` closes the gap with a **flaky sink** (503 for the
first 2 POSTs, then 204) driving a real isolated hub through fail-then-recover.

## What it proves (end-to-end)

One `channel.post` on the target's topic; the sink 503s the initial dispatch, so
only the **background retry loop** can produce a later 204-served delivery
(`fan_out` fires exactly once per post — nothing else re-sends):

1. **Retryable failure engaged:** ≥1 sink request served 503 (classified
   `Retryable`, enqueued).
2. **Retry loop re-dispatched:** a LATER request served 204 — the same signed
   payload, re-sent by `spawn_retry_loop` (interval set to 500 ms via
   `TERMLINK_WEBHOOK_RETRY_INTERVAL_MS` for a fast deterministic run; attempt-1
   backoff ≈ 2 s ± jitter).
3. **HMAC intact through retry:** the signature on the 204-served body verifies
   against the configured `signing_key` (raw UTF-8 key bytes, per T-2343).
4. **Telemetry moved (T-2335):** `hub status --governor --json` shows
   `.governor.webhook_retry_success_total >= 1` and `webhook_enqueued_total >= 1`.

## Result (2026-07-04, fresh `target/release/termlink`)

```
=== arc-004 webhook retry-loop E2E demo (T-2344, proves T-2334 wiring) ===
flaky sink:          http://127.0.0.1:8802/hook   (503 x2 then 204)
deliveries:          503-served=2  204-served=1  total=3
HMAC on retried:     yes   (signature verifies on the 204-served body)
governor telemetry:  webhook_retry_success_total=1  webhook_enqueued_total=2
RESULT: PASS — spawn_retry_loop wiring is LIVE.
```

`enqueued_total=2` reads correctly: the inline first attempt failed → enqueue (1);
the first background retry hit the sink's second 503 → re-enqueue (2); the second
retry landed 204 → `retry_success_total=1`. PASS on the first run — no product
defect (the wiring was correct); the value is the reproducer: a future regression
in the retry classification, the queue drain, or the loop wiring now fails a
script instead of silently dead-lettering.

Exit codes: `0` PASS · `2` binary missing/pre-webhook · `3` hub/sink failed ·
`4` python3 missing · `5` no initial 503 · `6` no recovered 204 (retry loop dead)
· `7` HMAC mismatch on retried body · `8` retry telemetry did not move.

Isolation contract: temp `TERMLINK_RUNTIME_DIR` + temp `HOME` + loopback sink;
never touches `:9100` or `~/.termlink`; teardown on exit.

## Verification

- `bash -n scripts/demo-webhook-retry.sh` — clean.
- `scripts/demo-webhook-retry.sh` — PASS (exit 0), output above.
- No Rust changed (shell-only add) — exercises shipped T-2334/T-2335 code.

## Files

- `scripts/demo-webhook-retry.sh` — the flaky-sink retry-loop reproducer.
- `docs/reports/T-2344-arc-004-webhook-retry-demo.md` — this report.
