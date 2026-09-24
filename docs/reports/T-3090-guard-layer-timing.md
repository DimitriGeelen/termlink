# T-3090: guard-layer per-member timing — a real, contended-host measurement

**Status: this is a CONTENDED-host measurement, not a quiet-host one.** See "Why contended, not
quiet" below before treating the total as a stable baseline. It is offered as the best available
real data rather than withheld, per the task's own acceptance criterion — but it should not be
read as the floor the guard layer can achieve on an idle machine.

## Run identity

- Command: `bash scripts/run-guard-layer.sh --json`
- Start: `2026-09-24T21:56:27Z`
- End: `2026-09-24T22:12:12Z`
- **Wall time: 945s (15m 45s)**
- **Sum of per-member `elapsed_s`: 939.5s** — wall time and the sum of member times are within
  6s of each other, i.e. the runner's own overhead (fork/exec, JSON assembly) is negligible; the
  cost is genuinely in the 137 members, not the harness around them.
- Members scanned: 137 total — 131 PASS, 6 FAIL, 0 ERROR, 76 unmarked/unclassified (not run; see
  the runner's own `# guard-layer:` marker convention in CLAUDE.md).
- Exit code: 1 (the layer is currently FIRING — see "Firing members" below; unrelated to timing,
  reported for completeness since it's the same run).

## Why contended, not quiet

`uptime` immediately before dispatch: `load average: 11.09, 12.05, 13.67` on a 24-core host.
Immediately after: `9.99, 14.51, 15.53`. Roughly 450 `claude`/`termlink register` processes were
running concurrently for the whole measurement window — this is the T-3089 orchestrated
procAsFit sequence itself (4 rounds' worth of tmux/termlink sessions still alive) plus a large
number of unrelated persistent agent sessions on this shared host (T-3044's value-review worker,
several other task sessions, systemd-managed production agents for other projects). None of that
was stopped to get a cleaner number — killing other sessions' in-flight work to produce a nicer
timing figure would be a worse trade than an honestly-labelled contended one.

The operator's ruling on this task (recorded verbatim in
`.context/runs/T-3089-procasfit-x4.yaml` under `operator_rulings`) says: *"Measure it properly:
per-member timing, quiet host, then correct CLAUDE.md."* This host was not quiet, is not quiet as
a matter of routine operation (it appears to run many concurrent long-lived agent sessions as its
normal state, not as a transient spike), and this round did not have the authority to make it
quiet. This measurement is therefore offered as the honest, labelled, contended-host figure the
task's own acceptance criterion anticipates ("explicitly labelled as contended... given this
host's load during the run") — not as proof the "quiet host" ruling has been satisfied. See the
R4 handback's Sovereign Questions for the open question this leaves: whether a contended-but-
honest number is an acceptable substitute given the practical difficulty of ever finding this
host quiet, or whether a dedicated quiet window needs to be scheduled.

Three independent measurements now agree on order of magnitude, all under some degree of
contention:
- R1 of T-3089 (2026-09-24, per T-3090's own filing): ~17 minutes.
- The orchestrator's separate run cited in T-3090's filing: exceeded a 550s timeout, resumed, ~15
  more minutes.
- This run: 15m 45s (945s).

"Seconds" (the current CLAUDE.md claim) is wrong by roughly two orders of magnitude regardless of
which of these three is closest to a true quiet-host figure.

## Where the time actually goes (top 15 members by elapsed_s)

| elapsed_s | verdict | member |
|---:|---|---|
| 274.90 | PASS | `check-verification-heading-shadow.sh` |
| 118.42 | PASS | `check-unbounded-rpc-call.sh` |
| 91.79 | PASS | `unbounded-rpc-call-fixtures.sh` |
| 53.09 | FAIL | `test-pushwaker-ready-loop.sh` |
| 32.88 | PASS | `check-error-swallowing-predicate.sh` |
| 32.73 | PASS | `error-swallowing-check-fixtures.sh` |
| 31.68 | PASS | `check-platform-lock.sh` |
| 21.10 | PASS | `check-drain-sink-caps.sh` |
| 20.97 | PASS | `check-episodic-parse.sh` |
| 19.43 | PASS | `check-busy-spin.sh` |
| 16.86 | PASS | `check-task-id-collisions.sh` |
| 15.61 | PASS | `notify-wake-supervisor-fixtures.sh` |
| 14.99 | FAIL | `check-unpaired-capture.sh` |
| 14.42 | PASS | `check-vacuous-verification.sh` |
| 14.02 | PASS | `check-fleet-recipient-agreement.sh` |

The single slowest member, `check-verification-heading-shadow.sh` (274.9s), is by itself ~29% of
the entire run's wall time. Combined with `check-unbounded-rpc-call.sh` + its own fixture suite
(118.4s + 91.8s = 210.2s), these three members alone account for 485.1s — over half of the total
939.5s — while the remaining 134 members share the other ~454s. This is a real long-pole
distribution, not uniform slowness across the layer: a future optimisation pass has a small,
named target list rather than "make everything faster."

## Firing members (unrelated to timing, recorded for completeness)

Six members reported FAIL in this run: `check-arc-claim-drift.sh`, `check-pickup-deferred-
freshness.sh`, `check-receiver-ack-lag.sh`, `check-unpaired-capture.sh`, `runme-fixtures.sh`,
`test-pushwaker-ready-loop.sh`. Diagnosing or fixing these is out of scope for T-3090 (which is
about the runner's timing claim, not the current health of the layer's members) — noted here only
because they were produced by the same run and a reader comparing this report against a future one
should know the exit code was not 0.

## Conclusion for CLAUDE.md

The documented `(seconds)` budget is wrong by roughly two orders of magnitude under real-world
conditions on this host. `CLAUDE.md` has been corrected to cite an order-of-magnitude figure
(minutes, not seconds) with a pointer to this report, rather than asserting a single precise
number this report cannot honestly stand behind as a quiet-host baseline.
