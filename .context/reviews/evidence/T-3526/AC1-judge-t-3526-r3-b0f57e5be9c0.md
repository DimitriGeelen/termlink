# Reviewer evidence — T-3526 AC#1

- dispatch: `judge-t-3526-r3-b0f57e5be9c0`
- worker kind: `antigravity` (vendor `google`, model `worker default`)
- seat: `antigravity`
- reviewed revision: `57a519680408665f49c62b94983441bf50260b9d`
- recorded by: the dispatch runtime on the worker's behalf (T-3582)
- result.md sha256: `912056cf3efda8c06cd6eb0061f107331af51f399f3b0e3be7720fa2608f7130`

## The worker's verdict block

```
1. [AC] [REVIEW] Close-readiness: the work meets its Agent criteria and the Verification section passes (T-3526, reviewer-judged, T-4156)
VERDICT: green
WHY: Verified both Agent ACs and the Verification command in [.tasks/active/T-3526-wc03-hub-a-running-attempt-never-seen-al.md](file:///tmp/tl-dispatch/judge-t-3526-r3-b0f57e5be9c0/tree/.tasks/active/T-3526-wc03-hub-a-running-attempt-never-seen-al.md#L66-L119). (1) Agent AC#1: [run-wc-vectors.sh](file:///tmp/tl-dispatch/judge-t-3526-r3-b0f57e5be9c0/tree/scripts/run-wc-vectors.sh#L10-L21) pins AEF REF="25f56ed68", whose vectors.yaml adds seen_alive_epoch: E1 preconditions on CC-03 (line 163) and CC-06 (line 305) and documents in vocabulary.ops.confirm_end (line 62) that an attempt never seen alive leaves only through deadlines (T6b/T6), never T4 (§3.4). In [hub.rs](file:///tmp/tl-dispatch/judge-t-3526-r3-b0f57e5be9c0/tree/crates/termlink-hub/tests/wc_conformance/hub.rs#L297-L303), [Hub::seed](file:///tmp/tl-dispatch/judge-t-3526-r3-b0f57e5be9c0/tree/crates/termlink-hub/tests/wc_conformance/hub.rs#L253-L339) seeds seen_alive_epoch via self.live.insert((id.clone(), n)) and self.sweep_seed(t0, true) while the store's [absence](file:///tmp/tl-dispatch/judge-t-3526-r3-b0f57e5be9c0/tree/crates/termlink-bus/src/worker_contract/sweep.rs#L217-L249) check in [sweep.rs](file:///tmp/tl-dispatch/judge-t-3526-r3-b0f57e5be9c0/tree/crates/termlink-bus/src/worker_contract/sweep.rs#L218-L231) remains strict (c.binding_seen_at.is_some() and c.seen_epoch == Some(epoch)). (2) Agent AC#2 & Verification: executed bash scripts/run-wc-vectors.sh against the 25f56ed68 vectors using $TMPDIR scratch space (rc=0, summary: 25 cases: pass=18 fail=0 not-implemented=6 n/a=1) and confirmed both ^  CC-03 +pass  (CC-03  pass             Supervisor crash before commit) and ^  CC-06 +pass  (CC-06  pass             Retry while the old worker survives) match and pass.
GUIDANCE: none

Summary: 1 green, 0 amber, 0 red, 0 escalate, 0 not-evaluated
```

## The worker's full output (result.md)

```
1. [AC] [REVIEW] Close-readiness: the work meets its Agent criteria and the Verification section passes (T-3526, reviewer-judged, T-4156)
VERDICT: green
WHY: Verified both Agent ACs and the Verification command in [.tasks/active/T-3526-wc03-hub-a-running-attempt-never-seen-al.md](file:///tmp/tl-dispatch/judge-t-3526-r3-b0f57e5be9c0/tree/.tasks/active/T-3526-wc03-hub-a-running-attempt-never-seen-al.md#L66-L119). (1) Agent AC#1: [run-wc-vectors.sh](file:///tmp/tl-dispatch/judge-t-3526-r3-b0f57e5be9c0/tree/scripts/run-wc-vectors.sh#L10-L21) pins AEF `REF="25f56ed68"`, whose `vectors.yaml` adds `seen_alive_epoch: E1` preconditions on `CC-03` (line 163) and `CC-06` (line 305) and documents in `vocabulary.ops.confirm_end` (line 62) that an attempt never seen alive leaves only through deadlines (`T6b`/`T6`), never `T4` (§3.4). In [hub.rs](file:///tmp/tl-dispatch/judge-t-3526-r3-b0f57e5be9c0/tree/crates/termlink-hub/tests/wc_conformance/hub.rs#L297-L303), [`Hub::seed`](file:///tmp/tl-dispatch/judge-t-3526-r3-b0f57e5be9c0/tree/crates/termlink-hub/tests/wc_conformance/hub.rs#L253-L339) seeds `seen_alive_epoch` via `self.live.insert((id.clone(), n))` and `self.sweep_seed(t0, true)` while the store's [`absence`](file:///tmp/tl-dispatch/judge-t-3526-r3-b0f57e5be9c0/tree/crates/termlink-bus/src/worker_contract/sweep.rs#L217-L249) check in [sweep.rs](file:///tmp/tl-dispatch/judge-t-3526-r3-b0f57e5be9c0/tree/crates/termlink-bus/src/worker_contract/sweep.rs#L218-L231) remains strict (`c.binding_seen_at.is_some()` and `c.seen_epoch == Some(epoch)`). (2) Agent AC#2 & Verification: executed `bash scripts/run-wc-vectors.sh` against the `25f56ed68` vectors using `$TMPDIR` scratch space (`rc=0`, `summary: 25 cases: pass=18 fail=0 not-implemented=6 n/a=1`) and confirmed both `^  CC-03 +pass ` (`CC-03  pass             Supervisor crash before commit`) and `^  CC-06 +pass ` (`CC-06  pass             Retry while the old worker survives`) match and pass.
GUIDANCE: none

Summary: 1 green, 0 amber, 0 red, 0 escalate, 0 not-evaluated
```
