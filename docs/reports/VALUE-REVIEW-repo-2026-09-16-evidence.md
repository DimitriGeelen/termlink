# Value Review — EVIDENCE FILE (GATHERER output)

**Scope:** whole repo (PROVISIONAL — IW-1 unconfirmed)
**Task:** T-2971 · **Role:** GATHERER (facts only; no classification, no recommendations)
**Snapshot taken:** 2026-09-16T21:08:03Z, before the review generated any of its own data.
**Status:** Phase 0 complete, Phase 3 partial. Phase 1 [ASK] not yet answered.

> Every row cites a command or path. Rows marked UNVERIFIED are not evidence.
> Nothing here is a verdict. Non-use readings (A–E) are collected, not resolved.

---

## 0. Baseline

| Check | Result | Source |
|---|---|---|
| `cargo test --workspace` | **2962 passed, 0 failed, 4 ignored** across 10 suites | baseline run, this session |
| `fw audit` (structure) | Pass 38 · Warn 8 · Fail 2 | audit 2026-09-16, both FAILs worktree/host-scoped (`/etc/cron.d`), not in any committed ref |
| `bash scripts/run-guard-layer.sh` | **PENDING** — exceeded 300s, still running at write time | not yet evidence |
| `termlink --version` | 0.11.1766 | CLI |
| `fw --version` | v1.6.29 (vendored) | CLI |

The guard-layer row is deliberately left PENDING rather than filled from a previous
session's memory. A baseline reconstructed from recollection is not a baseline.

---

## 1. Project shape (structure)

| Measure | Value | Source |
|---|---|---|
| Product Rust LOC | **180,686** across 7 crates | `find crates -name '*.rs' \| xargs wc -l` |
| — termlink-cli | 72,790 (40%) | per-crate count |
| — termlink-mcp | 50,828 (28%) | per-crate count |
| — termlink-hub | 24,550 | per-crate count |
| — termlink-session | 23,838 | per-crate count |
| — termlink-bus | 5,240 | per-crate count |
| — termlink-protocol | 3,093 | per-crate count |
| — termlink-test-utils | 347 | per-crate count |
| Shell in `scripts/` | 39,689 LOC, 207 tracked files | `wc -l` |
| Shell in `tests/` | 18,452 LOC, 85 files | `wc -l` |
| Tracked files, total | **11,818** | `git ls-files \| wc -l` |
| — `.context/` | 5,094 (43%) | tracked count by dir |
| — `.tasks/` | 2,680 (23%) | tracked count by dir |
| — `.agentic-framework/` (vendored) | 2,611 (22%) | tracked count by dir |
| — `crates/` | 131 (1.1%) | tracked count by dir |
| — `scripts/` / `tests/` / `docs/` | 207 / 116 / 394 | tracked count by dir |

**Two crates are 68% of the product.** `termlink-cli` + `termlink-mcp` = 123,618 of
180,686 LOC. (Cost datum. Not a value datum.)

**Untracked mass on the host** (costs disk, not repo):
- `.claude/` — **44 GB** on disk, 36 files tracked
- `.agentic-framework.rollback/` — 25 MB, **0 files tracked** (leftover rollback copy)

---

## 2. Activity (the clock for "unused" — activity, not calendar)

| Window | Measure | Value |
|---|---|---|
| all time | commits | 7,149 |
| last 90 days | commits | 2,019 |
| last 90 days | commits touching `crates/` | **247** |
| last 90 days | commits touching `scripts/` + `tests/` | **219** |
| last 90 days | commits touching `.tasks/` + `.context/` | **1,929** |

**A raw governance-vs-product churn ratio is misleading and is recorded here so it is
not used naively.** In the last 500 commits `crates/` appears 24 times against
`.context/` 1,543 — a ~64:1 ratio. Decomposing it removes most of the signal:

- 84% of `.context/` churn is **machine-written session bookkeeping**:
  `working/` 654, `handovers/` 349, `episodic/` 175, `audits/` 124 = 1,302 of 1,543.
- Over the wider 90-day window the product is actively worked: 247 commits.

The 500-commit sample was governance-heavy because recent sessions were audit work.
Recorded as a **measurement hazard**, not as a finding.

---

## 3. Guard layer and acknowledged debt

### 3a. Canary logs — the convention is "empty log = healthy" (CLAUDE.md, ×21)

Snapshot of `.context/working/.*-canary.log`, sizes in bytes:

