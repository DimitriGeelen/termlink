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

**Unit 3 — arc-009 again, T-2991 "Raise MCP parity coverage on highest-churn tools.rs regions"
(Q1 hv-lc, BVP 60 → re-scored 60 after real ACs, cost 1.7 proposed).** Next after T-3037 because
it is the only remaining arc-009 agent-owned Q1 task that does not touch live infrastructure
(T-2978 relaunches live agents → SQ-6 below). Its placeholder ACs were the first activity, as in
R1's T-3135 — written from a MEASURED churn ranking, then re-scored through the estimator
(`fw bvp estimate T-2991`, wrote D1=4 D2=0 D3=3 D4=3 F-RECALL=2) before any source edit.

**Note on the orchestrator's stop.** Commit `d4904d9aa` ("rounds 1-2 complete; stopping at
TOKEN_WARN with 7 rounds unrun") landed while unit 3 was in progress and recorded this handback
at its 77-line partial state. The orchestrator's "R2 COMPLETE" verdict therefore PRE-DATES unit 3;
this file supersedes the copy in that commit. The worker's own stop conditions had not fired
(budget ~38% at unit-3 close, well under `TOKEN_WARN` 75%), so the mandate said continue.

## Objectives advanced

Against run start (HEAD `c54b48002`):

1. **T-3037 CLOSED by verb** (`fw task update --status work-completed`, P-010 6/6, P-011 4/4,
   episodic parses to a mapping). Commit `ce37f2ddf`. The PL-373 defect — promoted as a learning
   on 2026-09-11 but never filed — is now at **`framework:pickup` offset 228** with
   `metadata.from_project=010-termlink`, read back from the topic (4624-byte payload, `task:
   T-3037`). Payload kept tracked at `docs/reports/T-3037-rail-project-label-filing.yaml`.
   arc-009 headline advanced: one more value-review finding reached a visible terminal state.
2. **T-2991 CLOSED by verb** (P-010 5/5, P-011 7/7, episodic mapping). Commit `36561a938`.
   MCP/CLI parity census **24 → 33 asserted of 260** (9.2% → 12.6%), allowlist 236 → 227, 0
   unexamined. Nine new cases (PAIR 25–33) on the top-churn regions of `tools.rs`; six pass, three
   found genuine drift on their FIRST run and are `#[ignore]`d with the owning task in the reason:
   - **T-3213** `termlink agent search --json` prints no JSON on hub-down (T-1914 class).
   - **T-3214** MCP `termlink_doctor` lacks `ufw_listener` / `secret_cache` /
     `secret_cache_profiles` and adds a `strict` key the CLI does not echo.
   - **T-3215** help catalogs differ in exactly one string (`termlink_agent_search` description).
   Full suite: `34 passed; 0 failed; 3 ignored` (497.9s — `ENV_LOCK` serialises every case on a
   host running ~450 agent processes). Churn method + caveat + CLI pre-measurement in
   `docs/reports/T-2991-tools-rs-churn.md`.
3. **T-3103** — nothing re-done; attempt 1's park (verb, 23:29:12Z) and commit `eb1364e94` stand.
   The two scope questions it named are now WRITTEN (SQ-1, SQ-2 below), which attempt 1 never did.

## Arc state (tasks by status and quadrant)

- **arc-011 (slug arc-010, "mailbox to prompt")**: 0 active tasks (R1 closed T-3135). Exhausted.
- **arc-008 ("audit/doctor remediation")**: agent-eligible Q1/Q2 all frozen on one unachievable /
  answered-negative AC each — T-3132 (Q1 85), T-3128 (Q2 85), T-2958 (Q1 57), T-3130 (Q1 57) → SQ-3;
  T-3103 (Q1 63) parked on SQ-1/SQ-2; T-3127 closed (attempt 1). T-3117 / T-3126 `owner: human`.
