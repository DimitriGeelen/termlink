# VALUE REVIEW — repo — 2026-09-21 — Round 1 (R-3044 step R1S1)

**Run:** R-3044-value-review-procasfit · **Orchestrator task:** T-3044 · **Step:** R1S1
**Date:** 2026-09-21 · **Scope:** whole repo (/opt/termlink) · **Budget:** 200k tokens
**Phases executed:** 2–5 (Phase 0/1 recorded separately in
`docs/reports/VALUE-REVIEW-repo-2026-09-21-phase01.md`).
**Halt point:** end of Phase 5. **PHASE 6 NOT ENTERED.** No DELETE, REFACTOR or ADD item
has been executed. Nothing in this report is authorization.

**Rescope (human decision, 2026-09-21T15:40Z, recorded in `.context/runs/T-3044-sequence.yaml`):**
this is **not** a cold eighth review. It is (a) a verification of the 45 consolidated
findings C-01…C-45 against the current tree, and (b) Phases 2–5 over ground arc-009 does
not already cover.

---

## 0. The one-paragraph answer

The verification says the project's 45 open findings are overwhelmingly still true — but the
more useful result is *why* four of them turned out to be false. C-31, C-32, C-40 and C-43
were not defects in the product; they were **defects in the instruments that measured it**.
A zero-call reading taken from an audit sink that never observed the method. A "rail
collapsed" reading from a counter that was scanning a trimmed window. A "nobody acks" reading
from rows that were senders, not readers. In the same sweep this review found the same class
one level up and still live: **four hub-dependent canaries on this host cannot reach the hub
at all, exit 2, write nothing, and are reported `HEALTHY`.** The dominant failure mode in
this repo is not missing capability and not unmaintainable code. It is *instruments that
cannot see, reporting green* — and a governance layer that then believes them. Every
top-ranked proposal below attacks that.

---

## 1. Yardstick (confirmed)

Confirmed by the human 2026-09-21T15:40Z, as recorded in the Phase 0/1 artifact §1.

- **Purpose:** TermLink is a hub-mediated, durable append-log message bus with terminal
  endpoints — the coordination substrate that lets a fleet of AI agents (and humans) discover
  each other, exchange durable messages, claim work, and control terminal sessions across one
  or many machines.
- **Users/consumers:** AI agents in a cooperating fleet, plus human operators; the AEF builds
  orchestration policy on top.
- **Core capabilities (4 charter verbs):** discover peers · exchange durable messages ·
  claim work · control terminal sessions.
- **Non-goals (5):** not inter-hub federation · not a durable database / system of record ·
  not a social/engagement platform · not a workflow/orchestration engine · not a security
  boundary between mutually-distrusting tenants.
- **Value drivers:** protected **D1** Antifragility 9 · **D2** Reliability 7 · **D3**
  Usability 5 · **D4** Portability 3. Free: **F-RECALL** 6 · **F-ORCH** 5.

> **Standing assumption, stated because rankings depend on it (human decision Q2).**
> `fw audit` WARNs that free driver **F-ORCH's `retire_when` condition appears met**. The
> human did **not** resolve this; F-ORCH stays **LIVE at weight 5** for this run, because
> no-change is the only default an agent may take on a sovereign parameter. **Every ranking
> in §6 and §8 therefore carries the assumption "F-ORCH is live at 5."** Where a ranking
> would move if F-ORCH were retired, the row says so. This is re-raised as **SQ-R1-1**.

---

## 2. Data availability map (confirmed) + snapshot windows

Confirmed unchanged from the Phase 0/1 artifact §4, which the human accepted. Only the rows
whose status **moved during this step** are restated here.

| Source | Status | Window / location | Note |
|---|---|---|---|
| RPC method audit | **EXISTS (measured)** | `/var/lib/termlink/rpc-audit.jsonl`, **983,713 records, 2026-08-16T14:53Z → 2026-09-21T19:19Z (36.2 days)** | rotates on size; method-grain only |
| Topic append-log | **EXISTS (measured)** | 47 topics / 4,523 records, read via `channel list --json` | **only reachable with `TERMLINK_RUNTIME_DIR` set — see N-1** |
| Per-tool / per-verb invocation telemetry | **DESIGNED-ONLY (no data)** | writer shipped `c1c8165f6`; **no production sink in any of the 3 runtime dirs** | per-tool usage **UNMEASURED this round** (human decision Q4) |
| BVP realization log | **ABSENT** | — | "did shipped arcs deliver?" still unanswerable |
| `policy/capabilities.yaml` | **ABSENT** | — | prompt's indicative path does not exist here |
| Coverage / unused-dep tooling | **ABSENT** | — | caps REFACTOR CHECK 2 on C-06/C-07 |
| GO-propagation audit | **EXISTS (measured)** | `.context/audits/go-scope-unpropagated/LATEST.md` | 198 completed inceptions / 168 GO / **86 unpropagated** |
| Gate bypass log | **EXISTS (measured)** | `.context/working/.gate-bypass-log.yaml`, **485 entries** | breakdown corrected in C-37 |

**Snapshot.** Phase 0 snapshot `.context/working/value-review-R3044-snapshot/` taken
2026-09-21T15:00:21Z. All usage figures above are reads of append-only logs, taken
2026-09-21T19:00–21:30Z.

**Self-induced pollution — declared.** In verifying C-39/C-22 I ran four canary scripts
ad hoc without `--no-heartbeat`, which **refreshed four canary heartbeats at 21:22**
(`stuck-claims`, `topic-growth`, `forever-archival`, `framework-pickup`). This is exactly the
pollution the prompt's ground rules warn against and it is my error. It does not affect any
finding below (all rest on log *content* and exit codes, not heartbeat mtimes) — and it
incidentally **demonstrates C-13's mechanism live**: an ad-hoc run refreshes the heartbeat
without appending, which is precisely the FIRING→HEALTHY flip C-13 describes.

---

## 3. Role setup