| Bytes | Canary |
|---|---|
| 76,252 | `.substrate-preflight-canary.log` |
| 61,281 | `.framework-pickup-canary.log` |
| 31,619 | `.waker-liveness-canary.log` |
| 8,197 | `.stuck-claims-canary.log` |
| 6,307 | `.hook-counter-integrity-canary.log` |
| 4,216 | `.stale-waker-code-canary.log` |
| 418 | `.fleet-doorbell-mail-canary.log` |
| 0 | the other **14** canaries |

**7 of 21 canary logs are non-empty.** Under the documented convention that is
7 canaries reporting findings. T-2685 (CLAUDE.md) states the compounding property
directly: once a log is non-empty, a later genuine finding appends to an
already-dirty file and *changes nothing an operator can see* — "not merely noisy,
but deaf until someone truncates it by hand."

NOT YET ESTABLISHED (do not infer): whether each non-empty log holds real findings
or accumulated tooling noise; how long each has been non-empty; whether any operator
reads them. Resolving this requires reading each log's contents and dates.

### 3b. Acknowledged-debt ledgers under `.context/checks/`

Non-comment entries per allowlist. Each entry is a site a guard detects, reports,
and deliberately does not fire on:

| Entries | Allowlist |
|---|---|
| 236 | `mcp-parity-census-allowlist` |
| 155 | `verification-pipefail-allowlist` |
| 155 | `unbounded-rpc-call-allowlist` |
| 28 | `charter-drift-allowlist` |
| 9 | `error-swallowing-allowlist` |
| 6 | `drain-sink-allowlist` |
| 5 each | `strict-star`, `platform-lock`, `alloc-sink` |
| 4 each | `task-template-idioms`, `busy-spin` |
| 3 each | `error-code-emission`, `addressed-aliases` |
| 2 | `planted-default-gate-allowlist` |
| 0 | `version-derivation`, `verification-misfile`, `stranded-finalized`, `silent-exit`, `handover-staleness` |

**~626 acknowledged sites total.** Five allowlists are empty — those guards are at
zero debt.

---

## 4. Tool surface vs charter

| Measure | Value | Source |
|---|---|---|
| Tools in registry | **260** | `termlink help --json` |
| — deprecated | 46 | `deprecated` flag |
| — **live** | **214** | 260 − 46 |
| Live tools flagged off-charter, acknowledged | **28** | `charter-drift-allowlist` |
| Tools with NO MCP/CLI parity assertion | **236 of 260 (90.8%)** | `mcp-parity-census-allowlist` |

Registry category names include `agent_engagement_metrics`, `agent_poll`,
`agent_rankings`, `agent_stats`. Charter non-goal #3 states TermLink is
**"Not a social / engagement platform"** and that social-analytics surfaces tracing
to no coordination purpose are "removal candidates, not features".

**Intent evidence exists on the other side and must not be ignored:** the 28 are
acknowledged pending **T-2548**, an open human decision. CLAUDE.md records the
allowlist as "a ledger of an open question, not a permanent exemption."

---

## 5. Work ledger (AEF)

| Measure | Value |
|---|---|
| Tasks in `.tasks/active/` | **245** |
| Tasks in `.tasks/completed/` | 2,433 |
| active · `captured` | 123 |
| active · `started-work` | **52** |
| active · `work-completed` | **70** |
| active · owner `human` | **136** |
| active · owner `agent` | 107 |
| active · horizon `now` / `next` / `later` | 149 / 53 / 43 |

**All 70 `work-completed`-in-active are legitimate T-193 partial-complete**, not the
T-2833 stranded latch: `check-stranded-finalized-tasks.sh --json` reports
`firing_count: 0`, `partial_complete_count: 70`.

So the 70 are **agent-complete work awaiting human AC verification**. Combined with
136 human-owned active tasks, the human review step is where work accumulates.
NOT YET ESTABLISHED: how long each has waited (needs per-task `date_finished` age).

### Arcs

7 arcs: **5 in-progress** (`arc-008`, `arc-parallel-substrate`,
`arc-substrate-fitness`, `comms-loudness`, `mcp-slimming`), 2 closed
(`push-transport`, `reliable-comms`). `fw audit` reports `mcp-slimming` with no task
commits in 30 days; trend analysis shows `arc-substrate-fitness` likewise (4 of last
14 audits).

### Concerns register

48 concerns: 23 `watching`, 12 `closed`, 6 `resolved`, 5 `mitigated`, 2 `decided-build`.

---

## 6. Governance behaviour — gate bypasses

`.context/working/.gate-bypass-log.yaml`, **480 entries** (snapshot):