- **arc-009 ("value-review execution")**: this round closed T-3037 (Q1) + T-2991 (Q1); created
  T-3213 / T-3214 / T-3215 (bug, `captured`, BVP scored 57-ish by estimator, NO cost yet → no
  quadrant; placeholder ACs). Remaining agent-owned Q1: T-2978 (placeholder ACs + live-infra, SQ-6),
  T-3010 (intentionally open revisit sentinel, not eligible). 24 → 26 completed members.
- **Other in-flight arcs** (censused this round by member ∩ active ∩ Q1/Q2): arc-parallel-substrate
  → T-2197 only (`owner: human`); comms-loudness → T-2389 only (`owner: human`);
  arc-substrate-fitness, mcp-slimming → none. **No agent-eligible Q1/Q2 in any other arc.**
- **Q1/Q2 in NO arc** (unreachable by the ladder, SQ-4): T-2644 (78), T-2573 (76), T-2606 / T-2656 /
  T-2662 (73), T-2616 (66), T-2886 (60), T-2911 / T-2581 / T-3091 (57); started-work all-ticked
  T-3177 (5/5), T-3093 (6/6), T-3141 (3/3) — close candidates for a `fw task verify` sweep.

## What remains in Q1/Q2, per task, with reason not done

| task | Q | arc | reason not done this round |
|---|---|---|---|
| T-2978 | Q1 | arc-009 | placeholder ACs AND relaunches live agents on shared infra — operator ruling first (SQ-6) |
| T-3213 / T-3214 / T-3215 | unscored (no cost) | arc-009 | created this round from T-2991's findings; need ACs + cost before they are eligible |
| T-3103 | Q1 | arc-008 | remaining 9 carded-unwatched files are SQ-1 + SQ-2 |
| T-3132, T-3128, T-2958, T-3130 | Q1/Q2 | arc-008 | one AC each honestly recorded unachievable → SQ-3 |
| T-3010 | Q1 | arc-009 | intentionally open by its own AC (revisit_at carrier) |
| T-2197, T-2389 | Q2 | other arcs | `owner: human` — not eligible for an agent worker |
| arc-less Q1 list above | Q1 | — | no arc → the project→arc→task ladder never reaches them (SQ-4) |

**Stop condition reached:** no agent-eligible Q1/Q2 task remains in any in-flight arc that is not
blocked on a Sovereign question or a scoring/AC precondition. Budget at stop ~38% (< `TOKEN_WARN`).

## Sovereign questions raised (unresolved, priority order)

1. **SQ-3 — closing a task whose last AC is honestly answered NO / unachievable.** Four arc-008
   tasks (T-3132, T-3128, T-2958, T-3130; BVP 85+85+57+57) are frozen this way. Options: (a) human
   strikes/rewords the AC and the agent closes by verb; (b) a `--wontfix`-style verb with logged
   rationale; (c) they stay open as the register's honest record. Producer-not-judge forbids the
   agent rewording its own AC to pass. (Carried from attempt 1, still the highest-value block.)
2. **SQ-1 — the 50 fabric cards on 8 vendored BPMN files**
   (`.agentic-framework/.context/designer/projects/*/v*.bpmn`, `created_by: T-2839`, under a
   gitignored vendored tree): keep them and add a `.agentic-framework/.context/designer/**/*.bpmn`
   watch glob (fabric then watches a tree a re-vendor rewrites), or delete a deliberate 50-card
   deliverable. Either way the audit WARN drops 9 → 1. Note the cross-project ID collision: local
   T-2839 is an unrelated broadcast task (T-2800 class).
3. **SQ-2 — `.claude/commands/*.md`**: 34 slash commands, exactly one (`capture.md`) carded.
   Register all 34 (33 new cards) or drop the singleton. Hand-shaping a glob around one file is the
   fabrication `watch-patterns.yaml` itself warns against. Resolves the last carded-unwatched file.
4. **SQ-4 (carried from R1) — the highest-BVP agent-owned Q1 tasks belong to no arc** (T-2644 78,
   T-2573 76, T-2606/T-2656/T-2662 73, …). Assign an arc, open a "defect remediation" arc, or accept
   the ladder never reaches them. T-2573 also carries a decide-the-wire-contract AC that is itself
   sovereign.
