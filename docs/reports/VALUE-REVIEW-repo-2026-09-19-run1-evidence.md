# VALUE REVIEW — EVIDENCE FILE (repo scope) — run1

Project: /opt/termlink (TermLink). HEAD `e28e8abe7`. Date: 2026-09-19.
Task: T-2971. Phase 0-3 output (GATHERER). **Facts only — no classification, no recommendations.**

Snapshot taken 2026-09-19T18:50:56+02:00, before any review-generated bus/cron activity.
Collected by one orchestrating gatherer + four dispatched gatherer workers
(2 via the Agent tool, 3 via `fw termlink dispatch` after a framework gate capped Agent dispatches at 2).

---

# Phase 1 — Yardstick (PROVISIONAL, UNCONFIRMED — headless run, no human available)

## [ASK] GATE 1 — verbatim, what I would have asked the human

> "Before I classify anything, please confirm or correct this yardstick:
>
> 1. **Purpose** (from `docs/CHARTER.md`, human-blessed): TermLink is a hub-mediated,
>    durable append-log message bus with terminal endpoints — the coordination substrate
>    that lets a fleet of AI agents (and humans) discover each other, exchange durable
>    messages, claim work, and control terminal sessions across one or many machines.
> 2. **Users/consumers**: is TermLink's consumer set (a) only this host's agents,
>    (b) the 5-hub ring20 fleet, (c) external/public users via the Homebrew formula +
>    GitHub releases + `install.sh`, or (d) all three? This materially changes every
>    DELETE verdict (DELETE CHECK 5 = 'no external consumer'). I can see release
>    artifacts published publicly but I have NO download or install telemetry.
> 3. **Value drivers and weights** (`policy/value-drivers.yaml` v3): D1 Antifragility 9,
>    D2 Reliability 7, D3 Usability 5, D4 Portability 3, F-RECALL Recall Leverage 6,
>    F-ORCH Orchestration Leverage 5. These are §ACD-sovereign — confirm they are the
>    yardstick for ranking and that I may NOT propose changing them.
> 4. **Non-goals** — I will treat the charter's five non-goals as binding. Confirm.
> 5. **Scope of 'the guard layer' question**: ~19 cron canaries + ~15 static checks
>    + ~85 fixture suites now exist. Is the guard layer itself in scope for DELETE/
>    REFACTOR review, or is it protected like a ratified workflow? (Ground rule says
>    never weaken checks to look cleaner — I will not propose deleting a *firing*
>    guard; the question is whether consolidation proposals are welcome.)"

## [ASK] GATE 2 — verbatim, what I would have asked about data

> "Here is the data availability map. Three things I especially need you to answer:
> (a) Does ANY MCP tool invocation telemetry exist anywhere I cannot see — a Claude
> Code transcript store, an MCP server log, a proxy? I found none in-repo, which means
> I have NO usage data for the 260+ MCP tools and must treat all of them as UNMEASURED
> (reading D), never as unused.
> (b) Is there download/install telemetry for the public release artifacts (GitHub
> release asset download counts, Homebrew analytics)? Without it I cannot rule out an
> external consumer for any CLI verb.
> (c) The `.tasks/` ledger has 2679 tasks. Is the completed/ corpus trustworthy as
> history, or was it bulk-migrated (T-2290 found 157 tasks archived without
> finalization)?"

## PROVISIONAL yardstick adopted (marked UNCONFIRMED in the report)

**Purpose** (one sentence, verbatim from the human-blessed charter):
> TermLink is a hub-mediated, durable append-log message bus with terminal endpoints —
> the coordination substrate that lets a fleet of AI agents (and humans) discover each
> other, exchange durable messages, claim work, and control terminal sessions across
> one or many machines.

**Users/consumers** (provisional): (1) agents + operator on this host (workstation-107);
(2) the ring20 fleet — 5 declared hubs, 4 reachable at snapshot; (3) an UNKNOWN external
audience reachable via `install.sh`, Homebrew formula and GitHub Releases. Assumed
non-empty because artifacts are published publicly and `install.sh` is the README's
first-listed install path. **This assumption is load-bearing for every DELETE verdict
on a public CLI surface.**

**Core capabilities** (the four charter verbs):
1. Discover — presence, listeners, find-idle
2. Exchange durable messages — topics, offsets, acks, replay, DM threads, doorbell
3. Claim work — claim/renew/release/transfer leases
4. Control terminal sessions — PTY spawn/exec/inject/stream/attach

**Non-goals** (binding): not inter-hub federation; not a durable database/system of
record; not a social/engagement platform; not a workflow/orchestration engine; not a
security boundary between mutually-distrusting tenants.

**Value drivers + weights** (§ACD-sovereign, NOT modifiable by this review):
D1 Antifragility 9 · D2 Reliability 7 · F-RECALL Recall Leverage 6 · D3 Usability 5 ·
F-ORCH Orchestration Leverage 5 · D4 Portability 3.

## Contradictions between stated purpose and actual code (flagged in Phase 1, per prompt)

- **C1.** Charter non-goal #3 says TermLink is "not a social/engagement platform" and
  names social-analytics surfaces as "removal candidates, not features". The live tool
  registry still serves 28 such tools; they are *acknowledged* in
  `.context/checks/charter-drift-allowlist` pending human decision T-2548, not removed.
  Purpose and code disagree, knowingly, with the disagreement written down.
- **C2.** README advertises "270+ MCP tools" as a headline capability. The charter names
  four verbs. A surface that large is not derivable from four verbs; the README frames
  breadth as the product while the charter frames it as drift risk.
- **C3.** Charter non-goal #2 says TermLink is "not a durable database". 18 of 45 live
  topics are `retention: forever`, one (`health:ring20-fedprobe`) holding 1663 records
  = 39% of all traffic on this hub.

---

# Evidence — collected first-hand by the orchestrating GATHERER (run1)

Snapshot taken 2026-09-19T18:50:56+02:00, BEFORE any review-generated activity.
Facts only. No classification.

