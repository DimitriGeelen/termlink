# T-3093 R1S2 handback (audit + housekeeping remediation, round 1 of 4)

**Status:** IN PROGRESS.

## Step 0 — Orientation (read-only)

- `fw --help` confirmed the real verb surface. No literal `housekeeping` subcommand exists;
  this project's own convention (arc-008's `headline_mechanic`, and its cycle-1/cycle-2
  census docs) treats **`fw doctor`** as the housekeeping verb paired with `fw audit`. I am
  following that established local convention rather than inventing a new one.
- `fw arc list` — 9 arcs. **arc-008 "Audit and doctor finding remediation"** already exists
  and is exactly the remediation arc this prompt calls for (in-progress, 30 tagged tasks,
  headline mechanic: "operator runs fw audit and fw doctor and reads a clean report").
  Per Step 2 ("if a remediation arc exists, use it"), I am using arc-008, not creating a
  new one.
- arc-008 has **two prior census cycles already on record**:
  `.context/audits/arc-008-cycle1-census.md` (2026-09-09) and
  `-cycle2-census.md` (2026-09-20). This run's three cycles continue that same numbering
  as cycle 3 / cycle 4 / cycle 5 of the arc's own history, while remaining "cycle 1/2/3 of
  this R1S2 dispatch" for the purposes of this handback's own accounting.
- arc-008 currently has 5 open (non-work-completed) tasks: T-2938, T-2939, T-2940
  (`owner: human`, `started-work` — parked to human review, NOT agent-closeable, per
  R-033 sovereignty gate verified directly against T-2940 in the cycle-2 census) and
  T-2957, T-2958 (`owner: agent`, `captured`, template-only ACs never filled in —
  legitimate candidates for this run's Step 6 execution if they score into Q1/Q2).
- `fw bvp --help` confirmed the scorer surface (`fw bvp estimate-cost`, `fw bvp weight`,
  `fw bvp --quadrant`). arc-008 has `scoped_drivers: []` — no arc-scoped drivers declared,
  so the global free drivers (F-RECALL, F-ORCH) plus D1-D4 apply, per Step 4's "otherwise
  the global free drivers apply."
- Context budget at start of this step: ~190K tokens per `fw doctor`'s own report (see
  below) — mid-range, not yet in the P-009 60-75% caution band. Monitoring as the run
  proceeds; three full cycles of `fw audit` (each observed taking several minutes to
  tens of minutes under this host's heavy, ambient multi-agent contention — same
  condition the operator already ruled on in T-3090: "the contended number IS the
  answer") is the dominant cost driver for this step, not my own token spend.

## Step 1 — Audit + doctor (cycle 1 of 3)

`fw doctor` (full, no `--quick`) completed cleanly: **4 WARN, 0 FAIL**. Full output
captured. `fw audit` (full, all sections) is still running in the background — this audit
script is a large one (~30 report sections; observed reaching only the ENFORCEMENT CHECKS
section, roughly a third of the way through, after several minutes of wall time on this
contended host) and carries its own internal 3000s self-watchdog
(`.agentic-framework/agents/audit/audit.sh:407`). Waiting for it to finish naturally before
building the findings table, per the same "don't fight the contended host" precedent.

This section will be replaced with the full findings table once both verbs complete.
