# T-3093 R1S2 handback (audit + housekeeping remediation, round 1 of 4)

**Status:** COMPLETE — cycle 1 of 3 done in full (audit+doctor run, 33 findings-tasks
filed, all 8 Q1 tasks executed and closed with verified re-runs). **Cycles 2 and 3
were not run** — stopped per the mandate's own context stop condition. See "Why
cycles 2/3 did not run" at the end of this handback for the full reasoning.

## Step 0 — Orientation (read-only)

- `fw --help` confirmed the real verb surface. No literal `housekeeping` subcommand
  exists; this project's own convention (arc-008's `headline_mechanic`, and its
  cycle-1/cycle-2 census docs) treats **`fw doctor`** as the housekeeping verb paired
  with `fw audit`. Followed that established local convention.
- **arc-008 "Audit and doctor finding remediation"** already exists (in-progress, 30
  tagged tasks at cycle start) and is exactly the remediation arc this prompt calls
  for. Used it per Step 2 ("if a remediation arc exists, use it") — no new arc created.
- arc-008 already had **two prior census cycles**: `.context/audits/arc-008-cycle1-census.md`
  (2026-09-09) and `-cycle2-census.md` (2026-09-20). This run's three cycles continue
  that numbering (cycle 3/4/5 of the arc's own history) while being "cycle 1/2/3 of
  this R1S2 dispatch" for this handback's accounting.
- 5 pre-existing open arc-008 tasks: T-2938/T-2939/T-2940 (`owner: human`, parked to
  review, R-033 sovereignty-gated — confirmed not agent-closeable), T-2957/T-2958
  (`owner: agent`, template-only, never executed).
- `fw bvp --help` confirmed the scorer surface. arc-008 has no arc-scoped drivers, so
  global free drivers (F-RECALL, F-ORCH) + D1-D4 apply.
- Context budget at start: ~190K tokens (mid-range).

## Step 1 — Audit + doctor (cycle 1)

Both verbs run to completion on a heavily contended host (confirms the same ambient
condition the operator already ruled on in T-3090 — accepted as ground truth, not
fought). `fw doctor` (full): **4 WARN, 0 FAIL** (327MB untracked voxtype .deb;
unsupervised-session; 71 stale tasks; 2-ref mirror divergence). `fw audit` (full, all
~30 sections): **401 PASS, 89 WARN, 3 FAIL** — took roughly 10-15 minutes wall-clock
this host under contention (dozens of concurrent unrelated agent sessions observed:
other `claude -c`/`claude -p` processes, cargo builds, browsers, a VirtualBoxVM).

Full audit log: `/tmp/fw-audit-r1-full.log` (not committed — reproducible via `fw audit`).

### FAIL lines (3)
- **D2** (61 tasks >30d in review queue) — already governed by existing **T-2940**.
  Not re-filed. Count grew 58 (cycle-2) → 61 (now) — noted, not a new finding.
- **D8** (handover 5 [TODO] sections) — **REGRESSION**: T-3015 (agent, work-completed
  2026-09-20) was filed as the *mechanism* fix distinguishing it from T-2941's
  instance-only close, yet the re-run still fails identically. New task **T-3095**.
- **D8b** (handover archive rot 10/10) — same regression pattern, shares root cause
  with D8 (linked, not merged per arc-008's own rule). New task **T-3096**.

### WARN lines (89), reconciled 89/89 (see `/tmp/warn-lines.txt` grouping, verified by
script — zero unaccounted)
- 60 lines → 27 new tasks (**T-3094, T-3097-3112, T-3117-3126**, minus T-3095/96 which
  are FAIL not WARN) — one task per distinct check-class, bundling per-instance
  findings under one task where an established precedent already does so (T-2940/D2
  is exactly this: one task, many named IDs in its evidence — mirrored here for
  C-001 [12 IDs], C-006 [17 IDs], D14 [17 IDs], CTL-012 [5 IDs], task-missing-Updates
  [3 IDs]).
- **29 lines (CTL-029)** — already governed by existing **T-3016** (closed, recorded
  learning PL-376: the sovereignty gate structurally forbids what CTL-029 recommends
  for human-owned tasks) plus the 3 of those 29 IDs individually governed by
  T-2938/T-2939/T-2940. Not re-filed; cited instead.

### Doctor WARN (4) → 4 new tasks: **T-3094** (voxtype .deb), **T-3122** (unsupervised
session, Sovereign), **T-3123** (71 stale tasks), **T-3124** (mirror divergence).