| # | Item | Source | Source status | Data point (citation) | Window | Kind |
|---|---|---|---|---|---|---|
| O-01 | Repo shape | `wc -l` over `crates/*/**.rs` | EXISTS | cli 72790 · mcp 50828 · hub 24550 · session 23838 · bus 5240 · protocol 3093 · test-utils 347 LOC | at snapshot | structure |
| O-02 | Repo shape | `ls`/`find` | EXISTS | 198 files in `scripts/`; 85 in `tests/`; 390 `.md` under `docs/`; 34 skills in `.claude/commands/`; 503 cards in `.fabric/components/` | at snapshot | structure |
| O-03 | Task ledger | `ls .tasks/*` | EXISTS | 246 active · 2433 completed; highest ID T-2972 | at snapshot | structure |
| O-04 | Baseline — tests | `cargo test --workspace --no-fail-fast` run at 18:51 | EXISTS | **ALL GREEN**: 1143+497+926+174+113+99+4+3+3 = **2962 passed, 0 failed, 4 ignored** across 10 targets | 2026-09-19 | structure |
| O-05 | Baseline — hub | `termlink hub status --governor --json` | EXISTS | running, pid 12343, runtime_dir `/var/lib/termlink` (NOT volatile /tmp). `capacity_hits_total:0`, `rate_hits_total:0`, `cv_index_overflow_total:0`, `dedupe_hits_total:32`, `rate_buckets_evicted_total:19404`, `retention_sweep_pruned_total:2943` | at snapshot | usage |
| O-06 | Bus traffic | `termlink channel list --json` | EXISTS | **45 topics, 4313 messages total** on the local hub | lifetime of hub SQLite | usage |
| O-07 | Bus traffic by class | derived from O-06 | EXISTS | health probe (`health:ring20-fedprobe`) 1663 = **38.6%** · presence 1040 = 24.1% · chat-arc 1001 = 23.2% · operator-durable 377 = 8.7% · **DM work threads 185 = 4.3%** (14 topics) · test/demo debris 27 = 0.6% (12 topics) · conv/listeners 16 = 0.4% | lifetime | usage |
| O-08 | Retention vs non-goal #2 | O-06 | EXISTS | 18 of 45 topics are `retention: forever`; the largest (`health:ring20-fedprobe`, 1663 records) is a self-health probe | at snapshot | structure |
| O-09 | Topic debris | O-06 | EXISTS | 12 topics are test/demo/probe artefacts (`aef-elect-smoke-*`, `aef-g1-demo-*`, `aef-s8-probe-*`, `t2838-s1/s2-*`, `substrate-*-demo`, `agent-conv-selftest-*`); 9 topics hold ≤1 record | at snapshot | structure |
| O-10 | Fleet | `termlink fleet doctor --json` | EXISTS | 5 declared hubs; 4 reachable, 1 error (`laptop-141` — no route to host). Versions: `0.11.1766`×2, `0.11.1411`×2, `unknown`×1 | at snapshot | usage |
| O-11 | **Discover verb — live state** | `scripts/agent-listeners-fleet.sh --json` | EXISTS | **2 LIVE listeners fleet-wide**: `penelope` (role email-relay, host dimitrimintdev) and `ring20-dashboard-agent` (role listener). **Neither carries `pty_session`. Neither advertises any `capabilities`.** | at snapshot | usage |
| O-12 | Discover verb — cv_index | `termlink channel cv-keys agent-presence --json` | EXISTS | `count: 1` — a single cv_key (`penelope`) on the local hub's presence index | at snapshot | usage |
| O-13 | **Claim verb — live state** | `termlink channel claims-summary --all --json` | EXISTS | 45 topics scanned. **0 active claims. 0 stuck.** 11 expired rows, ALL on self-test/demo topics: `substrate-drain-demo` (9), `aef-elections-*` (1), `aef-s8-probe2-*` (1). **No real work topic carries any claim history.** | lifetime | usage |
| O-14 | **Session-control verb — live state** | `termlink list --json` | EXISTS | **38 registered sessions, all with locally-alive PIDs, 0 with heartbeat older than 600 s.** Tags show real work (`project=001-CashWeb-Lightspeed-Ecwid-integration`, `project=termlink`, `master,claude,framework`) | at snapshot | usage |
| O-15 | Registration-store disagreement | O-14 vs `check-frozen-husk-freshness.sh --json` vs `ls` | EXISTS | Three counts of "a registration" disagree: `termlink list` = **38**; husk canary (`sessions/*.json`) = **5** (`total_registrations:5, dead_orphans:0`); filesystem = 5 `.json` + 5 `.sock` + **382 `.sock.data` orphans** (20 KB total, oldest Aug 16, newest Sep 19) | 34 days | structure |
| O-16 | Offline queue | `termlink channel queue-status --json` | EXISTS | `pending:0`, `dead_letters:0`, `cap:1000` — queue healthy and drained | at snapshot | usage |
| O-17 | **Doorbell rail health** | `.context/working/.fleet-doorbell-mail-canary.log` (418 B) | EXISTS | `Fleet doorbell+mail health: DRIFT — total=4 pass=1 fail=0 unreachable=2 transient_skipped=1`. **Only `workstation-107-public` (local) passes (483 ms). `ring20-management` and `ring20-dashboard` both `verdict=setup-fail`.** | last cron run | usage |
| O-18 | Commit activity | `git log --since="12 months ago"` | EXISTS | **7167 commits**; by month 2026-03 1054 · 04 2114 · 05 1090 · 06 1077 · 07 594 · 08 1055 · 09 183 | 12 mo | structure |
| O-19 | Author concentration | `git log --format=%an` | EXISTS | Dimitri Geelen 6533 · DimiDev32 603 · others 31. **>91% of commits from one identity** | 12 mo | structure |
| O-20 | **Release staleness** | `git describe`, `git tag` | EXISTS | `v0.11.2-1939-ge28e8abe7`. Last tag **v0.11.2 on 2026-06-24 = 87 days ago, 1939 commits ago.** Tags: v0.1.0, v0.1.1, v0.9.0, v0.9.1, v0.10.0, v0.11.0, v0.11.1, v0.11.2 | 87 days | usage |
| O-21 | **Homebrew formula staleness** | `homebrew/Formula/termlink.rb:12`; `git log -1 -- <file>` | EXISTS | `version "0.9.1"`. Last commit to the formula **2026-04-19 (`aa019d024`) = 153 days ago**. README advertises `brew install termlink` at README.md:96 and README.md:318 | 153 days | friction |
| O-22 | Install-path divergence | O-20 + O-21 + `install.sh:86-89` | EXISTS | `install.sh` resolves `latest` → v0.11.2 (87 d old). Homebrew pins v0.9.1 (153 d old). **The two advertised install paths deliver different releases, 5 tag generations apart.** `scripts/check-release-artifact-drift.sh` compares artifact NAMES only and is documented as not checking versions | 87–153 days | friction |
| O-23 | **CI trigger scope** | `.github/workflows/release.yml:9-11`, job list | EXISTS | `release.yml` triggers **only** on `push: tags: v*`. It contains jobs `test` (`cargo test --workspace` + guard layer) and `test-macos`. Both were added **2026-08-14** (`b25b1c3f4`, T-2686/T-2691) — **after** the last tag (2026-06-24). **No tag has been pushed since. Therefore neither job has ever executed in CI.** | 36 days | friction |
| O-24 | CI per-push coverage | `.github/workflows/doc-lint.yml:26-81`, `install-check.yml:29-33` | EXISTS | Per-push/PR CI = `doc-lint.yml` (jobs: `error-code-doc-lint`, `env-var-doc-lint`, `guard-layer` running `bash scripts/run-guard-layer.sh`) + `install-check.yml`. **`cargo test --workspace` runs per-push nowhere.** | 36 days | friction |
| O-25 | **CLAUDE.md size** | `wc`, `python3 len()` | EXISTS | **3283 lines · 274,908 chars ≈ 68,700 tokens**, auto-loaded into every session. `## Core Principle` at line 2461 (2461 above the re-vendor split, 822 below). 313 distinct `T-XXXX` IDs cited. 94 `###` subsections | at snapshot | cost |
| O-26 | **CLAUDE.md growth** | `git show <sha>:CLAUDE.md \| wc -l` | EXISTS | 2026-04-01 **850** → 05-01 1013 → 06-01 1186 → 07-01 1398 → 08-01 1554 → 09-01 **3169** → today **3283**. **+286 % in 5.5 months; +1729 lines in Aug alone** | 5.5 mo | cost |
| O-27 | **CLAUDE.md guard-layer share** | line-span `### Mirror drift canary` → `## Project-Specific Rules` | EXISTS | lines 39–1948 = **1909 lines (58 % of file), 134,339 chars ≈ 33,600 tokens** of prose describing canaries/checks. 38 of 94 `###` subsections are canary/check narrative | at snapshot | cost |
| O-28 | **MCP tool-description context tax** | parsed `crates/termlink-mcp/src/tools.rs` | EXISTS | **260 descriptions, 105,512 chars ≈ 26,400 tokens.** Max 1546 (`termlink_help`); **1** over 1000 chars; 23 over 600 | at snapshot | cost |
| O-29 | Standing per-session tax | O-25 + O-28 | EXISTS | CLAUDE.md ≈68.7k + MCP descriptions ≈26.4k ≈ **95,100 tokens loaded before the first tool call** | at snapshot | cost |
| O-30 | arc-005 delivered a real reduction | `.context/arcs/mcp-slimming.yaml` vs O-28 | EXISTS | Arc description (2026-07-11) measured 273 tools / ~156 KB / ~39k tokens, worst 11,751 chars, 24 over 1000, 94 over 600. **Today: 260 / 103 KB / ~26.4k tokens, worst 1546, 1 over 1000, 23 over 600.** S1 task T-2406 completed 2026-07-11. **The reduction is real and measurable** | 70 days | value |
| O-31 | arc-005 never closed | `.context/arcs/mcp-slimming.yaml`; `git log -1` | EXISTS | `status: in-progress`, `closed_at: null`, `decision: null`, `bvp_scores: {}`. Last commit touching the file **2026-07-11** = 70 days | 70 days | friction |
| O-32 | Arc staleness generally | `git log -1 --date=short -- .context/arcs/*` | EXISTS | arc-parallel-substrate 2026-06-13 (98 d) · arc-substrate-fitness 2026-06-23 (88 d) · comms-loudness 2026-07-21 (60 d) · mcp-slimming 2026-07-11 (70 d) · arc-008 2026-09-09 (10 d). **5 of 7 arcs `in-progress`; 4 of those untouched 60–98 days** | 98 days | friction |
| O-33 | **Allowlist backlog** | `.context/checks/*` (19 files) | EXISTS | **620 acknowledged entries total.** Largest: `mcp-parity-census` **236** · `unbounded-rpc-call` **155** · `verification-pipefail` **155** · `charter-drift` **28** · `error-swallowing` 9 · `drain-sink` 6 · `alloc-sink` 5 · `platform-lock` 5 · `strict-star` 5 · `busy-spin` 4 · `task-template-idioms` 4 · `addressed-aliases` 3 · `error-code-emission` 3 · `planted-default-gate` 2; 5 files empty | at snapshot | structure |
| O-34 | **Allowlist reason diversity** | `sed`/`uniq -c` over the four largest | EXISTS | `unbounded-rpc-call`: **140 of 155 carry the identical reason "NOT YET MIGRATED — can hang forever; pending T-2669 per-verb timeout decision"**. `mcp-parity-census`: **235 of 236 identical "unexamined — no parity assertion exists; expansion tracked in T-2748"**. `verification-pipefail`: 155 distinct file:line, all "predates T-2775; retrofit is a human decision". `charter-drift`: all 28 "T-2548 pending: …" | at snapshot | structure |
| O-35 | **Status of the tasks the allowlists defer to** | `.tasks/` frontmatter | EXISTS | **T-2669** `started-work`, created 2026-08-12 (38 d open) — 140 entries parked on it. **T-2748** `status: captured`, **`horizon: later`** — 235 entries parked on an explicitly-backlogged task. **T-2548 `work-completed`, `date_finished: 2026-08-20T17:54:28Z`** — 28 entries still say "pending". **T-2775 has no task file in `.tasks/active` or `.tasks/completed`** — 155 entries cite a non-existent ID | 25–38 days | friction |
| O-36 | **T-2548's recorded decision** | `.tasks/completed/T-2548-*.md` Updates block | EXISTS | `**Decision:** GO` (subtract), 2026-08-20T17:54:27Z. Rationale cites zero first-party callers and failure of all four charter verbs. Decision is **gated on IW-1**, a cross-project external-consumer check the T-559 boundary blocked. KEEP-list named: `agent_search_thread`, `agent_thread_path`, `agent_recent_window` | 30 days | value |
| O-37 | **No follow-on build task from T-2548** | `grep -rl T-2548 .tasks/`; name search | EXISTS | Files referencing T-2548 are all prior/parallel purpose-review tasks (T-2470, T-2549, T-2678, T-2680, T-2683, T-2690, T-2716) plus this review. **No task named for the subtract build exists.** No active task has `analytics` or `subtract` in its name | 30 days | friction |
| O-38 | Guard-layer artefact count | `ls` | EXISTS | **66 `scripts/check-*.sh` · 62 `tests/*fixtures*.sh` · 27 `.context/cron/*.crontab` = 155 artefacts.** CLAUDE.md narrates 38 of them | at snapshot | structure |
| O-39 | **Canary heartbeats — CORRECTION** | `find .context/working -name '*.heartbeat'` | EXISTS | **31 heartbeat files exist** (they are dotfiles, invisible to a `*.heartbeat` glob). **Every `-canary.log` has a heartbeat companion.** `scripts/check-canary-aliveness.sh` runs and reports freshness. A sub-gatherer's claim that "no `.heartbeat` companion files exist" is **FALSE** and is not carried into this review | at snapshot | structure |
| O-40 | **Dead canaries** | `stat -c %Y` on each heartbeat | EXISTS | 28 of 31 fresh (<48 h). **3 stale: `.unbounded-rpc-call-canary` 510 h (21 d) · `.error-swallowing-canary` 506 h (21 d) · `.task-template-idioms-check` 428 h (18 d)** | 21 days | friction |
| O-41 | Pairing of O-34 and O-40 | derived | EXISTS | The canary guarding the 155-entry `unbounded-rpc-call` allowlist (140 sites self-described "can hang forever") is one of the three that stopped firing 21 days ago | 21 days | friction |
| O-42 | **Value-drivers fork — EXAMINED AND DISMISSED** | `policy/value-drivers.yaml` vs `.agentic-framework/policy/value-drivers.yaml`; `lib/bvp.sh:159,615` | EXISTS | The two files carry different free-driver sets (project: F-RECALL 6, F-ORCH 5; framework: F-RECALL 6, F-AUTONOMY 4, F3 7, F1 7, F2 6). **`lib/bvp.sh` reads `PROJECT_ROOT / 'policy' / 'value-drivers.yaml'` and `lib/bvp.sh:43` names the framework copy "the canonical template".** This is template-vs-instance, **not a fork.** The yardstick used by this review is the project file and is intact | at snapshot | structure |
| O-43 | BVP data availability | `ls`; frontmatter grep; `fw bvp --quadrant` | PARTIAL | **`.context/audits/bvp-realization.jsonl` ABSENT** — no realized-value data exists. 190/246 active tasks carry `bvp_scores_proposed`; **0 carry confirmed `bvp_scores`**; **0 carry a cost score.** `fw bvp` itself reports "135/164 (82%) have no known cost — blast_radius unmeasured, so no quadrant" | at snapshot | value |
| O-44 | Dangling framework reference | `fw bvp` stdout vs O-03 | EXISTS | `fw bvp` prints "Cost becomes measurable once `components:` is resolved; see **T-3068**". Highest existing task ID is **T-2972**. T-3068 does not exist | at snapshot | friction |
| O-45 | **Budget-gate thresholds — docs vs code** | `.agentic-framework/agents/context/budget-gate.sh:101-108`; CLAUDE.md:2907-2910, 2916, 3052 | EXISTS | Code: `CONTEXT_WINDOW` default **300000**; warn 75 % (225 k), urgent 85 % (255 k), critical 95 % (285 k). CLAUDE.md states three mutually inconsistent versions: L2907-2910 "60 %/75 %/85 % = 120 K/150 K/170 K"; L2916 "**170K** urgent→critical (**BLOCK**)"; L3052 "blocks … when context reaches critical level (**>=150K** tokens, ~75 %)". **L2916 and L3052 contradict each other; all three contradict the code by 115–135 k tokens** | at snapshot | friction |
| O-46 | Budget gate observed behaviour | `.context/working/.budget-status` read during this run | EXISTS | `{"level": "ok", "tokens": 223601}` — consistent with the code's 225 k warn threshold, inconsistent with every number in CLAUDE.md. **The gate is working; the documentation of it is wrong.** | at snapshot | friction |
| O-47 | Session churn | `ls .context/handovers`; `.compact-log` | EXISTS | **1701 handover files**; 211 in the last 30 days (**≈7/day**), 19 in the last 7. `.compact-log` holds **487** compaction/handover events | 30 days | friction |
| O-48 | MCP invocation telemetry | grep of `crates/termlink-mcp/src/*.rs`; `ls ~/.termlink/` | **ABSENT** | No `tool_invocation` / `usage_counter` / `invocation_count` / `telemetry` symbol in the MCP crate; no per-tool log, counter or sink under `~/.termlink/`. **For 260 MCP tools, zero have a recorded call count. "No data" — explicitly not "zero use".** | — | usage (GAP) |
| O-49 | Release/download telemetry | repo search | **ABSENT** | No GitHub release asset download counts, Homebrew analytics, or any install telemetry in-repo. **The size of the external consumer set is unknown**, which is load-bearing for DELETE CHECK 5 | — | usage (GAP) |
| O-50 | Out-of-band delivery observer | `scripts/session-message-selftest.sh` (T-2876) | EXISTS | An out-of-band observer for message delivery DOES exist: it asserts on the RECEIVER's own `~/.claude/projects/*/<sessionId>.jsonl` transcript rather than on TermLink's own bus, and distinguishes DELIVERED / BLOCKED / ENQUEUED / UNDELIVERED. Ground rule "a channel cannot report its own failures" is therefore **satisfied for the DM path** | at snapshot | structure |
| O-51 | Ratified workflows | repo search for BPMN / `aef:` namespace | (pending docs gatherer) | see `ev-docs.md` | — | structure |

## NON-USE DIAGNOSIS evidence collected (readings recorded, NOT resolved — that is the JUDGE's job)

| Item | A BROKEN | B NEVER WIRED | C UNDISCOVERABLE | D UNMEASURED | E NOT WANTED |
|---|---|---|---|---|---|
| Push-wake / doorbell rail | **Strong**: waker-liveness canary reports `RAIL DARK` in **36/36** entries; doorbell-mail canary `setup-fail` on 2 of 3 reachable remote hubs (O-17); stale-waker-code canary names 3–4 pushwaker processes on pre-current code | 2 LIVE listeners, **neither carries `pty_session`** (O-11) — nothing is armed to be woken | — | — | **Ruled out**: charter names it; arc-004 shipped for it; 3 canaries built to watch it; T-2388 live-proved it |
| Claim/lease work-stealing | — | No production work topic exists; the only claim rows ever recorded are on `substrate-drain-demo` and `aef-*` probe topics (O-13) | — | — | **Ruled out**: charter verb 3; `/claim` `/renew` `/release` `/claim-transfer` skills all shipped; `substrate-smoke.sh` exercises it daily |
| `find-idle --capability` | — | Neither LIVE listener advertises any `capabilities` (O-11) — the filter can never match | — | — | **Ruled out**: charter-adjacent, documented as substrate primitive #2 |
| 260 MCP tools | — | — | — | **Total**: no instrumentation of any kind (O-48) | 28 acknowledged off-charter with a recorded **GO-to-subtract** (O-36) |
| Conversation-analytics tool family (28) | — | — | — | O-48 | **Strong**: T-2548 GO decision, 2026-08-20; "zero first-party callers" verified by grep in the decision record (O-36) |
| Homebrew install path | **Strong**: pinned to v0.9.1, 153 days stale (O-21), while README advertises it (O-21) | — | — | No download telemetry (O-49) | Not ruled out either way — no consumer data |
| `cargo test` in CI | **Strong**: wired to a trigger that has not fired since the job was created (O-23) | Job exists and is correct; the *trigger* never fires | — | — | **Ruled out**: T-2686's stated purpose |
| 3 stale canaries (O-40) | Unknown — cron may have been removed, or the script may be failing | — | — | Their `.stderr` companions not inspected in this run | — |

## Contradictions recorded (source vs source, docs vs code)

1. **O-45/O-46** — CLAUDE.md states the budget-gate block threshold three different ways (170 K at L2916, 150 K at L3052, 85 %/170 K at L2910); the code uses 95 % of a 300 K window = 285 K.
2. **O-15** — three stores disagree on how many terminal sessions are registered (38 / 5 / 382 artefacts).
3. **O-21/O-22** — README advertises two install paths that deliver releases 5 tag generations apart.
4. **O-35** — 28 allowlist entries say "T-2548 pending"; T-2548 completed 30 days ago. 155 entries cite T-2775, which does not exist.
5. **O-30/O-31** — the `mcp-slimming` arc is `in-progress` with `closed_at: null` while its measurable objective was substantially delivered 70 days ago.
6. **Charter vs README** — charter non-goal #3 names social-analytics surfaces "removal candidates, not features"; README headlines "270+ MCP tools" as a capability.
7. **Charter vs live state** — non-goal #2 ("not a durable database") vs 18 `forever` topics, the largest being a 1663-record self-health probe (O-08).
8. **O-42** — an apparent value-drivers fork, examined and **dismissed**: it is template-vs-instance, and the project file is the one the code reads.

---

# Fragment A — MCP / CLI surface (dispatched gatherer)

# Evidence — MCP tool surface + CLI counterpart (/opt/termlink)

Gathered 2026-09-19. Binary probed: `/root/.cargo/bin/termlink` v0.11.1766. Repo HEAD branch `main`.
EVIDENCE ONLY — no classification, no recommendation.

