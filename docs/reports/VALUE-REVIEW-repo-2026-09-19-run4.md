# Project Value Review — TermLink (whole repo)

- **Scope:** whole repo (`/opt/termlink`)
- **Date:** 2026-09-19
- **Run:** run4 of a 5-run consecutive repeatability series. Independent — prior runs' outputs were **not read**.
- **Task:** T-2971
- **Evidence file:** [`VALUE-REVIEW-repo-2026-09-19-run4-evidence.md`](VALUE-REVIEW-repo-2026-09-19-run4-evidence.md) (all `S…`/`N…` citations below refer to it)
- **Status:** **Phases 0–5 complete. Phase 6 (execute) and Phase 7 (close) NOT run** — no human approval exists. Nothing in this repo was deleted, restructured, or built as a result of this review.

> **Every proposal below is a proposal.** Research is not authorization. Human sovereignty over execution is absolute.

---

## 1. Yardstick — **PROVISIONAL / UNCONFIRMED**

Source: `docs/CHARTER.md` (the canonical sentence is marked human-blessed).

- **Purpose:** TermLink is a hub-mediated, durable append-log message bus with terminal endpoints — the coordination substrate that lets a fleet of AI agents (and humans) discover each other, exchange durable messages, claim work, and control terminal sessions across one or many machines.
- **Users / consumers:** AI agents (Claude Code sessions) and human operators across a small fleet; the Agentic Engineering Framework (AEF) as the policy layer on top.
- **Core capabilities (four charter verbs):** discover · exchange durable messages · claim work · control terminal sessions.
- **Non-goals:** (1) not inter-hub federation; (2) not a durable database / system of record; (3) not a social/engagement platform; (4) not a workflow/orchestration engine — "the substrate stays mechanism, not policy"; (5) not a security boundary between mutually-distrusting tenants.
- **Value drivers:** D1 Antifragility (weight 9), D2 Reliability, D3 Usability, D4 Portability — the Constitutional Directives, in priority order. Weights are §ACD human-sovereign and were **not** touched by this review.

### [ASK] gate 1 — what would have been asked, verbatim

> "Here is the yardstick I extracted from `docs/CHARTER.md`. Two things I want you to confirm or correct before I judge anything against it:
> **(a)** Is the charter still the operative statement of purpose, or has the direction moved since T-2470 wrote it?
> **(b)** Non-goal #4 says 'the substrate stays mechanism, not policy'. A large amount of this repo is now *policy* — 24 cron canaries, 66 static checks, ~1,900 lines of CLAUDE.md governance narrative. Is that in scope as TermLink, or is it AEF's layer that happens to live in this repo? My classification of roughly a third of the findings depends on your answer."

**No human was available.** The yardstick above was adopted **provisionally** and is marked UNCONFIRMED. Question (b) is unresolved and is carried into §12 as a Sovereign Question.

---

## 2. Data availability map — **PROVISIONAL / UNCONFIRMED**