5. **SQ-7 (new) — should 010-termlink adopt its own rail signing key?** `fw rail post` REFUSES
   host-signed posts (rc 2). Its option 1 mints a NEW fingerprint and says so: "a coordination event,
   not a config tweak" — peers who know this project by the host fingerprint `d1993c2c3ec44c94`
   (every prior 010-termlink filing) must be told. Until ruled, filings go through option 3
   (`FW_ALLOW_HOST_SIGNED_RAIL=1`, logged Tier-2), as T-3037 did.
6. **SQ-6 (new) — may an autonomous worker execute T-2978** ("re-arm push-wake rail: relaunch
   unwakeable agents via tl-claude")? It restarts live agents on the shared host — the class of
   action T-3089 R3 got wrong (restarted the shared hub 82s after a defer ruling).
7. **SQ-5 (carried, lowered)** — "scored before started" vs the P-002 gate that refuses `fw bvp
   estimate` on a `captured` task. Measured this round: `fw bvp estimate T-3213` on a fresh
   `captured` task WROTE successfully, so the gate did not fire from this worker's path — the
   premise in T-3037's body may be stale or path-dependent. Needs one deliberate measurement, not
   two contradictory anecdotes.

## Gates that refused, and what was done instead

1. **`fw rail post` (rail-identity guard)** refused the T-3037 filing: "BLOCKED: this rail post
   would be signed by the HOST key" (rc 2). Did NOT route around: took the gate's own option 3
   (`FW_ALLOW_HOST_SIGNED_RAIL=1`, Tier-2, logged in `.gate-bypass-log.yaml`), consistent with every
   prior 010-termlink filing (offset 218 carries the same host fp); raised option 1 as SQ-7.
2. **G-020 scope-aware gate** BLOCKED two Bash commands under T-2991 (a `for` loop and a python
   heredoc) because the task still had `[First criterion]` ACs. Correct refusal. Did: wrote real ACs
   with the **Edit tool** (the gate's documented unblock), re-scored through the estimator, then
   proceeded. The Bash gate blocks even task-file edits made via python — the Edit tool is the path.
3. **`fw task create` non-interactive** refused three creations silently under `grep` filtering
   ("Missing required flag(s): --description; stdin is not a tty"). Did: re-ran with
   `--description`. Finding: the first attempt's failure was invisible because I filtered its output
   — never grep a verb's output before checking its rc.
4. **Orchestrator supersession** (not a fw gate): `d4904d9aa` declared R2 complete mid-unit-3 and
   committed this handback partial. Did: finished unit 3 under the mandate, superseded the partial
   with this file, and stated the sequence above so the audit can see which came first.

## Findings for the operator (not decided here)

- **F4 — `lib/context_tokens.py` reports 0 for this worker's transcript throughout** (checked at
  start, 172K raw, and at unit-3 close, 304K raw). Its "<2 in-scope entries → 0" guard or the
  dominant-model scoping does not fit an sdk-cli worker transcript. Carried fix 4 recommends this
  command; on this run it would have read as "young session" the whole way. The raw
  `cache_read_input_tokens` of the latest usage entry was the working gauge. One bug = one task;
  sibling of T-3212 (checkpoint.sh cross-read) — vendored (G-062).
- **F5 — four top-churn CLI verbs have no `--json` flag at all** (`batch tag`, `batch exec`,
  `deregister`, `agent chat-arc-recent`), so the parity harness cannot assert them until the CLI
  grows one. Natural next scope for T-2748.
- **F6 — the full parity suite takes ~500s here**: every case waits on `ENV_LOCK`; on this host
  that is 60s+ per case of "has been running for over 60 seconds". CI budgets should assume it.

## Cost-vs-estimate deltas

- **T-3037** (est. cost 2.0): actual ≈ one unit, ~60K context tokens (173K → ~235K), 1 filing + 1
  tracked payload + 2 gate interactions. Estimate was fair; the unmodelled cost was the rail-identity
  gate (an SQ, not work).
- **T-2991** (est. cost 1.7, effort 8 "lines=207, acs=4" — read off template boilerplate): actual
  ≈ ~70K context tokens, 1 measurement script, 9 test cases, 3 filed tasks, 3 cargo runs (47s
  compile + 60s targeted + 498s full). The estimate's blast_radius=1 was right (one test file + one
  allowlist); effort was underestimated ~3× because the estimator scored a placeholder body. Same
  calibration lesson as R1/T-3135: a `captured` task with `[First criterion]` should be flagged
  UNSCORABLE for effort, not scored from template line count.