| Item | Source | Source status | Data point (citation) | Window | Kind |
|---|---|---|---|---|---|
| Total `#[tool(name = "termlink_…")]` entries | `crates/termlink-mcp/src/tools.rs` | EXISTS | 262 raw grep hits; **260** after stripping comment lines; **260 distinct names** (`grep -n 'name = "termlink_' \| grep -v ':\s*//' \| wc -l`) | HEAD | structure |
| Live vs deprecated | `termlink help --json` (v0.11.1766) | EXISTS | **260 tools, 46 deprecated, 214 live** across **29 categories** | HEAD | structure |
| `is_deprecated()` definition | tools.rs:1270-1276 | EXISTS | substring test on description: `legacy` \| `retirement` \| `deprecated` \| `t-1166` | HEAD | structure |
| Category→tool map (29 cats) | `termlink help --json` | EXISTS | agent_chat 12(5 dep) · agent_engagement_metrics 8(8) · agent_inbox 8(0) · agent_poll 3(3) · agent_presence 19(0) · agent_rankings 8(3) · agent_read 14(0) · agent_stats 10(0) · agent_thread 17(6) · agent_thread_health 8(0) · batch 4 · channel 23 · channel_admin 13(2) · channel_engagement 11(6) · channel_moderation 10(3) · channel_poll 4(4) · channel_threading 7 · diagnostics 15(3) · dispatch 2 · events 8 · execution 3 · files 2 · fleet 12 · hub 8 · kv 5 · remote 9(3) · session 12 · tofu 3 · tokens 2 | HEAD | structure |
| Charter-drift allowlist size | `.context/checks/charter-drift-allowlist` | EXISTS | **28 entries**, all reason-suffixed `T-2548 pending: …` | HEAD | structure |
| Allowlist entries (verbatim reasons, compact) | same file | EXISTS | `top_replied`/`top_repliers`/`top_thread_starters` = "leaderboard, traces to no charter verb"; `first_post_by`/`first_responders` = "'who got there first' ranking"; `agent_stats` = "per-agent posting analytics"; `response_latency` = "engagement-timing analytics"; `topic_stats`/`topic_summary` = "per-topic posting analytics"; `topic_metadata_history` = "topic-metadata archaeology"; `user_summary` = "per-user activity profile"; `agent_snippet`/`channel_snippet` = "excerpt rendering for analytics views"; `search_by` = "author-scoped search over chat history"; `recent_threads` = "thread-activity listing"; `forwards_of` = "forward-provenance analytics"; `thread_health` = "thread engagement scoring"; `threads_by` = "author-scoped thread listing"; `busiest_threads` = "activity ranking"; `idle_threads`/`quiet_threads` = "activity ranking (inverse)"; `orphan_replies`/`unanswered`/`self_replies` = "thread-hygiene analytics"; `channel_mentions`/`channel_mentions_of`/`channel_search` = "plausible verb-2 reading — human to split"; `channel_digest` = "engagement digest" | HEAD | value |
| Charter-drift check result | `bash scripts/check-charter-drift-freshness.sh --json --no-heartbeat` (rc=0) | EXISTS | `{"ok":true,"checked":214,"live_off_charter":0,"acknowledged_count":28,"off_charter_total":28,"firing":[],"detectors":["name-pattern","category"],"probe_version":"0.11.1766"}` | 2026-09-19 | value |
| Charter check self-declared scope | same JSON `scope` field | EXISTS | "detects known off-charter shapes (social-analytics names + analytics categories); not a full charter-traceability audit of every tool" | HEAD | structure |
| Parity census | `bash scripts/check-mcp-parity-census.sh --json` (rc=0) | EXISTS | `{"ok":true,"total":260,"covered":24,"acknowledged":236,"unexamined":0,"coverage_pct":"9.2","firing":[]}` | 2026-09-19 | structure |
| Parity allowlist | `.context/checks/mcp-parity-census-allowlist` | EXISTS | **236** non-comment entries, every one with identical reason `unexamined — no parity assertion exists; expansion tracked in T-2748`. Sample 10: `termlink_agent_ack`, `_ack_history`, `_ack_status`, `_active_in_thread`, `_active_now`, `_ancestors`, `_ask`, `_busiest_threads`, `_chat_arc_recent`, `_contact` | HEAD | structure |
| Parity suite size | `crates/termlink-mcp/tests/parity.rs` | EXISTS | 1493 LOC, **24** `fn parity_*` cases | HEAD | cost |
| Parity check directionality | `scripts/check-mcp-parity-census.sh` | PARTIAL | Script asks only "is every MCP tool asserted-or-acknowledged". Grep for `cli`/`verb` yields one hit (a remediation message, line 172). **No CLI→MCP direction**: CLI verbs lacking an MCP counterpart are not enumerated by any check found. | HEAD | structure |
| tools.rs LOC | `wc -l` | EXISTS | **46,458** lines in one file; crate total 46,891 (`crates/termlink-mcp/src/*.rs`) | HEAD | cost |
| CLI crate LOC | `wc -l crates/termlink-cli/src/**/*.rs` | EXISTS | **67,954** lines | HEAD | cost |
| `fn *_mcp` helpers | recursive grep over `crates/termlink-mcp/src` | EXISTS | **71 distinct names**, **96 total definitions**. Of those, `fn to_json_mcp` alone is defined **26 times** (local helper redefined per function). (CLAUDE.md T-2747 note cites 79 distinct; measured here = 71.) | HEAD | cost |
| Git churn, 6 months | `git log --since="6 months ago" --oneline -- crates/<c>` | EXISTS | termlink-cli **770** · termlink-mcp **420** · termlink-hub **183** · termlink-session **127** · termlink-bus **44** | 2026-03-19→2026-09-19 | cost |
| Git churn, all-time (for contrast) | `git log --oneline -- crates/<c>` | EXISTS | termlink-cli 829 · termlink-mcp **420** · termlink-hub 206 · termlink-session 169 · termlink-bus 44 | all | cost |
| — implication of the two rows above | derived | EXISTS | **100% of termlink-mcp's lifetime commits (420/420) and 100% of termlink-bus's (44/44) fall inside the last 6 months**; cli 770/829 = 93% | all | cost |
| **MCP tool-invocation instrumentation** | grep `invocation\|call_count\|usage_count\|tool_metric\|telemetry\|counter\|tool_call\|record_call` over `crates/termlink-mcp/src/` | **ABSENT** | **No per-tool invocation counter, no call log, no telemetry sink exists.** Only hits are unrelated (a `malformed_counter` local var at tools.rs:112/122, and prose in tool descriptions). | HEAD | usage |
| MCP server per-call logging | `crates/termlink-mcp/src/server.rs` (412 LOC), `lib.rs` (21 LOC) | **ABSENT** | grep for `tracing::\|log::\|metrics\|counter\|audit` in server.rs returns **zero matches**. No dispatch-level logging. | HEAD | usage |
| Runtime usage artifacts | `ls ~/.termlink/` | ABSENT | 20 entries (hubs.toml, outbound.sqlite, awaiting_ack.sqlite, be-reachable/pushwaker/presence-sweep/rotation logs, identities, secrets). `grep -rl 'termlink_agent\|termlink_channel' ~/.termlink/` → **no matches**. No file records which MCP tools were called. | 2026-04→2026-09 | usage |
| Tool-name mentions across repo (NOT invocations) | `grep -rohE 'termlink_[a-z0-9_]{3,}'` | EXISTS | distinct-name counts: `.claude` 388 · `.tasks` 319 · `.context` 317 · `docs` 185 · `scripts` 27. Bulk concentrated in `.context/audits/orchestrator-mcp-baseline.yaml` and `.context/handovers/S-*.md` — these are **inventories/handover prose, not call records**. | all | usage |
| `mcp__termlink__…` (invocation-shaped) references | `grep -ro 'mcp__termlink__termlink_[a-z0-9_]*'` | PARTIAL | **38 distinct tool names** across the whole repo; occurrence counts `.claude` 191 · `.tasks` 8 · `docs` 7 · `.context` 3 · `scripts` 0. Files are `.claude/settings.json`, `.claude/settings.local.json` (permission allowlists), plus `.context/episodic/T-1812.yaml`, `T-2881.yaml`, `.context/project/decisions.yaml`. **These are permission grants and narrative records, not invocation counts.** | all | usage |
| Net statement on usage data | derived from the four rows above | **ABSENT** | **There is no instrumentation of MCP tool invocation anywhere in this repo.** For 260 tools, zero have a recorded call count. "No data" ≠ "zero use" — the question is unanswerable from this repo as it stands. | HEAD | usage |
| `orchestrator-mcp-baseline.yaml` — what it actually tracks | `.context/audits/orchestrator-mcp-baseline.yaml` (356 lines) | EXISTS | Tracks `check_task_governance()` gating status per tool (`gated` / `mutators_ungated` / `readonly_exempt`) — a **governance ratchet, not usage**. Header notes classification was done "by naming convention" because "/opt/termlink is outside PROJECT_ROOT and unreadable from here, so handler-level verification is deferred". Baseline grew 136→154 across T-1755/T-1760/T-1867. | 2026-05-01→ | structure |
| Deprecated set — social analytics (40 tools) | `git log --grep='T-2478'` → `6778f92bb` | EXISTS | 2026-08-02: "P4 Stage 2 — deprecate (not delete) 40 social-analytics MCP tools + 15 CLI twins; [DEPRECATED]+hint markers, hide=true, 880 tests green, **zero consumers**" | 2026-08-02 → today = **48 days** | structure |
| — still registered? | `termlink help --json` + live MCP session tool list | EXISTS | **Yes.** All 40 still in the registry and still exposed to MCP clients: this very session's tool list includes `mcp__termlink__termlink_agent_react`, `_agent_pin`, `_agent_star`, `_agent_typing`, `_agent_poll_start/_vote/_end`, `_agent_top_reacted`, `_channel_poll_results`, etc. | 2026-09-19 | structure |
| Deprecated set — `*_inbox_*` (6 tools, T-1166) | `git log -S` on tools.rs | EXISTS | Added **2026-04-13** (`d214a10e2` "T-1010: Add 3 remote inbox MCP tools … (59 MCP tools)"). Deprecation marker present in the curated help registry by **2026-06-03** (`5b75f4906` T-1942); deprecated-flag surfacing added 2026-06-04 (`c6be04055` T-1960). Registry text: tools.rs:1019 `"Inbox status on a remote hub (legacy primitive, T-1166 retirement WIP) (use termlink_channel_subscribe instead)"` | deprecated ≈ **108 days** | structure |
| — still registered? | live MCP session tool list | EXISTS | **Yes** — `mcp__termlink__termlink_remote_inbox_status`, `_remote_inbox_list`, `_remote_inbox_clear`, `_inbox_status`, `_inbox_list`, `_inbox_clear` all exposed to this session's client. | 2026-09-19 | structure |
| **Deprecation marker lives in a second, parallel registry** | tools.rs:1019 vs tools.rs:16303-16305 | EXISTS | The `#[tool(...)]` attribute description actually sent to MCP clients for `termlink_remote_inbox_status` (line 16304) is **"Show inbox status on a remote hub — total pending file transfers…"** — it carries **no deprecation marker at all**. The `legacy … T-1166 retirement WIP (use … instead)` text lives only in the hand-curated `help_categories()` list at line 1019. `is_deprecated()` reads the curated copy. Counts: **40** `description = "[DEPRECATED` markers on tool attributes; **52** deprecation-marker lines inside the `help_categories()` block (lines ~900-1270). | HEAD | structure |
| — consequence | derived | EXISTS | The 40 social tools carry `[DEPRECATED — use X]` in their client-visible attribute; the **6 inbox tools do not** — an MCP client sees them as ordinary live tools. `termlink help --json` nonetheless reports all 46 as `deprecated:true`. | HEAD | structure |
| `hide = true` in registry | grep tools.rs | ABSENT | `grep -c 'hide = true'` → **0**. The T-2478 commit message claims "hide=true"; no such attribute exists in tools.rs today. (Hiding, if any, is via the `help` surface, not the tool attribute.) | HEAD | structure |
| Anything still CALLING the deprecated tools | grep across repo | ABSENT | No caller found outside tools.rs itself + its test block (`tools.rs:39825-39832` lists the 6 inbox names as an expected-flagged invariant test). T-2478 commit message asserts "zero consumers" for the 40. | HEAD | usage |
| CLI top-level subcommand census | `termlink --help` | EXISTS | **41 subcommands**: register, list, ping, status, info, send, interact, exec, signal, pty, event, mirror, tag, discover, whoami, kv, run, request, spawn, dispatch, dispatch-status, clean, hub, mcp, token, webhook, agent, file, remote, fleet, substrate, net, inbox, tofu, identity, channel, doctor, vendor, completions, version, help | 2026-09-19 | structure |
| CLI verbs with no MCP counterpart | no check exists | **ABSENT** | Not measurable from shipped tooling: the parity census is MCP→CLI only (see directionality row). Candidates visible by name inspection (`mirror`, `webhook`, `pty`, `identity`, `vendor`, `completions`) have no `termlink_*` MCP tool of the same name in the 260-name list, but nothing in the repo asserts or tracks this. | HEAD | structure |
| Related open tasks | `.tasks/` | EXISTS | **T-2548** (`owner: human`, `started-work`) — the pending decision on the 28 off-charter tools. **T-2748** — parity-coverage expansion, cited by all 236 allowlist entries. **T-1415** (`active`) "T-1166 post-cut cleanup: delete retired primitives". **T-1632** (`active`) "Bump DefaultProtocolVersion to 3 (T-1166)". | HEAD | friction |

---

# Fragment B — AEF governance (dispatched gatherer)

# Evidence summary — AEF governance (`/opt/termlink`, 2026-09-19)

Facts only. Full detail + citations: `/tmp/vr1/ev-aef.md`.

## Key numbers

- **Ledger:** 246 active / 2433 completed (2679 total). Episodic entries: **2435**.
- **Active status:** `captured` 119 · `work-completed` 71 · `started-work` 56.
- **Active owner:** human 137 · agent 107 · claude-code 2. **Horizon:** now 151 · next 53 · later 42.
- **Age (since `created:`):** median **42 d**, mean 69.5 d. **>30 d: 181 (73.6%)** · >60 d: 111 (45.1%) · >90 d: 95 (38.6%). Oldest: T-212 at **182 d** (`started-work`, human).
- **Stranded-finalized (T-2833 class):** **0**. All 71 active `work-completed` tasks carry `date_finished` and `owner: human` — they are T-193 partial-completes. Newest handover states **75 tasks with unchecked Human ACs**.
- **Learnings:** 389 entries, 2026-03-08 → **2026-09-18** (newest PL-375).
- **Concerns:** 52 total — watching 27, closed 12, resolved 6, mitigated 5, decided-build 2. Severity across all: medium 30, high 15, low 6, critical 1.
- **Arcs:** 7 files. **5 never closed** (arc-001 104 d, arc-002 89 d, arc-005 70 d, arc-007 60 d, arc-008 10 d); 2 closed (arc-003, arc-004). `bvp_scores: {}` empty in all 7; arc-008 has an empty `anchor_task`.
- **Rework check** (`check-task-id-collisions.sh --json`, 7 branches): `ok:true` — collisions **0**, duplicate files **0**, duplicate titles **0**; **axis D `duplicate_fix_count: 7`** (same-line fixes on both sides, incl. `tools.rs` 8 shared deleted lines, `check-cron-install-drift.sh` 4). Gatherer scan of 2679 titles found **6 exact-duplicate titles (12 files)** — 3 pairs both still ACTIVE (T-2931/2932, T-2936/2937, T-2944/2945, all `pickup:` filings) — plus 36 near-duplicate pairs, e.g. the same governor keying defect, the same Tier-0 self-approval bypass, the same `claimed_by→claimer` doc fix, and the same Watchtower-500 RCA each filed twice.
- **BVP:** `.context/audits/bvp-realization.jsonl` **ABSENT**. 60/246 active tasks (24.4%) carry `bvp_scores_proposed`; **0** carry confirmed `bvp_scores`. **COST axis: 0 of 246 — empty across the entire active ledger.**
- **Value drivers** (`policy/value-drivers.yaml`): protected D1 Antifragility **9**, D2 Reliability **7**, D3 Usability **5**, D4 Portability **3**; free F-RECALL Recall Leverage **6**, F-ORCH Orchestration Leverage **5**. The vendored `.agentic-framework/policy/value-drivers.yaml` (newer mtime) carries a **different active free-driver set**: F-RECALL 6, F-AUTONOMY 4, F3 7, F1 7, F2 6 — no F-ORCH.

## Bypass-log breakdown (`.context/working/.gate-bypass-log.yaml`, gitignored)

**482 entries, 291 distinct tasks, 2026-04-12 → 2026-09-18 (159 days).**

