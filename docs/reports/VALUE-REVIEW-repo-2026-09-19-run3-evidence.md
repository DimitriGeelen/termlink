# Value Review — Evidence File (whole repo, 2026-09-19, run 3)

**Role:** GATHERER output. Facts only, with citations. No classification, no
recommendations, no verdicts — those are the JUDGE's job (Phase 4) and appear in
the sibling report `VALUE-REVIEW-repo-2026-09-19-run3.md`.

**Snapshot taken:** 2026-09-19T18:01:36Z, at repo HEAD `323d9c8ef`, before this
review ran any command that could alter a counter, log or canary heartbeat.

**Gatherers:** five workers — two Claude Code sub-agents (guard layer; tool
surface) and four TermLink-dispatched workers (ledger; code; runtime; docs). The
orchestrator gathered §A, §F, §G, §H, §I directly.

**Independence note.** Runs 1 and 2 of this five-run series left artifacts in
`docs/reports/`. This run did not read them. One gatherer (docs) independently
encountered `VALUE-REVIEW-repo-2026-09-19-run1-evidence.md` while globbing
`docs/reports/*.md` and cited it once as corroboration for a finding it had
already reached by its own method (§E-2, the `closeable` skill). That is a
corroboration, not a dependency; it is flagged here rather than hidden.

---

## A. Snapshot and baseline

| Item | Value | Citation |
|---|---|---|
| Repo HEAD | `323d9c8ef` | `git log --oneline -1` |
| Repo `VERSION` | 0.11.1944 | `cat VERSION` |
| `git describe --tags` | `v0.11.2-1944-g323d9c8ef` | gatherer-runtime §9 |
| Uncommitted files at snapshot | 52 | `git status --porcelain \| wc -l` |
| Framework | `fw v1.6.29`, mode **vendored** | `fw version` |
| Installed CLI binary | **0.11.1716** (mtime 2026-08-29) | `termlink --version`; `ls -la /root/.local/bin/termlink` |
| Running hub daemon | **0.11.1766** | `termlink fleet doctor --json` |
| Workspace tests | **3,618 passed, 0 failed, 4 ignored** across 24 suites; **`cargo test rc=0`** | `cargo test --workspace`, `.context/working/vr-run3/baseline-cargo-test.txt` |
| Guard layer | 113 members — **109 PASS, 4 FAIL, 0 ERROR**, exit 1 | `scripts/run-guard-layer.sh --json` (§L-1) |
| Cron install | 26 OK, **1 MISSING**, 0 uninstalled-jobs, 0 drift | `scripts/check-cron-install-drift.sh --json` |
| `cargo build --workspace` | Finished, no errors | gatherer-code §7 |
| Dead-code warnings | **0** | `cargo build 2>&1 \| grep -E 'never used\|never read\|dead_code'` |

**Baseline is green.** No failing test, no build error, no dead-code warning.

### A-1. Repo surface by category (lines)

| Category | Lines | Files |
|---|---|---|
| Rust, all `.rs` incl. tests | 180,686 | — |
| Rust, `crates/*/src` only | 168,577 | — |
| `scripts/*.sh` | 39,455 | 190 |
| `tests/*.sh` | 12,704 | 83 |
| `docs/**/*.md` | 69,078 | 393 |
| — of which `docs/reports/*.md` | 51,347 | 306 |
| — of which `docs/operations/*.md` | — | 44 |
| `.claude/commands/*.md` (skills) | 6,814 | 34 |
| `CLAUDE.md` | 3,283 (276,749 bytes) | 1 |
| `.context/cron/*.crontab` | — | 27 |

### A-2. Rust size per crate

| crate | lines |
|---|---|
| termlink-cli/src | 67,954 |
| termlink-mcp/src | 46,891 |
| termlink-hub/src | 23,932 |
| termlink-session/src | 21,120 |
| termlink-bus/src | 5,240 |
| termlink-protocol/src | 3,093 |
| termlink-test-utils/src | 347 |

Largest single files: `termlink-mcp/src/tools.rs` **46,458** (25.7% of all Rust in
the repo; 99.1% of its crate), `termlink-cli/src/commands/channel.rs` **20,435**,
`termlink-cli/src/commands/remote.rs` **11,891**.

---

## B. MEASURED usage — hub RPC audit (the strongest source in this review)

Source: `/var/lib/termlink/rpc-audit.jsonl` (T-1304, always-on, written by the hub
on every authenticated dispatch). 92.7 MB, **920,151 records**.
**Activity window: 2026-08-16 → 2026-09-19 (34 days).**

This is *measured* data, not inferred: the hub records every dispatch itself.

**Trustworthiness caveats, recorded rather than assumed:**
- It records the **hub's JSON-RPC method vocabulary** (`channel.post`,
  `session.discover`, …), NOT MCP tool names. An MCP tool that aggregates
  in-process appears here only as the generic read it performs (gatherer-toolsurface §6).
- `rpc_audit.rs:42-46` deliberately **skips some plumbing methods**, so absolute
  totals understate traffic.
- It is the **local hub only**. Four other fleet hubs are not covered.
- Per the ground rules, a channel cannot report its own failures: this shows what
  the hub *accepted*, not what was dropped before reaching it.

### B-1. Full method distribution, 34 days (every method, none omitted)

| count | method | charter verb |
|---|---|---|
| 387,084 | channel.subscribe | 2 exchange |
| 233,392 | hub.auth | (plumbing) |
| 94,345 | session.discover | 1 discover |
| 53,486 | channel.post | 2 exchange |
| 52,643 | channel.create | 2 exchange |
| 45,351 | channel.list | 2 exchange |
| 30,604 | hub.capabilities | (plumbing) |
| 18,148 | hub.version | (plumbing) |
| 1,555 | channel.receipts | 2 exchange |
| 1,368 | channel.claims_summary | 3 claim (read) |
| 1,032 | hub.governor_status | (observability) |
| 437 | hub.legacy_usage | (observability) |
| 312 | event.emit_to | 2 exchange |
| **94** | **command.execute** | **4 control sessions** |
| 74 | channel.cv_keys | 1 discover |
| **51** | **channel.claim** | **3 claim (write)** |
| 45 | channel.delete | (admin) |
| 34 | channel.sweep | (admin) |
| **24** | **command.inject** | **4 control sessions** |
| 21 | channel.claims | 3 claim (read) |
| **18** | **channel.release** | **3 claim (write)** |
| 10 | channel.trim | (admin) |
| **9** | **channel.transfer_claim** | **3 claim (write)** |
| **9** | **channel.renew** | **3 claim (write)** |
| 6 | termlink.ping | (plumbing) |
| **5** | **agent.find_idle** | **1 discover** |
| 1 | query.status | (plumbing) |
| 1 | hub.bus_state | (plumbing) |

