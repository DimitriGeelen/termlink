# VALUE REVIEW — EVIDENCE FILE (run 2)

**Scope:** whole repo (`/opt/termlink`)
**Date:** 2026-09-19
**Role:** GATHERER (facts only — no classification, no recommendations)
**Snapshot taken:** 2026-09-19T17:32Z, before any review-generated activity
**Purpose source:** `docs/CHARTER.md`

> Facts only. Every row cites a path, command, or record. Conflicting data points are
> recorded side by side. Classification is the JUDGE's job (Phase 4).

---

## S0. Baseline

| Item | Source | Status | Data point | Kind |
|---|---|---|---|---|
| Test suite | `cargo test --workspace` (run 2026-09-19T17:33Z) | EXISTS | **3618 passed, 0 failed, 4 ignored**, exit 0. 24 test binaries. Runtime ~40s. Output: `.context/working/vr-run2-snapshot/baseline-tests.txt` | structure |
| Test distribution | `grep -c '#[test]'` per crate | EXISTS | cli 1324, mcp 1049, session 510, hub 424, bus 109, protocol 106, test-utils 5 | structure |
| Build | workspace builds clean (tests compiled + ran) | EXISTS | exit 0 | structure |
| `fw audit` (latest) | `.context/audits/2026-09-19.yaml` | EXISTS | pass 39, warn 8, fail 1 (48 findings) | friction |
| Repo version | `VERSION` | EXISTS | `0.11.1940` | structure |
| Installed binary | `termlink --version` | EXISTS | `0.11.1766`, built 2026-09-01 13:38 | structure |

---

## S1. Yardstick (from `docs/CHARTER.md`)

| Item | Source | Status | Data point | Kind |
|---|---|---|---|---|
| Canonical purpose | `docs/CHARTER.md` §Canonical purpose | EXISTS | "hub-mediated, durable append-log message bus with terminal endpoints — the coordination substrate that lets a fleet of AI agents (and humans) **discover each other, exchange durable messages, claim work, and control terminal sessions** across one or many machines." Human-blessed. | value |
| Four core verbs | ibid | EXISTS | V1 discover · V2 exchange durable messages · V3 claim work · V4 control terminal sessions | value |
| Non-goals | `docs/CHARTER.md` §Non-goals | EXISTS | 1 not federation · 2 **not a durable database / system of record** · 3 not social/engagement · 4 not a workflow engine · 5 not a security boundary between distrusting tenants | value |
| Value drivers | `.agentic-framework/policy/value-drivers.yaml` v3 | EXISTS | Protected D1 Antifragility (w9), D2 Reliability (w7), D3 Usability, D4 Portability. Free: F-RECALL Recall Leverage (w6). F-ORCH retired 2026-07-07 (T-2511). | value |
| Consumers | `~/.termlink/hubs.toml` + `fleet doctor --json` | EXISTS | 5 declared hubs; 4 reachable (local-test, ring20-dashboard .121, ring20-management .122, workstation-107 .107); 1 down (laptop-141, no route to host). Versions: 0.11.1766 ×2, 0.11.1411 ×2, unknown ×1. | usage |
| External consumer project | `aef-install-findings` topic payloads | EXISTS | `/opt/1409-sprind` posts install/remediation findings over TermLink (38 records, 2026-09-18→19) — a real cross-project consumer of V2 | usage |

---

## S2. USAGE — per charter verb (measured, from live hub)

This is the core usage evidence. All figures from the live local hub, 2026-09-19T17:35–17:40Z.