By gate (`caller`): **`check-active-task focus-drift` 164 (34.0%)** · `check_human_sovereignty` 149 · `partial_complete_recheck` 94 · `check_acceptance_criteria` 26 · `owner_change` 10 · `run_verification_commands` 8 · `check_rca_for_bugfix` 8 · `check_inception_decision` 8 · `human-ac-self-validate` 7 · unset 4 · `create-task.sh` 4.

By flag: `FW_SWITCH_FOCUS=1` 157 · `--skip-sovereignty` 149 · `--skip-acceptance-criteria` 120 · `--skip-human-ownership` 10 · `--skip-verification` 8 · `--skip-rca` 8 · `--skip-inception-decision` 8 · `--switch-focus` 7 · `FW_ALLOW_HUMAN_AC_TICK=1 + sed` 7 · `FW_ALLOW_EMPTY_RECOMMENDATION` 4.

By month: Apr 109 · May 122 · Jun 95 · Jul 67 · Aug 79 · Sep 10 (partial). **199 of 482 (41.3%) record no reason.**

Three representative reasons, verbatim:
1. `'Phase A batch close — agent-evidenced (tool registered + silent-OK soak)'` (93×)
2. `'Inception decision: GO'` (82×)
3. `'Completed via Watchtower UI (human action)'` (51×)

## Audit findings that never close

151 audit files, 2026-03-09 → 2026-09-19. The newest three (09-17/18/19) all run `sections: "structure"`; **09-18 and 09-19 are byte-identical**. Summaries: 39/8/1, 39/8/1, 38/8/2.

**All 8 WARNs and 1 FAIL are present in all three**, with first-ever appearance and span:

| Finding | First seen | Audits |
|---|---|---|
| Fabric: **326/491** cards have no edges | 2026-03-25 (**178 d**) | 90 |
| Found **77** GO-scope-not-propagated inceptions of 158 examined | 2026-06-06 (**105 d**) | 73 |
| No stale-slice-references (L-417) — NOT EVALUATED, candidate set empty | 2026-06-06 (**105 d**) | 73 |
| OBS-097 PROJECT_ROOT rail — NOT EVALUATED, candidate set empty | 2026-08-14 (36 d) | 22 |
| Fabric: 10 cards point at files no watch pattern covers | 2026-08-14 (36 d) | 22 |
| Onboarding-seed corpus refs — NOT EVALUATED, candidate set empty | 2026-08-25 (25 d) | 18 |
| Arc `mcp-slimming` has no task commits in 30 days | 2026-08-25 (25 d) | 18 |
| free driver F-ORCH: `retire_when` condition appears met | 2026-08-27 (23 d) | 16 |
| **FAIL** cron(substrate-smoke-canary): USER-field syntax but no install in /etc/cron.d | 2026-09-08 (11 d) | 7 |

A second FAIL — "Cron drift: `.context/cron/agentic-audit.crontab` differs from deployed `/etc/cron.d/agentic-audit-termlink`" — is in 09-17 but not 09-18/09-19. Both cron findings have open tasks (T-2938, T-2939, both `started-work`).

Note: three of the eight WARNs are `NOT EVALUATED: candidate set empty` — the check found nothing to scan and reported a warning rather than a result.

## What the large canary logs say, and since when

All under gitignored `.context/working/`, so **only `.substrate-preflight-canary.log` carries timestamps**; the other five write undated entries and only mtime is available. Of 21 canary logs, **6 are non-empty and 15 are zero-byte**. No `.heartbeat` companion files exist.

- **`.substrate-preflight-canary.log`** (80 KB, 74 entries, **2026-07-06 → 2026-09-19, daily 03:23 UTC**): same two warnings almost every day — installed binary older than project VERSION (71/74) and running hub older than VERSION (26/74); 0 FAILs. The gap widened from 53 commits (`0.11.324` vs `0.11.377`) to **222 commits** (`0.11.1716` vs `0.11.1938`) over 75 days.
- **`.framework-pickup-canary.log`** (64 KB, **63 entries**, undated, last write 2026-09-18): unprocessed inbound peer filings on `framework:pickup`. Ack offset advanced **42 → 99** across the file; backlog per firing ranged 2–26, most commonly 4 (23×). Entries list offsets, filing type, `from=`, and the `--ack` remediation; own filings are suppressed and counted (`10 own filing(s) from 010-termlink not counted`).
- **`.waker-liveness-canary.log`** (33 KB, **36 entries**, undated, last write 2026-09-19): **`RAIL DARK` in 36/36 (100%)**. 17 entries report 1 unwakeable LIVE agent; 14 report 4 dead wakers (`aef`, `sonnenstall`, `workflow-designer`, `workshop-designer`). Latest entry names `penelope` as presence-advertised with no `pty_session`.
- **`.stuck-claims-canary.log`** (8 KB, **7 entries, all byte-identical**, last write **2026-08-16 — 34 days ago**): `11 topic(s) with stuck/expired claims`. **Every listed topic has `active=0` and `oldest_active_age=0ms`** — the firing is driven entirely by expired rows (one topic carries 81). Topic names are demo/test debris (`substrate-drain-demo*`, `drain-probe-*`) plus `work-queue`.
- **`.hook-counter-integrity-canary.log`** (6 KB, **8 entries**, undated, last write 2026-09-16): hook-telemetry counter file corrupt — duplicate keys plus reader disagreement (e.g. `audit-task-tools  25  vs  67`). Names the mechanism (`unlocked truncate+write in lib/hook-telemetry.sh`) and states the effect: "each duplicate roughly halves the apparent failure ratio (false silence)".
- **`.stale-waker-code-canary.log`** (4 KB, **5 entries**, undated, last write **2026-08-13 — 37 days ago**): 3–4 pushwaker processes running pre-current code. Same four agents and same pids as the waker-liveness log.

## Open concerns (29: 27 watching + 2 decided-build)

G-005 high/decided-build `cargo install --git` no `--locked`, no CI coverage · G-008 64 tasks stuck partial-complete, no review workflow surfaces them · G-009 Proxmox .180 `/var/log` full → LXC reboot loop · G-010 T-212 Homebrew tap ticked but repo absent · G-011 hub secret cache drift · G-015 completion gate doesn't verify file-path claims in Agent ACs · G-053 DEFER/date-triggered follow-ups have no structural revisit mechanism · G-055 `fw upgrade` CLAUDE.md overwrite regresses project customizations · G-057 medium/decided-build MCP wrappers hide protocol fields · G-059 Human-AC over-deferral · G-061 blind to bus bridges falling back to retired primitives · G-062 Watchtower inception-decide strands staged evidence · G-063 `framework:pickup` write-only sink, zero receipts · G-064 hub has no per-user authorization model · G-065 BVP estimator corrupts anchor-less frontmatter · G-066 157 tasks reached `completed/` with finalization bypassed · G-067 pre-push audit ~80-100 s, intermittently blocks every push · G-068 Human-AC tick via Bash bypasses the hook undetected · G-069 fleet hubs run stale/deleted-exe binaries for weeks · G-070 hub unit crash-looped 2178× unnoticed · **G-083 high** wake layer has no consumption-confirmation · G-084 can't express "UP but doorbell-incapable" · **G-086 high** claim ownership rests on a spoofable `claimer` string · **G-087 high** identity-binding gap below the verification boundary · G-088 exactly-once doesn't survive hub restart · G-090 bus envelopes carry no project attribution · **G-091 high** project-boundary hook conflates checkout with project · **G-092 high** `fw cron install --help` performed the install · G-093 partial-complete pins focus, write gate then refuses that task's own commit.

## Handover / recurring-blocker signal

8 newest handovers span 2026-09-17 → 2026-09-19; all carry the same 17 section headers and **`Showing 5 of 165`** WIP total unchanged across all eight. Five themes recurring 8/8:

1. **Un-enriched handover** — `enrichment_status: pending`, 5 `[TODO]` sections each (8/8). Across all handovers carrying the field: **49 pending vs 7 enriched**. Two completed tasks already filed this exact finding (`d8`, `d8b`).
2. **Partial-complete / unchecked Human ACs** — 37 mentions; "**75 task(s) with unchecked Human ACs**".
3. **`pickup:` filings** — 36 mentions.
4. **Deferred with no revisit date** — the same block re-explained every session, including "There is **no CLI verb** that sets this field — verified, not assumed (T-2865)."
5. **Ripe revisits that stay ripe** — T-1898 ripe since 2026-07-06 (**75 d**), T-2250 since 2026-07-25 (**56 d**), plus T-2022/T-2024/T-2026.

Newest handover session metrics: 92.1M tokens / 393 turns, `commits_per_turn: 0.0483`, `first_commit_turn: 58`, `productive_turns_ratio: 0.369`, 44 uncommitted changes.

---

# Fragment C — Docs / skills / workflows (dispatched gatherer)

# Evidence Summary — Docs / CLAUDE.md / Skills (facts only)

Repo `/opt/termlink`, HEAD `e28e8abe7`, 2026-09-19. Binary `termlink 0.11.1716`.
Every "does not resolve" row was verified by **executing** `termlink <path> --help`
and reading the exit code (the `Commands:` help listing omits real subcommands, so a
listing-only method over-fires).

## Doc counts and staleness
- **390** `.md` under `docs/`. **309 (79.2%) are `docs/reports/`**; 44 `docs/operations/`;
  13 `docs/specs/`; **1** `docs/architecture/`; 7 loose at `docs/` root (incl. three
  `T-121x-settings-patch.md`).
- Largest: `docs/reports/T-908-api-relay-governance.md` at **212,654 B** — 2.6× the next.
  10 of the 15 largest are reports.
- **287 / 390 (73.6%) untouched ≥ 90 days**; **113 (29.0%) ≥ 180 days**; **0 ≥ 365 days**
  (oldest commit 2026-03-08). By directory: reports **239/309 (77.3%)** stale ≥90d,
  operations **16/44 (36.4%)**.
- 20 oldest are all `docs/reports/` from 2026-03-08→03-10, including a single 10-file
  batch `reflection-result-{arch,cli,e2e,enhance,evschema,proto,security,session,testcov,watcher}.md`.
- Oldest operations docs: `termlink-hub-runtime-migration.md` 2026-04-12,
  `api-usage-metrics.md` 2026-04-27, `ring20-dashboard-binary-upgrade.md` 2026-05-02.

## EVERY documented command that does not resolve
1. **`--retention-value` does not exist — it hard-errors.** `CLAUDE.md:652` (operator
   remediation for the T-2562 forever-archival canary), `CLAUDE.md:1989`, and
   `docs/operations/substrate-orchestrator-recipe.md:705,712,719`. Real:
   `--retention "messages:N"`. Running the documented form →
   `error: unexpected argument '--retention-value' found`.
   Related: `docs/reports/T-2214-…:85-87` records this as "silently ignored" — measured, it
   does not parse at all.
2. **`termlink deregister` does not exist.** `CLAUDE.md:166`, frozen-husk canary operator
   action. → `error: unrecognized subcommand 'deregister'`. (MCP tool `termlink_deregister`
   exists; the CLI verb does not.)
3. **`termlink channel create --name`** — `docs/operations/substrate-orchestrator-recipe.md:704`.
   NAME is positional; no `--name` flag.
4. **`termlink channel post --topic`** — `docs/operations/ring20-dashboard-binary-upgrade.md:285`.
   Topic is positional; `channel post` has no `--topic`.
5. **`termlink list-sessions`** — `docs/operations/topic-lint.md:123`. CLI verb is `termlink list`.
6. **`termlink identity rotate-file`** — `docs/migrations/T-1700-per-agent-identity.md:172`.
   `identity` has only `init show rotate`.
7. **`fw fleet …` — `fw` has no `fleet` subcommand** (`Unknown command: fleet`). Three sites:
   `ring20-dashboard-binary-upgrade.md:134` (`fw fleet deploy-binary`),
   `substrate-governor.md:250` (`fw fleet doctor`),
   `substrate-orchestrator-recipe.md:532` (`fw fleet bootstrap-check`). All should be `termlink fleet`.
8. **`scripts/t1418-checkin.sh`** absent — `ring20-dashboard-binary-upgrade.md:297`
   (conditional: line 291 tells the reader to create it from the existing `t1438-checkin.sh`).
9. **`scripts/agent-recent-dm.sh`** absent — `docs/plans/T-2296-…:192,195`; actual file is
   `scripts/recent-dm.sh`. Design-doc only.

**Clean controls:** `README.md`, `docs/CHARTER.md`, `docs/ARCHITECTURE.md` — zero
unresolvable references. `docs/operations/e2e-charter-validation-runbook.md:90` correctly
states `termlink agent listeners` does not exist (true negative, not drift).

**Not-drift but status-drift:** `docs/operations/agent-conversations.md` (81,768 B)
documents `channel react/pin/pinned/typing/star/starred/poll` as current at 11 sites; they
all still resolve, and are the P4 social-analytics surface CLAUDE.md records as deprecated.

**Wider `docs/` tree:** 66 raw hits; after the rows above the remainder are historical
design proposals, not operator instructions — `termlink relay start` (T-908),
`termlink dispatch init/defer/merge/clean` (T-789), `termlink hub trust/connect/inject`
(T-186), `termlink agent checkout/commit/publish` (T-2090), `termlink self-update` (T-1070),
`termlink token grant` (T-008), `termlink hub rotate-secret` (T-930).

## Zero-reference skills and scripts
- **34** skills in `.claude/commands/`. **5 have zero references in `docs/`, `scripts/`,
  `.context/handovers/` AND zero in CLAUDE.md** — the whole T-2209 family:
  `claims-history`, `find-idle-history`, `governor-history`, `queue-history`,
  `substrate-history`. Their only citations are `.tasks/active/T-2209-…` (still `active`)
  plus two completed tasks. `closeable` (T-2207) is next: 0 docs / 0 scripts / 0 CLAUDE.md.
- 8 of 34 skills are absent from CLAUDE.md entirely (the 5 above plus `capture`,
  `closeable`, `heartbeat`).
- `.claude/commands/substrate-history.md:3,66` is the **only DRIFT** the repo's own
  `lint-doc-cli-references.sh` reports repo-wide (cites `termlink substrate history`, PL-206).
- **190** `scripts/*.sh`; **62 (32.6%) named in no doc, CLAUDE.md, or README** —
  24 `test-*.sh`, 22 `check-*.sh`, 16 other (`agent-reply`, `hub-binary-swap`,
  `learnings-exchange`, `lint-command-hints`, `lint-doc-cli-references`,
  `lint-doc-fenced-bash`, `list-closeable`, `peer-presence-lookup`, `relay-hop-check`,
  `remediate-main-checkout`, `substrate-resilience-demo`, `verify-register-union`, …).
  15 of the 22 undocumented `check-*.sh` ARE guard-layer members, so they execute on every
  push/PR while appearing in no documentation.