| Role | How it ran | Effect on confidence |
|---|---|---|
| GATHERER | Split: 2 independent sub-agents (MCP-surface cluster; canary/guard cluster) + the orchestrator for the remaining 24 findings | partial separation |
| JUDGE | **Not separated** — the orchestrator classified the evidence it partly gathered | **−1 confidence level on every self-gathered row**, applied |
| HUMAN | Confirmed yardstick + data map (Phase 1); pre-authorised Phases 0–5 as read-only; **retains Phase 6** | gates intact |

**Why separation is only partial — and it is a finding, not an excuse.** Five GATHERER
workers were dispatched. **Three were blocked** by the framework's own agent-dispatch gate
(`BLOCKED: Agent dispatch #3 exceeds limit (2)`). That gate is **C-16**, a known-defective
vendored check: it is documented as per-session but behaves per-project-lifetime, and
**blocked attempts themselves increment the counter** — the counter read **5** after three
refusals, so being blocked made it more blocked. I did **not** run `fw dispatch approve`:
self-approving a governance gate in an autonomous step is not mine to do. The 24 remaining
findings were therefore verified by the orchestrator directly, and every such row is marked
**self-judged** and carries the one-level confidence penalty the prompt requires.

---

## 4. Baseline

| Check | Result |
|---|---|
| `cargo test --workspace` | **1,142 passed / 1 FAILED** — `commands::dispatch::tests::isolate_rejects_non_git_dir` (`dispatch.rs:1195`), panic: *"Expected git repo error, got: Hub is not running"* — **same root cause as N-1** |
| `fw audit` (cron 14:38Z, `sections: structure` only) | 38 PASS / 9 WARN / **1 FAIL** (`cron(substrate-smoke-canary)` not installed) |
| Guard layer (`run-guard-layer.sh`) | exit 1 (FIRING) — 117 PASS / 3 FAIL / 1 ERROR of 121 members; **703 s measured** vs documented "(seconds)"; **75 unmarked scripts not run** |
| `/canaries` | 7 non-empty logs; reports `firing:0` while `waker-liveness` prints *"FIRING — RAIL DARK"* |

---

## 5. Summary

### 5a. Verification of the 45 prior findings (rescope part a)

| Verdict | Count | IDs |
|---|---|---|
| **STILL-TRUE** | **33** | C-02,03,05,06,07,08,09,11,12,14,15,16,17,18,19,21,22,23,25,26,27,28,29,30,34,35,36,37,38,39,41,42,44,45 *(C-45 reframed)* |
| **PARTIALLY-FIXED** | **4** | C-01, C-10, C-13, C-24 |
| **FIXED** | **1** | C-20 |
| **SUPERSEDED** | **1** | C-33 |
| **NOW-WRONG** | **4** | C-31, C-32, C-40, C-43 |

Of the 33 still-true, **9 are measurably worse** than 2 days ago (C-03, C-11, C-12, C-14,
C-21, C-35, C-38, C-39, C-42). **None** of the four hotspot LOC figures (C-06/07/08) moved by
a single line: no refactor has begun.

### 5b. Execution state of arc-009 — a correction to the record

The rescope decision was taken on the figure *"3 of 18 slices complete, 14 captured."*
**Measured: arc-009 has 38 slices (T-2975…T-3012), of which 12 are work-completed, 2 are
started-work, and 24 are captured.** The record undercounted both the denominator (18 vs 38)
and the completions (3 vs 12). The rescope's *direction* survives this correction — 63% of
the arc is still unstarted — but the human should know the premise was understated.

### 5c. Top 3 per axis

**DELETE** — *(no new DELETE is proposed this round; all three remain human-gated from the prior round)*
1. **C-02** — 46 deprecated-but-compiled MCP tools, 48-day soak, zero callers. The 40-vs-46 conflict is now settled at **46**. Blocked only on the written human go-ahead.
2. **C-01** — 28 off-charter analytics tools. Removal tasks **now exist** (T-2993/T-2994, both `captured`, `owner: human`); IW-1 still undischarged.
3. **C-03** — 22 of 47 topics are test/demo/probe residue. Ops hygiene, agent-runnable, excludes fedprobe.

**REFACTOR**
1. **C-04** — CLAUDE.md is **byte-identical** to the prior review (276,749 B / 3,283 lines). Its inception (T-2989) recorded GO and produced **no follow-up task**.
2. **C-06/C-07** — `tools.rs` 46,458 LOC at 9.2% parity coverage; `channel.rs` 20,435 LOC. Unchanged. Coverage work (T-2991) gates the split and has not started.
3. **C-25** — governance artifacts 90 MB `.context` + 26 MB `.tasks` against **7.3 MB** of product source.

**ADD**
1. **N-1 (new)** — the runtime-dir split silently disables hub-dependent canaries, breaks the default CLI, and fails a workspace test. **Highest value-per-cost item in this report.**
2. **N-2 (new)** — arc-009's own GO inceptions leak scope 8-of-9; the remediation arc reproduces the defect (C-35) that one of its own slices diagnosed.
3. **C-45** — per-tool invocation telemetry: writer shipped today, **no production sink**; it is one rebuild+restart from existing, and it unblocks C-01/IW-1, C-30, C-38.

---

## 6. Findings

### 6a. NEW findings — Phases 2–5 over ground arc-009 does not cover (rescope part b)

Ranked by value per unit of cost and risk. `Conf` already carries the −1 self-judged penalty
where it applies.