| Item | Source | Status | Data point | Kind |
|---|---|---|---|---|
| **V4 control terminal sessions** | `termlink list --json` | EXISTS | **40 registered sessions, all state=`ready`**, heartbeating. Spread across ≥8 distinct project cwds: /opt/termlink (9), /opt/050-email-archive (5), CashWeb-integration (4), Claude-Partner-Network (3), Azure-DevOps (3), Voxtype (2), Deco-m4r (2), chromium-vault (2). Ages: 32×2d, plus fresh (13m–1h). | usage |
| V4 registry size | `ls /var/lib/termlink/sessions/` | EXISTS | 392 session files on disk | usage |
| **V2 exchange durable messages** | `termlink channel list --json` | EXISTS | 45 topics, 4299 total records. Live traffic topics: `channel:learnings` 191, `framework:pickup` 126, `aef-install-findings` 38, `broadcast:global` 16, 14 `dm:*` topics (83/32/16/13/12/…) | usage |
| V2 chat-arc activity | `channel state agent-chat-arc --json` | EXISTS | 950 records, window **2026-08-26 → 2026-09-19 (24 days)**, **5 distinct senders**; dominated by one: `d1993c2c3ec44c94` = 871/950 (**92%**), next 74, then 3/1/1 | usage |
| **V1 discover (presence rail)** | `bash scripts/agent-listeners.sh --json` | EXISTS | **total_listeners 1, live 1**. Sole listener `penelope`, role `email-relay`, host dimitrimintdev, `pty_session: null`, `capabilities: ""` | usage |
| V1 presence over full retained window | `channel state agent-presence --json` | EXISTS | 1026 records, window 2026-09-19T00:13 → 17:35 (**17.4 h**), **1 distinct sender**, **1 distinct role** (`email-relay`). No AI-coding-agent participant in the entire retained window. | usage |
| V1 cv_index | `channel cv-keys agent-presence --json` | EXISTS | `count: 1` — one advertising key (`penelope`) | usage |
| **V3 claim work** | `channel claims-summary --all --json` | EXISTS | 45 topics scanned. **0 topics with ACTIVE claims. stuck_count 0.** Expired rows exist on exactly 3 topics, all synthetic: `substrate-drain-demo` (9), `aef-elections-…` (1), `aef-s8-probe2-…` (1) | usage |
| Push-wake doorbell rail | `.context/working/.waker-liveness-canary.log` (tail) | EXISTS | Repeated `FIRING — RAIL DARK`: "ZERO LIVE listeners carry pty_session on this hub — the G-069 '0 wakers' state". Most recent entries name `penelope` as `[LIVE-no-waker]`. | usage |
| Delivery reliability (self-reported) | `channel queue-status --json`, `channel awaiting-ack --json`, `check-dead-letter-freshness.sh --json` | EXISTS | pending 0, dead_letters 0, awaiting-ack pending 0. **Caveat (ground rule "a channel cannot report its own failures" + "no data is not zero"): these are TermLink's own self-reports on a rail carrying ~1 active participant. Zero pending on an idle rail is not evidence of reliability.** | usage |

### S2b. NON-USE DIAGNOSIS — V3 "claim work"

Evidence collected for each reading; **not** resolved here.

| Reading | Evidence found | Citation |
|---|---|---|
| A BROKEN | **G-086 (HIGH, watching)**: claim/renew/release/claim_transfer/force_release read `claimer`/`by`/`to_owner` straight from JSON params and compare to stored `claimed_by` with **no caller-identity check** — the exclusivity guarantee is enforced only against a spoofable param, while `channel.post` binds identity cryptographically (T-1427). **G-087 (HIGH)**: same root cause has 3 instances. | `.context/project/concerns.yaml` G-086/G-087; `crates/termlink-hub/src/channel.rs:1618`, `meta.rs:431/557/623` |
| B NEVER WIRED | Repo-wide grep for callers of `channel claim` outside crates/docs finds **only demo and prover scripts**: `substrate-drain-demo.sh:129`, `substrate-cooperative-handoff-demo.sh:104/112/120`, `substrate-lease-expiry-demo.sh:110/117/129`, `substrate-orchestrator-loop.sh:12`, `substrate-smoke.sh`. **No production consumer.** The AEF orchestrator (`.agentic-framework/lib/*.sh`) does not call it. | grep output, this file S2b |
| C UNDISCOVERABLE | **Ruled out by abundance**: 6 dedicated skills (`claim.md`, `release.md`, `renew.md`, `claims.md`, `claims-history.md`, `claim-transfer.md`), 9 CLI verbs, 9 MCP tools, a master recipe (`docs/operations/substrate-orchestrator-recipe.md`), and extensive CLAUDE.md coverage | `ls .claude/commands/`, `termlink channel --help` |
| D UNMEASURED | **Ruled out**: `claims-summary` measures it directly and reports 0 | S2 row above |
| E NOT WANTED | **Contradicted by strong intent evidence**: V3 is one of the four verbs in the human-blessed charter sentence | `docs/CHARTER.md` |
| Tooling built vs used | 9 MCP tools + 9 CLI verbs + 6 skills + 1 cron canary (T-2556) + 1 smoke prover + 4 observability layers (watch/notify/log/history) | inventory above |

### S2c. NON-USE DIAGNOSIS — V1 presence / doorbell rail