## CLAUDE.md split
- **3,283 lines total.** `## Core Principle` at **line 2,461**.
  **2,460 lines (74.9%) above** the marker survive `fw upgrade`; **823 lines (25.1%) from
  the marker to EOF are destroyed and replaced** by the framework template.
- CLAUDE.md's own recorded measurement (2026-08-20) was 2,493 lines / marker at 1,650 /
  844 destroyed. The file has grown **+790 lines (+31.7%)** since, all above the split.
- **313 distinct `T-NNNN` IDs cited, 658 total mentions** (2.10 per ID); 38 distinct
  `PL-`/`G-`/`L-`/`D-`/`C-`/`P-` governance refs.

## Workflows and capability registry
- **Workflows: DESIGNED-ONLY.** 13 `.bpmn` files across **8** designer projects under
  `.agentic-framework/.context/designer/projects/` (`aef-task-lifecycle` v1–v3,
  `aef-dispatch-loop` v1–v3, `aef-greenfield-onboarding` v1–v2, plus `aef-audit-cron`,
  `aef-inception-flow`, `aef-session-lifecycle`, `aef-tier0-escalation`,
  `aef-existing-project-onboarding` at v1). **No `meta.json` carries any `ratified`,
  `approved`, `signed`, or `status` field** — only `{id,title,versions,latest,updated,uuid}`.
  `.agentic-framework/tools/conformance-registry.yaml` states "Maps absent from this
  registry are descriptive-only by definition … CLAUDE.md prose wins", and its
  `gate-referent` primitive is "not yet implemented".
- **`.agentic-framework/policy/capabilities.yaml`: ABSENT** — `find . -name capabilities.yaml`
  returns zero hits repo-wide. Nearest artifact is
  `.agentic-framework/policy/capability-overlay/tool-set.yaml` (1 file, 9,897 B).
- **Prompt-drift report: ABSENT** — no file matching `*prompt*drift*`, no
  `prompt.drift` text in `docs/` or `.agentic-framework/docs/`.
- `policy/prompts/`: **5** entries (`arc-delivery-session.md`, `artefact-template.md`,
  `bvp-driver-session.md`, `bvp-references/`, `README.md`).
  `policy/standards/`: **2** (`aef-bpmn-mapping-v1-partI.md` + its `.provenance.yaml`).

## Charter / doc-lint / CI
- `check-charter-sentence-drift.sh --no-heartbeat` → **exit 0**, "in-sync — all 3 surfaces
  carry the identical canonical sentence."
- `check-error-code-docs.sh` → **exit 0**, "CLEAN — all SYMBOL(-320NN) pairings match control.rs".
- `.github/workflows/` holds **3 files**: `doc-lint.yml` (push/PR; jobs
  `error-code-doc-lint`, `env-var-doc-lint`, `guard-layer`), `install-check.yml` (push/PR;
  job `install-from-git`), `release.yml` (tag push; jobs `test`, `test-macos`,
  `build-macos`, `build-linux`, `release`).
- `run-guard-layer.sh` (113 members: 38 static-checks + 75 fixture-suites) runs in both
  `doc-lint.yml` and `release.yml`. **`lint-doc-cli-references.sh` and
  `lint-doc-fenced-bash.sh` appear in no workflow and are not guard-layer members** — they
  are the only two repo tools targeting the documented-command-does-not-resolve class, and
  both found real hits when run manually here.

---

# Fragment D — Rust code structure (dispatched gatherer)

# Evidence: Rust workspace — /opt/termlink

Gatherer output. **Evidence only — no classification, no recommendations.**

- Repo: `/opt/termlink`, branch `main`, HEAD `e28e8abe7`
- Collection date: 2026-09-19
- Window for churn/fix analysis: `--since="12 months ago"` (≈2025-09-19 → 2026-09-19)
- Scope: `crates/termlink-{cli,mcp,hub,bus,protocol,session,test-utils}`
- Toolchain runs: `cargo build --workspace` (exit 0), `cargo check --workspace --all-targets` in isolated `CARGO_TARGET_DIR=/tmp/vr1/target` (exit 0), plus per-crate `cargo check -p <crate> --all-targets`

Kind legend: `metric` = counted/measured · `artifact` = file/decl read · `cmd-output` = command result · `derived` = computed from two measured inputs

---

## 1. Size and shape

### 1.1 LOC per crate

Method: `find crates/<c> -name '*.rs' -type f -exec cat {} + | wc -l`

Item | Source | Source status | Data point (citation) | Window | Kind
---|---|---|---|---|---
LOC termlink-cli | `crates/termlink-cli` | EXISTS | **72,790 LOC** across 33 `.rs` files | HEAD | metric
LOC termlink-mcp | `crates/termlink-mcp` | EXISTS | **50,828 LOC** across 6 `.rs` files | HEAD | metric
LOC termlink-hub | `crates/termlink-hub` | EXISTS | **24,550 LOC** across 26 `.rs` files | HEAD | metric
LOC termlink-session | `crates/termlink-session` | EXISTS | **23,838 LOC** across 42 `.rs` files | HEAD | metric
LOC termlink-bus | `crates/termlink-bus` | EXISTS | **5,240 LOC** across 8 `.rs` files | HEAD | metric
LOC termlink-protocol | `crates/termlink-protocol` | EXISTS | **3,093 LOC** across 8 `.rs` files | HEAD | metric
LOC termlink-test-utils | `crates/termlink-test-utils` | EXISTS | **347 LOC** across 1 `.rs` file | HEAD | metric
Workspace total | all 7 crates | EXISTS | **180,686 LOC** across 124 `.rs` files | HEAD | derived
cli+mcp share | derived | EXISTS | 123,618 / 180,686 = **68.4%** of workspace LOC in 2 crates | HEAD | derived
File-count asymmetry | derived | EXISTS | termlink-mcp = 50,828 LOC in **6 files** (avg 8,471 LOC/file); termlink-session = 23,838 LOC in **42 files** (avg 568 LOC/file) | HEAD | derived

### 1.2 The 20 largest individual `.rs` files

Method: `find crates -name '*.rs' -exec wc -l {} + | sort -rn`

\# | File | LOC | >5000?
---|---|---|---
1 | `crates/termlink-mcp/src/tools.rs` | **46,458** | **YES**
2 | `crates/termlink-cli/src/commands/channel.rs` | **20,435** | **YES**
3 | `crates/termlink-cli/src/commands/remote.rs` | **11,891** | **YES**
4 | `crates/termlink-cli/src/cli.rs` | **7,001** | **YES**
5 | `crates/termlink-hub/src/channel.rs` | 4,949 | no
6 | `crates/termlink-cli/src/commands/agent.rs` | 4,258 | no
7 | `crates/termlink-cli/tests/cli_integration.rs` | 4,208 | no
8 | `crates/termlink-hub/src/router.rs` | 3,843 | no
9 | `crates/termlink-hub/src/server.rs` | 3,723 | no
10 | `crates/termlink-bus/src/lib.rs` | 3,091 | no
11 | `crates/termlink-session/src/handler.rs` | 2,827 | no
12 | `crates/termlink-cli/src/commands/infrastructure.rs` | 2,562 | no
13 | `crates/termlink-mcp/tests/mcp_integration.rs` | 2,366 | no
14 | `crates/termlink-cli/src/commands/substrate.rs` | 2,150 | no
15 | `crates/termlink-cli/src/main.rs` | 1,931 | no
16 | `crates/termlink-cli/src/commands/pty.rs` | 1,764 | no
17 | `crates/termlink-cli/src/commands/session.rs` | 1,662 | no
18 | `crates/termlink-cli/src/commands/events.rs` | 1,554 | no
19 | `crates/termlink-mcp/tests/parity.rs` | 1,493 | no
20 | `crates/termlink-cli/src/commands/metadata.rs` | 1,386 | no

**Files over 5000 lines: 4.** Combined = 46,458 + 20,435 + 11,891 + 7,001 = **85,785 LOC = 47.5% of the entire workspace in 4 files.**

`tools.rs` alone is 46,458 LOC = **25.7% of workspace LOC**, and is **91.4%** of its own crate (46,458 / 50,828).

---

## 2. Hotspots (churn × size)

Method: for each of the top-20 largest files, `git log --since="12 months ago" --oneline -- <path> | wc -l`

### 2.1 Raw churn and LOC

File | Commits (12mo) | LOC
---|---|---
`crates/termlink-cli/src/main.rs` | **421** | 1,931
`crates/termlink-mcp/src/tools.rs` | **389** | 46,458
`crates/termlink-cli/src/cli.rs` | **360** | 7,001
`crates/termlink-cli/src/commands/channel.rs` | 166 | 20,435
`crates/termlink-cli/src/commands/remote.rs` | 144 | 11,891
`crates/termlink-hub/src/router.rs` | 94 | 3,843
`crates/termlink-cli/src/commands/session.rs` | 78 | 1,662
`crates/termlink-cli/src/commands/agent.rs` | 70 | 4,258
`crates/termlink-hub/src/server.rs` | 60 | 3,723
`crates/termlink-hub/src/channel.rs` | 58 | 4,949
`crates/termlink-cli/tests/cli_integration.rs` | 58 | 4,208
`crates/termlink-cli/src/commands/infrastructure.rs` | 58 | 2,562
`crates/termlink-cli/src/commands/events.rs` | 48 | 1,554
`crates/termlink-cli/src/commands/metadata.rs` | 42 | 1,386
`crates/termlink-mcp/tests/mcp_integration.rs` | 41 | 2,366
`crates/termlink-bus/src/lib.rs` | 39 | 3,091
`crates/termlink-session/src/handler.rs` | 30 | 2,827
`crates/termlink-mcp/tests/parity.rs` | 27 | 1,493
`crates/termlink-cli/src/commands/pty.rs` | 25 | 1,764
`crates/termlink-cli/src/commands/substrate.rs` | 10 | 2,150

### 2.2 Churn × LOC ranking — top 15

Rank | File | Commits | LOC | churn×LOC
---|---|---|---|---
1 | `crates/termlink-mcp/src/tools.rs` | 389 | 46,458 | **18,072,162**
2 | `crates/termlink-cli/src/commands/channel.rs` | 166 | 20,435 | **3,392,210**
3 | `crates/termlink-cli/src/cli.rs` | 360 | 7,001 | **2,520,360**
4 | `crates/termlink-cli/src/commands/remote.rs` | 144 | 11,891 | **1,712,304**
5 | `crates/termlink-cli/src/main.rs` | 421 | 1,931 | 812,951
6 | `crates/termlink-hub/src/router.rs` | 94 | 3,843 | 361,242
7 | `crates/termlink-cli/src/commands/agent.rs` | 70 | 4,258 | 298,060
8 | `crates/termlink-hub/src/channel.rs` | 58 | 4,949 | 287,042
9 | `crates/termlink-cli/tests/cli_integration.rs` | 58 | 4,208 | 244,064
10 | `crates/termlink-hub/src/server.rs` | 60 | 3,723 | 223,380
11 | `crates/termlink-cli/src/commands/infrastructure.rs` | 58 | 2,562 | 148,596
12 | `crates/termlink-cli/src/commands/session.rs` | 78 | 1,662 | 129,636
13 | `crates/termlink-bus/src/lib.rs` | 39 | 3,091 | 120,549
14 | `crates/termlink-mcp/tests/mcp_integration.rs` | 41 | 2,366 | 97,006
15 | `crates/termlink-session/src/handler.rs` | 30 | 2,827 | 84,810

Rank-1 `tools.rs` score is **5.3×** rank-2 and **larger than ranks 2–15 combined** (18,072,162 vs 10,531,210).

The top 4 by churn×LOC are exactly the 4 files over 5000 LOC.

---

## 3. Fix / revert concentration

Method: `git log --since='12 months ago' --oneline -i --grep='fix' --grep='bug' --grep='regression' --grep='revert' --grep='hotfix'` (OR-semantics, case-insensitive)

Item | Source | Source status | Data point (citation) | Window | Kind
---|---|---|---|---|---
Total commits | `git log` | EXISTS | **7,167** commits | 12mo | cmd-output
Fix-ish commits | `git log -i --grep` | EXISTS | **1,144** commits = **16.0%** of all commits | 12mo | cmd-output
…containing "fix" | `git log -i --grep=fix` | EXISTS | 955 | 12mo | cmd-output
…containing "bug" | `git log -i --grep=bug` | EXISTS | 166 | 12mo | cmd-output
…containing "regression" | `git log -i --grep=regression` | EXISTS | 158 | 12mo | cmd-output
…containing "revert" | `git log -i --grep=revert` | EXISTS | 91 | 12mo | cmd-output
…containing "hotfix" | `git log -i --grep=hotfix` | EXISTS | 1 | 12mo | cmd-output

Per-keyword counts sum to 1,371 > 1,144 because commits may match several keywords.

### 3.1 Top files by fix-commit count

Method: `git log --since='12 months ago' --name-only --format='%n' -i --grep=… -- 'crates/**/*.rs' | grep '^crates/.*\.rs$' | sort | uniq -c | sort -rn`

Rank | File | Fix-commits | LOC | fix-commits per 1k LOC
---|---|---|---|---
1 | `crates/termlink-mcp/src/tools.rs` | **81** | 46,458 | 1.74
2 | `crates/termlink-cli/src/main.rs` | **58** | 1,931 | **30.0**
3 | `crates/termlink-cli/src/commands/channel.rs` | **53** | 20,435 | 2.59
4 | `crates/termlink-cli/src/cli.rs` | **49** | 7,001 | **7.00**
5 | `crates/termlink-cli/src/commands/remote.rs` | 34 | 11,891 | 2.86
6 | `crates/termlink-hub/src/server.rs` | 22 | 3,723 | 5.91
7 | `crates/termlink-hub/src/router.rs` | 22 | 3,843 | 5.72
8 | `crates/termlink-hub/src/channel.rs` | 22 | 4,949 | 4.44
9 | `crates/termlink-cli/src/commands/infrastructure.rs` | 21 | 2,562 | 8.20
10 | `crates/termlink-cli/src/commands/agent.rs` | 18 | 4,258 | 4.23
11 | `crates/termlink-bus/src/lib.rs` | 15 | 3,091 | 4.85
12 | `crates/termlink-cli/tests/cli_integration.rs` | 13 | 4,208 | 3.09
13 | `crates/termlink-cli/src/commands/events.rs` | 12 | 1,554 | 7.72
14 | `crates/termlink-cli/src/commands/session.rs` | 11 | 1,662 | 6.62
15 | `crates/termlink-bus/src/meta.rs` | 11 | — | —
16 | `crates/termlink-cli/src/commands/dispatch.rs` | 10 | 1,229 | 8.14
17 | `crates/termlink-session/src/client.rs` | 8 | — | —
18 | `crates/termlink-mcp/tests/mcp_integration.rs` | 8 | 2,366 | 3.38
19 | `crates/termlink-cli/src/commands/file.rs` | 8 | 1,281 | 6.25
20 | `crates/termlink-cli/src/commands/substrate.rs` | 7 | 2,150 | 3.26