| ID | Item | Location | Class | Reading | Evidence | Counter-evidence | Conf | Proposal | Size | Rev? | Risk if wrong | Expected effect |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **N-1** | **Runtime-dir split silently disables hub-dependent canaries, breaks the default CLI, and fails a workspace test.** The hub (pid 21462, systemd `active`) serves from `/var/lib/termlink`; the default client resolution lands on `/tmp/termlink-0`, whose `hub.sock` is stale (14:07 vs the live 08:52). | `/tmp/termlink-0/hub.sock` vs `/var/lib/termlink/hub.sock`; `.context/cron/*.crontab` | **ADD (REPAIR)** | **A — BROKEN** | Measured 3 ways: (1) `termlink channel list` → *Connection refused*; same command with `TERMLINK_RUNTIME_DIR=/var/lib/termlink` → lists 47 topics. (2) `check-{stuck-claims,topic-growth,forever-archival,framework-pickup}-freshness.sh` **exit 2** under default env; framework-pickup exits **1 with real unacked filings** under the correct env. (3) `cargo test` failure `isolate_rejects_non_git_dir` panics with *"Hub is not running"*. **Only 1 of 27 crontabs sets `TERMLINK_RUNTIME_DIR`** — and that one sets it to `/var/lib/termlink`, proving the correct value is known. | `topic-growth` and `forever-archival` logs are 0 bytes, which is *also* what a genuinely healthy canary looks like — so the blindness cannot be proven from the log alone. That is the finding, not a weakness in it. | **HIGH** — measured, 3 independent surfaces, reproducible both directions | Set `TERMLINK_RUNTIME_DIR` in the canary crontabs (the presence-sweep crontab is the template), **or** make the client prefer a live socket over a stale one. Then re-run the 4 canaries and triage whatever they surface. Do **not** delete the stale socket alone — that masks the resolution defect. | Small | Yes | Low. Wrong fix = canaries still blind, detectable by exit code. | The 4 canaries return exit 0/1 (never 2) under cron env; `.stderr` sidecars stay empty; the failing test passes |
| **N-2** | **arc-009 reproduces the defect it was created to fix.** 8 of its 9 completed GO inceptions produced **no follow-up task**; the systemic leak is now **86/168 = 51.2%** (C-35 measured 48.7%), and the **8 most recent entries in the audit's own findings list are arc-009's own slices**. | `.context/audits/go-scope-unpropagated/LATEST.md`; `.tasks/completed/T-29xx,T-30xx` | **INVESTIGATE (process)** + ADD(WIRE) | — | Per-task reference sweep: T-2989, T-2995, T-3001, T-3004, T-3005, T-3007, T-3011, T-3012 → 0 referencing tasks in `active/`. Only T-3003 propagated (→ T-3039, T-3040) — **and T-3003 is the slice that diagnosed the leak.** | Some of the 86 may have shipped work that was merely never linked; the audit says so itself. | **MEDIUM** (measured; self-judged −1) | Do not file 8 tasks by hand — that treats the symptom. Make `fw inception decide … go` **require** a `related_tasks:` target or an explicit `defer` reason, i.e. move the link from agent judgement to framework code. Vendored ⇒ **file upstream** (G-062). | Medium | Yes | Medium — a naive required-field prompt gets stub-filled, reproducing the boilerplate-Decision problem seen in T-3005/T-3011/T-3012 | Unpropagated ratio falls below 45% over 30 days of audits |
| **N-3** | **`substrate-smoke` canary is shipped-but-dark** — crontab git-tracked, absent from `/etc/cron.d`. This is the **root cause** of C-44's red fixture suite (the suite fails its own "the real tree passes" assertion) and of the single `fw audit` FAIL. So the verb-3 composition prover has never run on cron on this host. | `.context/cron/substrate-smoke-canary.crontab` | **ADD (WIRE)** | **B — NEVER WIRED** | `check-cron-install-drift.sh` → `MISSING: substrate-smoke-canary.crontab → /etc/cron.d/termlink-substrate-smoke-canary`; `tests/cron-drift-firing-fixtures.sh` 12 passed / **1 failed** | Tracked as T-2939 per the prior round's §4 note — so known, but still dark and still reddening a fixture suite. | **HIGH** (measured, 2 sources) | Install the crontab. One `cp`. Then C-44 closes as a consequence rather than as its own investigation. | Small | Yes | Nil | `fw audit` FAIL → PASS; `cron-drift-firing-fixtures.sh` 13/13 |
| **N-4** | **A third bypass category was never surfaced by any of the five prior runs.** C-37 reported 482 bypasses as "313 = focus-drift 164 + skip-sovereignty 149". Measured now: **485 entries — `FW_SWITCH_FOCUS=1` ×157, `--skip-sovereignty` ×152, and `--skip-acceptance-criteria` ×120**, plus `--skip-human-ownership` ×10 and `--skip-verification` ×8. | `.context/working/.gate-bypass-log.yaml` | **INVESTIGATE** | — | Field-keyed count over the `flag` field of all 485 entries | 120 AC-skips may be concentrated in a small number of sanctioned batch operations (the log shows a "Phase A batch close — agent-evidenced" reason ×93) | **MEDIUM** (measured; self-judged −1) | Fold into the C-37 bypass-sample audit, but **widen it to the AC-skip category**, which bypasses P-010 — the completion gate, not a focus convenience. Sample 20 AC-skips with reasons before concluding anything. | Small | — | Medium — concluding "gate misfit" from volume alone would weaken a completion gate | Sample characterises the 120; C-37 answerable on evidence |
| **N-5** | **The fabric card generator emits edgeless cards.** C-38's edgeless share rose 338/503 → **344/509**: every one of the 6 cards created since 2026-09-19 has neither `depends_on` nor `depended_by`. That reframes C-38 from a backfill backlog to a **generator defect** — backfilling would be refilling a leaking bucket. | `.fabric/components/*.yaml` | **INVESTIGATE → ADD(REPAIR)** | **A — BROKEN (generator)** | 509-card census; 344 with both edge lists empty/absent; 0 parse errors | Some components genuinely have no edges (leaf scripts) — but 67.6% cannot all be leaves | **MEDIUM** (measured, sub-agent-gathered) | Fix `fw fabric register` to populate edges (or fail loudly when it cannot) **before** any backfill task is scheduled. | Small–Medium | Yes | Low | New cards carry edges; edgeless share falls rather than rises |
| **N-6** | **`check-installed-binary-drift.sh` silently ignores `--json`** — the flag appears nowhere in the script; it emits the human table and exits 1. A caller parsing it as JSON gets a wrong answer, not an error. | `scripts/check-installed-binary-drift.sh` | **ADD (REPAIR)** | **A — BROKEN** | `grep -n '\-\-json'` → 0 hits; `--json` output `grep -c '^{'` → 0 | — | **MEDIUM** (measured, sub-agent-gathered) | Implement `--json` or reject the unknown flag with exit 2. This is the repo's own Directive-#2 class, in its own guard layer. | Small | Yes | Nil | `--json` yields parseable JSON or a loud refusal |
| **N-7** | **The RPC audit log cannot distinguish prober traffic from organic traffic.** `event.broadcast` now shows **2 calls** — both on 2026-09-21 08:43:58 and 08:44:03, i.e. T-2995's own wiring sweep. A future review running `grep -c` will read 2 and infer the method is alive. | `/var/lib/termlink/rpc-audit.jsonl` | **INVESTIGATE (data quality)** | — | Timestamp extraction of both records; T-2995 completed 11:06 the same morning and documents running the sweep | The volume is trivially small today | **MEDIUM** (measured; self-judged −1) | Tag prober/self-test traffic at source (a `probe: true` field or a reserved sender id) so usage queries can exclude it. Cheap now, and it protects every future usage verdict. | Small | Yes | Low | Usage queries can filter probes; zero-call readings stay admissible |
| **N-8** | **623 MB of stale dead disk.** `.context/working/fw-vec-index.db` is 623 MB, last written **2026-08-03** (49 days idle). It is correctly gitignored (`.gitignore:139`) and untracked, so it is **not** a G-058 risk — but it is 87% of the `.context` tree and inflates every governance-volume measurement. | `.context/working/fw-vec-index.db` | **KEEP-with-hygiene** (no action proposed) | — | `ls -la`; `git check-ignore -v` confirms ignored; `git ls-files --error-unmatch` confirms untracked | It may be a warm cache something rebuilds on demand — **verify before removing** | **MEDIUM** (measured; self-judged −1) | No deletion proposed. Recorded so the next review does **not** repeat my own first reading, which was that `.context` had grown 105 MB → 641 MB. It has not. | — | — | — | — |

