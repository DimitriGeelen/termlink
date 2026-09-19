# Project Value Review — TermLink (whole repo)

- **Scope:** whole repo (`/opt/termlink`)
- **Date:** 2026-09-19
- **Run:** run5 of a 5-run consecutive repeatability series. Independent — no prior run's reports were read.
- **Evidence file:** [`VALUE-REVIEW-repo-2026-09-19-run5-evidence.md`](VALUE-REVIEW-repo-2026-09-19-run5-evidence.md) (59 evidence rows + 9 non-use diagnoses)
- **Task:** T-2971
- **Phases executed:** 0–5. **Phases 6 and 7 were NOT executed** — no human approval exists in this headless run. Nothing was deleted, restructured, or built.

> **Research is not authorization.** Every proposal below awaits item-by-item human approval.

---

## 0. Headless-run deviations and the [ASK] gates

This run had no human available. Per the harness authorization, at each `[ASK]`
gate I recorded verbatim what I would have asked, adopted a provisional answer,
marked it **UNCONFIRMED**, and continued.

### [ASK] gate 1 — Phase 1, yardstick + data map

> *Verbatim, as it would have been put to the human:*
>
> 1. "I've taken TermLink's purpose from `docs/CHARTER.md` — the canonical sentence,
>    the four verbs (discover / exchange durable messages / claim work / control
>    terminal sessions), and the five non-goals. The charter says it is human-blessed
>    and authoritative. **Is this still the yardstick I should judge value against,
>    or has it drifted from what you actually want TermLink to be?**"
> 2. "The data availability map is below. The load-bearing gaps are: **no MCP
>    per-tool telemetry**, **no CLI per-verb telemetry**, **no out-of-band observer
>    of message delivery**, **no BPMN/workflow determinism map**, and
>    `cargo-udeps`/`cargo-machete` not installed. **Does data exist that I cannot
>    see** — production logs, an issue tracker, consumer-project usage, or fleet-wide
>    telemetry from the other four hubs?"
> 3. "The hub RPC audit covers **only this hub** (`.107`/localhost) and **only
>    hub-routed calls**. Four other hubs exist. **May I probe them, or should I treat
>    this as a single-hub sample?**"

**Provisional answers adopted (UNCONFIRMED):** charter is the yardstick; no external
data beyond the repo and the local hub; single-hub sample, no remote probing beyond
the read-only `fleet doctor` call already made.

**What this costs:** every usage-based judgement is scoped to one hub over 34.2 days.
This is stated inline wherever it matters and is the single largest limit on this review.

### [ASK] gate 2 — Phase 5, the report itself

