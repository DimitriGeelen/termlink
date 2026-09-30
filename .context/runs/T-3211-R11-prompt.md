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

## ORCHESTRATOR CONTEXT — round 11 of 11 (T-3211)

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

OPERATOR RULINGS IN FORCE: SQ-4 YES; SQ-3 SURFACE-THEN-ASK; SQ-9 approved; guard classification approved in full.
NEW RULING, 2026-09-30 — SQ-22 = OPTION C: "A plus the next agent round commits automatically." Nothing commits
unattended; files written by unattended jobs are committed by the next SESSION, as a standing first step. The
operator added: this must be STRUCTURAL — in /resume and in the pre-compact/handover commit — and it belongs in the
FRAMEWORK too, so inform the AF agent (framework:pickup), "possibly include the files with it". NEVER tag, NEVER push.

ORCHESTRATOR FINDINGS (measured, 2026-09-30) — your starting facts:
 - handover --commit (the pre-compact path) commits ONLY the handover file + LATEST.md, pathspec-scoped since T-3090
   (.agentic-framework/agents/handover/handover.sh ~1455-1475). It does NOT commit canary filings. Keep that scope
   discipline — the T-3090/T-3231 whole-index sweeps are why it exists.
 - /resume (.claude/commands/resume.md, installed from the vendored template .agentic-framework/lib/templates/
   resume-md.md; a user-level copy also exists at ~/.claude/commands/resume.md and DIFFERS) only runs git status and
   reports "N uncommitted files" — on this host ~20 routine working files, so a canary filing drowns.
 - post-compact-resume.sh has no uncommitted-state handling.
 - scripts/warn-escalation-file.sh (T-3267) and the release canary write the ledger
   .context/checks/guard-warn-first-red and new .tasks/active/T-*.md, and leave them uncommitted by design.

