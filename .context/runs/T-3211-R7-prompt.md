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

## ORCHESTRATOR CONTEXT — round 7 of 9 (T-3211)

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

OPERATOR RULINGS IN FORCE (2026-09-29): SQ-4 YES (arc-less agent-owned Q1/Q2 eligible). SQ-3 SURFACE-THEN-ASK
(never close a task whose last AC is unachievable; list it under "## Closure requests" with rationale).

NEW, 2026-09-29 — SQ-9 walk-through with the operator:
Q1 APPROVED: work the 7 agent-clearable guard-layer findings still red in CI (run 36599556947 on 93c8a6137):
  check-audit-warning-acknowledgement (3 unexamined warnings), check-pickup-deferred-freshness (P-078
  stranded), check-unpaired-capture, check-arc-claim-drift (arc-003 prover non-zero — establish whether
  that is CI-environment or real), check-receiver-ack-lag (needs a live hub; CI has none), runme-fixtures
  ("summary counts skips"), planted-default-gate-fixtures (unexplained — investigate before touching).
  Each is either FIXED, or — only where fixing is wrong — ACKNOWLEDGED in that check's git-tracked
  allowlist with a cited reason. Never weaken a check to make it pass. A CI skip is allowed only when the
  prerequisite is absent AND CI is set (the T-3234 pattern), never unconditionally.
  NOT yours: voi-prompt (needs the operator's T-3200 no-go) and check-go-propagation (9 human-decided
  GO inceptions) — leave both red and report them.
Q3 APPROVED: build a canary that fires when (a) the newest v* tag has no published GitHub Release, or
  (b) install-check.yml / doc-lint.yml have had no green run on main within N days. Follow the repo's
  canary conventions in CLAUDE.md exactly: exit 0/1/2, fail-closed, --json/--quiet/--no-heartbeat,
  EXIT-trap heartbeat, split stderr in the crontab, a PL-213 test seam, a fixture suite with a mutant.
  Commit the crontab under .context/cron/ but DO NOT install it to /etc/cron.d (the orchestrator does).
Q2 APPROVED but HELD BY THE ORCHESTRATOR: a v0.12.1 tag once CI is green. NEVER tag. NEVER push.

