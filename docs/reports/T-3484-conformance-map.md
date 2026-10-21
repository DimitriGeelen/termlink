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
- Run at: 2026-10-21, AEF `afe974124`, TermLink `main` at the T-3522 commit (T-3518/T-3519 fixes in).

## Result

```
backend: hub (termlink-bus worker_contract, S1-S3), vectors: AEF afe974124
  CC-01  fail             Completion racing session end
         [main] step confirm_end: expected {"any_of":[{"accepted":"T4"},{"accepted":"T3"}]}, got {"no_transition":true,"notices":0,"ok":true,"values":{"observation_logged":false}}
  CC-02  pass             Same race, opposite order
  CC-03  fail             Supervisor crash before commit
         [evidence-present] step confirm_end: expected {"any_of":[{"accepted":"T4"},{"accepted":"T3"}]}, got {"no_transition":true,"notices":0,"ok":true,"values":{"observation_logged":false}} | [no-evidence] step confirm_end: expected {"accepted":"T4"}, got {"no_transition":true,"notices":0,"ok":true,"values":{"observation_logged":false}}
  CC-04  pass             Supervisor crash after commit
  CC-05  not-implemented  Duplicate finalisation
         [different-evidence] step complete: expectation names a surface the backend does not expose: {"fence_reject":"done_conflict","notices":1}
  CC-06  fail             Retry while the old worker survives
         [main] step confirm_end: expected {"accepted":"T4"}, got {"no_transition":true,"notices":0,"ok":true,"values":{"observation_logged":false}}
  CC-07  pass             Stale-attempt write rejected
  CC-08  not-implemented  Hub restart
         [absent-after-restart] step advance: expectation names a surface the backend does not expose: {"any_of":[{"accepted":"T6","notices":1},{"no_transition":true}]} | [undecidable-then-alive] step advance: expectation names a surface the backend does not expose: {"accepted":"T6","notices":1}
  CC-09  not-implemented  Partition
         [via-unknown] op:partition (the hub has no model of worker-host contact) | [via-lost] op:partition (the hub has no model of worker-host contact)
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
  CC-16  pass             Outcome derivation
  CC-17  not-implemented  Absence across a registry epoch
         [absences-at-last-seen-epoch] step sweep: expectation names a surface the backend does not expose: {"ok":true,"values":{"reconciliation_requested":true}} | [fenced-done-inside-G-wins] step sweep: expectation names a surface the backend does not expose: {"ok":true,"values":{"reconciliation_requested":true}}
  CC-18  pass             Caller-chosen dispatch id
  CC-19  not-implemented  Paused is terminal, not success
         [next-attempt] op:consume (no consumer surface; refusals of review_verdict / pass_rate live outside the store) | [re-dispatch] op:consume (no consumer surface; refusals of review_verdict / pass_rate live outside the store)
  CC-20  not-implemented  Lost attempt seen alive
         [main] then.operator_notices: the backend does not expose it
  CC-21  not-implemented  Dispatched never seen
         [never-registers] step advance: expectation names a surface the backend does not expose: {"accepted":"T6b","notices":1} | [refused-before-start] then.operator_notices: the backend does not expose it
  CC-22  not-implemented  Attested done is not finished work
         [no-material] step resolve: expectation names a surface the backend does not expose: {"accepted":"T7d","notices":1} | [unbound-material] T7d material binding (ResolveRequest carries no material or outcome) | [bound-evidence] T7d material binding (ResolveRequest carries no material or outcome)
  CC-23  pass             The nonce alone cannot complete
  CC-24  not-implemented  Late T5 whose evidence postdates the lost entry
         [persisted-after-lost] step complete: expectation names a surface the backend does not expose: {"fence_reject":"forbidden_transition","notices":1}
  CC-25  not-implemented  Backend-initiated stop
         [exit-confirmed-valid-handback] op:stop (no backend-initiated stop in the store; transition takes no stop cause) | [exit-confirmed-handback-absent] op:stop (no backend-initiated stop in the store; transition takes no stop cause) | [no-exit-evidence] op:stop (no backend-initiated stop in the store; transition takes no stop cause) | [revoked] op:stop (no backend-initiated stop in the store; transition takes no stop cause)
summary: 25 cases: pass=7 fail=4 not-implemented=13 n/a=1
```