### B-2. Derived per-verb rates over the 34-day window

| Charter verb | Dispatches | Per day |
|---|---|---|
| 2 — exchange durable messages | 540,631 | ~15,900 |
| 1 — discover | 94,424 | ~2,777 |
| 3 — claim work (write: claim+release+renew+transfer) | **87** | **~2.6** |
| 4 — control terminal sessions (execute+inject) | **118** | **~3.5** |

Plumbing (`hub.auth` + `hub.capabilities` + `hub.version`) = 282,144 = **30.7%**
of all dispatches.

### B-3. Topic concentration (same source, 34 days)

132 distinct topics touched; **22 distinct sender fingerprints**.

| count | topic |
|---|---|
| 173,668 | agent-chat-arc |
| 84,100 | agent-presence |
| 73,330 | dm:61e262f085a07585:9219671e28054458 |
| 72,678 | dm:9219671e28054458:d1993c2c3ec44c94 |
| 7,636 | inbox:framework-agent-systemd |
| 7,634 | inbox:termlink-agent |
| 7,633 | inbox:pen-agent-systemd |
| 7,633 | inbox:email-archive |
| 4,745 | health:ring20-fedprobe |
| 683 | framework:pickup |

Those top 4 topics = 403,776 dispatches. Several topics carry exactly 97 touches
and are test/smoke artifacts by name: `dm:design-smoke-test`, `smoke:t2146`,
`agent-send-test-2355299`, `agent-send-test-4176293`, `t-1358-inbox-1777360315`,
`dm:d1993c2c3ec44c94:t2152-orch-real`, `dm:aef-t2904-idprobe:…`.

---

## C. MEASURED non-use — the observability-arc audit logs

CLAUDE.md documents five substrate observability arcs, each built out in 5–6
slices to the same template: `--watch` → `--notify` → `--log <PATH>` → a CLI
`*-history` verb that reads that log → an MCP parity tool for the history verb.

The `--log` file is created **only** when an operator actually runs the watch loop
with `--log`. Its presence or absence is therefore a direct, measurable test of
whether those slices were ever exercised on this host.

| Arc | Slices (tasks) | Documented default log path | Status on this host |
|---|---|---|---|
| find-idle / DISPATCH | T-2078…T-2082 | `~/.termlink/find-idle.log` | **ABSENT** |
| claims / CLAIM | T-2072…T-2077 | `~/.termlink/claims.log` | **ABSENT** |
| queue / RESILIENCE | T-2083…T-2087 | `~/.termlink/queue.log` | **ABSENT** |
| governor / BACKPRESSURE | T-2064…T-2071 | `~/.termlink/governor.log` | **ABSENT** |
| auto-heal | T-1685…T-1687 | `~/.termlink/heal.log` | **ABSENT** |
| rotation (predates the template, T-1671) | T-1671 | `~/.termlink/rotation.log` | EXISTS — **3 lines, last written 2026-05-17** (125 days ago) |

Citation: `for f in find-idle claims queue governor heal rotation; do ls ~/.termlink/$f.log; done`;
full `ls -la ~/.termlink/` in the orchestrator transcript.

**What this does and does not show.** It shows that on this host the `--log`
flag has never been used for five of the six arcs, so the `*-history` CLI verbs
and their MCP twins have never had a data source to read. It does **not** show
this for the other four fleet hosts, which were not inspected. It does not by
itself say the underlying primitives are unused — §B shows `hub.governor_status`
at 1,032 and `channel.claims_summary` at 1,368 dispatches.

### C-1. Corroborating rate data for the primitives these arcs wrap

- `agent.find_idle`: **5 dispatches in 34 days** (§B-1).
- `channel.claim` + `release` + `renew` + `transfer_claim`: **87 in 34 days**.
- Live claim state right now: **0 active claims across all 45 topics**
  (`termlink channel claims-summary --all --json` → `stuck_count:0`,
  every topic `active_count:0`). The only claim history anywhere is **11 expired
  rows on 3 topics**, all demo/probe-named: `substrate-drain-demo` (9),
  `aef-elections-d29583b21ef18b5f` (1), `aef-s8-probe2-2469681` (1).
  **No production traffic topic** (agent-chat-arc, any `dm:*`, framework:pickup,
  channel:learnings) has ever carried a claim. (gatherer-runtime §7)

---

## D. Live runtime state

### D-1. Topics (`termlink channel list --json`): 45 topics, 4,332 retained messages

| Topic | Count | Retention | Distinct senders with a recorded receipt |
|---|---|---|---|
| health:ring20-fedprobe | 1,668 | **forever** | 0 |
| agent-presence | 1,053 | messages:1000 | 0 |
| agent-chat-arc | 1,001 | messages:1000 | 1 |
| channel:learnings | 191 | **forever** | 0 |
| framework:pickup | 126 | messages:5000 | 1 (at `up_to:81`; topic head is 126) |
| dm:61e262f085a07585:9219671e28054458 | 83 | messages:1000 | 0 |
| aef-install-findings | 38 | **forever** | 0 |
| dm:9219671e28054458:d1993c2c3ec44c94 | 32 | messages:1000 | 0 |
| broadcast:global | 16 | messages:1000 | 0 |
| (35 further topics, each ≤16) | | | |

(gatherer-runtime §2, full 45-row table there.)

**Receipt caveat, recorded not assumed:** 0 receipts is normal and expected for
heartbeat, presence and health-probe topics — nobody is supposed to ack them.
The gatherer states this explicitly and does not treat 0 as a defect.

### D-2. Debris topics

- **8** `agent-conv-selftest-<pid>-<ts>-<n>` topics, 2 messages each, from
  selftest runs (gatherer-runtime §2).
- **6** `aef-*` probe/demo topics with **1 message each and `forever` retention**:
  `aef-elect-smoke-d08075953866498b`, `aef-elections-d29583b21ef18b5f`,
  `aef-g1-demo-3610db060173be3b`, `aef-g1-demo-d8578be146c5e088`,
  `aef-g3-demo-fd3dc6a7d1e793da`, `aef-s8-probe2-2469681`; plus
  `aef-s8-probe-2467346` at 0 messages.
- `substrate-drain-demo` (9), `substrate-handoff-demo` (4),
  `substrate-lease-expiry-demo` (1), `t2838-s1-unconsumed` (2),
  `t2838-s2-probe` (5), `broadcast-chat` (0), `sprind-streichliste` (4).

### D-3. `channel:learnings` content quality

191 posts, `forever` retention, single sender. Two adjacent sampled payloads
decode to `{"origin_project":"proj","learning_id":"PL-011","learning":"next",
"task":"T-500","source":"P-001"}` and a near-identical row for `PL-235` with
`"learning":"should be 235"` — placeholder text, not learnings
(gatherer-runtime §3).

