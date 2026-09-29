# PROMPT — procAsFit (autonomous mandate)

Mandate

Proceed autonomously. Select your own work, execute it, and keep going until a stop
condition fires. You are not waiting for instruction between units of work — you are
waiting only for a Sovereign decision when one is genuinely required.

Framework governance applies to this run in full. AEF governs its own development; no
exemption applies because the work is autonomous.

Selection — what to work on

Work is selected top-down. Each level is a gate on the level below it.

Project. Start from the project goals and objectives. Anything that does not advance
them is not eligible, however tractable it looks.
Arc. Among eligible work, pick the arc whose completion moves a project objective
furthest. Prefer an arc already in flight over opening a new one, unless the in-flight
arc is blocked.
Task. Within the chosen arc, select by BVP quadrant:
Q1 — high value / low cost: work first, to exhaustion.
Q2 — high value / high cost: work second.
Low-value tasks are out of scope for this run regardless of how cheap they are. Leave
them scored and parked.
Activity. Within a task, do only the activities its acceptance criteria require. An
activity that does not close an acceptance criterion is not part of the task.

State the selection explicitly before starting each unit of work: which objective, which
arc, which task, which quadrant, and why this one over the next candidate. Selection
rationale precedes execution — never reconstructed afterwards.

If nothing in the current arc is Q1 or Q2, say so and re-enter at level 2 rather than
descending into low-value work to stay busy.

Governance bindings
Verb gates only. All state changes go through fw verbs. No direct writes to focus.yaml,
arc-focus.yaml, or .next-directive.yaml. A gate that refuses you is a finding to be
recorded, not an obstacle to route around.
Producer-not-judge. You do not certify your own output. A task closes when its
acceptance criteria are independently checkable and checked — not when you judge the
work adequate. Do not adjust BVP calibration parameters or rescore your own completed
work upward.
Research is not authorization. Discovery is read-only. Findings do not ratify anything.
Sovereign questions are surfaced, not resolved. Anything requiring an architectural,
scope, or priority decision that is not already settled: write it as a Sovereign
question, park the task, move to the next. Do not decide it to keep momentum.
One lock at a time. Do not open a second structural change while the first is ungated.
Reliable-but-ungated is the dangerous state.
Scored before started. No task is executed before it has a BVP score. If an unscored
task is the obvious next move, score it first through the scorer, not by estimate.

TermLink

Use TermLink where it is the right instrument, not decoratively:

Dispatch BVP estimation to the bvp-estimator worker rather than scoring inline.
Run independent tasks concurrently where they touch disjoint paths; serialize anything
touching shared state.
Carry the run record on it so state survives a context reset.

If TermLink is unavailable, or using it would obscure the audit trail, work directly and
record why.

Execution loop

Per unit of work:

State the selection (objective → arc → task → quadrant) and the rationale.
Execute the activities the acceptance criteria require.
Run the check that closes each criterion. Record result, pass or fail.
Close or park the task through the proper verb.
Log: what changed, what it cost against estimate, what it surfaced.

If a task fails its acceptance criteria twice, stop working it, record the failure mode,
and move on. Three attempts at the same wall is context burned, not progress.

Stop conditions

Stop at the first of:

All Q1 and Q2 tasks in the active arc are complete, and no other arc has eligible Q1/Q2
work, or
context reaches the TOKEN_WARN threshold (75% of CONTEXT_WINDOW — read the live value
with `.agentic-framework/agents/context/checkpoint.sh status`, never a remembered
number), or
a Sovereign question blocks every remaining eligible path.

Do not stop mid-task. Close or park the current task, then write the handback.

> THRESHOLD BY NAME, NOT BY NUMBER (T-3192). Earlier revisions of this prompt carried a
> literal figure and both were wrong. "~300k" was never a threshold at all — it is
> `budget-gate.sh`'s DEFAULT window size quoted as if it were a budget, and against this
> project's 800000 window it is 37%, below even the first warning, so the run stopped
> before doing any work. "~800k" is wrong in the opposite direction: it IS the window, so
> it cannot fire before `TOKEN_CRITICAL` (95% = 760000) hard-blocks the worker — mid-task,
> which the clause directly above forbids. A literal number in a mandate goes stale
> silently. Name the threshold; it auto-scales and cannot rot.