### 6b. Corrections to the prior round's findings

These four rows are the most valuable output of the verification, because each one *removes*
work from the backlog rather than adding it.

| ID | Prior claim | Verdict | What is actually true | Source |
|---|---|---|---|---|
| **C-31** | `session.*` shows zero hub-observed calls | **NOW-WRONG (evidence inadmissible)** | `termlink-session/src/handler.rs` serves the entire `session.*` lifecycle and has **zero audit references**; `server.rs:1610` is the only general dispatch audit site. The zero was never evidence of disuse. | T-2995 IW-2 |
| **C-32** | `event.*` family shows zero calls | **NOW-WRONG (evidence inadmissible)** | `rpc_audit.rs:47 SKIP_METHODS` excludes `event.poll`/`event.collect` **by design** (T-1307). Same blind spot, different mechanism. | T-2995 IW-2 |
| **C-40** | Chat-arc collapsed — 0 posts / 0 speakers for 10 daily snapshots | **NOW-WRONG** | The rail is **alive** (posts today, 4 speakers/30d). The snapshot counter is structurally blind on hubs without `latest_offset`: the `tail = count-1` fallback lands the scan window entirely in the trimmed past. Reproduced live. | T-3004 |
| **C-43** | 4 of 5 identities never ack on `agent-chat-arc` — write-only-sink (G-063) | **NOW-WRONG (framing over-reached)** | The guard's rows are the topic's **senders**, not its readers. On a broadcast rail the never-ackers are its own posting bots, and subscribe-reads are invisible to receipts by design. T-3004 proved live readers the same day. | T-3007 |
| **C-33** | `event.broadcast` is a DELETE candidate that failed DELETE CHECKS 4–5 | **SUPERSEDED** | It was **cut 2026-05-31** (`router.rs:1017`, T-1166/T-1415); the hub returns `-32601`. Residue is a constant, a `LEGACY_METHODS` warn entry, and `tests/no_legacy_callers.rs`, which *enforces* the retirement by naming it. Nothing to delete. | T-2995 IW-3 |
| **C-20** | Hook-counter corruption, unlocked read-modify-write | **FIXED / handled per G-062** | Registered in `.vendor-divergence.yaml:80` with a race-fixture harness `tests/hook-telemetry-race-fixtures.sh`. Handled the way a vendored defect is supposed to be handled. | `.vendor-divergence.yaml` |
| **C-01 (sub-claim)** | run2/run4/run5 disagreed on whether T-2548 records a GO (disagreement D-1) | **RESOLVED — not a data error** | The file carries **two headings**: an empty `## Decisions` template stub at the usual position, and a separate `## Decision` further down carrying `**Decision**: GO`. A reader anchored on the first sees empty. Run3 was right. **The defect is the duplicate heading, not a wrong record** — so C-01's remediation step 1 is misframed. | sub-agent read of `.tasks/completed/T-2548-*.md` |

### 6c. Verification table — all 45 prior findings against the current tree

`Δ` marks a finding measurably **worse** than 2 days ago. All rows verified 2026-09-21.

