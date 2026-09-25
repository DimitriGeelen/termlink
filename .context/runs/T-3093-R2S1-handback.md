# T-3093 R2S1 handback (Review, round 2 of 4)

**Status:** COMPLETE — halted at Phase 5 [ASK] per this run's own `known_gates` entry
("The value-review prompt HALTS at PHASE 5 [ASK] for per-item human approval before
PHASE 6 EXECUTE. A review step that halts there is COMPLETE, not failed"). Same shape as
R1S1.

## Orientation

- Read the run record (`.context/runs/T-3093-review-audit-procasfit-x4.yaml`), R1S3's fed
  handback (inlined in this step's dispatch prompt, 19073B), and — since this round is
  explicitly a *delta* review — R1S1's own report + evidence file
  (`docs/reports/VALUE-REVIEW-repo-2026-09-25-R1{,-evidence}.md`) to avoid re-deriving
  work already done.
- Session context at start: 176,478 tokens (~22% of window) via `checkpoint.sh status` —
  the real per-session reader, not `.context/working/.budget-status` (see F2 below, which
  is exactly the defect this session's own orientation step had to route around).
- `git log --oneline --since="2026-09-25 01:00"` confirmed the 7 commits R1S2/R1S3
  produced since R1S1; re-read at each point evidence touched shared state (audit files,
  process table) rather than trusting the run record's narrative alone.

## What this round is

Per the run record, R2S1 = the value_review prompt again, round 2 of 4, fed by R1S3's
procAsFit handback. Consistent with R1S1's own precedent (explicitly a delta round, not
a from-scratch review — this is at least the 9th value-review pass over this repo), I ran
this round the same way: reconfirm the yardstick and data-availability map (Phase 0/1,
unchanged), then focus Phase 2/3 evidence-gathering on **what changed since R1S1** and
**closing out R1S1's own explicitly carried-forward INVESTIGATE items** rather than
re-walking the whole repo inventory a second time in one afternoon.

## Execution

Ran Phase 0-3 (GATHERER) and Phase 4-5 (JUDGE — same worker, confidence dropped one level
per the prompt's own rule) as a single pass:

1. **Re-confirmed yardstick/data-map** — unchanged since R1S1, no new contradiction.
2. **Re-checked R1S1's three explicit carry-forwards:**
   - **Invocation-audit sink growth** (predicted to grow by R2S1) — checked directly:
     `wc -l /var/lib/termlink/invocation-audit.jsonl` still reads **4**, same two
     timestamps R1S1 already cited, ~7 hours later, despite heavy MCP/CLI activity in the
     interim. R1S1's own prediction is **falsified** by this direct re-measurement — worth
     saying so plainly rather than letting the thread quietly drop. Filed as **R2-F1**
     (ADD, REPAIR-or-WIRE undetermined; needs a code read out of this round's scope).
   - **~30 long-lived `termlink mcp serve` processes, leaked or legitimate?** — ran the
     liveness cross-check R1S1 named but didn't attempt: `ps -eo pid,tty,etimes,cmd`
     cross-referenced against `who` (426 sessions) and `/dev/pts` (848 allocated) on this
     host. **30 of 31 are pts-attached** → resolved as legitimate open sessions, closing
     R1S1's open question (**R2-F3**, KEEP). **1 of 31 has no controlling tty** and the
     longest uptime (~32h) → cannot be an open interactive session by definition, but also
     cannot be asserted a leak from one data point alone (this run itself is proof
     legitimate headless dispatched workers exist) — filed as **R2-F4**, INVESTIGATE, not
     DELETE (no positive reason, single instance).
   - **DEFER-inception `revisit_at` coverage** — re-ran `fw review-queue`, read
     frontmatter on all 6 current DEFER-recommended pending-decision items directly. Only
     1 of 6 (T-2486) carries `revisit_at`, but this is **expected, not a defect**: the
     field is set by the human at `fw inception decide ... defer` time (per CLAUDE.md),
     and the other 5 haven't been decided yet — `fw review-queue`'s "DEFER" is `fw task
     verify`'s *recommendation*, not a recorded decision. Filed as **R2-F4** (renumbered
     in the report as part of the finding set) — the interesting residual is T-2486
     carrying `revisit_at` while still pending decision at 54 days old, flagged as a
     cheap follow-up, not chased further this round (single data point, budget
     discipline).
3. **New evidence surfaced organically while checking the above:** the run record's own
   `cross_step_findings` entry ("TWO workers independently found `.budget-status` stale...
   worth its own task") turned out, on inspection, to be a **4th recurrence** of an
   already-3×-closed defect class (T-2950/T-3018/T-3034, all `work-completed`, all filed
   upstream per G-062) — but in a genuinely **new framing**: those three closures were all
   about a single interactive session's missing `checkpoint.sh budget` subcommand; this
   session's recurrence is about **concurrent orchestrated `claude -p` workers** racing on
   one shared cache file, which is a materially different hazard the three prior closures
   never addressed and CLAUDE.md doesn't warn about. Verified no existing task/concern
   already names this concurrency framing (`grep` across `.tasks/active/`,
   `concerns.yaml`, `gaps.yaml` — no hits). Filed as **R2-F2** (ADD-SURFACE: one paragraph
   in CLAUDE.md's Context Budget Management section).
4. **Re-verified R1-F1/F1b (the 4 unused Cargo deps) had not regressed or been silently
   picked up** — still present, unexecuted, unchanged. Carried forward as **R2-F5**,
   still awaiting the same [ASK] R1S1 already raised — no new evidence, no re-litigation.

Wrote the evidence file (`docs/reports/VALUE-REVIEW-repo-2026-09-25-R2-evidence.md`) and
the report (`docs/reports/VALUE-REVIEW-repo-2026-09-25-R2.md`), following R1S1's own
structure so the two rounds read as a continuous series.

## Findings summary (see report for full table)

| ID | Class | One-line |
|---|---|---|
| R2-F1 | ADD (REPAIR/WIRE, undetermined) | Invocation-audit sink stalled at 4 records for ~7h despite heavy activity — R1S1's growth prediction falsified |
| R2-F2 | ADD (SURFACE) | G-087-class stale-shared-cache defect recurred a 4th time in a new concurrency framing; CLAUDE.md doesn't warn about it |
| R2-F3 | KEEP (resolves R1S1 open question) | 30/31 long-lived mcp-serve processes confirmed legitimate via pts/who cross-check |
| R2-F4 | INVESTIGATE | 1 anomalous no-tty mcp-serve process, ~32h uptime — single data point, not classified |
| R2-F5 | DELETE (carried, unchanged) | 4 unused Cargo deps — still pending R1S1's [ASK], not re-litigated |

No DELETE/REFACTOR item was executed this step (Phase 6 gate, per known_gates — same as
R1S1). Nothing here requires a NEW Sovereign decision; the two ADD findings are
agent-actionable once root-caused/drafted, and the carried DELETE candidate's existing
[ASK] is not reopened or duplicated.

## Stop condition

Context at write time: ~210K tokens (~26% of window) via `checkpoint.sh status`, well
inside the mandate's ~200k-token gathering allowance and nowhere near this session's
critical threshold. This round's evidence-gathering was lighter than R1S1's (fewer new
surfaces to stand up — mostly targeted re-checks of already-identified open questions)
so no early stop was needed.

## Gates / governance notes

- No Tier-0, G-020, or P-002 gate fired this step — this was read-only investigation plus
  two new-file writes under the already-focused, already-started-work T-3093 umbrella
  task, consistent with how R1S1 operated (the value-review prompt itself is a GATHERER/
  JUDGE role that produces a report, not code changes).
- Confirmed via direct re-read at the moment of checking (not from memory) that no
  live/shared infrastructure was touched: the `ps`/`who`/`wc -l`/`fw review-queue`
  invocations this step performed are all read-only, matching the GATHERER role's "Read-
  only. Does not classify [beyond Phase 4]" constraint and this run's own carried fix #2
  ("re-read at the moment of action" — applied here in spirit even though nothing
  mutating was on the table this step).

## Suggested next step (for R2S2 / audit-remediation)

R2-F1 (invocation-audit stall) and R2-F2 (CLAUDE.md concurrency warning) are both small,
concrete, agent-executable items an audit-remediation pass could adopt as governed tasks
under arc-009, mirroring how R1S2 treated R1S1's fully-evidenced findings as in-scope for
its own "convert findings into tasks" mandate. R2-F2 in particular is a one-paragraph doc
change with zero risk and immediate payoff for R3S1/R4S1 (the next two Review steps in
this same sequence), so it's worth prioritizing if R2S2 has any Q1-equivalent capacity
left after its own audit findings.

## Commits this step

None — this step produced two new report files
(`docs/reports/VALUE-REVIEW-repo-2026-09-25-R2{,-evidence}.md`) and this handback; commit
happens as the final action of this handback, referencing this step.
