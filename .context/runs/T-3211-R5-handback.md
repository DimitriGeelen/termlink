# T-3211 — procAsFit round 5 of 9 — handback

**Status: COMPLETE. Stopped because a recorded census found no ungated agent-workable Q1/Q2 unit left** (stop condition 1 as the operator's R5 directive frames it; census in § Census). Budget at stop: ~320K of 800K (~40%), below `TOKEN_WARN` (75%).
**8 tasks closed by verb** (7 with P-011 green and committed; T-3231's finalize is NOT committed, see ⚠ below), 2 new tasks filed and closed (T-3230, T-3231), 1 parked (T-3227), 1 dispatcher defect fixed (F15).
Worker: `claude -p` (Opus 5.5), 2026-09-29. Transcript `~/.claude/projects/-opt-termlink/97b53dfa-….jsonl` (confirmed mine: `entrypoint: sdk-cli`, contains "round 5 of 9").

## ⚠ FIRST — HEAD carries a G-066-class defect I introduced (needs one human-authorised commit)

My commit `47201ad2a` (T-3211, dispatcher fix) swept in an index entry `update-task.sh` had staged: T-3231's `active→completed` **rename with the pre-completion content**. So **HEAD has `.tasks/completed/T-3231-*.md` with `status: started-work`, empty `date_finished`**.
- **Measured:** `check-task-finalization-freshness.sh --tasks-dir <git archive HEAD>` → **rc 1, `[started-work] T-3231`**. The working tree is correct (rc 0): the file there says `work-completed` with `date_finished`, and `.context/episodic/T-3231.yaml` exists untracked. **Any clean clone or CI checkout of HEAD sees the defect**, and the task-finalization canary will fire on it.
- **Why I did not fix it:** the commit targets completed task T-3231 while focus is T-3211. The T-1730 gate only offers a logged Tier-2 bypass for a non-active task, and Tier 2 is a human-authorisation tier. I did not self-grant it.
- **Repair (one line, needs your Tier-2 OK):**
  `cd /opt/termlink && git add .tasks/completed/T-3231-flaky-cli-test-dispatch-isolaterejectsno.md .context/episodic/T-3231.yaml && FW_SWITCH_FOCUS=1 .agentic-framework/bin/fw git commit -m "T-3231: finalize (work-completed, episodic)"`
- **Root cause of my error:** `fw git commit` commits the whole index, and `fw task update --status work-completed` stages the rename. The six earlier closes were safe only because I committed each finalize immediately. Here the finalize commit was refused first, the rename stayed staged, and the next unrelated commit picked it up. Next rounds: run `git status --porcelain` before any commit that follows a refused finalize.

## Baseline at run start
- HEAD `83d8c6ede`. Budget 160,717 (~20%). 42 unpushed commits at stop (see SQ-9).

## Selection

Objective throughout: comms correctness and substrate parity (charter verbs 2 and 4). Candidates were the six open twin-drift gaps R4 left (T-3222/3224/3225/3226/3227/3228), all arc-less and unscored.
- **Scored first (scored-before-started):** each got `components:` so the estimator had a blast radius, then `fw bvp estimate` + `estimator.py cost-one`. T-3228/3222/3224/3226/3227 → **60/2.0 Q1**; T-3225 → **57/2.0 Q1**. I ran the estimator inline, not via the TermLink `bvp-estimator` worker: it takes 0.05s per task, and a dispatch would add a hop without adding any audit value.
- **Order (the estimator does not separate them, so I ordered by failure severity):** T-3228 false `ack.received=false` (analysis already done) → T-3226 / T-3222 silent misdelivery (same fn, serialized) → T-3230 (target_fp half, filed on the way) → T-3224 missing filter → T-3225 misframed error text → T-3231 flake (found on the way).
- **SQ-4 interpretation, flagged for review, not ruled:** these tasks carry no arc tag. R4 and I treated them as **arc-009's own discovered work** (they come from finding C-26 via T-2999/T-3219), the T-3140 sub-case of SQ-4. I did **not** extend that to the older arc-less defects (T-2644, T-2573, …) that R3 recorded as SQ-4-gated. If the operator reads SQ-4 strictly, this round's 7 units were ineligible by the ladder even though each is a real, now-fixed defect.

## Objectives advanced
Every close went through `fw task update --status work-completed` with P-010 + P-011 green; each episodic parses to a mapping.

| task | quadrant (BVP/cost) | outcome | commit |
|---|---|---|---|
| T-3228 | Q1 (60/2.0) | MCP `ack_required` wait now walks by offset cursor (T-2507 port) via `walk_pages_mcp` + `ContactHub::walk_from`; `fetch_recent` untouched (presence safe). 3 tests; mutants A (no pagination) / B (cursor not carried) red; mcp lib 936/936; P-011 4/4. Not live-proven (needs a live peer ack on a swept topic) | `cc0861179` + finalize |
| T-3226 | Q1 (60/2.0) | MCP agent_contact prefers the peer's LIVE presence fp over the host registration fp for local peers (T-2384 port, `prefer_presence_fp_mcp`). 1 test (4 cases); inverted-precedence mutant red; lib 937/937; P-011 3/3. Checked: CLI `agent ping` also uses the reg fp, so MCP ping is NOT twin drift | `7542e8b74`, `4517fb6cd` |
| T-3222 | Q1 (60/2.0) | MCP agent_contact routes to the peer's DECLARED home hub (`metadata.addr`) on the name-miss and local-registered paths (T-2386 port); explicit `hub` still wins. 2 tests; "always read hub" mutant red; lib 939/939; P-011 3/3. target_fp half filed as **T-3230** | `4bec64f6a`, `494b424f0` |
| T-3230 | Q1 (60/2.0, filed + scored this round) | MCP agent_contact `target_fp` with no hub: fp-keyed fleet walk (bounded 8s/hub) routes to the fp's declared home hub (T-2386 half). `agent_id_for_fp_mcp` test (5 cases); drop-`.rev()` mutant red; lib 940/940; P-011 3/3. The network walk itself is neither unit-tested nor live-smoked | `b05dc1283`, `19091ab9c` |
| T-3224 | Q1 (60/2.0) | MCP find-idle-history gains a `kind` filter + `summary.kind_filter` (T-2208 port). Test mirrors the CLI's plus 2 extra cases; filter-removed mutant red; lib 941/941; P-011 3/3 | `f769f7de3`, `c06498f94` |
| T-3225 | Q1 (57/2.0) | CLI connect failure on cert change leads with the TOFU cause (twin of MCP T-2268) via pure `render_connect_failure`; other failures keep the exact old context; tests assert fleet doctor's real `classify_fleet_error` still classifies both. Pre-fix mutant red. RCA block added (bug-class gate). Full CLI suite 1156/1156 on rerun | `9b11e29dd`, `97e1f9390`, `60744d38c` |
| T-3231 | Q1 (57/2.0, filed + scored this round) | Flaky `dispatch::tests::isolate_rejects_non_git_dir` (failed 1 of 2 full runs; mutated CWD without `ENV_LOCK`) now holds the lock. 3 consecutive full runs 1156/1156, which is weak evidence alone; the fix rests on the mechanism. P-011 2/2. **Finalize committed WRONG — see ⚠** | `07fd573a9` |
| T-3227 | Q1 (60/2.0) | **Parked, not started.** Gated on T-2385's unticked human `[REVIEW]` AC (the WARNING wording this port would copy). Park note written to its Context but **not committed** (Gates #3) | — |
| (T-3211) | — | **F15 fixed:** the round prompt's budget-read instruction fed the transcript as an argument; `context_tokens.py` reads **stdin** (`argv[1]` is a timestamp), so it printed **0**. Now emits `… < ~/.claude/projects/-opt-termlink/<session-id>.jsonl` and says why. Measured on my transcript: arg form 0, stdin form 306,256 | `47201ad2a` (⚠ swept) |

Net: the **5 open DRIFT rows** of the T-3219 triage minus T-3227 (human-gated) are closed, plus 3 neighbouring gaps (T-3222's local path, T-3230, T-3228's adjacent gap). The twin-drift list from R4 is empty except T-3227.

## Sovereign questions (priority order; none ruled here)

1. **SQ-9 (carried, updated) — CI red, v0.12.0 unbuilt, tap 404.** New fact: **local `main` is 42 commits ahead of `origin/main`** (origin HEAD `2726e37a5`), so none of R4/R5's fixes, including R4's T-3008 CI fix, have reached CI. The last 5 Doc Lint runs are still `failure`, and the latest release is still v0.11.2. Pushing is outward-facing and not mine to do. **Question addition: push now so CI re-measures, or hold until SQ-9 is ruled?**
2. **⚠ T-3231 finalize (above)** — one Tier-2 authorisation for one commit.
3. **SQ-4 (carried, sharpened by this round)** — is "work discovered by an arc finding" (the T-3219 twin gaps from arc-009 C-26) arc-eligible? R4 and R5 acted on yes. The same ruling decides whether T-3091, T-3140 and the older arc-less defects (T-2644 78, T-2573 76, T-2656/T-2662 73, …) are reachable.
4. **T-3006 ready for decide** — inception complete (research artifact `docs/reports/T-3006-completion-rate-drop-investigation.md`, Recommendation **NO-GO**, Problem/Assumptions filled; Agent ACs auto-tick on decide). Only `fw inception decide T-3006 no-go` remains.
5. **T-2385 human `[REVIEW]` AC** — unblocks T-3227 (MCP reachability preflight port).
6. **SQ-10, SQ-3, SQ-8, SQ-1/2, SQ-7, SQ-6** — carried unchanged.
7. **SQ-5** — fifth counter-measurement: the estimator wrote to 7 more `captured` tasks with no refusal.

## Gates that refused, and what was done instead
1. **G-020 on T-3228** — refused a source edit while ACs were placeholders. Correct; I wrote real ACs first (same for every later task).
2. **P-002 task gate after each completion** — focus was cleared by the close; I refocused T-3211 via `fw context focus` (bare: the gate refuses the bootstrap when the same line carries a redirect or heredoc).
3. **P-002 "captured" gate on T-3227** — refused committing my park note to a `captured` task. The only ways past were `fw work-on` (which would record work started on a task I am deliberately NOT working) or a logged Tier-2 bypass (a human-authorisation tier). I took neither: the park note and scores sit **uncommitted** in `.tasks/active/T-3227-*.md`. The same gate is why I did **not** write T-2990's measurement into its file (it lives here instead, § Census).
4. **T-1730 focus-drift on T-3231's finalize commit** — I refocused T-3211 before committing the finalize, a completed task cannot be re-focused, and the only offered path is Tier-2. I did not self-authorise. **Consequence: see ⚠**, which is worse than the refusal itself.
5. **G-019 RCA gate on T-3225** — refused completion with an empty `## RCA` (the title contains "error"). Correct; I wrote the RCA and then closed. I did not use `--skip-rca`.

## Findings for the operator
- **F15 (fixed, `47201ad2a`)** — see the table. The first four rounds' prompts carried the same instruction. R4 recorded its budget from `cache_read` directly, so its figures stand. Any round that trusted the command would have read 0.
- **F16 — the RCA gate classifies "bug" from title keywords only** (`update-task.sh::check_rca_for_bugfix`: `fix|bug|error|fail|broken|crash|regression|…`). T-3225 needed an RCA because its title says "error". T-3231 ("Flaky … test") and T-3228 ("can miss the ack") are real bugs that passed with no RCA. Coverage depends on wording. The code is vendored (G-062), so this is upstream's to fix; not filed this round (outward, SQ-7 shape).
- **F17 — `fw git commit` + a staged finalize rename = silent sweep.** It caused the ⚠ defect. A pre-commit check that the commit's task ID matches every staged `.tasks/` path would have refused it. Candidate for a local guard.
- **F18 — T-2990 (C-05) "shorten remaining outliers" is low-value as measured:** 260 descriptions, 105,840 chars (~26.5k tokens); capping at 1000 / 800 / 600 saves 0.5% / 0.9% / 3.0% (≤ ~800 tokens). C-05's target (15–23k tokens) needs a 13–43% cut, reachable only through the human-gated C-01/C-02 deletions. The arc-005 close is human (T-2408 partial-complete; guard wired and passing: `test-mcp-desc-budget.sh` rc 0, max 1546 / ceiling 1560, total 105,844 / 109,000).
- **F12 (carried)**, **F13 (carried)** unchanged.

## Census (why I stopped)
- **146 agent-owned open tasks; 143 have a cost**, so R4's F10 unplaced set is effectively gone. The 3 without one: T-3211 (this), T-3191 (upstream amend: outward), T-3212 (vendored `checkpoint.sh`: upstream).
- **Q1 (hv-lc), agent-owned open: 23.** In an arc: T-3132/T-3103/T-2958/T-3130 (arc-008: SQ-3, SQ-1/2), T-3214 (SQ-8), T-2978 (SQ-6), T-3010 (intentionally open). Arc-less: T-3227 (human AC), orchestration records T-3089/T-3093, and T-2644, T-2573, T-2606, T-2656, T-2662, T-2616, T-3177, T-2886, T-2581, T-2911, T-3091, T-3139, T-3141, all SQ-4-gated per R3.
- **Q2 (hv-hc), agent-owned open: 6** — T-3128 (arc-008, SQ-3); T-2016, T-2669, T-2532, T-2398, T-2015, all arc-less (SQ-4).
- **Q3/Q4 in an arc, agent-owned open: 1** — T-2017 (arc:T-2013): its next AC redeploys and restarts the hub on remote host .141, which is live shared infra on another machine (SQ-6 shape).
- **arc-009 unscored remainder (R4's list):** T-2980 / T-2985 (likely sovereign: auth refusal of live callers; autonomous task creation), T-2981 / T-3000 (vendored, outward), T-2987 (DELETE-gated), T-3009 (needs non-existent fw telemetry), T-2990 (F18: close is human, outlier half low-value), T-3006 (ready for decide).
- **Conclusion:** with SQ-4 unruled, no ungated agent-workable Q1/Q2 unit remains. **If the operator rules SQ-4 "arc-less defects are eligible", R6 has ≥ 13 Q1 units ready** (T-2644 78 first). Otherwise the sequence should end.

## Arc state
- **arc-009:** unchanged membership; open agent-owned T-3214 (SQ-8), T-2978 (SQ-6), T-3010, T-3006 (ready for decide), T-2988 (SQ-10), unscored remainder above.
- **arc-008:** unchanged (T-3132/T-3128/T-2958/T-3130 SQ-3; T-3103 SQ-1/2; T-2957 enforcement change).
- **arc-011:** T-3211 (this), T-2389 (human).
- **Twin-drift follow-ups (arc-less, from C-26):** closed T-3222, T-3224, T-3225, T-3226, T-3228, T-3230 (+ T-3231 flake); open T-3227 (parked).

## What remains in Q1/Q2, per task, with the reason not done
| task | reason |
|---|---|
| T-3227 | gated on T-2385's human `[REVIEW]` AC |
| T-3214, T-2978, T-3132/T-3128/T-2958/T-3130, T-3103, T-2988 | SQ-8, SQ-6, SQ-3, SQ-1/2, SQ-10 |
| T-2644, T-2573, T-2606, T-2656, T-2662, T-2616, T-2581, T-2886, T-2911, T-3091, T-3139, T-3141, T-3177, T-2016, T-2669, T-2532, T-2398, T-2015 | arc-less → SQ-4 |
| T-3006 | human decide (recommendation ready) |

## Cost-vs-estimate deltas
- **All 7 builds were estimated at cost 2.0; actuals were at or under**, and the MCP ports especially so. They were pure-helper extractions with lib-level tests (`cargo test -p termlink-mcp --lib` ≈ 2s run, ~1–2 min incremental build). Calibration: MCP/CLI twin ports sit at the bottom of cost 2.
- **T-3222 grew one surface** (the local-registered path) once scoping showed my own T-3226 change had discarded the hub there. The 3-tuple return signature was the only rework.
- **T-3225** cost ~1.3× estimate: the full CLI suite (30s run plus a debug build) ran 3× (the flake investigation), and the RCA gate added a round trip.
- **Waste: ~10 min of wall time** on `cargo test -p termlink-mcp --tests` (the integration build, R3's F7 path). It hit the 600s tool cap and was backgrounded, then killed by PID. It was not needed for any AC. Calibration: never run `--tests` for a lib-only change.
- **The estimator does not discriminate among same-component tasks:** 6 of 7 scored exactly 60/2.0 (blast_radius 1, tier 2, effort 8). Ordering by value needed human-legible severity, not the score.

## Checks recorded
| # | check | result |
|---|---|---|
| 1 | own transcript id + usage | 97b53dfa, 160,717 (~20%) |
| 2 | `git log -8` at start | HEAD 83d8c6ede |
| 3 | `context_tokens.py <path>` vs `< path` on own transcript | 0 vs 306,256 (direct last-usage 306,256) → F15 |
| 4 | T-3228 tests / mutants A, B / lib | 3/3; A 1 red, B 2 red; 936/936 |
| 5 | T-3226 test / inverted mutant / lib | 1/1; red; 937/937 |
| 6 | CLI `resolve_target_name_to_fp` (agent ping) | reads `reg.metadata.identity_fingerprint` → MCP ping not drift |
| 7 | T-3222 tests / mutant / lib | 2/2; 1 red; 939/939 |
| 8 | T-3230 test / mutant / lib | 1/1; red; 940/940 |
| 9 | T-3224 test / mutant / lib; grep for other struct constructors | 1/1; red; 941/941; none |
| 10 | T-2385 status | `owner: human`, `[REVIEW]` AC unticked → T-3227 parked |
| 11 | T-3225 tests / pre-fix mutant / full CLI ×2 | 2/2; 1 red; 1155+1 flake, then 1156/1156 |
| 12 | `isolate_rejects_non_git_dir` alone; `ENV_LOCK` refs in dispatch.rs | pass; 0 |
| 13 | T-3231 full CLI ×3 | 1156/1156 ×3 |
| 14 | P-011 on each close | 4/4, 3/3, 3/3, 3/3, 3/3, 3/3, 2/2 |
| 15 | own-transcript budget mid-run | 215K, 260K, 288K, 306K, ~320K |
| 16 | `test-mcp-desc-budget.sh`; description-length distribution | rc 0; 260 tools, 105,840 chars; caps save 0.5/0.9/3.0% |
| 17 | `gh run list doc-lint` ×5; `gh release list`; `rev-list origin/main..main` | 5× failure; v0.11.2 latest; 42 ahead / 0 behind |
| 18 | quadrant × owner × arc census (hv-lc, hv-hc, lv-lc, lv-hc) | § Census |
| 19 | `git show --stat 47201ad2a` | includes T-3231 rename, 0 content lines |
| 20 | finalization canary on `git archive HEAD` vs working tree | rc 1 `[started-work] T-3231` vs rc 0 |