The Phase 5 `[ASK]` ("Present summary and questions. Human approves / rejects /
modifies per item") **is this document**. No decisions were recorded, no items
approved. Section 12 carries the sovereign questions that must not be answered by
an agent at all.

---

## 1. Yardstick (PROVISIONAL — UNCONFIRMED)

From `docs/CHARTER.md`, which states it is human-blessed and authoritative.

**Purpose:** TermLink is a hub-mediated, durable append-log message bus with terminal
endpoints — the coordination substrate that lets a fleet of AI agents (and humans)
**discover each other**, **exchange durable messages**, **claim work**, and **control
terminal sessions** across one or many machines.

**Users/consumers:** AI agents in a multi-host fleet; humans operating that fleet;
the Agentic Engineering Framework (AEF) as the policy layer above the substrate;
peer projects filing on `framework:pickup`.

**Non-goals:** (1) not inter-hub federation · (2) not a durable database / system of
record · (3) not a social/engagement platform · (4) not a workflow/orchestration
engine · (5) not a security boundary between mutually-distrusting tenants.

**Contradictions between stated purpose and actual code** — carried into §10.

---

## 2. Data availability map (PROVISIONAL — UNCONFIRMED) + snapshot windows

Full map in the evidence file. Summary:

| Source | Status | Window |
|---|---|---|
| Hub RPC audit (`/var/lib/termlink/rpc-audit.jsonl`) | **EXISTS** — 921,543 calls, 0 malformed | **34.2 days** (2026-08-16T14:53Z → 2026-09-19T18:55Z) |
| Canary logs + heartbeats | EXISTS | up to 75 days |
| Task ledger / concerns / audits / episodic / handovers | EXISTS | project lifetime |
| Topic state, fleet hub versions | EXISTS | point-in-time |
| Git churn | EXISTS | 180 days |
| **MCP per-tool invocation telemetry** | **ABSENT** | — |
| **CLI per-verb invocation telemetry** | **ABSENT** | — |
| **Out-of-band delivery observer** | **ABSENT** (on-demand prover only) | — |
| **Workflow/BPMN determinism map** | **ABSENT** | — |
| `cargo-udeps` / `cargo-machete` | **ABSENT** | — |

**Snapshot discipline.** All usage data was copied at **2026-09-19T18:55:23Z**, before
any check was run, and judged from the snapshot. This mattered: see F-01, where
separating pre-existing heartbeat touches from my own footprint was the difference
between a real finding and a self-inflicted artifact.

**A channel cannot report its own failures.** Delivery evidence here is largely
bus-self-reported. The one out-of-band prover (`session-message-selftest.sh`) is
on-demand and unscheduled. Recorded as a data gap, not a clean bill of health.

---

## 3. Role setup

| Role | How it ran |
|---|---|
| **GATHERER** | This session — Claude **Opus 5**. Read-only collection, Phases 0–3. |
| **JUDGE** | **Separate worker**, `fw termlink dispatch --name judge-run5`, running **Claude Sonnet** — a **different model family**. Its only input was the evidence file; its transcript confirms it read that one file and nothing else. Output: `docs/reports/.judge-run5-verdicts.md` (41.8 KB). |
| **HUMAN** | **Absent.** No approvals given. Phases 6–7 not executed. |

**Separation achieved**, so the prompt's `-1` confidence penalty is **not** applied to
the JUDGE's findings (F-01…F-22 below).

**One exception, explicitly marked.** Evidence rows E-49…E-59 and F-23 were gathered
*after* the JUDGE had already read the evidence file. Those are **self-judged** and
carry a **one-level confidence reduction**, except where they merely corroborate a row
the JUDGE already saw.

A note on how this separation was obtained: the `Agent` tool was blocked by the T-533
dispatch gate on this session's first attempt (stale cross-session counter). I did
**not** self-approve or reset that gate — in a headless run, an agent clearing its own
governance counter is precisely what the autonomy rules forbid. I used the sanctioned
path the gate itself names (`fw termlink dispatch`), which produced better separation
than the blocked tool would have.

---

## 4. Baseline

| Check | Result |
|---|---|
| `cargo test --workspace` | **PASS** (exit 0) |
| `cargo build --workspace` | **PASS — zero warnings**; only 4 `#[allow(dead_code)]` repo-wide |
| `scripts/run-guard-layer.sh` | **COMPLETED** (text mode): **109 PASS / 4 FAIL / 0 ERROR** across 113 members — `guard layer: FIRING — 4 guard(s) found something`. FAILs: `check-installed-binary-drift.sh`, `check-pickup-deferred-freshness.sh`, `check-receiver-ack-lag.sh`, `cron-drift-firing-fixtures.sh`. (`cargo test --workspace` not included — needs `--tests`.) The `--json` form separately emitted **0 bytes** when cut off at 600 s (F-06). |
| `fw audit` (2026-09-19) | 39 pass / 8 warn / **1 fail** — a stable, non-zero failing steady state |
| `scripts/check-cron-install-drift.sh` | `ok:false` — 1 missing crontab (already tracked, T-2939) |
| `scripts/arc-live-probe.sh` | **SHIPPED-BUT-NOT-LIVE** — hub serves 0.11.1766 vs floor 0.11.1944 |

---

## 5. Summary

**Counts:** ADD **6** · REFACTOR **3** · INVESTIGATE **14** · **DELETE 0** · KEEP 13
*(plus 1 self-judged post-cutoff finding, F-23 → total 24 non-KEEP items)*

**Why DELETE is zero.** This is the review's main negative result and it is deliberate.
Every candidate that looked deletable failed a DELETE CHECK on a *positive* reason:
`orchestrator.route` and `dialog.presence` are unwired but carry live intent evidence
(a tripwire test; a documented in-code purpose); the 28 off-charter analytics tools have
an agent recommendation to subtract but **no recorded human decision**; `event.broadcast`
carries an explicit in-code retirement declaration and is the closest thing to a clean
removal in the repo, but its full reference sweep was never run. **Absence of use is
never a reason to delete**, and in a codebase this well-instrumented, absence of use
turned out to mean "unmeasurable" far more often than "unwanted".

### Top 3 per axis

**DELETE** — *none reached the bar.* The three nearest misses, ranked by how close:
1. `event.broadcast` — explicit `LEGACY_METHODS` retirement declaration + 0 calls; needs only a reference sweep to convert (F-18).
2. The 28 off-charter analytics MCP tools — charter non-goal #3 + recorded GO-to-subtract recommendation; blocked on a sovereign decision, not on evidence (F-19, §12 Q1).
3. `dialog.presence` / `orchestrator.route` — unwired and unreachable, but intent evidence unexplained (F-12, F-13).

**REFACTOR**
1. **`tools.rs` — 46,458 lines in one file, 390 commits/180 d (#1 churn), 9.2% parity coverage** (F-07). Do *not* split it yet: coverage is the gating fact.
2. **`cli/src/commands/channel.rs` — 20,435 lines, 166 commits/180 d** (F-08); worth splitting *jointly* with `tools.rs`, since they wrap the same `channel.*` surface.
3. `cli.rs` (7,001 L/356 commits) + `main.rs` (1,931 L/351 commits) — secondary; investigate churn cause before acting (F-09).

**ADD**
1. **Fix the `/canaries` FIRING→HEALTHY masking bug** (F-01) — one predicate fix restores trust in all 30 canaries at once.
2. **Restart the hub onto the current binary** (F-02) — 75 days of daily WARNs, three independent sources agreeing, pure operator remediation.
3. **Re-arm the push-wake rail** (F-03) — 36 firings, charter verb 2 degraded to the poll floor fleet-wide.

---

## 6. Findings

Ranked within axis by value per unit of cost and risk. **Confidence is the JUDGE's**
unless marked *self-judged*. Full reasoning per item in `.judge-run5-verdicts.md`.

| ID | Item | Location | Class | Non-use reading | Evidence | Counter-evidence | Conf. | Proposal | Size | Revers.? | Risk if wrong | Expected effect |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **F-01** | `/canaries` masks genuinely FIRING canaries as HEALTHY | `scripts/canary-status.sh:20-21` | **ADD(REPAIR)** | — | E-22, E-23: predicate is `FIRING = log non-empty AND log_mtime >= heartbeat_mtime`. An **ad-hoc run refreshes the heartbeat without appending**, flipping FIRING→HEALTHY. Measured: substrate-preflight log 09-19 **05:23** vs heartbeat 09-19 **20:46**; same at 20:13 for pickup/waker/stale-waker. `/canaries` reported `firing:0` the same day two of those logs were appended to | None. E-23 timestamps these touches **9–42 min before this session began**, ruling out my own footprint | **HIGH** — mtimes on both sides + predicate read from source; two independent measured facts agree | Key FIRING on content newer than the last **acknowledged/cleared** marker, not newer-than-heartbeat. A heartbeat touch must never by itself clear a fire. Pin with a fixture | Small | Yes | Could over-fire on heartbeat-only refreshes — mitigate by changing only the *clear* condition, not the *fire* condition | `/canaries firing` count tracks non-empty-log count exactly; watch 7 daily runs |
| **F-02** | Binary staleness firing **75 days** with no remediation | `.substrate-preflight-canary.log` | **ADD(REPAIR)** | — | E-21: 74 dated entries, `[WARN] binary` 71×, `[WARN] hub-binary` 26×. E-25: repo's own `arc-live-probe.sh` returns **SHIPPED-BUT-NOT-LIVE** (0.11.1766 vs floor 0.11.1944). E-20: the charter-drift guard itself audits the **stale** binary | None — this is the exact G-069 class the project built five tools to catch, and all five agree | **HIGH** — three independently measured sources converge | Restart the hub onto the current binary via the documented systemd path (G-070); re-run the probe. Then root-cause why 75 daily WARNs produced no action — likely F-01 hid them | Small | Yes | If the version is pinned for an unstated reason a restart could disrupt; no such reason appears anywhere in evidence | `arc-live-probe.sh` exits 0 within one restart cycle; `[WARN] binary` lines stop |
| **F-03** | Push-wake rail **dark** — 0 live wakers, verb 2 degraded fleet-wide | `.waker-liveness-canary.log` | **ADD(REPAIR)** | — | E-24: **36 FIRING** entries; latest: "ZERO LIVE listeners carry pty_session… Every DM sent here waits on the ~15s poll floor at best, **forever at worst**"; named instance `penelope`, presence-advertised but unwakeable | Reads "healthy" in `/canaries` only because of F-01 — live defect, not stale noise | **HIGH** — 36 dated entries + named current instance + explicit mechanism | Relaunch affected agents through `scripts/tl-claude.sh start --reachable …` (running headless agents cannot be retrofitted — arm at relaunch) | Small per agent, fleet-wide | Yes | Low — brief presence gap during relaunch, which DM durability already tolerates | 0 new FIRING entries for 7 days; `pty_session` present on relaunched agents |
| **F-04** | Hook-counter corruption feeding a decay alarm with **false silence** | `.hook-counter-integrity-canary.log`, vendored `lib/hook-telemetry.sh` | **ADD(REPAIR)** | — | E-37: FIRING, duplicate key `check-arc-id`, two readers of the same file disagree (**1 vs 7**). Mechanism: unlocked truncate+write, labelled "**L-023 recurring**". E-38: this counter is the **denominator** of the T-1626 decay alarm, so duplication "roughly halves the apparent failure ratio" | None | **HIGH** — measured firing log, explicit cited mechanism, concrete reader disagreement | `flock` the read-modify-write in `_fw_telemetry_increment`. **Vendored code (G-062)** — file upstream rather than patch locally, or the fix dies at the next re-vendor | Small | Yes | Low — negligible contention on a low-frequency counter | Readers agree; canary stops firing |
| **F-05** | **11 unacknowledged inbound peer filings** on `framework:pickup` | `.framework-pickup-canary.log` | **ADD(REPAIR + WIRE)** | **B** (no automatic consumer) | E-28: offsets 106–120 — P-064…P-076 = **4 bug reports, 2 feature proposals, 5 learnings**. E-29: literal recurrence of **G-063**, open **105 days** | Footer confirms 10 self-filings correctly excluded; the 11 are genuinely inbound | **HIGH** — measured log with offsets + categories, corroborated by a 105-day concern | Two owners: (1) triage + ack the 11 now (cheap, bounded); (2) G-063's structural consumer is **new-subsystem territory** — inception-size it, don't build ad hoc | Small / Large | Yes | Low for triage; scope creep for the consumer if built without inception | Pickup canary clear of these 11 after ack |
| **F-06** *(corrected post-JUDGE — see note)* | Guard-layer `--json` is all-or-nothing, and the layer far exceeds its documented budget | `scripts/run-guard-layer.sh` | **ADD(REPAIR)** | — | `--json` run cut off at 600 s produced a **0-byte** file — no partial verdicts recoverable, because JSON is emitted only at the end. Documented budget is "**seconds**" for the source tier; the layer is now **113 members**. Runs in CI on every push/PR | **The runner does NOT hang.** A later text-mode run **completed normally**, exiting 1 on its own FIRING verdict (109 PASS / 4 FAIL / 0 ERROR). So this is a budget-and-output-format defect, not a deadlock | **HIGH** *(raised from the JUDGE's MEDIUM: a second, completing run resolved the reproducibility question the JUDGE correctly flagged)* | Stream/flush per-member results so an interrupted run still yields partial output; and reconcile the "seconds" claim in CLAUDE.md with a 113-member layer | Small | Yes — incremental writer is strictly additive | Low | `--json` yields usable partial output when interrupted; documented budget matches measured runtime |
| **F-07** | `tools.rs` — 46,458-line #1-churn monolith at **9.2%** parity coverage | `crates/termlink-mcp/src/tools.rs` | **REFACTOR** | — | E-14: 46,458 L, **390 commits/180 d** (highest in repo). E-13: only 24 of 260 tools parity-asserted | E-42: build is clean — this is a maintainability cost, not a correctness alarm today | **HIGH** (cost) / **MEDIUM** (readiness) | **Do not split yet — REFACTOR CHECK 2 fails at 9.2%.** Sequence: (a) raise parity coverage via the existing T-2748 ratchet on the highest-churn portions, (b) *then* split along tool-category boundaries | Large, gated behind Medium | Partially | **High if the sequence is skipped** — a churn-heavy, low-coverage file is exactly where refactors introduce silent regressions | Churn spreads off the single file; coverage % rises *before* any split lands |
| **F-08** | `commands/channel.rs` — 20,435 L, 166 commits/180 d | `crates/termlink-cli/src/commands/channel.rs` | **REFACTOR** | — | E-15; `cli`+`mcp` = **68% of all code** (E-16) | None | **HIGH** | Same gating caveat as F-07. Because it backs the same `channel.*` surface the MCP layer wraps, a **coordinated** split (CLI groups mirroring MCP categories) beats splitting either alone — treat as one task | Large | Partially | Medium | Churn spreads across resulting files |
| **F-09** | `cli.rs` + `main.rs` secondary churn hotspots | `crates/termlink-cli/src/` | **REFACTOR** | — | E-15: 7,001 L/356 commits; 1,931 L/351 commits | Could be legitimate high-frequency low-risk edits (subcommand registrations) | **MEDIUM** | Lower priority. Break down churn cause (one-line registrations vs substantive logic) before any structural work | Small (investigation) | N/A | Low | N/A until investigated |
| **F-10** | Project-boundary gate false-positives on the project's **own** hub runtime dir | `check-project-boundary` hook | **ADD(REPAIR)** | — | E-45: read-only `cd` into `/var/lib/termlink` blocked as "outside project root"; the identical read succeeded via absolute path | None | **MEDIUM** — single instance, but the inconsistency is demonstrated | Normalise `cd`-based and absolute-path access through the same check, or recognise the declared hub `runtime_dir` as in-bounds **for reads** | Small | Yes | Low — narrowing a false positive does **not** weaken the boundary itself | No more false blocks on read-only access to declared runtime paths |
| **F-11** | Agent-dispatch gate blocked by a **stale cross-session** counter | dispatch gate (T-533) | **ADD(REPAIR)** | — | E-46: first dispatch of a fresh session blocked ("exceeds limit (2)"); counter carried over from a prior session | None | **MEDIUM** — single instance | Scope the counter to session lifetime so a new session starts at 0 without a manual reset | Small | Yes | Low — changes *when* the counter resets, not the limit; policy intent preserved | New sessions don't need `fw dispatch reset` |
| **F-12** | `orchestrator.route` — hub-implemented, **zero** client surface, zero calls | `hub/src/router.rs:78,1186` | **INVESTIGATE** | **B** + weak **E** | E-07: dispatched in hub; **0** CLI and **0** MCP references — no first-party caller can reach it. D ruled out (hub-routed) | `no_federation_tripwire.rs:43,177` treats it as a live residual path — **unexplained intent evidence blocks DELETE** | **MEDIUM** | Determine from task/design history whether this is shelved scaffolding or deliberate hub-only infrastructure. If shelved → DELETE becomes runnable; if wanted → ADD(WIRE) | Small (investigation) | N/A | Deleting code the tripwire needs, or wiring a surface nobody wants | N/A until investigated |
| **F-13** | `dialog.presence` — hub-implemented, zero client surface, zero calls | `hub/src/router.rs:140`, `channel.rs:28` | **INVESTIGATE** | **B** | E-08: implemented (T-1286/T-243), documented purpose, traces to verb 2, but **0** CLI/MCP surface. D ruled out | The in-code purpose statement is itself intent evidence | **MEDIUM** | Check T-1286/T-243 history; if the purpose is now served elsewhere → DELETE, else a **small** ADD(WIRE) — the tracker already exists | Small | Yes if WIRE | Low — inert code, no callers | N/A until investigated |
| **F-14** | `artifact.get`/`put` — fully wired, hub-visible, **still zero calls** | hub + cli + mcp | **INVESTIGATE** | none affirmatively established (N-4) | E-06/N-4: wired and discoverable, hub-routed (so **not** subject to the audit blind spot), yet genuinely 0 calls in 34.2 d | None | **MEDIUM** — the zero is measured; the *reason* is not established | **Extend the window before concluding anything** — 34.2 d may be too short for a rare artifact-exchange path. No code action | N/A | N/A | Deleting a fully-built capability on too short a sample | Re-check at 90+ days |
| **F-15** | `kv.*` usage **structurally invisible** to the only measurement source | hub/cli/mcp | **INVESTIGATE + ADD(instrument)** | **D** — decisive | E-09: `kv.get` has **0 hub implementation**; CLI verb + 5 MCP tools exist. E-12: the audit fires from two sites in `hub/src/server.rs`; session-daemon calls never pass through | None needed — D follows from the wiring itself | **HIGH** (for the diagnosis; **no** claim about actual kv volume) | **No value verdict is possible until instrumented.** Add a session-daemon counter, or route `kv.*` through the shared audit sink | Small→Medium | Yes | Low | A future review can classify `kv.*` on real data instead of a blind spot |
| **F-16** | `session.*` lifecycle methods — zero hub-observed calls | E-06 | **INVESTIGATE + ADD(instrument)** | **D** (strong by analogy to F-15) | Same structural blind spot as `kv.*` plausibly applies; the per-method wiring check was **not** run for this group | None | **MEDIUM** — inferred for this group, not individually measured | Run the same wiring check used for N-1…N-4 before drawing any conclusion | Small | N/A | Low | Confirms or refutes the D reading |
| **F-17** | `event.emit/subscribe/topics/state_change/error` — zero calls despite CLI/MCP surface | E-06 | **INVESTIGATE** | **unclear** | `event.subscribe` has hub=3/cli=4/mcp=1 — it **is** hub-implemented yet still zero-call, which argues against a pure D reading, unlike kv/session | Sibling `event.broadcast` has a direct E reading — the family may not share one fate | **LOW** — conflicting signal, no per-member investigation | Per-method `LEGACY_METHODS` + wiring check on each member before judging | Small | N/A | Low | Resolves retirement-family vs underused-but-wanted |
| **F-18** | `event.broadcast` — **recorded intent to retire**, zero calls | `hub/src/rpc_audit.rs:54-58` | **INVESTIGATE (DELETE-leaning)** | **E** — direct evidence | E-10: listed in `LEGACY_METHODS`, "targeted for retirement" by the T-1166 entry gate; 0 calls. **The strongest E-reading in this review** | DELETE CHECKS **4 and 5** (references in strings/config/hooks/CI/prompts; external consumers) were **not** verified — only legacy-list membership is established | **MEDIUM** — the reading is strong; the checklist cannot be certified from this evidence alone | Run a targeted reference sweep; if clean this converts to **DELETE** with high confidence — the closest any item comes to a clean removal. State the revert plan (git revert) in that follow-up | Small | Yes (git revert) | Low — the project already declared the retirement | Sweep confirms zero references; method removed, `LEGACY_METHODS` entry dropped |
| **F-19** | 28 off-charter analytics MCP tools — GO **recommended**, no **recorded** decision, no removal task | `.context/checks/charter-drift-allowlist`, T-2548 | **INVESTIGATE** | **D** (unmeasurable) stacked with **E** (non-goal #3 + recorded GO-to-subtract) | E-17: 28 tools, every allowlist line reading "traces to no charter verb". E-18: T-2548 is `work-completed` **30 days ago**, body records "Agent recommendation = **GO to subtract**", but `## Decision` is **empty**. E-19: **no** follow-on removal task exists anywhere | The project's own rules put sovereignty over exactly this call with the human; **an agent recommendation is not authorization.** The empty Decision section is not an oversight to route around — it *is* the state of the decision | **HIGH** on the facts; the **value verdict cannot exceed LOW** because it is gated on an unrecorded human decision, not on more evidence | **Not a proposal to delete** (see §12 Q1). The non-sovereign step is procedural: T-2548 must not read as a closed GO with an empty Decision — either record the decision or re-flag the task as pending | Small (process); Medium if removal is later approved | Yes | Treating a recommendation as a ratified decision, **or** leaving it silently unresolved, equally misrepresent project state | A recorded decision ends an ambiguity that reviews keep re-discovering |
| **F-20** | `channel.create` ≈ `channel.post` (0.98 ratio) against 45 stable topics | E-05 | **INVESTIGATE** | — (cost question, not use) | E-05: **52,695** creates vs 53,538 posts, against only **45** topics that exist — 5.7% of all hub RPCs | May be an intentional cheap ensure-topic convention with negligible hub-side cost; per-call cost was not measured | **LOW** — ratio measured precisely, interpretation not | Measure the hub-side cost of a create on an existing topic; if non-trivial, cache "topic known to exist" client-side | Small | N/A | Low | `channel.create` share of RPC volume drops |
| **F-21** | `health:ring20-fedprobe` — largest topic (1,672 rec), `forever` retention, readership unknown | E-11, N-9 | **INVESTIGATE** | **D** | Largest topic on the hub, `forever`, actively growing; below the 50,000 archival ceiling so nothing fires; **not** on the operator-durable exclusion list | No evidence it is unwanted — it is the most actively written topic | **LOW** — growth trend and readership both unmeasured | Extract subscriber cursors before judging retention | Small | N/A | Low — no urgency, but trend unknown | Enables a data-based retention verdict |
| **F-22** | G-008 concern text is **stale relative to its own subject** (64 → 71) | E-31, E-32 | **INVESTIGATE** | — (register hygiene) | G-008 recorded 64 partial-complete tasks **157 days** ago; measured **71** today. The defect-class check reports `firing_count: 0` — these are legitimate T-193 partial-completes, growing | The check that would catch a *defect* here is clean — backlog growth, not correctness failure | **HIGH** on the fact (two measurements 157 d apart) | Governance, not code: whether human-AC triage needs a cadence is an operational judgement | N/A | N/A | N/A | Track the count each review to see if it stabilises |
| **F-23** *(self-judged, post-cutoff)* | **Half of GO inceptions never propagate scope into build tasks** | `.context/audits/*.yaml` | **INVESTIGATE** | — | E-51: "**77 GO-scope-not-propagated inception(s) of 158** GO-recorded completed inceptions" = **48.7%**; recurs in ≥11 of the last 30 audits. **F-19's T-2548 is one instance of this systemic pattern, not an isolated lapse** | The audit detects and reports it daily — the framework is not blind, it is unheeded | **MEDIUM** *(measured, but reduced one level: gathered after the JUDGE's read, so self-judged)* | Treat the inception→build handoff as a process defect in its own right, not 77 individual oversights. Root-cause why a detected, daily-reported 48.7% leak produces no action — the same "reported but unheeded" shape as F-01/F-02 | Medium (process) | N/A | Low to investigate; high to keep ignoring — it silently drops half of all approved work | The ratio falls at subsequent audits |

### Post-cutoff corroborations (self-judged; they strengthen, not replace, the above)

- **E-49** — **three** installed binaries at **two** versions (`/root/.cargo/bin` 0.11.1766; `/root/.local/bin` and `/usr/local/bin` 0.11.1716) vs repo 0.11.1944. Which one runs depends on `PATH` order. Third independent source for **F-02**, and sharper than "stale": the install state is *ambiguous*.
- **E-50** — 1 **STRANDED** pickup envelope (`P-078-learning.yaml`, 9 days, no breadcrumb → can never be promoted). Corroborates **F-05**; already tracked as T-2960.
- **E-57** — the 28 off-charter tools are **13,364 chars ≈ 3,341 tokens = 12.7%** of the **26,378-token** MCP description payload loaded into **every agent context every session**. This converts **F-19** from a tidiness question into a measurable recurring cost, and supplies its expected effect: **removal should reclaim ~3.3k tokens per agent per session.**
- **E-56** — a correction I owe arc-005: the audit's 18/30 "no task commits in 30 days" WARN reads like abandonment, but the slimming work **largely landed** (156 KB → 105 KB; worst description 11,751 → 1,546 chars; >1000 chars: 24 → 1). The arc is **bookkeeping-stale, not rotting**. I would have mis-reported this had I not measured it.
- **E-54** — **338 of 503 (67%)** component-fabric cards have **no edges**. CLAUDE.md instructs agents to run `fw fabric blast-radius` before committing and `fw fabric deps` before modifying a file; for two-thirds of registered components those verbs can return no downstream information.
- **Guard FAIL `check-receiver-ack-lag.sh`** — on `agent-chat-arc`, **4 of 5 distinct identities have NEVER acked**, lag **1,533**. Verb 2's write side works; the read side is largely unconsumed. Same write-only-sink shape as G-063, on the fleet's main broadcast topic. *(The check warns its rows are keyed by identity fingerprint, so where agents share a keypair this measures a host, not an agent — T-2838.)*
- **Guard FAIL `cron-drift-firing-fixtures.sh`** — a **fixture suite** failing, i.e. a guard's own regression pins are red. Not investigated further; flagged because a failing fixture suite is the one failure class that can silently degrade every other guard's reliability.
- **Guard layer completed** at **109 PASS / 4 FAIL / 0 ERROR** over **113 members**, self-reporting `guard layer: FIRING — 4 guard(s) found something`. Note `cargo test --workspace` is **not** included unless `--tests` is passed — so the CI guard job and the test suite are separate gates.

---

## 7. KEEP

Charter verb 2 core path (`channel.subscribe`/`post`/`hub.auth`/`session.discover`) ·
Charter verb 3 full claim lifecycle · Charter verb 4 (`command.execute`/`inject`) ·
`no_federation_tripwire.rs` · guard-layer marker/fixture mechanism · charter-drift
canary + allowlist ratchet · MCP parity census ratchet (T-2748) · `cargo build`/`test`
hygiene · zero orphan scripts · cron-install-drift check · stranded-finalized-tasks
check · topic-growth / forever-archival canary mechanism · gate-bypass log as an audit
trail.

**On verb 3 ("claim work") specifically.** Its write-side lifecycle saw **108 calls in
34.2 days — 0.012%** of hub traffic, which is the single most tempting DELETE signal in
this review. It is **KEEP**, and the reasoning is the prompt's own: readings A–D are all
ruled out (it is built, wired, discoverable, and precisely measured), no decision
abandons it, and it is **named in the charter's canonical sentence**. Low volume is what
a work-stealing primitive *should* look like when contention is rare. Use is not value,
and non-use is not a verdict.

---

## 8. INVESTIGATE — and the data each needs

| Item | Data needed |
|---|---|
| F-12 `orchestrator.route` | Task/design history: shelved scaffolding, or deliberate tripwire infrastructure? |
| F-13 `dialog.presence` | T-1286/T-243 history: why was no consumer built? |
| F-14 `artifact.*` | Re-measure over **90+ days** before any verdict |
| F-15 `kv.*` | A session-daemon-local counter (or route through the shared audit sink) |
| F-16 `session.*` | Per-method hub/cli/mcp wiring check (as run for N-1…N-4) |
| F-17 `event.*` | Per-method `LEGACY_METHODS` + wiring check |
| F-18 `event.broadcast` | **Full reference sweep** (strings/config/hooks/CI/prompts) to complete DELETE CHECKS 4–5 |
| F-19 28 tools | Whether a human decision was ever actually made (§12 Q1) |
| F-20 create/post ratio | Hub-side cost of a create on an existing topic |
| F-21 `health:ring20-fedprobe` | Subscriber cursor/offset data |
| F-22 G-008 | None — the open question is a governance decision on triage cadence |
| `auth.token`, `query.*`, `orchestrator.bypass_*` | None received the per-method wiring check; needed before any reading |
| `channel.force_release`, `channel.set_retention` | Same check; likely rare-by-design (Tier-0/operator) but that is inferred, not established |
| Fleet hubs `.121`/`.122` | Actual build sha per hub — patch numbers are **not** comparable across tag epochs (E-27) |

---

## 9. Data gaps that capped confidence

1. **No MCP per-tool and no CLI per-verb telemetry.** Caps every judgement about 262 MCP tools and 41 CLI verbs to hub-RPC-visible traffic, which explicitly excludes session-scoped surfaces. **Closing it unlocks F-15, F-16, and the D-component of F-19** — i.e. it is the prerequisite for ever deciding the 28-tool question on evidence rather than on charter reasoning alone. *This is the highest-value ADD in the review that nobody has asked for.*
2. **Single-hub, 34.2-day sample.** Four other hubs unprobed ([ASK] gate 1).
3. **No out-of-band delivery observer.** Verb 2's durability claim rests on bus-self-reported receipts except when an on-demand prover is manually run — a channel reporting on itself.
4. **Subscriber cursors per topic not extracted.** Caps F-21 and any "topic usage" claim beyond raw counts.
5. **Unused-dependency analysis not run** (`cargo-udeps`/`cargo-machete` absent). Caps any claim about the 30 workspace dependencies.
6. **Audit-history trend analysis** across 151 audit files not performed — would turn F-22 and F-23 from point measurements into trend lines.

---

## 10. Contradictions

1. **`/canaries` says `firing: 0` while four canary logs carry current findings** — two appended the same day. Both readings are internally consistent with the tool's own (flawed) predicate. Resolved by F-01. *This is the review's central finding: the layer built to prevent silent failure was itself failing silently.*
2. **The charter-drift guard audits a stale binary** (0.11.1766) against a repo at 0.11.1944 — it measures the installed artifact, not the source (E-20).
3. MCP tool count **260** (census, comments stripped) vs **262** (raw grep) — methodology, not a real discrepancy.
4. Charter-drift "214 checked" vs census "260" — the two guards scan **different artifacts**, not a disagreement about count.
5. **Clean build/test hygiene coexists with extreme structural concentration** (zero warnings; 46,458 lines in one file). Cost and correctness are separate axes — stated plainly so "builds clean" is not read as "well-factored".
6. **T-2548 is `work-completed` recommending "GO to subtract", with an empty `## Decision` and no removal task.** Whether a human decided is not determinable from the file.
7. **G-008 records 64; measurement says 71.** The register entry is stale relative to the condition it describes.
8. Guard layer documented as "seconds"; measured >600 s incomplete.
9. **arc-004 is recorded `closed`** while the capability it shipped is measurably dark (F-03) — "shipped" again not meaning "live".

---

## 11. Not reviewed

- The four remote hubs (`.121`, `.122`, `.141`, `.107`-as-distinct-from-localhost) beyond a read-only `fleet doctor`.
- Per-item test coverage; fix/revert ratios; change coupling; author concentration.
- Token telemetry per task/arc; BVP scores; the realization log.
- The 151 audit files as a trend series (only the last 30 were sampled, and only for finding recurrence).
- Install-findings harness and upstream-report contents.
- `docs/` (396 files) for drift against code — only counted, not read.
- Dependency value (30 workspace deps).
- Episodic memory (2,435 files) and handovers (1,706) as a corpus — sampled only via concerns/audits.
- `cargo test --workspace` **within** the guard layer (it ran separately and passed; the layer itself excludes it without `--tests`).
- The 4th guard FAIL (`cron-drift-firing-fixtures.sh`) beyond noting it — no root cause established.

---

## 12. Sovereign questions

These touch the charter, gates, and authority. **An agent must not answer them.** Each
carries a recommendation, which is a proposal only.

**Q1 — Do the 28 off-charter analytics MCP tools get removed?**
Charter non-goal #3 plus T-2548's recorded "GO to subtract" point toward removal; the
`## Decision` section is empty and no human sign-off exists anywhere in evidence. The
measured cost of keeping them is **~3,341 tokens (12.7% of the tool-description payload)
in every agent context every session** (E-57).
**Recommendation:** record an explicit decision now — GO (opening a scoped removal task)
or NO-GO/defer (updating the allowlist reasons accordingly). Leaving 28 tools
permanently acknowledged-but-undecided is itself a standing cost on every future
charter-drift review, and this review re-discovered it exactly as the last one did.

**Q2 — Does `orchestrator.route`'s existence conflict with non-goal #4 ("not a workflow
or orchestration engine")?**
It is hub-implemented, unreachable by any client, and its name directly overlaps the
non-goal's language — yet a live tripwire test treats it as intentional residual
infrastructure.
**Recommendation:** a human ruling on whether an unreachable method may continue to exist
under non-goal #4, or whether it is the non-goal's most literal violation-in-waiting and
should be scoped out.

**Q3 — Does G-064 ("Hub has no per-user authorization model", open 103 days) conflict
with non-goal #5 ("not a security boundary between mutually-distrusting tenants")?**
These read as being in tension: one an open high-severity concern, the other a canonical
non-goal.
**Recommendation:** a human ruling on whether G-064 targets something narrower (e.g. audit
attribution) that does **not** require tenant-isolation guarantees, so the concern can be
scoped precisely instead of sitting ambiguous against a non-goal for over three months.

**Q4 — Should the `/canaries` predicate change be treated as a gate change?** (raised by
this GATHERER, not the JUDGE)
F-01 modifies when a protection layer reports green. It *strengthens* the gate, and the
prompt forbids weakening checks — but it alters authority-bearing behaviour.
**Recommendation:** treat it as an ordinary bug fix, because the change only narrows the
*clear* condition and never the *fire* condition. Flagged rather than assumed.

---

## 13. Expected-effect predictions (pre-registered)

To be checked after any approved execution — before/after, in the stated window.

| Finding | Metric | Direction | Window |
|---|---|---|---|
| F-01 | `/canaries` `firing` count vs non-empty-log count | converge to equal | 7 daily runs |
| F-02 | `arc-live-probe.sh` exit code; `[WARN] binary` appends | 1 → 0; daily → none | 1 restart cycle, then 7 days |
| F-03 | waker-liveness FIRING entries; `pty_session` on LIVE listeners | 36 → 0 new; 0 → N armed | 7 days |
| F-04 | Agreement between the two hook-counter readers | 1 vs 7 → equal | next inspection |
| F-05 | Unacked inbound filings | 11 → 0 | immediate after triage |
| F-07/F-08 | MCP parity coverage; churn concentration | 9.2% → higher **before** any split; churn spreads | 90 days |
| F-19 *(if GO)* | MCP description payload | 26,378 → ~23,037 tokens (**−12.7%**) per agent per session | immediately on removal |
| F-23 | GO-scope-not-propagated ratio | 48.7% → lower | 30 days of audits |

---

## 14. What should change in how the next review runs

1. **Instrument MCP/CLI invocations first.** Gap #1 blocked more verdicts than any other single factor. Without it, the 262-tool surface can only ever be judged on charter reasoning, never on use.
2. **Snapshot canary heartbeats as well as logs.** F-01 was nearly invisible; only the pre-run snapshot and a timezone check separated a real finding from my own footprint. The 9-minute margin was uncomfortably thin.
3. **Run the guard layer in text mode, early, in the background.** The `--json` form yields nothing if it doesn't finish (F-06); text mode gave incremental verdicts and produced three findings.
4. **Dispatch the JUDGE before finishing the evidence file, not after.** Its ~10-minute think time was dead wall-clock; and anything gathered afterwards falls outside its input and must be self-judged at reduced confidence (E-49…E-59, F-23).
5. **Verify every glob on hidden files.** My first heartbeat check used `*.heartbeat`, which silently matches no dotfiles, and briefly produced a confident and completely wrong "no heartbeats exist" reading. A check that *looks* conclusive and is merely mis-globbed is exactly the failure class this repo builds guards against.

---

## Provenance

- GATHERER: Claude Opus 5 (this session), Phases 0–3.
- JUDGE: Claude **Sonnet**, separate worker `judge-run5`, input = evidence file only. Raw verdicts: `docs/reports/.judge-run5-verdicts.md`.
- **Phases 6 and 7 were not executed.** No item was approved, and nothing in the repo was deleted, refactored, or built as a result of this review.

### Commit status — BLOCKED, deliberately not forced

These three files are **staged but uncommitted**. `fw git commit` was refused by the
inception gate:

> `BLOCKED: Inception gate — T-2971 has no go/no-go decision` (2 exploration commits already, decision required)

T-2971 is `workflow_type: inception`, `owner: human`. The two available paths were
`fw inception decide T-2971 go|no-go` — a **sovereign** act the Authority Model reserves
to the human ("Agent → INITIATIVE → can propose, request, suggest — **never decides**") —
and `git commit --no-verify`, a Tier-0 bypass forbidden both by this run's instructions
and by the autonomous-mode boundaries. I took neither. The gate behaved correctly.

**To land these files, one line:**

```
cd /opt/termlink && .agentic-framework/bin/fw inception decide T-2971 go --rationale 'Value review run5 complete; report + evidence ready to land' && .agentic-framework/bin/fw git commit -m "T-2971: Project value review (run5) — delete/refactor/add, whole repo"
```

*(Substitute `no-go` if the review should be recorded as closed without authorising
follow-on build work — the report itself lands either way.)*
