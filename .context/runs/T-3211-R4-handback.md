# T-3211 — procAsFit round 4 of 9 — handback

**Status: COMPLETE — stopped by choice at ~60% budget (478K of 800K; `TOKEN_WARN` = 75%) with **16 tasks closed by verb** (all 16 verified finalized in `HEAD`: `status: work-completed`, `date_finished: 2026-09-29`, none left in `active/`), 1 parked on a new SQ (T-2988), 1 analysed and left open (T-3228), and 14 new tasks created (T-3216..T-3229). Remaining agent-workable units exist (listed below), so this is NOT a "nothing left" stop. I stopped to leave room for an accurate handback, not because a stop condition fired.**
Worker: `claude -p` (Opus 5.5), started 2026-09-29. Transcript `~/.claude/projects/-opt-termlink/e0f760f2-….jsonl`
(confirmed mine: `entrypoint: sdk-cli`, `lastPrompt` = this mandate).

## Baseline at run start
- HEAD `273de18cd`. Focus T-3211. Budget: own transcript latest cache_read 159,852 (~20% of 800000). Stop threshold by name: TOKEN_WARN (75%).

## Selection

**Census finding that opened R4's work.** R3's census read only the quadrant lists (`fw bvp --quadrant …`), and those **exclude cost-less tasks**. arc-009 carried ~17 agent-owned, `captured`, BVP 57–71 tasks with `components: []`, so they had no cost and appeared in no quadrant. They are not SQ-gated; they were unplaced. Scoring one needs `components:` (the estimator returns `blast_radius=None` otherwise), which means reading its finding in `docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md` first.

**Unit 1 — arc-009, T-3008 (lv-lc / Q3, BVP 36 / cost 1.5 after scoping).** Objective: the arc-009 headline (every non-KEEP value-review finding reaches a terminal state). Picked as the most concrete finding (C-44: a red fixture suite). It scored 57 before scoping; once its body carried the real, smaller scope, the estimator placed it at 36, which is **Q3**. Worked under the operator's R4 directive ("in-flight arcs' Q3/Q4 work once Q1/Q2 are gated"), labelled Q3 honestly, not rescored.

**Unit 2 — T-3216 (new, Q1 hv-lc, BVP 69 / cost 1.7).** T-3008's evidence showed Doc Lint has 0 green runs in 300 and v0.12.0 was never built. Filed as its own task (one lock at a time). The deliverable is a census plus an options list: prerequisite work that makes SQ-9 cheap to rule on. No tags: it came out of an arc-009 finding, but assigning arc membership is not mine to do (SQ-4 shape).

**Unit 3 — arc-009, T-3002 (Q2 hv-hc, BVP 57 / cost 4.3 after scoping).** The next arc-009 finding with a measurable remaining gap (C-39). Scoped to the undelivered half only.

## Objectives advanced

Against run start (HEAD `273de18cd`). Every close went through `fw task update --status work-completed` with P-010 + P-011 green (`fw task verify`), and each episodic parses to a mapping.

