# Value Review — Evidence File (GATHERER output)

- **Scope:** whole repo (`/opt/termlink`)
- **Date:** 2026-09-19
- **Run:** run4 of a 5-run consecutive repeatability series (independent; prior runs' outputs not read)
- **Task:** T-2971
- **Role:** GATHERER. Facts only. No classification, no recommendations.
- **Snapshot time:** 2026-09-19T18:31:59Z (all usage data read before this review generated any)

---

## S0. Baseline

| Check | Command | Result |
|---|---|---|
| Rust workspace tests | `cargo test --workspace` | **3618 passed, 0 failed, 4 ignored**, exit 0 |
| Product LOC (Rust, `crates/*/src` + tests) | `find crates -name '*.rs' \| xargs wc -l` | **180,686** |
| Shell LOC (`scripts/*.sh`) | `cat scripts/*.sh \| wc -l` | **39,455** |
| Shell LOC (`tests/*.sh`) | `cat tests/*.sh \| wc -l` | **12,704** |
| Guard-layer runner | `bash scripts/run-guard-layer.sh --json` | **did not finish within 12 min** (191 declared members; per-member timeout 300s). Result unavailable at report time. |
| `fw` version | `.agentic-framework/bin/fw version` | v1.6.29, vendored |
| `termlink` binary | `termlink --version` | 0.11.1766 |
| `VERSION` file | `cat VERSION` | 0.11.1944 |

---

## S1. Project shape

| Item | Source | Data point | Kind |
|---|---|---|---|
| S1.1 | `ls crates/` | 7 crates: bus, cli, hub, mcp, protocol, session, test-utils | structure |
| S1.2 | `wc -l` per crate | cli 67,954 · mcp 46,891 · hub 23,932 · session 21,120 · bus 5,240 · protocol 3,093 · test-utils 347 (src only) | structure |
| S1.3 | `grep -c '#\[(tokio::)?test\]'` | tests per crate: cli 1324 · mcp 1048 · session 510 · hub 420 · bus 109 · protocol 106 | structure |
| S1.4 | `ls scripts/*.sh` | **190** shell scripts; 66 are `check-*.sh` (14,176 LOC) | structure |
| S1.5 | `ls tests/*.sh` | 83 fixture suites | structure |
| S1.6 | `find docs -name '*.md'` | 394 markdown docs, `du -sb docs/` = 4,070,414 bytes | structure |
| S1.7 | `ls .context/cron/` | 28 crontab/config files | structure |
| S1.8 | `ls /etc/cron.d` | 24 `termlink-*` jobs installed + `agentic-audit-termlink`, `agentic-pickup-termlink`, `agentic-learnings-exchange-termlink`, `termlink-watchdog` | structure |
| S1.9 | `ls .claude/commands/*.md` | 34 skills, `du -sb` = 255,164 bytes | structure |
| S1.10 | `git rev-list --count HEAD` | 7,172 commits total; 2,022 in last 90d; 447 in last 30d | structure |

---

## S2. Purpose source (`docs/CHARTER.md`, read verbatim)

| Item | Data point |
|---|---|
| S2.1 | Canonical sentence: "TermLink is a hub-mediated, durable append-log message bus with terminal endpoints — the coordination substrate that lets a fleet of AI agents (and humans) discover each other, exchange durable messages, claim work, and control terminal sessions across one or many machines." Marked human-blessed. |
| S2.2 | Four core verbs: discover · exchange durable messages · claim work · control terminal sessions. |
| S2.3 | Five declared non-goals: (1) not inter-hub federation; (2) **not a durable database / system of record**; (3) **not a social/engagement platform**; (4) **not a workflow or orchestration engine** — "the substrate stays mechanism, not policy"; (5) not a security boundary between mutually-distrusting tenants. |
| S2.4 | `CLAUDE.md:4` states "For the provider-neutral framework guide, see `FRAMEWORK.md`." — `ls FRAMEWORK.md` → **No such file or directory** at repo root. The file exists only at `.agentic-framework/FRAMEWORK.md` and `.agentic-framework.rollback/FRAMEWORK.md`. |

**Value drivers** (`.agentic-framework/policy/value-drivers.yaml`, v3): protected D1 Antifragility (w9), D2 Reliability, D3 Usability, D4 Portability; plus ≤5 free drivers. Weights are §ACD sovereignty-gated.

---

## S3. Always-loaded context cost

| Item | Source | Data point | Kind |
|---|---|---|---|
| S3.1 | `wc -lc CLAUDE.md` | **3,283 lines / 276,749 bytes ≈ 69,000 tokens**, loaded into every session | cost |
| S3.2 | `git show <rev>:CLAUDE.md \| wc -c` at month boundaries | 2026-04: 43,069 B / 850 ln · 05: 52,126 / 1,013 · 06: 72,414 / 1,186 · 07: 141,355 / 1,398 · 08: 153,058 / 1,554 · 09: 268,670 / 3,169 · now: 276,749 / 3,283 | cost |
| S3.3 | derived from S3.2 | **6.4× growth in 5 months**; +115,612 bytes (+1,615 lines) in the single month 2026-08 → 2026-09 | cost |
| S3.4 | `grep -n '^## Core Principle' CLAUDE.md` | line 2,461. Per the file's own T-2015 note, `fw upgrade` replaces everything from this line to EOF; **822 lines are in the destroyed half**, 2,460 lines are project-specific prelude | structure |
| S3.5 | `grep -n '^### ' CLAUDE.md` (lines 40–1948) | **~42 sections**, each 20–70 lines, one per cron canary or static check. Span lines 40–1948 ≈ **1,908 lines ≈ 58% of the file** | cost |
| S3.6 | `awk` header count on three sample check scripts | `check-charter-drift-freshness.sh` 73 header lines (289 total); `check-stuck-claims-freshness.sh` 35 (158); `check-verification-pipefail.sh` 55 (188) — each script carries its own rationale header | structure |
| S3.7 | `grep -c <task-id> CLAUDE.md` + `grep -rln` in scripts/docs | For T-2483, T-2556, T-2696, T-2818, T-2800: each appears in CLAUDE.md **and** in the corresponding `scripts/check-*.sh` header **and** (4 of 5) in `docs/reports/` | structure |
| S3.8 | `help.json` parse | MCP tool name+description bytes total **21,971 ≈ 5,492 tokens** — i.e. CLAUDE.md costs ~12.6× the entire MCP tool catalogue's descriptive text | cost |

---

## S4. MCP / CLI surface

| Item | Source | Data point | Kind |
|---|---|---|---|
| S4.1 | `termlink help --json` | **260 tools total: 214 live, 46 deprecated**, across 29 categories | structure |
| S4.2 | same | Categories whose tools are wholly deprecated: `agent_engagement_metrics` (0 live / 8 dep), `channel_poll` (0/4), `agent_poll` (0/3) | structure |
| S4.3 | same | Live analytics-category tools (charter-drift "category detector"): `agent_stats` 10, `agent_thread_health` 8, `agent_rankings` 5, `channel_engagement` 5 | structure |
| S4.4 | `bash scripts/check-charter-drift-freshness.sh --json` | `checked: 214, off_charter_total: 28, acknowledged: 28, firing: 0` — 28 live tools classified off-charter, all suppressed by allowlist | structure |
| S4.5 | `.context/checks/charter-drift-allowlist` header | Allowlist text states the 28 are "an open question … pending T-2548 (`started-work`, `owner: human`)" | claimed |
| S4.6 | `ls .tasks/completed/T-2548*` | **T-2548 is in `.tasks/completed/`**, `status: work-completed`, `date_finished: 2026-08-20T17:54:28Z` | observed |
| S4.7 | T-2548 `## Decision` | **"Decision: GO"** — "GO (subtract), gated on IW-1 external-consumer check." Rationale cites: zero first-party callers (grep of `scripts/`, `.claude/commands/`, `crates/termlink-cli/src` returned empty); fails all four charter verbs; direct non-goal #4 violation. KEEP-list named: `agent_search_thread`, `agent_thread_path`, `agent_recent_window`. | observed |
| S4.8 | `grep -rl T-2548 .tasks/active/` | Referenced only by T-2470 (the purpose-reconcile task) and T-2971 (this review). **No build task exists to execute the GO.** 30 days elapsed. | observed |
| S4.9 | `git log -S'deprecated P4/T-2478'` | 46 tools deprecated in commit `6778f92bb`, **2026-08-02** — 48 days ago. `docs/operations/p4-surface-reduction.md` states: "Deletion is a **later** step, gated on a deprecation soak and a separate human go-ahead." | observed |
| S4.10 | `bash scripts/check-mcp-parity-census.sh --json` | `acknowledged: 236, coverage_pct: 9.2, firing: 0` — 24 of 260 tools have a parity assertion | structure |
| S4.11 | regex over both crates | **71** distinct `fn *_mcp` helpers in `termlink-mcp`; **67** of them have an identically-named non-`_mcp` twin in `termlink-cli` — duplicated implementation pairs by the T-2069 convention ("no cross-crate sharing") | structure |
| S4.12 | S4.11 sample | Duplicated pairs include `compute_emoji_stats`, `compute_reactions_of`, `compute_reactions_on`, `compute_pin_history`, `compute_pinned_set`, `compute_poll_state`, `compute_quote_stats` — helpers serving the deprecated social-analytics set | structure |
| S4.13 | CLAUDE.md § MCP parity census | Records that `parity_topics` drifted undetected **from 2026-08-12** until T-2686 wired CI; cause was a CLI change not mirrored to MCP | observed |

---

## S5. Hotspots and duplication in product code

| Item | Source | Data point | Kind |
|---|---|---|---|
| S5.1 | `git log --since=180.days --name-only -- crates/` | Top churn: `termlink-mcp/src/tools.rs` **390** touches · `cli/src/cli.rs` 356 · `cli/src/main.rs` 351 · `cli/commands/channel.rs` 166 · `cli/commands/remote.rs` 142 | structure |
| S5.2 | `wc -l` | `tools.rs` **46,458 LOC in one file**; `cli/commands/channel.rs` 20,435; `cli/commands/remote.rs` 11,891. These three = **78,784 LOC = 44%** of all product Rust | structure |
| S5.3 | S5.1 × S5.2 | The two highest-churn files are also the two largest (churn × size hotspot) | structure |
| S5.4 | `git log --since=180.days --oneline -- crates/` | 1,329 crate commits; **133 (10.0%)** match `fix\|revert\|regress\|bug` | structure |

---

## S6. Runtime / fleet usage (out-of-band observer: `fleet-adoption-snapshot`, daily cron)

`.context/working/.fleet-adoption-snapshot.log` — 89 daily snapshots, 74,962 bytes, last write 2026-09-19 09:47. This is a **separate observer** from the bus itself (it probes each hub), so it is not the channel reporting on itself.

| Item | Data point | Kind |
|---|---|---|
| S6.1 | State distribution across 89 snapshots: **HOT 34 · WARM 48 · COLD 7** | usage |
| S6.2 | Segmented by thirds — `chat_arc_posts` daily average: **oldest ⅓ = 270.4 · middle ⅓ = 12.1 · newest ⅓ = 31.9** (max ever 813) | usage |
| S6.3 | **Last 10 consecutive daily snapshots: `chat_arc_posts = 0` and `unique_speakers = 0`** | usage |
| S6.4 | `dm_topics_active` daily average: oldest ⅓ **226.8** → middle ⅓ 182.3 → newest ⅓ **13.2** | usage |
| S6.5 | `live_listeners` average: 2.7 → 4.6 → 2.3; last-10 snapshots reachable hubs = 2 or 3 of 4 | usage |
| S6.6 | `termlink agent listeners --json` at snapshot time: **empty result — zero LIVE listeners** | usage |
| S6.7 | `termlink channel list --json`: 45 topics, **4,303 records total**. Distribution: `health:ring20-fedprobe` 1,670 (38.8%) · `agent-presence` 1,022 (23.8%) · `agent-chat-arc` 1,001 (23.3%) · `channel:learnings` 191 · `framework:pickup` 126. **All `dm:*` topics combined ≈ 200 records (4.6%)** | usage |
| S6.8 | `termlink fleet doctor --json`: 5 hub entries; `192.168.10.141` status `error`; `.121` and `.122` serve **0.11.1411** vs local **0.11.1766** (≈355 commits behind) | usage |
| S6.9 | `.context/working/.fleet-adoption-snapshot.log.stderr` (3,680 B): repeated `skipping duplicate 127.0.0.1:9100 (same hub as 192.168.10.107:9100)` | usage |

---

## S7. Guard layer — firing state (snapshot, read before this review ran anything)

`ls -la .context/working/.*-canary.log` — 22 canary logs present, 30 `.heartbeat` companions.

**Empty (healthy) logs (15):** charter-drift, charter-sentence-drift, dead-letter, fleet-binary, fleet-capability, forever-archival, frozen-husk, preflight-doc-set-drift, release-mirror, session-control, task-finalization, topic-growth, unconfirmed-delivery, woken-but-silent.

**Non-empty (FIRING) logs (7):**

| Item | Canary | Size / lines | Distinct days present | Last-entry content (verbatim extract) |
|---|---|---|---|---|
| S7.1 | `substrate-preflight` | 80,430 B / 994 ln | **74 distinct dates, 2026-07-06 → 2026-09-19** | `[WARN] binary termlink 0.11.1716 older than project VERSION 0.11.1938` + `[WARN] hub-binary running hub serves 0.11.1766, older than project VERSION 0.11.1938`; `Summary: 4 pass, 2 warn, 0 fail` |
| S7.2 | `waker-liveness` | 33,097 B / 341 ln | n/a (no ISO dates in body) | `FIRING — 1 unwakeable LIVE agent(s), 0 dead waker(s), RAIL DARK`; `[rail-dark] ZERO LIVE listeners carry pty_session on this hub — the G-069 '0 wakers' state. Every DM sent here waits on the ~15s poll floor at best, forever at worst.` |
| S7.3 | `framework-pickup` | 63,993 B / 1,175 ln | 1 | Unprocessed inbound filings listed: `off=114 pickup-feature-proposal`, `off=116 pickup-bug-report P-073`, `off=120 pickup-bug-report P-076`; `(10 own filing(s) from 010-termlink not counted)` |
| S7.4 | `stuck-claims` | 8,197 B / 112 ln | n/a | Firing topics all `active=0`: `substrate-drain-demo-1481552 active=0 expired=9`, `…-1483085 active=0 expired=15`, `…-1486453 active=0 expired=12`, `work-queue active=0 expired=1` |
| S7.5 | `hook-counter-integrity` | 6,307 B / 120 ln | n/a | Duplicate counter keys; `MECHANISM: unlocked truncate+write in lib/hook-telemetry.sh _fw_telemetry_increment. This is L-023 recurring.` |
| S7.6 | `stale-waker-code` | 4,216 B / 51 ln | n/a | 3 STALE wakers: `aef`, `sonnenstall`, `workshop-designer` — "running old waker code" |
| S7.7 | `fleet-doorbell-mail` | 418 B / 6 ln | n/a | `DRIFT total=4 pass=1 fail=0 unreachable=2 transient_skipped=1`; `.122` and `.121` = `verdict=setup-fail`; only `.107` (local) `verdict=pass` |
| S7.8 | `release-mirror` **stderr** | 150 B | — | `.log` empty but `.log.stderr` non-empty (the T-2685 split working as designed) |
| S7.9 | cross-check | `grep -l -iE "rail dark\|waker-liveness\|stale waker\|preflight WARN\|doorbell.*setup-fail" .tasks/active/*.md` → **6 files**. None of the 7 firing canaries' *current* findings has a dedicated open remediation task naming it. | observed |
| S7.10 | CLAUDE.md § canary convention | States repeatedly: "Empty log = healthy. Any entry = …". T-2685 note states: *"once a tooling error dirties the log, a subsequent genuine finding appends to an already-non-empty file and changes nothing an operator can see — the canary is not merely noisy, it is deaf until someone truncates it by hand."* Applies equally to a genuine finding repeated 74 days running (S7.1). | claimed / observed |

---

## S8. Static-check allowlists (acknowledged-debt ledgers)

`wc -l .context/checks/*` — **19 files, 1,224 lines total.**

| Item | File | Lines | Header states |
|---|---|---|---|
| S8.1 | `mcp-parity-census-allowlist` | 266 | 236 tools unexamined for parity |
| S8.2 | `unbounded-rpc-call-allowlist` | 255 | "CLASS 2: NOT YET MIGRATED … the T-2641 hang class, still live … This is NOT a claim that they are safe. **Each one can still hang forever.**" |
| S8.3 | `verification-pipefail-allowlist` | 187 | "These 158 lines across 55 task(s) predate the check. Every one of them can report 141 — a FAILED gate — at the moment its check SUCCEEDS" |
| S8.4 | `charter-drift-allowlist` | 83 | 28 off-charter tools, "pending T-2548" (see S4.5/S4.6 — T-2548 is closed) |
| S8.5 | others (15 files) | 433 | alloc-sink 43, error-code-emission 43, strict-star 43, drain-sink 42, error-swallowing 38, busy-spin 27, addressed-aliases 21, silent-exit 20, stranded-finalized 20, platform-lock 62, planted-default-gate 17, task-template-idioms 17, verification-misfile 16, version-derivation 16, handover-staleness 8 |
| S8.6 | derived | Every allowlist header declares itself "a ledger of an open question, NOT a permanent exemption". No file carries an expiry, owner, or review date. | structure |

---

## S9. Script/check inventory

| Item | Source | Data point |
|---|---|---|
| S9.1 | `ls scripts/*.sh` | 190 |
| S9.2 | `grep -l '^# guard-layer: source' scripts/*.sh` | 44 carry the guard-layer marker |
| S9.3 | `ls scripts/check-*.sh` | 66 (14,176 LOC) |
| S9.4 | `grep -ohE 'scripts/[a-z0-9_.-]+\.sh' .context/cron/*.crontab \| sort -u` | 32 scripts referenced by a crontab |
| S9.5 | orphan scan (each script's basename grepped across `.context/cron`, `.claude`, `docs`, `CLAUDE.md`, `scripts`, `tests`, `.github`) | **0 orphans** — every script is referenced somewhere |
| S9.6 | `bash scripts/run-guard-layer.sh --list` | **191 declared members** |
| S9.7 | measured | `run-guard-layer.sh --json` **exceeded 12 minutes** without completing. `scripts/run-guard-layer.sh:77` sets `MEMBER_TIMEOUT=300` per member. |
| S9.8 | CLAUDE.md § Running the guard layer | Claims: "`bash scripts/run-guard-layer.sh` # all static checks + fixture suites (**seconds**)" — contradicted by S9.7 |
| S9.9 | `.github/workflows/doc-lint.yml:72-81` | A `guard-layer` job runs `bash scripts/run-guard-layer.sh` on **every push and PR**; `release.yml:57` runs it as a gate both build jobs `needs:` |

---

## S10. AEF task ledger

| Item | Source | Data point | Kind |
|---|---|---|---|
| S10.1 | `ls .tasks/{active,completed}` | **246 active**, 2,433 completed | structure |
| S10.2 | frontmatter parse of all 246 | status × owner: `captured/agent` 91 · **`work-completed/human` 71** · `started-work/human` 39 · `captured/human` 27 · `started-work/agent` 16 · other 2 | structure |
| S10.3 | derived from S10.2 | **137 of 246 (55.7%) active tasks are `owner: human`** | friction |
| S10.4 | `created:` frontmatter parse | age of active tasks: >30d **181** · >90d **95** · >180d 1; median **42 d**, max 182 d | structure |
| S10.5 | `bash scripts/check-stranded-finalized-tasks.sh --json` | `checked: 246, firing_count: 0, partial_complete_count: **71**` | observed |
| S10.6 | `last_update` parse of the 71 partial-complete tasks | days waiting for human verification: **median 125 d**, max 142 d; **56 > 30 d**, **53 > 90 d** | friction |
| S10.7 | `date_finished` month histogram over `.tasks/completed/` | 2026-03 **562** · 04 **609** · 05 412 · 06 359 · 07 **165** · 08 279 · 09 (19 days) **47** ≈ 75/mo run-rate | usage |
| S10.8 | S10.7 vs S1.10 | Task completion fell ~8× (609/mo → ~75/mo) while commit rate remained 447/30d | usage |
| S10.9 | 10 oldest active | T-212 (182d, human, later), T-1124 (154d), T-1137 (153d, human, now), T-1261 (147d), T-1296 (146d, human, now), T-1429 (142d), T-1428, T-1426, T-1420, T-1419 — 7 of 10 `owner: human` | structure |
| S10.10 | `ls .context/arcs/` | 7 arcs: arc-008, arc-parallel-substrate, arc-substrate-fitness, comms-loudness, mcp-slimming, push-transport, reliable-comms | structure |

---

## S11. Governance behaviour — gate bypasses

`.context/working/.gate-bypass-log.yaml`, parsed.

| Item | Data point | Kind |
|---|---|---|
| S11.1 | **482 bypass entries total**; 46 since 2026-08-20 | friction |
| S11.2 | By `caller`: `check-active-task focus-drift` **164** · `check_human_sovereignty` **149** · `partial_complete_recheck` **94** · `check_acceptance_criteria` 26 · `owner_change` 10 · `run_verification_commands` 8 · `check_rca_for_bugfix` 8 · `check_inception_decision` 8 · `human-ac-self-validate` 7 · `create-task.sh` 4 | friction |
| S11.3 | By `flag`: `FW_SWITCH_FOCUS=1` **157** · `--skip-sovereignty` **149** · `--skip-acceptance-criteria` **120** · `--skip-human-ownership` 10 · `--skip-verification` 8 · `--skip-rca` 8 · `--skip-inception-decision` 8 · `FW_ALLOW_HUMAN_AC_TICK=1 + sed` 7 · `--switch-focus` 7 | friction |
| S11.4 | Context | `--skip-sovereignty` bypasses the Authority Model's highest gate (only a human may tick a Human AC). It is the **second-most-bypassed** gate in the project's history. | friction |
| S11.5 | Last 4 entries (tail) | All `FW_SWITCH_FOCUS=1` / `check-active-task focus-drift`, dated 2026-09-18, committing under a different task id than focus | friction |

---

## S12. Hook/gate defect measured live during this review

| Item | Source | Data point | Kind |
|---|---|---|---|
| S12.1 | live | Two `Agent` dispatches in this session were **BLOCKED**: `BLOCKED: Agent dispatch #4 exceeds limit (2)` and `#5`. This session had made **zero** prior dispatches. | measured |
| S12.2 | `cat .context/working/.agent-dispatch-counter` | value `5` (was `3` before this session's two attempts) | measured |
| S12.3 | `.agentic-framework/agents/context/check-agent-dispatch.sh:10` | Comment: "Tracks Agent dispatches **per session** via counter file" | claimed |
| S12.4 | `:29`, `:54-63`, `:66` | `COUNTER_FILE="$PROJECT_ROOT/.context/working/.agent-dispatch-counter"`; increments **before** the limit test; `DISPATCH_LIMIT` default 2 | inferred |
| S12.5 | `grep -rn agent-dispatch-counter .agentic-framework/` | The **only** code that deletes it is `agents/context/post-compact-resume.sh:56` (in `VOLATILE_FILES`) | inferred |
| S12.6 | `post-compact-resume.sh:42-48` | On `SOURCE_TAG = "startup"`, the hook `exit 0`s **before** reaching the volatile-file loop unless `.context/working/.auto-restart-pending` exists. Comment: *"a genuine cold `claude` start ALSO emits 'startup' … otherwise no-op. compact/resume are unaffected."* (T-2376) | inferred |
| S12.7 | `ls .context/working/.auto-restart-pending` | **absent** → the `exit 0` branch is the one taken on a cold start | measured |
| S12.8 | derived S12.2–S12.7 | On a cold session start the counter is never reset. The gate advertises a **per-session** limit of 2; it enforces a **per-project-lifetime** limit of 2 until a human runs `fw dispatch reset`. | inferred |
| S12.9 | `post-compact-resume.sh:56-64` `VOLATILE_FILES` list | The same cold-start `exit 0` also skips clearing `.budget-gate-counter`, `.edit-counter`, `.tool-counter`, `.prev-token-reading`, `.handover-cooldown`, `.loop-detect.json`, `.new-file-counter`, `.approval-notified` — the file's own comments name the consequences: *"old count blocks agent dispatch … old cooldown prevents handover in new session … old patterns cause false loop detection"* | inferred |
| S12.10 | `.context/working/.hook-counter` | File begins with a stray `error-watchdog=4` line, a blank line, then a second block also containing `error-watchdog=20` — duplicate keys, consistent with the hook-counter-integrity canary finding (S7.5) | measured |
| S12.11 | The gate's own remediation text | Recommends `fw termlink dispatch --name worker-1 --prompt '…'`. Invoked: `fw termlink dispatch --help` → `ERROR: Unknown option: --help`; `fw termlink dispatch` → `ERROR: Missing --name`. No usage text is reachable from either invocation. | measured |
| S12.12 | This review's consequence | Role separation (GATHERER vs JUDGE) could not be achieved by `Agent` dispatch. See report §3. | friction |

---

## S13. Concerns register and learnings

| Item | Source | Data point |
|---|---|---|
| S13.1 | `.context/project/concerns.yaml` | 52 entries. Status: `watching` **27** · `closed` 12 · `resolved` 6 · `mitigated` 5 · `decided-build` 2. Severity: medium 30 · **high 15** · low 6 · **critical 1** |
| S13.2 | open/watching entries touching charter core verbs | **G-086 (high)** — "claim primitive #1 promises exclusive ownership … on spoofable claimer"; **G-087 (high)** — same class, round-13 adversarial review; **G-088 (medium)** — "the substrate's advertised exactly-once p…"; **G-083 (high)** — "The doorbell/relay design assumes a PTY-inject wake becomes a turn. Field evidence (2…)" |
| S13.3 | `grep -rl <G-id> .tasks/active/*.md` | G-093 **0** · G-092 **0** · G-091 1 · G-088 **0** · G-087 1 · G-086 **0** · G-083 1 — 4 of 7 sampled open concerns have **no** active task |
| S13.4 | `.context/project/learnings.yaml` | **354** PL-* learnings recorded |
| S13.5 | `bash scripts/check-pickup-deferred-freshness.sh --json` | `ok: false`, `stranded_count: 1` — `P-078-learning.yaml`, class **STRANDED**, age 9.0 d, `blocking_task: null`, `age_source: mtime` |
| S13.6 | `ls .context/upstream/` | 9 `U-00N-*.yaml` upstream defect reports + 13 herdr evaluation/adoption documents |

---

## S14. Governance memory volume

| Item | Source | Data point | Kind |
|---|---|---|---|
| S14.1 | `du -sh` | `.context/handovers` **59 MB** (1,706 `.md` files) · `.tasks` **25 MB** · `.context/episodic` **12 MB** · `.context/audits` **8.4 MB** | cost |
| S14.2 | derived | ≈**105 MB** of governance artefacts vs 180,686 LOC (~6 MB) of product source | cost |
| S14.3 | `git log --since=90.days --name-only` top-level tally | `.context/working` 2,749 file-touches · `.tasks/completed` 2,511 · `.tasks/active` 1,924 · `.agentic-framework/docs` 1,371 · `.context/handovers` 1,283 · … vs **`crates/*` combined 421** | cost |
| S14.4 | derived from S14.3 | Over 90 days, governance-artefact file-touches outnumber product-source file-touches by roughly **24:1** | cost |
| S14.5 | `.context/audits/*.yaml` | 151 audit files |

---

## S15. NON-USE DIAGNOSIS — evidence per low/no-use item

Per the protocol, evidence for **each** reading is recorded; no reading is selected here.

### N1 — `agent-chat-arc` broadcast rail (1,001 records; 0 posts/day in last 10 snapshots, S6.3)

| Reading | Evidence found |
|---|---|
| A BROKEN | `waker-liveness` canary FIRING **RAIL DARK** — zero LIVE listeners carry `pty_session` (S7.2). `fleet-doorbell-mail` DRIFT: `.121` and `.122` `verdict=setup-fail` (S7.7). `stale-waker-code`: 3 wakers running old code (S7.6). `termlink agent listeners` returns empty (S6.6). |
| B NEVER WIRED | Contra-indicated: it *was* used — 270 posts/day average across the oldest third of snapshots, max 813 (S6.2). |
| C UNDISCOVERABLE | Contra-indicated: `/broadcast-chat`, `/recent-chat`, `/pulse`, `/peers` skills all exist (S1.9) and are documented in CLAUDE.md. |
| D UNMEASURED | Contra-indicated: the fleet-adoption snapshot measures it daily, out-of-band (S6). |
| E NOT WANTED | No recorded decision to retire it. Charter verb 1 ("discover each other") and verb 2 ("exchange durable messages") depend on it. No abandoned-arc record; `comms-loudness` and `push-transport` arcs exist (S10.10). |
| Intent evidence | Charter S2.1/S2.2; 7 canaries built specifically to protect this rail; `arc-004 push-transport` recorded shipped. |

### N2 — 46 deprecated MCP tools (S4.1, S4.9)

| Reading | Evidence found |
|---|---|
| A BROKEN | None found — they remain callable by design (`p4-surface-reduction.md`: "Nothing is deleted; `git revert` restores the full surface"). |
| B NEVER WIRED | n/a — they are wired. |
| C UNDISCOVERABLE | Deliberately so: CLI twins carry `#[command(hide = true)]`; MCP descriptions prefixed `[DEPRECATED]`. |
| D UNMEASURED | **Yes.** No per-tool invocation telemetry exists: `grep -rln "api-usage\|command-usage\|verb_usage" scripts/ crates/termlink-cli/src/commands/` → empty; `termlink api-usage --json` → `unrecognized subcommand`. (A CLAUDE.md-referenced task T-1419 mentions `api-usage --json`; the verb is not present in the 0.11.1766 binary.) |
| E NOT WANTED | **Yes, with a recorded decision.** `p4-surface-reduction.md` records the human directive (2026-08-02) and the charter non-goal #3 rationale; T-2471 already deleted the 12 zero-consumer siblings. |
| Intent evidence | The doc states deletion is gated on "a deprecation soak and a separate human go-ahead". No go-ahead record found. Soak elapsed: **48 days**. |

### N3 — 28 live off-charter analytics tools (S4.4, S4.7)

| Reading | Evidence found |
|---|---|
| A BROKEN | None found. |
| B NEVER WIRED | Partially: T-2548 records "ZERO first-party callers (grep of `scripts/`, `.claude/commands/`, `crates/termlink-cli` returned empty)" — built, nothing calls them. |
| C UNDISCOVERABLE | Contra-indicated: they are LIVE in `termlink help` and in the MCP tool list. |
| D UNMEASURED | **Yes** — same absence of per-tool telemetry as N2. External (cross-project) consumers are explicitly unmeasured: T-2548 names **IW-1 external-consumer check** as an outstanding gate, blocked by the T-559 project boundary. |
| E NOT WANTED | **Yes, with a recorded human decision: T-2548 GO (subtract), 2026-08-20** (S4.6/S4.7). |
| Intent evidence | KEEP-list carved out in the same decision (`agent_search_thread`, `agent_thread_path`, `agent_recent_window`). |

### N4 — Canary findings that fire daily and are never acted on (S7)

| Reading | Evidence found |
|---|---|
| A BROKEN | The *detectors* are not broken — they fire correctly and name precise remediations (S7.1, S7.2, S7.6). |
| B NEVER WIRED | **Yes, for the response half.** No mechanism converts a firing canary into a task: 4 of 7 sampled open concerns have no active task (S13.3); no active task names any of the 7 current firings (S7.9). `/canaries` is a manual read-verb (CLAUDE.md § Canaries). |
| C UNDISCOVERABLE | Partially: the findings live in dotfiles under `.context/working/`, surfaced only by a human typing `/canaries`. |
| D UNMEASURED | No — they are measured; nothing consumes the measurement. |
| E NOT WANTED | Contra-indicated: CLAUDE.md devotes ~1,908 lines to them (S3.5); they were each built by a dedicated task. |
| Intent evidence | Overwhelming — 42 CLAUDE.md sections, 24 installed cron jobs, 191 guard-layer members. |

### N5 — 71 partial-complete tasks awaiting human verification (S10.5, S10.6)

| Reading | Evidence found |
|---|---|
| A BROKEN | **Yes.** CLAUDE.md § T-2859 records that the rendered approval page **dropped the `**Steps:**` block** for 8 of 129 human ACs — "the human approves an action whose command the page never showed them". Repaired locally per the note, but the renderer is **vendored** (G-062) and therefore reverts on re-vendor. |
| B NEVER WIRED | No — the path exists (`fw task verify`, Watchtower review page). |
| C UNDISCOVERABLE | Partially — no notification mechanism surfaces a task that has been waiting 125 days. |
| D UNMEASURED | No — `check-stranded-finalized-tasks.sh` counts them (71) and explicitly **does not fire** on them by design. |
| E NOT WANTED | Contra-indicated: `--skip-sovereignty` used 149 times (S11.2) indicates the work is wanted done, not abandoned. |
| Intent evidence | T-193 partial-complete is a designed state; 71 tasks' agent ACs are all complete. |

---

## S16. Contradictions found (recorded, not resolved)

| Item | Claim | Measured |
|---|---|---|
| S16.1 | `CLAUDE.md:4` — "see `FRAMEWORK.md`" | No `FRAMEWORK.md` at repo root (S2.4) |
| S16.2 | CLAUDE.md § guard layer — runner takes "(seconds)" | >12 min, incomplete (S9.7) |
| S16.3 | `.context/checks/charter-drift-allowlist` + CLAUDE.md § charter-drift — T-2548 is "`started-work`, `owner: human`", "an open decision" | T-2548 `work-completed`, **Decision: GO**, 2026-08-20 (S4.6/S4.7) |
| S16.4 | `check-agent-dispatch.sh:10` — "Tracks Agent dispatches **per session**" | Per project lifetime on cold starts (S12.8) |
| S16.5 | CLAUDE.md canary convention — "Empty log = healthy. Any entry = [action needed]" | `substrate-preflight` log non-empty for 74 consecutive days with no recorded action (S7.1, S7.9) |
| S16.6 | CLAUDE.md § stuck-claims (T-2709) — "The arm now keys on `newest_expired_at_ms` so it self-clears" | Firing on four topics all with `active=0` (S7.4) |
| S16.7 | CLAUDE.md § T-1419 entry references `api-usage --json` | `termlink api-usage` → `unrecognized subcommand` on 0.11.1766 (S15/N2) |
| S16.8 | Charter non-goal #2 — "Not a durable database or system of record" | `health:ring20-fedprobe` retention `forever`, 1,670 records = 38.8% of all bus traffic (S6.7) |
| S16.9 | Dispatch gate remediation text — "use `fw termlink dispatch`" | No reachable usage/help for that verb (S12.11) |

---

## S17. Data gaps (recorded — no data is not zero)

| Item | Gap | Consequence |
|---|---|---|
| S17.1 | **No per-tool / per-verb invocation telemetry** anywhere (S15/N2). | Cannot distinguish "never called" from "called and not measured" for any of the 260 MCP tools or the CLI verbs. All DELETE reasoning on the tool surface must rest on recorded decisions and caller greps, not usage. |
| S17.2 | **No external/cross-project consumer data** (IW-1, blocked by the T-559 project boundary). | DELETE CHECK #5 cannot be satisfied from inside this repo for any MCP tool. |
| S17.3 | **No hub-side telemetry for discarded posts / silent drops.** CLAUDE.md § TermLink data layer states this is "NOT AVAILABLE from the bus itself". | Delivery reliability cannot be judged from the bus's own data. |
| S17.4 | `substrate-preflight` canary log holds 74 days of entries but **no timestamps per entry in a machine-readable field**; day count derived by grepping ISO dates in the body. | Fine-grained "when did this start" is approximate. |
| S17.5 | Several canary logs (waker-liveness, stuck-claims, stale-waker-code, hook-counter, fleet-doorbell-mail) contain **no ISO date in the entry body**, so firing-frequency over time is unmeasurable from the log alone. | Cannot say whether S7.2/S7.4/S7.6 are one-day or many-day conditions. |
| S17.6 | `run-guard-layer.sh` did not complete → **no PASS/FAIL/ERROR census for the 191 members** in this review. | Guard-layer health is UNVERIFIED for this run. |
| S17.7 | No coverage instrumentation (`cargo llvm-cov` / tarpaulin) present or run. | Per-item test-coverage evidence unavailable; only test *counts* (S1.3). |
| S17.8 | No BVP realization log at `.context/audits/bvp-realization.jsonl` (searched; not found). | "Did shipped arcs deliver?" is unanswerable from data. |

---

## S19. ADDENDUM — guard-layer run (completed after S0 was written)

`bash scripts/run-guard-layer.sh --json` finished after **~20 minutes** wall-clock.
This supersedes S0's "did not finish" row and closes gap S17.6.

| Item | Data point | Kind |
|---|---|---|
| S19.1 | `summary`: `{total: 113, passed: 109, fired: 4, errored: 0, unclassified: 75, with_tests: false, exit_code: 1}` — **the guard layer is currently RED** | measured |
| S19.2 | The 4 firing members: `check-installed-binary-drift.sh` (rc 1) · `check-pickup-deferred-freshness.sh` (rc 1) · `check-receiver-ack-lag.sh` (rc 1) · `cron-drift-firing-fixtures.sh` (rc 1) | measured |
| S19.3 | **`unclassified: 75`** — 75 scripts carry no `# guard-layer:` marker and are therefore neither run nor excluded. CLAUDE.md § run-guard-layer states an unmarked check "is reported as unclassified rather than silently ignored — a forgotten marker is itself the shipped-but-dark condition." | measured |
| S19.4 | `.github/workflows/doc-lint.yml:72-81` — the `guard-layer` job has **`timeout-minutes: 10`**, against a measured local runtime of ~20 min (S19.1). `release.yml:57` runs the same script as a gate both build jobs `needs:`. | observed |
| S19.5 | `bash tests/cron-drift-firing-fixtures.sh` — 12 of 13 assertions PASS; the single FAIL is the **real-tree control assertion** "the real tree passes the firing check", i.e. the fixture suite is correctly reporting drift in the live tree, not a broken test | measured |
| S19.6 | `bash scripts/check-cron-install-drift.sh --json` → `ok: false, missing_count: 1, drift_count: 0, uninstalled_jobs_count: 0`. Missing: **`substrate-smoke-canary.crontab → /etc/cron.d/termlink-substrate-smoke-canary`** — a canary committed but never installed (the shipped-but-dark class T-2561 exists to catch). An active task T-2939 already names this condition. | measured |
| S19.7 | `bash scripts/check-receiver-ack-lag.sh` — **`agent-chat-arc`: 4 identities NEVER-ACKED at lag=1533**; 1 BEHIND (`d1993c2c3ec44c94`, lag=609, up_to=923). **`framework:pickup`: 2 identities NEVER-ACKED at lag=126**; 1 BEHIND (lag=44, up_to=81). | measured |
| S19.8 | S19.7 cross-referenced with S6.7 | `agent-chat-arc` holds 1,001 records and `framework:pickup` 126; the ack lag numbers (1533 / 126) indicate receivers that have consumed nothing on those topics for the whole retained history. This is the charter's verb 2 ("exchange durable messages") measured on the **receive** half. | measured |
| S19.9 | `check-installed-binary-drift.sh` firing | Consistent with S7.1 and S0: installed `termlink` 0.11.1766 vs `VERSION` 0.11.1944 | measured |
| S19.10 | `tmux capture-pane` on a freshly spawned session during this review | `Data plane listening path=/tmp/termlink-0/sessions/tl-o3kztzjb.sock.data` — the **session** runtime dir is `/tmp/termlink-0` while the **hub** runs from `/var/lib/termlink` (per `termlink hub status`). Two different runtime roots in one live system. | measured |

---

## S20. Role separation achieved (recorded for the report's §3)

| Item | Data point |
|---|---|
| S20.1 | `Agent` tool dispatch was BLOCKED twice by `check-agent-dispatch` (S12.1). The gate was **not bypassed**; `fw dispatch reset` and `fw dispatch approve` were deliberately not run, because no human was available to authorize passing a structural gate (CLAUDE.md § Autonomous Mode Boundaries). |
| S20.2 | The gate's own recommended path was used instead: `bash scripts/tl-dispatch.sh --name vr4-judge --prompt-file … --model sonnet --timeout 900`. This spawns a separate `claude -p` process in its own tmux session with its own context window (`scripts/tl-dispatch.sh:1-15`). |
| S20.3 | JUDGE ran on **Sonnet**; GATHERER ran on **Opus 5** — different model families, satisfying the protocol's preference. |
| S20.4 | The JUDGE's only inputs were the confirmed-provisional yardstick and this evidence file; it was instructed not to explore the repo. |

---

## S18. Not covered by this evidence file

- `.agentic-framework/` internals beyond the specific files cited (vendored; G-062 — changes there are upstream's).
- Per-function complexity metrics (no tool installed; not added per the no-new-dependency rule).
- `web/` Watchtower surfaces beyond the T-2859 record cited in CLAUDE.md.
- Cross-hub state on `.121`, `.122`, `.141` (2 unreachable; no foothold).
- Prior runs (run2, run3) of this same review — deliberately not read, per the independence requirement.