| Reading | Evidence found | Citation |
|---|---|---|
| A BROKEN | **G-083 (HIGH, watching)**: "The doorbell/relay design assumes a PTY-inject wake becomes a turn. Field evidence (2026-07-10) shows it does NOT when the recipient session is busy or in manual-accept mode: the injected text lands in the input box UNSUBMITTED and is discarded on the next `claude --continue`." Also: heartbeat comes from a background script decoupled from the claude session's real availability, so `LIVE + armed` can be true while the session is unavailable. | `.context/project/concerns.yaml` G-083 |
| B NEVER WIRED | Waker canary states arming requires relaunch through `scripts/tl-claude.sh`; PL-237 (cited in CLAUDE.md) states running headless claudes **cannot be retrofitted** — must be armed at launch | `.context/working/.waker-liveness-canary.log`; CLAUDE.md §waker-liveness |
| C UNDISCOVERABLE | Not supported — `/be-reachable`, `/peers`, `/pulse` skills exist and are documented | `.claude/commands/` |
| D UNMEASURED | Ruled out — presence IS the instrumentation and it is reporting | S2 |
| E NOT WANTED | Contradicted — V1 is a charter verb | `docs/CHARTER.md` |
| Contrast | V4's own discovery surface (`termlink list`, 40 sessions) is healthy — so "discovery" is alive via the session registry while the **agent-presence** rail specifically is near-dead | S2 |

---

## S3. COST — context budget (measured)

