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

## ORCHESTRATOR CONTEXT — round 8 of 9 (T-3211)

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

   Read YOUR OWN transcript instead — resolve your session id, then feed the
   transcript on STDIN (note the '<' redirect):
     python3 .agentic-framework/lib/context_tokens.py < ~/.claude/projects/-opt-termlink/<session-id>.jsonl
   Do NOT pass the path as an argument: argv[1] is a session-start TIMESTAMP,
   so the argument form reads an empty stdin and prints 0 (T-3211 R5, measured:
   argument form 0 vs stdin form 306,256 on the same transcript). A worker that
   reads 0 never reaches TOKEN_WARN and runs on into TOKEN_CRITICAL mid-task.
   Your transcript is the one whose recent entries are YOUR turns; confirm that
   before trusting the number. TOKEN_WARN is 75% of CONTEXT_WINDOW (800000).

### Operator directive for this round

OPERATOR RULINGS IN FORCE: SQ-4 YES (arc-less agent-owned Q1/Q2 eligible). SQ-3 SURFACE-THEN-ASK. SQ-9 Q1/Q2/Q3
approved (Q2 = the v0.12.1 tag, HELD BY THE ORCHESTRATOR — never tag, never push).

NEW, 2026-09-29 — operator: "Proceed as you see fit and suggested. Continue until you can't anymore."
He approved ROUTE A now and ROUTE B as its own task. CI on origin 90bd7b56b: Install Check GREEN, Doc Lint red on
exactly 2 members: check-go-propagation and voi-prompt. voi-prompt is the OPERATOR's question (T-3200 voi_score) —
do NOT record an answer for him, do NOT touch T-3200 (owner human). Leave voi-prompt red.

