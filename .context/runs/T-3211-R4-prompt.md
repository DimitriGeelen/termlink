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

## ORCHESTRATOR CONTEXT — round 4 of 9 (T-3211)

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

### Operator directive for this round

The operator (2026-09-29) reviewed R3's stop and directed: "Run it till we're out as far as you can."
So the sequence continues past R3's stop condition. This does NOT rule on any Sovereign question:
SQ-1..SQ-8 remain UNRULED. Do not decide them, reword human ACs, or route around them.

Your job this round: find agent-workable units that are NOT gated by an open SQ or owner:human.
Places R3 may not have exhausted: in-flight arcs' Q3/Q4 work once Q1/Q2 are gated; unassessed or
cost-less tasks that need scoring before they can be placed (scoring is allowed; confirming is not);
tasks that can be closed with cited evidence (e.g. SQ-5's "not reproduced" route, only if the evidence
is yours and recorded); prerequisite work that makes an SQ cheaper for the human to rule on (e.g. a
written options analysis with measurements) — surfacing, not resolving.

If after an honest, recorded census nothing agent-workable remains, say so in the handback with the
census as evidence and stop. That is a valid round outcome, not a failure — and it tells the
orchestrator to end the sequence rather than launch another round into the same wall.

### Your handback

Write it to EXACTLY this path, in markdown, following the Handback section of
the mandate above:

    /opt/termlink/.context/runs/T-3211-R4-handback.md

It is the only thing the next round receives. A claim in it that is not
traceable to a recorded check or a verb-gated state change counts as an open
task, not a closed one.

### Previous round's handback (round 3)

This is the state you inherit. Do not redo its completed work; do pick up
its unfinished Q1/Q2 items and its unresolved Sovereign questions.