### D-4. Presence and wakers

`scripts/agent-listeners-fleet.sh --json`: 3 hubs scanned, 1 failed, **2 LIVE
listeners** (`ring20-dashboard-agent`, `penelope`).
**0 of 2 carry `metadata.pty_session`** — i.e. **0 armed push-wakers fleet-wide**
at observation time. Local `agent-presence` shows **1 distinct sender** in the
retained 1000-message window (`penelope`) (gatherer-runtime §5).

### D-5. Fleet

5 hubs configured, 4 reachable, 1 unreachable (`laptop-141`, "No route to host").
`fleet_versions`: `{"0.11.1411": 2, "0.11.1766": 2, "unknown": 1}`.
`fleet doctor` overall `"ok": false`.

Two hubs (`ring20-dashboard`, `ring20-management`) serve **0.11.1411** — 533
commits behind repo HEAD. (gatherer-runtime §6)

### D-6. Receiver ack frontiers — verified directly by the orchestrator

`bash scripts/check-receiver-ack-lag.sh` (reads the hub's own `channel ack-status`
aggregate; threshold 25), rc=1:

```
agent-chat-arc  (5 distinct identity rows)
  NEVER-ACKED  1da4fd998a04ba8c  lag=1533
  NEVER-ACKED  33df8954b2a9b70d  lag=1533
  NEVER-ACKED  9219671e28054458  lag=1533
  NEVER-ACKED  fd794e5408011572  lag=1533
  BEHIND       d1993c2c3ec44c94  lag=609 (up_to=923)

framework:pickup  (3 distinct identity rows)
  NEVER-ACKED  1da4fd998a04ba8c  lag=126
  NEVER-ACKED  9219671e28054458  lag=126
  BEHIND       d1993c2c3ec44c94  lag=44 (up_to=81)
```

On `agent-chat-arc` — the highest-traffic topic in the fleet (§B-3, 173,668
dispatches) — **4 of 5 identities have never acked a single message** and the
fifth is 609 behind. On `framework:pickup` — the cross-project filing rail that
CLAUDE.md's T-2231 canary exists to protect — **2 of 3 have never acked** and the
third's frontier is at offset 81 against a head of 126, i.e. **45 filings past
the last acknowledgement**.

The check prints its own resolution caveat: rows are keyed by identity
**fingerprint**, so where agents share a keypair the rows collapse and this
measures a *host*, not an agent (T-2838 item 1).

### D-7. Stranded pickup envelope — verified directly

`bash scripts/check-pickup-deferred-freshness.sh`, rc=1:
**1 STRANDED envelope** — `.context/pickup/auto-deferred/P-078-learning.yaml`,
deferred **9 days**, no breadcrumb, therefore un-promotable by construction.
Content begins: `'T-1976 --hub bare-IP class cross-checked across opencode/005-Deco estate:`.
The check's own words: *"A stranded envelope is not waiting, it is lost."*

