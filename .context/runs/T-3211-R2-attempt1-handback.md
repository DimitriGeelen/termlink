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