| ID | Verdict | Measured now (vs prior) | Evidence |
|---|---|---|---|
| C-01 | PARTIALLY-FIXED | 28 off-charter, all allowlisted, 0 firing. **Removal tasks now exist** (T-2993/T-2994, `captured`, human). IW-1 undischarged. | `check-charter-drift-freshness.sh --json`; `.context/checks/charter-drift-allowlist` |
| C-02 | STILL-TRUE | **46** deprecated / 260 total / 214 live — settles the 40-vs-46 conflict at 46 | `termlink help --json`; `tools.rs:1270` |
| C-03 | STILL-TRUE Δ | **22 of 47** topics residue-shaped, 1,942 records (was 20/45) | `channel list --json` |
| C-04 | STILL-TRUE | **276,749 B / 3,283 lines — byte-identical.** T-2989 GO, no split executed, no follow-up task | `wc -c CLAUDE.md`; `.tasks/completed/T-2989-*` |
| C-05 | STILL-TRUE | **261 description literals / 105,516 B ≈ 26.4k tokens** — exact match. arc-005 still `in-progress`, `closed_at: null` | `.context/arcs/mcp-slimming.yaml` |
| C-06 | STILL-TRUE | **46,458 LOC, 1,403 fns, 382 commits/180d, coverage still 9.2%** (24/260) | `wc -l`; `check-mcp-parity-census.sh --json` |
| C-07 | STILL-TRUE | **20,435 LOC / 166 commits** — exact | `wc -l`; `git log --since` |
| C-08 | STILL-TRUE | cli.rs **7,001 L**, main.rs **1,931 L** — exact | `wc -l` |
| C-09 | STILL-TRUE | `let claimer = match param_str(params, "claimer")` — caller-supplied, unbound. **G-086/G-087 still `watching`/high, open 61 days** | `channel.rs:1725`; `.context/project/concerns.yaml` |
| C-10 | PARTIALLY-ADDRESSED | T-3001 GO: "premise holds but the pre-filled text mis-stated it". Nothing shipped; no follow-up task | `.tasks/completed/T-3001-*` |
| C-11 | STILL-TRUE Δ | **38 FIRING blocks** (was 36); today `FIRING — 1 unwakeable LIVE agent, RAIL DARK` | `.waker-liveness-canary.log` |
| C-12 | STILL-TRUE Δ | VERSION **0.11.2068**; PATH binary **0.11.1716**; hub **0.11.1766**. Still 3 binaries / 2 versions | `check-installed-binary-drift.sh` (exit 1) |
| C-13 | PARTIALLY-FIXED | T-2975 shipped **disclosure** (`SCOPE_NOTE`), **not** a predicate change — its own gate found no masked instance. `canary-status.sh:349` still sets HEALTHY in the log-older-than-heartbeat branch. **Verified live: `HEALTHY waker-liveness` immediately above `↳ FIRING — RAIL DARK`** | `canary-status.sh:76,349`; commit `5336c2cb3`; live `/canaries` |
| C-14 | STILL-TRUE Δ | `health:ring20-fedprobe` **1,857 records** (was ~1,669), `forever`, **41.1% of all bus traffic** (was 38.8%) | `channel list --json` |
| C-15 | STILL-TRUE | No rate/velocity trigger in either check; neither file modified since 09-19 | `check-{forever-archival,topic-growth}-freshness.sh` |
| C-16 | STILL-TRUE | **Blocked this review's own workers 3×**; counter read **5** after the refusals — blocked attempts increment it | live `PreToolUse` refusals; `.agent-dispatch-counter` |
| C-17 | STILL-TRUE (narrowed) | The subcommand **exists** and exits 1 correctly; the defect is that it emits **no usage** (`--help` → *Unknown option*). *(My first reading claimed exit 0 — that was pipe masking, corrected.)* | `fw termlink dispatch --help` |
| C-18 | STILL-TRUE | JSON still a single terminal `printf` (`:309-320`) — all-or-nothing. `CLAUDE.md:1861` still says "(seconds)" against **703 s** measured | `run-guard-layer.sh:309-320` |
| C-19 | STILL-TRUE | 7 non-empty canary logs, **no auto-filing anywhere**; T-2985 still carries placeholder ACs (`[First criterion]`) | grep sweep; `.tasks/active/T-2985-*` |
| C-20 | **FIXED** | Registered in `.vendor-divergence.yaml:80` + race-fixture harness | `.vendor-divergence.yaml` |
| C-21 | STILL-TRUE Δ | **141** human-owned active (was 137); **71** partial-complete; **median 127 d** (was 125); 56 >30 d; **45 >100 d** | per-file `last_update` census |
| C-22 | STILL-TRUE | Canary exits **1** with real unacked inbound filings **when it can reach the hub**; exits 2 blind otherwise (→ N-1) | `check-framework-pickup-freshness.sh` both envs |
| C-23 | STILL-TRUE | Observed live again this session (Phase 0/1 `/run` reads refused; `/root/.cargo/bin` out of bounds) | Phase 0/1 §6.6; sub-agent note |
| C-24 | PARTIALLY-TRUE | **3 of 6 claims still contradicted**: `FRAMEWORK.md` referenced at `:4` but **the file does not exist**; "(seconds)" at `:1861`; canary tally says "the seventeen"/"all eighteen" at `:692-693` vs **27 crontabs / 21 logs / 21 sections**. Two claims (`per session`, `api-usage --json`) are **not present in CLAUDE.md** at all | per-claim grep |
| C-25 | STILL-TRUE | `.context` **90 MB** excluding the stale index + `.tasks` **26 MB** vs **7.3 MB** product; handovers **61 MB / 1,911 files** (+205 in 2 days) | `du -sh` |
| C-26 | STILL-TRUE | **67 pairs exactly** (81 `_mcp` fns, 67 with same-named CLI twin). **No paired-commit check exists** | cross-match; `ls scripts/` |
| C-27 | STILL-TRUE (sharpened) | 0 calls/36.2 d — **but it is not dead code**: a 350-line handler, a dedicated `route_cache` module, and a live E2E driver at `tests/e2e/level8-orchestration-harness.sh:121`. The tripwire says it explicitly cannot cover this path | rpc-audit; T-2995 IW-4 |
| C-28 | STILL-TRUE | 0 calls; hub arm present, zero CLI/MCP surface | rpc-audit; T-2995 IW-1 |
| C-29 | STILL-TRUE | `artifact.get` **0**, `artifact.put` **0** over **36.2 d** — window still short of the 90 d the finding itself required before any verdict | rpc-audit |
| C-30 | STILL-TRUE (confirmed) | `kv.get`/`kv.set` 0 hub-observed; confirmed structural — one audit site only | rpc-audit; T-2995 IW-2 |
| C-31 | **NOW-WRONG** | Evidence inadmissible — session handler has no audit sink | T-2995 IW-2 |
| C-32 | **NOW-WRONG** | Evidence inadmissible — `SKIP_METHODS` excludes by design | T-2995 IW-2 |
| C-33 | **SUPERSEDED** | Cut 2026-05-31; hub returns `-32601` | T-2995 IW-3 |
| C-34 | STILL-TRUE | `channel.create` **54,201** vs `channel.post` **56,267** → ratio **0.96**; create is **5.5%** of 983,713 RPCs | rpc-audit histogram |
| C-35 | STILL-TRUE Δ | **86 of 168 GO inceptions unpropagated = 51.2%** (was 48.7%). See **N-2** | `.context/audits/go-scope-unpropagated/LATEST.md` |
| C-36 | STILL-TRUE | Claim family **151 calls / 36.2 d** (claim 83, release 23, claims 21, transfer 12, renew 12) = **0.015%** of all RPCs | rpc-audit |
| C-37 | STILL-TRUE + **INCOMPLETE** | 485 entries; focus **157**, sovereignty **152**, **AC-skip 120 never surfaced** (→ N-4) | `.gate-bypass-log.yaml` by `flag` |
| C-38 | STILL-TRUE Δ | **344 / 509 = 67.6%** edgeless (was 338/503); **all 6 new cards edgeless** (→ N-5) | card census |
| C-39 | STILL-TRUE Δ | `stale-waker-code` silent **39 d**, `stuck-claims` **36 d**; **0 ISO dates** across all 6 firing logs. **Root cause for the hub-dependent ones found: N-1** | `stat`; grep |
| C-40 | **NOW-WRONG** | Rail alive; counter blind (see 6b) | T-3004 |
| C-41 | STILL-TRUE | T-3005 closed with a 23-word rationale, **no per-hub sha, no floor change, no follow-up task**. `.122` sits at *exactly* its floor `0.11.1411` while VERSION is `0.11.2068`; canary exits 0 "healthy" | `.tasks/completed/T-3005-*`; `fleet-version-floors.conf` |
| C-42 | STILL-TRUE | Completions/month: Apr **609** → Jul 165 → Aug 279 → **Sep 79 in 21 days (~113/mo pace)**; commits **552/30 d** | `date_finished` census; `git log` |
| C-43 | **NOW-WRONG** | Framing over-reached (see 6b) | T-3007 |
| C-44 | STILL-TRUE + root cause | 12 passed / **1 failed**; fails its own *"the real tree passes"* assertion because **N-3** | `tests/cron-drift-firing-fixtures.sh` |
| C-45 | STILL-TRUE (reframed) | Writer shipped `c1c8165f6`; `invocation_audit.rs` + `invocation-usage.sh` present; **sink ABSENT in all 3 runtime dirs**. Per-tool usage **UNMEASURED**. T-2996's own commit message corrects the original framing: hub RPC telemetry already existed — *"the gap was resolution, not absence"* | `ls` × 3 dirs; `c1c8165f6` |

