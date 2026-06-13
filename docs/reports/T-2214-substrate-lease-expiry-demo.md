# T-2214 — Substrate Lease-Expiry Resilience Demo

**Arc:** arc-parallel-substrate (arc-001) · **Primitive:** #1 claim/renew/release
**Constitutional directive proven:** #1 Antifragility — *the system strengthens
under stress; failures are learning events.*

## What this proves

The drain demo (T-2211) proves **work-stealing** under live contention; the
cooperative-handoff demo (T-2212) proves **directed assignment** with atomic
ownership transfer. Neither proves the substrate's **self-healing** path:

> A worker claims a unit, then **dies** (crash / network partition / hang) while
> still holding the lease. What happens to the unit?

If the answer were "it stays claimed forever," a single dead worker would wedge
its unit indefinitely — the opposite of antifragile. The substrate's actual
answer: the lease **auto-expires**, the slot **reopens** to another worker, and
the lapsed original owner is **locked out** from resurrecting it. This demo
proves that answer live.

`scripts/substrate-lease-expiry-demo.sh` composes only shipped verbs
(`channel create / post / claim / renew / release`) — no new primitive, no hub
change. A worker claims with a deliberately short TTL (default 2000 ms), the
demo waits the lease out (simulating death-by-no-renewal), then asserts the
recovery.

## Assertion table (6/6)

| # | Kind | Step | Asserts |
|---|------|------|---------|
| 1 | POS | `worker-a-claim` | A acquires the offset under a short lease |
| 2 | NEG | `reject-contested-claim` | B is refused while A's lease is **live** (exclusivity) |
| 3 | POS | `worker-b-reclaim-after-expiry` | after the lease lapses, B claims the **same** offset (auto-reopen) |
| 4 | NEG | `reject-lapsed-renew` | the lapsed owner A can no longer `renew` |
| 5 | NEG | `reject-lapsed-release` | the lapsed owner A can no longer `release` |
| 6 | POS | `worker-b-release-ack` | the new owner B completes the reopened unit with `--ack` |

3 positive + 3 negative. A regression that failed to reopen an expired slot,
or that let a lapsed owner mutate a reclaimed slot, fails the demo.

## Observed lapsed-claim error vocabulary (empirical, 2026-06-13)

Probed live before authoring the assertions (per PL-213 — assert the property,
match the *real* error strings, not assumed ones):

- **Contested while lease live:**
  `channel.claim failed: offset N of topic "…" is already claimed by another worker`
- **Lapsed owner renew/release:**
  `channel.renew failed: claim "…" not found (never existed, released, or expired)`

The lapsed path returns **not-found**, NOT the `held by another claimer`
(CLAIM_NOT_OWNED) vocabulary the handoff demo greps — the claim record is gone,
not reassigned. The demo's `LAPSED_RE` matches the not-found family accordingly.

## Live runs

```
$ bash scripts/substrate-lease-expiry-demo.sh
  [PASS] worker-a-claim — claimer=demo-worker-a ttl_ms=2000
  [PASS] reject-contested-claim — exclusivity held — B refused while A's lease live
  # worker-a stops renewing (simulated death); waiting 4s for lease to lapse...
  [PASS] worker-b-reclaim-after-expiry — slot reopened — B claimed offset 0
  [PASS] reject-lapsed-renew — lapsed owner demo-worker-a renew refused (lease gone)
  [PASS] reject-lapsed-release — lapsed owner demo-worker-a release refused (lease gone)
  [PASS] worker-b-release-ack — reopened work completed by B (ack=true)
LEASE-EXPIRY RESILIENCE DEMO PASS — 6/6 assertions green
```

JSON verdict (`--json`): `{"ok":true,"verdict":"PASS","passed":6,"failed":0,"total":6}`
— green across consecutive runs.

## Regression protection

Wired into `scripts/substrate-smoke.sh` as the 9th stage
(`lease-expiry-demo`), asserting `ok:true`. The smoke suite now guards all
three arc mechanics: work-stealing (T-2211), directed-assignment ownership
gates (T-2212), and worker-death auto-reclaim (T-2214).

## Design notes

- **Short TTL by design.** `--ttl-ms` defaults to 2000 ms and the demo sleeps
  `ttl_ms/1000 + 2` s. This keeps the proof fast (~4 s) while still exercising
  real wall-clock lease expiry — not a mock.
- **Bounded topic.** Uses `--retention messages:50` (the single-arg form; the
  `--retention messages --retention-value N` spelling is silently ignored by the
  shipped CLI — T-2213).
- **Cleanup trap** releases B's claim if the demo aborts mid-flight; A's claim
  is expected to be gone by then.