| task | arc | quadrant (BVP/cost) | outcome | commit |
|---|---|---|---|---|
| T-3008 | arc-009 (C-44) | Q3 lv-lc (36/1.5) | Fixed: cron-drift fixture case 9 read the live host's `/etc/cron.d`; skipped when `CI` is set. Root cause of a suite red on every GitHub run | `3d0b93598` |
| T-3216 | none (new) | Q1 (69/1.7) | Census of the 26 CI-red guard members → `docs/reports/T-3216-ci-guard-layer-red-census.md`. 18 are runner artifacts (10 from depth-1 checkout), 8 are real. **v0.12.0 never built** | `6d4cdf772` |
| T-3002 | arc-009 (C-39) | Q2 (57/4.3) | 5 canaries date each firing `--quiet` entry (`=== <ts> ===`); heartbeat half cited as already delivered (T-2842/2843). New suite `canary-entry-date-fixtures.sh` 10/10; 4 fail on the old scripts | `184cc0ab0` |
| T-2983 | arc-009 (C-18) | Q1 (69/3.1) | `run-guard-layer.sh --jsonl PATH` streams each member's verdict + full output as it completes; a SIGKILLed run keeps finished verdicts. Fixtures 50→57; 7 fail on the old runner | `b0f859c20` |
| T-2997 | arc-009 (C-21) | Q1 (69/1.8) | No build needed: the staleness surface already existed (audit D2 since 2026-04-12, handover footer, `fw review-queue` with AGE). G-008 refreshed with measured figures | `25250a4ce` |
| T-3032 | arc-009 (C-45) | Q1 (74/3.1) | CLI verbs recorded in `invocation_audit` (`SURFACE_CLI` had 0 call sites); names only, never values. 3 unit tests; 1154/1154 CLI tests pass | `59c4a1966` |
| T-3033 | arc-009 (C-30/31) | Q1 (60/3.0) | Session-daemon `kv.*`/`session.*` recorded (surface `session-rpc`). `invocation_audit` + the shared capped-append writer moved to `termlink-session`, re-exported from hub (one write path). Live-proven on a scratch session | `afcd56926` |
| T-3217 | none (follow-up) | Q1 (57/2.0) | `invocation-usage.sh` SCOPE text no longer under-claims its coverage | `1026a561a` |
| T-3105 | arc-008 | Q1 (57/3.2) | Triage of 5 "unclosed-satisfied" tasks: none agent-closeable (2 already closed, 3 `owner: human`). T-212's verification fails: **Homebrew tap repo 404s** | `097d5d13d` |
| T-2999 | arc-009 (C-26) | Q1 (60/3.0) | `scripts/check-mcp-cli-twin-drift.sh`: one-sided changes to established `_mcp`/CLI twin pairs over a commit range. 3230 commits → 29 hits. Hermetic fixtures 7/7, mutant-pinned | `0d8ebca80` |
| T-3218 | none (found by T-2999) | Q1 (60/2.0) | Real drift fixed: 4 MCP history parsers silently dropped bad-`ts` rows (T-2619/T-2621 had fixed only the CLI twins). 4 tests were `(1,0)` before the fix; termlink-mcp lib 930/930 | `cc2f523fb` |
| T-3220 | arc-009 (C-15) | Q1 (57/3.2) | Forever-archival canary gains a records/day trigger (offset-0 ts as time base; no state file). First fixture suite for this canary 13/13, 7 fail on the old script. **Live: it now fires daily on `health:ring20-fedprobe` (91.6/day) until SQ-10 is ruled** | `c7986cb0d` |
| T-2988 | arc-009 (C-14) | — | Investigated read-only (producer = peer ring20 process, zero consumers); evidence in its Context. **Parked `captured` on SQ-10** | `c7986cb0d` |
| T-3219 | none (from T-2999) | Q1 (72/1.7) | Triage of the 29 twin-drift hits → `docs/reports/T-3219-twin-drift-triage.md`: DRIFT 8 (3 fixed by T-3218), LEGIT 5, MISATTRIBUTED 16, plus 4 more gaps in neighbouring fns. Read-only sub-agent wrote it; I re-verified the row count, tallies and one DRIFT claim at HEAD. 9 gaps filed as **T-3221..T-3229** | `4a3c59e3d`, `c3fbf8949` |
| T-3221 | none | Q1 (60/2.0) | MCP fleet presence walk: per-hub fetch now bounded at 8s (a silent hub hung it; CLI had T-2659) | `c379b107c` |
| T-3229 | none | Q1 (60/2.0) | MCP ack-status frontier counts content only (phantom lag ≥1; CLI had T-2838). 2 unit tests | `b0feb8efb` |
| T-3223 | none | Q1 (60/2.0) | MCP transport-level auth failure names `fleet doctor`/`fleet reauth` (CLI had T-2625). 1 unit test; mcp lib 933/933 | `31e664ab8` |
| T-3228 | none | unscored | Analysed, not started: confirmed at HEAD, and the tempting fix in `fetch_recent` would regress presence reads. Fix shape recorded in its Context | `f5c69d604` |

arc-009 headline: 8 more value-review findings reached a terminal state (C-44, C-39, C-18, C-21, C-45, C-30/31, C-26, C-15), and C-14 is investigated and parked on SQ-10 reached a terminal state the operator can see.

## Sovereign questions