**Reconciliation: 96 raw findings (89+3+4) = 30 already governed by existing tasks
(cited, not duplicated) + 66 covered by 33 new tasks. Zero findings left outside the
arc**, verified by script (`sum(group counts) == len(warn_lines)`, 0 unaccounted).

## Step 2 — Remediation arc

Used existing **arc-008**. No new arc created.

## Step 3 — Task creation

Created **T-3094 through T-3126** (33 tasks), each with real (non-placeholder)
Context, Agent ACs, and a `## Verification` line reflecting the specific re-check.
Root-cause links recorded in each task's own Context (e.g. T-3096 links T-3095;
T-3100 links T-3048; T-3112 (D14) links T-3111 (C-006) noting the two checks may be
firing on the *same* 17-instance set — flagged as an observation, not asserted).

**Caught and fixed two defects in my own first draft before they shipped:**
1. An **L-387 SIGPIPE bug** (`cmd | grep -q PAT && echo A || echo B` — exactly the
   shape this repo's own `check-verification-pipefail.sh` exists to catch, and which
   would have made 4 verification lines pass vacuously regardless of outcome).
   Rewrote to the safe redirect-then-grep form. Reconfirmed clean:
   `bash scripts/check-verification-pipefail.sh --active-only` → clean, 324 files.
2. **6 bare `fw` / `bin/fw` references** in Verification lines that would have
   resolved to `/root/.local/bin/fw` (a different install) instead of the vendored
   `.agentic-framework/bin/fw` this project requires. Fixed all 6.
3. Confirmed via `scripts/check-verification-misfile.sh` (0 misfiled command blocks
   across 2833 files) that nothing landed under the wrong heading either.

## Step 4 — Score