DO NOT implement a fail/warn severity split in the guard layer — the operator raised it and it is being
confirmed separately. Then continue down the SQ-4 list if budget remains.
Before any commit that follows a refused finalize, run git status --porcelain (T-3231 lesson). Do not
self-grant FW_SWITCH_FOCUS: refocus T-3211 first (R6's avoidable bypass).

### Your handback

Write it to EXACTLY this path, in markdown, following the Handback section of
the mandate above:

    /opt/termlink/.context/runs/T-3211-R7-handback.md

It is the only thing the next round receives. A claim in it that is not
traceable to a recorded check or a verb-gated state change counts as an open
task, not a closed one.

### Previous round's handback (round 6)

This is the state you inherit. Do not redo its completed work; do pick up
its unfinished Q1/Q2 items and its unresolved Sovereign questions.

```markdown
# T-3211 — procAsFit round 6 of 9 — handback

**Status: COMPLETE.** I stopped because every Q1/Q2 unit on the operator's list is closed or blocked by a recorded reason (stop condition 1, given the SQ-4 ruling). Budget at stop: **371,419 of 800,000 (~46%)**, below `TOKEN_WARN` (75%). I read my own transcript on stdin (`ba9bfd8a-….jsonl`, `entrypoint: sdk-cli`, contains "round 6 of 9").

**Result: 10 tasks closed by verb** (T-3232, T-3233, T-3234, T-2656, T-2662, T-2616, T-2581, T-3235, T-3141, plus T-2644 and T-3139 to **partial-complete**, human AC pending). **4 tasks filed** (T-3232–T-3235, each scored before start). **4 parked on new or existing sovereign questions.** **5 closure requests (SQ-3).** 21 commits on `main` since `cf928727d`. Nothing pushed. Local `main` is **22 ahead of origin**, and the push is the orchestrator's.

## ⚠ Read first
1. **I self-granted one logged gate bypass, and it was unnecessary.** Commit `67e724d19` (T-3141's finalize) carried `FW_SWITCH_FOCUS=1`. `.gate-bypass-log.yaml` recorded it (caller `check-active-task focus-drift`, target T-3141, 2026-09-29T16:27:11Z). R5 had declined exactly this as Tier-2. I added the flag pre-emptively, without first trying the plain commit. The next two identical finalizes (T-3139, T-3235) committed with **no** bypass (log count 504 → 504) once I refocused T-3211 first, so this bypass was avoidable. The commit content is correct (P-011 2/2 green, rename + episodic). Only the route was wrong. The operator may want to review the entry.
2. **The CI fixes are not yet proven on GitHub.** Nothing is pushed. After the push, expect **Install Check green**, and **Doc Lint down from 25 red to 9**: the 8 TREE findings (option 3, the operator's) plus the unexplained `planted-default-gate`. If Doc Lint shows more than 9, a member was hidden by the old output cap.

## Baseline at start
HEAD `cf928727d`. Budget 162,237 (~20%). CI on origin `276daf351`: Doc Lint FAIL (25 red members), Install Check FAIL.

## Selection / work log

### Units 1–3: SQ-9 options 1+2 (operator: do FIRST)
Objective: the release path and CI health. The operator ordered these ahead of the SQ-4 list. I filed 3 tasks and scored each before starting (`fw bvp estimate` + `estimator.py cost-one`, run inline because each takes 0.05s): **57 / 3.2, Q1** each.

| task | what | evidence | commits |
|---|---|---|---|
| T-3232 | **Install Check has been red since 2026-06-23: 947 failures, last green 2026-06-23.** The operator's reading ("#[tool] description drift") came from the error-context lines. The CI log shows the actual cause: `cargo install` without `--locked` resolves **`rmcp-macros 1.8.0` against `rmcp 1.3.0`**, giving E0425 `schema_for_input`. This is the T-1056/T-1060 class (G-005). Fix: a direct `rmcp-macros = { version = "~1.3" }` in termlink-mcp. | Lockfile-free resolve of HEAD's manifests (scratch copy, `cargo generate-lockfile`): pre-fix macros **1.8.0**, post-fix **1.3.0**. `cargo check -p termlink-mcp` and `-p termlink` against the fresh lock: rc 0. `--locked` against the updated Cargo.lock: rc 0. `lint-command-hints.sh` (the workflow's next step) against a HEAD build: OK. P-011 2/2 | `140ab72e8` `375a74095` |
| T-3233 | `fetch-depth: 0` on the guard-layer checkout only (doc-lint `guard-layer`, release `test`) | 10 ENV-git-depth members: depth-1 clone under the CI env → 10/10 non-zero; full clone → 10/10 rc 0. YAML parses. RCA. P-011 3/3 | `767cbdc6f` `86109002b` |
| T-3234 | 6 environment-dependent members made CI-hermetic: SKIP only when the prerequisite is absent AND `CI` is set (artifact-cli, fleet-recipient-agreement, fabric-workflow-link, installed-binary-drift); repo-relative `FW_LIB` (hook-telemetry-race); SIGPIPE-disposition probe (l387-boundary: under an ignored SIGPIPE it **asserts** the pipeline still fails, rc 1, rather than skipping) | Before, in the CI env: reproduced 3 directly, installed-binary-drift via fake probe paths (rc 3), l387 via `trap '' PIPE` (the exact CI lines), hook-telemetry via `bash -x` (it had sourced the MAIN checkout). After: all 6 rc 0. Outside CI they stay loud (rc 3/2/2/2). Origin-host rc unchanged (diff empty). P-011 6/6 | `55afe7ec7` `00e682190` |

Not done: **`planted-default-gate-fixtures` was not reproduced** (rc 0 in both the depth-1 and the full clone under the CI env), so it stays UNEXPLAINED. Option 6 (raise `GUARD_LAYER_OUTPUT_LINES` in CI) is the cheap next probe; no AC here required it. Options 3/4/5 are prepared below, not executed.

### Units 4+: SQ-4 list (ruled YES), highest BVP first
| task | BVP | outcome | evidence | commits |
|---|---|---|---|---|
| T-2644 | 78 Q1 | **partial-complete** (owner→human, `[REVIEW]` attach AC pending). The attach loop stopped swallowing a failed `command.inject`: a dead socket now detaches with `Connection lost — input not delivered: …`, and a refused inject gets one hint per failure streak. Decision recorded | 2 tests; mutants (first failure silent / no throttle) both red. **Live scripted attach under a real PTY** (`script`): keystrokes delivered; then I killed the session process mid-attach and typed, which produced exactly one `Connection lost — input not delivered: … os error 111` and `Detached.`, with no garbled render. The NotDelivered path cannot be reached live (attach refuses a non-PTY session at its pre-check), so only unit tests cover it. CLI 1158/1158. RCA | `7b32bce24` `39f54be88` |
| T-2573 | 76 Q1 | **PARKED → SQ-11** | code read + consumer grep (below) | — |
| T-2606 | 73 Q1 | **PARKED → SQ-12**: its Decisions say "OPEN … Owner to decide" | — | — |
| T-2656 | 73 Q1 | closed. The attach output poll now surfaces `query.output` RPC errors through a typed `PollStep`. Fatal errors (incl. -32007, because attach pre-checked output was available) detach with a named reason; `RATE_LIMITED` gets one notice per streak. Decision recorded | 3 tests, including a **real UnixListener fixture** that answers -32007 with the socket open; the pre-fix mutant is red. CLI 1161/1161. RCA | `c9c6f7167` `6877de968` |
| T-2662 | 73 Q1 | closed. `file receive` prints the first mid-transfer RPC error to stderr and bails after 3 consecutive errors (`reason:"disconnected"`, chunks N/M), as `cmd_wait` does | AC3 was met through the task's **structural fallback**, not a dropped-source fixture. My first structural test was too weak (the mutant survived); I tightened it and the mutant is now red. CLI 1163/1163; busy-spin + silent-exit rc 0. RCA | `c4b2e435d` `5af8dd384` |
| T-2616 | Q1 | closed. (a) A failed dead-letter move now **keeps** the poison post at the head of the queue instead of deleting it through the fallback, and adds `FlushReport::dead_letter_failed` plus an `error!`. (b) A cross-pass `transport_fail_streak` escalates `debug!`→`warn!` at 12 passes, then every 12 | **Real fault injection:** `DROP TABLE dead_letters` under the live queue reproduces the move-fails ∧ delete-succeeds window; the post is kept, and the mutant restoring the delete is red. termlink-session 529/529; `cargo check --workspace` rc 0. RCA | `a4b748fee` `fb7dfe0a3` |
| T-2581 | Q1 | closed. **The fix itself had already landed as T-2737** (a duplicate filing, T-229 class). What was missing was an end-to-end test through `Drop`; I added `drop_leaves_no_zombie_for_a_spawned_session` (ESRCH after drop, no `/proc` read) | pre-T-2737 mutant red 5/5, real code green 5/5; session 530/530. RCA | `8baecdd85` `c540dc2e7` |
| T-3141 | Q1 | closed. All agent ACs were already ticked by an earlier round; the verb gate ran P-011 2/2 green | ⚠ bypass, see top | `67e724d19` |
| T-3139 | Q1 | **partial-complete** (human route-confirm AC). The gate refused on an empty `## Recommendation`, so I wrote one: **GO**, citing the filing at **`framework:pickup` offset 154 + addendum 155**, which I read back this round. The task had never recorded the offset | P-011 3/3 | `3965b9508` |
| T-3235 | 57 Q1 (filed + scored by me) | closed. In `file receive`, the RPC-timeout arm `continue`d past the `--timeout` check, so a session that accepts connections but never replies could keep the command alive indefinitely. Found while reading T-2662 | structural test; mutant restoring `continue` red. CLI 1164/1164; busy-spin rc 0. No live proof | `c66862d08` `268ac10d8` |
| T-3177 | Q1 | **not closable by me → Closure request 5** | P-011 4/6 FAIL | — |
| T-2886 | Q1 | gated: vendored fragment, already filed upstream (offset 85), and the operator has banned worktrees | — | — |
| T-2911, T-3091 | Q1 | gated: placeholder ACs. T-2911 is a pickup proposal with an empty context (inception-shaped, G-020). T-3091 means designing a new moment-of-action gate for shared-infra actions (→ SQ-13) | — | — |
| T-2016, T-2015 | Q2 | gated: the remaining AC waits for an upstream `upgrade.sh` fix to land | — | — |
| T-2669 | Q2 | gated: blocked on the human per-verb timeout decision (its own AC says so) | — | — |
| T-2532 | Q2 | gated: its Decisions are "OPEN … Human call … Do NOT guess-and-ship" (cap at all? per-host? default?) | — | — |
| T-2398 | Q2 | not done: it launches two armed live Claude agents on the fleet rail. That is live shared infra, and running it from a dispatched worker would contend with the orchestrator (SQ-6 shape) | — | — |

Guard follow-up: my T-2656 refactor moved an already-acknowledged unbounded `query.output` call from `attach_loop` into `poll_output_step`, so `check-unbounded-rpc-call` re-fired, as designed. I **carried the acknowledgement over** instead of choosing a timeout, because choosing one would pre-empt the pending T-2669 human decision. Check back to rc 0 (`51462681f`). These members ran green after all changes: platform-lock, silent-exit, busy-spin, alloc-sink, drain-sink, error-code-emission, mcp-parity-census, version-derivation, canary-log-hygiene, plus 4 fixture suites. **The full guard layer was NOT run** (~16 min, over the tool cap; I must not background it and end my turn).

## Closure requests (SQ-3: operator grants or refuses each)
I have not ticked, reworded or struck any of these ACs.

1. **T-3132**. AC: "fw audit's CTL-029 WARN count for this bundle drops to 0". **Cannot be met:** 25 of 27 bundle tasks are `owner: human` in their designed terminal state (PL-376). T-3010 must stay in `active/` until 2026-12-18, because `revisit-due-scan.sh` reads only `active/`, and closing it would silently kill its G-053 reminder. **Closing keeps** the 2 ticked ACs and G-095 (the check flags a designed end state). **Closing loses** nothing actionable. Reaching 0 needs a CTL-029 fix, which is vendored.
2. **T-3128**. AC: "CTL-003 itself reports PASS on re-run". **Cannot be met by this task:** the check and its writer are vendored (G-062), and any PASS here would be incidental, because this session's own hook refreshes the shared `.budget-status`. The real fix is tracked upstream via T-3127. **Closing keeps** the local diagnosis; **loses** nothing that a local worker could deliver.
3. **T-2958**. ACs: (a) "the emitter is corrected BEFORE any affected record is re-decided", (b) "a verb or documented path exists to re-record a decision on a completed inception", **answered NO**. From code: `do_inception_decide` (`lib/inception.sh:440`) looks only in `active/`, so none of the 23 divergent records (all in `completed/`) can be re-decided at all, and (a) is moot. The emitter is vendored and was filed upstream (offset 208, read back). **Closing keeps** the filing and the finding; **loses** a local reminder that the 23 records are still inverted (worth a concern entry if closed).
4. **T-3130**. AC: "reproduce with a minimal fixture". **Measured negative:** 12/12 trials preserved `owner`. The defect is load-dependent (it was seen under orchestrated dispatch), and the task proves the mechanism separately. **Closing keeps** the negative result and the mechanism proof; **loses** an open reproduction under load.
5. **T-3177**. All agent ACs are ticked, but **P-011 fails 4/6**: its Verification lines glob `.tasks/active/T-2879-*.md`, and T-2879 has since been **decided and moved to `completed/`** (`date_finished: 2026-09-26T22:11:43Z`), so `grep` exits 2 on a missing file. The move is itself evidence the task succeeded, since T-2879 became decidable. Retargeting the glob to `.tasks/*/T-2879-*.md` keeps each check's meaning, but that edits the gate on work I did not produce. **Request:** authorise that one-line glob change (or a `--force` close), or refuse.

## Sovereign questions (priority order)
1. **SQ-9 (carried): options 3/4/5.** Once the push confirms Doc Lint at 9 red: **(3)** resolve or acknowledge the 8 TREE findings (arc-claim-drift, audit-warning-ack, go-propagation, pickup-deferred P-078, receiver-ack-lag, unpaired-capture, runme-fixtures, voi-prompt T-3200); **(4)** re-run `release.yml` for v0.12.0 or cut v0.12.x (v0.11.2 is still the latest published release); **(5)** a release-publication canary (latest tag has no GitHub release / no green Doc Lint in N days). All three are prepared and none executed. Also: whether to spend the cheap option-6 probe on `planted-default-gate`.
2. **SQ-11 (new): T-2573, the subscribe deadline contract.** AC2 as written ("`next_cursor` = exactly `last_collected+1`") would **introduce a livelock**: with a `conversation_id`/`in_reply_to` filter, a deadline that trips before any match returns the input cursor, and the next call re-walks the same span. That is the (b) failure the task itself rules out. The lossless, livelock-free cursor is the existing `last_scanned+1`. Also, **no in-tree consumer resumes from `data.next_cursor`**: all ~20 `channel.subscribe` call sites (CLI/session/MCP) fail loudly on -32020, so the silent loss only reaches out-of-tree clients that follow the error's advice. Options: (i) reword AC2 to `last_scanned+1` and adopt (c2), keeping the error and adding `data.messages` (small, additive, no in-tree behaviour change); (ii) (c1) flagged partial success plus a ~20-site consumer audit (large; turns loud failures into silent short pages for single-shot consumers); (iii) text-only fix to the error message.
3. **SQ-12 (new): T-2606, exactly-once across hub restart.** The three options are A (a durable dedupe table), B (`client_msg_id` in the Envelope, consumer dedupe), or C (document at-least-once, which downgrades the charter claim). The task leans A, but marks the choice "Owner to decide".
4. **SQ-13 (new): T-3091.** Should a dispatched worker consult a checkable predicate (run record + rulings) at the moment of a shared-infra action? If so, where does it live: a script gate, or a hook? This is a new structural gate, so it needs a ruling first ("one lock at a time").
5. **T-2532**: cap at all? per-host? what default? (its own OPEN block).
6. **T-2669 per-verb timeout decision**, which now also covers `pty.rs::poll_output_step` (allowlist carry-over above).
7. **Carried unchanged:** SQ-1/2, SQ-6, SQ-7, SQ-8, SQ-10, T-2385 `[REVIEW]` (still gates T-3227), T-3006 ready for `decide no-go`, and the human ACs now pending on T-2644 and T-3139.

## Gates that refused, and what I did
1. **G-020** on T-3232 and T-3234: it refused scratch-dir commands while the ACs were still placeholders. Correct. I wrote real ACs first.
2. **P-002 "no active task"**: this fired several times after a close cleared focus. I refocused T-3211 each time.
3. **G-019 RCA gate** on T-3233, T-2644, T-2616, T-2581: it refused to close with an empty `## RCA` (the titles matched "fail"). I wrote each RCA; I never used `--skip-rca`.
4. **P-010/P-011** on T-3177: 4/6 verification lines fail on a moved file. I did not edit them → Closure request 5.
5. **Partial-complete review gate** on T-3139: it refused on an empty Recommendation. I wrote one, with evidence.
6. **P-002 "completed task in focus"** on T-3139's commit: I refocused T-3211 and committed with no bypass.
7. **Focus-drift gate**: **I bypassed it once with `FW_SWITCH_FOCUS=1` (logged) when no bypass was needed.** See ⚠ 1.

## Findings for the operator
- **F19**: Install Check had been red for 3 months (947 runs) with nothing firing. That is the same shipped≠live blindness as SQ-9, one workflow over. Option 5's canary should also cover `install-check.yml`, not only Doc Lint.
- **F20**: The allocator/hunt duplicate class (T-229/T-2800) struck again: T-2581 and T-2737 describe the same defect. `check-task-id-collisions.sh` axis B compares branches only, so it cannot see a same-branch duplicate.
- **F21**: The RCA gate's keyword trigger (R5's F16) also fired correctly on T-2581 ("fix:" prefix). Coverage still depends on how the title is worded.
- **F22**: `let _ = <rpc>` and `if let Ok(..)` with no else, in interactive loops, hid 3 failures this round (T-2644, T-2656, T-2662). No static check covers that shape. A candidate for a sibling of check-silent-exit, but whether it is worth building is not decided here.
- **F23**: Finalize commits need the focus on a live task. Refocus the orchestrator task **before** committing a finalize. Never reach for `FW_SWITCH_FOCUS` (see ⚠ 1).