| Source | Status | Location | Window | Trustworthiness |
|---|---|---|---|---|
| Rust test suite | **EXISTS** | `cargo test --workspace` | now | measured, reliable |
| Git history | **EXISTS** | 7,172 commits | full | observed |
| AEF task ledger | **EXISTS** | `.tasks/{active,completed}` | full | observed |
| Concerns / learnings registers | **EXISTS** | `.context/project/*.yaml` | full | observed |
| Gate bypass log | **EXISTS** | `.context/working/.gate-bypass-log.yaml` (482 entries) | full | observed |
| Canary logs (22) | **PARTIAL** | `.context/working/.*-canary.log` | varies | **5 of 7 firing logs carry no per-entry ISO date** → firing frequency unmeasurable (S17.5) |
| Guard-layer runner | **EXISTS** | `scripts/run-guard-layer.sh` | now | measured — but ~20 min runtime, census only obtained late (S19) |
| Fleet adoption snapshot | **EXISTS**, **out-of-band** | `.context/working/.fleet-adoption-snapshot.log`, 89 daily snapshots | ~89 days | **The one observer that is not the bus reporting on itself.** High trust. |
| Hub topic state | **EXISTS** | `termlink channel list --json` | current retention window | measured, but retention-bounded (not history) |
| **Per-tool / per-verb invocation telemetry** | **ABSENT** | — | — | **No data at all.** `termlink api-usage` → `unrecognized subcommand` (S17.1) |
| **External / cross-project consumer data (IW-1)** | **ABSENT** | blocked by the T-559 project boundary | — | (S17.2) |
| **Hub-side discarded-post / silent-drop telemetry** | **ABSENT** | CLAUDE.md states it is "NOT AVAILABLE from the bus itself" | — | (S17.3) |
| **BVP realization log** | **ABSENT** | `.context/audits/bvp-realization.jsonl` not found | — | (S17.8) |
| Coverage instrumentation | **ABSENT** | no llvm-cov/tarpaulin; not installed per the no-new-dependency rule | — | (S17.7) |
| Workflow execution traces (`fw workflow run`) | **ABSENT / DESIGNED-ONLY** | — | — | not present in this repo |

**Snapshot window:** all usage data was read at **2026-09-19T18:31:59Z**, before this review executed anything. Judgements are made from that snapshot.

### [ASK] gate 2 — what would have been asked, verbatim

> "Four sources I need are ABSENT, and three of them change answers rather than just confidence:
> **1.** There is no per-tool invocation telemetry anywhere. That means for all 260 MCP tools I can tell you *whether anything in this repo calls them* but not *whether anything anywhere calls them*. Does usage data exist somewhere I can't see — another project, a log, your own memory of what you actually use?
> **2.** T-2548 names an 'IW-1 external-consumer check' as the gate on deleting 28 tools, and says the T-559 project boundary blocks it from here. Has that check been done? If not, who can do it?
> **3.** There's no BVP realization log, so I cannot tell you whether any arc recorded as 'shipped' actually delivered.
> **4.** Five canary logs have no timestamps per entry, so I can't tell a one-day blip from a chronic condition for five of the seven things currently firing.
> Are any of these available outside the repo?"

**No human was available.** The map above is adopted **provisionally** and marked UNCONFIRMED. The consequence is stated explicitly wherever it caps a finding.

---

## 3. Role setup

**Separation was achieved.** GATHERER and JUDGE ran as separate processes on different model families.

| Role | Model | Process | Input |
|---|---|---|---|
| GATHERER (Phases 0–3) | **Opus 5** | this session | the repo, read-only |
| JUDGE (Phase 4) | **Sonnet** | separate `claude -p` in its own tmux session, own context window | **only** the evidence file + the provisional yardstick; explicitly instructed not to explore the repo |

**How, and one thing worth recording.** The `Agent` tool was **BLOCKED** by the `check-agent-dispatch` gate — twice, with zero prior dispatches this session (S12.1). **The gate was not bypassed.** `fw dispatch reset` and `fw dispatch approve` were deliberately not run: passing a structural gate is a human decision (CLAUDE.md § Autonomous Mode Boundaries), and no human was present. Instead the gate's own recommended path was used — `scripts/tl-dispatch.sh --name vr4-judge --model sonnet` — which spawns a real separate worker (S20).

That block is itself finding **A1**: the gate advertises a per-*session* limit of 2 and enforces a per-*project-lifetime* limit of 2. It is reported here because it happened, not because it obstructed the review.

**No confidence penalty is applied for role separation.** Confidence is reduced only where the evidence file itself records a data gap.

---

## 4. Baseline

| Check | Result |
|---|---|
| `cargo test --workspace` | **3,618 passed · 0 failed · 4 ignored**, exit 0 |
| `bash scripts/run-guard-layer.sh --json` | **RED — exit 1**: `total 113 · passed 109 · fired 4 · errored 0 · unclassified 75`; **~20 min** wall-clock (S19.1) |
| Product source | 180,686 LOC Rust across 7 crates |
| Ops/guard shell | 39,455 LOC (`scripts/`) + 12,704 LOC (`tests/`) |
| `CLAUDE.md` | 3,283 lines / **276,749 bytes ≈ 69,000 tokens**, loaded every session |
| Active tasks | 246 (137 human-owned); completed 2,433 |
| Fleet | 4–5 hubs, **2–3 reachable**; **0 LIVE listeners** at snapshot time |