1. **SQ-9 (new, highest) — the CI guard layer has been red on every GitHub run (0/300 Doc Lint green) and the v0.12.0 release never built.** `release.yml` for v0.12.0: `Workspace test suite + guard layer: failure` → build-linux / build-macos / release **skipped**; `gh release list` latest = **v0.11.2 (2026-06-24)**, so `install.sh` and Homebrew ship a 3-month-old binary. The mirror canary checks tag presence, not release publication, so it is quiet. T-3086 had written "NOT tagging v0.12.0" ~49 min before the tag was pushed. Measured options (T-3216 report): (1) `fetch-depth: 0` on the guard checkouts clears 10 members; (2) CI-aware skips/fixes for 7 environment-dependent members (T-3008 already did 1); (3) resolve or acknowledge the 8 real TREE findings; (4) re-run or re-cut the release, or make the guard layer non-blocking for release (reverses T-2686); (5) a canary for "tag without a published release" / "no green CI in N days". Agent recommendation, not decision: 1+2 are mechanical and restore the declared hermetic contract; 3–5 are yours. Also: **the Homebrew tap repo `DimitriGeelen/homebrew-termlink` does not exist (404)**, yet README:95 recommends `brew tap DimitriGeelen/termlink` for macOS. T-212 (`owner: human`) is the task that would create it. **Question: does the operator know v0.12.0 is unpublished and the tap is missing, and which options are authorised?**
2. **SQ-10 (new) — `health:ring20-fedprobe` is a write-only sink on the local hub (T-2988, C-14).** 2,563 records, `forever` retention, about 90/day, from one peer process (`proxmox-ring20-management` @ .122, `fed-probe` every ~15 min), and **zero consumers** (only receipt row: the producer's own, lag 2563, `up_to null`). Options: (a) `channel set-retention health:ring20-fedprobe --retention messages --retention-value N` + `sweep` on the live hub, which deletes records on shared infra; (b) ask the ring20 project to stop or re-target the probe (a peer decision; an outward-facing filing); (c) leave it and accept the growth. Evidence is in T-2988's Context; the task is parked `captured`.
3. **SQ-3, SQ-8, SQ-4, SQ-1/2, SQ-7, SQ-6** — carried from R3, unchanged, unruled.
4. **SQ-5** — fourth counter-measurement: this round, `fw bvp estimate` + `cost-one` wrote to 11 `captured` tasks (T-3008, T-3002, T-2983, T-2997, T-3032, T-3033, T-3216, T-3217 and others) with no refusal.

## Gates that refused, and what was done instead

1. **Task gate (P-002), several times** — refused reads/writes between tasks when focus had been cleared by a completion. Correct behaviour; re-focused on T-3211 through `fw context focus` for read-only census work.
2. **G-020 on T-3216** — refused a shell (heredoc) write to a task with placeholder ACs. Did: wrote ACs with the Edit tool first (R2's documented unblock).
3. **`fw inception start` under CLAUDECODE** — requires `--recommendation/--rationale` up front. I had no evidence yet, so I filed the census as a `test` task instead (a measurement, not a decision).
4. **`fw task create` without `--description`** — non-tty refusal. My own `| grep` had hidden it at first; re-ran with the flag. Recorded because a filtered pipe made a refusal look like silent success.
5. Self-inflicted: `pkill -f <pattern>` killed my own shell (pattern in its command line). Killed the probe by PID instead.

## Findings for the operator (not decided here)

- **F10 — R3's census method hid ~17 placeable tasks.** `fw bvp --quadrant` excludes cost-less tasks, and arc-009 had ~17 agent-owned BVP 57–71 tasks with `components: []`. Next rounds should census by arc membership × status, not by quadrant list.
- **F11 — value-review findings were partly stale at filing.** C-21's surface (D2) predates the review by 5 months; C-39's heartbeat half and C-18's profiling half shipped after the review but before these tasks were worked. Three of six closes this round were "already delivered + cite". Scoping before building paid for itself.
- **F13 — a guard-layer member mutates tracked repo state.** `scripts/voi-prompt.sh` (`# guard-layer: source --check --quiet`) increments `runs:` in git-tracked `.context/checks/voi-decisions.yaml` on every `--check`. So every local `run-guard-layer.sh` run advances the human's voi snooze counter, and my T-3216 census bumped it 3→4. I reverted my bump. A `source` member is supposed to be side-effect-free.
- **F14 — twin drift is systemic, not incidental.** 29 one-sided changes to established `_mcp`/CLI pairs across 3230 commits; 8 real drifts plus 4 in neighbouring fns. 5 are fixed this round (T-3218/T-3221/T-3223/T-3229, plus the T-3218 trio); 6 are open (T-3222/T-3224..T-3228). Whether to revisit T-2069 (extract shared fns instead of duplicating) is the sovereign alternative C-26 named.
- **F12 — `termlink register` outlived `timeout 25`** (SIGTERM did not end it; the timeout process was still waiting). Observed once on a scratch session and not investigated.

## Arc state

Census at stop (arc tag × owner × status × `fw bvp --include-proposed`):

- **arc-009** (value-review execution), agent-owned, still open: **T-3214** Q1 60/3.2 (SQ-8), **T-2978** Q1 57/2.0 (SQ-6), **T-3010** Q1 57/3.2 (intentionally open), **T-3006** 70/3.7 started-work inception (not touched this round), **T-2988** parked (SQ-10), and **unscored** (no `components:`): T-2980 (C-09 claim-verb identity binding: rejects live callers, T-2700 shape, likely sovereign), T-2981 + T-3000 (vendored, need upstream filings = outward, SQ-7 shape), T-2985 (C-19 auto-file tasks from canaries: a new autonomous task-creation mechanism, likely sovereign), T-2987 (C-03, DELETE-class, gated), T-2990 (C-05: arc-005 close is human-approved via `fw arc review`; outlier-shortening half is agent work), T-3009 (C-38: needs fw-side telemetry that doesn't exist). **Closed this round:** T-3008, T-3002, T-2983, T-2997, T-3032, T-3033, T-2999, T-3220.
- **arc-008** (audit/doctor remediation): **closed** T-3105 this round. Open agent-owned: T-3132 (85/2.0), T-3128 (85/4.4), T-2958 (57/3.2), T-3130 (57/2.0), all SQ-3; T-3103 (63/2.0) SQ-1/2; **T-2957** (57, unscored: seven PreToolUse hooks matched Write|Edit only, so a Bash heredoc can tick Human ACs. Widening the matchers changes `.claude/settings.json` enforcement and needs an enforcement-baseline refresh, which I treated as a governance change and did not touch).
- **arc-011**: T-3211 (this orchestration), T-2389 (`owner: human`). Unchanged.
- **reliable-comms**: arc is **closed**; T-3063/T-3064 (agent-owned, captured) are leftovers of a closed arc, so the ladder can't reach them (SQ-4 shape).
- **Arc-less, filed this round** (SQ-4 shape; each came straight out of an arc-009 finding): T-3216, T-3217, T-3218, T-3219, T-3221, T-3223, T-3229 closed; **T-3222, T-3224, T-3225, T-3226, T-3227, T-3228 open and unscored.**

## What remains in Q1/Q2 (or likely Q1 once scored), per task, with the reason not done

| task | state | reason not done this round |
|---|---|---|
| T-3228 | analysed, captured | fix is a local incremental-walk rewrite of the MCP ack-wait loop. Not started this late (stop by choice); analysis recorded |
| T-3224 | captured, unscored | MCP find-idle-history `kind` filter (feature parity). Not reached |
| T-3222 / T-3226 / T-3227 | captured, unscored | MCP agent-contact routing/fp/preflight parity (T-2386/T-2384/T-2385 not ported). Larger, and they touch the same function: serialize them, one at a time |
| T-3225 | captured, unscored | CLI cert-change error wording (low severity) |
| T-2990 (half) | captured | outlier-shortening half is agent work; the arc-close half needs human approval |
| T-3214, T-2978, T-3132/T-3128/T-2958/T-3130, T-3103, T-2988 | parked | SQ-8, SQ-6, SQ-3, SQ-1/2, SQ-10 respectively |
| T-2980, T-2985 | unscored | my read: sovereign (auth refusal of live callers; autonomous task creation). Not asked formally; flagged here |

## Cost-vs-estimate deltas

- **"Already delivered" closes cost a fraction of their estimate.** T-2997 (est 1.8) and the scoped halves of T-3002/T-2983 were mostly citation work, because the value-review finding had been overtaken by later tasks (F11). Calibration: when scoping shows half the finding shipped, re-run `cost-one` after narrowing (I did; T-3008 dropped 57→36 BVP).
- **T-3033** (est 3.0): about 1.5× estimate. The scoping missed a second cross-crate dependency (`append_line_capped`), so the move grew to two modules. Scoping by reading `use`/`crate::` references, not only the module header, would have caught it.
- **T-3216** (est 1.7): the local re-run of 26 members plus a depth-1 clone cost more wall time (about 3 min) than tokens; accurate.
- **Rust fixes (T-3218/T-3221/T-3229/T-3223, est 2.0 each):** at or below estimate, **because I used debug `cargo test -p termlink-mcp --lib` (2s) and never the release/parity build** that cost R3 over 10 min per commit (F7). Calibration: MCP pure-helper fixes are cost 2 when tested at lib level.
- **T-2999** (est 3.0): about right. One E2BIG false start (env var carrying 3000 SHAs).
- **T-3219** (est 1.7): about 225K sub-agent tokens against about 10K of mine for verification. Delegation kept my budget flat; the estimator does not see sub-agent cost at all.
## Checks recorded
| # | check | result |
|---|---|---|
| 1 | own transcript id + usage | e0f760f2, 159,852 cache_read (~20%) |
| 2 | `git log -8` + run record tail | HEAD 273de18cd; record says stopped at R3, operator directive overrides for R4 |
| 3 | `fw bvp --quadrant` vs arc-tag census | ~17 arc-009 agent tasks cost-less → absent from quadrant lists (F10) |
| 4 | `bash tests/cron-drift-firing-fixtures.sh` (host) | 13/13 green; CI run 36533229835 FAIL |
| 5 | `CRON_DRIFT_INSTALLED_DIR=<empty> check-cron-install-drift.sh` | rc 1, 31 MISSING (reproduces CI) |
| 6 | `gh run list --workflow doc-lint.yml --limit 300` | 0 successes |
| 7 | `gh run view` release v0.12.0 jobs; `gh release list` | test+guard failure → build/release skipped; latest v0.11.2 |
| 8 | 26 CI-red members run locally / in depth-1 clone with CI=true | 8 red locally; 13 of 18 CI-only reproduced in clone |
| 9 | `canary-status.sh --json` | stale-waker/stuck-claims/hook-counter/doorbell HEALTHY; waker-liveness FIRING |
| 10 | canary fixture suites after T-3002 | hook-counter 33/0, stale-waker ALL PASS, stuck-claims 28/0, doorbell 7/0, new 10/0; 4 fail on old scripts |
| 11 | `guard-layer-runner-fixtures.sh` after T-2983 | 57/0; 7 fail on old runner |
| 12 | audit D2 source + `fw review-queue` | D2 at audit.sh:5069 since 313e9eaea (2026-04-12); AGE column present |
| 13 | `cargo test -p termlink --bin termlink` (T-3032) | 1154/1154; live sink record `cli channel list` |
| 14 | `cargo test -p termlink-session --lib` / hub `rpc_audit` (T-3033) | 486/486, 41/41; live `session-rpc kv.set/kv.get` records |
| 15 | T-3105 verifies | T-1428 2/2, T-1451 8/8, T-212 7/8 (tap repo 404; `gh repo view` → not found) |
| 16 | `check-mcp-cli-twin-drift.sh --range HEAD~3000..HEAD` | 3230 commits, 67 pairs, 29 one-sided; fixtures 7/0, mutant fails 2 |
| 17 | T-3218 tests before/after fix | all 4 `(1,0)` before → 4/4 pass; mcp lib 930/930 |
| 18 | `check-forever-archival-freshness.sh` live after T-3220 | fires only on health:ring20-fedprobe 91.6/day; fixtures 13/0, 7 fail on old script |
| 19 | fedprobe subscribe + `channel ack-status` | single producer ring20 @.122; receipts lag 2563, up_to null |
| 20 | T-3219 report structure + resolve_contact twins at HEAD | 29 rows, tallies match; MCP fetch unbounded vs CLI timeout confirmed |
| 21 | mcp lib after T-3221 / T-3229 / T-3223 | 930 / 932 / 933 passed, 0 failed, no warnings |
| 22 | `git show HEAD:` for all 16 closed tasks | 16/16 finalized in HEAD, none in active/ |
| 23 | own transcript at stop | cache_read 478,115 (~60%) |