Handback
Objectives advanced, and by how much — against the state at run start.
Arc state: tasks by status and quadrant.
What remains in Q1/Q2, per task, with the reason it was not done.
Sovereign questions raised, unresolved, in priority order.
Gates that refused you, and what you did instead.
Cost-vs-estimate deltas worth feeding back into calibration.
Auditability

This run will be reviewed against the bindings above. Every claim in the handback must
be traceable to a recorded check or a verb-gated state change. An assertion that
something works, without the check that demonstrates it, counts as an open task and not
a closed one.

---

## ORCHESTRATOR CONTEXT — round 3 of 9 (T-3211)

You are ONE round in a sequence. Rounds are serialized; no sibling is running.

### Carried fixes — these cost earlier runs a whole round each. Non-negotiable.

1. NON-INTERACTIVE WORKER. You are `claude -p`. Your turn end IS your process
   end. NEVER background a long command and end your turn. T-3089 R4-attempt-1
   did exactly that, exited 0, reported 'complete', and no deliverable was ever
   written.
2. AT-THE-MOMENT RE-READ. Before any shared or live-infrastructure action,
   re-read the run record AND `git log` right then. T-3089 R3 restarted the
   shared hub 82s after the operator had ruled to defer it, because its
   information was stale rather than absent.
3. WRITE THE HANDBACK EARLY as a skeleton and fill it as you go. A handback
   written only at the end is a handback you may never write.
4. BUDGET READ — DO NOT USE checkpoint.sh, IT WILL LIE TO YOU (T-3212).
   Both documented options report ANOTHER session's figure to a dispatched
   worker. `.context/working/.budget-status` is a single shared path (T-3127),
   and `checkpoint.sh status` — the remedy CLAUDE.md prescribes for exactly
   that problem — picks the GLOBALLY-NEWEST transcript (checkpoint.sh:79), which
   in a dispatched run is the orchestrator, not you.

   MEASURED: round 2 was told 582,524 (~72%) when its own usage was 162,629
   (~20%). A worker that believes it is at 72% is one point from TOKEN_WARN and
   stops almost immediately. R2 ran 479 seconds against a mandate to work until
   a stop condition fires. Read your budget wrong and you end the round, not the
   task.

   Read YOUR OWN transcript instead — resolve your session id, then:
     python3 .agentic-framework/lib/context_tokens.py <your-own-transcript.jsonl>
   Your transcript is the one whose recent entries are YOUR turns; confirm that
   before trusting the number. TOKEN_WARN is 75% of CONTEXT_WINDOW (800000).

### Your handback

Write it to EXACTLY this path, in markdown, following the Handback section of
the mandate above:

    /opt/termlink/.context/runs/T-3211-R3-handback.md

It is the only thing the next round receives. A claim in it that is not
traceable to a recorded check or a verb-gated state change counts as an open
task, not a closed one.

### Previous round's handback (round 2)

This is the state you inherit. Do not redo its completed work; do pick up
its unfinished Q1/Q2 items and its unresolved Sovereign questions.

