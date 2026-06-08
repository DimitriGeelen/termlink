# T-2023 Inception Research — Substrate primitive #5: client-side reconnect + outbound queue

**Status:** MOSTLY-SHIPPED via T-1439. Two small gaps remain (idempotency + recipe docs); file each as a small build task.
**Artifact created:** 2026-06-08
**See also:** T-2018 ADR §6 #5; T-1439 (the original offline queue implementation); `crates/termlink-session/src/offline_queue.rs`.

## 1. The §6 framing

ADR §6 primitive #5: *"Channel durability protects READERS. A spoke that briefly loses the hub has its outbound post DISCARDED — there is no client-side reconnect or outbound queue. A worker that finishes during a hub blip loses its completion report."*

The framing is a problem statement. Investigation shows the *primary* solution already exists.

## 2. Ground-truth from the running substrate

`crates/termlink-session/src/offline_queue.rs` (~383 LOC) provides:

| Capability                                          | Status                                                              |
|-----------------------------------------------------|---------------------------------------------------------------------|
| SQLite-backed durable queue (`pending_posts` table) | ✅ Implemented                                                       |
| `OfflineQueue::open(path)` / `enqueue` / `pop`      | ✅ Implemented                                                       |
| Default path (`~/.termlink/<name>/outbound.sqlite`) | ✅ Via `default_queue_path()`                                        |
| Capacity cap (default `DEFAULT_CAP = 1000`)         | ✅                                                                   |
| Overflow policy = "loud-fail" (R3)                  | ✅ — refuses new posts when full, returns `QueueError::Full {cap}`   |
| `attempts` counter per row (poison-pill avoidance)  | ✅ — flush loop can age out stuck entries                            |
| `enqueued_ms` timestamp + index                     | ✅                                                                   |
| Drain/flush mechanism                               | ✅ — referenced in module docstring + T-1439                         |
| CLI integration in channel.rs + remote.rs           | ✅ — `default_queue_path()` imported and used in dispatch paths      |

The §6 problem statement was correct AT THE TIME OF FILING (the queue did not exist), but the work was completed under a different task ID (T-1439). T-2023 was filed before the relationship was clear.

## 3. What's actually missing

Auditing against the IW questions:

- **IW-1 (queue location — in-memory vs disk):** ✅ Resolved. Disk-backed (SQLite).
- **IW-2 (cap + overflow — backpressure / oldest-drop / fail-loudly):** ✅ Resolved. Fail-loudly (R3, matches ADR's `loud-not-silent` stance).
- **IW-3 (reconnect strategy — backoff, max attempts, fail-permanent signal):** ⚠ PARTIAL. The `attempts` counter exists for poison-pill detection but explicit exponential-backoff + jitter parameters are not configurable. Default behavior is whatever T-1439's flush loop implemented. Audit needed.
- **IW-4 (idempotency — dedupe on hub if a queued post was actually delivered before disconnect):** ❌ MISSING. No `client_msg_id` field, no hub-side dedupe. A spoke that posted, lost the hub before receiving the ack, queued the post, and the hub later applied BOTH would produce a duplicate envelope.

## 4. Remaining-gap recommendations

### Gap A — Idempotency (the real gap)

The double-apply scenario is concrete and reproducible:
1. Spoke sends `channel.post topic X payload P`.
2. Hub writes the envelope at offset N, but the TCP response is lost (RST or timeout) BEFORE the spoke sees the ack.
3. Spoke retries → queue → eventually delivered → hub writes again at offset N+1.
4. Subscribers see P twice with different offsets.

**Fix shape:** Spoke generates a `client_msg_id` (UUID or content-hash) for every post. Hub maintains a short-TTL recently-seen set keyed by `(sender_fingerprint, client_msg_id)` and silently no-ops a duplicate. The TTL bound (e.g. 5 minutes) keeps the set small; longer than realistic reconnect window.

**Cost:** ~80 LOC. New field in the post envelope, small in-memory hub-side LRU, optional spike to confirm collision probability is acceptable.

### Gap B — Backoff parameter audit

The existing flush loop's backoff behavior isn't visible from the offline_queue module alone (it's elsewhere). An audit task should:
1. Locate the flush loop implementation.
2. Document the backoff parameters (initial delay, max delay, jitter, max attempts).
3. Confirm "explicit fail-permanent signal" exists (after N attempts, mark row as dead-letter, alert operator).
4. If any of those are missing or hard-coded with poor defaults, file a small follow-up.

**Cost:** ≤1 session for the audit + any follow-up <50 LOC.

### Gap C — Documentation

`docs/operations/substrate-claim-primitive.md` (or a sibling doc) does not yet describe the offline-queue recipe — how a CLI invocation handles hub-blip, where the queue lives, how an operator inspects it, how poison-pill rows are surfaced. Without this, the feature exists but isn't discoverable.

**Cost:** ~50 lines of documentation, no code.

## 5. IW dispositions

- **IW-1:** ✅ Resolved. Disk-backed (SQLite) per implementation. Confidence=4.
- **IW-2:** ✅ Resolved. Fail-loudly with `QueueError::Full` per existing implementation; matches R3 stance. Confidence=4.
- **IW-3:** ⚠ Partial. `attempts` counter exists, but full backoff/jitter/fail-permanent parameters need audit. Confidence=2 (audit will resolve to 4).
- **IW-4:** ❌ Missing. No `client_msg_id` / dedupe. The real remaining gap. Confidence=4 (problem clear, solution clear).

## 6. Recommendation

**MOSTLY-SHIPPED, partial-GO on remaining gaps.**

- File Gap A as a small build task (idempotency via `client_msg_id` + hub-side dedupe). ~80 LOC.
- File Gap B as an audit task (locate flush loop + document backoff params). ≤1 session.
- File Gap C as a doc task (offline-queue recipe in operations docs). ~50 lines docs.

**Why not full GO of T-2023 as captured:** the majority of the work is already shipped. Closing this as full-GO would create a fictitious build task that just re-discovers existing code. Splitting into three small follow-ups is honest about what's left.

**Why not NO-GO:** the remaining gap (idempotency) is concrete and small. Closing T-2023 entirely would leave that gap unfiled.

## 7. ADR alignment check

| ADR section | Alignment |
|-------------|-----------|
| §3 "channel durability is recovery story" | ✓ Outbound queue extends durability to the *client side* of a post. |
| §5 "one writer, serialized" | ✓ Queue holds at the spoke; hub remains the single serialized writer. |
| §6 #5 framing | ⚠ Captured framing was correct at filing; the work shipped under T-1439. |
| §9 "AEF dispatch reliability rests on this" | ✓ With idempotency (Gap A) closed, governance-plane ledger writes are exactly-once across hub blips. |

## 8. Open follow-up tasks to file

- **Gap A build task:** Idempotency — `client_msg_id` on post envelope + hub-side LRU dedupe. ~80 LOC, ≤1 session.
- **Gap B audit task:** Locate flush loop, document backoff params (initial delay, max delay, jitter, max attempts, dead-letter behavior). Output: short audit report; conditional follow-up <50 LOC if params are poor.
- **Gap C doc task:** Offline-queue recipe in `docs/operations/`. ~50 lines.