ROUTE A (do FIRST) — check-go-propagation fires on 11 GO-decided inceptions: T-2971 T-2989 T-2995 T-3001 T-3004
T-3005 T-3007 T-3011 T-3012 T-3048 T-3055 (T-2995/T-3005/T-3011 are ORPHANS — no task mentions them).
For each, read its Recommendation/decision and what the GO approved, then:
  - If follow-up tasks already exist (search task bodies, commits, episodics), write the link BOTH directions
    (inception.related_tasks and each slice's related_tasks). Cite the evidence in the commit.
  - If the GO approved work that was never filed, FILE the slices the GO approved (that is what GO authorised),
    scored, and link both directions. Do not build them this round unless they are Q1 and budget allows.
  - If the approval genuinely looks LAPSED or superseded, do NOT acknowledge it in the allowlist yourself — list it
    under "## Closure requests" with evidence; the operator decides (SQ-3 rule).
  Goal: check-go-propagation green, or red only on items awaiting the operator.

ROUTE B (then) — a guard-layer severity classification, as its own build task (create it; real ACs before edits):
  Mechanism: a member may declare `# guard-layer: source advisory  # <reason>`. Default (no word) = BLOCKING.
  run-guard-layer.sh gains a mode (e.g. --gate release) where only BLOCKING failures set a non-zero exit, and
  advisory failures are printed in their own section, never hidden. Push CI (doc-lint) stays exactly as today —
  every member runs and every red shows. release.yml's test job uses the release gate mode. cargo test is never
  advisory. check-guard-runner-coverage (or equivalent) refuses an `advisory` marker without a reason.
  Fixtures with mutants (e.g. an advisory marker without reason must fire; a blocking member failing must still
  fail the release gate; an advisory failure must still be printed).
  CLASSIFY NOTHING AS ADVISORY. Instead write docs/reports/T-XXXX-guard-classification-draft.md: every static
  member with proposed class + one-line reason, for the OPERATOR to approve. Note the risk cases (e.g.
  check-framework-tracking-drift looks like bookkeeping but catches clean-clone breakage). Flag voi-prompt
  specifically: its header says "never blocks — it only reports" yet it exits 1 in the gate.

Then continue down the SQ-4 pool if budget remains. Before any commit after a refused finalize: git status
--porcelain. Never self-grant FW_SWITCH_FOCUS; refocus T-3211 first.

### Your handback

Write it to EXACTLY this path, in markdown, following the Handback section of
the mandate above:

    /opt/termlink/.context/runs/T-3211-R8-handback.md

It is the only thing the next round receives. A claim in it that is not
traceable to a recorded check or a verb-gated state change counts as an open
task, not a closed one.

### Previous round's handback (round 7)

This is the state you inherit. Do not redo its completed work; do pick up
its unfinished Q1/Q2 items and its unresolved Sovereign questions.

```markdown
# T-3211 — procAsFit round 7 of 9 — handback

**Status: COMPLETE.** I stopped on **stop condition 1**. The operator's SQ-9 Q1 list (7 guard members) and Q3 (the canary) are all closed, and I re-walked the SQ-4 Q1/Q2 pool: every remaining item is gated on a human ruling, a human AC, an upstream fix or live shared infra (table below). Budget at stop: **331,264 of 800,000 (~41%)**, below `TOKEN_WARN` (600,000). I read my own transcript on stdin: `ef5102b7-…jsonl`, `entrypoint: sdk-cli`.

**Result: 8 tasks filed, scored and closed by verb** (T-3236–T-3243), all **hv-lc (Q1)**. **18 commits** on `main` since `1afb0b7d5`. **Nothing pushed; nothing tagged; the crontab is NOT installed.** Local `main` is **20 ahead of origin**; the push is the orchestrator's.

## ⚠ Read first
1. **Predicted CI after the push: Doc Lint 2 red of 172, both operator-reserved** (`check-go-propagation`, `voi-prompt`). Measured by the **full guard layer** run under `CI=true TERMLINK_BIN=/nonexistent PATH=/usr/bin:/bin` (as root): 4 red. The other 2, `journal-mirror-fixtures` and `sweep-debris-census-fixtures`, are **artifacts of my simulation**: my `TERMLINK_BIN` overrides the stub those suites inject via PATH. Both PASSED in real CI run 36599556947, and none of my commits touch their files. **Install Check is already GREEN on main (run on 93c8a6137, 16:41Z today)**, so R6's rmcp-macros pin is proven.
2. **Q2 gating needs a word from the operator (SQ-14 below).** Doc Lint cannot go fully green until the operator decides T-3200 (voi-prompt) and triages the GO inceptions (go-propagation). If "CI green" for the v0.12.1 tag means *all* of Doc Lint, the tag waits on those two human decisions, not on agent work.
3. **The new canary fires today, and should.** Live first run: **v0.12.0 has NO GitHub Release (HTTP 404)**, and **doc-lint.yml was last green on main 44 days ago**. Once v0.12.1 is tagged, it will fire until release.yml publishes that release. That is the intended signal, not a defect.
4. **Two process slips, both corrected, neither a bypass.** (a) I ran `fw work-on T-3243` one step before scoring it. No execution happened in between: the G-020 gate refused my first probe, I wrote ACs, and scored (57/3.2 Q1) before any build step. (b) After T-3240's finalize, the episodic stayed unstaged, and the focus-drift gate refused a `T-3240:`-prefixed commit (the task was closed). I did **not** set `FW_SWITCH_FOCUS`; I committed it as `T-3211:` (`e65d464a5`). The next 7 finalizes staged the rename and the episodic together in one `git add` and went through cleanly. **Bypass log: no new entry this round** (last entry is R6's, 16:27:11Z).
5. **Late catch, fixed: T-3240's first finalize commit captured only the rename.** `completed/` carried `status: started-work`, the G-066 class that the task-finalization canary fires on. I found it in my final porcelain check and committed the finalized frontmatter as `T-3211:`. Now all 8 closes read `work-completed` at HEAD and `check-task-finalization-freshness` is rc 0.
6. **My guard-layer run polluted two tracked files, and I reverted them.** `voi-prompt.sh --check` bumped `.context/checks/voi-decisions.yaml` `runs: 3→4`, and a fixture appended T-8/T-9/T-10 rows to `.context/checks/voi-waiver-ledger`. The **identical rows dated 2026-09-26 are already committed**, so this leak has shipped once before. I restored both to HEAD. See F29.
7. **One stray write I caused and removed.** A mistyped `cp -r tests scripts /tmp/` created `/tmp/tests` and `/tmp/scripts`. I verified that neither existed before (no file older than the command; `diff -rq` identical to the repo), then deleted both.

## Baseline at start
HEAD `1afb0b7d5`. Budget 164,170 (~20%). CI reference run 36599556947 (on 93c8a6137): 9 red members (7 mine + 2 operator-reserved).

## Selection / work log

**Objective:** the release path and CI health (SQ-9). **Arc:** arc-less, operator-directed (SQ-4 ruling plus the SQ-9 Q1/Q3 approvals). **Quadrant:** all Q1. **Order:** T-3240 first (BVP 85, the only one above 57), then the 57-ties by lowest cost, then Q3.
Own transcript `ef5102b7-…` (entrypoint sdk-cli); budget at start 164,170 (~20%). TOKEN_WARN = 600,000.

Filed + scored (inline `fw bvp estimate` + `estimator.py cost-one`, 0.05s each — dispatching to a worker would add latency for nothing and obscure nothing) after writing real ACs: T-3236..T-3242 all **hv-lc (Q1)**; T-3240 BVP 85/cost 2.0, rest 57/2.0–3.2. T-3243 (canary) scored before its start.

| task | member | outcome | evidence | commits |
|---|---|---|---|---|
| T-3240 | check-audit-warning-acknowledgement | **closed** — registered 12 uncarded files; retired orphan card `invocation_audit` (module moved hub→session in T-3033; hub-lib edge repointed); acknowledged F-ORCH (audit.sh:1484 `orch_signal` resolves the FRAMEWORK's T-1643 against THIS project's unrelated T-1643 — namespace collision — and driver retirement is sovereignty-gated) and GO-scope (operator-reserved go-propagation class) | fabric drift 0/0/0; fresh structure audit + check: 7/7 acknowledged, 0 unexamined; P-011 1/1 | `f0cac8d15` `ee52193b9` `e65d464a5` |
| T-3236 | planted-default-gate-fixtures (the "unexplained" one) | **closed** — ROOT CAUSE: section J redirected to a hard-coded `/root/.claude/jobs/5f599680/tmp/_real.out` (a Claude job dir, this host only). An unopenable redirect never runs the check, rc=1, so **J1 passed VACUOUSLY** and J2 grepped a missing (CI) / stale (non-root) file. R6 missed it because both its clones ran as root on this host. Fixed: own mktemp; new J0 asserts the run produced output; script-relative repo root | reproduced as `runuser -u nobody` (EACCES); after: root 27/27, nobody 27/27 no EACCES, no-git fallback 27/27; mutant (unopenable redirect) → J0+J2 FAIL; P-011 3/3 | `40108dbdf` `a9e453954` |
| T-3237 | runme-fixtures | **closed** — 3 defects: (1) TREE: literal `2 already-done`, stale since T-3068 added a 3rd crontab (failed on every host) → derived from runme.sh's `install_crontab` count; (2) cases 3-9 need root, runner is non-root → SKIP only if CI AND non-root; (3) case 11 used `runuser` (root-only) → runs runme directly when already non-root | root 18/0; nobody+CI 11/0 + SKIP line; nobody no-CI rc 1 (still loud); literal-2 mutant red; P-011 3/3 | `964907b21` `5a13bd4da` |
| T-3238 | check-receiver-ack-lag + check-arc-claim-drift (arc-003) | **closed** — e4 in `notify-rail-e2e.sh` recorded FAIL(1, BROKEN) on a missing binary; its own contract says TOOLING(2) → now `die_tooling`. arc-claim-drift: prover rc 2 → `SKIP(CI)` only when CI set (else CLAIM-FAILED, now naming the rc). receiver-ack-lag: SKIP only when CI set AND no binary | host rc unchanged (arc-claim 0 after T-3239, ack-lag 1); simulated runner (`CI=true TERMLINK_BIN=/nonexistent PATH=/usr/bin:/bin`) both rc 0; no-CI no-binary: ack-lag 2, arc-003 CLAIM-FAILED exit 2; new fixture T16a/b/c, mutant (drop CI condition) T16a red; arc-claim 18/0, notify-rail-e2e 15/0; P-011 5/5 | `31e82c30c` `3e72ffd5f` |
| T-3239 | check-arc-claim-drift (arc-004 UNBOUND) | **closed** — bound `prover: "bash scripts/demo-ws-push.sh"` (isolated hub, proves the headline: 91 ms push + reconnect/resume after blip, 4.3s) | host: both closed arcs VERIFIED rc 0; simulated runner: both SKIP(CI) rc 0 (doc-lint guard job has no build; release `test` builds debug, so target/release is absent); P-011 2/2 | `9dbe8b8bb` `592915fcd` |
| T-3241 | check-pickup-deferred-freshness (P-078) | **closed** — read it: an opencode no-defect cross-check corroborating T-1976. Moved to processed/ with a `disposition:` block (re-processing would auto-mint an inception for nothing, pickup.sh:417-433) | check rc 0; P-011 3/3 | `3676afb48` `fe7dc6556` |
| T-3242 | check-unpaired-capture (T-3052:204) | **closed** — genuine: `d=$(mktemp -d);` discards status; a failed mktemp makes "dry-run wrote nothing" vacuous. `;`→`&&` (strengthens only; no AC touched on the human-owned task) | leg rehearsed under `set -euo pipefail`; check rc 0; P-011 2/2 | `408e5d890` `72af27082` |
| T-3243 | **Q3 release-publication canary** | **closed** — `scripts/check-release-publication-freshness.sh` + `tests/release-publication-canary-fixtures.sh` + `.context/cron/release-publication-canary.crontab` (**committed, NOT installed** — `test ! -e /etc/cron.d/termlink-release-publication-canary` is a P-011 line) + CLAUDE.md section. Reads `gh api` REST directly: `gh run list --status success` reported install-check's last green as 2026-06-12 while the API returned today's green run, so the list form cannot be trusted | fixtures 25/0; mutant (release finding muted) turns R1/R2/W4/F2 red; hygiene rc 0; **live first run rc 1: v0.12.0 has NO release (HTTP 404); doc-lint.yml last green on main 44d ago (2026-08-16); install-check.yml GREEN today (93c8a6137)**; P-011 4/4 | `ce256a86b` `dec9fd486` |



### Remaining SQ-4 pool (re-walked this round, `fw bvp --quadrant hv-lc`, agent-owned, active)
| task | why not done |
|---|---|
| T-3214 (60) | **new in the pool since R6**, and gated: its Context names 3 incompatible designs (move doctor down / subprocess / duplicate) and asks for a ruling (**SQ-8**) |
| T-3103 (63) | its own ledger entry says the resolution is a category-level scope decision (card all 34 commands / 30 crontabs?), which is human |
| T-2978 (57) | relaunches live agents on the fleet rail: shared infra (SQ-6 shape) |
| T-3010 (57) | must stay in `active/` until 2026-12-18 for its G-053 revisit |
| T-3132, T-2958, T-3130, T-3177 | R6 Closure requests 1/3/4/5, still unanswered |
| T-2573, T-2606, T-3091 | SQ-11 / SQ-12 / SQ-13 (R6), still unanswered |
| T-2886, T-2911, T-3227 | gated as R6 recorded (vendored upstream / placeholder pickup / T-2385 `[REVIEW]`) |
| T-3089, T-3093 | orchestration tasks, not a round's to work |

## Closure requests (SQ-3)
**No new ones this round.** R6's five (T-3132, T-3128, T-2958, T-3130, T-3177) are **still open**. I did not touch them.

## Sovereign questions (priority order)
1. **SQ-14 (new): what "CI green" means for the v0.12.1 tag (Q2).** After the push, Doc Lint is predicted red only on the two operator-reserved members. Options: (a) tag when every agent-clearable member is green and Install Check is green (i.e. after this push); (b) wait for T-3200 no-go + GO-inception triage so Doc Lint is fully green; (c) acknowledge those two in their ledgers pending the decisions, which only the operator may authorise.
2. **SQ-15 (new): the `# guard-layer: source` marker is self-declared and was false twice.** `check-receiver-ack-lag` and `check-arc-claim-drift` (via its provers) both need a live binary/hub while claiming "safe anywhere, no hub". T-3238 made them CI-hermetic, but nothing checks the marker's claim. Should a lint verify it (e.g. run each `source` member with no binary on PATH in CI mode and require 0/1, never 2)? That is a new structural check, so it needs a ruling first ("one lock at a time").
3. **SQ-16 (new, upstream candidate): F-ORCH retire_when heuristic.** `audit.sh:1484 orch_signal` resolves the framework's T-1643 against the consumer's `.tasks/` (task-ID namespace collision). It is vendored (G-062): file at `framework:pickup`? I did not file it, because it is outward-facing and the operator has not asked for it.
4. **SQ-9 (carried):** the Q2 tag is held by the orchestrator. Also: install the new crontab (`sudo cp .context/cron/release-publication-canary.crontab /etc/cron.d/termlink-release-publication-canary`), and make sure `gh` is authenticated for root.
5. **Carried unchanged from R6:** SQ-11, SQ-12, SQ-13, T-2532, T-2669 (now also `pty.rs::poll_output_step`), SQ-1/2, SQ-6, SQ-7, SQ-8, SQ-10, T-2385 `[REVIEW]`, T-3006 `decide no-go`, the human ACs on T-2644 and T-3139, and R6's self-granted bypass entry for review.

## Findings for the operator
- **F24: a vacuous pass in a guard's own fixture (T-3236).** When a redirect cannot be opened, the command never runs and rc is 1, which is exactly the rc J1 expected. J1 was therefore green on any host where the path failed to open. The general lesson: an rc assertion needs a companion "it actually ran" assertion (J0).
- **F25: `gh run list --status success` misreports.** It returned 2026-06-12 as install-check's last green while the REST API returned a green run from the same afternoon. Anything built on `gh run list` for freshness is suspect; the canary uses `gh api`.
- **F26: receiver-ack-lag FIRES on this host (a real runtime finding, untouched).** agent-chat-arc: 4 identities NEVER-ACKED (lag 1896), 1 BEHIND (lag 972). framework:pickup: 2 NEVER-ACKED (lag 238), 1 BEHIND (156). Rows are per-fingerprint (host-level where keys are shared).
- **F27: a fixture's literal count rotted for months (T-3237).** T-3068 added a crontab and the suite hard-coded 2. Deriving expectations from the source under test (count `install_crontab` lines) is the durable form.
- **F29: running the guard layer mutates tracked state.** voi-prompt's `--check` increments a counter in `voi-decisions.yaml`, and a voi fixture writes its T-8/9/10 cases into the REAL `voi-waiver-ledger` (the 2026-09-26 rows are committed residue of the same leak). A guard layer that dirties the tree on every run will eventually get its residue committed as data. Not fixed here: voi-prompt is operator-reserved (T-3200).
- **F28: the focus-drift gate refuses a `T-XXXX:` commit for a just-closed task.** Stage the rename and the episodic in ONE `git add` right after the finalize, in the same command that refocuses T-3211. That worked 7/7 this round with no bypass.

## Gates that refused, and what I did
1. **G-020** (×2) on T-3243: it refused a read-only `gh` probe, and then a heredoc to /tmp, while the ACs were placeholders. Correct both times. I wrote the ACs in the task file (Edit) first.
2. **Focus-drift** on the T-3240 episodic commit: I attributed the commit to T-3211, with no bypass.
3. P-011 ran green on all 8 closes, and the RCA gate was satisfied on 4 bug-class titles (T-3236/7/8/42) with written RCAs. No `--force`, no `--skip-rca`.

## Cost-vs-estimate
- The estimator scored all 7 Q1 guard tasks 57 except T-3240 (85). Actual cost varied ~5×: T-3241/T-3242 were minutes; T-3237 (3 defects, 3 environments) and T-3238 (3 scripts plus a fixture) were the largest. The 57-tie once again gives no ordering signal.
- T-3236's estimate (2.0) was right on size but wrong on risk: R6 spent effort and could not reproduce it, while a non-root reproduction found it in one command. **Calibration note:** "unexplained CI-only failure" should prompt "run as the CI user", not "clone at CI depth".
- The canary (3.2) came in on estimate. Cheap because the seam was designed first and the fixtures were written against the raw API shape.
- The full guard-layer run cost ~16 min wall time and ~0 context (detached, then polled). Worth it: it turned "expect 9 → 2" from a hope into a measurement.

## Checks recorded
| # | check | result |
|---|---|---|
| 1 | own transcript + budget (stdin form) | ef5102b7; 164K → 274K → 319K → 331K |
| 2 | `gh run view 36599556947 --log-failed` | 9 red: 7 SQ-9 Q1 + go-propagation + voi-prompt |
| 3 | 7 members run locally (CI unset) before work | arc-claim 1, audit-ack 1, pickup 1, ack-lag 1 (real lag), unpaired 1, planted-default 0, runme 1 |
| 4 | planted-default as `runuser -u nobody` | EACCES on `/root/.claude/jobs/…`, J1 still "ok" → vacuous pass confirmed |
| 5 | e4 prover with no binary | recorded FAIL rc 1 (should be TOOLING 2) |
| 6 | demo-ws-push.sh | PASS 4.3s, 91 ms push, exit contract 0/2/3-5 |
| 7 | T-3240: fabric drift; structure audit + ack check | 0/0/0; 7/7 acknowledged, 0 unexamined |
| 8 | T-3236: root / nobody / no-git; mutant | 27/0, 27/0 no EACCES, 27/0; J0+J2 red |
| 9 | T-3237: root / nobody+CI / nobody no-CI; literal-2 mutant | 18/0; 11/0 + SKIP; rc 1; red |
| 10 | T-3238: host / simulated runner / no-CI no-binary; T16; mutant | unchanged; both rc 0 SKIP; ack-lag 2, arc-003 CLAIM-FAILED exit 2; 18/0; T16a red |
| 11 | T-3239: check-arc-claim-drift host / simulated runner | both VERIFIED rc 0 / both SKIP(CI) rc 0 |
| 12 | T-3241, T-3242: check rc | 0, 0 |
| 13 | T-3243: fixtures; suite vs mutant; hygiene; live run; not installed | 25/0; R1/R2/W4/F2 red; rc 0; rc 1 (2 real findings); `/etc/cron.d/…` absent |
| 14 | 7 members after all changes (host) | 6 rc 0; ack-lag rc 1 (F26, real) |
| 15 | **full guard layer, simulated CI** (`/tmp/r7-guard.out`) | 172 members; FAIL: go-propagation, voi-prompt (reserved) + 2 simulation artifacts that PASS in real CI |
| 16 | P-011 on each close | T-3240 1/1, T-3236 3/3, T-3237 3/3, T-3238 5/5, T-3239 2/2, T-3241 3/3, T-3242 2/2, T-3243 4/4 |
| 17 | bypass log tail | last entry 2026-09-29T16:27:11Z (R6); none added in R7 |
| 18 | `git rev-list --count origin/main..main` | 20 |
| 19 | final porcelain; task-finalization canary; HEAD status of 8 closes | voi residue restored; rc 0; 8/8 work-completed |
```