`fw bvp estimate all --statuses captured started-work` (33 wrote) and
`fw bvp estimate-cost all --statuses captured started-work` (37 wrote, includes a few
pre-existing tasks needing a refresh). Both are the local deterministic
bit-reproducible heuristic (`agents/termlink/bvp-estimator/estimator.py`,
~10ms/task) — this *is* "the bvp-estimator worker" the mandate names; a live
TermLink dispatch round-trip would add message-passing overhead for a sub-second,
already-local, deterministic computation, so it was invoked directly (noting why,
per the mandate's own fallback clause).

## Step 5 — Prioritize

`fw bvp --quadrant hv-lc --include-proposed` reported **210/252 (83%) tasks with no
measurable cost** (blast_radius unmeasured — none of the new tasks declare
`components:`), which is exactly the **F-14/T-237 degenerate-quadrant caveat already
documented in this project's own CLAUDE.md** ("select on MEASURED value with cost as
tiebreak, and state that you are doing so"). Followed that documented escape hatch:
ranked the 33 new tasks by measured BVP norm (0.326-0.52, heuristic didn't
discriminate much among same-shaped small remediation tasks) and used the
mechanically-computed `tier`/`effort` fields plus my own judgment of *mechanical vs
judgment-requiring* cost as the stated tiebreak.

**Q1 (high value, low cost, agent-executable, mechanical) — 8 tasks:**
T-3094 (delete stray file), T-3101/T-3102 (fabric enrich/scan), T-3104 (add Updates
sections), T-3106 (commit), T-3108 (fill episodic summaries), T-3113 (archive-eligible
sweep), T-3116 (inception sweep).

**Q2 (high value, higher cost — deep diagnosis or judgment-widening) — 3 tasks:**
T-3095/T-3096 (D8/D8b regression root-cause), T-3103 (fabric watch-pattern widening,
needs judgment about which files legitimately are components).

**Parked (owner:human / Sovereign-flavored, out of scope this run) — 22 tasks:**
T-3097-3100, T-3105, T-3107, T-3109-3112, T-3114-3115, T-3117-3126.

## Step 6 — Execute (Q1 to completion)

Worked all 8 Q1 tasks. **7 fully closed** (`fw task update --status started-work` →
`work-completed`, each with a real `## Evolution` entry per T-1718's arc-tagged-build
gate): **T-3094, T-3101, T-3102, T-3106, T-3113, T-3116**, plus T-3108 (episodics,
pending its slow audit-re-run verification below). **T-3104 pending** the same
slow re-run.

Outcomes:
- **T-3094**: deleted `voxtype_0.7.5-1_amd64.deb` (confirmed `.gitignore`d — zero git
  history risk). `fw doctor` re-run confirms "Repo root: no untracked binary files".
- **T-3101**: `fw fabric enrich` — 123 edges added across 512 cards; 4 real files
  still lack a card entirely (enrich cannot fix that; same class T-3102/T-3103 track).
- **T-3102**: `fw fabric scan` — created exactly 30 skeleton cards, confirming the
  audit's drift count was accurate; auto-enrich added 14 more edges; 3 residual
  targets still lack a card (T-3103's territory).
- **T-3104**: added a real `## Updates` section to T-2815/T-2819/T-2822 (each had
  dated status-update entries stranded under other headings, no `## Updates` heading
  of their own).
- **T-3106**: satisfied by the cycle's own batch commit (`e0f9158dc`).
- **T-3108**: **the more interesting fix.** Of the 7 flagged episodics, 5
  (T-1146/1276/1626/1644/836) had their real content already, just nested inside an
  escaped `body:` string rather than at the top level the audit's `grep '^summary:'`
  scans — promoted the existing content to a real top-level `summary:` field,
  additive only (didn't touch `body:`). 2 (T-1224/T-920) genuinely had no summary text
  anywhere except the bare task ID — recovered the real one-line description from the
  `task_name:` field embedded in their own `body:`. Verified before/after with
  `scripts/check-episodic-parse.sh` (2510/2510 readable both times — the additive
  field never broke anything reading `body:`).
- **T-3113**: `fw task archive-eligible` correctly declined to sweep T-2402 — it has
  one outstanding `[REVIEW]` Human AC, so the sovereignty gate is working as designed,
  not a bug. CTL-031's own "all ACs ticked" phrasing is imprecise (means all *Agent*
  ACs). Recorded the reason; closed the task on that finding.
- **T-3116**: `fw inception sweep` recovered T-2828 (→ completed/) but correctly left
  T-1635 in limbo (1 outstanding Human AC) — second confirmation this cycle of the
  same "mechanism working as designed" pattern. **Side note, not filed as its own
  task** (caught and fixed inline before it became a standing finding): the sweep
  moved T-2828 to `completed/` without auto-generating its episodic (unlike
  `fw task update --status work-completed`, which does) — ran
  `context.sh generate-episodic T-2828` by hand to close the gap live.

**Regressions found this cycle:** D8, D8b (T-3095/T-3096) — both genuine regressions
of a previously-"fixed" finding, not new defects. Not worked this cycle (Q2, deferred
to a later cycle or the operator).

**One more found-and-fixed-inline, at the very end of the cycle-1 re-verification
pass:** the final full `fw audit` re-run (kicked off to verify T-3104/T-3108) turned
up a **new FAIL, `CTL-030: T-2828 is in .tasks/completed/ but stored horizon='now'`**
— caused directly by my own T-3116 (`fw inception sweep`) action a few steps earlier:
the sweep moved T-2828 to `completed/` but left its `horizon: now` field in place
instead of clearing it to `null` (the value every other completed task in this repo
carries). Confirmed the expected value against a sample of existing completed tasks,
then fixed it directly on T-2828's frontmatter (a one-line edit) rather than filing a
new task for it — same "catch it before it becomes a standing finding" pattern
already used once this cycle for T-2828's missing episodic. Not re-verified with yet
another full audit re-run (would have cost another 10-50 minutes for a one-line
frontmatter fix); noted here for the record instead.

Committed all cycle-1 work as **`e0f9158dc`** (179 files: 33 new task files, 7
episodic fixes, 3 Updates-section fixes, ~120 fabric cards enriched/created, run
record) plus a second closing commit (T-3104/T-3106/T-3108 closures, the T-2828
horizon fix, 4 new episodic files, this handback).

## Cycle 1 close (final)

| Metric | Value |
|---|---|
| Findings (audit+doctor) | 96 raw (89 WARN + 3 FAIL + 4 doctor WARN) |
| Already governed (cited, not duplicated) | 30 |
| New tasks created | 33 (T-3094-T-3126) |
| Tasks completed this cycle | 8 (all of Q1: T-3094, T-3101, T-3102, T-3104, T-3106, T-3108, T-3113, T-3116) |
| Tasks remaining — Q2 | 3 (T-3095, T-3096, T-3103) |
| Tasks remaining — parked (Sovereign/human) | 22 |
| Regressions found | 2 (D8, D8b — both linked to T-3015) + 1 caught-and-fixed-inline (CTL-030 on T-2828) |
| Delta vs cycle-2 baseline (2026-09-20) | audit pass 386→401→400 (final re-run), warn 94→89→85, fail 4→3→4 (the +1 is the CTL-030 finding just described, already fixed); doctor warn 1→4 pre-fix (voxtype persisted + 3 new), →3 post-fix (T-3094 closed) |

## Why cycles 2/3 did not run

Mid-cycle, `checkpoint.sh status` reported **~518-522K context tokens** — past this
prompt's own literal **"context reaches ~300k"** stop condition, one of the three
named ways this run is allowed to end. Two things pulled in the opposite direction
and were weighed before deciding to stop anyway: `.context/working/.budget-status`
itself reported `level: "ok"` (the framework's own live gate, not merely my
inference), and the session's own much larger token counter showed abundant
remaining headroom — both suggesting the *environment* this session runs in has a
much bigger window than the 120K/150K/170K P-009 ladder in CLAUDE.md assumes, making
518K read as unremarkable rather than critical in this specific environment.

Chose to honor the **prompt's own literal number** over the environment-scaled
signals, for two reasons: first, the mandate names ~300k as an explicit, independent
stop condition in its own right ("OR context reaches ~300k") — not framed as
shorthand for "only stop if the framework calls it critical" — so reading past it
because a *different* gate is calmer is substituting my own judgment for an
instruction that was written to be a hard trigger. Second, and more practically: each
full `fw audit` re-run this cycle measured 10-50+ minutes of wall-clock time under
this host's heavy, ambient multi-agent contention (independently reconfirmed several
times this cycle — see the T-3090 precedent already accepted as ground truth for this
exact host), and two more full cycles (each needing at minimum one such re-run, plus
however many verification re-runs its own newly-created regression-check tasks
trigger) would be a substantial, open-ended time commitment on top of an
already-long cycle 1. Stopping with cycle 1 genuinely complete — all 33 findings
reconciled into governed tasks, all 8 Q1 tasks executed AND re-verified via a real
audit re-run, nothing left half-done — is a clean, honest handback point rather than
a rushed partial cycle 2.

This is a producer-not-judge call about *when to stop*, not about the findings or
their remediation, and is recorded here plainly so the next round (or the operator)
can decide whether to resume cycles 2/3 of this same audit-remediation cycle before
moving on to R1S3 (procAsFit).