```markdown
# T-3211 — procAsFit round 2 of 9 — handback — ATTEMPT 1 (superseded; YIELDED, not crashed)

**Status: YIELDED at 01:36 local.** This worker (transcript `0d28539b…jsonl`, PID chain under dispatch
`D-3547765`, started 01:20:44) was alive and working the whole time. The orchestrator's wait declared it
`crashed_workers: [procasfit-r2-1]` at **59.5 s** (`T-3211-R2-dispatch.log`), re-dispatched round 2 as
**attempt 2** at 01:33:28 (PID 36066, dispatch `D-34604`), and renamed this worker's skeleton to
`T-3211-R2-attempt1-skeleton.md`. Attempt 2 was confirmed live and progressing (its transcript
`8ff0310c…jsonl` written 01:35:54). Two workers sharing `focus.yaml`, the task register and the git index
is the shared-state race the mandate forbids, so attempt 1 stopped **after** restoring shared state to
what attempt 2 had censused. The canonical `T-3211-R2-handback.md` belongs to attempt 2 and was not
touched. Everything below is traceable to a commit or a verb-gated transition.

## Baseline at run start

- HEAD `bcc8905bb`; focus T-3211. Own budget **162,629 tokens (~20%)** via `lib/context_tokens.py` on the
  own transcript. `checkpoint.sh status` reported **582,524 (~72%)** — a cross-read (finding F1 below).
- Inherited from R1: T-3103 started but UNCOMMITTED and at `captured`; R1 handback had "What remains" and
  "Sovereign questions" unfilled.

## Selection

- **Unit 1** — Objective: clean `fw audit` (arc-008 headline mechanic). Arc: arc-008 (in flight, 83%
  closed, not blocked; arc-010 exhausted by R1). Task **T-3103** Q1 hv-lc (BVP 63 / cost 2.0) — finishing
  an already-paid-for unit beats opening one. Next candidate T-3132 (Q1 85) is parked by R1 with its last
  AC recorded not achievable.
- **Unit 2** — same objective/arc. Task **T-3127** Q1 hv-lc (BVP 57 / cost 3.2): spot-check of R1's
  "vendored class, filed upstream" showed all 5 Agent ACs ticked and `fw task verify` 3/3 PASS since
  09-25; only the T-1718 Evolution gate stood between it and `completed/`.
- **Unit 3 (not executed — yielded)** — level-2 re-entry into arc-009 (in flight). Task **T-3037** Q1
  hv-lc (BVP 57 / cost 2.0, real ACs). Chosen over T-2991 (60) and T-2978 (57), which still carry
  `[First criterion]` placeholders (G-020 territory), and T-3010, which is intentionally-open by its own
  AC. T-3037's body parks itself on "cannot be scored before started" — that premise is stale: the
  2026-09-27 estimator pass (T-3189) scored it, so starting it decides no Sovereign question.

## Objectives advanced

1. **T-3103 — committed `eb1364e94`, then PARKED by verb** (`fw task update --status captured`, Updates
   entry 23:29:12Z). `.context/cron/*.crontab` glob added to `.fabric/watch-patterns.yaml`; 31
   cron-declaration cards registered (30 new + `canary-aliveness-sweep` rewritten from `TODO`).
   Measured effect: `fw audit` WARN "Fabric: N card(s) point at files no watch pattern covers" **10 → 9**
   (this round's audit output line 38; the 12-run history shows 10). P-011 rehearsal of its 4
   Verification lines under `bash -c 'set -eo pipefail; …'`: 5/5 PASS. AC1/AC2 remain unticked on
   purpose — the 9 remaining files are SQ-1 and SQ-2, not work.
2. **T-3127 — CLOSED by verb** (`fw task update T-3127 --status work-completed`): P-010 5/5, P-011 3/3,
   episodic `.context/episodic/T-3127.yaml` generated and `yaml.safe_load`s to a mapping (T-2805 class
   clean). Commit `c54b48002` (via the gate's own `FW_SWITCH_FOCUS=1` path, see Gates #3). Evolution entry
   records finding F1.
3. **T-3037 — started and returned to `captured` by verb, no work under it** (Updates 23:34:28Z →
   23:36Z, reason states the yield). Attempt 2 inherits it exactly as censused, plus the fact that its
   park reason is stale.

## Arc state (agent-owned Q1/Q2, measured this round)

- **arc-010:** 0 active tasks (unchanged from R1's close of T-3135). Exhausted.
- **arc-008:** T-3127 **closed**. T-3103 **parked** (SQ-1/SQ-2). T-3132 (Q1 85), T-3128 (Q2 85),
  T-2958 (Q1 57), T-3130 (Q1 57): each has ALL other ACs ticked and ONE AC recorded as *not achievable /
  not closable by this task / answered NO / not reproduced*. No agent activity can tick those; they need
  SQ-3. Agent-owned Q1/Q2 in arc-008 is therefore **exhausted pending SQ-3**.
- **arc-009:** T-3037 (Q1 57) READY — real ACs, scored, `captured`. T-2991 (Q1 60, test) and T-2978
  (Q1 57) have placeholder ACs — need scoping before G-020 lets work start; T-2978 also relaunches live
  agents (shared infrastructure — operator ruling advisable before an autonomous worker does it).
  T-3010 (Q1 57) intentionally open (a revisit_at carrier), not eligible.
- Q1 tasks in NO arc (unreachable by the project→arc→task ladder, R1's finding, still true): T-2644 (78),
  T-2573 (76), T-2606 / T-2656 / T-2662 (73), T-2616 (66, design), T-2886 (60), T-2911 (57), T-2581 (57),
  T-3091 (57), T-3139 / T-3141 (57, started-work). Human-owned Q1/Q2 not eligible for an agent worker.

## What remains in Q1/Q2, per task, with reason not done

| task | Q | reason not done this round |
|---|---|---|
| T-3037 | Q1 | selected; yielded to attempt 2 before executing (sibling race). Ready to run. |
| T-3103 | Q1 | remaining 9 files are SQ-1 + SQ-2 |
| T-3132, T-3128, T-2958, T-3130 | Q1/Q2 | one AC each recorded unachievable → SQ-3 |
| T-2991, T-2978 | Q1 | placeholder ACs; T-2978 also live-infra |
| T-3010 | Q1 | intentionally open by its own AC |
| arc-less Q1 list above | Q1 | no arc → not reachable under the mandate's selection ladder (SQ-4) |

## Sovereign questions raised (unresolved, priority order)

1. **SQ-3 — How is a task closed whose last AC is honestly recorded as unachievable / answered-NO?** Four
   arc-008 tasks (T-3132, T-3128, T-2958, T-3130; combined BVP 85+85+57+57) are frozen this way. Options:
   (a) the human strikes/rewords the AC and the agent closes by verb; (b) a `fw task update --wontfix`
   style verb with logged rationale; (c) they stay open as the register's honest record. Producer-not-judge
   forbids an agent from rewording its own AC to pass.
2. **SQ-1 — The 50 fabric cards on 8 vendored BPMN files** (`.agentic-framework/.context/designer/projects/*/v*.bpmn`,
   `created_by: T-2839`, under a gitignored vendored tree). Keep them and add a
   `.agentic-framework/.context/designer/**/*.bpmn` glob (fabric then watches a tree a re-vendor rewrites),
   or delete a deliberate 50-card deliverable. Either way the audit WARN drops 9 → 1. Note the task-ID
   collision: local T-2839 is an unrelated broadcast task (T-2800 class).
3. **SQ-2 — `.claude/commands/*.md`**: 34 slash commands, exactly one (`capture.md`) carded with a real
   edge. Register all 34 (33 new cards) or drop the singleton. Hand-shaping a glob around one file is the
   fabrication `watch-patterns.yaml` itself warns against. Resolves the last carded-unwatched file.
4. **SQ-4 (carried from R1) — the highest-BVP agent-owned Q1 tasks belong to no arc** (T-2644 78, T-2573
   76, T-2606/T-2656/T-2662 73 …). Assign them an arc, open a "defect remediation" arc, or accept that the
   ladder never reaches them. T-2573 additionally contains a decide-the-wire-contract AC that is itself
   sovereign.
5. **SQ-5 (from T-3037's body, priority lowered)** — "scored before started" vs the P-002 gate that refuses
   `fw bvp estimate` on a `captured` task. Still true in general; no longer blocks T-3037 because the
   blanket T-3189 estimator pass scored it from outside.

## Gates that refused, and what was done instead

1. **P-002 / check-active-task** BLOCKED a census command (redirect to `/tmp`) because focus was still
   on the just-parked `captured` T-3103. Correct refusal. Did: `fw context focus T-3211` (verb), re-ran.
2. **T-1730 focus-drift gate** BLOCKED `git add`/commit of T-3127's close while focus was T-3211
   (a completed task cannot hold focus). Correct refusal; the gate names the sanctioned path. Did: prefixed
   `FW_SWITCH_FOCUS=1` per command (logged Tier 2 in `.gate-bypass-log.yaml`). A first try with
   `export FW_SWITCH_FOCUS=1; …` was ALSO refused — the gate matches the literal prefix form only.
3. **Orchestrator supersession** (not a fw gate): declared this worker crashed at 59.5 s and re-dispatched.
   Did: yielded (this document) rather than race the successor.

## Findings for the operator (not decided here)

- **F1 — `checkpoint.sh status` cross-reads under concurrent dispatch.** `find_transcript` (checkpoint.sh:79-86)
  picks the globally-newest `*.jsonl`, so the command carried fix 4 recommends reported 582,524 tokens for a
  worker whose own transcript held 162,629. T-3127 keyed the WRITER (`budget-gate.sh`); the READER has the
  sibling defect. One bug = one task — needs its own task (vendored → file upstream, G-062). The carry text
  for attempt 2 already states this correctly.
- **F2 — the dispatch harness cannot distinguish "worker crashed" from "waiting shell died" and
  re-dispatches onto a live worker.** `T-3211-R2-dispatch.log`: `crashed_workers: [procasfit-r2-1]`,
  `elapsed_secs: 59.5`, while this worker ran 16 more minutes and made two commits. Commit `9117241be`
  ("launch rounds DETACHED — the waiting shell was what kept dying") diagnoses the shell half; the
  re-dispatch half means "Rounds are serialized; no sibling is running" was false for ~3 minutes with three
  `claude -p` procAsFit processes alive (R1 worker PID 2021865 started 00:47:47 was STILL alive at 01:36,
  48 min after writing its handback; attempt 1; attempt 2). A re-dispatch should first check the prior
  worker's PID/transcript liveness, or the prior worker should be killed before the successor starts.
- **F3 — R1's process did not exit after its handback** (see F2). Whether it is idle-blocked or still
  mutating state is unknown; nothing in `git log` after 01:14 is attributed to it.

## Cost-vs-estimate deltas

- **T-3103** (est. cost 2.0): the crontab class was ~one unit of R1 plus a commit/park here; the OTHER
  nine-tenths of the AC is two scope decisions. Estimator read "10 files" as one homogeneous fix; it was
  three causes (PL-386 shape). Calibration: audit-count tasks should be costed per CAUSE, not per count.
- **T-3127** (est. cost 3.2): zero build cost this round — the whole remaining cost was a missing Evolution
  paragraph. Tasks sitting fully-ticked in `started-work` are the cheapest Q1 in the register; a
  `fw task verify` sweep of started-work tasks would find them (T-3132's CTL-029 bundle was that sweep and
  hit SQ-3).

## Checks recorded (traceability ledger)

| # | check | result |
|---|---|---|
| 1 | `context_tokens.py` on own transcript at start | 162,629 (~20%) |
| 2 | `checkpoint.sh status` at start | 582,524 (~72%) — cross-read, F1 |
| 3 | `fw work-on T-3103` | captured → started-work, focus set |
| 4 | audit `carded_unwatched` replica (audit.sh:2213 logic) | 51 cards / **9 distinct files** (8 BPMN + capture.md); 6 unregistered |
| 5 | P-011 rehearsal T-3103, `bash -c 'set -eo pipefail; …'` ×5 | 5/5 PASS |
| 6 | `fw fabric drift` | unregistered 6, orphaned 0, stale 0 |
| 7 | `fw git commit` T-3103 | `eb1364e94` (33 files) |
| 8 | `fw task update T-3103 --status captured` | started-work → captured (parked) |
| 9 | P-002 gate on census with focus=T-3103 | BLOCKED → `fw context focus T-3211` |
| 10 | `fw task verify T-3127` | 3/3 PASS |
| 11 | `fw task update T-3127 --status work-completed` | completed; episodic generated |
| 12 | `yaml.safe_load(.context/episodic/T-3127.yaml)` | mapping |
| 13 | T-1730 gate on T-3127 commit ×2 | BLOCKED → `FW_SWITCH_FOCUS=1` prefix form → `c54b48002` |
| 14 | `fw audit` (backgrounded, completed) | "Fabric: 9 card(s) point at files no watch pattern covers" (was 10) |
| 15 | `fw work-on T-3037` | captured → started-work |
| 16 | ps / transcript mtime of attempt 2 | PID 36066 alive; `8ff0310c…jsonl` written 01:35:54 |
| 17 | `fw task update T-3037 --status captured` | started-work → captured (yield) |
| 18 | `fw context focus T-3211` | focus restored |
```