| Count | Reason (truncated) |
|---|---|
| 162 | *(no gate/reason field — unattributed)* |
| 93 | `Phase A batch close — agent-evidenced` |
| 82 | `Inception decision: GO` |
| 51 | `Completed via Watchtower UI (human action)` |
| 31 | *(empty string)* |
| 8 | `Confirmed duplicate of T-2256 (P-047 self-echo…)` |
| 8 | `Self-echo duplicate of T-2259 …` |
| 5 | `Human reviewed` |
| 5 | `Inception decision: NO-GO` |
| 4 | `human-ac-tick-guard` |

Two facts worth separating:

1. **193 of 480 entries (40%) carry no usable attribution** (162 unattributed + 31
   empty). The bypass log's own data quality limits what can be concluded from it.
2. **93 entries are batch closes.** CLAUDE.md states the opposite rule verbatim:
   *"No 'batch-close stale tasks' — each task needs individual evidence."*

NOT ESTABLISHED: whether those 93 were individually evidenced despite the batch
framing. The log records a reason string, not the evidence. Reading the referenced
tasks would settle it.

Hook invocation counts (snapshot, `.hook-counter`): `budget-gate` 62,
`check-project-boundary` 56, `check-tier0` 29, `checkpoint` 28, `error-watchdog` 27,
`check-active-task` 27, `audit-task-tools` 15.

---

## 7. DATA AVAILABILITY MAP

Verified against the live repo. **Designed is not built.**

| Source | Status | Location / note |
|---|---|---|
| Task ledger | **EXISTS** | `.tasks/` — 2,678 files |
| Arcs | **EXISTS** | `.context/arcs/` — 7 |
| Handovers / episodic | **EXISTS** | `.context/handovers/`, `.context/episodic/` |
| Audit history | **EXISTS** | `.context/audits/` — 148 yaml |
| Gate bypass log | **PARTIAL** | 480 entries, **40% unattributed** |
| Hook counters | **EXISTS** | `.context/working/.hook-counter` |
| Canary logs | **EXISTS** | 21 logs; 7 non-empty |
| Component fabric | **EXISTS** | `.fabric/` — 506 files, 461 cards |
| Value drivers | **EXISTS** | `policy/value-drivers.yaml` v3 — D1–D4 + free drivers |
| BVP scores per task | **PARTIAL** | fields exist; coverage not yet measured |
| **BVP realization log** | **ABSENT** | `.context/audits/bvp-realization.jsonl` does not exist |
| Workflow execution traces | **ABSENT** | no `fw workflow run` traces found |
| Ratified BPMN workflows | **PARTIAL** | `.fabric/components/workflow-aef-*.yaml` cards exist; no `*.bpmn` found |
| Runtime telemetry / usage counters per verb | **ABSENT** | no per-verb invocation counts found in-repo |
| TermLink topic state | **EXISTS (out-of-repo)** | hub SQLite; 175 topics on .122 |
| Out-of-band observer for TermLink delivery | **ABSENT** | see below |
| Issue tracker | **UNVERIFIED** | not checked; may be out-of-repo |
| External consumer usage | **UNVERIFIED** | depends on IW-3 |

**Two absences that directly cap this review:**

1. **No BVP realization log.** "Did shipped arcs deliver?" has *no* data source. Any
   claim that a capability did or did not pay off is unbacked.
2. **No per-verb usage telemetry.** For 214 live tools there is no measured
   invocation data in-repo. This is the single largest constraint: **"no telemetry"
   is not "zero use"**, so every DELETE argument resting on non-use is capped at LOW
   confidence and must route through NON-USE DIAGNOSIS reading **D (UNMEASURED)**
   before any other reading is considered.

**A channel cannot report its own failures.** TermLink delivery evidence available
here comes from TermLink's own bus. CLAUDE.md already records the gap: discarded
posts on hub loss and silent drops are "NOT AVAILABLE from the bus itself". No
out-of-band observer was found. Recorded as a data gap, not a clean bill.

---

## 8. Live observation made during gathering (out-of-band, this session)

`.122` (ring20-management) answers `fleet doctor` in 43 ms and `channel list` fast,
but `channel.subscribe` on `agent-presence` **times out at 30 s**. Effect: no agent
on that hub is discoverable via presence from outside it. Filed as **T-2970**.
Relevant here only as evidence that **discovery — charter verb 1 — is dark on one
fleet hub**, which bears on any reasoning about presence-based features.

---

## 9. Open at time of writing

- `run-guard-layer.sh` baseline — still executing
- Orphan/reference sweep (scripts, docs, CLAUDE.md split, per-crate test density,
  near-duplicate script groups) — delegated, not yet returned
- Phase 2 inventory — **not started** (blocked on IW-1 scope)
- NON-USE DIAGNOSIS per item — **not started** (blocked on IW-3; reading D is
  currently unfalsifiable for most items)
