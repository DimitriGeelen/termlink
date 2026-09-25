# VALUE REVIEW — Phase 0/1 record (run R-3044, round 1 step R1S1)

**Role:** GATHERER (read-only). **Orchestrator task:** T-3044. **Date:** 2026-09-21.
**Snapshot taken:** 2026-09-21T15:00:21Z (`.context/working/value-review-R3044-snapshot/`).
**Halt point:** PHASE 1 [ASK] — yardstick + data availability map await human confirmation.
**Nothing here is authorization.** No classification is performed (that is the JUDGE's role,
Phase 4). Facts only.

## 1. Yardstick (from docs/CHARTER.md + policy/value-drivers.yaml)

- **Purpose (canonical, human-blessed):** TermLink is a hub-mediated, durable append-log
  message bus with terminal endpoints — the coordination substrate that lets a fleet of AI
  agents (and humans) discover each other, exchange durable messages, claim work, and
  control terminal sessions across one or many machines.
- **Users/consumers:** AI agents in a cooperating fleet, plus human operators; the Agentic
  Engineering Framework (AEF) builds orchestration policy on top.
- **Core capabilities (four charter verbs):** (1) discover peers, (2) exchange durable
  messages, (3) claim work, (4) control terminal sessions.
- **Non-goals:** not inter-hub federation; not a durable database / system of record; not a
  social/engagement platform; not a workflow/orchestration engine; not a security boundary
  between mutually-distrusting tenants.
- **Value drivers (weights):** protected D1 Antifragility 9 · D2 Reliability 7 ·
  D3 Usability 5 · D4 Portability 3. Free: F-RECALL Recall Leverage 6 · F-ORCH
  Orchestration Leverage 5.
- **Yardstick self-consistency:** the canonical sentence's three copies (CHARTER, README,
  ARCHITECTURE) are in sync — `check-charter-sentence-drift.sh` PASS this run.

### Yardstick flags for the human
- **Y-1.** `fw audit` (2026-09-21) WARNs that free driver **F-ORCH's `retire_when`
  condition appears met**. F-ORCH carries weight 5 and would shape every ranking in this
  review. Retire, keep, or re-weight? (Sovereign — weights are not changed by this review.)
- **Y-2.** Two `value-drivers.yaml` exist. `policy/value-drivers.yaml` (project) is
  authoritative — `lib/bvp.sh` resolves `PROJECT_ROOT/policy/...` at 4 sites; the
  `.agentic-framework/policy/` copy is only the bootstrap template. No contradiction, but
  the template carries *different* free drivers (F-AUTONOMY, F1, F2, F3), so quoting the
  wrong file would silently change the yardstick.
- **Y-3. Scope boundary.** `{{SCOPE}}` = "whole repo". The repo contains both the TermLink
  product (`crates/`, 181k LOC Rust) and a vendored AEF framework (`.agentic-framework/`)
  whose code is governed upstream (G-062: local fixes are erased by the next re-vendor).
  All 13 BPMN workflows live in the vendored tree. Does "whole repo" include it?

## 2. Project shape (verified)

| Dimension | Measured |
|---|---|
| Product source | 7 Rust crates, 181,037 LOC (`cli` 72,790 · `mcp` 50,881 · `hub` 24,848 · `session` 23,838 · `bus` 5,240 · `protocol` 3,093 · `test-utils` 347) |
| Shell surface | 193 `scripts/*.sh`, 88 `tests/*.sh` |
| Docs | 409 markdown files under `docs/` |
| Operator surface | 34 `.claude/commands/*.md` slash commands |
| Governance ledger | 286 active + 2,465 completed tasks; 8 arcs; 1,752 handovers; 2,466 episodic; 509 fabric components; 153 daily audits |
| Git | 7,293 commits, 2026-03-08 → 2026-09-21 |
| CI | `.github/workflows/`: doc-lint.yml, install-check.yml, release.yml |
| Versions | VERSION file `0.11.2065`; installed binary `0.11.1716` |

## 3. Baseline

- **fw audit (cron, 2026-09-21T14:38:24Z, `sections: "structure"` only):** 38 PASS / 9 WARN
  / 1 FAIL. FAIL = `cron(substrate-smoke-canary): USER-field syntax but no install in
  /etc/cron.d`. Note the audit runs the **structure section only** — it is not full coverage.