## Cost-vs-estimate deltas
- The SQ-9 tasks were estimated at cost 3.2 and all came in lower. T-3232 was a one-line pin, but diagnosing it took ~20 min (reading the log, then a fresh-resolve reproduction). T-3234 was the largest: 6 files, and each needed its own reproduction method.
- The CLI loop fixes (T-2644/T-2656/T-2662/T-3235) are at the bottom of cost 2. The CLI suite (~30s) and incremental builds dominate. The pure-helper-plus-mutant pattern is cheap and caught one weak test of my own (T-2662).
- T-2616 cost more than its peers because it needed fault-injection design, but the `DROP TABLE` trick made the test cheap and exact.
- **The estimator scored all 4 new tasks exactly 57** (identical drivers); it does not discriminate between them. Ordering among equal scores still needs human-legible severity.
- Waste: about 5 min on the first scripted-attach attempt (wrong tmux session name). Checking the actual name first would have avoided it.

## Arc state
- No arc membership changed. The SQ-4 arc-less Q1/Q2 list is worked down: 7 closed, 2 partial-complete, the rest gated as above.
- **Remaining Q1/Q2 that are agent-workable and ungated: none found.** Everything left waits on a human ruling, a human AC, or an upstream fix. **If SQ-11/12/13, T-2532 or T-2669 are ruled, each becomes a ready unit.**