**This baseline is not clean.** The guard layer is red *before* any change from this review. Any Phase 6 execution must treat `exit 1 / 4 fired` as the starting state, not regress from a green one.

---

## 5. Summary

**2 DELETE · 6 REFACTOR · 7 ADD · 6 INVESTIGATE** (JUDGE classification).

**Top DELETE** (only two — the JUDGE found no third that passes DELETE CHECKS)
- **D1** — Execute the already-decided GO on 28 live off-charter analytics tools. The human decided **GO (subtract) on 2026-08-20**; 30 days later nothing has executed it and no build task exists (S4.6–S4.8).
- **D2** — Finish removing the 46 deprecated MCP tools. 48-day soak elapsed, zero-caller check done; the project's own doc requires a separate human go-ahead that is not on record (S4.9).

**Top REFACTOR**
- **R1** — Cut CLAUDE.md's always-loaded canary narrative (~1,908 of 3,283 lines). Duplicates each script's own header; costs ~69k tokens *every session* (S3.1–S3.8).
- **R2** — Split `termlink-mcp/src/tools.rs`: 46,458 LOC in one file, 390 touches/180d — the repo's largest churn × size hotspot (S5.1–S5.3).
- **R3** — Address the 67 duplicated `_mcp`/CLI helper pairs. This exact duplication already produced a 45-day undetected drift (S4.11–S4.13).

**Top ADD**
- **A1** — Repair the agent-dispatch gate (per-lifetime, not per-session; blocked this review live) (S12).
- **A2** — Repair `run-guard-layer.sh`: documented as "seconds", measured at ~20 min, gates every push/PR against a `timeout-minutes: 10` CI budget (S9.7–S9.9, S19.4).
- **A3** — Wire firing canaries to task creation: 7 firing, one for 74 distinct days, **zero** corresponding active tasks (S7, S13.3).

### The one-paragraph reading

The detection layer works and the response layer does not. Seven canaries are firing right now, one of them daily since 2026-07-06, and none has produced a task. The fleet the substrate exists to coordinate is measurably quiet — **0 chat-arc posts and 0 unique speakers for ten consecutive daily snapshots**, 0 LIVE listeners, 2–3 of 4 hubs reachable, and the push-wake rail reporting `RAIL DARK` (S6.3, S6.6, S7.2). The diagnosis for that quiet is **A (BROKEN)**, not E (not wanted): the charter names these capabilities, seven canaries were built to protect them, and every one of those canaries is correctly reporting the breakage. Meanwhile the human — sole owner of 137 of 246 active tasks, with 71 of them waiting a **median of 125 days** for verification — is the saturated resource, and the highest-authority gate in the model (`--skip-sovereignty`) has been bypassed 149 times. That is what this review found: not a project that built the wrong things, but one whose guard layer grew 6.4× in five months while the loop from *finding* to *fix* stayed manual and one human wide.

---

## 6. Findings table

*JUDGE output, relayed unchanged. Rows are ranked within each axis by value per unit of cost and risk. `S…`/`N…` cite the evidence file.*

