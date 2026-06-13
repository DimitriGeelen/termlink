# T-2212 — Substrate Cooperative-Handoff Demo (arc-parallel-substrate proof #2)

**Captured:** 2026-06-13T14:22:39Z · **Binary:** `termlink 0.11.1230` · **Hub:** local (`/var/lib/termlink`)

## What this proves

The arc's headline mechanic has two complementary shapes. T-2211's
`substrate-drain-demo.sh` proves **work-stealing** (N workers race to claim
disjoint units). This demo proves the other shape — **directed assignment via
cooperative handoff**, the canonical orchestrator pattern documented in
[`docs/operations/substrate-orchestrator-recipe.md`](../operations/substrate-orchestrator-recipe.md):
an orchestrator claims a slot on a worker's behalf and atomically hands the
lease over (substrate primitive #3, `claim-transfer`) with **zero
release-then-claim race window**, after which the worker renews and releases.

The proof is in the assertions: every step hard-checks BOTH the positive path
AND the **ownership-enforcement negative path** (`CLAIM_NOT_OWNED`, surfaced by
this binary's CLI as `held by another claimer` + non-zero exit). A regression
that silently let a non-owner mutate someone else's claim would FAIL the demo.

Composes ONLY shipped verbs — no new primitive, no hub change:
`channel create / post / claim / claim-transfer / renew / release`.

## The seven assertions

| # | Step | Kind | Asserts |
|---|------|------|---------|
| 1 | orchestrator-claim       | POS | orchestrator (`demo-orch`) claims the offset → `claim_id` returned |
| 2 | reject-stale-by-transfer | NEG | `claim-transfer --by demo-intruder` (non-owner) is REFUSED |
| 3 | transfer-to-worker       | POS | `claim-transfer --by demo-orch --to-owner demo-worker` moves `claimed_by`; lease `claimed_at` survives the transfer |
| 4 | reject-ex-owner-renew    | NEG | the ex-owner orchestrator can no longer `renew` |
| 5 | worker-renew             | POS | the new owner `demo-worker` renews (proves ownership truly moved) |
| 6 | reject-ex-owner-release  | NEG | the ex-owner orchestrator can no longer `release` |
| 7 | worker-release-ack       | POS | the new owner releases with `--ack` (work complete, cursor advanced) |

## Captured runs

```
=== substrate cooperative-handoff demo (T-2212) ===
# topic=substrate-handoff-demo orchestrator=demo-orch worker=demo-worker ttl_ms=60000
# seeded work unit at offset 6
  [PASS] orchestrator-claim — claim_id=clm-1781360559918690282-substrate_handof-6 claimer=demo-orch
  [PASS] reject-stale-by-transfer — ownership gate held (--by=demo-intruder refused)
  [PASS] transfer-to-worker — claimed_by -> demo-worker (lease claimed_at=1781360559918 survived)
  [PASS] reject-ex-owner-renew — ex-owner demo-orch renew refused after handoff
  [PASS] worker-renew — new owner demo-worker renewed (proves ownership moved)
  [PASS] reject-ex-owner-release — ex-owner demo-orch release refused after handoff
  [PASS] worker-release-ack — work completed (ack=true, cursor advanced)

COOPERATIVE-HANDOFF DEMO PASS — 7/7 assertions green
  (3 positive lifecycle steps + 3 ownership-gate refusals, atomic handoff verified)
exit=0
```

Repeated for determinism (reused bounded topic, no per-run topic growth):

```
run 1: {"verdict":"PASS","passed":7,"failed":0}
run 2: {"verdict":"PASS","passed":7,"failed":0}
run 3: {"verdict":"PASS","passed":7,"failed":0}
channel count for substrate-handoff-demo: 1
```

## Notes / design

- **`renew --additional-ttl-ms` computes the new lease from `claimed_at`, not
  from the current `claimed_until`** — so a +30000ms renew on a 60000ms claim
  can yield an *earlier* expiry. The demo therefore asserts `renew` returns
  `ok==true` with the correct `claimer`, NOT a `claimed_until` increase. (The
  point of step 5 is ownership, not lease arithmetic.)
- The ownership gate is the same one the catalog labels `CLAIM_NOT_OWNED`
  (-32017); binary termlink 0.11.1230 surfaces it at the CLI as a
  stderr `held by another claimer` string with non-zero exit. The demo matches
  on all three forms so it stays correct across CLI surface revisions.
- Reused, retention-capped topic `substrate-handoff-demo` (hubs are
  append-only — no channel-delete verb), so repeated runs don't accumulate
  throwaway topics. Override with `--topic NAME` for isolated runs.

## Reproduce

```
bash scripts/substrate-cooperative-handoff-demo.sh          # human-readable
bash scripts/substrate-cooperative-handoff-demo.sh --json   # machine-readable
```

## Related

- `scripts/substrate-drain-demo.sh` + `docs/reports/T-2211-substrate-drain-demo.md` — proof #1 (work-stealing)
- `docs/operations/substrate-orchestrator-recipe.md` — the canonical pattern this demo exercises end-to-end
- Substrate primitive #3 (claim-transfer): T-2046 / T-2021 (GO)