```markdown
# T-3211 — procAsFit round 3 of 9 — handback

**Status: COMPLETE — stopped on the mandate's first stop condition (every remaining Q1/Q2 path in every in-flight arc is behind a Sovereign question). Budget at stop ~42% (< `TOKEN_WARN` 75%).**
Worker: `claude -p` (Fable 5.1), started 2026-09-29 ~08:44 local. Run record: `.context/runs/T-3211-R3-*`.
Transcript: `~/.claude/projects/-opt-termlink/d57d80a2-….jsonl` (confirmed mine: `entrypoint: sdk-cli`,
`lastPrompt` is this mandate).

## Baseline at run start

- HEAD `d33a730db`. Focus T-3211. VERSION 0.12.232.
- Budget at start: **158,890 cache_read (~20% of CONTEXT_WINDOW=800000)** from own transcript's latest
  usage entry. `lib/context_tokens.py` printed **0** for this file (R2's F4 reproduces). Stop
  threshold by NAME: `TOKEN_WARN` = 75%.
- Inherited (canonical R2 handback = attempt 2, `T-3211-R2-handback.md`): T-3037 + T-2991 closed;
  T-3213/T-3214/T-3215 created (captured, no cost, placeholder ACs); 7 SQs open.

## Selection

**Unit 0 — register repair (no selection needed; verb output never committed).** HEAD's copy of
`.tasks/completed/T-3135-*.md` still declared `status: captured` — R1's `244d7d03f` committed the
`git mv` but not the finalized content the verb wrote. Committed as `d09fdc3b2`; no content authored.

**Unit 1 — arc-009, T-3215 (Q1 hv-lc, BVP 60 / cost 2.0 after scoring).**
- **Objective:** arc-009 headline — every non-KEEP value-review finding reaches a terminal state the
  operator can see (T-2991's three filed findings are arc-009 members).
- **Arc:** arc-009, in flight, not blocked. arc-011 exhausted (R1); arc-008 agent-eligible Q1/Q2 all
  frozen on SQ-1/2/3 (R2 census, unchanged in `git log` since).
- **Task:** T-3215 over T-3213 / T-3214 because the tree already carried an UNGATED, uncommitted
  claim about it (R2 attempt 2's 02:15 edit to `parity.rs` saying "closed as not-a-defect" while the
  task sat `captured`; R2's transcript ends at 02:25 with "You've hit your session limit"). "One
  lock at a time" says resolve the inherited open lock before opening another; it is also the
  cheapest of the three to reach a terminal state.
- **Quadrant:** unscored on cost at pickup (blast_radius UNMEASURED, no components). Scored first
  (`fw bvp estimate` → no-change D1=4 D2=0 D3=3 D4=3; `estimator.py cost-one` → blast_radius=1
  tier=2 effort=8 → cost 2.0) → **Q1 hv-lc, BVP 60**. Then `fw work-on T-3215`.

**Unit 2 — arc-009, T-3213 (Q1 hv-lc, BVP 60 / cost 3.2 after scoring).**
- **Objective / Arc:** as unit 1. **Task:** T-3213 over T-3214 because it is a single-site CLI
  fix (`grep -c 'fetch_chat_arc_full(hub)' agent.rs` = 1) with a live parity pair waiting, whereas
  T-3214 needs an implementation-approach decision (port 3 checks vs subprocess the CLI verb) and
  touches `tools.rs` (45k lines). **Quadrant:** scored first (BVP no-change 60; cost-one →
  blast_radius=3 tier=2 effort=8 → 3.2) → Q1. Then `fw work-on T-3213`.

**Unit 3 — arc-009, T-3214 (Q1 hv-lc, BVP 60 / cost 3.2 after scoring) — PARKED on SQ-8, not started.**
- **Objective / Arc:** as unit 1. **Task:** the last T-2991 finding with real content (T-2978 is
  SQ-6; T-3010 is intentionally open). Measured the divergence myself (4 missing checks, not 3:
  `ufw_listener`, `inbox`, `secret_cache`, `secret_cache_profiles`; plus `strict` echo and an `ok`
  disagreement under strict). Read the three ways to reach parity; each changes a crate boundary or
  a test-harness contract that no existing convention settles (see SQ-8). Wrote the options and
  conditional ACs (AC0 = ruling recorded first) into the task, scored it, left it `captured`.
- **Why not decide it:** "Sovereign questions are surfaced, not resolved." The tempting option
  (subprocess the CLI verb, ~60 lines) would silently break two in-process integration tests, which
  is exactly the ungated-structural-change the mandate forbids.


## Objectives advanced

Against run start (HEAD `d33a730db`):

0. **T-3135 register repaired** — `d09fdc3b2`: HEAD's `completed/` copy said `status: captured`;
   now carries the verb's finalized content (`work-completed`, `date_finished` 2026-09-28T23:05:04Z).
1. **T-3215 CLOSED by verb as NOT-A-DEFECT** (`fw task update --status work-completed`; P-010 5/5,
   P-011 7/7 via `fw task verify`; episodic `.context/episodic/T-3215.yaml` parses to a mapping).
   Commit `7a38fceaf`. The "help catalog drift" was a stale `target/release/termlink` (built before
   T-3199's string edit) compared against a freshly compiled MCP side; `help.rs` wraps
   `termlink_mcp::build_cli_help_json`, one registry. Harness fix landed: all no-hub pairs resolve the
   CLI via `find_termlink_bin_fresh()`; `parity_help` is live and green (`1 passed; 0 ignored`).
   Census unchanged: 33 asserted / 227 acked / 0 unexamined. arc-009 headline advanced: one more
   T-2991 finding at a visible terminal state, with the reason recorded rather than a string patched.
2. **T-3213 CLOSED by verb** (P-010 5/5, P-011 6/6 via `fw task verify`; episodic mapping). Commit
   `2ed0583a6`. `termlink agent search <q> --json` on hub-down now prints
   `{"ok":false,"error":"Hub is not running …"}` on stdout, exit 1 (was: empty stdout + anyhow
   chain). The Err arm of the chat-arc fetch routes through `json_error_exit`, covering hub-down AND
   RPC error like the MCP twin. `parity_agent_search_no_hub` un-ignored, green; `check-silent-exit`
   clean. Second T-2991 finding at a terminal state.


## Arc state (tasks by status and quadrant)

Measured this round (`fw bvp --quadrant hv-lc|hv-hc --include-proposed` × active-file `tags`/`owner`/`status`):

- **arc-009 ("value-review execution")**: this round closed **T-3215** (Q1, not-a-defect) and
  **T-3213** (Q1, fixed); **T-3214** (Q1, BVP 60 / cost 3.2) scored and PARKED on SQ-8 (`captured`,
  AC0 = ruling first). Remaining agent-owned: T-2978 (Q1 57, `captured`, SQ-6 live-infra), T-3010
  (Q1 57, `started-work`, intentionally open by its own AC). 26 → 28 completed members.
- **arc-008 ("audit/doctor remediation")**: unchanged from R2 — T-3132 (Q1 85), T-3128 (Q2 85),
  T-2958 (Q1 57), T-3130 (Q1 57) frozen on one honestly-unachievable AC each (SQ-3); T-3103 (Q1 63)
  parked on SQ-1/SQ-2; T-3117 / T-3126 `owner: human`.
- **arc-011 ("mailbox to prompt")**: T-3211 (this orchestration), T-2389 (`owner: human`). T-3093 and
  T-3140 are NAMED in the arc yaml's notes but carry `tags: []` — not members by the ladder's
  definition (SQ-4).
- **Other arcs** (parallel-substrate, comms-loudness, substrate-fitness, mcp-slimming, push-transport,
  reliable-comms): active members are all `owner: human` or none. No agent-eligible Q1/Q2.
- **Q1/Q2 in NO arc, agent-owned** (unreachable by the ladder, SQ-4): captured — T-2644 (78),
  T-2573 (76), T-2606 / T-2656 / T-2662 (73), T-2616 (66), T-2886 (60), T-2532 / T-2581 / T-2911 /
  T-3091 (57); started-work — T-2015, T-2016, T-2398, T-2669, T-3089, T-3093 (6/6 ticked, P-011 3/3
  PASS, Evolution template-only), T-3139, T-3141, T-3177. Plus T-3140 (BVP 57, cost unmeasured,
  placeholder ACs — see SQ-4).


## What remains in Q1/Q2, per task, with reason not done

| task | Q | arc | reason not done this round |
|---|---|---|---|
| T-3214 | Q1 (60/3.2) | arc-009 | parity needs a crate-boundary or test-harness decision — SQ-8; options + conditional ACs written, scored, `captured` |
| T-2978 | Q1 (57/2.0) | arc-009 | relaunches live agents on the shared host — SQ-6 (carried); placeholder ACs |
| T-3010 | Q1 (57/3.2) | arc-009 | intentionally open (revisit_at carrier) by its own AC |
| T-3103 | Q1 (63/2.0) | arc-008 | remaining 9 carded-unwatched files are SQ-1 + SQ-2 (carried) |
| T-3132, T-3128, T-2958, T-3130 | Q1/Q2 | arc-008 | one AC each honestly unachievable — SQ-3 (carried) |
| T-2389, T-2197 | Q2 | arc-011 / parallel-substrate | `owner: human` |
| arc-less list above | Q1/Q2 | — | no arc → the project→arc→task ladder never reaches them (SQ-4); T-3093 is fully ticked and one Evolution paragraph from closable |


## Sovereign questions raised (unresolved, priority order)

1. **SQ-3 (carried, still the highest-value block)** — how is a task closed whose last AC is honestly
   recorded unachievable / answered NO? Four arc-008 tasks (BVP 85+85+57+57) are frozen this way.
   (a) human strikes/rewords the AC and the agent closes by verb; (b) a `--wontfix`-style verb with
   logged rationale; (c) they stay open as the honest record.
2. **SQ-8 (new) — how should `termlink_doctor` (MCP) reach parity with `termlink doctor` (CLI)?**
   MCP lacks `ufw_listener`, `inbox`, `secret_cache`, `secret_cache_profiles`; the `ok` verdicts
   disagree under strict. `termlink-mcp` is a dependency OF the CLI, so a port needs one of:
   (1) move the check collection into `termlink-mcp` and have both call it — the `help.rs` /
   `build_cli_help_json` pattern that T-3215 just proved makes drift impossible, but it drags the
   hubs.toml loader + dispatch manifest + secret-cache audit across a crate boundary; (2) MCP
   subprocesses `current_exe() doctor --json` (T-1689 pattern) — ~60 lines, but the MCP test harness is
   in-process so `current_exe()` is the TEST binary and two existing integration tests break unless
   tests plumb `TERMLINK_BIN` to a prebuilt CLI (the T-3215 freshness hazard, relocated); (3) duplicate
   the helpers (T-2069 says tiny helpers only; a second hubs.toml parser is the duplicated-registry
   class). Also: mirror `strict` into the CLI envelope + make CLI `ok` honour it, or drop it from MCP.
   Agent recommendation, not decision: (1) + mirror. Full write-up in T-3214's Context; AC0 records the
   ruling before any edit.
3. **SQ-4 (carried, sharpened)** — the highest-BVP agent-owned Q1 tasks belong to no arc, and so does
   **T-3140** ("artifact manifest.from must be identity fingerprint, not cli-pid label": three callers
   live-broken against any T-1427-enforcing hub, BVP 57, placeholder ACs) even though arc-011's own
   yaml names it as work filed out of T-3134. One ruling ("T-3140 is an arc-011 member" — or a
   defect-remediation arc for the whole list) makes it immediately eligible; without it the ladder
   never reaches a defect the arc itself discovered. T-3093 (6/6 ticked, verification green, arc-less)
   is the same shape from the other side.
4. **SQ-1 / SQ-2 (carried)** — fabric cards on 8 vendored BPMN files under a gitignored tree; the 34
   `.claude/commands/*.md` with one carded. Each resolves part of the audit's last carded-unwatched WARN.
5. **SQ-7 (carried)** — should 010-termlink adopt its own rail signing key (`fw rail post` refuses
   host-signed posts; option 3 `FW_ALLOW_HOST_SIGNED_RAIL=1` used so far, logged Tier-2).
6. **SQ-6 (carried)** — may an autonomous worker execute T-2978 (relaunch live agents on the shared host)?
7. **SQ-5 (carried, lowered)** — "scored before started" vs a P-002 refusal of `fw bvp estimate` on a
   `captured` task. **Third data point this round:** `fw bvp estimate` + `estimator.py cost-one` wrote
   to three `captured` tasks (T-3215, T-3213, T-3214) without any refusal. R2's measurement said the
   same. The premise in T-3037's body now has 0 reproductions against 4 counter-measurements; unless
   someone can name the path that refuses, this SQ can be closed as "not reproduced".


## Gates that refused, and what was done instead

0. **Self-inflicted, not a gate — recorded for the audit.** The T-3215 commit `7a38fceaf` also
   recorded four unrelated active task files (T-2723, T-3211, T-3213, T-3214) as DELETED: a
   `git rm --cached` meant to unstage them staged their deletion instead. Files never left disk.
   Restored in `20790e959` with T-2723/T-3211 at their pre-error blobs and T-3213/T-3214 with
   R2's on-disk estimator output. Lesson: never use `git rm --cached` to unstage; `git restore
   --staged` is the verb.
1. **T-1730 focus-drift gate** (expected, not hit): the T-3135 repair and both task-close commits
   used the `FW_SWITCH_FOCUS=1` per-command prefix form R2 documented; logged Tier-2 in
   `.gate-bypass-log.yaml`.
2. **Bash tool guard** (harness, not fw): refused `sleep 90` chained with a check ("use Monitor /
   run_in_background"). Did: a background `until` loop — which then never exited because `pgrep -f`
   matched its own shell; then a foreground wait with a bounded timeout. Both false starts are in the
   ledger (checks 11–12).
3. **No fw gate refused any verb this round.** G-020 did not fire because real ACs were written
   with the Edit tool BEFORE any source edit on all three tasks (R2's documented unblock).

## Findings for the operator (not decided here)

- **F7 — `find_termlink_bin_fresh()` costs >10 min per commit on this host.** `build.rs` re-runs
  on every `.git/logs/HEAD` change (T-1057, correct for version freshness) and this repo receives
  commits from several concurrent sessions, so the nested `cargo build -p termlink --release` inside
  the parity suite is almost never a no-op. Measured: 12m29s top-level; two 580–590s timeouts; the
  same test 0.25s with `TERMLINK_BIN`. R2's 498s full-suite time was this, not `ENV_LOCK`. CI (T-2686)
  pays it on every push. Candidate remediation for T-2748's backlog: replace the nested build with a
  freshness ASSERTION (`bin -nt sources`) that fails loudly, so a stale binary is a red test rather
  than a 10-minute rebuild.
- **F8 — the five existing subprocess-backed MCP tools have no in-process test, structurally.**
  `current_exe()` inside the in-process harness is the test binary. T-3214 surfaced it; it applies to
  `fleet_bootstrap_check`, `substrate_status`, `fleet_secrets_audit`, `fleet_reauth` and the T-1836
  script family too — their parity is un-assertable by the same mechanism.
- **F4 (carried, confirmed again)** — `lib/context_tokens.py` printed 0 for this worker's transcript at
  158K, 265K, 310K and 337K raw. Raw `cache_read_input_tokens` of the latest usage entry was the gauge.
- **F9 — HEAD carried a `completed/` task at `status: captured`** (T-3135) for ~10 hours after R1's
  close; the T-2290 canary reads the working tree and could not see it. A `git show HEAD:` variant of
  that check (or running it in CI on a clean checkout) would have.


## Cost-vs-estimate deltas

- **T-3215** (est. cost 2.0 after scoring): source work ≈ zero (R2's uncommitted edit was correct);
  the whole cost was VERIFICATION — one release build 12m29s + a 590s-timeout false start + a second
  580s-timeout false start before the `TERMLINK_BIN` form ran in 0.25s. ~70K context tokens
  (~265K → ~335K including unit 2). Calibration: on this host any AC whose check needs a release build
  should be costed at +1 tier regardless of code size; "effort=8 (lines=303)" measured the task
  body, not the build.
- **T-3213** (est. cost 3.2, blast_radius=3 from 2 components): actual ≈ one `match`, a debug build
  (46s), one pair run. Over-estimated ~2×: the second component (`parity.rs`) was a one-attribute
  removal. A test-file component should not count as a full blast-radius unit when the change is an
  `#[ignore]` flip.
- **T-3214** (est. cost 3.2): not executed. The estimator's blast_radius=3 is the cost of option 3
  (duplicate); option 1 is a crate-boundary refactor the components list cannot express. Cost is
  option-dependent, which is itself an argument for AC0 before scoring is trusted.
- **Unit 0 + the staging error**: ~15 min and two commits of pure overhead caused by one wrong git
  verb. Not an estimate delta; a process cost recorded so the lesson (never `git rm --cached` to
  unstage) survives.


## Checks recorded (traceability ledger)

| # | check | result |
|---|---|---|
| 1 | own-transcript usage entry at start | cache_read 158,890 (~20%); `context_tokens.py` printed 0 (F4) |
| 2 | `git log` + run record + R2 canonical handback re-read | HEAD `d33a730db`; R2 = attempt 2 file |
| 3 | `git show HEAD:.tasks/completed/T-3135-*.md` status line | `status: captured` in HEAD (finalize content never committed) |
| 4 | `FW_SWITCH_FOCUS=1 fw git commit` (T-3135 file) | `d09fdc3b2` |
| 5 | `grep -n 'fn find_termlink_bin' crates/termlink-test-utils/src/lib.rs` | `find_termlink_bin_fresh` exists (T-1928) — R2's edit compiles |
| 6 | `git log -S'chat-arc ONLY' -- tools.rs` | `7a114d1e6` T-3199 2026-09-28 17:42 (after the release binary R2 first compared) |
| 7 | `fw bvp estimate T-3215` / `estimator.py cost-one T-3215` | D1=4 D2=0 D3=3 D4=3 (no-change); blast_radius=1 tier=2 effort=8 → BVP 60 cost 2.0 Q1 |
| 8 | `fw work-on T-3215` | captured → started-work, focus T-3215 |
| 9 | 4 build-independent Verification lines under `bash -c 'set -eo pipefail'` | 4/4 PASS |
| 10 | `cargo build -p termlink --release` (foreground) | 12m29s wall; binary 09:01 embeds `d09fdc3b2` |
| 11 | `cargo test … parity_help` with nested fresh build | did NOT finish in 590s (nested `cargo build --release` > 8 min) — F7 |
| 12 | top-level `cargo build --release` re-run | did NOT finish in 580s (build.rs re-runs on every `.git/logs/HEAD` change; a foreign commit `2726e37a5` landed 08:49) — F7 |
| 13 | `test target/release/termlink -nt tools.rs && -nt help.rs` | FRESH |
| 14 | `TERMLINK_BIN=… cargo test -p termlink-mcp --test parity parity_help` | `ok. 1 passed; 0 ignored` in 0.25s |
| 15 | `bash scripts/check-mcp-parity-census.sh` | clean: 33 asserted, 227 acknowledged, 0 unexamined (12.6%) |
| 16 | `fw task verify T-3215` | P-011 7/7 PASS |
| 17 | `fw task update T-3215 --status work-completed` | completed; episodic mapping |
| 18 | `FW_SWITCH_FOCUS=1 fw git commit` (T-3215) | `7a38fceaf` — ALSO wrongly deleted 4 task files (my `git rm --cached`) |
| 19 | `git ls-tree HEAD .tasks/active/` for T-2723/T-3211/T-3213/T-3214 | 0/0/0/0 in HEAD after `7a38fceaf`; restored → `20790e959`, 1/1/1/1 |
| 20 | `fw bvp estimate T-3213` / `cost-one T-3213` | no-change D1=4 D2=0 D3=3 D4=3; blast_radius=3 tier=2 effort=8 → BVP 60 cost 3.2 Q1 |
| 21 | `fw work-on T-3213` | captured → started-work |
| 22 | `cargo build -p termlink --quiet` (debug) after fix | rc 0 in 46s |
| 23 | `target/debug/termlink agent search needle --json` (empty runtime dir) | stdout `{"error":"Hub is not running …","ok":false}`, rc 1; human path unchanged (stderr chain, rc 1) |
| 24 | `TERMLINK_BIN=target/debug/termlink cargo test … parity_agent_search_no_hub` | `ok. 1 passed; 0 ignored` |
| 25 | `bash scripts/check-silent-exit.sh` | clean, 39 scanned, 0 firing |
| 26 | `fw task verify T-3213` | P-011 6/6 PASS |
| 27 | `fw task update T-3213 --status work-completed` | completed; episodic mapping |
| 28 | `FW_SWITCH_FOCUS=1 fw git commit` (T-3213) | `2ed0583a6` (4 files; rename staged by the verb) |
| 29 | CLI vs MCP doctor check inventory (debug bin, empty runtime dir; `tools.rs:13143-13350` read) | CLI 11 checks incl. ufw_listener/secret_cache/secret_cache_profiles (+inbox when hub up); MCP 8 + `strict` |
| 30 | `parity.rs::mcp_client` read | in-process `TermLinkTools::new()` — `current_exe()` in tests = test binary |
| 31 | `grep substrate_status|bootstrap_check|…` in parity.rs + mcp_integration.rs | 0 hits — no subprocess tool has an in-process test (F8) |
| 32 | `fw bvp estimate T-3214` / `cost-one T-3214` | no-change; blast_radius=3 tier=2 effort=8 → BVP 60 cost 3.2 Q1; left `captured` (SQ-8) |
| 33 | arc census: `.context/arcs/*.yaml` IDs ∩ `.tasks/active` × Q1/Q2 lists × `tags`/`owner` | no agent-eligible Q1/Q2 outside SQ-gated items; T-3093 / T-3140 `tags: []` |
| 34 | `fw task verify T-3093` (read-only probe) | 3/3 PASS, 6/6 ACs ticked, Evolution template-only, arc-less |
| 35 | own-transcript usage at stop | cache_read 336,752 (~42%) |

```
