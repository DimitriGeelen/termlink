# Webhook fan-out operator recipe (arc-004, Candidate B)

**What this is:** the single operator-facing walkthrough for the shipped arc-004
webhook fan-out capability — the hub POSTing signed, allowlisted HTTP payloads to
external consumers (Watchtower / Slack / CI / any HTTP endpoint) when a bus topic
receives a post. It consolidates the five build slices (T-2332…T-2336) into one
navigation hub.

**Status:** the webhook subsystem is **feature-complete** (S1–S5 shipped). It is
**opt-in** — a hub with zero configured targets is entirely unaffected (no thread,
no client, no behaviour change). Origin decision: the T-2331 inception recommended
DEFER (agents are not HTTP servers, so webhooks are external-only fan-out with no
in-fleet consumer); the human OVERRODE to GO. See
`docs/reports/T-2331-webhooks-external-fan-out-inception.md`.

---

## 1. Mental model — one-way, opt-in, external fan-out

```
   ┌─────────────────────────────────────────────────────────────┐
   │ DURABLE BUS (authoritative, unchanged):                      │
   │   channel.post → topic → offset (dm:/inbox:/agent-* …)       │
   └──────────────────────────┬──────────────────────────────────┘
                              │ (post succeeded → Ok(offset) arm)
                              ▼
   ┌─────────────────────────────────────────────────────────────┐
   │ WEBHOOK FAN-OUT (opt-in, best-effort, one-way OUTBOUND):     │
   │   for each target whose `topics` match this topic:           │
   │     POST body  +  X-Termlink-Signature: sha256=<hmac>        │
   │     → only if the URL host is on the allowlist (SSRF guard)   │
   └─────────────────────────────────────────────────────────────┘
```

**Invariants:**

- **Fan-out never gates the post.** A webhook only fires from the *success* arm of
  `channel.post` — a failed post fans out nothing, and a failed/refused webhook
  never affects the durable write. The bus is the source of truth; webhooks are a
  side-effect notification.
- **One-way only.** The hub POSTs *out*. There is no inbound webhook receiver —
  agents are not HTTP servers. This is external fan-out to non-TermLink consumers.
- **Opt-in / no hard dependency (Directive 4 — Portability).** Zero targets ⇒
  `WebhookConfig::is_enabled()` is false ⇒ the whole subsystem is a no-op.

## 2. The security model — two independent layers

Outbound HTTP from a hub is a classic SSRF liability, so the feature is
security-first and **deny-by-default** on both axes:

### 2a. SSRF host allowlist (exact-match, deny-by-default)

A target only dispatches if its URL **host is an exact member** of
`allowed_hosts`. No suffix or substring matching — `hooks.example.com.evil.com`
never matches `hooks.example.com`. The check runs **before any network activity**,
so a cloud-metadata SSRF probe (`http://169.254.169.254/…`) is refused without the
hub ever opening a socket. An empty allowlist ⇒ nothing dispatches.

### 2b. HMAC-SHA256 payload signing

Every POST carries an `X-Termlink-Signature: sha256=<hex>` header — HMAC-SHA256 of
the exact request body, keyed by the target's `signing_key`. The consumer
recomputes the HMAC over the received bytes and compares; a mismatch means the
payload was not sent by a hub holding that key.

> The `signing_key` is **distinct from** the hub's peer-auth `hub.secret`. A
> compromised webhook key grants an attacker nothing on the TermLink substrate —
> it only lets them forge payloads to *that one consumer*. Never reuse `hub.secret`
> as a webhook signing key.

**Consumer-side verification** (Python, framework-agnostic — the header format is
`sha256=<lowercase-hex>`):

```python
import hmac, hashlib

def verify(signing_key: str, body: bytes, header: str) -> bool:
    expected = "sha256=" + hmac.new(
        signing_key.encode(), body, hashlib.sha256
    ).hexdigest()
    # constant-time compare — do NOT use `==`
    return hmac.compare_digest(expected, header)

# in your HTTP handler:
#   ok = verify(MY_KEY, raw_request_body, request.headers["X-Termlink-Signature"])
#   if not ok: return 401
```

Verify over the **raw received bytes**, before any JSON re-serialization —
re-encoding can reorder keys and break the signature.

## 3. Config surface — `TERMLINK_WEBHOOK_CONFIG`

The hub loads targets at startup from the JSON file named by the
`TERMLINK_WEBHOOK_CONFIG` env var (unset / unreadable / unparseable / zero-targets
⇒ subsystem disabled, no panic). Schema:

```json
{
  "allowed_hosts": ["hooks.example.com", "ci.internal"],
  "targets": [
    {
      "url": "https://hooks.example.com/termlink",
      "signing_key": "9a946935c41cabc3e83f4ab1b031e047d37a6e45659115a0e22361c35d85d601",
      "topics": ["agent-presence", "channel:learnings"]
    },
    {
      "url": "https://ci.internal/hook",
      "signing_key": "…",
      "topics": ["*"]
    }
  ]
}
```

Field semantics:

| Field | Meaning |
|---|---|
| `allowed_hosts` | Exact-match SSRF allowlist. A target whose URL host is absent here is **refused** at dispatch. |
| `targets[].url` | Absolute `http`/`https` URL the hub POSTs to. |
| `targets[].signing_key` | HMAC-SHA256 key (see §2b). Share with the consumer out-of-band. |
| `targets[].topics` | Topics that fire this target: exact membership **or** the `"*"` wildcard. An **empty list never fires** (opt-in by construction). |

Set the env var wherever the hub is launched (systemd `Environment=`, watchdog
script `export`, etc.) and restart the hub to load changes — the config is read
once at startup into a process-global `OnceLock` runtime.

## 4. The CLI config verbs (T-2336) — author without hand-editing JSON

Three `termlink webhook` verbs manage the config file so operators never hand-edit
JSON. All resolve the config path from `--config <PATH>` if given, else the
`TERMLINK_WEBHOOK_CONFIG` env var — there is **no silent default path** (a webhook
config holds signing keys + an SSRF allowlist, so an unspecified location is an
error, not a guess).

### `webhook add` — merge a target

```bash
termlink webhook add \
  --url https://hooks.example.com/termlink \
  --topic agent-presence --topic channel:learnings \
  --config ~/.termlink/webhooks.json
```

- Reads the existing config (or starts empty), appends the target, and
  **auto-adds the URL's host to `allowed_hosts`** when absent (a target whose host
  is not allowlisted would never dispatch — `add` wires it in for you). Add extra
  hosts (e.g. a redirect target) with repeated `--allowed-host H`.
- `--signing-key K` sets the key explicitly; **omit it and a random 32-byte key is
  generated** (printed once — copy it to the consumer). `--topic` is repeatable;
  use `--topic '*'` for all topics.
- Writes atomically (temp + rename). Refuses a URL that is not http/https or has no
  parseable host.

### `webhook list` — inspect the config

```bash
termlink webhook list --config ~/.termlink/webhooks.json
termlink webhook list --config ~/.termlink/webhooks.json --json
```

- Text output renders each target's url / host / topics with the **signing key
  REDACTED** (shown as `<redacted, N chars>` — never printed to a terminal or log).
  It flags any target whose host is **not** allowlisted (`⚠ NOT-ALLOWLISTED`).
- An empty / missing config prints an explicit `webhook fan-out disabled
  (0 targets)` line — not a silent blank.
- `--json` emits the parsed config **verbatim** (including keys — it already lives
  in plaintext on disk; this is the machine surface you asked for).

### `webhook test` — smoke-test a target with a signed sample payload

```bash
# a configured target (key + allowlist sourced from the config):
termlink webhook test --url https://hooks.example.com/termlink \
  --config ~/.termlink/webhooks.json

# an ad-hoc URL you must explicitly permit:
termlink webhook test --url http://127.0.0.1:8799/hook \
  --signing-key mysecret --allowed-host 127.0.0.1 --topic demo
```

Dispatches a signed sample payload by **reusing the exact production
`webhook::dispatch` path** — the SSRF host-allowlist guard and HMAC signing run
identically to a real fan-out — and reports the HTTP status code (or the classified
error). It sources the signing key + allowlist from a matching config target by
default; `--signing-key` / `--allowed-host` override or augment.

> **Gotcha (PL-239) — `test` is deny-by-default and does NOT auto-permit the tested
> host.** If the URL's host is not on the effective allowlist, `test` refuses
> **loudly** (non-zero exit) with a `--allowed-host` hint — because that IS the
> useful signal: *"this target would be SSRF-refused in production."* A `test` verb
> that auto-permitted its own target would print a green `✓ dispatched` for a URL
> the hub would actually refuse, silently bypassing the guard it exists to
> exercise. Permit an ad-hoc host explicitly with `--allowed-host <host>`.

## 5. Retry, backoff, and dead-letter (T-2334)

Dispatch outcomes are classified and handled automatically:

| Outcome | Trigger | Action |
|---|---|---|
| Success | HTTP 2xx | done |
| Permanent drop | HTTP 4xx, or a config error (invalid URL / host not allowlisted) | dropped + logged (retrying won't help) |
| Retryable | HTTP 5xx or a transport error (timeout, connection refused) | enqueued for retry with backoff |

- Retries use monotonic exponential backoff capped at 60 s, with ±25 % jitter
  (derived from wall-clock nanoseconds — no `rand` dependency).
- The retry queue is **in-memory and bounded** (`TERMLINK_WEBHOOK_RETRY_CAP`,
  default 1000). When full, further enqueues are **loudly refused and counted**
  (never silently dropped).
- A payload that keeps failing is dead-lettered after **5 attempts**
  (`WEBHOOK_MAX_ATTEMPTS`) into a bounded dead-letter ring with counters.
- The background retry loop drains due entries every
  `TERMLINK_WEBHOOK_RETRY_INTERVAL_MS` (default 2000, clamped 250…60000).
- Per-request timeout is **10 s** (`WEBHOOK_TIMEOUT_SECS`) — a hung endpoint can
  never pin a dispatch task.

> **Durability trade-off (deliberate, PL-111):** the retry queue is in-memory, so
> in-flight retries are **lost on hub restart**. This is acceptable for a
> best-effort, opt-in notification channel; it can graduate to a durable
> `runtime_dir` table later if a consumer needs at-least-once across restarts.

## 6. Observability — `hub.governor_status` `webhook_*` fields (T-2335)

The webhook subsystem surfaces its counters as additive fields on the
`hub.governor_status` JSON-RPC response — visible via the MCP tool
`termlink_hub_governor_status` (pass-through, inherits the fields for free) and the
local render `termlink hub status --governor`:

| Field | Meaning |
|---|---|
| `webhook_enabled` | bool — a config was loaded with ≥1 target |
| `webhook_target_count` | number of configured targets |
| `webhook_retry_depth` | entries currently queued for retry |
| `webhook_enqueued_total` | lifetime retries enqueued |
| `webhook_retry_success_total` | retries that eventually succeeded |
| `webhook_dropped_full_total` | enqueues refused because the queue was full |
| `webhook_dead_letter_total` | payloads dead-lettered after max attempts |

Operational reading: a climbing `webhook_dropped_full_total` means the retry cap is
too low for the failure rate (raise `TERMLINK_WEBHOOK_RETRY_CAP` or fix the
flapping endpoint); a climbing `webhook_dead_letter_total` means a target is
persistently unreachable or rejecting (check the consumer). On a hub that predates
T-2335 these render as `n/a` (graceful degradation).

## 7. Failure modes — operational reading

| Symptom | Likely cause | Fix |
|---|---|---|
| `webhook test` → `host not allowlisted (SSRF guard)` | host absent from allowlist | `webhook add --allowed-host <host>`, or `--allowed-host` for a one-off test |
| Config edited but nothing changed | hub loads config once at startup | restart the hub |
| Target never fires despite matching posts | `topics: []` (empty never fires) | add a topic or `"*"` via `webhook add --topic` |
| `webhook_dead_letter_total` climbing | consumer down / rejecting | check the endpoint; dead-letters are dropped after 5 attempts |
| `webhook_dropped_full_total` climbing | retry cap too small for failure rate | raise `TERMLINK_WEBHOOK_RETRY_CAP` or fix the endpoint |
| Consumer sees signature mismatch | verifying re-serialized JSON, or wrong key | verify over raw received bytes; confirm the shared `signing_key` |

## 8. Map — where each piece shipped

| Slice | Task | What shipped |
|---|---|---|
| Inception (GO overriding DEFER) | T-2331 | `docs/reports/T-2331-webhooks-external-fan-out-inception.md` |
| S1 — send primitive | T-2332 | `sign_payload` + `host_allowed` (SSRF) + `dispatch` + config types |
| S2 — event → dispatch wiring | T-2333 | `targets_for` topic filter + `fan_out` + `init` from env; crypto-provider pin (PL-238) |
| S3 — retry/backoff/dead-letter | T-2334 | `classify_outcome` + backoff/jitter + bounded `RetryQueue` + retry loop |
| S4 — governor_status telemetry | T-2335 | 7 `webhook_*` fields on `hub.governor_status` (RPC + MCP + `hub status --governor`) |
| S5 — CLI config verbs | T-2336 | `termlink webhook add/list/test` (`crates/termlink-cli/src/commands/webhook.rs`) |
| S6 — this recipe | T-2337 | operator documentation |

Core implementation: `crates/termlink-hub/src/webhook.rs`. CLI verbs:
`crates/termlink-cli/src/commands/webhook.rs`.