Related: `G-063` ("framework:pickup topic accumulates pickups with zero receipts
— write-only sink") is in `concerns.yaml` with `status: watching`, **105 days**
old (§G-4).

### D-8. Sessions and queue

44 registered sessions, all state `ready`; `termlink clean --dry-run` → 0 would be
removed. Offline queue: `pending:0, dead_letters:0` — healthy.

---

## E. Structure, references, orphans — the counter-evidence

These are the checks that **failed to find** the problems one might expect. They
are recorded because absence of a finding is itself evidence.

### E-1. Scripts

- 190 `scripts/*.sh`. Basename-grep across `.context/cron/`, `.claude/`, `docs/`,
  `scripts/`, `tests/`, `.github/`, `crates/`, `CLAUDE.md`, excluding self:
  **0 scripts with zero external references.**
- Caveat recorded by the gatherer: this is a substring grep, not a call graph — a
  match may be a comment or doc mention rather than an invocation.

### E-2. Skills

34 `.claude/commands/*.md`. Exactly **one** has zero references outside its own
file and CLAUDE.md's table: **`closeable`** (T-2207). Its backing script
`scripts/list-closeable.sh` exists. Next-lowest are five history skills at 1
reference each (`claims-history`, `find-idle-history`, `governor-history`,
`queue-history`, `substrate-history`). Highest: `be-reachable` 82,
`release` 71, `substrate` 60, `check-arc` 56.

### E-3. Fixture suites

83 `tests/*.sh`; 62 match `*fixtures*.sh`. `run-guard-layer.sh:156` globs
`*fixtures*.sh` **unconditionally**, so all 62 run regardless of whether anything
names them. 8 fixture files have no external textual reference but **are still
executed** by that glob — the gatherer explicitly records that none of the 8 is
provably dead.
Of the 21 non-fixtures files, **7 carry the `guard-layer:` marker** (auto-run) and
**14 do not** (not auto-run; may be invoked by other means, not checked).

### E-4. Duplication

CLAUDE.md documents a "T-2069 convention" of duplicating small pure helpers
between `termlink-cli` and `termlink-mcp`. Measured:
- Exact fn-name matches between the two crates' `src`: **25**, and that list
  includes `new`, `fmt`, `default` and test-fn names — i.e. most are not copy-paste.
- `fn *_mcp` names in `tools.rs`: **71 unique**; of those, **11** have a
  same-base-name twin in the CLI crate.
- `to_json_mcp` occurs 26 times as a small helper redefined inside separate
  functions, which CLAUDE.md's own T-2747 entry warns against counting as 26
  parallel implementations.

**Conclusion recorded as fact:** measured duplication is far smaller than the
convention's prominence in the docs would suggest.

### E-5. Dependencies and dead code

39 unique dependency entries across per-crate manifests; 30 in
`[workspace.dependencies]` (24 external + 6 internal). `cargo-udeps` and
`cargo-machete` are **ABSENT** (not installed; per instruction, not installed to
produce this evidence) — so unused-dependency analysis **was not performed**.
0 dead-code warnings; 5 `allow(dead_code)` sites, each with a stated reason.

---

## F. Doc–code contradictions (verified directly by the orchestrator)

### F-1. Context-budget thresholds: CLAUDE.md vs the code that enforces them

| Source | Critical / BLOCK threshold |
|---|---|
| `CLAUDE.md:2916` | "**170K** urgent→critical (**BLOCK**)" |
| `CLAUDE.md:3052` | "blocks … when context reaches critical level (**>=150K tokens, ~75%**)" |
| `CLAUDE.md:2907-2910` | warn 120K · urgent 150K · handover-immediately 170K |
| `.agentic-framework/agents/context/budget-gate.sh:103-108` | `CONTEXT_WINDOW=300000` (default, **no project override found**) → `TOKEN_WARN=225000`, `TOKEN_URGENT=255000`, `TOKEN_CRITICAL=**285000**` |

`fw config get CONTEXT_WINDOW` returns empty — the 300,000 default is in force.

Live corroboration: `.context/working/.budget-status` at snapshot read
`{"level": "warn", "tokens": 230173}` — 230K classified `warn`, which is
impossible under CLAUDE.md's ladder (230K would be well past its 170K BLOCK) and
exactly right under the code (225K ≤ 230,173 < 255K).

The two CLAUDE.md lines also **contradict each other** (170K vs 150K).

### F-2. Charter sentence — no drift

`bash scripts/check-charter-sentence-drift.sh` → `in-sync — all 3 surfaces carry
the identical canonical sentence`, rc=0. README (378 lines), `docs/CHARTER.md`
and `docs/ARCHITECTURE.md` agree.

### F-3. Documented CLI commands vs the installed binary — no drift

12 documented commands/flags exercised against the installed 0.11.1716 binary
(`agent find-idle`, `channel claims-summary --only-stuck`, `channel cv-keys`,
`fleet governor-status --only-pressured --watch`, `channel claim-transfer`,
`channel queue-status`, `channel renew`, `channel release`, `hub status
--governor`, `fleet doctor`, `agent find-idle-history`, `channel claims-history`):
**all 12 returned rc=0.** No stale-binary doc drift in this sample, despite the
228-commit gap. (gatherer-docs §4)

### F-4. Two commands documented in the gatherer brief do not exist

`termlink agent listeners` → `unrecognized subcommand` (correct verb is the
fleet wrapper script); `termlink list-sessions` → does not exist (correct verb is
`termlink list`). Both were invented in *my* gatherer brief, not taken from repo
docs — recorded so the error is attributable. (gatherer-runtime §5, §10)

### F-5a. `fw audit` did not complete within 240 seconds

`timeout 240 .agentic-framework/bin/fw audit` → **exit 143 (SIGTERM)**. The audit
baseline could therefore not be captured in this review.

This corroborates concern **G-067** — "Pre-push audit T-2067 frontmatter check
spawns one python per task (1937+) — ~80s…" — `status: watching`, **84 days**
old (§G-4). The task corpus has since grown to **2,679** (§G-1), so the per-task
spawn cost has grown ~38% since the concern was filed.

Note the daily cron audit *does* complete (148 dated audit files exist, newest
2026-09-19, §G-3) — so this is a latency finding about the interactive/pre-push
path, not a claim that auditing is broken.

### F-5. `fw metrics` emits a shell error on current data

```
/opt/termlink/.agentic-framework/metrics.sh: line 99: [: 0
0: integer expression expected
```
Vendored script. (gatherer-ledger §10)

---

## G. AEF ledger, governance behaviour, process friction

### G-1. Task ledger

| Metric | Value |
|---|---|
| Active tasks | **246** |
| Completed tasks | 2,433 |
| Active by status | captured 119 · **work-completed 71** · started-work 56 |
| Active by owner | **human 137** · agent 107 · claude-code 2 |
| Active by horizon | now 151 · next 53 · later 42 |
| Median age of active tasks | **42 days** |
| Older than 30 / 60 / 90 / 180 days | **181 / 111 / 95 / 1** |
| `owner: human` active tasks older than 30 days | **115 of 137 (83.9%)** |

Oldest: T-212 "Create Homebrew tap for TermLink distribution", **182 days**,
`owner: human`, `started-work`.

**71 active tasks carry `status: work-completed`** — work declared done but the
task has not left `active/`. CLAUDE.md documents two distinct mechanisms that
produce exactly this state (T-193 partial-complete by design, and the T-2833
finalize-latch defect); this evidence file does not attribute the 71 between them.

### G-2. Gate bypasses

`.context/working/.gate-bypass-log.yaml` (snapshotted before this review ran):
**482 entries**, 2026-04-12 → 2026-09-18.

| count | gate |
|---|---|
| 164 (34.0%) | check-active-task focus-drift |
| 149 | check_human_sovereignty |
| 94 | partial_complete_recheck |
| 26 | check_acceptance_criteria |
| 10 | owner_change |
| 8 each | run_verification_commands · check_rca_for_bugfix · check_inception_decision |
| 7 | human-ac-self-validate |
| 4 | create-task.sh |

Top stated reasons: "Phase A batch close — agent-evidenced" 93 · "Inception
decision: GO" 82 · "Completed via Watchtower UI (human action)" 51.

### G-3. Audit findings that never close

148 dated daily audits, 2026-03-09 → 2026-09-19, with **46 calendar days missing**
across 18 gap intervals (longest 5 days; most recent gap 2026-09-10→09-16).
Newest audit `2026-09-19.yaml`: `{pass: 39, warn: 8, fail: 1}`.

Findings firing on **10 of the last 10 audit runs** (never closing):

| Days | Finding |
|---|---|
| 10/10 | Arc `mcp-slimming` has no task commits in the last 30 days |
| 10/10 | free driver F-ORCH: `retire_when` condition appears met — review whether to retire |
| 10/10 | **77 of 158 GO-recorded completed inceptions have GO recorded but `related_tasks` empty, nobody back-references, no `unlocks_inception_decision`** |
| 10/10 | Fabric: 10 cards point at files no watch pattern covers |
| 10/10 | 3 further checks "NOT EVALUATED: candidate set empty" |
| 7/10 | `cron(substrate-smoke-canary)`: USER-field syntax but no install in `/etc/cron.d` |
| 6/10 | Cron drift: `.context/cron/agentic-audit.crontab` differs from deployed `/etc/cron.d/agentic-audit-termlink` |
| 6/10 | Fabric: **326/491 cards have no edges** |
| 4/10 | Arc `arc-substrate-fitness` has no task commits in the last 30 days |

### G-4. Concerns register

52 entries: **27 `watching`** · 12 closed · 6 resolved · 5 mitigated · 2 decided-build.
By severity: 30 medium · **15 high** · 6 low · 1 critical.
Oldest `watching`: **G-008 at 157 days** ("64 tasks stuck in partial-complete
state"). Four `watching` entries are `high`: G-087, G-086 (both 59 days), G-092,
G-091 (both 1 day), plus G-083 (no parseable date).

### G-5. Learnings

389 entries in `.context/project/learnings.yaml`, highest id **PL-375** — count
and highest id disagree, not further investigated.

### G-6. Handover churn

| Metric | Value |
|---|---|
| Handover `.md` files in `.context/handovers/` | **1,706** |
| Commits in repo history | 7,172 |
| Commits whose subject contains "Session handover" | **2,127 = 29.7%** |
| Git traceability (commits with a task ref) | 7,123 / 7,172 = 99% |

Observed cadence at snapshot — four handover commits in six minutes, all under
T-2723: `S-2026-0919-1954`, `-1956`, `-1958`, `-2000`
(`git log --oneline -6`). `.context/working/.compact-log` shows the matching
auto-generation events at 17:54:20Z, 17:56:10Z, 17:58:06Z, 18:00:26Z.

**All three most-recent handovers carry an unfilled template placeholder** in
their "Open Questions / Blockers" section:
`[TODO: questions left open and anything blocking the next session. Unfilled by
the generator — see above.]` — and identical "Where We Are" and "Suggested First
Action" content, differing only in the prior-handover timestamp referenced.

### G-7. Arcs

7 arc files: 2 closed, **5 `in-progress`**. Days since the arc file's last commit:

| Arc | Status | Last commit | Days stale | Task refs |
|---|---|---|---|---|
| arc-001 parallel-substrate | in-progress | 2026-06-13 | **98** | 17 |
| arc-002 substrate-fitness | in-progress | 2026-06-23 | **88** | 12 |
| arc-005 mcp-slimming | in-progress | 2026-07-11 | **70** | 4 |
| arc-007 comms-loudness | in-progress | 2026-07-21 | **60** | 8 |
| arc-008 audit-remediation | in-progress | 2026-09-09 | 10 | 3 |
| arc-004 push-transport | closed | 2026-07-05 | — | 33 |
| arc-003 reliable-comms | closed | 2026-07-02 | — | 6 |

---

## H. Stranded work — git worktrees

`.claude/worktrees/` is gitignored (`.gitignore:143`); **0 tracked files**. It
holds **5 directories, 44 GB**, 4 of which are live `git worktree` registrations
on unmerged branches.

| Worktree branch | Commits ahead of `main` | Last modified | Days stale | Size |
|---|---|---|---|---|
| `worktree-charter-review-2026-0814` | **147** | 2026-08-28 | 22 | 43 GB |
| `worktree-governance-canary-signal` | 8 | 2026-08-19 | 31 | 657 MB |
| `worktree-T-2398-findings` | 2 | 2026-07-10 | 71 | 97 MB |
| `worktree-T-2209-history-skills` | 2 | 2026-06-13 | 98 | 83 MB |
| `t2687-pickup-failopen` (dir only) | — merged into main — | 2026-08-26 | 24 | 4 KB |

**Total unmerged: 159 commits.** Sample subjects from the 147-commit branch:
`T-2848: triage the 75 finished-and-waiting tasks into ~22 decisions`,
`T-2848: audit + metrics + VERSION residue from the push-time audit`.
From the 8-commit branch: `T-2692: finalize the canary-signal arc — T-2691
complete, T-2690/T-2692 partial-complete pending host observation`.

Also present and gitignored: `.agentic-framework.rollback/` (25 MB, 1,380 `.md`
files, 0 tracked). Working tree `target/` is **121 GB**.

CLAUDE.md's own T-2800 section documents that per-branch task-ID allocation across
worktrees is the mechanism behind cross-branch ID collisions and duplicated work.

---

## I. The tool surface and the charter

### I-1. Census

| Metric | Value | Citation |
|---|---|---|
| Total MCP tools (`termlink help --json`) | **260** | gatherer-toolsurface |
| Deprecated | 46 | same |
| Live | **214** | same |
| Categories | 29 | same |
| Categories that are **100% deprecated yet still compiled** | 3 — `agent_engagement_metrics` (8), `channel_poll` (4), `agent_poll` (3) = **15 tools** | same |

### I-2. Charter-drift canary result

`bash scripts/check-charter-drift-freshness.sh --json`: 214 live tools scanned,
**`off_charter_total: 28`**, `acknowledged_count: 28`, **firing: 0** (all 28 sit
in the git-tracked allowlist). Groups: `agent_rankings` 5 · `agent_stats` 10 ·
`agent_thread_health` 8 · `channel_engagement` 5.

The canary prints its own scope disclaimer: it detects known off-charter *shapes*,
it is "not a full charter-traceability audit of every tool".

### I-3. Caller evidence for the 28

**0 of 28 have a call site** in any skill (`.claude/commands/`), operator script,
integration test, or the CLI crate. Every hit under `scripts/` or `tests/` is the
charter-drift checker's own prose comment or its canned JSON fixture — i.e. the
tool name appears as *input to the detector that flags it*. **8 of 28 have zero
references anywhere** outside `tools.rs`, the allowlist and fixtures.

This independently reproduces the finding T-2548 recorded on 2026-08-08, 42 days
earlier.

### I-4. T-2548 — the decision that was made and not executed

Verified directly by the orchestrator from
`.tasks/completed/T-2548-charter-non-goal-4-conversation-analytic.md`:

| Field | Value |
|---|---|
| `status` | `work-completed` |
| `workflow_type` | `inception` |
| `owner` | `human` |
| `created` | 2026-08-08T19:34:48Z |
| `date_finished` | **2026-08-20T17:54:28Z** (30 days before this review) |

Recorded Decision block, verbatim:

> **Decision**: GO
> **Rationale**: Recommendation: GO (subtract), **gated on IW-1 external-consumer
> check.** … GO to subtract restores the charter and removes advertised-but-uncalled
> surface. Human owns the subtract-vs-grandfather product decision; **the removal
> must be gated on a cross-project external-consumer check (IW-1) the T-559
> boundary blocks here.**
> … "Full family enumeration + live/deprecated split is GO-build scope."

**Two facts follow, both verified:**
1. The GO-build task was never created. 8 tasks reference T-2548; all are
   reviews/meta (T-2470, T-2716, T-2690, T-2680, T-2549, T-2683, T-2678, and this
   review T-2971). T-2548's own scope fence says "OUT of scope: the actual removal
   (a GO build task)". (gatherer-toolsurface §3)
2. **The IW-1 external-consumer check was never discharged.** The decision names
   it as a precondition and states the T-559 project boundary blocked it from that
   session. No evidence of it having been run since was found.

### I-5. The allowlist still describes the question as open

`.context/checks/charter-drift-allowlist` header (line 20) still says
"T-2548, `started-work`, `owner: human`" and all 28 entries still read
`# T-2548 pending: …` — 30 days after T-2548 closed. The file's own stated
contract is "a ledger of an open question, not a permanent exemption … should end
up empty or re-justified."

One in-file NOTE splits the set: `channel_engagement` (5 tools — mentions /
search / digest) "have a plausible coordination reading (arguably charter verb
2) … flagged for the human to split rather than subtract wholesale." The other
23 carry no such caveat.

### I-6. MCP/CLI parity census

`bash scripts/check-mcp-parity-census.sh --json`: 260 tools · **24 asserted
(9.2%)** · 236 acknowledged · 0 unexamined. Its own scope note: it does **not**
verify that an asserted pair actually agrees.

### I-7. No MCP tool-invocation telemetry exists

`crates/termlink-mcp/src/server.rs` (412 lines) and `lib.rs` (21 lines): zero
matches for tool_call / invocation / telemetry / metric / counter / tracing. The
only `AtomicU64` in the crate is `XFER_NONCE`, a file-transfer nonce. No
`rpc_audit::record` call anywhere in the crate.

The hub-side audit (§B) **exists but structurally cannot attribute an MCP tool** —
its vocabulary is the 31 hub RPC methods in `router.rs`; no `termlink_*` name
appears in it, and analytics tools aggregate in-process so the hub sees only a
generic read.

**Therefore: per-MCP-tool usage is UNMEASURABLE with current instrumentation.**
Per the ground rules this is recorded as a data gap, never as evidence of zero use.

---

## J. Non-use diagnosis inputs

For each low/no-use item, the evidence for each competing reading (A broken ·
B never wired · C undiscoverable · D unmeasured · E not wanted) is recorded
side by side. **No reading is selected here** — that is the JUDGE's job.

### J-1. The 28 off-charter analytics tools

| Reading | Evidence found |
|---|---|
| A BROKEN | None. They compile; workspace tests pass; charter-drift canary parses them as live. No error, healing event or issue on that path was found. |
| B NEVER WIRED | Partial. They are registered and callable via MCP (`is_deprecated()` false), so they *are* wired to the MCP surface. But 0 of 28 has a first-party caller (§I-3). |
| C UNDISCOVERABLE | Against: they appear in `termlink help --json` and are listed in the allowlist. No skill, doc or prompt surfaces them to an operator. |
| D UNMEASURED | **Strong.** §I-7 — per-MCP-tool invocation telemetry does not exist. An external MCP client calling these would leave no trace. |
| E NOT WANTED | **Strong and positive:** a recorded human-owned inception decision, T-2548, 2026-08-20: **GO (subtract)**, on the stated ground that they trace to none of the four charter verbs and violate non-goal #4. |
| Intent evidence (rules out E until explained) | The same decision explicitly *preserves* 3 sibling tools as charter-tracing (`agent_search_thread`, `agent_thread_path`, `agent_recent_window`), and flags `channel_engagement`'s 5 as "plausible coordination reading … for the human to split rather than subtract wholesale". The decision is **conditional on IW-1**, undischarged (§I-4). |

### J-2. The five observability-arc history/log/notify slices

| Reading | Evidence found |
|---|---|
| A BROKEN | Against: `agent find-idle-history` and `channel claims-history` both returned rc=0 on `--help` against the installed binary (§F-3). Not exercised with data. |
| B NEVER WIRED | Against: CLI verbs, MCP twins and skills all exist. |
| C UNDISCOVERABLE | **Mixed.** Documented at length in CLAUDE.md. But the five `*-history` skills sit at the very bottom of the skill cross-reference table — **1 reference each** (§E-2). |
| D UNMEASURED | Against, unusually: the `--log` file's existence *is* the measurement, and 5 of 6 are absent (§C). |
| E NOT WANTED | No positive recorded decision found. No arc closure, no changed non-goal, no supersession. |
| Intent evidence | Strong: built deliberately across 5 arcs × 5–6 slices; each has a CLAUDE.md section; arc-001 (17 task refs) is still `in-progress`. |

### J-3. `closeable` skill

| Reading | Evidence found |
|---|---|
| A BROKEN | Not tested (invoking a skill is not a shell command; not exercised). |
| B NEVER WIRED | Against: `scripts/list-closeable.sh` exists and the skill file exists. |
| C UNDISCOVERABLE | **Strong.** 0 references in CLAUDE.md, 0 in `docs/`, 0 in `scripts/` outside itself — the only skill of 34 with zero (§E-2). |
| D UNMEASURED | **Yes** — no skill-invocation telemetry exists anywhere. |
| E NOT WANTED | No positive evidence. |

### J-4. Claim-work verb (charter verb 3)

| Reading | Evidence found |
|---|---|
| A BROKEN | Against: `substrate-smoke.sh` drives the full claim→transfer→release lifecycle and is wired to a daily canary (T-2696); 87 successful claim-family dispatches in 34 days (§B-1). |
| B NEVER WIRED | Against: CLI, MCP, 5 skills (`/claim`, `/release`, `/renew`, `/claim-transfer`, `/claims`) all exist. |
| C UNDISCOVERABLE | Against: heavily documented (`docs/operations/substrate-claim-primitive.md`, `substrate-orchestrator-recipe.md`, CLAUDE.md). |
| D UNMEASURED | Against: §B-1 measures it directly. |
| E NOT WANTED | No positive decision found; it is a **charter verb**, so intent evidence is maximal. |
| Observed | ~2.6 claim-writes/day; **0 active claims now**; **all 11 historical claim rows are on demo/probe topics**; no production topic has ever carried a claim (§C-1). |

### J-5. The 15 tools in three 100%-deprecated categories

`agent_engagement_metrics` (8) · `channel_poll` (4) · `agent_poll` (3). Every
tool in each category is marked deprecated by the binary's own `is_deprecated()`,
yet all 15 are still compiled and registered.

| Reading | Evidence found |
|---|---|
| A BROKEN | None found. |
| B NEVER WIRED | They are registered and callable. |
| C UNDISCOVERABLE | They carry a deprecation marker, which is the opposite of surfacing them. |
| D UNMEASURED | **Yes** — §I-7, no per-tool telemetry. |
| E NOT WANTED | **Strong and positive:** deprecation is itself a recorded intent-to-remove, applied by the project to 46 tools. CLAUDE.md documents the `remote_inbox_*` (T-1166) deprecate-then-remove convention and `docs/operations/p4-surface-reduction.md`. |
| Intent evidence against E | None found — nothing references them; no task proposes reviving them. |
| Not established | **How long they have been deprecated**, and whether the T-1166 convention specifies a removal window. Not gathered. |

### J-6. `substrate-smoke` canary — shipped but never scheduled

`check-cron-install-drift.sh` reports its crontab MISSING from `/etc/cron.d`
(§L-1). The daily audit has flagged it on **7 of the last 10 runs** (§G-3).
CLAUDE.md T-2696 documents it as the prover of the claim-work *composition*.

| Reading | Evidence found |
|---|---|
| A BROKEN | Not tested here (not executed). |
| B **NEVER WIRED** | **Direct and strong.** The crontab exists in `.context/cron/` and is absent from `/etc/cron.d` — the exact "shipped but dark" condition CLAUDE.md's T-2561 section says this check exists to catch. |
| C UNDISCOVERABLE | Against — documented at length. |
| D UNMEASURED | n/a. |
| E NOT WANTED | No positive evidence. |
| Intent evidence | Maximal: it was built, documented, and a check exists specifically to detect it being unscheduled — and that check has been reporting it for at least 10 days. |

### J-7. `health:ring20-fedprobe`

1,668 messages, **`forever` retention**, single sender, 0 receipts, payload
`FED-PROBE-RT fedprobe-<ts>-<rand>` from project `proxmox-ring20-management`.
CLAUDE.md's T-2562 section documents a canary for exactly this shape (a
`forever` topic used as archival storage) with a **default ceiling of 50,000** —
1,668 is far below it, so that canary is silent by design. Charter non-goal #2
says topics are "retention-bounded append logs sized for coordination, not
archival". No recorded decision on this topic was found either way.