Counts: **pass 7** (CC-02, 04, 07, 14, 16, 18, 23) · **fail 4** (CC-01, 03, 06, 12) · **not-implemented 13** · **n/a 1** (CC-11).

Before T-3522 (same tree, adapter as of T-3518): pass 6 (CC-04, 07, 14, 16, 18, 23) · fail 1 (CC-12) ·
not-implemented 17 · n/a 1. The earlier version of this map showed CC-16 and CC-21 as fail; that
predated the T-3518/T-3519 fixes. The table above is the adapter's current output.

## Failing cases — filed as bug tasks

| Case | Diverging step | Bug task | Note |
|---|---|---|---|
| CC-12 | `append_answer` naming `question_id: q1` that the given never asks → refused `cas_mismatch` (vector expects `ok`) | T-3520 | Either the store or the vector is wrong. Needs an AEF ruling. Not patched. |
| CC-01, CC-03 (evidence-present) | `confirm_end` (hub, clock 20, signed E1 persisted at 10) → `no_transition`; vector expects T4 or T3 | T-3525 | The hub cannot read signed worker evidence before an end. The store has no hub-side evidence intake. Backend gap. |
| CC-03 (no-evidence), CC-06 | `confirm_end` on a running attempt never seen alive → `no_transition`; vector expects T4 | T-3526 | Vector conflicts with §3.4 / CC-17 (absence counts only for an attempt seen alive). Needs an AEF ruling before any backend change. |

CC-16 and CC-21 are no longer failing (see the Result note above). CC-21 is not-implemented now,
on its `T6b` / operator-notice steps.

The adapter does not patch backend behaviour, except where a case shows a real divergence with a
failing-first unit test (T-3522: `confirm_end` refusing a done attempt, CC-02). The other fixes belong in
the backend, and the verdict is the honest state until they land.

## Not-implemented, by missing piece

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

## `confirm_end` (T-3522): what the adapter maps and what the store does

`confirm_end` is now mapped, not reported missing. The adapter calls `Bus::wc_confirm_end`, which refuses
a `done` attempt with `forbidden_transition` (CC-02 passes), and otherwise runs the ordinary S2 sweep
twice at the current epoch and once more G later on the injected clock. Nothing writes state on that
path except the sweep's own T4.

Decision (quoted vector text: "confirm_end: the sweep writer confirms the attempt's end by the backend's own
evidence (hub two sweeps at the last-seen epoch then G; hubless independent probe, evidence read first) and
acts on it"): the S2 sweep already has the absence rule. The gap was the adapter's mapping, plus one real
backend divergence (a hub end over a done attempt answered nothing instead of the refusal CC-02 names).
The rest is a backend gap that the adapter cannot close: the store does not read signed evidence before an
end (CC-01, CC-03 evidence-present), and it does not count absence for an attempt never seen alive (CC-06,
CC-03 no-evidence), which §3.4 and CC-17 require.

## Observed by reading, not verified by a run

These are not in the verdicts, because the step that reaches them is not-implemented or fails first. Treat
them as suspicions for a follow-up, not as findings:

- **CC-22 (attested done).** `resolve_tx` sets `evidence_digest` to `operator:<sha256 of reason>`, and the
  no-material variant expects `evidence_digest: null`. It also leaves `outcome` NULL, and the vector expects
  `succeeded`. Both would diverge if the `then` were reached.

## Earlier map, superseded

The earlier version of this table marked cases covered or partial from unit tests. It did not check the vector
text, and it could not see CC-12, 16 and 21. The S1–S3 unit tests still run (`cargo test -p termlink-bus`). They
are backend tests, not conformance verdicts. Use the table above for conformance.

## Next

1. AEF ruling on CC-12 (store or vector), on CC-06 / CC-03 no-evidence (§3.4 against the T4 expectation for a
   never-seen attempt), and on whether confirm_end over signed evidence writes T4 or leaves the end to T3.
2. Backend evidence intake for `confirm_end` (T-3525), once the ruling in 1 says which way it goes.
3. The not-implemented list is the backlog for S4 and the operator-surface work. Each item names its missing piece.
4. When the vectors are on the mirror, send AEF the hub-side `applicability` and `backend_notes` patch (the
   README contract). Nothing is sent until the verdicts are settled.