- **Guard layer** (`scripts/run-guard-layer.sh`): COMPLETE, exit 1 (FIRING).
  **117 PASS / 3 FAIL / 1 ERROR** across 121 members.
  - `check-installed-binary-drift.sh` **FAIL** — independent corroboration of D-1 below.
  - `check-pickup-deferred-freshness.sh` **FAIL** (stranded/stale pickup envelope).
  - `cron-drift-firing-fixtures.sh` **FAIL** — consistent with the `fw audit` FAIL
    (`substrate-smoke-canary` declared but not installed in `/etc/cron.d`).
  - `check-receiver-ack-lag.sh` **ERROR** — the guard could not run; per the layer's own
    contract ERROR is deliberately not PASS, so this is an unknown, not a clean bill.
  - **Measured runtime 17:05:30 → 17:17:13 = ~11m43s (703 s)** against a documented
    "(seconds)" budget — a third independent measurement of prior finding C-18
    (run4 ~20 min, run5 >600 s), and it gates every push/PR against a 10-min CI cap.
  - **Coverage caveat (read this green narrowly):** the runner reports that **75
    check/suite scripts carry no `# guard-layer:` marker and were not run**. 117 PASS
    describes the marked members only, not the repo's whole guard surface.
  - `cargo test --workspace` was NOT included (needs `--tests`).
- **cargo test --workspace:** NOT RUN this phase (minutes-scale; deferred to Phase 3).

## 4. Data availability map

Status key: EXISTS · PARTIAL · DESIGNED-ONLY · ABSENT. Trust key: measured > observed >
inferred > claimed.

### Layer A — generic

| Source | Status | Location / window | Trust & caveat |
|---|---|---|---|
| References / call graph | EXISTS | `grep`/`cargo tree`; `.fabric/` 509 cards | inferred; dynamic refs invisible |
| Dead-code / unused-dep tools | ABSENT | cargo-udeps, cargo-machete, vulture, knip, jscpd all absent | local install permitted by ground rules |
| shellcheck | EXISTS | `/usr/local/bin/shellcheck` | inferred; covers 281 shell files |
| Complexity / duplication / coupling | PARTIAL | derivable from git + source; no tool installed | inferred |
| Git churn / hotspots / change coupling / author concentration | EXISTS | 7,293 commits, 6.5 months | observed; strongest structural source |
| Fix/revert ratio | EXISTS | git log | observed |
| Coverage per item | ABSENT | no llvm-cov/tarpaulin | — |
| Tests exercising an item | EXISTS | `cargo test`, 88 fixture suites, guard layer | observed |
| Flaky/slow/skipped tests | PARTIAL | guard-layer verdicts; no historical CI record locally | observed-now, not history |
| Runtime logs / errors | EXISTS | `/var/lib/termlink/rpc-audit.jsonl` 979,022 records | **measured**; see window caveat below |
| Usage counters per feature | **PARTIAL — critical** | see D-1 below | — |
| Issues per component | ABSENT | no local issue tracker; `.context/concerns.yaml` is the nearest proxy | observed |
| CI failures/duration per job | ABSENT locally | GitHub-side only; repo is mirrored read-only | — |
| Workarounds | EXISTS | 193 scripts, 34 slash commands, `.context/learnings.yaml` (289 KB) | observed |
| Docs vs code drift | EXISTS | 29 `scripts/check-*.sh` static checks + canaries | measured/inferred |

### Layer B — AEF / TermLink / workflows

