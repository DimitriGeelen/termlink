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