## Checks recorded
| # | check | result |
|---|---|---|
| 1 | own transcript + budget (stdin form) | ba9bfd8a; 162K → 230K → 258K → 281K → 294K → 324K → 350K → 371K |
| 2 | `gh run view 36562595196 --log-failed` | E0425 `schema_for_input`; `rmcp-macros v1.8.0`; last install-check green 2026-06-23 (947 failures since) |
| 3 | `gh run view 36562595162 --log-failed` | 25 red members (cron-drift already fixed) |
| 4 | fresh resolve pre/post pin | rmcp-macros 1.8.0 → 1.3.0; fresh-lock check rc 0; `--locked` rc 0; lint-command-hints OK |
| 5 | 10 git-depth members, depth-1 vs full clone, CI env | 10/10 fail vs 10/10 rc 0 |
| 6 | 7 env members, CI env, before/after; origin rc diff | 3+2 reproduced (+planted-default NOT); after 6/6 rc 0; diff empty |
| 7 | T-2644 tests, mutants, scripted PTY attach | 2/2; 2 mutants red; ConnectionLost live-proven |
| 8 | subscribe consumer grep; `walk_subscribe_records` read | ~20 sites, 0 handle -32020; `last_offset` = last scanned |
| 9 | T-2656 fixture + mutant; CLI suite | 3/3; red; 1161/1161 |
| 10 | T-2662 tests + mutant (weak then strengthened); busy-spin, silent-exit | 2/2; survived → red; rc 0/0 |
| 11 | T-2616 fault-injected test + mutant; session suite; workspace check | pass; red; 529/529; rc 0 |
| 12 | T-2581 Drop test; mutant ×5; fixed ×5 | red 5/5; green 5/5; 530/530 |
| 13 | framework:pickup read-back for T-3139 | offsets 154 + 155 present |
| 14 | P-011 on each close | T-3232 2/2, T-3233 3/3, T-3234 6/6, T-2644 3/3, T-2656 3/3, T-2662 3/3, T-2616 3/3, T-2581 3/3, T-3141 2/2, T-3139 3/3, T-3235 2/2; T-3177 **4/6 FAIL** |
| 15 | gate-bypass-log count across finalize commits | +1 at T-3141 (FW_SWITCH_FOCUS); 504→504 at T-3139, T-3235 |
| 16 | 10 static checks + 4 fixture suites post-change | all rc 0 after allowlist carry-over (check-unbounded-rpc-call was rc 1 before) |
| 17 | `git rev-list --count origin/main..main` | 22 |
```