| ID | Item | Location | Class | Non-use reading | Evidence | Counter-evidence | Conf. | Proposal | Size | Reversible? | Risk if wrong | Expected effect (pre-registered) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **D1** | 28 live off-charter analytics tools | `crates/termlink-mcp`, categories `agent_stats` / `agent_thread_health` / `agent_rankings` / `channel_engagement` | DELETE | **B** (never wired — zero first-party callers) + **D** (unmeasured externally) + **E** (human GO, T-2548, 2026-08-20) | S4.4, S4.6, S4.7, S4.8 | KEEP-list already carves 3 tools out of the same set (S4.7); IW-1 external-consumer check still formally open, blocked by T-559 (S17.2) | **HIGH** | Create the build task T-2548 already authorizes; remove the 25 non-KEEP tools; re-run charter-drift check | Medium | Yes — `p4-surface-reduction.md` convention keeps it `git revert`-able | If IW-1 later surfaces a real external caller, a consumer breaks silently | `check-charter-drift-freshness.sh --json`: `checked` 214→189; `charter-drift-allowlist` 28→3 entries; check within 1 session of task creation |
| **D2** | 46 deprecated MCP tools (8 wholly-deprecated categories) | `crates/termlink-mcp`; `crates/termlink-cli` hidden twins | DELETE | **E** (recorded decision to *deprecate*, 2026-08-02) — deletion step itself not yet decided | S4.1, S4.2, S4.9 | `p4-surface-reduction.md` explicitly requires "a separate human go-ahead" for deletion; no such record found (S4.9) | **MEDIUM** | Present the 48-day soak for the go-ahead the doc itself requires, then delete | Medium | Yes (`git revert`) | No telemetry rules out a hidden external caller (S17.1, S17.2) | MCP tool count 260→214 (fewer after D1); parity-census denominator drops, coverage % rises with no new tests |
| **R1** | CLAUDE.md: ~42 canary sections, 1,908 of 3,283 lines | `CLAUDE.md` | REFACTOR | — | S3.1–S3.8, S16.1, S16.2 | The rationale is arguably load-bearing; S3.7 shows cross-referencing with tasks/scripts | **HIGH** | Move per-canary rationale out of the always-loaded region into per-script headers (already present, S3.6) or a linked reference doc; keep a one-line pointer table | Medium — touches the T-2015 clobber boundary | Partially — content must be **preserved**, not deleted | Losing a canary's operator remediation text if extraction is sloppy | CLAUDE.md 276,749 B → target <80,000 B; per-session preload ~69k → ~20k tokens; verify at next `fw context init` |
| **R2** | `tools.rs` monolith | `crates/termlink-mcp/src/tools.rs` | REFACTOR | — | S5.1, S5.2, S5.3 | None found | **HIGH** | Split by tool category (mirror the 29 `help` categories) into submodules | **Needs its own design** — 46K LOC, 390 recent touches | Yes, mechanically, if tests come first | Merge conflicts against 447 commits/30d if not sequenced carefully | One file 46,458 LOC → N files <3,000 LOC; the churn-hotspot report no longer names one file in the top 2 |
| **R3** | 67 duplicated `_mcp` / CLI helper pairs | `termlink-mcp/src/tools.rs` ↔ `termlink-cli/src/commands/*` | REFACTOR | — | S4.11, S4.12, S4.13 | T-2069 convention explicitly forbids cross-crate sharing "for these tiny pure helpers" — a deliberate prior decision (→ §12 Q3) | **MEDIUM** | If T-2069 is revisited: extract shared pure fns into `termlink-protocol` or a new crate. **Else:** add a check that both twins changed in the same commit | Medium | Yes | A shared crate reintroduces exactly the coupling T-2069 was written to avoid | Duplicated pairs 67→0 (if extracted), **or** 0 undetected-drift incidents in the next 90 days (if the paired-commit check ships instead) |
| **R4** | Governance-artefact volume: 105 MB vs ~6 MB product source; 24:1 touch ratio | `.context/handovers` (59 MB / 1,706 files), `.tasks`, `.context/episodic`, `.context/audits` | REFACTOR | — | S14.1–S14.4 | Some is intentionally durable — episodic memory is a designed subsystem | **MEDIUM** | Apply the retention discipline the project enforces on the bus (its own T-2562 forever-archival pattern) to its own handover/audit trees — compress/archive >90 days | Small–Medium | Yes (**archive, don't delete**) | Losing forensic value if archived data becomes unreachable | `.context/handovers` 59 MB → <20 MB active + archived remainder |
| **R5** | `health:ring20-fedprobe` on `Retention::Forever`, 1,670 records = **38.8% of all bus traffic** | hub channel config | REFACTOR | — | S6.7, S16.8 | Below the forever-archival canary's 50,000 threshold, so it does not fire — the canary calls it healthy | **MEDIUM** | Set bounded retention per the project's own recommended-settings table | Small | Yes | Health-probe history is not typically needed indefinitely | Topic 1,670 → ≤N records; bus-wide record count drops ~35% |
| **R6** | Cluster of stale/contradicted CLAUDE.md factual claims | `CLAUDE.md:4`, §guard-layer, §agent-dispatch, §T-1419, dispatch remediation text | REFACTOR | — | S16.1, S16.2, S16.4, S16.7, S16.9 | None | **HIGH** | Fix or remove five claims: (a) `FRAMEWORK.md` reference, (b) "(seconds)", (c) "per session", (d) `api-usage --json`, (e) dispatch remediation text | Small (5 one-line fixes) | Yes | None | 0 of the 5 contradicted on the next verification pass |
| **A1** | Agent-dispatch gate: claims per-session, behaves per-project-lifetime | `.agentic-framework/agents/context/check-agent-dispatch.sh`, `post-compact-resume.sh:42-48` | **ADD (REPAIR)** | **A** — applies to the mechanism itself | S12.1–S12.9 (**measured live, this session**) | None | **HIGH** | Reset the counter on a genuine cold start (not only on `.auto-restart-pending`), or key the limit to a real per-conversation identifier | Small | Yes | An over-eager reset could let a runaway session dispatch unboundedly — needs a real session id, not just cold-start detection | Next cold-start session: two `Agent` dispatches succeed without `BLOCKED`; counter resets to 0 |
| **A2** | `run-guard-layer.sh` no longer completes in the documented time | `scripts/run-guard-layer.sh`, wired into `doc-lint.yml` + `release.yml` | **ADD (REPAIR)** | **A** — the runner itself | S9.6–S9.9, S16.2, S19.1, S19.4 | 44 of 191 members carry the fast `source` marker; slowness may be concentrated in the rest | **HIGH** | Profile the 191 members; parallelize or split fast/slow CI jobs; **update the doc claim to match reality regardless** | Medium | Yes | If CI is silently soft-timing-out today, real defects pass through on every merge | `--json` returns within a bounded, stated time (<2 min) with a full census for all 191 members |
| **A3** | 7 firing canaries, **0** corresponding active tasks | `.context/working/.*-canary.log`, `.tasks/active/` | **ADD (WIRE)** | **B** — for the response half (detectors work; nothing converts a firing into a task) | S7.1–S7.10, S13.3, N4 | ~1,908 CLAUDE.md lines show intent to act; the mechanism to act is what is missing | **HIGH** | Auto-file a task on firing (mirror the framework-pickup canary's own self-filing pattern), de-duplicated by canary name | Medium | Yes | A naive auto-filer spams duplicates per cron cycle — de-dup is load-bearing | ≥1 active task naming each currently-firing canary within 24h of the next cron run |
| **A4** | Stale binary unresolved across 74 distinct days | `substrate-preflight` canary; local + remote hubs | **ADD (REPAIR)** | **A** | S7.1, S19.9 | None | **HIGH** | Upgrade `termlink` to match `VERSION` 0.11.1944 and restart the hub through its systemd unit | Small (operational) | Yes | None | Next `substrate-preflight.sh`: 0 WARN lines on binary-version checks |
| **A5** | Hook-counter race (unlocked truncate+write) | `.agentic-framework/lib/hook-telemetry.sh` | **ADD (REPAIR)** | **A** | S7.5, S12.10 | **Vendored (G-062)** — a local patch is erased by the next re-vendor | **HIGH** | File upstream per the project's own G-062 convention; local `flock` wrapper only as an explicit stopgap | Small | Yes | None | `hook-counter-integrity` log stays empty 30 consecutive days after the upstream fix lands |
| **A6** | 71 partial-complete tasks, median **125 days** waiting, no staleness surface | `.tasks/active/*` with `owner: human` | **ADD (SURFACE)** | **C** (no notification for a 125-day-old approval) + **A** (T-2859 renderer defect, vendored, reverts on re-vendor) | S10.5, S10.6, N5 | `check-stranded-finalized-tasks.sh` deliberately does not fire on this class — T-193 partial-complete is a valid designed state, so non-firing is not itself a defect | **MEDIUM** | Add a staleness surface (extend `/canaries` or the handover banner) listing partial-complete tasks over a threshold — mirrors the existing `revisit_at` G-053 pattern | Small–Medium | Yes | None | The 56 tasks >30 days old are surfaced at next session start |
| **A7** | Dispatch-gate remediation text cites a dead CLI path | `check-agent-dispatch.sh` remediation string | **ADD (REPAIR)** | **A** | S12.11, S16.9 | None | **HIGH** | Fix the text to a working invocation, or make `fw termlink dispatch --help` work | Trivial | Yes | None | `fw termlink dispatch --help` prints usage, not `Unknown option` |
| **I1** | Fleet-binary canary reports healthy while `.121`/`.122` run ≈355 commits behind | canary vs S6.8 | INVESTIGATE | D-adjacent | S6.8, S7 (fleet-binary in the healthy list) | Floors may be deliberately exempt for these hosts | MEDIUM | **Need:** `fleet-version-floors.conf` entries for `.121`/`.122` — deliberate exemption or a floor set too low? | — | — | — | — |
| **I2** | `chat_arc_posts` / `unique_speakers` = 0 for 10 consecutive snapshots | S6.3, S6.9 | INVESTIGATE | D/E ambiguous | S6.2, S6.3, S6.9 | The hub-dedup stderr line (S6.9) coincides in time — could be an artifact of a fixed double-count rather than real collapse | MEDIUM | **Need:** the dedup fix's commit date vs. the start of the zero streak | — | — | — | — |
| **I3** | Task completion fell ~8× (609/mo → ~75/mo) while commits held at 447/30d | S10.7, S10.8 | INVESTIGATE | — | S10.7, S10.8 | Could reflect bigger tasks, not less work | **LOW** | **Need:** LOC-changed or session-count per completed task, by month | — | — | — | — |
| **I4** | `--skip-sovereignty` used 149× (2nd-most-bypassed gate) | S11.2, S11.4 | INVESTIGATE (also Sovereign, §12 Q4) | — | S11.1–S11.5 | Tier-2 bypass is a designed, logged escape hatch — volume alone is not misuse | MEDIUM | **Need:** a sample of 20 bypass entries with stated reasons | — | — | — | — |
| **I5** | G-086/G-087 (high-severity, spoofable-claimer) — 4 of 7 sampled open concerns have no active task | S13.2, S13.3 | INVESTIGATE | B-adjacent | S13.2, S13.3 | Work may be tracked under a task that doesn't contain the literal G-id | MEDIUM | **Need:** full-text search of task *bodies* for the G-086/G-087 substance (S13.3 greps the G-id string only) | — | — | — | — |
| **I6** | Was a human go-ahead for actual deletion of the 46 ever given? | S4.9 | INVESTIGATE (feeds D2) | E claimed, unconfirmed for the deletion step | S4.9 | — | **LOW** (absence of evidence only) | **Need:** ask the human — fastest route to resolving D2 | — | — | — | — |

### GATHERER notes on the JUDGE's output (recorded, classifications unchanged)

The JUDGE's verdicts stand as issued. Three things I can add from the gathering side without altering them:

1. **A3 contains an arithmetic slip.** It says "94-day-old substrate-preflight firing"; the evidence is **74 distinct dates spanning 2026-07-06 → 2026-09-19** (S7.1). The finding is unaffected; the number should read 74.
2. **On I2** — the JUDGE reasonably flags the hub-dedup stderr line as a possible artifact. The gathered series is a *progressive* decline (270.4 → 12.1 → 31.9 daily average, then ten zeros), not a step change, and `dm_topics_active` fell 226.8 → 13.2 over the same thirds (S6.2, S6.4). An artifact of a counting fix would more likely present as a step. This does not resolve I2 — the JUDGE's requested evidence (the dedup commit date) is still the right test.
3. **On D1/D2** — both DELETE rows correctly leave IW-1 unsatisfied. Per DELETE CHECK #5, **neither can proceed to Phase 6 on this evidence alone**, regardless of the T-2548 GO.

---

## 7. KEEP list

Rust workspace (3,618 passing tests) · the four charter verbs' core primitives (claim / release / renew / claim-transfer; discover / find-idle; cv_index broadcast-with-replay; session spawn / exec) · `docs/CHARTER.md` · the T-2548 KEEP-listed tools (`agent_search_thread`, `agent_thread_path`, `agent_recent_window`) · the guard-layer **concept** (static checks + allowlist-ledger pattern, distinct from its current runtime problem) · the fleet-adoption-snapshot out-of-band observer · the four provers (`comms-selftest`, `session-selftest`, `substrate-smoke`, `session-message-selftest`) · the T-1857 sender-resolution chain and the R2/R3 auth-rotation protocol · the script inventory itself (**0 of 190 scripts unreferenced** — S9.5; this is not bloat).

---

## 8. INVESTIGATE list

I1–I6 above, each with the specific datum needed. Additionally: **N4's "D UNMEASURED" reading for canary response** and **N2/N3's telemetry gap** need the same missing instrument (per-tool / per-canary action tracking) — closing one closes both.

---

## 9. Data gaps that capped confidence

Each is an ADD candidate in its own right.

- **S17.1 / S17.2 — no per-tool invocation telemetry, no external-consumer visibility.** Caps every MCP-tool DELETE to *decision-based* rather than *usage-based*, and blocks D1/D2's own stated gate (IW-1). **Unlocks:** usage-grounded surface decisions instead of 30-day-old inferences. Closing it means per-verb counters, the way `fleet-adoption-snapshot` already does for the bus.
- **S17.3 — no hub-side telemetry for silently discarded posts.** Caps any reliability claim about "exchange durable messages"; G-088 (exactly-once) is precisely this class of unverifiable claim. **Unlocks:** the ability to judge the charter's second verb at all.
- **S17.6 — the guard-layer census arrived only after ~20 min** (and drove A2). **Unlocks:** a usable per-commit guard signal.
- **S17.5 — five firing canary logs carry no per-entry date.** Blocks distinguishing a blip from a chronic condition for S7.2 / S7.4 / S7.6, which directly weakens A3's urgency ranking for those three (vs. the dated 74-day S7.1). **Unlocks:** correct triage order.
- **S17.8 — no BVP realization log.** Blocks answering "did the arcs recorded as shipped actually deliver?" and is the only route to eventually validating this review's own pre-registered expected effects against the weighted value drivers.
- **S17.7 — no coverage instrumentation.** Only test *counts* were available. Caps REFACTOR CHECK #2 ("behaviour to preserve is covered by tests") to an inference for R2 and R3.

---

## 10. Contradictions

All nine in S16 stand:

1. `CLAUDE.md:4` points at `FRAMEWORK.md`; no such file at repo root (S2.4).
2. Guard-layer runner documented as "(seconds)"; measured ~20 min (S19.1).
3. The charter-drift allowlist and CLAUDE.md both describe T-2548 as an open, `started-work`, human-owned decision; **T-2548 is `work-completed` with Decision: GO, dated 2026-08-20** (S4.6).
4. `check-agent-dispatch.sh:10` says "per session"; behaviour is per project lifetime (S12.8).
5. "Empty log = healthy. Any entry = [action]" — the substrate-preflight log has been non-empty across 74 distinct days with no recorded action (S7.1, S7.9).
6. CLAUDE.md states the stuck-claims arm "self-clears"; it is firing on four topics all with `active=0` (S7.4).
7. CLAUDE.md references `api-usage --json`; the verb does not exist in the running binary (S15/N2).
8. Charter non-goal #2 ("not a system of record") vs. `health:ring20-fedprobe` at `Forever` retention holding 38.8% of all bus traffic (S6.7).
9. The dispatch gate's remediation text recommends `fw termlink dispatch`; neither `--help` nor a bare invocation yields usage (S12.11).

**Plus one the JUDGE found by cross-reading:** the **fleet-binary-freshness canary reports healthy (empty log)** while `fleet doctor --json`, read in the same snapshot window, shows `.121`/`.122` ≈355 commits behind. The canary's floor configuration is silently permissive for exactly the hosts the raw data flags (→ I1).

---

## 11. Not reviewed

- `.agentic-framework/` internals beyond the specific files cited — **vendored (G-062)**; changes there belong upstream.
- Per-function complexity metrics — no tool installed, and none added, per the no-new-dependency rule.
- `web/` Watchtower surfaces beyond the T-2859 record already in CLAUDE.md.
- Cross-hub state on `.121`, `.122`, `.141` — two unreachable, no foothold.
- Per-item test coverage — no instrumentation (S17.7).
- **Prior runs (run2, run3) of this same review** — deliberately not read, to keep run4 independent.
- **Phases 6 and 7** — not executed. No human approval exists.

---

## 12. Sovereign questions

Each carries a **recommendation only**. None of these is this review's to decide.

1. **Execute D1 (the 28-tool subtract)?** A human decided **GO on 2026-08-20**; 30 days on, nothing has executed it and no build task exists. *Recommendation:* create the build task — the decision is not being revisited, only its execution proposed. **Note:** DELETE CHECK #5 (no external consumer) is still unsatisfied because IW-1 was never run, so the build task should carry IW-1 as its first step, not skip it.
2. **Execute D2 (delete the 46 deprecated tools)?** The *deprecation* was human-decided; `p4-surface-reduction.md` makes the *deletion* a distinct, still-open decision. *Recommendation:* given the 48-day soak and zero-caller evidence, grant it — but the project's own written gate makes this explicitly yours.
3. **Revisit the T-2069 "no cross-crate sharing" convention (R3)?** A deliberate architectural choice that has already cost one 45-day undetected parity drift (S4.13). *Recommendation:* worth revisiting on the measured cost — but it is a prior sovereign decision, not this review's to override. The cheaper alternative (a paired-commit check) needs no change to the convention at all.
4. **`--skip-sovereignty` at 149 uses (I4).** This touches the Authority Model directly. *Recommendation:* **audit a sample before concluding anything, and do not weaken the gate based on volume.** The more likely reading, given 71 tasks waiting a median 125 days for the same human, is that the gate is fine and the human-verification path is saturated — which is A6, not a gate change.
5. **CLAUDE.md restructuring (R1)** intersects the T-2015 clobber boundary (`fw upgrade` destroys everything from `## Core Principle` onward). *Recommendation:* extract into script headers — already the established pattern (S3.6) — rather than inventing a second protected region and a second clobber problem.
6. **Unresolved from [ASK] gate 1:** is the guard/governance layer in this repo *TermLink*, or is it AEF policy that happens to live here? Charter non-goal #4 says the substrate stays mechanism, not policy. Roughly a third of the findings' framing depends on the answer, and the yardstick remains UNCONFIRMED without it.

---

## Provenance and limits

- **Phases 0–5 only.** Phase 6 (execute) and Phase 7 (close) were not run and must not be inferred from this document.
- The **yardstick (§1)** and the **data availability map (§2)** are **PROVISIONAL / UNCONFIRMED** — both [ASK] gates were passed without a human, with the questions recorded verbatim.
- Role separation **was** achieved (§3): JUDGE on Sonnet in a separate process, reading only the evidence file. No blanket confidence penalty applied; per-finding gaps are stated in §9.
- **No gate was bypassed during this review.** The `Agent` dispatch block was routed around via the gate's own recommended path, not overridden.
- Nothing was deleted, refactored, or added. The only files this review wrote are this report and its evidence file.