| Item | Source | Status | Data point | Kind |
|---|---|---|---|---|
| CLAUDE.md size | `wc -c CLAUDE.md` | EXISTS | **276,749 chars ≈ 69,187 tokens**, 3,283 lines. Auto-loaded in full into every session (directly observed: this review session's own system prompt contained the entire file). | cost |
| MCP tool-description payload | regex over `crates/termlink-mcp/src/tools.rs` | EXISTS | 261 description strings, **105,516 chars ≈ 26,379 tokens**; median 420 chars, longest 1546 | cost |
| Combined fixed session overhead | sum of the two above | EXISTS (inferred sum) | **≈ 95,500 tokens consumed before any product code is read** — ~48% of a 200k window | cost |
| Budget gate thresholds | CLAUDE.md §Context Budget Management | EXISTS | warn 120k · urgent 150k · **critical 170k = Write/Edit to source files BLOCKED** | friction |
| Budget gate actually truncating work | `git log --grep="budget gate" -i --all` | EXISTS | **17 commits** explicitly record work stopped by the budget gate. Examples: `ea468c254` T-2871 "Tests NOT yet run (budget gate)"; `2c0d1eedc` T-2916 "Canary ack still owed (budget gate)"; `4dbd1004c` T-2833 "bank … before the budget gate closes" | friction |
| Budget gate mentions in task files | grep over `.tasks/` | EXISTS | 81 occurrences | friction |
| Guard-layer source size | `cat scripts/check-*.sh \| wc -l` | EXISTS | 66 check scripts, **14,176 LOC**; `tests/*.sh` **18,452 LOC** → ~32.6k LOC guard layer vs 180,686 LOC product (18%) | cost |
| Repo LOC balance | `find`/`wc` | EXISTS | crates .rs 180,686 · docs .md 68,509 · scripts .sh 39,689 · tests .sh 18,452 · .claude/commands 6,814 · agents 1,880 | cost |

---

## S4. ACTIVITY — where work actually goes

| Item | Source | Status | Data point | Kind |
|---|---|---|---|---|
| Commits/month (all) | `git log --since="12 months ago"` | EXISTS | 2026-03 **1054** · 04 **2114** · 05 **1090** · 06 **1077** · 07 **594** · 08 **1055** · 09 **184** (19 days elapsed) | usage |
| Commits/month touching `crates/` | same, `-- crates/` | EXISTS | 03 **400** · 04 321 · 05 304 · 06 200 · 07 64 · 08 **156** · **09: 3** | usage |
| Sept file-touches by area | `git log --since=2026-09-01 --name-only` | EXISTS | `.context` **598** · `.tasks` **329** · tests 26 · scripts 25 · .fabric 18 · .termlink-task 9 · VERSION 6 · docs 4 · CLAUDE.md 3 · **crates 1** · .agentic-framework 1. **927 of 1023 touches (90.6%) are `.context`+`.tasks` bookkeeping; 1 touch (0.1%) is product code.** | usage |
| August comparison | same, Aug window | EXISTS | .context 3387 · .agentic-framework 1949 · .tasks 1548 · .fabric 361 · **crates 239** · scripts 199 — product work was still happening in August | usage |
| The one Sept product commit | `git log --since=2026-09-01 -- crates/` | EXISTS | `ea468c254` T-2871 — a **WIP** commit whose message says "Tests NOT yet run (budget gate)". T-2871 has since reached `work-completed` in `.tasks/completed/`. | usage |
| Binary staleness, in context | `stat $(which termlink)` + git log | EXISTS | Binary built 2026-09-01; only **1** `crates/` commit has landed since. **Conflicting reading recorded:** the preflight canary reports the binary as 174 versions stale, but VERSION is git-derived over *all* commits, so the gap overwhelmingly reflects governance commits, not un-shipped product code. | usage / friction |
| **Challenge to the September finding — all branches** | `git log --all --since=2026-09-01 -- crates/` | EXISTS | **Exactly 1 commit touches `crates/` across ALL branches** since 2026-09-01 (the same T-2871 WIP). The finding is not an artifact of work sitting on a side branch. `git status` also shows no uncommitted product changes. | usage |
| **Unmerged worktree branches** | `git worktree list` + `git rev-list --count main..<branch>` | EXISTS | 5 live worktrees + 1 detached merge-trial. Commits ahead of `main`: **`worktree-charter-review-2026-0814` 147** (last commit 2026-08-28, **22 d stale**) · `worktree-governance-canary-signal` 8 (2026-08-19, 31 d) · `worktree-T-2398-findings` 2 (2026-09-18) · `worktree-T-2209-history-skills` 2 (**2026-06-13, 98 d**) · `pre-merge-backup-2026-0824` 1. **≈160 commits unmerged.** This is the live instance of the duplicate-work/task-ID-collision class the project documented in T-2800/T-2915. | friction |

---

## S5. TOOL / VERB SURFACE

Source: GATHERER sub-agent (Sonnet), full table at `.context/working/vr-run2-snapshot/ev-surface.md`.

| Item | Source | Status | Data point | Kind |
|---|---|---|---|---|
| MCP tool count | `check-mcp-parity-census.sh --json` + grep `name = "termlink_` | EXISTS | **260 tools** (both methods agree) | structure |
| Parity coverage | same | EXISTS | **24 asserted by `parity.rs`, 236 acknowledged-uncovered, 0 unexamined → 9.2%** | structure |
| Deprecated-but-compiled | grep `(deprecated P4/T-2478)` in tools.rs:1077–1217 | EXISTS | **40 distinct tool names** marked deprecated, all still registered and compiled in | cost |
| Off-charter live tools | `check-charter-drift-freshness.sh --json` | EXISTS | 214 live tools scanned, **28 off-charter, all 28 acknowledged**, 0 unacknowledged firing | value |
| Off-charter families | `.context/checks/charter-drift-allowlist` | EXISTS | agent_rankings 5 · agent_stats 10 · agent_thread_health 8 · channel_engagement 5 = 28 | value |
| Analytics categories (incl. deprecated) | `termlink help --json` | EXISTS | 29 categories / 260 tools; the 4 analytics categories total **37** tools (rankings 8, stats 10, thread_health 8, engagement 11) — larger than the 28 allowlist because some members are already deprecated and excluded from the "live" scan | value |
| **Callers of the 28 off-charter tools** | repo-wide grep (scripts/, .claude/, docs/, crates/, tests/, .agentic-framework/, CLAUDE.md) | EXISTS | **0 of 28 have a real caller.** Every hit resolves to (a) the tool's own registry line or the allowlist file, (b) the charter-drift checker's own fixtures/comments describing detector history, or (c) `docs/reports/T-1697-human-ac-audit.md:322` quoting an old task's AC prose. No slash command, script, recipe, or crate invokes any of them. | usage |
| CLI surface | `termlink --help` | EXISTS | 40 top-level subcommands | structure |
| Skills | `.claude/commands/*.md` | EXISTS | 34 skills; **all 34 have external references** beyond their own file and beyond CLAUDE.md alone (e.g. `agent-handoff` referenced by 30+ files incl. `crates/termlink-cli/src/commands/agent.rs`). Zero orphan skills. | usage |
| Blocker to reducing the surface | `.context/arcs/arc-005.yaml` + audit history | EXISTS | Arc **arc-005 "MCP tool-description slimming"**, created 2026-07-11, status `in-progress`; `fw audit` has warned **18 times** "Arc 'mcp-slimming' has no task commits in the last 30 days" | friction |
| **Deprecated tools are still advertised to clients** | **direct observation of this review session's own MCP tool list** + `crates/termlink-mcp/src/tools.rs:1270,1326,1790,1822` | EXISTS | `is_deprecated()` gates the **help/catalog** surfaces only; no filtering of the MCP `list_tools` registration was found. **Measured directly: this session's connected TermLink MCP server advertises the deprecated/off-charter tools** — `termlink_agent_top_reacted`, `_top_starrers`, `_top_pinners`, `_reaction_rate`, `_emoji_stats`, `_emoji_users`, `_typing`, `_typers`, `_star`, `_starred`, `_pin`, `_pinned`, `_poll_start/vote/end`, `channel_react`, `channel_typing_emit/list`, … are all present in the live tool list. **Deprecation here removes the tool from the catalog but not from the client's context budget.** | cost |
| Decision status on the 28 off-charter tools | `.tasks/completed/T-2548-*.md`; `docs/reports/T-2548-conversation-analytics-non-goal-4.md` | EXISTS | Inception **T-2548 completed 2026-08-20** with all three key questions **`disposition: deferred` to the human**. Agent recommendation on record: **"GO to subtract"**, mirroring the T-2471/T-2478 precedent. Removal gate is **IW-1** — "are there external (non-first-party) consumers across the fleet?" — which "cannot be confirmed from `/opt/termlink` (T-559 boundary)". **The decision has been open 30 days.** | value |

---

## S6. STRUCTURE — hotspots and duplication

| Item | Source | Status | Data point | Kind |
|---|---|---|---|---|
| **Top hotspot (churn × size)** | `git log --since="6 months ago" --name-only -- crates/` + `wc -l` | EXISTS | **`crates/termlink-mcp/src/tools.rs`: 46,458 LOC in ONE file, 390 commits in 6 months, 1,403 functions** — #1 by both churn and size | structure |
| #2 hotspot | same | EXISTS | `crates/termlink-cli/src/commands/channel.rs`: 20,435 LOC, 166 commits, 674 functions | structure |
| Other high-churn | same | EXISTS | `cli/src/main.rs` 365 commits (1,931 LOC) · `cli/src/cli.rs` 360 commits (7,001 LOC) · `cli/commands/remote.rs` 144 commits (11,891 LOC) | structure |
| **Duplication inside tools.rs** | grep | EXISTS | **191 `fn *_mcp` definitions of only 80 distinct names**; `fn to_json_mcp` **defined 26 separate times**. CLAUDE.md §T-2747 documents this as deliberate ("no cross-crate sharing for these tiny pure helpers", T-2069 convention) and notes three circulating counts (83/68/94) were all wrong. | structure |
| Dependencies | `Cargo.lock` | PARTIAL | 310 lockfile packages. **Per-crate direct dep counts not extractable** (crates use `workspace = true` inheritance, so the naive parse returned 0 for all 7). | cost |
| Unused-dependency analysis | `cargo machete` / `cargo udeps` | **ABSENT** | Neither installed. **Data gap** — cannot assess unused dependencies. | cost |
| Fabric connectivity | `.context/audits/*.yaml` | EXISTS | Recurring warn: **"Fabric: 326/491 cards have no edges"** (6 occurrences); "10 card(s) point at files no watch pattern covers" (10×) | structure |

---

## S7. GOVERNANCE BEHAVIOUR

| Item | Source | Status | Data point | Kind |
|---|---|---|---|---|
| Active task ledger | `.tasks/active/` | EXISTS | **246 active** / 2433 completed | structure |
| Active by status | grep frontmatter | EXISTS | `captured` **119** (never started) · `work-completed` **71** · `started-work` 56 | friction |
| The 71 work-completed-in-active | `check-stranded-finalized-tasks.sh --json` | EXISTS | firing_count **0**; all 71 are legitimate **T-193 partial-complete** — every one carries a `date_finished` and `owner: human`. i.e. **71 tasks are done and blocked solely on human AC verification.** | friction |
| Active by owner | grep | EXISTS | `human` **137** · `agent` 107 · `claude-code` 2 | friction |
| Active by type | grep | EXISTS | build 201 · inception 36 · refactor 4 · test 2 · design 2 · decommission 1 | structure |
| Active task age | `created:` frontmatter | EXISTS | 2026-03 1 · 04 11 · **05 61** · 06 27 · 07 15 · **08 85** · 09 46 — 100 active tasks are ≥3 months old | friction |
| Completion rate | `date_finished:` frontmatter | EXISTS | 03 **562** · 04 609 · 05 412 · 06 359 · 07 165 · 08 279 · **09 47** | usage |
| Gate bypasses (total) | `.context/working/.gate-bypass-log.yaml` | EXISTS | **482 entries**. By month: 04 109 · 05 122 · 06 95 · 07 67 · 08 79 · **09 10** (declining) | friction |
| Bypass by caller | same | EXISTS | `check-active-task focus-drift` **164** · `check_human_sovereignty` **149** · `partial_complete_recheck` **94** · `check_acceptance_criteria` 26 · owner_change 10 · run_verification_commands 8 · check_rca_for_bugfix 8 · check_inception_decision 8 | friction |
| Bypass by flag | same | EXISTS | `FW_SWITCH_FOCUS=1` 157 · `--skip-sovereignty` 149 · `--skip-acceptance-criteria` 120 · `--skip-human-ownership` 10 · `--skip-verification` 8 | friction |
| Concerns register | `.context/project/concerns.yaml` | EXISTS | 52 total. Status: **watching 27** · closed 12 · resolved 6 · mitigated 5 · decided-build 2. Severity: medium 30 · **high 15** · low 6 · critical 1 | friction |
| Open HIGH concerns | same | EXISTS | G-092, G-091, **G-087**, **G-086**, **G-083** all `watching` (see S2b/S2c) | value |
| **Recurring audit findings** | 25 most recent `.context/audits/*.yaml` | EXISTS | 18× "onboarding-seed corpus references — **NOT EVALUATED**: " · 18× "**Arc 'mcp-slimming' has no task commits in the last 30 days**" · 18× "No PROJECT_ROOT resolution — **NOT EVALUATED**" · 18× "No stale-slice-references — **NOT EVALUATED**: candidate set empty" · 16× "free driver F-ORCH retire_when appears met" · 12× "Arc 'arc-substrate-fitness' no task commits in 30 days" · 11× "**77 GO-scope-not-propagated inception(s) of 158**" · 10× fabric edge warnings · **7× FAIL cron(substrate-smoke-canary) no install in /etc/cron.d** · **6× FAIL cron drift agentic-audit.crontab** | friction |
| Arcs | `.context/arcs/*.yaml` | EXISTS | 7 arcs: arc-001 (2026-06-07) **in-progress** · arc-002 (06-22) **in-progress** · arc-003 closed · arc-004 closed · arc-005 mcp-slimming (07-11) **in-progress** · arc-007 (07-21) **in-progress** · arc-008 (09-09) in-progress. **4 of 5 in-progress arcs predate August; audit confirms ≥2 have had no task commits in 30 days.** | friction |
| Task-ID collisions across branches | `check-task-id-collisions.sh --json` | EXISTS | **0 colliding IDs, 0 duplicate new files, 0 near-duplicate titles** (axes A/B/C clean across 7 branches) — but **axis D: 7 duplicate-fix warnings**, i.e. two sides modified the same pre-existing file and deleted identical original lines. One is the product hotspot: **`crates/termlink-mcp/src/tools.rs`, sides `worktree-charter-review-2026-0814` vs `worktree-t2687-pickup-failopen`, 8 shared deleted lines.** Axis D warns, never fires, by design. | friction |
| Vendor divergence | `check-vendor-divergence.sh` | EXISTS | 22 commits touch vendored code since baseline `8c1cca561`, **all registered** — clean | structure |
| Framework tracking drift | `check-framework-tracking-drift.sh --quiet` | EXISTS | no output = clean | structure |
| Dispatch gate defect (observed live this session) | `.agentic-framework/agents/context/check-agent-dispatch.sh:29,54-63` | EXISTS | Header claims "Tracks Agent dispatches **per session** via counter file", but `.context/working/.agent-dispatch-counter` has **no session-reset mechanism**, and **blocked attempts still increment it** (line 63 writes `NEW_COUNT` before the limit test at line 66). Observed: counter stood at 4 from prior sessions; two blocked dispatches drove it to 6 without either succeeding. The gate ratchets permanently shut until `fw dispatch reset` is run by hand. | friction |

---

## S8. GUARD LAYER

| Item | Source | Status | Data point | Kind |
|---|---|---|---|---|
| Check-script inventory | `ls scripts/check-*.sh` | EXISTS | **66** check scripts, 14,176 LOC | cost |
| Canary logs on disk | `ls .context/working/.*-canary.log` | EXISTS | **21** logs (+1 `.resolved.log`) | structure |
| Crontabs declared | `ls .context/cron/*.crontab` | EXISTS | **27** crontabs | structure |
| CLAUDE.md canary sections | `grep -cE '^### .*[Cc]anary' CLAUDE.md` | EXISTS | 21 sections — but CLAUDE.md's running tally sentence stops at **"all eighteen follow the same convention"**. Minor doc drift: the narrative count trails the real count. | structure |
| **Empty (healthy) canary logs** | snapshot 2026-09-19T17:32Z | EXISTS | 14 of 21 are 0 bytes: canary-aliveness, charter-drift, charter-sentence-drift, dead-letter, fleet-binary, fleet-capability, forever-archival, frozen-husk, preflight-doc-set-drift, release-mirror, session-control, task-finalization, topic-growth, unconfirmed-delivery, woken-but-silent | usage |
| **Non-empty (firing) canary logs** | same | EXISTS | substrate-preflight **80,430 B** · framework-pickup **63,993 B** · waker-liveness **33,097 B** · stuck-claims 8,197 B · hook-counter-integrity 6,307 B · stale-waker-code 4,216 B · fleet-doorbell-mail 418 B | friction |
| **Preflight canary firing duration** | `grep -oE '2026-[0-9]{2}-[0-9]{2}' .context/working/.substrate-preflight-canary.log \| sort -u \| wc -l` | EXISTS | **74 distinct days, 2026-07-06 → 2026-09-19, continuously unresolved** | friction |
| **What it fires on** | same log, normalised | EXISTS | **71× `[WARN] binary — termlink 0.11.N older than project VERSION 0.11.N`** · 16× `hub-binary older than project VERSION` · 10× hub exe replaced on disk · 9× be-reachable dead pid. Every recent entry's summary line reads "4 pass, 2 warn, 0 fail". | friction |
| **Mechanism making it unsatisfiable** | `crates/termlink-cli/build.rs:77,93-95` + CLAUDE.md §CI/Release | EXISTS | Version is `git describe`-derived: `major.minor.<commits-since-tag>` over **all** commits. VERSION advanced through 0.11.1716→1927→1938→1940 across this log while `crates/` received **3** commits in September. The check compares a binary against a counter driven by governance commits, so **it cannot be satisfied except momentarily after a rebuild.** Conflicting data recorded: the check's stated intent (catch stale binaries serving "unknown flag") is legitimate; its chosen comparator is not stable. | friction |
| Consequence per project's own convention | CLAUDE.md §T-2685 "Canary log hygiene" | EXISTS (claimed) | The project states: "'empty log = healthy' is a **one-bit channel**. Once a tooling error dirties the log, a subsequent genuine finding appends to an already-non-empty file and changes nothing an operator can see — the canary is not merely noisy, it is **deaf** until someone truncates it by hand." | friction |
| Guard layer finds real product defects | `git log --grep="^T-XXXX" --name-only` per task | EXISTS | Tasks originating from static checks that produced **`crates/` changes**: T-2667 (2 files) · T-2673 (3) · T-2687 (3) · T-2691 (2) · T-2699 (5). T-2748 (0). **5 of 6 verified to have produced real product fixes.** | value |
| Test/probe residue in runtime state | `topics.json` + `ls ~/.termlink/identities/` | EXISTS | **20 of 45 topics (44%) are self-test / demo / probe residue** that was never reaped: 8× `agent-conv-selftest-*`, 5× `aef-*-demo`/`aef-elect*`, 2× `aef-s8-probe*`, 3× `substrate-*-demo`, `t2838-s2-probe`, `health:ring20-fedprobe`. **40 of 66 identity keys** are likewise test residue (`t-1832-*`, `probe-*`, `*smoke*`, `relay-test-*`, `e2e-worker`, `demo-c7`). CLAUDE.md claims substrate-smoke "self-reaps its `smoke:*` topic on every exit path (T-2754)" — true for `smoke:*`, but these families do not self-reap. | friction |
| **Non-goal #2 violation, unguarded** | `topics.json` + `channel state health:ring20-fedprobe` | EXISTS | `health:ring20-fedprobe`: **1,666 records, retention `forever`**, window 2026-09-01 → 09-19 (19 days ⇒ ~88/day), payloads are synthetic `FED-PROBE-RT fedprobe-<ts>-<pid>` strings. **Invisible to both guards designed for this**: T-2252 topic-growth watches only `agent-presence`/`agent-listeners-*`/`agent-conv-*`/`dm:*` patterns (this matches none), and T-2562 forever-archival fires only above **50,000** records (this is at 1,666; at current rate it reaches the threshold in ~18 months). | friction |

*(Detailed cron-install / CI / heartbeat rows: see `.context/working/vr-run2-snapshot/ev-guards.md` — second GATHERER sub-agent. See §S9 for status.)*

---

## S9. DATA AVAILABILITY MAP

| Source (DATA LAYER A/B) | Status | Location / window | Trustworthiness |
|---|---|---|---|
| References / call graph | EXISTS | grep-based, whole repo | inferred — dynamic refs (MCP dispatch by name) not resolvable statically |
| Dead-code / unused-dep tools | **ABSENT** | `cargo machete`/`udeps` not installed | **data gap** |
| Complexity per unit | PARTIAL | LOC + fn count as proxy; no cyclomatic tool | inferred |
| Duplication detection | PARTIAL | grep on `fn *_mcp` names only | inferred |
| Git churn / hotspots | EXISTS | `git log`, 6-month window | observed — strong |
| Coverage per item | PARTIAL | test counts per crate; **no line-coverage tool run** | **data gap** — cannot say which paths are uncovered |
| Runtime logs / telemetry | PARTIAL | canary logs, `~/.termlink/*.log` | observed; but see "channel cannot report own failures" |
| Usage counters per feature | **ABSENT** for CLI/MCP verbs | no per-verb invocation counter exists | **data gap** — usage for 260 MCP tools inferred from *callers in repo*, not from invocation counts |
| Issues per component | ABSENT | no issue tracker in repo | — |
| CI failures/duration | see ev-guards.md | `.github/workflows/` | — |
| Task ledger | EXISTS | `.tasks/`, 2679 files | observed — strong |
| Arcs | EXISTS | `.context/arcs/` (7) | observed; `tasks:` list empty in every arc file, so membership read from audit output |
| Handovers / episodic | EXISTS | 1702 handovers, 2435 episodic | observed (not deeply mined this run — budget) |
| Component fabric | EXISTS | `.fabric/`, 491 cards | observed; 326 have no edges |
| Audit logs | EXISTS | `.context/audits/`, 151 files | observed — strong |
| Gate bypass log | EXISTS | 482 entries, Apr–Sep | observed — strong |
| Healing events | NOT SAMPLED | `fw healing` | **not reviewed — budget** |
| Token telemetry per task/arc | NOT SAMPLED | — | **not reviewed — budget** |
| BVP scores / realization log | PARTIAL | `.context/audits/bvp-realization.jsonl` not verified this run | **data gap** — predicted-vs-realized value not assessed |
| TermLink topic log | EXISTS | live hub SQLite via `channel list/state` | measured — strongest usage source |
| Subscriber cursors | PARTIAL | `~/.termlink/cursors.json` (688 B) present, not parsed | — |
| Presence / heartbeats | EXISTS | agent-presence, 17.4 h retained window | measured; **window is short** (1000-msg cap ÷ 60 s beats) |
| Out-of-band observer of TermLink | PARTIAL | canary logs + `fleet doctor` are TermLink-adjacent; `aef-install-findings` from `/opt/1409-sprind` is genuinely external | **partial gap** |
| Workflow Designer / BPMN | **ABSENT** | no `aef:`-namespace workflow files found in repo | not applicable |

---

## S10. CONTRADICTIONS RECORDED (facts in tension — not resolved here)

1. **"Shipped == live" vs the rails.** CLAUDE.md documents an extensive shipped-equals-live gate (T-2480) and three fleet-fitness canaries, yet the push-wake rail is measured DARK and V3 claims measured 0 — i.e. the capabilities are shipped and the gate exists, but the rails they guard carry no traffic.
2. **Preflight canary self-report vs its own summary line.** Every recent entry ends "4 pass, 2 warn, **0 fail** — substrate will run", while the canary's contract treats any entry as a firing that dirties a one-bit channel.
3. **Guard-layer value is real AND its noise is real.** 5 of 6 sampled static-check tasks produced genuine `crates/` fixes (S8), while the preflight canary has produced 74 days of unactioned output (S8).
4. **Binary staleness.** Preflight reports the binary 174 versions behind; the git record shows only 1 `crates/` commit since it was built. Both are true; they measure different things.
5. **Charter non-goal #3 (not social/engagement) vs the surface.** 28 live off-charter analytics tools remain, all acknowledged in an allowlist described as "a ledger of an open question, not a permanent exemption", pending T-2548 — an open human decision since before 2026-08.
6. **CLAUDE.md canary tally** says "all eighteen"; there are 21 canary sections, 21 logs, 27 crontabs.

---

## S11. NOT REVIEWED (budget / out of reach)

- Healing events (`fw healing`), token telemetry per task/arc, BVP realization log.
- Line-level test coverage; unused dependencies (tooling absent).
- Handover/episodic corpus mining (1702 + 2435 files) beyond audit-derived summaries.
- Peer hubs' own state (`.121`, `.122`, `.107`) — project-boundary gate prevents cross-project reads from this session.
- `.agentic-framework/` internals as a deletable surface — vendored (G-062); local edits are erased on re-vendor, so it is upstream's scope.