---

## 7. KEEP (names only)

The four charter verbs and their substrate; the guard layer as an institution (121 marked
members, 117 passing); the canary convention; `.vendor-divergence.yaml` and the G-062
file-upstream discipline; the arc/task ledger; TOFU + hub auth rotation tooling;
`substrate-smoke` / `comms-selftest` / `session-selftest` / `session-message-selftest` as
affirmative provers; offline queue + post idempotency; the claim lifecycle verb set
(**subject to C-09** — keep the verb, fix the guarantee).

---

## 8. INVESTIGATE — and the data each one needs

| ID | Question | Data that would decide it |
|---|---|---|
| N-2 | Is the GO-propagation leak a linking-hygiene artifact or genuinely dropped work? | Sample 20 of the 86: did shipped commits reference them? |
| N-4 | Are the 120 AC-skips sanctioned batch operations or a saturated completion gate? | Read reasons on a 20-entry sample |
| N-7 | How much of every "zero-call" reading is prober-contaminated? | A probe tag at source; re-run usage queries |
| C-27 | May `orchestrator.route` exist at all under non-goal #4? | **Human ruling** — evidence is complete (T-2995 IW-4) |
| C-29 | Is `artifact.*` unused or merely unexercised? | 90-day window (54 more days) — **do not verdict before then** |
| C-36 | Is verb-3 adoption roadmapped in AEF? | Unreachable from this repo (T-559) |
| C-38/N-5 | Are fabric verbs consulted at all? | Per-tool telemetry (C-45) |
| C-41 | Are `.121`/`.122` floors deliberate or stale? | Per-hub build sha — needs a session on those hosts |
| C-42 | Bigger tasks, or less work? | LOC-changed / sessions per completed task, by month |

---

## 9. Data gaps that capped confidence

1. **Per-tool / per-verb invocation telemetry — DESIGNED-ONLY.** One rebuild + MCP restart away. Unlocks C-01/IW-1, C-30, C-38, N-5, N-7. *Deferred to round 2 by human decision Q4; labelled, not estimated around.*
2. **No out-of-band observer of bus loss.** Structural; verb-2 durability remains self-reported.
3. **BVP realization log ABSENT** — no pre-registered expected effect from any prior review has ever been checked. This report adds 8 more predictions to an unverifiable pile.
4. **Line coverage ABSENT** — REFACTOR CHECK 2 cannot pass for C-06/C-07.
5. **Unused-dependency analysis ABSENT** — a whole DELETE axis (30 workspace deps) unexamined.
6. **CI run history ABSENT locally** — cannot tell whether the guard layer passes in CI or silently soft-times-out at the 10-min cap.
7. **`policy/capabilities.yaml` ABSENT.**
8. **75 guard scripts carry no `guard-layer:` marker** and were not run — 117 PASS is not a whole-surface green.
9. **Cross-project consumer visibility (IW-1) blocked by T-559** — blocks DELETE CHECK 5 for C-01/C-02.
10. **The context budget-gate's denominator is wrong for this model.** It halted this step at *"296,373 tokens — ~98% of context window"* while the session was running a 1M-context model with ~14.7M tokens of budget remaining. The gate is calibrated against a ~200k window. It is a Directive-#2 instrument defect of exactly the class in §0, and it fires on every long session this model runs. Recorded as **N-9**; no bypass was used.