| Source | Status | Location / window | Trust & caveat |
|---|---|---|---|
| Task ledger | EXISTS | 2,751 tasks | observed; richest origin source |
| Arcs | EXISTS | 8 arcs; arc-009 is the prior review's execution arc | observed |
| Handovers / episodic | EXISTS | 1,752 / 2,466 | observed |
| Upstream reports | EXISTS | `.context/upstream/` 9 files | observed |
| Component fabric | EXISTS | 509 cards | inferred |
| Capability registry (`policy/capabilities.yaml`) | **ABSENT** | no such file anywhere in repo | prompt's indicative path does not exist here |
| Prompts / dispatch templates | EXISTS | `docs/prompts/` (2), `.agentic-framework/agents/dispatch/` (5) | claimed |
| Audit logs | EXISTS (different form) | `.context/audits/*.yaml` ×153 daily; **no `*.jsonl`** | observed; structure section only |
| Gate bypass log | EXISTS | `.gate-bypass-log.yaml`, **485 entries** | **measured** |
| Gate first-pass rate | PARTIAL | derivable from bypass log + hook counters; hook counter has a known corruption defect (C-20) | degraded |
| Healing events | **PARTIAL — near-dead** | `patterns.yaml` last changed **2026-03-14**; 12 patterns total | stale ≈6 months |
| Per-verb / per-tool usage | **DESIGNED-ONLY (data)** | see D-1 | — |
| RPC method usage | EXISTS | `rpc-audit.jsonl`, 979,022 records, **2026-08-16 → 2026-09-21 (36 days)** | **measured**; rotates on size — not full history; RPC-method grain cannot answer per-tool questions |
| Token telemetry per task/arc | PARTIAL | `.context/working/.session-metrics.yaml`, `.budget-status` | observed |
| Activity per scope | EXISTS | git + task ledger | observed |
| Value drivers | EXISTS | `policy/value-drivers.yaml` | authoritative |
| BVP scores | PARTIAL | frontmatter fields present; 73% of tasks lack the anchor (CLAUDE.md) | claimed |
| **Realization log** (`.context/audits/bvp-realization.jsonl`) | **ABSENT** | not present at documented path or anywhere under `.context` | "did shipped arcs deliver?" is unanswerable |
| Estimator override / calibration logs | ABSENT | — | — |
| TermLink topic append-log | EXISTS | 44 topics, `/var/lib/termlink/bus/` + `meta.db` | **measured** |
| Subscriber cursors / receipts | EXISTS | `meta.db` | measured |
| Presence / heartbeats | EXISTS | 14 + 21 session registrations | measured; in-memory presence resets on restart — not history |
| Out-of-band observer of bus loss | **ABSENT** | drops/discards on hub loss are unobservable from the bus itself | structural data gap (prompt names this) |
| Ratified workflows | **PARTIAL** | 13 BPMN, all under vendored `.agentic-framework/.context/designer/projects/`; **no machine-readable ratification marker** — "ratified" appears only in prose | claimed; weakens the DELETE protection the prompt relies on |
| Workflow execution traces | **ABSENT** | no `fw workflow run` traces | prompt predicted this |

### D-1 — the one data finding that changes what this review can conclude

Prior value-review runs capped every usage verdict at **UNMEASURED** because
`rpc-audit.jsonl` tallies by *RPC method*, and methods do not correspond to tools
(`channel.subscribe` alone is 403,555 dispatches and backs many distinct tools).
**T-2996 was built to close exactly that gap** — `scripts/invocation-usage.sh` reading
`<runtime_dir>/invocation-audit.jsonl`, written by
`crates/termlink-hub/src/invocation_audit.rs`.

Verified state:
- Writer code EXISTS and IS wired (`termlink-mcp/src/server.rs`, `termlink-hub/src/lib.rs`).
- It is **default-enabled** (only `TERMLINK_INVOCATION_AUDIT=0` disables).
- **The sink file does not exist** in `/tmp/termlink-0`, `/var/lib/termlink`, or `~/.termlink`.
- Cause: T-2996 landed **today**, commit `c1c8165f6` (2026-09-21). The running MCP server
  (pid 35408) started **08:52 today** on binary **0.11.1716**, which predates the commit.

**The writer is demonstrably functional.** A bounded search found exactly one sink on this
host — `/tmp/tl-test-0-parity-list-sessions/invocation-audit.jsonl`, a TEST fixture dir,
25 well-formed records written 2026-09-21 09:28, schema `{ts, surface, name}`, e.g.
`{"ts":1789975683863,"surface":"mcp","name":"termlink_kv_set"}`. The commit landed 01:07
local; the test exercised it at 09:28; the serving MCP process started 08:52 on the older
binary. So the code WORKS in isolation and NO production runtime_dir has a sink.

Both halves are recorded deliberately: the prompt's NON-USE DIAGNOSIS needs exactly this
evidence to separate reading **A (BROKEN)** from reading **B (NEVER WIRED / not yet
running)**, and the working test sink points away from A. I do not pick the reading — that
is the JUDGE's call in Phase 4.

