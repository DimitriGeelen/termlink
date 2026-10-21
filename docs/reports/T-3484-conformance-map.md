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
- Run at: 2026-10-21, AEF `b4c168aa9`, TermLink `main` with T-3521 (operator notices) in. T-3520's answer work was in flight in
  the same tree, so CC-12 reads as that task leaves it.

## Result

```
vectors: AEF b4c168aa9 (https://raw.githubusercontent.com/DimitriGeelen/agentic-engineering-framework/b4c168aa9/tests/conformance/worker-contract/vectors.yaml)
  CC-01  fail             Completion racing session end
         [main] step confirm_end: expected {"any_of":[{"accepted":"T4"},{"accepted":"T3"}]}, got {"no_transition":true,"notices":0,"ok":true,"values":{"observation_logged":false,"reconciliation_requested":false}}
  CC-02  pass             Same race, opposite order
  CC-03  fail             Supervisor crash before commit
         [evidence-present] step confirm_end: expected {"any_of":[{"accepted":"T4"},{"accepted":"T3"}]}, got {"no_transition":true,"notices":0,"ok":true,"values":{"observation_logged":false,"reconciliation_requested":false}} | [no-evidence] step confirm_end: expected {"accepted":"T4"}, got {"no_transition":true,"notices":0,"ok":true,"values":{"observation_logged":false,"reconciliation_requested":false}}
  CC-04  pass             Supervisor crash after commit
  CC-05  pass             Duplicate finalisation
  CC-06  fail             Retry while the old worker survives
         [main] step confirm_end: expected {"accepted":"T4"}, got {"no_transition":true,"notices":0,"ok":true,"values":{"observation_logged":false,"reconciliation_requested":false}}
  CC-07  pass             Stale-attempt write rejected
  CC-08  not-implemented  Hub restart
         [undecidable-then-alive] op:wait (no worker-side wait or collect surface)
  CC-09  not-implemented  Partition
         [via-unknown] op:partition (the hub has no model of worker-host contact) | [via-lost] op:partition (the hub has no model of worker-host contact)
  CC-10  not-implemented  Parent-session loss
         [main] op:inspect surface operator_queue
  CC-11  n/a              Answer delivered and acked
         capability not offered: real_worker
  CC-12  not-implemented  Answer TTL expiry bounces
         [deadline-passes] then.bounced_to_dispatcher: the backend does not expose it | [attempt-terminal-unacked] then.bounced_to_dispatcher: the backend does not expose it
  CC-13  not-implemented  Local backend parity
         [lock-timeout-fails-closed] op:hold_lock (the per-write lock is SQLite IMMEDIATE; no hook to hold it)
  CC-14  pass             Stale supervisor generation
  CC-15  not-implemented  Unknown resolution by operator
         [main] refusal:agent_session (T7d speed bump: resolve_on_host has no agent-session gate)
  CC-16  pass             Outcome derivation
  CC-17  pass             Absence across a registry epoch
  CC-18  pass             Caller-chosen dispatch id
  CC-19  not-implemented  Paused is terminal, not success
         [next-attempt] op:consume (no consumer surface; refusals of review_verdict / pass_rate live outside the store) | [re-dispatch] op:consume (no consumer surface; refusals of review_verdict / pass_rate live outside the store)
  CC-20  pass             Lost attempt seen alive
  CC-21  pass             Dispatched never seen
  CC-22  not-implemented  Attested done is not finished work
         [no-material] op:consume (no consumer surface; refusals of review_verdict / pass_rate live outside the store) | [unbound-material] T7d material binding (ResolveRequest carries no material or outcome) | [bound-evidence] T7d material binding (ResolveRequest carries no material or outcome)
  CC-23  pass             The nonce alone cannot complete
  CC-24  pass             Late T5 whose evidence postdates the lost entry
  CC-25  not-implemented  Backend-initiated stop
         [exit-confirmed-valid-handback] op:stop (no backend-initiated stop in the store; transition takes no stop cause) | [exit-confirmed-handback-absent] op:stop (no backend-initiated stop in the store; transition takes no stop cause) | [no-exit-evidence] op:stop (no backend-initiated stop in the store; transition takes no stop cause) | [revoked] op:stop (no backend-initiated stop in the store; transition takes no stop cause)
summary: 25 cases: pass=12 fail=3 not-implemented=9 n/a=1
```