---

## 9b. N-9 — budget gate halts on a wrong denominator

| Field | Value |
|---|---|
| **Item** | `budget-gate` PreToolUse hook computes % of context window against a fixed ~200k assumption |
| **Location** | `.agentic-framework` budget-gate hook (**vendored — G-062, file upstream only**) |
| **Class** | ADD (REPAIR) |
| **Reading** | **A — BROKEN** (works, wrong denominator) |
| **Evidence** | Fired at `SESSION WRAPPING UP (296373 tokens) … ~98% of context window` and blocked a `cat >>` append, while the harness reported ~14.7M tokens remaining on a 1M-context model. Measured live, this session. |
| **Counter-evidence** | The gate is *correct* on 200k-window models, which is most of them; and blocking early is the safe direction. |
| **Conf** | **HIGH** (measured live, unambiguous) |
| **Proposal** | Read the model's real window rather than assuming one; where it cannot be determined, say so instead of printing a percentage. File upstream. |
| **Size** | Small · **Reversible:** Yes · **Risk if wrong:** low (a too-generous gate loses uncommitted work — so fail toward the current behaviour) |
| **Expected effect** | Long sessions on 1M-context models stop being halted at ~20% of real budget |

---

## 10. Contradictions

1. **A healthy hub the client cannot reach.** systemd reports `termlink-hub` *active*, pid 21462 is serving, and `termlink channel list` says *Connection refused*. Both are true; they are about different sockets. (N-1)
2. **`/canaries` prints `HEALTHY` directly above `FIRING — RAIL DARK`** — the status word and the detail line disagree inside one output block. (C-13)
3. **A canary that cannot see reports identically to a canary that sees nothing wrong.** Exit 2 writes nothing to the findings log, and "empty log = healthy" then reads as an all-clear. The repo's own doctrine (T-2685) says a check that could not run must never read as a clean bill; here it does. (N-1)
4. **CLAUDE.md:4 points at `FRAMEWORK.md`, which does not exist in this repo.** (C-24)
5. **CLAUDE.md's two adjacent lines disagree with each other and with the tree** — `:692` "the seventeen canaries" / `:693` "all eighteen" vs 27 crontabs / 21 logs / 21 sections. (C-24)
6. **The remediation arc reproduces the defect it was built to remediate** — 8 of 9 of arc-009's GO inceptions leaked scope, and the audit's newest findings are arc-009's own slices. (N-2)
7. **T-3005 satisfied the completion gate while answering none of its questions** — 23-word rationale, no per-hub sha, no floor change, no follow-up task. The gate measures form, not substance.
8. **The one failing workspace test fails environmentally, not logically** — it asserts a git error and receives "Hub is not running". Greening it requires fixing the environment, not the code. (N-1)
9. **Two `value-drivers.yaml` exist** — project-authoritative vs vendored bootstrap template carrying *different* free drivers; quoting the wrong one silently changes the yardstick. (Phase 0/1 Y-2)
10. **`fw audit` is described as a compliance audit but the daily cron runs `sections: structure` only** — its 38 PASS speaks for one section.

---

## 11. Not reviewed

- **`.agentic-framework/` as a change target.** READ context only per human decision Q3; per G-062 every finding there is **file-upstream-only** (N-2, N-9, C-16, C-17, C-23). No local edit proposed for any.
- **All 13 BPMN workflows** — vendored, and still with **no machine-readable ratification marker**, so the prompt's DELETE-protection ("anything a ratified workflow references is not deletable") could not be mechanically applied to any item here.
- **75 unmarked guard scripts** (no `guard-layer:` marker, not run).
- **Dependency axis** — 30 workspace deps / 310 lockfile packages; no tooling installed.
- **Per-tool usage** — UNMEASURED by human decision Q4; MCP restart is round 2's first candidate action.
- **Cross-project / fleet consumers** — blocked by T-559.
- **R1S2 (procAsFit)** — not started; gated on this report.

---

## 12. Sovereign questions

Each carries a recommendation. **None is an agent's to decide**; none is resolved here.

| ID | Question | Recommendation |
|---|---|---|
| **SQ-R1-1** | **F-ORCH**: `fw audit` says its `retire_when` condition appears met, yet it carries weight 5 and shapes every ranking here. Retire, keep, or re-weight? | Decide before round 2, whose rankings inherit the same assumption. If F-ORCH retires, items resting on orchestration leverage (C-27, C-36, N-2) drop in rank; **nothing else in §6a moves** — N-1 and N-3 are D2-Reliability items and rank first under any weighting. |
| **SQ-R1-2** | The prior round's 12 sovereign questions were routed to **T-3012, now `work-completed` with a 30-word rationale and no follow-up task.** Were they actually decided? | Treat them as **still open**. A closed task is not a recorded decision. If they were decided, the decisions must live somewhere durable; if not, T-3012 closed prematurely and all 12 carry forward. |
| **SQ-R1-3** | Subtract the **28 off-charter analytics tools**, or amend the charter? | Unchanged: record the decision, discharge IW-1 via the invocation counter (C-45), then scope removal. **New:** C-01 step 1 is misframed — the record is not wrong, the **duplicate heading** is. Fix the heading; don't re-litigate. |
| **SQ-R1-4** | Execute deletion of the **46** deprecated tools? | The written gate in `p4-surface-reduction.md` makes this the human's. 48-day soak + zero callers + count now settled at 46; evidence is as complete as it gets without IW-1. |
| **SQ-R1-5** | Is **`--skip-acceptance-criteria` ×120** acceptable? P-010 is the completion gate; this was invisible to all five prior runs. | Do **not** change the gate on volume. Sample 20 with reasons first (N-4). Likely reading, as with C-21: the human-verification path is saturated — an ADD, not a gate relaxation. |
| **SQ-R1-6** | Does `orchestrator.route` conflict with **non-goal #4**? | Human ruling; evidence is complete (T-2995 IW-4: 350-line handler, `route_cache` module, live E2E driver, and a tripwire stating it cannot cover this path). The one INVESTIGATE row needing no further data. |
| **SQ-R1-7** | Should `fw inception decide … go` **require** a propagation target? (N-2) | Recommend yes, in framework code rather than agent discipline — but vendored, so a **file-upstream** decision; and a naive required field will be stub-filled (T-3005 is the worked example). |