BUILD (create a build task; real ACs before edits):
 1. A general "pending commit" mechanism, not WARN-specific: unattended writers APPEND the paths they wrote (+ the
    task id to attribute the commit to, + a one-line reason) to a manifest, e.g. .context/working/pending-commit.list.
    A helper (project-owned, e.g. scripts/commit-pending.sh) commits EXACTLY those paths (git commit -- <paths>),
    one commit per task id with that id as the message prefix, verifies HEAD carries each file's content, then
    removes the committed entries. It must: never widen to other paths; tolerate an entry whose file is gone
    (report, drop); never commit a path outside .tasks/ .context/ (refuse, loud); be idempotent; work when focus is
    another task WITHOUT a bypass (if the focus-drift gate refuses, find the supported path or report — do NOT
    self-grant FW_SWITCH_FOCUS). Wire warn-escalation-file.sh + the canary ledger refresh to append to it.
 2. Wire the helper in (local): the procAsFit round prompt (scripts/run-procasfit-round.sh) as a first step;
    .claude/commands/resume.md — Step 1 lists pending-commit entries by name (not in a count) and Step 3 offers/
    runs the helper; the handover/pre-compact path runs it before its own pathspec commit. handover.sh is VENDORED:
    either a project-owned wrapper point exists (use it), or patch it and REGISTER the divergence in
    .vendor-divergence.yaml (status local-only -> filed-upstream after step 3), per G-062/T-2812.
    check-vendor-divergence must end classified.
 3. Inform the AF agent: post a framework:pickup filing (see how earlier filings did it, e.g. offsets 162/228 and
    scripts/post-framework-pickup* or the documented recipe) proposing the mechanism for the framework itself:
    the manifest convention, the helper, the resume template change and the handover hook, with the measured
    reasons (T-3090, T-3231, this host's ~20-file baseline drowning a single filing). Include the actual proposed
    file contents / diffs in the filing (the operator asked for the files). Read the posting back (T-2876) and
    record the offset. Mark the divergence filed-upstream with that offset.
 Fixtures with mutants: helper commits only listed paths while unrelated files are staged (the sweep case);
 refuses outside-scope paths; idempotent; missing file dropped with a report; resume lists entries by name.
Before any commit after a refused finalize: git status --porcelain; add completed/ file + episodic together.

### Your handback

Write it to EXACTLY this path, in markdown, following the Handback section of
the mandate above:

    /opt/termlink/.context/runs/T-3211-R11-handback.md

It is the only thing the next round receives. A claim in it that is not
traceable to a recorded check or a verb-gated state change counts as an open
task, not a closed one.

### Previous round's handback (round 10)

This is the state you inherit. Do not redo its completed work; do pick up
its unfinished Q1/Q2 items and its unresolved Sovereign questions.

```markdown
# T-3211 — procAsFit round 10 of 10 — handback

**Status: COMPLETE.** I stopped on **stop condition 1**. Both operator units are done and proven, and so is one follow-up those units surfaced. Every remaining Q1/Q2 item in the SQ-4 pool is gated (table below).

**Budget at stop:** ≈280K of 800K (~35%), under `TOKEN_WARN` (600K). I read it on stdin from my own transcript (`f5147e4a-02c8-487c-bb75-eb13fa760534.jsonl`; its recent entries are my turns).

**Result:**
- 3 tasks filed, scored before starting, and closed by verb: **T-3267** (Unit 1), **T-3266** (Unit 2) and **T-3268** (the follow-up).
- 3 commits on `main` since `9c62d123f`. Local `main` is 4 ahead of origin: the orchestrator's operator-note commit plus my 3.
- Nothing pushed, nothing tagged, no crontab installed, no tier changed.

## ⚠ Read first
1. **The WARN filer has never filed a real task on this host, and that is expected.** Live runs (`--dry-run` and a real run) produced no decisions. The ledger holds 2 red WARN members, both under 14 days old:
   - check-receiver-ack-lag, since 2026-09-29T22:04:59Z;
   - check-audit-warning-acknowledgement, since 2026-09-30T00:33:22Z (added by a canary run before this round).

   So the first real filing is **2026-10-13 (receiver-ack-lag)**, unless SQ-19 is resolved first. Filing into a real `.tasks/` is proven only by fixture (fake `fw task create`). Two supporting checks were run directly:
   - the `ID:/File:` output format it parses matches what the real `fw task create` printed when I created T-3267;
   - `write_body` succeeds against a copy of the real `.tasks/templates/default.md`.
2. **SQ-22 is a PROPOSAL, not a ruling.** It is written into the T-3258 draft's rulings section. Until you rule, filed tasks and the ledger stay uncommitted but loud.
3. The ledger `.context/checks/guard-warn-first-red` is **modified and uncommitted** in the working tree (+check-audit-warning-acknowledgement). Under the SQ-22 proposal I deliberately did not commit it.

## Selection
- **Objective:** the operator's release-readiness ruling set. That means (1) the binding WARN condition, (2) the last macOS test red.
- **Arc:** guard-layer severity, the successor of T-3258/T-3260.
- **Order:** operator-directed; Unit 1 then Unit 2.
- **Scores (estimator + `cost-one`):**
  - T-3267: D1=4 D2=0 D3=3 D4=2; cost tier 2 / effort 8.
  - T-3266: D1=4 D2=0 D3=3 D4=3; tier 2 / effort 8.
  - T-3268: D1=4 D2=0 D3=3 D4=2; tier 2 / effort 8.
  - `blast_radius` is unmeasured for all three (no `components:`), so **no quadrant is computable**; `fw bvp --quadrant` excludes them. I could not state Q1/Q2 mechanically. I treated the BVP values as high-value because they are at or above the hv-lc tail, which is 57.
- **Why T-3268 after the operator units:** it is the defect class that produced the Unit 2 red, it is inside the same arc, and it is small. It was scored before starting. No ungated SQ-4 item outranked it; all were gated.

## Work log

| task | outcome | evidence | commit |
|---|---|---|---|
| **T-3267** (Unit 1) | **closed**. The WARN escalation now files tasks (details below) | `tests/warn-escalation-file-fixtures.sh` **34/34, 7 mutants killed**; release-publication fixtures 30/30; runner fixtures 57/57; severity fixtures 32/32; marker checker clean (93 marked, 23 warn, 1 info); live host filer run: no decisions, nothing filed; **P-011 7/7** | `e0b4da48f` |
| **T-3266** (Unit 2) | **closed**. The macOS procfs test asserts the platform fact on both families and asserts the documented degradation through a pure seam | `cargo test -p termlink-mcp --lib`: **942 passed, 0 failed**; the two target tests ok; clippy has no findings on the changed lines; check-platform-lock clean; **P-011 3/3** (reviewer PASS) | `4276d98a6` |
| **T-3268** (follow-up) | **closed**. check-platform-lock was blind to quoted bare `"/proc"` / `"/sys"` roots, which is exactly why T-3266's red had scanned clean | fixtures **26/26** (new fixture 10 plus a mutant); real tree: **10 scanned, 10 acknowledged**, clean; **P-011 3/3** | `95ac76d41` |

### T-3267, as built
- **`scripts/warn-escalation-file.sh`** (new) is run by `check-release-publication-freshness.sh` step (c2), on the host path only.
- **STALE:** a WARN member red for more than 14 days files one task. The task is owner agent, horizon now, tagged `warn-escalation,guard-layer`. Its Context carries:
  - the member's **current output** (the member is re-run once, `timeout 120`, last 40 lines plus rc);
  - its first-red date;
  - its operator tier reason.

  Its AC is "`<m>` is green, or the operator has reclassified it".
- **ACCUMULATION:** more than 5 WARN members red at once files **one** umbrella task naming all of them, immediately. The canary now also FIRES on "N WARN guard members red at once (> 5)".
- **DE-DUP:** a body marker, `<!-- warn-escalation: member=<m> -->` or `<!-- warn-escalation: umbrella -->`, is checked against every `.tasks/active/*.md` whatever its status. S6 proves this with status `issues`.
- **RE-FILE rule (I defined it; you can overrule it):** closing the task while the member is still red restarts the window at `date_finished`. The filer files again only when the member has been red for more than 14 days past max(first-red, close). The umbrella waits the same window after its close. I rejected the alternatives:
  - immediate re-file, because closing would become meaningless;
  - re-file only on a fresh red, because a member that never goes green would never re-escalate.
- **Fences:**
  - refuses under `$CI` (prints SKIP);
  - never files on a FAIL-tier member (N1);
  - an unreadable member list is rc 2, never "file without the tier check" (T3);
  - never fixes anything; no tier touched.
- **Canary wiring:** each filing FIRES as `filed T-XXXX for <m> (stale|umbrella) — UNCOMMITTED: review, then commit it`, and an open task is reported daily. A filing failure **FIRES**; it does not exit 2, so it cannot mask the release and workflow verdicts. The test seam files only with `RELEASE_PUB_TEST_FILE_TASKS=1`. There is a new flag, `--no-file-tasks`.
- **Runner:** `run-guard-layer.sh --list --json` now carries `reason` and `cmd` (additive). **A bug I introduced and caught along the way:** my first encoding used `printf '' | jq -R .`, which emits *nothing* for an empty string. That produced invalid JSON (`"reason":,`) for every FAIL member. The fixtures exposed it because the filer then fell back to an empty member list silently. I fixed it with `jq -n --arg`, and made the filer **fail closed** on an unreadable list.
- **Fixture coverage (the required cases):**
  - 13d does not file (S1); 15d files once (S2); the body content is right (S3); owner and horizon are right (S4);
  - the second run does not duplicate (S5/S6);
  - 6 red files one umbrella (U1); 5 does not (U3); 6 stale reds file 6 tasks plus an umbrella (U4);
  - after close and still red: WAIT at 0d and 13d, re-file at 15d (R1–R3); umbrella equivalent (R4/R5);
  - the canary fires and files (K1–K3); 5 young reds stay healthy (K4); a filing failure fires (K5); the seam does not file unless asked (K6);
  - **mutants:** de-dup, threshold, `>`→`>=`, re-file window, CI guard, canary filing call, accumulation line; all 7 killed.
- **Docs:** CLAUDE.md *Severity tiers* has a new "Escalation is an ACTION" paragraph. The T-3258 draft has an "Implemented" bullet plus the SQ-22 **PROPOSAL**.

### T-3266, as built
- New `whoami_helpers::auto_resolution_for(procfs: bool) -> (&str, String)`, a pure function. The live MCP whoami path now calls it (identical strings; the full lib suite is green).
- `mcp_procfs_probe_matches_cli_semantics` is **one test with a runtime `cfg!(target_os)` branch, not a `#[cfg]` skip**:
  - Linux: probe true, and mode `attempted`;
  - non-Linux: probe false, and mode `unavailable-no-procfs`.
- New `mcp_whoami_degradation_both_branches_via_seam` proves **both** branches on this Linux host.
- **Stated plainly: I could not run macOS here.** The non-Linux branch *of the platform test itself* has never executed. The seam proves the degradation mapping, and the release job's test-macos run is the real-platform proof.

## What remains in Q1/Q2, and why it was not done

| task | why |
|---|---|
| T-2573, T-2606, T-3091 (Q1) | gated on SQ-11/12/13 (carried from R6/R7) |
| T-2886, T-2911, T-3227 (Q1) | R7's gated pool (T-3227 has a human AC) |
| T-3177 (Q1) | R6 closure request, pending |
| T-2016, T-2015 (Q2) | the remaining AC waits for an upstream `upgrade.sh` fix |
| T-2669 (Q2) | human per-verb timeout decision |
| T-2532 (Q2) | its own Decisions say "OPEN … Human call … Do NOT guess-and-ship" |
| T-2398 (Q2) | launches two armed live Claude agents on the fleet rail (live shared infra, SQ-6 shape) |
| T-3245/46/47, T-3250–53, T-3255 | carried from R8/R9 (SQ-17; T-3250 human-owned; hub.version sha plus redeploy) |
| T-2644, T-3139 | human ACs pending (status work-completed, owner human) |

## Sovereign questions (priority order)
1. **SQ-22 (proposal written): who commits filed escalation tasks and the WARN ledger?**
   - **Proposed:** nobody unattended. They stay uncommitted but loud (the canary fires per filing, they appear in handover active lists), and the next interactive session commits them. Rationale: a cron commit races live sessions' shared index (T-3231 class).
   - **Alternative:** a pathspec-restricted cron commit.
   - Text is in the T-3258 draft § Operator rulings.
2. **Re-file rule (T-3267):** accept "new 14-day window from the close date", or choose otherwise.
3. **SQ-19 (carried):** check-receiver-ack-lag is WARN and red. It now **files a task on 2026-10-13**, not just escalates.
4. **SQ-9 (carried, approved):** install `release-publication-canary.crontab`. **Without it nothing is ever filed.** The filing path lives only in that cron job. Also `gh` auth for root.
5. **SQ-20, SQ-21 (carried from R9):** the E4 prover with a binary but no identity; heading-shadow against the 300s timeout.
6. Carried unchanged: SQ-17, SQ-15 (F37 voi-prompt writes), SQ-16, SQ-1/2, SQ-6/7/8, SQ-10–13, and R6's self-granted bypass entry.

## Gates that refused, and what I did
1. **G-020 build-readiness gate (×3):** it blocked reads and writes while T-3267 had placeholder ACs. The block included a `python3` heredoc edit of the task file. I wrote real ACs with the Edit tool first, then proceeded. Separate finding: `fw bvp estimate` is not on the read-only allowlist. The gate itself says that is "a gap in the allowlist worth filing"; I did not file it.
2. **T-1730 focus-drift gate** refused a `T-3267:`-prefixed commit. At that point I had already refocused T-3211, and T-3267 was completed, so it could not be focused. **I did not use FW_SWITCH_FOCUS.** I committed under `T-3211:` (the orchestrating task; the message names T-3267). For T-3266 and T-3268 I committed **before** refocusing, while focus was cleared, and the gate passed. **Recommended pattern for future rounds: finalize → `git add` → commit → only then `fw context focus`.**
3. `git add` with a pathspec for a never-tracked `active/` file aborted the whole add (fatal). I re-ran it without that path and checked the staged set.
4. **Bypass log:** no entries dated 2026-09-30. No `--force`, no `--skip-*`, no `--no-verify`.

## Findings for the operator
- **F39:** my first `--list --json` extension emitted invalid JSON for empty reasons, and the filer's fallback **silently** treated that as "no members". That is exactly the failure mode the scope fence must not have. Both are now fixed (encoding plus fail-closed). This is the T-2680 lesson in miniature: a fallback that reads "empty" as "fine".
- **F40:** the P-011 reviewer flagged **skip-as-pass** on T-3267's Verification line 5 (`warn-escalation-file.sh --dry-run`, rc-only; it would pass on an absent ledger). The finding is correct. The line is redundant with the fixture line, and the task is closed. I am leaving it recorded rather than reopening the task.
- **F41:** three of this round's tasks got **no BVP quadrant** because `blast_radius` needs `components:`. New tasks created by `fw task create` never have components, so "select by quadrant" is mechanically unavailable for fresh work (T-3068 territory).

## Cost-vs-estimate
- All three tasks scored effort 8. T-3267 took most of the round (≈60% of my tokens): three files, a fixture suite and two docs. T-3266 and T-3268 were each roughly a tenth of that. Effort does not discriminate; `blast_radius` would have, had it been measurable (F41).
- The filer fixture suite runs in about 25s. The full `termlink-mcp` lib test compile plus run was the longest wait, and it cost little context.

## Checks recorded

| # | check | result |
|---|---|---|
| 1 | own transcript + budget (stdin form) | f5147e4a; 165K → 243K → 256K → 280K |
| 2 | warn-escalation fixtures | 34/0, 7 mutants killed |
| 3 | release-publication / runner / severity fixtures; marker checker | 30/0 / 57/0 / 32/0 / clean |
| 4 | live filer on host (dry + real) | rc 0, no decisions, no files |
| 5 | `write_body` on a copy of the real default.md | ok |
| 6 | `cargo test -p termlink-mcp --lib` | 942 passed, 0 failed |
| 7 | platform-lock fixtures / live scan | 26/0 / 10 scanned, 10 acknowledged, clean |
| 8 | P-011 | T-3267 7/7, T-3266 3/3, T-3268 3/3 |
| 9 | `git rev-list --count origin/main..main` | 4 (3 mine) |
| 10 | bypass log entries on 2026-09-30 | 0 |
```
