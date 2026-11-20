# T-3700 review round 1 — dispositions

Seats (isolated, both `independent`): claude-code (`review-1-claude-code.md`) FAIL, 1 BLOCKER, 4 MAJOR, 10 MINOR;
opencode (`review-1-opencode.md`) FAIL, 2 MAJOR, 5 MINOR. A = accept (fixed in round 2), R = reject (reason),
D = defer (risk stated). CC n / OC n = the seat's finding number.

## Where the seats agree

| # | Finding | Seats | Disp. | Change |
|---|---|---|---|---|
| 1 | Self-spoof: an unsigned reader naming the owner (instance `-` or the owner's instance) merges into or is taken for the owner, never flagged before a bind | CC 1 (BLOCKER), OC 1, OC 4 | A | Hub keys entries by (agent, instance, strength, key_fp): a claimed reader never merges into a proven entry. `parse_reader` refuses instance `-` and empty. Sidecar counts as itself only `proven` entries with its OWN key fingerprint. |
| 2 | Recorded before the topic exists; topic length unbounded; eviction rows not rate-capped; full-table rewrite per junk call | CC 3, OC 2 | A | Record only for topics that exist on the bus; topic ≤ 256 bytes; eviction rows rate-capped (10/min, a `suppressed` count on the next row); an inbox with a reader inside the 24 h window is never evicted for a new topic (the new one is not recorded; a capped `inbox-consumer-table-full` row). |
| 3 | Liveness falls back to a content read on any `first_at` error | CC 9, OC 3 | A | Fallback only when the error is not from the new handler (an old hub answers -32001 "Missing 'target'", measured in `router.rs::forward_to_target`); a transport error (no code) is Unreadable. |
| 4 | Corrupt/unknown-schema table overwritten; `load_error` never cleared | CC 4, OC 5 | A | On a load error the file is moved to `inbox-consumers.json.corrupt-<ms>` with an admin row; the table restarts with `incomplete_since_ms`; `inbox.consumers` answers an error ("history incomplete since …") until the 24 h window has passed it; `load_error` cleared by that move. |
| 5 | Design 4.1 names `deliver.rs`; no claim path is signed or tested | CC 10, OC 7 | A | Measured: the sidecar never calls `channel.claim` (its "claim" is a local `O_EXCL` file, `deliver.rs:10`). Design corrected. Claim path tested at L4 through the hub (signed claim passes a bind, unsigned refused). |

## claude-code only

| # | Finding | Disp. | Change / reason |
|---|---|---|---|
| CC 2 | Status line has no age: a stale `one (proven)` survives a restart, early return or failed write | A | State carries `checked_ms`; `status` shows `unknown (no check since …)` after 2 × CHECK_EVERY_MS; cleared at start. |
| CC 5 | Tests: claim path, ack/claim refusal through the hub, replay across hub/method, stale ts after bind, bounds, mutant per gap | A | Added L4: signed/unsigned claim before and after bind, receipt post refused after bind, stale-ts reader refused after bind, self-spoof flagged; unit: replay across hub and method, topic length, table-full, overflow ts. New mutants: claim gate removed, bindings fail open, strength merge, self by name. Each L4 run rebuilds the binary first (the runner already rebuilds at the end). |
| CC 6 | Long-poll opened before a bind still delivers after it | A | Subscribe re-checks the binding (no recording) after a long-poll returns and refuses when the reader no longer passes. |
| CC 7 | A verbatim replay of the owner's old receipt passes a bound inbox | A | Ack proof needs the post's `ts` within ±120 s of the hub clock; outside → `claimed (stale ack)`, refused after a bind. |
| CC 8 | `limit` / `timeout_ms` / `conversation_id` unsigned; a captured reader opens reads at its cursor for 120 s | D | Ruling E accepts a 120 s proof window; capturing a reader object needs the hub's traffic (same host, root on .107 today). Risk: a root process that sees a sidecar's request can re-read that page within 120 s; it gains nothing root cannot read from disk. Revisit with PH-4 accounts. |
| CC 11 | One global mutex; bindings/cards/revocation read from disk per read; one corrupt bindings file refuses every inbox | D (perf) / R (fail closed) | Global lock is stronger than per-topic (correctness holds); revocation and key cards are cached (len, mtime) already. Bindings read per call: deferred, measured cost to be taken on .107 (sidecar reads every few seconds). A corrupt bindings file refusing all inbox reads is deliberate fail-closed (FCW-124): with unknown bindings no read can be judged. |
| CC 12 | `(now − ts).abs()` overflows on `ts = i64::MIN` | A | `abs_diff`. |
| CC 13 | Refused/eviction rows: failure ignored; global cap lets junk hide a real probe; suppressed count lost | A (partly) | Per-kind cap with a `suppressed` count carried on the next row. A failed row does not undo a refusal (refusing is the safe side); an eviction proceeds (the table bound wins). Per-topic caps: D (any caller can pick topics, so a per-topic cap is unbounded in total). |
| CC 14 | Codes and counters differ from design 3.1; revoked ack only claimed; bind ignores revocation | A (partly) | Design 3.1 corrected to the codes used (SIGNATURE_INVALID for a bad signature is the more precise code). `bind` refuses a revoked card key. A revoked ack key stays `claimed (key revoked)` (refused after a bind; a post path change is P-15's, not this unit's). Refusal counter: D (status P-24 reads counts). |
| CC 15 | `first_at` / `receipts` leak a bound inbox's metadata | D | Within ruling D (content-free). Metadata of who acked is visible to Observe callers as before P-12. Raised to the operator in the round-2 report as an open item, not decided by the agent. |

## opencode only

| # | Finding | Disp. | Change |
|---|---|---|---|
| OC 6 | An uncarded ack is listed `key:<fp>`, not its claimed `from_agent` as the design says | A | Uncarded ack: the payload's `from_agent` if a plain name, else `(unnamed)`; the key prefix moves into `why`. |
