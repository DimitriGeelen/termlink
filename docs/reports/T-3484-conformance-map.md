# Worker contract v0.3.3 §9 — TermLink hub backend conformance map

Verdicts come from the adapter, not from hand-written status. The adapter runs AEF's
`tests/conformance/worker-contract/vectors.yaml` (wc/0.3, 25 cases) against the hub backend
(`termlink-bus` worker_contract S1–S3, bus-level API with injected time):

```bash
bash scripts/run-wc-vectors.sh            # fetches vectors at AEF afe974124 into a temp dir; exit 0 only if no case fails
TERMLINK_WC_CASE=CC-16 bash scripts/run-wc-vectors.sh   # one case, e.g. to reproduce a finding (filter is in the adapter)
```

- Adapter: `crates/termlink-hub/tests/wc_conformance.rs` (runner, AEF's `run_local.py` semantics) and
  `crates/termlink-hub/tests/wc_conformance/hub.rs` (maps each op to the store). Without
  `TERMLINK_WC_VECTORS` the test prints SKIP; it never passes silently.
- Verdicts: **pass** (every step ran and every `then` assertion matched), **fail** (the backend answered
  differently; the diverging step is named), **not-implemented** (the backend does not expose the op,
  argument or surface; named), **n/a** (capability not offered, where the vector allows it).
- The vector file is AEF's. It is not committed here (`git ls-files` has no `vectors.yaml`).
- Run at: 2026-10-21, AEF `afe974124`, TermLink `main` at the T-3517 commit.

## Result

```
backend: hub (termlink-bus worker_contract, S1-S3), vectors: AEF afe974124
  CC-01  not-implemented  Completion racing session end
         [main] op:confirm_end (the sweep counts absence only for attempts seen alive, and does not read signed evidence first)
  CC-02  not-implemented  Same race, opposite order
         [main] op:confirm_end (the sweep counts absence only for attempts seen alive, and does not read signed evidence first)
  CC-03  not-implemented  Supervisor crash before commit
         [evidence-present] op:confirm_end (…) | [no-evidence] op:confirm_end (…)
  CC-04  pass             Supervisor crash after commit
  CC-05  not-implemented  Duplicate finalisation
         [different-evidence] step complete: expectation names a surface the backend does not expose: {"fence_reject":"done_conflict","notices":1}
  CC-06  not-implemented  Retry while the old worker survives
         [main] op:confirm_end (…)
  CC-07  pass             Stale-attempt write rejected
  CC-08  not-implemented  Hub restart
         [absent-after-restart] step advance: expectation names a surface the backend does not expose: {"any_of":[{"accepted":"T6","notices":1},{"no_transition":true}]} | [undecidable-then-alive] step advance: expectation names a surface the backend does not expose: {"accepted":"T6","notices":1}
  CC-09  not-implemented  Partition
         [via-unknown] op:partition (the hub has no model of worker-host contact) | [via-lost] op:partition (…)
  CC-10  not-implemented  Parent-session loss
         [main] op:inspect surface operator_queue
  CC-11  n/a              Answer delivered and acked
         capability not offered: real_worker
  CC-12  fail             Answer TTL expiry bounces
         [deadline-passes] step append_answer: expected {"ok":true}, got {"fence_reject":"cas_mismatch"} | [attempt-terminal-unacked] step append_answer: expected {"ok":true}, got {"fence_reject":"cas_mismatch"}
  CC-13  not-implemented  Local backend parity
         [lock-timeout-fails-closed] op:hold_lock (the per-write lock is SQLite IMMEDIATE; no hook to hold it)
  CC-14  pass             Stale supervisor generation
  CC-15  not-implemented  Unknown resolution by operator
         [main] refusal:agent_session (T7d speed bump: resolve_on_host has no agent-session gate)
  CC-16  fail             Outcome derivation
         [supervisor-refusal] final A1.outcome: expected "refused", got "handback-malformed"
  CC-17  not-implemented  Absence across a registry epoch
         [absences-at-last-seen-epoch] step sweep: expectation names a surface the backend does not expose: {"ok":true,"values":{"reconciliation_requested":true}} | [fenced-done-inside-G-wins] (same)
  CC-18  pass             Caller-chosen dispatch id
  CC-19  not-implemented  Paused is terminal, not success
         [next-attempt] op:consume (…) | [re-dispatch] op:consume (…)
  CC-20  not-implemented  Lost attempt seen alive
         [main] then.operator_notices: the backend does not expose it
  CC-21  fail             Dispatched never seen
         [refused-before-start] final A1.outcome: expected "refused", got "handback-malformed"
  CC-22  not-implemented  Attested done is not finished work
         [no-material] step resolve: expectation names a surface the backend does not expose: {"accepted":"T7d","notices":1} | [unbound-material] T7d material binding (…) | [bound-evidence] (same)
  CC-23  pass             The nonce alone cannot complete
  CC-24  not-implemented  Late T5 whose evidence postdates the lost entry
         [persisted-after-lost] step complete: expectation names a surface the backend does not expose: {"fence_reject":"forbidden_transition","notices":1}
  CC-25  not-implemented  Backend-initiated stop
         [exit-confirmed-valid-handback] op:stop (…) | [exit-confirmed-handback-absent] op:stop (…) | [no-exit-evidence] op:stop (…) | [revoked] op:stop (…)
summary: 25 cases: pass=5 fail=3 not-implemented=16 n/a=1
```

(`(…)` marks a reason repeated in full in the run output; the adapter prints the full text.)

Counts: **pass 5** (CC-04, 07, 14, 18, 23) · **fail 3** (CC-12, 16, 21) · **not-implemented 16** · **n/a 1** (CC-11).

## Failing cases — filed as bug tasks

| Case | Diverging step | Bug task | Note |
|---|---|---|---|
| CC-16 | `complete` with `supervisor_refused`, handback absent → `outcome` expected `refused`, got `handback-malformed` | T-3518 | `derive_outcome()` checks the handback before the refused flag. |
| CC-21 | same derivation, from `dispatched` (T3b) | T-3519 | Same root cause as CC-16. Filed separately, as the adapter reports each case. |
| CC-12 | `append_answer` naming `question_id: q1` that the given never asks → refused `cas_mismatch` (vector expects `ok`) | T-3520 | Either the store or the vector is wrong. Needs an AEF ruling. Not patched. |

The adapter does not patch backend behaviour. The fixes belong in the backend, and the verdict is
the honest state until they land.

## Not-implemented, by missing piece

- **Confirm-end (`confirm_end`) — CC-01, 02, 03, 06.** The store has no end-confirmation entry point. Its
  sweep counts absence only for attempts seen alive at the current epoch, and does not read signed evidence
  before confirming an end.
- **Operator notices (`notices`, `operator_notices`) — CC-05, 08, 15, 20, 22, 24.** No notice surface is
  exposed. The sweep reports notices only for a `lost` attempt whose binding reappears. T4/T6 and
  done-conflict refusals are not counted.
- **Reconciliation request (`reconciliation_requested`) — CC-17.** Not reported by the sweep.
- **Partition and worker-host contact (`partition`, `heal`) — CC-09.** The hub has no model of it.
  `probe_undecided` is the closest op and is mapped to a sweep that cannot see (`listing_complete` false).
- **Backend-initiated stop (`stop`) — CC-25.** `transition` takes a `cause` but there is no stop op.
- **Locks (`hold_lock`) — CC-13 (lock-timeout variant).** The per-write lock is SQLite IMMEDIATE, with no
  hook to hold it from outside. The concurrent-writer variants (eight writers) are judged and pass.
- **Consumers (`consume`) — CC-19, 22.** No consumer surface. Refusals `not_finished_work` live outside the store.
- **Worker wait / collect (`ask_wait`) — CC-11 (n/a), CC-08 (wait).** No worker-side wait surface.
- **Dispatcher ownership (`adopt`) and inspect surfaces (`operator_queue`, `worker_topics`) — CC-10.**
- **T7d gates — CC-15, 22.** `resolve_on_host` has no agent-session gate, and `ResolveRequest` carries
  no material or outcome. `via: remote` is refused by construction: resolve is not an RPC.
- **Backend addressing (`complete.args.backend`) — CC-09.** `dispatch.transition` has no backend address.
- **Lease (`lease`) and real worker (`real_worker`).** Not offered. Lease variants are not run. CC-11 is n/a.

## Observed by reading, not verified by a run

These are not in the verdicts, because the step that reaches them is not-implemented first. Treat them as
suspicions for a follow-up, not as findings:

- **CC-22 (attested done).** `resolve_tx` sets `evidence_digest` to `operator:<sha256 of reason>`, and the
  no-material variant expects `evidence_digest: null`. It also leaves `outcome` NULL, and the vector expects
  `succeeded`. Both would diverge if the `then` were reached.
- **CC-16 unit test.** The S1 test `cc16_outcome_derivation_and_cc19_paused` (marked covered in the earlier
  version of this map) did not cover `supervisor_refused` with an absent handback. The vector shows the gap.

## Earlier map, superseded

The earlier version of this table marked cases covered or partial from unit tests. It did not check the vector
text, and it could not see CC-12, 16 and 21. The S1–S3 unit tests still run (`cargo test -p termlink-bus`). They
are backend tests, not conformance verdicts. Use the table above for conformance.

## Next

1. AEF ruling on CC-12 (store or vector), and the outcome precedence for CC-16 / CC-21 (confirm refused wins
   over an absent handback).
2. Fix the backend in T-3518 and T-3519 (one function), then re-run the adapter.
3. The not-implemented list is the backlog for S4 and the operator-surface work. Each item names its missing piece.
4. When the vectors are on the mirror, send AEF the hub-side `applicability` and `backend_notes` patch (the
   README contract). Nothing is sent until the verdicts are settled.