Two incidental facts for the JUDGE: the schema carries a `surface` field (consistent with
the reader's note that CLI verbs are not instrumented, so a future CLI surface is
anticipated), and the test records include `termlink_kv_set/get/list/del` — the very tools
prior finding **C-30** called "structurally invisible". Once live, this sink would speak to
C-30, C-29, C-31 and the C-01/C-02 external-consumer checks (IW-1) that prior runs could
not discharge.

So the capability is **shipped but not live** — the project's own G-069 class. Consequence
for this run: **per-tool usage is still UNMEASURED**, and a rebuild + MCP restart would
begin producing the data (a Phase-6 action requiring approval; I have not taken it).

**Evidence scope (stated precisely).** Absence was verified in the three runtime dirs the
binary's own resolution order can select (`/tmp/termlink-0`, `/var/lib/termlink`,
`~/.termlink`) and by a bounded `find` over `/tmp /var/lib /root` (maxdepth 6). It is not
a whole-filesystem proof; a first attempt at that was killed by the OOM reaper before
producing output. `/run` could not be searched — the project-boundary hook blocked the
read (see contradiction 6).

## 5. Material context the human should weigh before Phase 2

**This is at least the seventh value-review run of this repo.** `docs/reports/` holds
`VALUE-REVIEW-repo-2026-09-16-evidence.md` and a five-run series dated 2026-09-19 (run1–run5
plus judge verdicts), consolidated in `VALUE-REVIEW-repo-2026-09-19-consolidated.md` —
**45 deduplicated findings C-01…C-45**, with cross-run recurrence marks, sovereign
questions, data gaps and a task decomposition.

Those findings are filed as **arc-009** (status `in-progress`, anchor T-2974), slices
S-1…S-18 mapped to T-2975…T-2992. Verified execution state:

- **3 of 18 completed** — T-2975 (C-13), T-2982 (C-20), T-2989 (C-04 inception).
- **15 open**, of which **14 are still `captured`** (not started).
- 3 of the open ones are `owner: human` (T-2984, T-2992, and the completed T-2989).

An eighth cold review would re-derive substantially the same findings against substantially
the same evidence, while 83% of the previous round's findings remain unexecuted.

## 6. Contradictions observed in Phase 0 (recorded, not classified)

1. **Version skew.** VERSION `0.11.2065` vs installed binary `0.11.1716` (~349 commits).
   Directly causes D-1.
2. **Two live runtime dirs.** MCP server uses `/tmp/termlink-0`; hub (pid 21462) uses
   `/var/lib/termlink`. Both hold `hub.pid`, `hub.sock`, `rpc-audit.jsonl` (32 KB vs 95 MB).
   `/tmp` is on rootfs here (not tmpfs), so the PL-021 tmpfs mechanism does not apply, but
   the split means telemetry written by one is invisible to a reader pointed at the other.
3. **Prompt's indicative paths vs reality.** `policy/capabilities.yaml`,
   `.context/audits/*.jsonl` and `.context/audits/bvp-realization.jsonl` do not exist here.
4. **Healing loop.** CLAUDE.md documents an active healing/pattern loop; `patterns.yaml`
   has not changed since 2026-03-14 and holds 12 patterns against 2,465 completed tasks.
5. **Audit coverage.** `fw audit` is described as a compliance audit; the daily cron runs
   `sections: "structure"` only, so its PASS count speaks for one section.
6. **Project-boundary gate blocks read-only access twice this session.** Attempts to
   `find`/stat `/run/user/0/termlink` and `/run` were refused by the
   `check-project-boundary` hook although both were read-only and one targeted this
   project's own hub runtime path. This is live corroboration of prior finding **C-23**,
   observed independently here rather than inherited from the earlier run.
7. **Guard-layer coverage.** The runner reports 75 `check-*`/suite scripts carrying no
   `# guard-layer:` marker, so they were not run. Its 117 PASS is not a whole-surface
   green — the same "read a green narrowly" caveat (T-2680) the repo applies elsewhere.

## 7. Not yet done (Phases 2–7)

Inventory, evidence file, classification, report, execution and close — all pending the
[ASK] confirmation. Budget consumed in Phase 0/1 is well inside the 200k allowance.
