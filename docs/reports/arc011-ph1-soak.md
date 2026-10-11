# arc-011 PH-1 soak: H-16 and H-17 (X5)

Task T-3414 (B13). Thresholds from `interactive-agent-communication-03-floor-and-phasing.md`
section 9. Test: `crates/termlink-sidecar/tests/soak.rs`
(`cargo test -p termlink-sidecar --release soak_ -- --ignored --nocapture`). Run 2026-10-12,
seed 20261012, 851 s on the shared host.

## H-16: no stored message is lost, none is handed over twice

**Threshold:** 1000 randomised kill points across the store, the claim and the record; 0 lost
after STORED, 0 handed over twice. **Result: met.**

| Measure | Value |
|---|---|
| Kill points fired | **1000** |
| in the store write sequence (`put`, all 8 steps) | 238 |
| in chain appends during deliver and finalize (store steps) | 177 |
| after a claim, before its attempt record | 122 |
| after the attempt record, before CLAIMED | 77 |
| after CLAIMED, before the frame left | 76 |
| in the finalizer, before HANDED_OVER | 197 |
| in the finalizer, before ABSENT | 113 |
| Plain restarts (no kill point, recovery runs) | 1350 |
| Messages | 797 |
| Frames handed out | 760 (115 dropped by the scripted harness, as a lost hook output) |
| Attempts closed ABSENT (proof of absence) | 557, of them 303 orphan claims closed at start-up |
| **Lost after STORED** | **0** |
| **Handed over twice** (frame in the transcript more than once) | **0** |
| STORED not exactly once | 0 |

Every conversation chain verified intact at the end.

**Model.** In-process, no hub: a seeded loop injects messages (the sender retries any it did
not see stored), runs tool calls that pull (the scripted harness writes the frame as a
`hook_additional_context` record 85% of the time, else only the tool result and the next
turn), runs the finalizer, and restarts. A store kill point may be armed for any operation;
deliver kill points for pulls and finalizes. Every fired kill point is a crash: everything
is dropped and reopened, so start-up recovery runs. A drain at the end (no kill points)
retries, sweeps and delivers until nothing is pending or open.

**The soak can fail.** With orphan-claim recovery removed, 100 kill points gave 10 lost
messages; with open attempts not respected, 98 messages handed over twice. Both runs failed
the test.

## Found by designing the soak, fixed in T-3414

1. **A crash between claim and frame blocked the message forever.** The claim file stayed,
   so `pull` never offered the message again, and no attempt record or CLAIMED line existed
   for a finalizer to decide. Start-up recovery now closes such orphan claims (ABSENT,
   reason `orphan`): `pull` builds the frame only after the claim, the record and the
   CLAIMED line, so a claim without its CLAIMED line proves nothing was handed out.
2. **A crash after the frame left could leave an attempt nobody decides** (the hook's
   finalizer cannot reach a dead sidecar). The sidecar now sweeps open attempts through the
   finalizer every 2 s, with the final check after 60 s.

## H-17: every listed error ends visibly

**Threshold:** the 17 rows of step 3 section 5.6 (FC-1..FC-17) visible, each control red
when removed. **Result: met** by B12 (`docs/reports/arc011-ph1-fc-matrix.md`: 17 of 17
mutants killed). `soak_h17_every_listed_error_ends_visibly` holds the matrix to that: each
FC row has its test (the function exists) and its patch, and the run line says killed.