The same 4 files lead on all three axes (size, churn×LOC, fix-count): `tools.rs`, `channel.rs`, `cli.rs`, `main.rs`/`remote.rs`.

---

## 4. Duplication between CLI and MCP crates (T-2069 convention)

The repo documents (CLAUDE.md, T-2069) a deliberate convention of duplicating "tiny pure helpers" into `tools.rs` rather than sharing across crates. Measurement follows.

### 4.1 Counts

Method: `grep -oE 'fn +[A-Za-z0-9_]+_mcp' crates/termlink-mcp/src/tools.rs`

Item | Source | Source status | Data point (citation) | Window | Kind
---|---|---|---|---|---
`_mcp` fn definitions (occurrences) | `crates/termlink-mcp/src/tools.rs` | EXISTS | **191** definition sites | HEAD | metric
`_mcp` fn **distinct names** | same | EXISTS | **81** distinct names | HEAD | metric
Confirmed duplicate pairs | cross-crate grep | EXISTS | **67 of 81** distinct names have a same-name-minus-`_mcp` fn in `crates/termlink-cli/src` | HEAD | metric
Unpaired `_mcp` names | same | EXISTS | **14 of 81** (list in §4.4) | HEAD | metric
Pairing rate | derived | EXISTS | 67/81 = **82.7%** | HEAD | derived

**Counting-unit note.** CLAUDE.md cites three circulating figures (83 / 68 / 94) and states the unit is ill-defined. Measured here: **81 distinct names, 191 definition sites.** The gap is name reuse inside separate scopes — `to_json_mcp` alone is defined **26 times**, `analyze_pl021_mcp` 12×, `compute_quote_stats_mcp` 11×, `compute_edit_stats_mcp` 11×, `stats_mcp` 9×, `compute_poll_state_mcp` 8×, `ack_status_mcp` 7×, `payload_matches_mcp` 6×.

### 4.2 LOC involved

Method: awk brace-matching from each `fn` header to its closing brace.

Measure | Value | Note
---|---|---
All `_mcp` fn bodies in `tools.rs` (191 sites) | **2,972 LOC** | includes repeated names
Paired-only `_mcp` fn bodies (67 names, all sites) | **2,459 LOC** | —
First definition per paired name, MCP side | **1,357 LOC** | apples-to-apples
First definition per paired name, CLI side | **1,772 LOC** | apples-to-apples
**Combined duplicated-pair surface (first-def, both sides)** | **3,129 LOC** | 67 pairs
Upper bound incl. all repeat sites | ≈**4,231 LOC** | 2,459 MCP (all sites) + 1,772 CLI

**Estimate: ~3,100–4,200 LOC** of paired near-duplicate helper code across the two crates.

### 4.3 Ten confirmed pairs with both citations

Base name | MCP side | CLI side | MCP LOC | CLI LOC
---|---|---|---|---
`compute_full_topic_stats` | `crates/termlink-mcp/src/tools.rs:5356` | `crates/termlink-cli/src/commands/channel.rs:6164` | 120 | 125
`compute_digest` | `crates/termlink-mcp/src/tools.rs:5905` | `crates/termlink-cli/src/commands/channel.rs:5562` | 110 | 111
`compute_poll_state` | `crates/termlink-mcp/src/tools.rs:11750` | `crates/termlink-cli/src/commands/channel.rs:5362` | 108 | 111
`compute_threads_index` | `crates/termlink-mcp/src/tools.rs:3852` | `crates/termlink-cli/src/commands/channel.rs:8440` | 87 | 89
`compute_state` | `crates/termlink-mcp/src/tools.rs:4559` | `crates/termlink-cli/src/commands/channel.rs:7274` | 87 | 88
`compute_edit_stats` | `crates/termlink-mcp/src/tools.rs:6136` | `crates/termlink-cli/src/commands/channel.rs:7103` | 79 | 80
`compute_quote_stats` | `crates/termlink-mcp/src/tools.rs:6253` | `crates/termlink-cli/src/commands/channel.rs:7680` | 74 | 74
`compute_pinned_set` | `crates/termlink-mcp/src/tools.rs:5513` | `crates/termlink-cli/src/commands/channel.rs:4960` | 63 | 65
`analyze_pl021` | `crates/termlink-mcp/src/tools.rs:189` | `crates/termlink-cli/src/commands/remote.rs:4256` | 63 | 53
`aggregate_topics_probes` | `crates/termlink-mcp/src/tools.rs:444` | `crates/termlink-cli/src/commands/events.rs:1117` | — | —

Additional confirmed pairs (sample): `aggregate_find_idle_entries` (`tools.rs:710` ↔ `commands/agent_find_idle.rs:630`), `aggregate_queue_entries` (`tools.rs:850` ↔ `commands/channel.rs:10561`), `aggregate_substrate_entries` (`tools.rs:925` ↔ `commands/substrate.rs:1276`), `auth_mismatch_class` (`tools.rs:10848` ↔ `commands/remote.rs:6493`), `build_thread` (`tools.rs:6918` ↔ `commands/channel.rs:3677`), `compute_ack_history` (`tools.rs:4946` ↔ `commands/channel.rs:8165`), `compute_ack_status` (`tools.rs:5730` ↔ `commands/channel.rs:8628`), `compute_active_typers` (`tools.rs:3761` ↔ `commands/channel.rs:4740`), `compute_edits_of` (`tools.rs:5053` ↔ `commands/channel.rs:8295`).

**LOC near-equality** across pairs (120/125, 110/111, 108/111, 87/89, 87/88, 79/80, 74/74, 63/65) is direct evidence the pairs are near-verbatim rather than merely same-named.

**Content spot-check — `compute_digest`, first 26 lines diffed:** 5 diff hunks, all cosmetic. `crates/termlink-mcp/src/tools.rs:5905` reads `fn compute_digest_mcp(envelopes: &[serde_json::Value], since_ms: i64) -> DigestSummaryMcp` vs `crates/termlink-cli/src/commands/channel.rs:5562` `pub(crate) fn compute_digest(envelopes: &[Value], since_ms: i64) -> DigestSummary`; remaining differences are `serde_json::Value` vs the imported alias `Value`, and `std::collections::HashSet` fully-qualified vs `use`-imported. Loop bodies and logic are identical.

### 4.4 The 14 unpaired `_mcp` names

`ack_status_mcp`, `agent_threads_parent_offset_of_mcp`, `build_cli_help_json_matches_mcp`, `compute_ancestors_mcp`, `cursor_list_for_fingerprint_mcp`, `fleet_by_project_mcp`, `fleet_mcp`, `fleet_presence_mcp`, `harden_mcp`, `resolve_mcp`, `resolve_message_or_file_mcp`, `shape_agent_request_events_mcp`, `stats_mcp`, `warn_mcp`

---

## 5. Dead code

### 5.1 Tooling availability — nothing installed

Item | Source | Source status | Data point (citation) | Window | Kind
---|---|---|---|---|---
`cargo udeps` | `which cargo-udeps`, `cargo udeps --version` | **ABSENT** | `error: no such command: 'udeps'`; not in `~/.cargo/bin` | HEAD | cmd-output
`cargo machete` | `which cargo-machete`, `cargo machete --version` | **ABSENT** | `error: no such command: 'machete'`; not in `~/.cargo/bin` | HEAD | cmd-output

Nothing was installed, per instruction.

### 5.2 Compiler dead-code warnings

`cargo build --workspace` → **exit 0, 0 warnings.** That run compiled only 4 of 7 crates (rest cached; cached crates do not re-emit warnings), so it is **not** sufficient evidence of a clean tree. Two further runs were made:

1. `CARGO_TARGET_DIR=/tmp/vr1/target cargo check --workspace --all-targets` (cold isolated target dir) → **exit 0, 175 dependency crates compiled, 3 warning lines**
2. Per-crate `cargo check -p <crate> --all-targets --message-format=short` for all 7 crates → confirms attribution

Crate | Warnings | Detail
---|---|---
termlink-protocol | 0 | —
termlink-session | **2** | see below
termlink-hub | 0 | —
termlink-mcp | 0 | —
termlink (cli) | 0 | —
termlink-bus | 0 | —
termlink-test-utils | 0 | —

**Total actual dead-code warnings workspace-wide: 2** (not 20; the full list is exhausted at 2).

```
crates/termlink-session/src/inbox_channel.rs:601:12: warning: struct `EnvGuard` is never constructed
crates/termlink-session/src/inbox_channel.rs:606:12: warning: associated functions `set` and `unset` are never used
    = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default
warning: `termlink-session` (lib test) generated 2 warnings
```

Both are in `termlink-session` **lib-test** configuration (`inbox_channel.rs:601` / `:606`, `fn set` at :606, `fn unset` at :615) — i.e. an unused test fixture, not production code. They surface only with `--all-targets`.

### 5.3 `#[allow(dead_code)]` counts per crate

Crate | `allow(dead_code)` occurrences
---|---
termlink-cli | 2
termlink-test-utils | 2
termlink-session | 1
termlink-bus | 0
termlink-hub | 0
termlink-mcp | 0
termlink-protocol | 0
**Total** | **5**

The 2 real warnings are not suppressed, and suppression is not being used to hide dead code at scale (5 sites across 180,686 LOC).

---

## 6. Tests

### 6.1 Test counts per crate

Method: `grep -rn '#\[test\]' / '#\[tokio::test'` per crate.

Crate | `#[test]` | `#[tokio::test]` | Total | `#[ignore]` | Tests per 1k LOC
---|---|---|---|---|---
termlink-cli | 1,300 | 25 | **1,325** | 5 | 18.2
termlink-mcp | 911 | 143 | **1,054** | 0 | 20.7
termlink-session | 305 | 211 | **516** | 0 | 21.6
termlink-hub | 333 | 177 | **510** | 0 | 15.6
termlink-bus | 26 | 87 | **113** | 0 | 21.6
termlink-protocol | 106 | 0 | **106** | 0 | 34.3
termlink-test-utils | 4 | 1 | **5** | 0 | 14.4
**Total** | **2,985** | **644** | **3,629** | **5** | 20.1

### 6.2 `#[ignore]` tests — all 5, with reason

All ignored tests are in one file, all with the same stated reason (PTY allocation):

Citation | Reason (verbatim from source)
---|---
`crates/termlink-cli/tests/interactive_integration.rs:87` | `#[ignore] // requires PTY allocation`
`crates/termlink-cli/tests/interactive_integration.rs:122` | `#[ignore] // requires PTY allocation`
`crates/termlink-cli/tests/interactive_integration.rs:149` | `#[ignore] // requires PTY allocation`
`crates/termlink-cli/tests/interactive_integration.rs:174` | `#[ignore] // requires PTY allocation`
File-level note | `crates/termlink-cli/tests/interactive_integration.rs:6` — `//! Tests are marked #[ignore] by default since they require PTY allocation`

Ignored share: 5 / 3,629 = **0.14%**.

### 6.3 Integration test files in `crates/*/tests/`

File | LOC
---|---
`crates/termlink-cli/tests/cli_integration.rs` | 4,208
`crates/termlink-mcp/tests/mcp_integration.rs` | 2,366
`crates/termlink-mcp/tests/parity.rs` | 1,493
`crates/termlink-session/tests/integration.rs` | 1,081
`crates/termlink-session/tests/claim_client_integration.rs` | 773
`crates/termlink-hub/tests/no_legacy_callers.rs` | 337
`crates/termlink-cli/tests/channel_claim_cli_integration.rs` | 329
`crates/termlink-session/tests/no_spoke_mesh_tripwire.rs` | 299
`crates/termlink-hub/tests/no_federation_tripwire.rs` | 223
`crates/termlink-cli/tests/interactive_integration.rs` | 196
`crates/termlink-session/tests/bus_client_integration.rs` | 176
**11 files** | **11,481 LOC**

Three are architectural tripwires rather than behaviour tests: `no_legacy_callers.rs`, `no_spoke_mesh_tripwire.rs`, `no_federation_tripwire.rs` (859 LOC combined).

`crates/termlink-mcp/tests/parity.rs` (1,493 LOC) is the CLI↔MCP parity suite referenced in §4; per CLAUDE.md it covers 24 of 260 MCP tools (9.2%), with the remaining 236 enumerated in `.context/checks/mcp-parity-census-allowlist`.

---

## 7. Dependencies

### 7.1 Workspace-level declarations

`Cargo.toml` (workspace root) — `[workspace.package] version = "0.9.0"`, `edition = "2024"`, 7 members. `[workspace.dependencies]` declares: tokio, serde, serde_json, thiserror, anyhow, tracing, tracing-subscriber, ulid, bytes, bitflags, clap, hmac, sha2, rand, base64, tokio-rustls, rustls, rustls-pemfile, rcgen, reqwest, tokio-tungstenite, futures-util, regex, libc + the 6 internal path crates.

Two dependency choices carry inline rationale comments in the root `Cargo.toml`: the `reqwest` crypto-provider pin (T-2333, aws-lc-rs vs ring conflict) and the `tokio-tungstenite` no-TLS-features choice (T-2305).

### 7.2 Direct dependencies per crate

Crate | Direct `[dependencies]`
---|---
termlink-protocol | serde, serde_json, thiserror, bytes, ulid, bitflags
termlink-bus | serde, serde_json, thiserror, sha2, tokio, **rusqlite 0.33 (bundled)** — dev: tempfile
termlink-test-utils | tokio, serde_json, termlink-protocol, termlink-session, libc
termlink-session | termlink-protocol, serde, serde_json, thiserror, tokio, tracing, libc, hmac, sha2, rand, base64, regex, tokio-rustls, rustls, rustls-pemfile, tokio-tungstenite, futures-util, **ed25519-dalek 2**, **rand_core 0.6**, **toml 0.8**, **rusqlite 0.33** — dev: termlink-test-utils, tempfile
termlink-hub | termlink-protocol, termlink-session, termlink-bus, **ed25519-dalek 2**, base64, serde, serde_json, **serde_yaml 0.9**, anyhow, thiserror, tokio, tracing, libc, tokio-rustls, rustls, rustls-pemfile, rcgen, tokio-tungstenite, futures-util, reqwest, hmac, sha2 — dev: tempfile
termlink-mcp | **rmcp ~1.3**, **schemars 1**, tokio, serde, serde_json, anyhow, tracing, libc, base64, sha2, regex, termlink-protocol, termlink-session, termlink-hub — dev: rmcp (client feats), termlink-test-utils, tokio, tempfile
termlink-cli | termlink-protocol, termlink-session, termlink-hub, termlink-mcp, tokio, tracing, tracing-subscriber, anyhow, clap, **clap_complete 4**, serde_json, libc, base64, sha2, serde, **toml 0.8**, **vte 0.13**, **unicode-width 0.1**, regex — dev: assert_cmd, predicates, tempfile, **rexpect 0.5**, termlink-test-utils

Note: `termlink-cli` depends on all three sibling libs **plus** `termlink-mcp`; `termlink-mcp` depends on `termlink-hub`. Dependency direction is cli → mcp → hub → session → protocol, with bus consumed by hub only.

