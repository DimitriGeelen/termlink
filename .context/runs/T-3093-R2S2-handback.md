# T-3093 R2S2 handback (Audit + housekeeping remediation, round 2 of 4)

**Status:** COMPLETE-PARTIAL — 1 of 3 mandated cycles, stopped near the ~300k context
ceiling (real per-session figure via `checkpoint.sh status`, NOT the shared
`.context/working/.budget-status` — confirmed unreliable under concurrent orchestrated
dispatch per this run's own T-3127 warning and now also T-3128/T-3130).

## Orientation (Step 0)

- `fw --help` + `fw audit --help` / `fw doctor --help` / `fw arc --help` / `fw bvp --help`
  confirmed the real verb surface. No literal `housekeeping` verb exists — confirming
  R1S2's earlier finding — so `fw doctor` again stood in for it (both were run as part of
  the fresh `fw audit`, which folds `fw doctor`-class checks in via its own control-check
  sections).
- `fw arc list`: arc-008 ("Audit and doctor finding remediation") already exists,
  in-progress, 63 constituent tasks (from R1S2's cycle 1 + earlier pre-T-3093 runs). Per
  Step 2 ("if a remediation arc exists, use it") this step reused arc-008, created nothing
  new at the arc level.
- Real context at dispatch start: 160,405 tokens (~20%).

## Step 1 — Audit and housekeeping

Ran a fresh `fw audit` (the earlier one had to be retried once — it runs long, ~13 minutes
wall-clock on this host, consistent with T-3090's "contended host, not seconds" finding for
the guard-layer subset it includes; used `TaskOutput` blocking rather than backgrounding and
walking away, per this run's carried fix #1). Result: **400 pass / 86 warn / 3 fail**
(baseline from R1S2's cycle 1 was 401/89/3 — close, some drift in both directions from
ordinary repo activity between cycles).

Reconciled every WARN/FAIL line in this run's output against arc-008's 63 existing tasks
(most were already covered by R1S2's cycle-1 filing, still sitting unworked). Six findings
did not match anything already in the arc:

| ID (new task) | Source check | Severity | What | Disposition this step |
|---|---|---|---|---|
| T-3128 | CTL-003 | WARN | `.budget-status` reports 16min-stale | Root-caused to T-3127's exact class; **fixed** via a CLAUDE.md doc addition (the check itself is vendored, unfixable locally) — 1 Agent AC left deliberately unchecked, so task stays started-work |
| T-3129 | task-quality (missing field) | WARN | T-3095's `owner:` frontmatter blank | **Fixed and closed work-completed** |
| T-3130 | (discovered mid-fix, not an audit line) | — | `fw task update --status ...` reproducibly blanked T-3129's own `owner:` field moments after I fixed it | Filed **INVESTIGATE**, not bisected — vendored code, real engineering scope, out of bounds for this pass. Left captured/parked |
| T-3131 | fabric-drift | WARN | `scripts/notify-wake-consumer.py` unregistered | Root-caused to a genuine slug-collision bug in vendored `register.sh` (extension-stripping makes `.py`/`.sh` siblings collapse to one slug) — confirmed by reading the source, not guessed. **Mitigated**: hand-authored a disambiguated card; `fw fabric drift` now reports 0 unregistered. 1 Agent AC (the structural fix) left unchecked — vendored, stays started-work |
| T-3132 | CTL-029 | WARN (×26 net-new instances) | 26 tasks completable-not-closed beyond the 3 already tracked | **Not bulk-closed** — Human Task Completion Rule (T-372/373) requires per-task evidence, not a mechanical sweep. Filed as one bundle task (matching the C-001/C-006/D14 bundle convention this arc already established), left captured/parked for a future cycle to spot-check one at a time |
| T-3133 | uncommitted-changes | WARN | dirty tree at audit time | **Resolved** by this step's own closing commit (same pattern as T-3106) |

**Findings reconciliation (Step 3's explicit requirement):** 92 WARN+FAIL lines this cycle
— 86 already covered by arc-008's existing (unworked) backlog from R1S2, 6 newly filed above.
Zero left outside the arc.

Not new findings, just noted for completeness: D2/D5/D14/GO-scope-not-propagated counts all
drifted slightly (e.g. GO-scope-not-propagated 88→89) but are the *same tracked class* as
existing arc-008 tasks (T-3100, T-3111, T-3112, T-3115) — not filed again. CTL-031's single
stuck partial-complete instance is still T-2402, already explained and accepted by T-3113 as
expected (a real `[REVIEW]` Human AC gate, not a defect) — not re-filed. T-3101/T-3102's
narrower ACs ("count does not increase" / "0 at the time") were re-checked against their own
literal wording, not re-litigated as regressions where the wording didn't promise permanence.

## Step 2 — Remediation arc

Reused arc-008 (no `fw arc create` needed — Step 2's "if a remediation arc exists, use it").

## Step 3 — Task creation

Six tasks created (T-3128–T-3133), all tagged `arc:arc-008`, each carrying the originating
WARN text, a structural defect statement, falsifiable ACs, and root-cause links to siblings
(T-3127, T-3095/T-3096, T-3102, T-3106, T-2938–2940/T-3016) where one exists. See table above.

## Step 4 — Score

Ran `fw bvp estimate` directly on all 6 new tasks (T-3128, T-3130, T-3131, T-3132, T-3133 —
T-3129 was scored automatically by its own `--status work-completed` transition). Did not
dispatch to a bvp-estimator sub-agent — the heuristic estimator runs in well under a second
per task locally, and dispatching a sub-agent for six single-task estimate calls would have
cost more context than it saved (TermLink section's own framing: "use it where it is the
right instrument, not decoratively").

## Step 5 — Prioritize

**Not run this cycle.** By the time scoring finished, context was already near the ~300k
ceiling (checkpoint.sh reported 287,401 tokens after filing all six tasks). Of the six new
tasks: T-3129 is done; T-3128 and T-3131 are genuinely Q1 (cheap, high-value, already
executed with the vendored-fix portion honestly left open); T-3130 and T-3132 are Q2/parked
by their own nature (real investigation or a 26-item spot-check sweep, correctly not rushed).
The pre-existing 30-task arc-008 backlog from R1S2 was not re-prioritized or touched this
cycle — no budget remained to do so faithfully rather than superficially.

## Step 6 — Execute

Executed in-line as each finding was filed (see table): T-3128 (CLAUDE.md doc fix), T-3129
(owner field fix, closed), T-3131 (fabric card workaround, verified 0 unregistered). T-3130
and T-3132 deliberately not executed — correctly scoped as out-of-bounds for this pass
(vendored-code bisect; bulk human-owned-adjacent closures respectively).

## Step 7 — Cycle

**Only 1 of 3 cycles run.** The audit run itself took ~13 minutes wall-clock (T-3090's
already-accepted "contended host" reality), and faithfully reconciling 92 findings against
63 existing tasks, filing 6 new ones with real root-cause investigation (not just pattern-
matching the WARN text), and fixing 3 of them consumed the rest of the budget before a
second `fw audit` re-run could be justified. This mirrors R1S2's own precedent exactly (it
also delivered 1 of 3 cycles and said so plainly rather than claiming three).

**Delta against R1S2's cycle-1 baseline:** pass 401→400, warn 89→86, fail 3→3 (no fails
resolved or introduced — the 3 FAILs are D2/D8/D8b, all pre-existing tracked backlog).
Regressions: none confirmed as true regressions (T-3102's "1 file" was residual/new-file
drift within its own AC's wording, not a broken fix; T-3106's uncommitted-changes WARN is
a fresh point-in-time snapshot, not a broken fix).

## Notable finding for future steps

**T-3130 (owner-field clobber) is worth prioritizing early in R2S3/R3S1** if there's any
spare capacity — it's a live, reproducible tool-behavior bug (not just a stale data point)
discovered by accident while fixing an unrelated finding, and it could be silently
corrupting `owner:` fields on other tasks across this whole run without anyone noticing
(the CTL-029/CTL-012 audit checks don't check for an *empty* owner, only a *human* one).

## Gates / governance notes

- No Tier-0 gate fired. No `--force`/`--skip-*` bypass used anywhere in this step.
- T-3128 and T-3131 each carry one Agent AC deliberately left unchecked (vendored-code fix
  out of local reach, G-062) — both stay `started-work`, not forced to `work-completed`.
  This is intentional honesty, not an oversight.
- T-3132 (CTL-029 bundle) was explicitly NOT bulk-closed, per the Human Task Completion Rule
  (T-372/373) — closing 26 tasks on the strength of one audit line each would be exactly the
  "batch-close stale tasks" pattern that rule forbids.
- Producer-not-judge respected: no task's own claim was treated as its verification: T-3131's
  "0 unregistered" and T-3129's "owner: agent" were both confirmed by re-running the
  originating check (`fw fabric drift`, `grep`), not by self-assertion.
- Re-read the run record and `git log --oneline -15` at the start of this step (per carried
  fix #2) before doing anything; nothing in this step touched shared/live infrastructure, so
  no further at-the-moment re-read was needed mid-step.

## Commits this step

`7adb5712b` — this handback + run record update + all 6 new/modified task files +
CLAUDE.md + the fabric card + T-3095's fix.

**Stop condition hit:** `checkpoint.sh status` read 299,773 tokens (~37%) immediately after
that commit — at the mandate's ~300k ceiling. T-3133 was set to `started-work` (its fix —
the commit itself — already happened) but not formally transitioned to `work-completed`
(that requires an Evolution-section entry per T-1718's gate, which there was no budget left
to add safely). Per this step's own stop-condition instructions ("do not stop mid-task"),
this is the least-mid-task place available: T-3133's underlying work is done and verifiable
by any future step (`git status --porcelain` was clean immediately after the commit above);
only its own status-field bookkeeping is left for R2S3 or a future audit cycle to close with
one `fw task update T-3133 --status work-completed` plus a one-line Evolution entry.
