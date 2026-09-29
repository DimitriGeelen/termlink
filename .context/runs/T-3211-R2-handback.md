# T-3211 — procAsFit round 2 of 9 — handback (attempt 2)

**Status: IN PROGRESS (skeleton written first per carried fix 3; sections fill as the round proceeds).**
Worker: `claude -p` (Fable 5.1), started 2026-09-29 ~01:34 local. Run record: `.context/runs/T-3211-R2-*`.
Transcript: `~/.claude/projects/-opt-termlink/8ff0310c-….jsonl` (confirmed mine: carries the
"re-running round 2" carry text ×3; attempt 1's `0d28539b…` carries it 0×).

## Baseline at run start

- HEAD: `c54b48002` (T-3127: close — closed by verb in T-3211 R2). Focus: T-3211.
- Budget at start: **~173K tokens (~22% of CONTEXT_WINDOW=800000)** — raw `cache_read_input_tokens`
  172,619 on my own transcript's latest usage entry. `lib/context_tokens.py` printed **0** for this
  file (its "<2 in-scope entries → return 0" guard; only one usage entry existed at read time).
  Stop threshold by NAME: `TOKEN_WARN` = 75%.
- Inherited: T-3103 committed as `eb1364e94` by attempt 1 AND already parked by verb
  (Updates: 2026-09-28T23:29:12Z status started-work→captured, reason "Parked R2: 9 carded-unwatched
  files remain on two Sovereign scope questions"). T-3127 closed by attempt 1 (`c54b48002`).
  Attempt 1's handback was a 7/9-placeholder skeleton; the two SQs were never written up.

## Selection

**Unit 1 — dispose the inherited T-3103 (arc-008, Q1).** Already parked BY VERB by attempt 1
(Updates 23:29:12Z, reason cites two Sovereign scope questions) with the widening committed as
`eb1364e94`. AC1/AC2 remain unticked because the remaining 9 carded-unwatched files need two scope
rulings this task does not own. Activity this attempt: write those rulings up as SQ-A / SQ-B
(below) — the part attempt 1 never did. No re-work, no re-parking.

**Level-2 re-entry after arc-011 (R1) and arc-008 (attempt 1).** arc-008's agent-eligible
Q1/Q2 are all parked or negative-result-terminal (T-3132 AC3 not achievable per G-095; T-3103 SQ-A/B;
T-2958 2 ACs ANSWERED-NO; T-3130 1 AC NOT-REPRODUCED; T-3128 AC3 "not closable by this task");
T-3117 / T-3126 are `owner: human`. Ticking a negative-result AC to close is producer-as-judge, so
those three ride as SQ-C rather than being closed. Next arc by nearest-completion + in-flight:
**arc-009** (anchor T-2974, 24 completed / 25 active, headline = every non-KEEP value-review
finding reaches a terminal state the operator can see).

**Unit 2 — objective → arc → task → quadrant.**
- **Objective:** charter Directive #2 / arc-009 headline — every value-review finding reaches a
  visible terminal state; specifically PL-373's promoted-but-never-filed defect.
- **Arc:** arc-009 (in flight, not blocked).
- **Task:** T-3037 "File upstream: rail_project_label() guesses an identity instead of refusing" —
  the ONLY arc-009 agent-owned Q1 task with real ACs AND a score AND status started-work.
- **Quadrant:** Q1 hv-lc (BVP 57 proposed, cost 2.0 proposed).
- **Why over the next candidate:** T-2991 / T-2978 (arc-009, Q1) carry `[First criterion]`
  placeholder ACs — their scores were computed on template boilerplate (the T-3135 R1 calibration
  finding), so they are effectively unscored; T-3010 is a deliberate stays-open revisit sentinel
  (AC3 says so). T-3037's six ACs are concrete and local (a filing + a read-back), cost 2.0.

## Objectives advanced

_(pending)_

## Arc state (tasks by status and quadrant)

_(pending)_

## What remains in Q1/Q2, per task, with reason not done

_(pending)_

## Sovereign questions raised (unresolved, priority order)

_(pending)_

## Gates that refused, and what was done instead

_(pending)_

## Cost-vs-estimate deltas

_(pending)_

## Checks recorded (traceability ledger)

| # | check | result |
|---|---|---|
| 1 | own-transcript usage entry at start | cache_read 172,619 (~22%) |
| 2 | `git log` + T-3103 Updates re-read | T-3103 parked by verb 23:29:12Z; `eb1364e94` in HEAD |