---

## K. Data availability map (verified against the live repo)

| Source | Status | Location | Window | Trustworthiness |
|---|---|---|---|---|
| Hub RPC audit | **EXISTS** | `/var/lib/termlink/rpc-audit.jsonl` | 2026-08-16→09-19 (34d) | **Measured.** Local hub only; hub-method vocabulary, not MCP tools; skips some plumbing |
| Topic/retention state | **EXISTS** | `termlink channel list --json` | current | Measured, point-in-time |
| Receipts per topic | **PARTIAL** | `termlink channel receipts` | current | LWW per (topic,identity); no subscriber enumeration verb exists |
| Presence / heartbeats | **EXISTS** | `agent-presence` topic | rolling 1000 msgs | Measured; window truncates history |
| Claims state | **EXISTS** | `channel claims-summary --all` | current | Measured |
| Offline queue | **EXISTS** | `~/.termlink/outbound.sqlite` | current | Measured |
| Observability `--log` trails | **5 of 6 ABSENT** | `~/.termlink/*.log` | — | Absence is itself the measurement (§C) |
| **MCP per-tool invocation telemetry** | **ABSENT** | — | — | §I-7. Blocks all per-tool usage judgement |
| **Skill / slash-command invocation telemetry** | **ABSENT** | — | — | Blocks §J-3 |
| **BVP realization log** | **ABSENT** | `.context/audits/bvp-realization.jsonl` does not exist | — | Blocks "did shipped arcs deliver?" entirely |
| Task ledger | **EXISTS** | `.tasks/` | 2026-03→09 | Observed |
| Gate bypass log | **EXISTS** | `.context/working/.gate-bypass-log.yaml` | 2026-04-12→09-18 | Observed; snapshotted pre-review |
| Daily audits | **EXISTS, gappy** | `.context/audits/*.yaml` | 2026-03-09→09-19, **46 days missing** | Observed |
| Concerns / learnings | **EXISTS** | `.context/project/*.yaml` | — | Observed |
| Handovers | **EXISTS** | `.context/handovers/` (1,706) | — | Observed; latest 3 have unfilled placeholders |
| Cron canary logs + heartbeats | **EXISTS** | `.context/working/.*-canary.log`/`.heartbeat` | — | Observed; snapshotted pre-review (§L) |
| Git history | **EXISTS** | 7,172 commits | 2026-03→09 | Observed |
| `fw metrics` | **PARTIAL** | — | — | Emits a shell error (§F-5); no throughput, cost or outcome metrics |
| `.session-metrics.yaml` | **PARTIAL** | overwritten per session | current session only | **Not a history** |
| Unused-dependency analysis | **NOT PERFORMED** | cargo-udeps / cargo-machete not installed | — | — |
| Out-of-band observer of the bus | **ABSENT** | — | — | Ground-rule gap: TermLink's reliability is judged only from TermLink's own data |
| External-consumer / download signal | **ABSENT** | — | — | Blocks DELETE check #5 for anything on the published install path; also the undischarged T-2548 IW-1 gate |
| Execution traces by workflow node | **ABSENT** | no `fw workflow run` in this repo | — | — |