Counts: **pass 12** (CC-02, 04, 05, 07, 14, 16, 17, 18, 20, 21, 23, 24) · **fail 3** (CC-01, 03, 06) · **not-implemented 9** · **n/a 1** (CC-11).

Before T-3521 (same tree, AEF `b4c168aa9`): pass 7 · fail 4 (CC-01, 03, 06, 12) · not-implemented 13 · n/a 1. T-3521 turned
CC-05, 17, 20, 21 and 24 into pass and moved CC-08 from an unreadable notice step to the `wait` op. CC-12 left `fail` for
not-implemented in the same run through T-3520's work, not this task's. No previously passing case regressed.

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

- **Operator notices: done (T-3521).** See the section below. What still stops these cases is not notices: CC-08 stops at `wait`
  (no worker-side wait surface), CC-15 at the `agent_session` refusal, CC-22 at `consume` and the T7d material binding.
- **Partition and worker-host contact (`partition`, `heal`) — CC-09.** The hub has no model of it.
  `probe_undecided` is the closest op and is mapped to a sweep that cannot see (`listing_complete` false).
- **Backend-initiated stop (`stop`) — CC-25.** `transition` takes a `cause` but there is no stop op.
- **Locks (`hold_lock`) — CC-13 (lock-timeout variant).** The per-write lock is SQLite IMMEDIATE, with no
  hook to hold it from outside. The concurrent-writer variants (eight writers) are judged and pass.
- **Consumers (`consume`) — CC-19, 22.** No consumer surface. Refusals `not_finished_work` live outside the store.
- **Worker wait / collect (`ask_wait`) — CC-11 (n/a), CC-08 (wait).** No worker-side wait surface.
- **Dispatcher ownership (`adopt`) and inspect surfaces (`operator_queue`, `worker_topics`) — CC-10.**
- **T7d gates — CC-15, 22 (T-3527).** `resolve_on_host` has no agent-session gate, and `ResolveRequest` carries
  no material or outcome. `via: remote` is refused by construction: resolve is not an RPC.
- **Backend addressing (`complete.args.backend`) — CC-09.** `dispatch.transition` has no backend address.
- **Lease (`lease`) and real worker (`real_worker`).** Not offered. Lease variants are not run. CC-11 is n/a.

## Operator notices and reconciliation requests (T-3521)

Vocabulary, as the vectors use it (the file has no prose for these keys, so this is read from the cases): `notices: N` on a
step's `expect` is how many operator notices that step raised; `operator_notices: N` under `then` is how many the dispatch carries
at the end, beyond what `given` already held; `values.reconciliation_requested: true` on a sweep says the hub asked the reconciler to
look for evidence before T4. A `refused` or `fence_reject` step can carry `notices` too.

What each asserting case needs: CC-05 done-conflict refusal 1; CC-08 T6 1; CC-15 T7d 1 (the refusals before it add none); CC-17
`reconciliation_requested` on the second absence at the last-seen epoch; CC-20 binding reappeared on a lost attempt 1 (the refused
`start` and the T5 add none); CC-21 T6b 1, and T3b 0; CC-22 T7d 1; CC-24 late T5 refusal 1.

Design: one durable table, `wc_notices (dispatch_id, attempt, kind, audience, ts, detail, history_id)`, in `meta.db`. Every
history-backed kind is an `AFTER INSERT` trigger on `wc_history`: the history row is the transition or refusal, so the notice
commits in the same transaction and no writer can record one without the other, including the T7d resolve in `answer.rs`, which
needed no change. Kinds: `done_conflict`, `late_evidence` (T5 refused; the refusal row now carries `cause = late_evidence`),
`unknown_by_deadline` (T6 and T6b), `binding_reappeared`, `operator_resolved` (T7d). `reconciliation_requested` has no history row, so
the sweep writes it inside its own transaction, once when absence is confirmed twice with G still running; its audience is
`reconciler`, and operator counts exclude it. T4 raises no notice (no vector asks for one). Not raised: T8 with the same evidence,
fence refusals other than the two above, a refused resolve. No backfill: history before the table has no notices.

Read API: `Bus::wc_operator_notice_count(dispatch, attempt?)`, `Bus::wc_notices(dispatch, attempt)`, and `dispatch.get` now returns
`attempts[].notices`. The adapter's per-step `notices` is the difference of the count across the op, and its `operator_notices` is
the count minus what the seeded record already held. No new RPC: the read surface is `dispatch.get`.

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