### 7.3 Declared-but-unreferenced dependency candidates

Method: for each declared dep, `grep -rE "\b(<name_with_underscores>|<name>)\b" <crate>/src --include='*.rs'`; zero hits = candidate. Verified by hand afterward.

Candidate | Citation | Evidence | Verdict-supporting detail
---|---|---|---
**`ulid`** in termlink-protocol | `crates/termlink-protocol/Cargo.toml` `[dependencies] ulid = { workspace = true }` | **1 hit total in `src/`, and it is a doc comment**: `crates/termlink-protocol/src/events.rs:141: /// Unique request identifier (ULID recommended).` | No code reference anywhere in the crate. Strongest candidate.
`serde_json` in termlink-test-utils | `crates/termlink-test-utils/Cargo.toml` | 0 hits in `src/` (crate has a single file, `src/lib.rs`) | —
`termlink-protocol` in termlink-test-utils | `crates/termlink-test-utils/Cargo.toml` | 0 hits for `termlink_protocol`/`termlink-protocol` in `src/` | —

Method sanity-check (guards against false positives from macro-path or fully-qualified use): `bytes` → 29 hits in termlink-protocol; `bitflags` → 1 hit and it is real code, `crates/termlink-protocol/src/data.rs:37: bitflags::bitflags! {`. So the method does catch fully-qualified macro invocations; `ulid`'s single hit being a `///` comment is therefore meaningful.

Caveat: the grep is over `src/` only. A dep used exclusively from `tests/`, `benches/` or `build.rs` would show as a candidate; none of the three candidates' crates have `tests/` or `build.rs` except as noted (termlink-test-utils has neither; termlink-protocol has no `tests/` dir in the integration-test listing in §6.3).

---

## 8. Module orphans

Method: enumerate `mod x;` declarations; extract each module's public items via `^pub (async )?(unsafe )?(struct|enum|fn|trait|type|const|static) NAME`; count workspace-wide references to each item name excluding the module's own file. Modules where **all** public items have 0 external references are candidates; each was then verified by hand.

**Methodology correction applied:** a first pass omitted `pub async fn` from the item regex and consequently mis-flagged `crates/termlink-session/src/ws_consumer.rs` as an orphan. It is **not** — `stream_ws_events` is consumed at `crates/termlink-cli/src/commands/channel.rs:496` (`use termlink_session::ws_consumer::stream_ws_events;`). The figures below use the corrected regex.

### 8.1 Confirmed orphan modules (4)

Module | Citation | LOC | Public items | Ext refs | Internal tests
---|---|---|---|---|---
`termlink-hub::trust` | `crates/termlink-hub/src/trust.rs`, declared `pub mod trust;` at `crates/termlink-hub/src/lib.rs:18` | **468** | `BlastRadius`, `Familiarity`, `Maturity`, `SupervisionLevel`, `TrustAssessment`, `TrustInput` | **0/6 referenced** | 14
`termlink-hub::template_cache` | `crates/termlink-hub/src/template_cache.rs`, declared `pub mod template_cache;` at `crates/termlink-hub/src/lib.rs:16` | **538** | `TemplateCache`, `TemplateEntry`, `TemplateLookup`, `compute_schema_hash`, `cache_path`, `PROMOTION_THRESHOLD` | **0/6 referenced** (see note) | 13
`termlink-session::assignment` | `crates/termlink-session/src/assignment.rs`, declared `pub mod assignment;` at `crates/termlink-session/src/lib.rs:15` | **357** | `AgentProfile`, `ArtifactRef`, `Assignment`, `ParseError`, `ResultManifest`, `ResultStatus`, `SCHEMA_ASSIGNMENT`, `SCHEMA_RESULT_MANIFEST` | **0/8 referenced** | 9
`termlink-session::known_peers` | `crates/termlink-session/src/known_peers.rs`, declared `pub mod known_peers;` at `crates/termlink-session/src/lib.rs:5` | **265** | `KnownPeers`, `PeerEntry`, `PeersError`, `Result` | **0/4 referenced** (see note) | 7
**Total** | 4 modules | **1,628 LOC** | 24 public items | **0 external references** | **43 tests**

**`template_cache` note.** The raw grep showed `cache_path` (12 hits) and `PROMOTION_THRESHOLD` (4 hits) as "used", but inspection shows every hit resolves to a **different module**: `crates/termlink-hub/src/router.rs:1297` and `:1339` call `crate::route_cache::cache_path()`, and `PROMOTION_THRESHOLD` is separately defined at `crates/termlink-hub/src/bypass.rs:37` and used at `bypass.rs:165` and `router.rs:1631` (`crate::bypass::PROMOTION_THRESHOLD`). No hit references `template_cache::`. Confirmed 0 external references.

**`known_peers` note.** `Result` showed 765 hits; it is a crate-local alias declared at `crates/termlink-session/src/known_peers.rs:41` (`pub type Result<T> = std::result::Result<T, PeersError>;`), and the 765 hits are the ubiquitous `std` `Result`. The three distinctive items `KnownPeers`, `PeerEntry`, `PeersError` each have exactly **0** external references. Confirmed.

**Cross-reference check:** `grep` for the module names across `crates`, `scripts`, `docs` found `template_cache` mentioned in `docs/ARCHITECTURE.md`, and `assignment` mentioned in `scripts/substrate-resilience-demo.sh` and `docs/architecture/parallel-execution-substrate.md` — i.e. documented/scripted, but with no Rust call site.

**Status qualifier:** all four are `pub mod` on library crates, so they are public API surface and a downstream consumer outside this workspace could in principle import them. Within this workspace there are zero call sites. Each is exercised only by its own in-file `#[cfg(test)]` block.

### 8.2 Non-orphans excluded after verification (6)

Module | Workspace refs | Note
---|---|---
`termlink-session::ws_consumer` | `stream_ws_events` used | `crates/termlink-cli/src/commands/channel.rs:496`; 6 of its 7 other pub items unreferenced
`termlink-protocol::control` | 395 refs | heavily used cross-crate
`termlink-protocol::events` | 48 refs | used
`termlink-protocol::data` | 7 refs | used
`termlink-protocol::governance` | 1 ref | used
`termlink-session::fleet_presence` | 12 refs | used
`termlink-session::data_server` | 8 refs | used
`termlink-session::endpoint` | 3 refs | used

---

## 9. Static checks

All seven scripts EXIST and were run as specified. Every one exits 0.

Check | Command | Exit | JSON verdict | Firing | Checked | Acknowledged
---|---|---|---|---|---|---
Platform lock | `bash scripts/check-platform-lock.sh --json` | **0** | `{"ok":true,"firing":[],"checked":5,"allowlisted":5}` | **0** | 5 | 5 allowlisted
Alloc-sink clamps | `bash scripts/check-alloc-sink-clamps.sh --json` | **0** | `{"ok":true,"firing":[],"checked":103,"candidates":0}` | **0** | 103 | 0 candidates
Drain-sink caps | `bash scripts/check-drain-sink-caps.sh --json` | **0** | `{"ok":true,"firing":[],"checked":6,"candidates":0}` | **0** | 6 | 0
Silent exit | `bash scripts/check-silent-exit.sh --json` | **0** | `{"ok":true,"firing":[],"checked":39,"candidates":0}` | **0** | 39 | 0
Busy spin | `bash scripts/check-busy-spin.sh --json` | **0** | `{"ok":true,"firing":[],"checked":14,"candidates":0}` | **0** | 14 | 0
Version derivation | `bash scripts/check-version-derivation.sh --json` | **0** | `{"ok":true,"firing":[],"checked":4,"candidates":0,"acknowledged_count":0,"acknowledged":[]}` | **0** | 4 crates | 0
Error-code emission | `bash scripts/check-error-code-emission.sh --json` | **0** | `{"ok":true,"firing":[],"checked":24,"allowlisted":3}` | **0** | 24 codes | 3 allowlisted

Notes carried by the checks' own documented scope (CLAUDE.md), relevant to reading these exits:

- **platform-lock**: 5 sites scanned, **all 5 allowlisted** — i.e. zero unacknowledged, not zero platform-coupled sites. The allowlist requires each entry to state non-Linux behaviour.
- **alloc-sink-clamps**: 103 sink calls scanned. CLAUDE.md documents 5 allowlisted confirmed-safe sites; the JSON reports `candidates:0` (nothing unacknowledged).
- **drain-sink-caps**: only 6 sink calls in scope; CLAUDE.md notes zero `read_to_end`/`read_to_string`/`collect::<Vec<u8>>` drains exist, so the first one added would fire.
- **error-code-emission**: 24 codes checked, **3 declared-reserved** (`SESSION_BUSY` -32002, `MESSAGE_EXPIRED` -32004, `PROTOCOL_VERSION_TOO_OLD` -32011 per CLAUDE.md T-2698 — codes with zero emission sites, acknowledged rather than emitted).
- **version-derivation**: 4 crates read the version (cli/mcp/hub/session); all 4 derive it; allowlist empty.

Build/check status alongside: `cargo build --workspace` exit **0**; cold `cargo check --workspace --all-targets` exit **0** with the 2 dead-code warnings in §5.2 as the only findings.

---

## Appendix: commands used

```
find crates/<c> -name '*.rs' -type f -exec cat {} + | wc -l
find crates -name '*.rs' -type f -exec wc -l {} + | sort -rn
git log --since="12 months ago" --oneline -- <path> | wc -l
git log --since='12 months ago' --oneline -i --grep='fix' --grep='bug' --grep='regression' --grep='revert' --grep='hotfix'
git log --since='12 months ago' --name-only --format='%n' -i --grep=... -- 'crates/**/*.rs' | grep -E '^crates/.*\.rs$' | sort | uniq -c | sort -rn
grep -oE 'fn +[A-Za-z0-9_]+_mcp' crates/termlink-mcp/src/tools.rs
cargo build --workspace
CARGO_TARGET_DIR=/tmp/vr1/target cargo check --workspace --all-targets
CARGO_TARGET_DIR=/tmp/vr1/target cargo check -p <crate> --all-targets --message-format=short
which cargo-udeps cargo-machete ; cargo udeps --version ; cargo machete --version
grep -rn '#\[test\]' / '#\[tokio::test' / '#\[ignore' crates --include='*.rs'
bash scripts/check-{platform-lock,alloc-sink-clamps,drain-sink-caps,silent-exit,busy-spin,version-derivation,error-code-emission}.sh --json
```

Intermediate artifacts retained: `/tmp/vr1/{build.out,check.out,mcp_fns.txt,pairs.txt,unpaired.txt,mcp_loc.txt,pairs_loc.txt,orphans.txt,orph2.txt}`

---

# Fragment E — orchestrator's own backup Rust measurements

| Item | Data point | Kind |
|---|---|---|
| Largest .rs files | tools.rs 46458 · cli/commands/channel.rs 20435 · cli/commands/remote.rs 11891 · cli/cli.rs 7001 · hub/channel.rs 4949 · cli/commands/agent.rs 4258 · cli/tests/cli_integration.rs 4208 · hub/router.rs 3843 · hub/server.rs 3723 · bus/lib.rs 3091 | structure |
| Concentration | the 3 largest files = 78,784 LOC = 44% of the 180,686-LOC workspace | structure |
| `#[allow(dead_code)]` | cli 2 · session 1 · test-utils 2 · bus/hub/mcp/protocol 0 | structure |
| `#[ignore]` tests | 4, all in `crates/termlink-cli/tests/interactive_integration.rs:87,122,149,174` — "requires PTY allocation". Baseline run confirms `4 ignored`. These are the only tests of the interactive PTY path (charter verb 4, the founding verb) | structure |
| Baseline verdict | `cargo test --workspace --no-fail-fast` exit 0: **2962 passed, 0 failed, 4 ignored** across 10 targets | structure |

---

# Fragment F — scripts / guard layer / cron (dispatched gatherer)

# Evidence — scripts/, tests/, cron/canary/guard layer (/opt/termlink)

Gathered 2026-09-19. Facts only; no classification, no recommendation.
Commands run from `/opt/termlink` on branch `main`.