---

## L. Guard-layer state

Canary log sizes, snapshotted **before** this review ran anything
(`.context/working/vr-run3/canary-log-sizes.txt`, 2026-09-19T18:01Z). Under the
project's "empty log = healthy" convention:

**Non-empty (firing) — 7:**

| bytes | canary log |
|---|---|
| 80,430 | `.substrate-preflight-canary.log` |
| 63,993 | `.framework-pickup-canary.log` |
| 33,097 | `.waker-liveness-canary.log` |
| 8,197 | `.stuck-claims-canary.log` |
| 6,307 | `.hook-counter-integrity-canary.log` |
| 4,216 | `.stale-waker-code-canary.log` |
| 418 | `.fleet-doorbell-mail-canary.log` |

**Empty (healthy) — 14:** charter-drift, charter-sentence-drift, dead-letter,
fleet-binary, fleet-capability, forever-archival, frozen-husk,
preflight-doc-set-drift, release-mirror, session-control, task-finalization,
topic-growth, unconfirmed-delivery, woken-but-silent.

**Heartbeats:** 33 `.heartbeat` files exist. Three are stale relative to a daily
cadence: `.error-swallowing-canary.heartbeat` (2026-08-29, **21 days**),
`.unbounded-rpc-call-canary.heartbeat` (2026-08-29, **21 days**),
`.task-template-idioms-check.heartbeat` (2026-09-01, **18 days**).