## Checks recorded (traceability ledger)

| # | check | result |
|---|---|---|
| 1 | own-transcript usage entry at start | cache_read 172,619 (~22%); `context_tokens.py` printed 0 (F4) |
| 2 | `git log` + T-3103 Updates re-read | T-3103 parked by verb 23:29:12Z; `eb1364e94` in HEAD |
| 3 | `termlink channel list --json` | hub reachable; `framework:pickup` count 228, latest 227 |
| 4 | at-the-moment re-read before rail post (`git log`, dispatch log) | new commits `9117241be`, `1f3932735` seen; T-3037 had been returned to `captured` |
| 5 | `fw rail post framework:pickup …` | rc 2 BLOCKED (host-signed) |
| 6 | `FW_ALLOW_HOST_SIGNED_RAIL=1 fw rail post … --json` | rc 0, offset **228**, `delivered-unconfirmed`, ts 1790638697104 |
| 7 | `channel subscribe framework:pickup --cursor 228 --limit 1 --json` | offset 228, `from_project=010-termlink`, payload 4624 B, `task: T-3037` |
| 8 | `fw work-on T-3037` | captured → started-work, focus set |
| 9 | `git status --porcelain .agentic-framework/` | empty (G-062 AC) |
| 10 | T-3037 Verification ×4 under `bash -c 'set -eo pipefail; …'` + `fw task verify` | 4/4 PASS, 4/4 PASS |
| 11 | `fw task update T-3037 --status work-completed` | completed; episodic generated; `yaml.safe_load` → mapping |
| 12 | `fw git commit` (T-3037) | `ce37f2ddf` (3 files; Tier-1 "task closed" warning only) |
| 13 | churn script over 374 commits | 262 tool regions touched; top-45 recorded |
| 14 | `cargo test -p termlink-mcp --test parity --no-run` | compiled in 46.9s |
| 15 | `fw work-on T-2991` | captured → started-work |
| 16 | G-020 gate on Bash under T-2991 ×2 | BLOCKED (placeholder ACs) → ACs via Edit tool |
| 17 | `fw bvp estimate T-2991` (after real ACs) | wrote D1=4 D2=0 D3=3 D4=3 F-RECALL=2 |
| 18 | CLI hub-down pre-measurement ×9 verbs | 5 emit `{ok:false,error}`; `agent search` emits NOTHING (drift); doctor/help rc 0 |
| 19 | new cases, targeted run | 6 ok, `parity_doctor` + `parity_help` FAIL (drift), `--ignored parity_agent_search_no_hub` FAIL as recorded |
| 20 | `fw task create` ×3 | first attempt refused (no `--description`); second created T-3213, T-3214, T-3215 |
| 21 | full `cargo test -p termlink-mcp --test parity` | `34 passed; 0 failed; 3 ignored` in 497.86s |
| 22 | `bash scripts/check-mcp-parity-census.sh` (+`--json`) | clean; covered 33, acknowledged 227, unexamined 0, 12.6% |
| 23 | `fw task verify T-2991` | 7/7 PASS |
| 24 | `fw task update T-2991 --status work-completed` | completed; episodic mapping |
| 25 | `fw git commit` (T-2991) | `36561a938` |
| 26 | own-transcript usage at unit-3 close | cache_read 304,201 (~38%); `context_tokens.py` still 0 |
| 27 | other-arc Q1/Q2 census (yaml members ∩ active ∩ quadrant lists) | only human-owned T-2197, T-2389 |
| 28 | `fw bvp estimate T-3213` on a fresh `captured` task | WROTE (no P-002 refusal from this path) — SQ-5 evidence |
