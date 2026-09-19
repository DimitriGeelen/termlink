# Value Review — Evidence File (GATHERER)

- **Scope:** whole repo (`/opt/termlink`)
- **Date:** 2026-09-19
- **Run:** run5 (execution 5 of 5 in a consecutive repeatability series; independent — prior runs' reports not read)
- **Role:** GATHERER. Facts only. No classification, no recommendations.
- **Snapshot taken:** 2026-09-19T18:55:23Z (= 20:55:23 local, CEST/UTC+2)
- **Host clock note:** filesystem mtimes shown in this file are **local (UTC+2)** unless suffixed `Z`.

> **Pollution control.** Usage data was snapshotted at 18:55:23Z before any check
> was executed (`/tmp/vr5/rpc-audit-snapshot.jsonl`, `/tmp/vr5/bypass-snapshot.yaml`).
> Every heartbeat-mutating observation below is timestamped against that boundary
> so reader can separate pre-existing state from this review's footprint.
> See E-23 for one case where that separation was load-bearing.

---

## Baseline

| Check | Command | Result |
|---|---|---|
| Workspace tests | `cargo test --workspace` | **PASS** (exit 0) |
| Workspace build | `cargo build --workspace` | **PASS**, **zero warnings** |
| Guard layer | `scripts/run-guard-layer.sh --json` | **did not complete** within 600s wall; JSON file 0 bytes at cutoff. Recorded as INCOMPLETE, not PASS. |
| Cron install drift | `scripts/check-cron-install-drift.sh --json` | `ok:false`, 1 MISSING, 26 OK |
| Stranded-finalized tasks | `scripts/check-stranded-finalized-tasks.sh --json` | `ok:true`, 0 firing, 71 partial-complete |
| Charter drift | `scripts/check-charter-drift-freshness.sh --json` | `ok:true`, 214 checked, 0 unacknowledged, 28 acknowledged |
| MCP parity census | `scripts/check-mcp-parity-census.sh --json` | `ok:true`, 260 total, 24 covered (9.2%), 236 acknowledged |

---

## Data availability map

| Source | Status | Location | Window | Trustworthiness |
|---|---|---|---|---|
| Hub RPC audit log | **EXISTS** | `/var/lib/termlink/rpc-audit.jsonl` (92.9 MB, 921,543 lines, 0 malformed) | 2026-08-16T14:53Z → 2026-09-19T18:55Z = **34.2 days** | **Measured.** Single central call site `server.rs:1610`. Skips only `event.poll`, `event.collect` (`rpc_audit.rs:46`). **Scope limit: hub-routed calls only** — see E-12. |
| Canary logs + heartbeats | **EXISTS** | `.context/working/.*-canary.log{,.heartbeat}` | per-canary, up to 75 days | Measured, but see E-21/E-22 (masking). |
| Gate bypass log | **EXISTS** | `.context/working/.gate-bypass-log.yaml` (2867 lines, 482 entries) | — | Observed |
| Task ledger | **EXISTS** | `.tasks/active` (246), `.tasks/completed` (2433) | — | Observed |
| Concerns register | **EXISTS** | `.context/project/concerns.yaml` (52 entries) | — | Observed |
| Audit history | **EXISTS** | `.context/audits/*.yaml` (151 files) | — | Observed |
| Episodic memory | **EXISTS** | `.context/episodic/` (2435 files) | — | Observed |
| Handovers | **EXISTS** | `.context/handovers/` (1706 `.md`) | — | Observed |
| Topic/bus state | **EXISTS** | `termlink channel list --json` (45 topics, 4336 records) | current | Measured (point-in-time, not history) |
| Fleet hub versions | **EXISTS** | `termlink fleet doctor --json` (5 hubs) | current | Measured |
| Git churn | **EXISTS** | `git log --since='180 days ago'` | 180 days | Observed |
| **MCP per-tool invocation telemetry** | **ABSENT** | — | — | **No instrumentation exists.** No per-tool log found under `~/.termlink/`; no audit hook in `termlink-mcp`. 262 tools have UNMEASURED usage. |
| **CLI per-verb invocation telemetry** | **ABSENT** | — | — | No local verb counter found. |
| Out-of-band observer of TermLink delivery | **ABSENT** | — | — | Delivery evidence is bus-self-reported. `scripts/session-message-selftest.sh` exists as an out-of-band prover but is on-demand, not scheduled. |
| `cargo-udeps` / `cargo-machete` | **ABSENT** | not installed | — | Unused-dependency analysis not performed. |
| Workflow Designer / BPMN | **ABSENT** | no `aef:`-namespace workflow files found in repo | — | Determinism map unavailable. |
| Execution traces by node uid | **ABSENT** | `fw workflow run` not present | — | Recorded as gap per prompt. |

---

## Yardstick (from `docs/CHARTER.md`, human-blessed)

**Purpose (canonical sentence):** "TermLink is a hub-mediated, durable append-log
message bus with terminal endpoints — the coordination substrate that lets a fleet
of AI agents (and humans) discover each other, exchange durable messages, claim
work, and control terminal sessions across one or many machines."

**Four core verbs:** (1) discover · (2) exchange durable messages · (3) claim work ·
(4) control terminal sessions.

**Non-goals:** 1 not inter-hub federation · 2 not a durable database/system of
record · 3 not a social/engagement platform · 4 not a workflow/orchestration engine ·
5 not a security boundary between mutually-distrusting tenants.

---

## Evidence rows

| ID | Item | Source | Status | Data point (citation) | Window | Kind |
|---|---|---|---|---|---|---|
| E-01 | Hub RPC surface vs. actual traffic | rpc-audit snapshot | EXISTS | **28 distinct methods observed** out of **54 declared** in `crates/termlink-protocol/src/control.rs`. Total 921,543 calls. | 34.2 d | usage |
| E-02 | Traffic concentration | rpc-audit | EXISTS | `channel.subscribe` 387,716 (42.1%); `hub.auth` 233,733 (25.4%); `session.discover` 94,539 (10.3%); `channel.post` 53,538 (5.8%); `channel.create` 52,695 (5.7%); `channel.list` 45,402; `hub.capabilities` 30,604; `hub.version` 18,180. Top 3 = 77.8% of all traffic. | 34.2 d | usage |
| E-03 | **Charter verb 3 ("claim work") traffic** | rpc-audit | EXISTS | `channel.claim` 51, `channel.claims` 21, `channel.release` 18, `channel.renew` 9, `channel.transfer_claim` 9, `channel.claims_summary` 1368, `agent.find_idle` 5. Excluding the read-only summary poll, the **write-side claim lifecycle totals 108 calls = 0.012%** of hub traffic. | 34.2 d | usage |
| E-04 | **Charter verb 4 ("control terminal sessions") traffic** | rpc-audit | EXISTS | `command.execute` 94, `command.inject` 24. `command.resize` 0, `command.signal` 0, `pty.mode` 0. | 34.2 d | usage |
| E-05 | `channel.create` ≈ `channel.post` | rpc-audit | EXISTS | 52,695 creates vs 53,538 posts — a ~0.98 create:post ratio against only **45 topics currently existing**. | 34.2 d | usage / friction |
| E-06 | Declared methods with **zero** hub-observed calls | rpc-audit ∆ `control.rs` | EXISTS | 31 methods: `artifact.get`, `artifact.put`, `auth.token`, `channel.force_release`, `channel.set_retention`, `command.resize`, `command.signal`, `dialog.presence`, `event.broadcast`, `event.collect`*, `event.emit`, `event.error`, `event.poll`*, `event.state_change`, `event.subscribe`, `event.topics`, `kv.delete`, `kv.get`, `kv.list`, `kv.set`, `orchestrator.bypass_invalidate`, `orchestrator.bypass_status`, `orchestrator.route`, `pty.mode`, `query.capabilities`, `query.output`, `session.deregister`, `session.heartbeat`, `session.register`, `session.update`, `session.whoami`. (*`event.poll`/`event.collect` are **skip-listed by design**, `rpc_audit.rs:46` — no inference possible.) | 34.2 d | usage |
| E-07 | `orchestrator.route` — wiring | source read | EXISTS | Implemented: `hub/src/router.rs:78` dispatch → `handle_orchestrator_route` at `router.rs:1186`. **No CLI surface** (`grep` over `termlink-cli/src` = 0 files), **no MCP tool** (0 files). 0 calls. | 34.2 d | structure / usage |
| E-08 | `dialog.presence` — wiring | source read | EXISTS | Implemented: `hub/src/router.rs:140` → `channel::handle_dialog_presence`; tracker at `hub/src/channel.rs:28` (T-1286/T-243). **No CLI surface, no MCP tool.** 0 calls. | 34.2 d | structure / usage |
| E-09 | `kv.*` — wiring | source read | EXISTS | `kv.get` matched in **hub = 0 files**, cli = 2, mcp = 1. CLI help lists `kv  Manage key-value metadata on a session`. MCP exposes 5 tools (`termlink_kv_{get,set,del,list,watch}`). | current | structure |
| E-10 | `event.broadcast` declared legacy | source read | EXISTS | Listed in `LEGACY_METHODS`, `hub/src/rpc_audit.rs:54-58`, described as "targeted for retirement" by the T-1166 entry gate. 0 calls in window. | 34.2 d | structure |
| E-11 | Topic inventory | `channel list --json` | EXISTS | 45 topics, 4,336 records total. Retention: 27 `messages`, **18 `forever`**. 2 topics hold zero records. Largest: `health:ring20-fedprobe` **1672 records, retention=forever**; `agent-presence` 1053; `agent-chat-arc` 1001; `channel:learnings` 191 (forever); `framework:pickup` 126. | current | usage / structure |
| E-12 | **Audit scope limit** | source read | EXISTS | `rpc_audit::record` is called from exactly two sites, both in `hub/src/server.rs` (1484, 1610). Session-scoped calls served by the session daemon over the local socket do **not** pass through it. `kv.*` has no hub implementation (E-09) yet appears in E-06's zero-list — establishing that **E-06 means "never routed through the hub", not "never used"**. | — | structure (caveat) |
| E-13 | MCP tool count | source + census | EXISTS | `check-mcp-parity-census.sh`: **260 tools**, 24 parity-asserted (**9.2%**), 236 acknowledged-uncovered, 0 unexamined. Raw source count `grep -cE 'name = "termlink_' tools.rs` = **262** (census strips comments; 2-tool delta). | current | structure |
| E-14 | **`tools.rs` size / churn** | `wc -l`, `git log` | EXISTS | `crates/termlink-mcp/src/tools.rs` = **46,458 lines in one file**; **390 commits touching it in 180 days** — the #1 churn file in the repo. | 180 d | cost / structure |
| E-15 | Other hotspots | `wc -l`, `git log` | EXISTS | `cli/src/cli.rs` 7,001 L / 356 commits; `cli/src/main.rs` 1,931 L / 351 commits; `cli/src/commands/channel.rs` **20,435 L** / 166 commits; `hub/src/router.rs` 3,843 L / 70 commits. | 180 d | cost / structure |
| E-16 | Crate sizes | `find`+`wc` | EXISTS | Total Rust 180,686 L. cli 72,790 · mcp 50,828 · hub 24,550 · session 23,838 · bus 5,240 · protocol 3,093 · test-utils 347. **mcp+cli = 68% of all code.** | current | structure |
| E-17 | Charter non-goal #3 surface | `check-charter-drift-freshness.sh --json` | EXISTS | 214 live tools scanned; **28 off-charter**, all 28 in `.context/checks/charter-drift-allowlist`, every line reading `# T-2548 pending: …` (e.g. `termlink_agent_top_repliers # T-2548 pending: leaderboard, traces to no charter verb`). `live_off_charter: 0`. | current | structure / value |
| E-18 | **T-2548 is closed** | task file | EXISTS | `.tasks/completed/T-2548-…md`: `status: work-completed`, `date_finished: 2026-08-20T17:54:28Z` (**30 days before this review**). Body line 125-126: "Agent recommendation = **GO to subtract** (restore non-goal #4), mirroring the T-2471/T-2478 precedent". Line 156: "**OUT of scope:** the actual removal (a GO build task)". Its `## Decision` section is **empty** (template text only). | — | value / structure |
| E-19 | No follow-on removal task | `grep -rl T-2548 .tasks/` | EXISTS | Hits are T-2971 (this review), T-2470, T-2549, T-2678, T-2680, T-2683, T-2690, T-2716 — **none is a build task to subtract the 28 tools**. `grep -rli 'conversation-analytics' .tasks/active/` = **0 files**. | — | structure |
| E-20 | **Canary probe binary is stale** | `check-charter-drift-freshness.sh --json` | EXISTS | Envelope reports `"probe_binary": "/root/.cargo/bin/termlink"`, `"probe_version": "0.11.1766"`. Repo `VERSION` = **0.11.1944**. The charter-drift guard audits the **installed binary's** tool surface, not the repo's. | current | structure |
| E-21 | **Binary staleness, fired daily for 75 days** | `.substrate-preflight-canary.log` | EXISTS | **74 dated entries**, first `2026-07-06T03:23:01Z`, last `2026-09-19T03:23:01Z`. `[WARN] binary` appears **71 times**; `[WARN] hub-binary` **26 times**. Latest entry: installed `0.11.1716` vs VERSION `0.11.1938`; hub serves `0.11.1766`. | 75 d | usage / friction |
| E-22 | **`/canaries` reports this layer green** | `scripts/canary-status.sh --json` | EXISTS | Summary at 2026-09-19 ~20:58 local: `{"total":30,"healthy":28,"firing":0,"stale":0,"no_heartbeat":0,"not_scheduled":2}`. **Zero firing** — while 7 logs are non-empty, two of them appended findings *the same day*. | current | friction |
| E-23 | **Mechanism of the green reading** | source + `stat` | EXISTS | `canary-status.sh:20-21`: `FIRING — log non-empty AND latest log entry mtime >= latest heartbeat mtime`. Heartbeat is touched **early** (`substrate-preflight.sh:162-164`, before findings print), so a *cron* run yields FIRING correctly. But an **ad-hoc run refreshes the heartbeat without appending to the log**, flipping FIRING→HEALTHY. Measured: `.substrate-preflight-canary.log` mtime **09-19 05:23** (cron is `23 5 * * *`), heartbeat mtime **09-19 20:46:12**. Similarly framework-pickup/waker-liveness/stale-waker-code heartbeats at **20:13**. **This review's first write was 20:55:23 local** (`/tmp/vr5/snapshot.txt`) — so these touches **predate this session by 9–42 minutes** and are **not** this review's pollution. None of the five scripts carries a `# guard-layer:` marker, so `run-guard-layer.sh` did not touch them either. | current | friction |
| E-24 | **Push-wake rail is dark** | `.waker-liveness-canary.log` | EXISTS | **36 `FIRING` entries.** Last (2026-09-19 07:53 local): `RAIL DARK … ZERO LIVE listeners carry pty_session on this hub — the G-069 '0 wakers' state. Every DM sent here waits on the ~15s poll floor at best, forever at worst.` Also `[LIVE-no-waker] penelope fp=d1993c2c3ec44c94 age=56s` — presence-advertised, unwakeable. | current | usage / friction |
| E-25 | **Repo's own shipped≠live probe fails** | `scripts/arc-live-probe.sh` | EXISTS | `bash scripts/arc-live-probe.sh --hub 127.0.0.1:9100 --min-version 0.11.1944 --capability cv-keys` → `SHIPPED-BUT-NOT-LIVE (hub '127.0.0.1:9100') — served version 0.11.1766 is below floor 0.11.1944`. | current | usage |
| E-26 | Fleet version spread | `fleet doctor --json` | EXISTS | 5 hubs. `127.0.0.1:9100` → 0.11.1766; `192.168.10.107` → 0.11.1766; `192.168.10.121` → **0.11.1411**; `192.168.10.122` → **0.11.1411**; `192.168.10.141` → version unknown. Repo VERSION 0.11.1944. Spread = **533 commits** on the two ring20 hubs (see E-27 for the lineage caveat). | current | usage |
| E-27 | Lineage caveat on version comparison | `CLAUDE.md` §fleet-binary canary | EXISTS | Documented: patch numbers are commits-since-tag and "are NOT comparable across build lineages OR across tag epochs"; `.121` is floor-**exempt**. So E-26's "533 commits behind" is **not** safely inferable for `.121`/`.122`. | — | structure (caveat) |
| E-28 | **11 unprocessed inbound peer filings** | `.framework-pickup-canary.log` | EXISTS | Last entry (mtime 2026-09-18 06:37 local) lists offsets 106–120 unacked: **P-064…P-076**, comprising **4 `pickup-bug-report`, 2 `pickup-feature-proposal`, 5 `pickup-learning`**. Footer: "10 own filing(s) from 010-termlink not counted". This canary reads HEALTHY in E-22. | current | usage |
| E-29 | The concern this recurs against | `concerns.yaml` | EXISTS | `G-063` — "framework:pickup topic accumulates pickups with zero receipts — write-only", status `watching`, **105 days** old. | 105 d | structure |
| E-30 | Concern register shape | `concerns.yaml` | EXISTS | 52 entries. Status: **watching 27**, closed 12, resolved 6, mitigated 5, decided-build 2. Severity: medium 30, **high 15**, low 6, **critical 1**. | — | structure |
| E-31 | Oldest open concerns | `concerns.yaml` | EXISTS | G-008 **157 d** ("64 tasks stuck in partial-complete state — Human ACs accumulate unchecked"); G-009 153 d; G-010 153 d; G-011 152 d; G-015 150 d; G-053 140 d; G-055 133 d; G-059 124 d; G-061 116 d; G-062 110 d; G-063 105 d; G-064 103 d ("Hub has no per-user authorization model"). | — | structure |
| E-32 | **G-008 has grown, not shrunk** | `check-stranded-finalized-tasks.sh` | EXISTS | G-008 recorded **64** partial-complete tasks 157 days ago. Measured today: **`partial_complete_count: 71`** across 246 active tasks. Same check reports `firing_count: 0` (none are the T-2833 stranded defect). | 157 d | usage |
| E-33 | Active backlog shape | `.tasks/active` frontmatter | EXISTS | 246 active. Status: captured 119, **work-completed 71**, started-work 56. Type: build 201, inception 36, refactor 4, test 2, design 2, decommission 1. Owner: **human 137**, agent 107, claude-code 2. Horizon: now 151, next 53, later 42. | — | structure |
| E-34 | Backlog age | `.tasks/active` frontmatter | EXISTS | **>90 days: 95 tasks**; >60: 16; >30: 70; ≤30: 65. So **111 of 246 (45%) are older than 60 days**. | — | structure |
| E-35 | Gate bypass volume | bypass-log snapshot | EXISTS | **482 entries**, 2867 lines. Top reasons: "Phase A batch close — agent-evidenced (tool registered + silent-OK…)" ×**93**; "Inception decision: GO" ×82; "Completed via Watchtower UI (human action)" ×51; empty `''` ×31; "Confirmed duplicate of T-2256 (P-047 self-echo into own pickup inbox)" ×8; "Self-echo duplicate of T-2259" ×8; "Human reviewed" ×5; "Inception decision: NO-GO" ×5. Only 4 entries carry a `gate:` field (all `human-ac-tick-guard`). | — | friction |
| E-36 | Cron install drift | `check-cron-install-drift.sh` | EXISTS | 26 crontabs OK, **1 MISSING**: `substrate-smoke-canary.crontab → /etc/cron.d/termlink-substrate-smoke-canary`. Matches already-filed active task **T-2939**. | current | structure |
| E-37 | Hook telemetry is corrupt | `.hook-counter-integrity-canary.log` | EXISTS | `FIRING — counter file is corrupt (21 line(s) scanned)`. DUPLICATE key `check-arc-id`; READER DISAGREEMENT on same file: `fw_hook_counter_get`=**1** vs `hook-threshold.py`=**7**. Stated mechanism: "unlocked truncate+write in `lib/hook-telemetry.sh _fw_telemetry_increment`. This is **L-023 recurring**." Log notes the summing reader "feeds the T-1626 decay alarm denominator, so each duplicate roughly halves the apparent failure ratio (**false silence**)". | current | friction |
| E-38 | Consequence of E-37 | inference from E-35+E-37 | EXISTS | Gate first-pass / bypass **rates** (a DATA LAYER B row) are computed from this counter file. With duplicate keys and 1-vs-7 reader disagreement, per-gate friction rates in this repo are **not trustworthy**. Raw bypass *entries* (E-35) remain countable. | current | structure (caveat) |
| E-39 | Script orphans | ref scan | EXISTS | **190 scripts** in `scripts/`; **0 orphans** — every filename is referenced by at least one other tracked file. (Caveat: "referenced" includes prose mentions in `docs/`/`CLAUDE.md`, so this does not prove execution.) | current | structure |
| E-40 | Guard-layer membership coverage | marker scan | EXISTS | **44** scripts carry `# guard-layer: source`. Five canaries verified to carry **no** marker: `substrate-preflight`, `check-framework-pickup-freshness`, `check-waker-liveness-freshness`, `check-stale-waker-code-freshness`, `check-hook-counter-integrity` — i.e. they are cron-tier only and are never exercised by `run-guard-layer.sh`. | current | structure |
| E-41 | Guard-layer runner did not complete | baseline | EXISTS | `run-guard-layer.sh --json` produced a **0-byte** JSON after >600 s wall clock; `--json` output is written only at the end, so no partial verdicts were recoverable. Documented budget is "seconds" for the source tier / "minutes" with `--tests` (CLAUDE.md §Running the guard layer). | current | friction |
| E-42 | Code hygiene | `cargo build` | EXISTS | Workspace builds with **zero warnings**. Only **4** `#[allow(dead_code)]` across all crates. | current | structure |
| E-43 | Dependency surface | `Cargo.toml` | EXISTS | **30** workspace dependencies. Unused-dependency analysis **not performed** (`cargo-udeps`/`cargo-machete` absent, E-map). | current | cost |
| E-44 | CLI verb surface | `termlink --help` | EXISTS | **41** top-level commands. (From the stale 0.11.1766 binary — the repo surface may differ; cf. E-20.) | current | structure |
| E-45 | Project-boundary gate fires on read-only access to hub runtime | observed | EXISTS | `cd /var/lib/termlink && cp rpc-audit.jsonl …` was **blocked** by `check-project-boundary` ("cd to /var/lib/termlink (outside project root)"), though the target is this project's own hub runtime dir and the operation is a read. Same read succeeded via absolute path with no `cd`. | current | friction |
| E-46 | Agent-dispatch gate blocked on a stale counter | observed | EXISTS | `Agent` tool blocked: "BLOCKED: Agent dispatch #6 exceeds limit (2)" on this session's **first** dispatch attempt — the counter carried over from a prior session. Offered remedies: `fw dispatch approve` (5-min window) or `fw dispatch reset`. Policy T-533. | current | friction |
| E-47 | Docs surface | `find docs -name '*.md'` | EXISTS | **396** markdown files under `docs/`. | current | structure |
| E-48 | Task/test-script surface | `ls` | EXISTS | 83 `tests/*.sh` fixture suites; 2433 completed tasks; 2435 episodic files; 1706 handover `.md`. | current | structure |

---

## NON-USE DIAGNOSIS

For each low/no-use item, evidence gathered for **each** reading. Readings are
**not** selected here — that is the JUDGE's job.

### N-1 — `orchestrator.route` (hub-implemented, 0 calls / 34.2 d)
- **A BROKEN?** No failing test found; no healing event or error referencing it. Not exercised (no client surface to exercise it *from*). **No evidence.**
- **B NEVER WIRED?** **Strong.** Implemented and dispatched (`router.rs:78`, `:1186`) but **no CLI verb and no MCP tool** reference it (E-07). A hub method with no client surface cannot be called by any first-party caller.
- **C UNDISCOVERABLE?** Consistent with B — absent from `termlink --help`'s 41 verbs (E-44).
- **D UNMEASURED?** Partially ruled out: it is a hub-routed method, so it **would** appear in the audit if called (E-12 caveat does not apply to hub-implemented methods).
- **E NOT WANTED?** `no_federation_tripwire.rs:43,177` actively reasons about it as a live residual path — that is **intent evidence**, arguing against E.
- **Intent evidence:** referenced by a current tripwire test; relates to charter verb 1 (discover) + delegation.

### N-2 — `dialog.presence` (hub-implemented, 0 calls / 34.2 d)
- **A BROKEN?** No evidence either way — never exercised.
- **B NEVER WIRED?** **Strong.** Implemented (`router.rs:140`, `channel.rs:28`, origin T-1286/T-243) with **no CLI verb, no MCP tool** (E-08).
- **C UNDISCOVERABLE?** Consistent with B.
- **D UNMEASURED?** Ruled out — hub-routed, would appear if called.
- **E NOT WANTED?** No recorded decision abandoning multi-turn dialog tracking was found.
- **Intent evidence:** carries a documented purpose in-code ("passive multi-turn dialog presence tracker", keyed on `metadata.conversation_id`), which serves charter verb 2.

### N-3 — `kv.*` (4 methods, 0 hub-observed calls)
- **A BROKEN?** Not exercised in this review (no sandbox run attempted).
- **B NEVER WIRED?** **Ruled out.** Fully surfaced: CLI verb `kv` in help, 5 MCP tools (E-09).
- **C UNDISCOVERABLE?** Ruled out — appears in `termlink --help`.
- **D UNMEASURED?** **Strong.** `kv.get` has **no hub implementation** (hub=0 files, E-09), so kv traffic is session-daemon-local and **structurally invisible** to the hub audit (E-12). Usage cannot show up in the only measured source.
- **E NOT WANTED?** No evidence.

### N-4 — `artifact.get` / `artifact.put` (0 hub-observed calls)
- **A/B/C:** Implemented in hub (2 files), CLI (2), MCP (1) — wired and surfaced. B and C weakly ruled out.
- **D UNMEASURED?** Unlikely to apply: hub-implemented, so hub-routed calls would be recorded.
- **E NOT WANTED?** No recorded abandonment decision found.
- **Net:** wired, surfaced, hub-visible, and still **zero calls in 34.2 days**. No reading is affirmatively established by the evidence collected.

### N-5 — `event.broadcast` and the `event.*` family (0 observed; 2 skip-listed)
- **D UNMEASURED?** **Decisive for two members:** `event.poll` and `event.collect` are in `SKIP_METHODS` (`rpc_audit.rs:46`) — **no inference is possible** about them.
- **E NOT WANTED?** **Direct evidence for `event.broadcast`:** it is enumerated in `LEGACY_METHODS` (`rpc_audit.rs:54-58`) as "targeted for retirement" by the T-1166 entry gate (E-10) — a recorded intent to retire.
- Remaining members (`event.emit`, `event.subscribe`, `event.topics`, `event.state_change`, `event.error`) have CLI/MCP surface (E-06 shows `event.subscribe` hub=3/cli=4/mcp=1) and zero hub-observed traffic.

### N-6 — Charter verb 3, "claim work" (108 write-side calls / 34.2 d, E-03)
- **A BROKEN?** **Ruled out.** `scripts/substrate-smoke.sh` drives the full claim→renew→release lifecycle and `check-stuck-claims-freshness.sh` exists; the stuck-claims canary log's last entry is 2026-08-16 and the current `claims_summary` poll runs 1368×/34 d without firing.
- **B NEVER WIRED?** Ruled out — full CLI + MCP + skill surface (`/claim`, `/release`, `/renew`, `/claim-transfer`, `/claims`).
- **C UNDISCOVERABLE?** Ruled out — extensively documented in CLAUDE.md and `docs/operations/substrate-orchestrator-recipe.md`.
- **D UNMEASURED?** Ruled out — hub-routed and measured precisely.
- **E NOT WANTED?** No decision abandoning work-stealing found; it is **named in the charter's canonical sentence**.
- **Net:** the verb is built, wired, discoverable, measured — **and used 108 times in 34 days**, against 921,543 total calls. Intent evidence (charter) is maximal.

### N-7 — The 28 off-charter analytics MCP tools (E-17/E-18/E-19)
- **A BROKEN?** No evidence.
- **B NEVER WIRED?** Ruled out — live and registered (they appear in `termlink help --json`, which is how the drift canary counts them).
- **C UNDISCOVERABLE?** Not established.
- **D UNMEASURED?** **Decisive.** No MCP per-tool telemetry exists (data map). Their usage is **structurally unmeasurable** today. Their hub-side reads would appear only as generic `channel.subscribe`, indistinguishable from any other reader.
- **E NOT WANTED?** **Direct evidence.** Charter non-goal #3 names social-analytics surfaces that trace to no coordination purpose as "removal candidates, not features". T-2548's own body records **"Agent recommendation = GO to subtract"** (E-18). The allowlist's 28 reasons each assert "traces to no charter verb".
- **Counter-evidence:** T-2548's `## Decision` section is **empty**; the recommendation is the *agent's*, and sovereignty over non-goal #3 is explicitly the human's. No follow-on removal task exists (E-19).

### N-8 — `command.resize`, `command.signal`, `pty.mode` (0 calls, E-04)
- **D UNMEASURED?** **Strong.** These are session-scoped; `pty.mode` has hub=0 (E-06 row / E-09 method). Per E-12, local session traffic bypasses the hub audit entirely.
- **B/C:** `pty.mode` has an MCP tool (mcp=1) but no CLI surface (cli=0).
- **E:** no evidence.

### N-9 — `health:ring20-fedprobe` topic (1672 records, retention=forever, E-11)
- **D UNMEASURED?** Subscriber cursors per topic were not extracted — readership unknown.
- **E NOT WANTED?** No evidence. It is the largest single topic on the hub and is actively grown.
- **Note:** it is `forever` and is **not** on the operator-durable exclusion list documented for the forever-archival canary (`channel:learnings`, `policy-decisions`, `framework:pickup`, `broadcast:global`). At 1,672 records it sits far below that canary's 50,000 ceiling, so nothing fires.

---

## Reverse map — capability/verb → what serves it

| Charter element | Items serving it | Measured activity |
|---|---|---|
| Verb 1 — discover | `session.discover`, `agent-presence` topic, `channel.cv_keys`, `agent.find_idle`, `/peers`, `/find-idle` | `session.discover` 94,539; `channel.cv_keys` 74; `agent.find_idle` 5 |
| Verb 2 — exchange durable messages | `channel.post/subscribe/receipts`, DM topics, offline queue, `--await-ack` | `channel.subscribe` 387,716; `channel.post` 53,538; `channel.receipts` 1,565 — **but push-wake rail dark (E-24)** |
| Verb 3 — claim work | `channel.claim/renew/release/transfer_claim`, `/claim*` skills, `substrate-smoke.sh` | **108 write-side calls (E-03)** |
| Verb 4 — control terminal sessions | `command.execute/inject/resize/signal`, `pty.mode`, `spawn`, `session-selftest.sh` | `command.execute` 94, `command.inject` 24; resize/signal/pty.mode 0 (E-12 caveat) |
| Non-goal 1 (no federation) | `no_federation_tripwire.rs` | Enforced at build time |
| Non-goal 2 (not a database) | topic-growth + forever-archival canaries | 45 topics / 4,336 records — small (E-11) |
| Non-goal 3 (not social) | charter-drift canary + allowlist | **28 live off-charter tools, all acknowledged (E-17)** |
| Non-goal 4 (not orchestration) | charter, T-2549 | `orchestrator.route` exists in hub, unwired (E-07) |
| Non-goal 5 (not a tenancy boundary) | — | **G-064 open 103 d: "Hub has no per-user authorization model" (E-31)** |

---

## Conflicting / contradictory data points (recorded side by side)

1. **`/canaries` green vs. 7 non-empty canary logs.** E-22 reports `firing: 0`; E-21/E-24/E-28/E-37 show four canaries with substantive findings, two appended the same day. Both readings are internally consistent with `canary-status.sh`'s stated rule (E-23) — the rule and the documented "empty log = healthy" convention disagree with each other.
2. **MCP tool count: 260 (census) vs 262 (raw grep).** E-13.
3. **Charter-drift "214 checked" vs 260/262 MCP tools.** The drift canary scans the **installed 0.11.1766** binary's registry (E-20); the census scans **repo source** (E-13). The two guards measure different artifacts.
4. **`cargo build` zero warnings and zero orphan scripts (E-42/E-39) vs. 46,458-line single file (E-14).** Hygiene metrics are clean while structural concentration is extreme.
5. **T-2548 `work-completed` + human AC ticked, vs. empty `## Decision` section and no follow-on task (E-18/E-19).** Whether a human GO was actually recorded is not determinable from the task file.
6. **G-008 states 64 partial-complete tasks; measurement today is 71 (E-32).** The concern text is stale relative to its own subject.
7. **Guard layer documented as "seconds" vs. >600 s incomplete run (E-41).**

---

## Not gathered (explicit gaps)

- Unused-dependency analysis (`cargo-udeps`/`cargo-machete` absent).
- Per-topic subscriber cursor/offset extraction (readership per topic).
- Coverage-per-item (no coverage tool run).
- Fix/revert ratio per area, change-coupling, author concentration.
- Token telemetry per task/arc; BVP scores; realization log (`bvp-realization.jsonl` not located).
- Install-findings harness / upstream report contents.
- Audit-history recurrence analysis across the 151 `.context/audits/*.yaml`.
- Workflow Designer determinism map (ABSENT — no `aef:` workflow files).
- Guard-layer per-member verdicts (run incomplete, E-41).

---

## ADDENDUM — gathered AFTER the JUDGE's read cutoff

> The JUDGE worker read this file at ~21:10 local. Everything below was gathered
> after that moment, so the JUDGE's verdicts **do not** account for it. These rows
> are therefore **self-judged** in the report and carry a one-level confidence
> reduction, except where they merely corroborate a row the JUDGE already saw.

| ID | Item | Source | Status | Data point (citation) | Window | Kind |
|---|---|---|---|---|---|---|
| E-49 | **Three installed binaries, two versions** | `scripts/check-installed-binary-drift.sh` via guard layer | EXISTS | **FAIL.** `/root/.cargo/bin/termlink` = **0.11.1766** (mtime 2026-09-01); `/root/.local/bin/termlink` = **0.11.1716** (2026-08-29); `/usr/local/bin/termlink` = **0.11.1716** (2026-08-29); `/usr/bin/termlink` absent. Repo VERSION **0.11.1944**. Which binary runs depends on `PATH` order. *Corroborates E-21/E-25/E-26 — third independent source.* | current | usage |
| E-50 | Stranded pickup envelope | `scripts/check-pickup-deferred-freshness.sh` via guard layer | EXISTS | **FAIL.** 1 STRANDED: `P-078-learning.yaml`, deferred 9 days, **no breadcrumb** → `fw pickup promote-deferred` can never promote it. Matches active task **T-2960**. | 9 d | usage |
| E-51 | **Half of GO inceptions never propagate scope** | `.context/audits/2026-09-19.yaml` findings | EXISTS | WARN: "Found **77 GO-scope-not-propagated inception(s) of 158** GO-recorded completed inception(s) examined — GO recorded, `related_tasks` empty, nobody back-references, no `unlocks_inception_decision`" = **48.7%**. Present in ≥11 of the last 30 audits (also seen at 76/157). **T-2548 (E-18/E-19) is one instance of this systemic pattern, not an isolated lapse.** | 30 audits | usage / structure |
| E-52 | Recurring audit findings that never close | 30 most recent `.context/audits/*.yaml` | EXISTS | Top recurrences: 3 separate WARNs reading "**NOT EVALUATED: candidate set empty**" (18/30 each — they warn because they had nothing to check); "Arc 'mcp-slimming' has no task commits in the last 30 days" 18/30; "free driver F-ORCH: retire_when condition appears met" 16/30; "Arc 'arc-substrate-fitness' no task commits in 30 days" 23/30 combined; `cron(substrate-smoke-canary)` **FAIL** 7/30; "Cron drift: agentic-audit.crontab differs" **FAIL** 6/30 (matches active T-2938). | 30 audits | friction |
| E-53 | Daily audit steady state | `.context/audits/` | EXISTS | 2026-09-19: 39 pass / 8 warn / **1 fail**; 09-18 identical; 09-17: 38/8/**2**. Stable, non-zero failing state. | 3 d | friction |
| E-54 | **Component fabric is two-thirds edgeless** | `.fabric/components/*.yaml` | EXISTS | **338 of 503 cards (67%) have neither `depends_on` nor `depended_by`.** Audit reports the same class as "Fabric: 326/491 cards have no edges". CLAUDE.md instructs agents to run `fw fabric blast-radius` before committing and `fw fabric deps` before modifying a file; for two-thirds of registered components those verbs can return no downstream information. | current | structure |
| E-55 | arc-005 quantified the MCP context tax | `.context/arcs/mcp-slimming.yaml` | EXISTS | Arc `arc-005` (`status: in-progress`, `created: 2026-07-11`, `closed_at: null`, anchor T-2406) states: "273 termlink MCP tool descriptions total ~156KB (~39k tokens) loaded into EVERY agent context each session; worst is 11751 chars, 24 exceed 1000, 94 exceed 600… an agent looped 9x on a rejected `recent_dm` call". | — | cost |
| E-56 | **arc-005 largely DELIVERED — the WARN is bookkeeping, not abandonment** | measured from `tools.rs` | EXISTS | Measured today: **261** description strings, **105,516 chars (~26.4k tokens)**, max **1,546** (was 11,751, **−87%**), **1** over 1000 (was 24), **23** over 600 (was 94). Payload down ~32% from the arc's stated 156KB. The audit's 18/30 "no task commits in 30 days" WARN reflects arc-closure bookkeeping, **not** rotting work. | current | cost / value |
| E-57 | **Off-charter share of the per-session context tax** | measured from `tools.rs` + allowlist | EXISTS | Pairing `name`/`description` in `#[tool(...)]`: **260 tools**, **105,512 desc chars ≈ 26,378 tokens loaded into every agent context every session**. The **28** allowlisted off-charter tools (E-17) account for **13,364 chars ≈ 3,341 tokens = 12.7%** of that payload; their mean description (477 chars) exceeds the overall mean (406). *All 28 allowlist names matched a live tool — no stale allowlist entries.* | current | cost / value |
| E-58 | Arc inventory | `.context/arcs/` | EXISTS | 7 arcs: **5 `in-progress`** (arc-001 parallel-substrate, arc-002 substrate-fitness, arc-005 mcp-slimming, arc-007 comms-loudness, arc-008 audit-remediation), 2 `closed` (arc-003 reliable-comms, arc-004 push-transport). Note arc-004 is recorded **closed** while its capability is measurably dark (E-24). | current | structure |
| E-59 | Guard layer, completed members | `run-guard-layer.sh` (text mode) | EXISTS | At 35 members: **2 FAIL** (`check-installed-binary-drift.sh`, `check-pickup-deferred-freshness.sh`), remainder PASS, 0 ERROR. Supersedes E-41's INCOMPLETE for those members; the `--json` form still never emitted (E-41 stands as a usability observation). | current | friction |
| E-60 | **Guard layer, COMPLETE run** | `run-guard-layer.sh` (text mode) | EXISTS | Completed normally, self-verdict `guard layer: FIRING — 4 guard(s) found something (109 passed, 0 errored)` over **113 members**; process exit **1** = its own FIRING roll-up, **not** a timeout. FAILs: `check-installed-binary-drift.sh`, `check-pickup-deferred-freshness.sh`, `check-receiver-ack-lag.sh`, `cron-drift-firing-fixtures.sh`. Footer: "cargo test --workspace not run — pass --tests to include it". **Supersedes E-41/E-59**: the runner does NOT hang; the `--json` form simply emits nothing if cut off, and 113 members exceed the documented "seconds" budget. | current | friction |
| E-61 | **`agent-chat-arc` receipts: 4 of 5 identities NEVER acked** | `check-receiver-ack-lag.sh` (guard layer) | EXISTS | **FAIL** at threshold 25. On `agent-chat-arc` (5 distinct identity rows): `NEVER-ACKED` for `1da4fd998a04ba8c`, `33df8954b2a9b70d`, `9219671e28054458`, `fd794e5408011572` — each **lag=1533**. Check states its own scope limit: rows keyed by identity **fingerprint**, so shared keypairs make this measure a HOST not an agent (T-2838 item 1). | current | usage |
