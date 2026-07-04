# T-2343 — arc-004 webhook fan-out isolated-hub regression demo

**Task:** T-2343 · **Arc:** arc-004 push-transport (Candidate B) · **Date:** 2026-07-04
**Artifact type:** regression-coverage add (reusable E2E reproducer, security-critical path)

## What this closes

arc-004 Candidate B — the webhook fan-out (S1–S6, `T-2332`…`T-2337`) — is
feature-complete and **security-critical**: outbound HTTP from the hub guarded by
a deny-by-default **exact-host SSRF allowlist** and **HMAC-SHA256 payload signing**
(`X-Termlink-Signature: sha256=<hex>`). It shipped with unit tests and a **one-time
manual smoke** (`T-2336`) but no reusable reproducer — the same gap `T-2342` closed
for the dm rail, on the higher-stakes candidate (a silent regression in the SSRF
guard or the HMAC is a *security* defect, not a missed notification).
`scripts/demo-webhook-fanout.sh` drives the full `channel.post → fan_out →
dispatch → HTTP` path against an isolated hub + a local sink and verifies the
signature over the wire.

## What it proves (end-to-end, real hub + real HTTP + real HMAC — no stub)

Against an **isolated** hub (temp `TERMLINK_RUNTIME_DIR` + temp `HOME` + a loopback
python sink, never touches `:9100` or `~/.termlink`), started with a
`TERMLINK_WEBHOOK_CONFIG` that allowlists `127.0.0.1` and points one target
(topic-filtered) at the sink:

1. **A — POSITIVE.** A `channel.post` on the target's matching topic fans out to
   the sink as a real signed POST. The demo recomputes HMAC-SHA256 over the **raw
   received body** with the configured `signing_key` and asserts it equals the
   `x-termlink-signature: sha256=<hex>` header. This exercises `sign_payload`
   (T-2332), `fan_out`/`targets_for` (T-2333), and the `channel.rs` `Ok(offset)`
   arm wiring — not a stub.
2. **B — TOPIC-FILTER negative.** A `channel.post` to a *non-matching* topic
   delivers nothing to the sink (the `topics` filter gates fan-out).
3. **C — SSRF deny-by-default.** `webhook test` against the `169.254.169.254`
   cloud-metadata address (not on the allowlist) is refused **loudly** — non-zero
   exit, `host not allowlisted (SSRF guard)` — with no delivery to the sink. This
   is the production `webhook::dispatch` guard at the CLI surface (PL-239: a
   test verb must run the real guard, never auto-permit its own target).

### Key correctness detail — the signing key is raw bytes

`webhook.rs sign_payload` keys the HMAC with `signing_key.as_bytes()` — the **raw
UTF-8 bytes of the key string**, NOT hex-decoded. The demo's recompute mirrors the
operator recipe's consumer-verify snippet (`hmac.new(key.encode(), body,
sha256)`), so a drift in key handling on either side fails the demo.

## Result (2026-07-04)

```
=== arc-004 webhook fan-out isolated-hub demo (T-2343, proves T-2332/T-2333) ===
hub:                 127.0.0.1:9200   (isolated, TERMLINK_WEBHOOK_CONFIG loaded=true)
positive delivery:   sink POSTs 0 -> 1   (>=1 signed POST on the matching topic)
HMAC verified:       yes   (recomputed sha256 over raw body == X-Termlink-Signature)
topic-filter negative: non-matching 'webhook-nomatch-<pid>' -> no new delivery
SSRF refused:        rc=1  Error: webhook test failed: host not allowlisted (SSRF guard): http://169.254.169.254/latest/meta-data/
RESULT: PASS
```

Exit codes: `0` PASS · `2` binary missing / predates the webhook subsystem · `3`
hub/sink failed · `4` python3 missing (sink) · `5` no signed delivery · `6` HMAC
mismatch · `7` topic-filter leak · `8` SSRF guard did not refuse.

## Defect caught while authoring — stale release binary (no product defect)

First run FAILED with `positive delivery: 0 -> 0` and `webhook_enabled=n/a`. Root
cause was **not** a webhook bug: the repo's `target/release/termlink` (built
09:20, before the webhook slices landed later the same day) contained **zero**
webhook symbols (`grep -a -c x-termlink-signature` = 0), so `webhook::init` did
not exist and the config never loaded. Two responses:

1. **Added a loud binary-guard** to the demo (mirror of the `dm.queued` guard in
   `demo-dm-rail-pushwake.sh`): if the binary lacks `x-termlink-signature`, fail
   with a clear "predates the webhook subsystem — rebuild" message (exit 2) rather
   than a mystery no-delivery.
2. **Rebuilt `target/release`** so the default-path run works and the stale
   pre-webhook release binary is refreshed.

Also fixed a cosmetic display bug: the `webhook_enabled` indicator read the wrong
jq path (`.webhook_enabled` — the field is nested under `.governor` in
`hub status --governor --json`); corrected to `.governor.webhook_enabled`. The
positive delivery already proves the config loaded (`fan_out` only fires when the
subsystem is enabled), so this was display-only.

Like T-2342, no *product* defect surfaced (the webhook code was already correct);
the value is the reusable reproducer + the two authoring-time fixes above.

## Verification

- `bash -n scripts/demo-webhook-fanout.sh` — clean.
- `scripts/demo-webhook-fanout.sh` — PASS (exit 0), output above.
- No Rust changed (shell-only add + a release rebuild of existing code) — the demo
  *exercises* the already-shipped T-2332/T-2333 code against a real hub + sink.

## Files

- `scripts/demo-webhook-fanout.sh` — the isolated-hub webhook fan-out reproducer.
- `docs/reports/T-2343-arc-004-webhook-fanout-demo.md` — this report.