**Corroborating, from the audit register (§G-3):** `cron(substrate-smoke-canary)`
has no install in `/etc/cron.d` (7 of last 10 audits) and `agentic-audit.crontab`
differs from its deployed copy (6 of last 10).

### L-1. Guard-layer runner result

`bash scripts/run-guard-layer.sh --json`, run by the orchestrator and
independently by the guard-layer gatherer — **identical results**:

```
total 113 · passed 109 · fired 4 · errored 0 · unclassified 75 · exit_code 1
```

Member kinds: 62 fixture-suite · 39 static-check · 12 suite. The run exceeded a
600s foreground timeout and completed in background.

**The 4 FAILs:**

| Member | Finding |
|---|---|
| `check-installed-binary-drift.sh` | **Three different binaries on one host**: `/root/.cargo/bin` = 0.11.1766 · `/root/.local/bin` + `/usr/local/bin` = 0.11.1716 · build artifact = **0.11.1940** |
| `check-receiver-ack-lag.sh` | `agent-chat-arc`: **4 identities NEVER-ACKED, lag = 1532** (threshold 25); 1 further identity BEHIND at lag 608 |
| `check-pickup-deferred-freshness.sh` | **1 STRANDED envelope** — `P-078-learning.yaml`, deferred ~9 days |
| `cron-drift-firing-fixtures.sh` | 12 assertions pass, 1 fails — **the failing assertion is "the real tree passes the firing check"**, i.e. a fixture coupled to live host state, failing because of the missing `substrate-smoke-canary` crontab above |

### L-2. Canary classification vs. log contents

`scripts/canary-status.sh`: **30 canaries — 27 healthy · 1 FIRING
(waker-liveness) · 0 stale · 2 not-scheduled.**