| Item | Source | Source status | Data point (citation) | Window | Kind |
|---|---|---|---|---|---|
| **1. SCRIPT CENSUS** | | | | | |
| scripts/ total files | `ls scripts/ \| wc -l` | EXISTS | 198 (190 `.sh`, 6 `.py`, 1 `.testdata`, 1 `.lib`) | snapshot | count |
| `scripts/check-*.sh` | `ls scripts/check-*.sh \| wc -l` | EXISTS | 66 | snapshot | count |
| non-`check-*` scripts | `ls scripts/ \| grep -vc '^check-'` | EXISTS | 131 | snapshot | count |
| largest non-check family | `ls scripts/ \| grep -v '^check-'` | EXISTS | `test-*` = 38; `substrate-*` = 11; `agent-*` = 10; `demo-*` = 9; `fleet-*` = 3 | snapshot | count |
| `# guard-layer:` marker present | `grep -l '^# guard-layer:' scripts/check-*.sh` | EXISTS | 38 of 66 check scripts carry the marker | snapshot | count |
| `# guard-layer:` marker ABSENT | same, complement | EXISTS | 28 of 66 (list below) | snapshot | count |
| runner's own unclassified tally | `run-guard-layer.sh --list` | EXISTS | "unclassified checks + suites (**75**) — no '# guard-layer:' marker" (28 checks + 47 `tests/test-*`/misc suites) | snapshot | count |
| **2. CRON WIRING** | | | | | |
| `.context/cron/` entries | `ls .context/cron/` | EXISTS | 27 files; 25 are `*.crontab`, plus `fleet-version-floors.conf` and `fleet-dm-canary-transient` | snapshot | count |
| cron-install-drift verdict | `bash scripts/check-cron-install-drift.sh --json` | EXISTS | `{"ok":false,"missing_count":1,"uninstalled_jobs_count":0,"job_drift_count":0,"drift_count":0,"acknowledged_count":0,"ok_count":26,"skipped_count":0}` rc=1 | live | verdict |
| MISSING (1) | same | EXISTS | `substrate-smoke-canary.crontab → /etc/cron.d/termlink-substrate-smoke-canary` | live | finding |
| UNINSTALLED_JOBS / JOB_DRIFT / DRIFT | same | EXISTS | 0 / 0 / 0 | live | count |
| OK | same | EXISTS | 26 | live | count |
| `/etc/cron.d` termlink-matching entries | `ls /etc/cron.d/ \| grep -iE 'termlink\|canary\|...'` | EXISTS | 29 files: 3 `agentic-*-termlink` + 26 `termlink-*` (incl. `termlink-watchdog`, `termlink-heartbeat`, `termlink-presence-sweep`) | snapshot | count |
| `/etc/cron.d` total | `ls /etc/cron.d/` | EXISTS | 64 files; 30 are `agentic-audit-*` for OTHER projects on the same host | snapshot | context |
| **3. CANARY FIRING STATE (snapshot, logs read not run)** | | | | | |
| canary logs present | `.context/working/.*-canary.log` | EXISTS | 21 logs | snapshot | count |
| EMPTY (=healthy) | `stat -c%s` per log | EXISTS | 15 of 21: charter-drift, charter-sentence-drift, dead-letter, fleet-binary, fleet-capability, forever-archival, frozen-husk, preflight-doc-set-drift, release-mirror, session-control, task-finalization, topic-growth, unconfirmed-delivery, woken-but-silent | snapshot | state |
| NON-EMPTY (content present) | same | EXISTS | 6: framework-pickup (63993 B / 1175 lines), substrate-preflight (80430 B / 994), waker-liveness (33097 B / 341), stuck-claims (8197 B / 112), hook-counter-integrity (6307 B / 120), stale-waker-code (4216 B / 51), fleet-doorbell-mail (418 B / 6) — 7 counting doorbell-mail | snapshot | state |
| newest mtimes on non-empty logs | `stat -c%y` | EXISTS | waker-liveness 2026-09-19 07:53; substrate-preflight 2026-09-19 05:23; framework-pickup 2026-09-18 06:37; hook-counter 2026-09-16 07:49; stuck-claims 2026-08-16 07:27; stale-waker 2026-08-13 08:13; doorbell-mail 2026-08-10 09:23 | snapshot | date |
| framework-pickup canary content | head/tail of log | EXISTS | head: `framework-pickup canary: 5 unprocessed filing(s) on framework:pickup (acked up to offset 42)`; tail: `off=120  pickup-bug-report  from=root` / `(10 own filing(s) from 010-termlink not counted — outbound, not inbound work)` | to 2026-09-18 | quote |
| waker-liveness canary content | head/tail | EXISTS | head: `waker-liveness canary: FIRING — 0 unwakeable LIVE agent(s), 4 dead waker(s), RAIL DARK`; tail: `[rail-dark] ZERO LIVE listeners carry pty_session on this hub — the G-069 '0 wakers' state.` | to 2026-09-19 | quote |
| substrate-preflight canary content | tail | EXISTS | `Summary: 4 pass, 2 warn, 0 fail — substrate will run but fix warnings when convenient.` (newest framed ts `2026-09-19T03:23:01`) | to 2026-09-19 | quote |
| hook-counter-integrity content | head/tail | EXISTS | head: `check-hook-counter-integrity: FIRING — counter file is corrupt (18 line(s) scanned)`; tail: `MECHANISM: unlocked truncate+write in lib/hook-telemetry.sh _fw_telemetry_increment. This is L-023 recurring.` | to 2026-09-16 | quote |
| stale-waker-code content | head/tail | EXISTS | head: `stale-waker-code canary: FIRING — 4 waker(s) running PRE-CURRENT code`; tail: `STALE workshop-designer: pushwaker pid 2079757 started 1786106186 < code 1786201032 (running old waker code)` | to 2026-08-13 | quote |
| stuck-claims content | head | EXISTS | `check-stuck-claims: 11 topic(s) with stuck/expired claims (verb-3 claim-work detection, T-2556)` | to 2026-08-16 | quote |
| fleet-doorbell-mail content | head/tail | EXISTS | head: `Fleet doorbell+mail health: DRIFT`; tail lines: `ring20-management@192.168.10.122:9100: verdict=setup-fail elapsed=3101ms`, `ring20-dashboard@192.168.10.121:9100: verdict=setup-fail`, `workstation-107-public@...: verdict=pass` | to 2026-08-10 | quote |
| `.stderr` companions | `ls .context/working/.*-canary.log.stderr` | EXISTS | 20 present; 18 are 0 bytes | snapshot | count |
| non-empty stderr (1) | `cat .release-mirror-canary.log.stderr` | EXISTS | 150 B / 6 lines, all `error: origin HEAD empty`; mtime 2026-09-16 07:13 — while the findings log is EMPTY (healthy) | to 2026-09-16 | quote |
| non-empty stderr (2) | `cat .stuck-claims-canary.log.stderr` | EXISTS | 88 B / 1 line: `check-stuck-claims: could not read claims-summary (exit=1); hub down or binary too old?`; mtime 2026-08-19 | to 2026-08-19 | quote |
| heartbeats present | `ls .context/working/.*heartbeat*` | EXISTS | 30 heartbeat files; 9 are stale (last touched Aug 29 or earlier): alloc-sink, busy-spin, drain-sink, error-swallowing, silent-exit, unbounded-rpc-call (all Aug 29), stranded-finalized (Sep 17), task-template-idioms (Sep 1) | snapshot | state |
| **4. GUARD LAYER RUNNER (BASELINE)** | | | | | |
| members | `run-guard-layer.sh --list` | EXISTS | **113** (38 static-check + 74 fixture-suite/suite + 1 static-check `fabric-workflow-link.sh`) | live | count |
| roll-up | `run-guard-layer.sh --json` | EXISTS | `{"total":113,"passed":109,"fired":4,"errored":0,"unclassified":75,"with_tests":false,"exit_code":1}`, `ok:false` | live | verdict |
| FAIL 1 | same, rc=1 | EXISTS | `check-installed-binary-drift.sh` — `DRIFT(1): 2 distinct versions across installed paths: 0.11.1716 0.11.1766`; build artifact at 0.11.1921 | live | finding |
| FAIL 2 | same | EXISTS | `check-pickup-deferred-freshness.sh` — `1 envelope(s) STRANDED … STRANDED: P-078-learning.yaml (deferred 9 days ago, age from mtime — unreliable in a fresh checkout)` | live | finding |
| FAIL 3 | same | EXISTS | `check-receiver-ack-lag.sh` — 4× `NEVER-ACKED … lag=1531`, 1× `BEHIND d1993c2c3ec44c94 lag=607 (up_to=923)` on `agent-chat-arc` (threshold 25) | live | finding |
| FAIL 4 | same | EXISTS | `tests/cron-drift-firing-fixtures.sh` — `FAIL the real tree passes the firing check`; `T-2821 fixtures: 12 passed, 1 failed` (fixture asserts the real tree FIRES; it does, via the missing substrate-smoke crontab — the assertion is inverted vs current state) | live | finding |
| ERROR / SKIP | same | EXISTS | 0 ERROR, 0 SKIP | live | count |
| **5. ORPHANED SCRIPTS** | | | | | |
| zero-reference scripts (any ref outside own file, excl. `.context/episodic/`) | `git grep -F -f <all basenames> -- . ':!.context/episodic'` → 26891 hits | EXISTS | **0** — every one of the 198 scripts is referenced somewhere | snapshot | count |
| confound: fabric cards | `.fabric/components/scripts-<name>.yaml` | EXISTS | every script has an auto-generated card that mentions its own basename; this alone guarantees ≥1 reference | snapshot | caveat |
| fixture-suite-ONLY referenced | derived | EXISTS | **0** | snapshot | count |
| no reference from cron / `.claude/` / `.github/` / another script / crates | derived from hit map | EXISTS | **79 of 198** (many are guard-layer members discovered by marker-glob, not by name — run-guard-layer.sh enumerates dynamically) | snapshot | count |
| referenced ONLY by `.tasks/` + `.context/` + own fabric card (no docs, no tests, no CLAUDE.md, no caller) | derived | EXISTS | **29** — list below | snapshot | finding |
| **6. TEST FIXTURES** | | | | | |
| `tests/` total | `ls tests/ \| wc -l` | EXISTS | 85 | snapshot | count |
| `tests/*fixtures*.sh` | `ls tests/*fixtures*.sh \| wc -l` | EXISTS | 62 | snapshot | count |
| other `tests/` files | derived | EXISTS | 23 | snapshot | count |
| orphan fixtures (reference no `scripts/*.sh\|.py`) | grep per fixture | EXISTS | **6**: `gitignore-framework-scope-fixtures.sh`, `handover-suggested-action-fixtures.sh`, `l387-boundary-fixtures.sh`, `pickup-failopen-fixtures.sh`, `reap-topic-fixtures.sh`, `revisit-due-cron-fixtures.sh` | snapshot | finding |
| orphan checks (no fixture names them) | grep per check | EXISTS | **27 of 66** — list below | snapshot | finding |
| **7. CHURN (6 months)** | | | | | |
| scripts/ | `git log --since="6 months ago" --oneline -- scripts/ \| wc -l` | EXISTS | 345 | 6 mo | count |
| tests/ | same | EXISTS | 165 | 6 mo | count |
| docs/ | same | EXISTS | 516 | 6 mo | count |
| .tasks/ | same | EXISTS | 4147 | 6 mo | count |
| crates/ | same | EXISTS | 1362 | 6 mo | count |
| repo total | `git log --since="6 months ago" --oneline \| wc -l` | EXISTS | 6770 | 6 mo | count |

---

## List A — 28 `scripts/check-*.sh` WITHOUT `# guard-layer: source`

check-addressed-posts.sh, check-approval-surfaces.sh, check-canary-aliveness.sh,
check-charter-drift-freshness.sh, check-cron-install-drift.sh,
check-dashboard-deleted-root.sh, check-dead-letter-freshness.sh,
check-fleet-binary-freshness.sh, check-fleet-capability-freshness.sh,
check-fleet-doorbell-mail-health.sh, check-forever-archival-freshness.sh,
check-framework-pickup-freshness.sh, check-frozen-husk-freshness.sh,
check-guard-runner-coverage.sh, check-hook-counter-integrity.sh,
check-mirror-freshness.sh, check-outbox.sh, check-pickup-cron-lock.sh,
check-session-control-freshness.sh, check-stale-waker-code-freshness.sh,
check-stuck-claims-freshness.sh, check-substrate-smoke-freshness.sh,
check-task-finalization-freshness.sh, check-topic-growth-freshness.sh,
check-unconfirmed-delivery-freshness.sh, check-vendored-arc-rollout.sh,
check-verification-pipefail.sh, check-waker-liveness-freshness.sh

(Most are documented as runtime cron canaries, which CLAUDE.md states belong to cron
rather than the source-guard runner. Exceptions visible from §2: `check-cron-install-drift.sh`,
`check-guard-runner-coverage.sh`, `check-verification-pipefail.sh`, `check-outbox.sh`,
`check-approval-surfaces.sh`, `check-addressed-posts.sh`, `check-dashboard-deleted-root.sh`,
`check-pickup-cron-lock.sh`, `check-vendored-arc-rollout.sh`, `check-canary-aliveness.sh`
are not in `.context/cron/` nor in the runner.)

## List B — 29 scripts referenced ONLY by `.tasks/` + `.context/` + own fabric card

(no docs mention, no test, no CLAUDE.md entry, no cron line, no caller script)

| script | .tasks refs |
|---|---|
| check-fabric-card-parse.sh | 0 |
| check-fleet-recipient-agreement.sh | 1 |
| check-hubs-parse-agreement.sh | 1 |
| check-task-frontmatter.sh | 2 |
| check-verification-legs.py | 2 |
| field-status.sh | 1 |
| independent-review.py | 1 |
| lint-doc-fenced-bash.sh | 7 |
| t1697-human-ac-audit.py | 1 |
| test-agent-chat-arc-recent.sh | 2 |
| test-agent-conversation-list.sh | 9 |
| test-agent-conversation-selftest.sh | 2 |
| test-agent-listeners-fleet.sh | 3 |
| test-agent-listeners.sh | 6 |
| test-agent-send-auto-discover.sh | 8 |
| test-be-reachable.sh | 2 |
| test-charter-drift-freshness.sh | 2 |
| test-charter-sentence-drift.sh | 2 |
| test-chat-arc-broadcast.sh | 2 |
| test-check-fleet-binary-freshness.sh | 2 |
| test-check-fleet-doorbell-mail-health.sh | 3 |
| test-check-framework-pickup-freshness.sh | 2 |
| test-check-unconfirmed-delivery-freshness.sh | 2 |
| test-fleet-adoption-snapshot.sh | 5 |
| test-fleet-rearm-wakers.sh | 3 |
| test-pushwaker-reap.sh | 2 |
| test-substrate-preflight.sh | 2 |
| test-tl-claude-cmd.sh | 2 |
| test-watchtower-guard.sh | 3 |

Note: `check-fabric-card-parse.sh` carries `# guard-layer: source` and IS executed by the
runner (marker discovery), but has zero prose/task/doc references — its only text mention
anywhere is its own fabric card.

## List C — 27 `scripts/check-*.sh` with NO fixture suite naming them

check-approval-surfaces.sh, check-charter-sentence-drift.sh, check-dead-letter-freshness.sh,
check-env-var-docs.sh, check-error-code-docs.sh, check-fabric-card-parse.sh,
check-fleet-binary-freshness.sh, check-fleet-capability-freshness.sh,
check-fleet-doorbell-mail-health.sh, check-fleet-recipient-agreement.sh,
check-forever-archival-freshness.sh, check-frozen-husk-freshness.sh,
check-hubs-parse-agreement.sh, check-installed-binary-drift.sh, check-mirror-freshness.sh,
check-preflight-doc-set-drift.sh, check-receiver-ack-lag.sh, check-session-control-freshness.sh,
check-stale-waker-code-freshness.sh, check-task-finalization-freshness.sh,
check-task-frontmatter.sh, check-topic-growth-freshness.sh,
check-unconfirmed-delivery-freshness.sh, check-unpaired-capture.sh,
check-vacuous-verification.sh, check-vendored-arc-rollout.sh, check-waker-liveness-freshness.sh

Note: `check-charter-sentence-drift.sh` has a `scripts/test-charter-sentence-drift.sh`
(under scripts/, not tests/) — likewise `check-charter-drift-freshness.sh`,
`check-fleet-binary-freshness.sh`, `check-fleet-doorbell-mail-health.sh`,
`check-framework-pickup-freshness.sh`, `check-unconfirmed-delivery-freshness.sh`. Those
`scripts/test-*` files are in List B (referenced only by tasks/context).

---

## GATHERER RECONCILIATION — two sub-gatherer claims corrected by direct measurement

1. **Canary heartbeats exist.** One gatherer reported "No `.heartbeat` companion files exist". FALSE — they are dotfiles and a `*.heartbeat` glob misses them. Direct `find`: **31 heartbeat files, one per canary log**. `scripts/check-canary-aliveness.sh` runs and reports freshness. NOT carried into the report.
2. **Stale-heartbeat count.** One gatherer reported "9 stale, 6 last touched 2026-08-29 (alloc-sink, busy-spin, drain-sink, error-swallowing, silent-exit, unbounded-rpc-call)". Direct measurement at 2026-09-19 19:05 with a >48h threshold returns **3**: `.unbounded-rpc-call-canary` 510h (21d), `.error-swallowing-canary` 507h (21d), `.task-template-idioms-check` 428h (17d). The other three named are fresh. **3 is the number carried into the report.**
3. **Binary version disagreement explained.** One gatherer observed `termlink 0.11.1716`, the orchestrator observed `0.11.1766`. `check-installed-binary-drift.sh` independently FIRES with `DRIFT(1): 2 distinct versions across installed paths: 0.11.1716 0.11.1766` — so both observations are correct and the disagreement is itself the finding.