---

## 13. Proposed tasks (arc-009 continuation) — NOT created

Per the prompt, tasks are proposed, not created. One deliverable each.

| # | Proposed task | IDs | Type | Owner | Why now |
|---|---|---|---|---|---|
| R1-1 | Set `TERMLINK_RUNTIME_DIR` in the 26 canary crontabs; re-run the 4 blinded canaries and triage what they surface | **N-1** | build | agent | Restores 4 detection channels and fixes the failing test; one line per crontab |
| R1-2 | Install the `substrate-smoke` crontab into `/etc/cron.d` | **N-3** | build | agent | Closes C-44 and the `fw audit` FAIL as consequences |
| R1-3 | Make the client prefer a live socket over a stale one (or fail loudly naming both dirs) | **N-1** | build | agent | The durable half of R1-1; without it the next host repeats it |
| R1-4 | Bypass-sample audit widened to `--skip-acceptance-criteria` (20 entries + reasons) | **N-4**, C-37 | inception | agent | Prerequisite for SQ-R1-5 |
| R1-5 | Fix the fabric card generator to populate edges or fail loudly | **N-5**, C-38 | build | agent | Backfill is pointless while the generator leaks |
| R1-6 | Implement `--json` in `check-installed-binary-drift.sh` or reject unknown flags | **N-6** | build | agent | Directive-#2 defect inside the guard layer itself |
| R1-7 | Tag prober/self-test traffic at source so usage queries can exclude it | **N-7** | build | agent | Protects every future usage verdict; cheap now |
| R1-8 | File upstream: `fw inception decide go` should require a propagation target | **N-2**, C-35 | build | agent | Vendored (G-062); structural fix for a 51.2% leak |
| R1-9 | Fix the duplicate `## Decision` / `## Decisions` heading in the task template | C-01 (D-1) | build | agent | Root cause of a disagreement that cost three prior runs |
| R1-10 | File upstream: budget-gate must read the model's real context window | **N-9** | build | agent | Halts long sessions at ~20% of real budget |
| R1-11 | Re-open or re-scope **T-3005** and **T-3012** — both closed without answering their questions | C-41, SQ-R1-2 | inception | **human** | Closing-without-answering is a governance call, not an agent's |

---

## 14. Expected effects (pre-registered predictions)

Recorded because **no prior review's predictions have ever been checked** — the realization
log is ABSENT (§9 gap 3). This report adds 8 more to that unverifiable pile, which is itself
an argument for building it.

| Item | Metric | Direction | Window |
|---|---|---|---|
| N-1 | Exit code of the 4 hub-dependent canaries under cron env | 2 → 0 or 1, never 2 | next cron cycle |
| N-1 | `cargo test --workspace` failures | 1 → 0 | immediate |
| N-3 | `fw audit` FAIL count; `cron-drift-firing-fixtures.sh` | 1 → 0; 12/13 → 13/13 | immediate |
| N-2 | GO-unpropagated ratio | 51.2% → <45% | 30 days of audits |
| N-5 | Edgeless share of *newly created* fabric cards | 100% → 0% | next 10 cards |
| N-7 | Prober-attributable RPCs excludable from usage queries | 0% → 100% | immediate |
| C-45 | Per-tool invocation records | 0 → >0 | first hour after MCP restart |
| C-13 | `/canaries` status word vs `↳` detail line | disagreement → agreement | next run |

---

## 15. What should change in how the next review runs

1. **Snapshot, then use `--no-heartbeat`.** I refreshed four heartbeats running canaries ad hoc (§2). Every canary here supports `--no-heartbeat` for exactly this reason.
2. **Check the instrument before believing the reading.** Four of 45 findings were instrument defects. A "zero calls" reading should be inadmissible until the audit path for that method is confirmed to exist — T-2995 made that check routine, and it belongs *before* a usage finding is filed, not after.
3. **Verify arc state before scoping a rescope.** This step was scoped on "3 of 18 complete"; the truth was 12 of 38.
4. **Count the denominator from the mapping table, not the first slice range** — the same error produced both of the above, and N-9 is the same mistake made by a hook.
5. **A completed task is not a recorded decision.** T-3005, T-3011 and T-3012 closed with rationales too thin to act on. The next review should read Decision *substance*, not completion status — and so should the audit.

---

## Appendix — provenance

- **Inputs read in full:** `docs/prompts/value-review.md`; `.context/runs/T-3044-sequence.yaml` (`human_decision`, `sovereign_question.resolution`); `docs/reports/VALUE-REVIEW-repo-2026-09-21-phase01.md`; `docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md`.
- **Measurement commands** cited inline per row; all read-only. No topic was posted to, no hub or MCP server restarted, no file outside `docs/reports/` and `.context/runs/` modified.
- **Sub-agent gathering:** 2 workers (MCP-surface cluster; canary/guard cluster). 3 further workers were refused by the dispatch gate (§3); no gate was bypassed or self-approved.
- **Halt:** Phase 5 complete. **PHASE 6 NOT ENTERED.** No item executed.