But §L's snapshot shows **7 non-empty logs**, and canary-status reports only 1 as
FIRING. The reason is mechanical and is recorded here as a fact about the
classifier, not a verdict: **FIRING means "a log entry is newer than the
heartbeat"**, so a finding that is merely *old* reads green.

**6 canaries are classed HEALTHY while carrying a non-empty log:**
fleet-doorbell-mail · framework-pickup · hook-counter-integrity ·
stale-waker-code · stuck-claims · substrate-preflight.

| Log | Entries | Last append | Live re-run today |
|---|---|---|---|
| substrate-preflight | **74** (74 distinct days, 2026-07-06 → 2026-09-19) | 2026-09-19 | — |
| framework-pickup | 63 | 2026-09-18 | **healthy, rc=0** |
| waker-liveness | 36 | 2026-09-19 07:53 | **still FIRING** |
| stuck-claims | 7 | 2026-08-16 (34d ago) | — |
| hook-counter-integrity | 8 | 2026-09-16 | — |
| stale-waker-code | 5 | 2026-08-13 (37d ago) | **healthy, rc=0** |
| fleet-doorbell-mail | **1** | 2026-08-10 (40d ago) | — |

**`substrate-preflight` has appended on every one of 75 consecutive days and
never once with zero warnings**: 51× "5 pass 1 warn", 14× "4 pass 2 warn", 9×
"3 pass 3 warn". WARN causes: `binary` 71× · `hub-binary` 26× · `be-reachable` 9×.

**`stuck-claims` fires on 11 topics that all have `active=0`** — e.g.
`substrate-drain-demo active=0 expired=81`. (CLAUDE.md's T-2709 note documents
narrowing this exact monotonic-latch predicate; the log entries predate or
survive that change — last append 2026-08-16.)

**`fleet-doorbell-mail`'s single entry:** `DRIFT total=4 pass=1` — hubs .122 and
.121 `verdict=setup-fail`, only .107 passes.

**Age is largely unknowable.** Only `substrate-preflight` uses the
`=== <ts> ===` framing. The other six logs carry **zero timestamps**, and
`.context/working/*` is gitignored (`.gitignore:139`), so there is no VCS
fallback — file mtime is the only age signal.

### L-3. Meta-canary and scheduling

- **`canary-aliveness` has no heartbeat file at all**; its log last moved
  **2026-06-06 — 105 days ago**. The meta-canary that watches whether canaries
  are still firing is itself unobserved.
- **2 canaries not scheduled**: `error-swallowing` (heartbeat 508h old) and
  `unbounded-rpc-call` (511h). A third, `task-template-idioms`, is 429h stale.
- **Malformed name:** `.stranded-finalized-canary.log.heartbeat` (double suffix);
  it has no matching log and the canary does not appear in `canary-status` output.
- ~14 heartbeats carry mtimes of 2026-09-19 18:55–20:11 — **stamped by this
  review's own guard-layer run, not by cron** (self-pollution, §M).
- 31 heartbeat files vs 30 canaries vs 21 logs — the three sets do not correspond.

### L-4. Marker coverage

`scripts/check-*.sh`: **66 total** · **39 carry `# guard-layer: source`** (59%) ·
42 carry any marker · **24 carry none** (mostly `*-freshness.sh` runtime canaries,
which belong to cron by design).

**Scale of the guard layer in CLAUDE.md:** lines 19–1948 (**1,930 lines,
136,213 bytes**) are the canary + static-check catalogue = **49.2% of CLAUDE.md
by bytes**. Composition of the whole file:

| Region | Lines | Bytes |
|---|---|---|
| Guard-layer catalogue (19–1948) | 1,930 | 136,213 |
| Project rules + substrate/skills table (1949–2460) | 512 | 96,630 |
| Framework-managed template (2461–3283) | 823 | 43,407 |
| **Total** | **3,283** | **276,749** |

At ~3.7 bytes/token this is **≈74,800 tokens auto-loaded into every session**.
Against the code's actual `CONTEXT_WINDOW=300000`, that is **~25% of every
session's budget consumed before any work begins**; against CLAUDE.md's own
(stale, §F-1) documented 170K BLOCK threshold it would be ~44%.

**`fw upgrade` boundary:** `## Core Principle` is at line **2461**; 823 lines sit
below it in the framework-managed region. Heading-by-heading comparison against
`.agentic-framework.rollback/lib/templates/claude-project.md` shows an **exact
1:1 match** in headings and order — **no project-specific content is currently at
risk** below the boundary. (gatherer-docs §1)

---

## M. Items exercised during this review (logged per the Phase-3 rule)

All read-only. None Tier-0. All after the Phase-0 snapshot.

- `cargo test --workspace`, `cargo build --workspace`
- `scripts/check-charter-sentence-drift.sh`, `check-charter-drift-freshness.sh`,
  `check-mcp-parity-census.sh`, `check-cron-install-drift.sh`,
  `run-guard-layer.sh`, `canary-status.sh`, `agent-listeners-fleet.sh`
- `termlink` read verbs: `--version`, `hub status`, `channel list`,
  `channel info`, `channel members`, `channel receipts`, `channel subscribe`
  (read), `channel claims-summary`, `channel queue-status`, `fleet doctor`,
  `list`, `clean --dry-run`, and `--help` on 12 documented commands
- `fw version`, `fw metrics`, `fw metrics api-usage`, `fw work-on T-2971`
- `git` read commands; `grep`/`awk` over the RPC audit

**Known self-pollution:** `fw work-on T-2971` at 18:01Z touched focus state;
running the charter-drift and mcp-parity checks refreshed their canary
heartbeats (`.charter-drift-canary.heartbeat` and
`.mcp-parity-census-canary.heartbeat` both read 20:02). Canary **log** sizes in
§L are from the 18:01Z pre-review snapshot and are unaffected. The four
TermLink-dispatched gatherers registered four new sessions, which is why §D-6's
count of 44 includes `vr3-*` entries.

---

## N. Not gathered

- **Unused-dependency analysis** — tooling absent, not installed by instruction.
- **The other four fleet hosts** — §C's conclusion about `--log` files is
  single-host.
- **Per-MCP-tool and per-skill usage** — no instrumentation exists (§I-7).
- **External consumers / downloads** — `{{EXTERNAL_DATA}} = none`; no network
  queries made. This is also the undischarged T-2548 IW-1 gate.
- **The 147-commit `charter-review-2026-0814` branch's contents** — only its
  commit subjects were read, not its diff.
- **Whether the 71 active `work-completed` tasks are T-193 partial-complete
  (by design) or T-2833 finalize-latch (a defect)** — not attributed.
- **Complexity metrics** (cyclomatic/cognitive) — no tool installed; size and
  churn used as proxies.
- **Change coupling / fix-revert ratio per area** — not computed.
