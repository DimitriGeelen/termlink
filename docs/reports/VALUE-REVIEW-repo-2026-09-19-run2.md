# PROJECT VALUE REVIEW — TermLink (whole repo)

**Date:** 2026-09-19 · **Scope:** whole repo (`/opt/termlink`) · **Run:** 2 of 5 (independent)
**Task:** T-2971 · **Evidence file:** [`VALUE-REVIEW-repo-2026-09-19-run2-evidence.md`](VALUE-REVIEW-repo-2026-09-19-run2-evidence.md)
**Phases executed:** 0–5. **Phase 6 (execute) and 7 (close) deliberately NOT run** — no human approval exists in this run.

> **Nothing in this report has been executed.** Research is not authorization. Every
> proposal below awaits item-by-item human approval.

---

## 1. Yardstick (PROVISIONAL — UNCONFIRMED)

> **[ASK] gate 1 was not answerable — this run is headless.** Verbatim, what I would
> have asked the human: *"Here is the yardstick I read out of `docs/CHARTER.md` — purpose,
> users, the four core verbs, the five non-goals, and the value-driver weights. Is this the
> right measuring stick for judging what to delete, refactor and add? Specifically: (a) is
> the charter still current, or has the real purpose moved since T-2470 filed it; (b) are
> the four verbs equally load-bearing, or is one of them the product and the rest support;
> (c) who are the users you actually care about — the LAN fleet, the external consumer
> project, or the human operator?"* Adopted as PROVISIONAL and marked UNCONFIRMED.

**Purpose** (`docs/CHARTER.md`, human-blessed): *TermLink is a hub-mediated, durable
append-log message bus with terminal endpoints — the coordination substrate that lets a
fleet of AI agents (and humans) discover each other, exchange durable messages, claim work,
and control terminal sessions across one or many machines.*

- **Users/consumers:** AI coding agents across a 5-hub LAN fleet (4 reachable); at least one
  genuine external consumer project (`/opt/1409-sprind`, posting to `aef-install-findings`);
  a human operator.
- **Core capabilities:** V1 discover · V2 exchange durable messages · V3 claim work ·
  V4 control terminal sessions.
- **Non-goals:** (1) not federation · (2) **not a durable database / system of record** ·
  (3) **not a social / engagement platform** · (4) not a workflow engine · (5) not a
  security boundary between distrusting tenants.
- **Value drivers:** D1 Antifragility (9) · D2 Reliability (7) · D3 Usability · D4
  Portability · F-RECALL Recall Leverage (6). *(F-ORCH retired 2026-07-07, T-2511.)*

**Contradiction between stated purpose and actual code (flagged per Phase 1):** the charter
says the substrate "stays mechanism, not policy" and is "not a social/engagement platform",
yet 28 live analytics tools remain on the MCP surface with zero callers, and the project's
own measured activity has shifted almost entirely into policy/governance bookkeeping (§4).

---

## 2. Data availability map (PROVISIONAL — UNCONFIRMED)

> **[ASK] gate 1 (part 2), verbatim:** *"Here is what I can and cannot see. The gaps that
> most limit this review are: no per-verb invocation counters anywhere (so 'is this MCP tool
> used?' is answered by grepping for callers, not by measurement); no line-coverage tool and
> no unused-dependency tool installed; the BVP realization log not verified; and I cannot
> read the peer hubs' or consumer projects' state because the T-559 project-boundary gate
> blocks it from this session. Does data exist that I can't see — CI run history, a
> consumer-side usage record, anything from the other hubs?"* Adopted PROVISIONAL.

| Source | Status | Window / location | Trustworthiness |
|---|---|---|---|
| TermLink live hub (topics, presence, claims, sessions) | **EXISTS** | snapshot 2026-09-19T17:32–17:40Z | **measured — strongest source** |
| Git history (churn, hotspots, velocity) | EXISTS | 12 mo | observed — strong |
| Task ledger (2679 files), audits (151), bypass log (482) | EXISTS | Mar–Sep 2026 | observed — strong |
| Canary logs + heartbeats | EXISTS | up to 74 days | observed |
| Concerns/gaps register (52) | EXISTS | current | observed |
| CI workflow definitions | EXISTS | `.github/workflows/` | observed (definitions only) |
| **CI run history / pass rate** | **ABSENT** | — | **gap** — no GitHub access; pushing is forbidden |
| **Per-verb / per-tool invocation counters** | **ABSENT** | — | **gap — the single biggest one.** Usage of 260 MCP tools is inferred from *callers present in the repo*, never from invocation counts |
| **Line coverage** | ABSENT | no tool run | gap |
| **Unused dependencies** | ABSENT | `cargo machete`/`udeps` not installed | gap |
| BVP realization log | PARTIAL | not verified this run | gap |
| Healing events, token telemetry per task/arc | NOT SAMPLED | — | gap (budget) |
| Peer hubs' / consumer projects' own state | **BLOCKED** | T-559 project-boundary gate | gap |
| Workflow Designer / BPMN (`aef:` namespace) | **ABSENT** | none in repo | not applicable |
| Out-of-band observer of TermLink itself | PARTIAL | `aef-install-findings` (external project) is genuinely out-of-band; canaries and `fleet doctor` are not | partial gap — see §9 |

**Activity windows used.** Per the "activity, not calendar" rule: presence judged over the
full retained agent-presence window (17.4 h, 1026 records — the window is short *because*
traffic is low); chat-arc over 24 days / 950 records; claims over all 45 topics; churn over
6 months; velocity over 12 months; canary firing over 74 days.

---

## 3. Role setup

| Role | How it ran | Separated? |
|---|---|---|
| GATHERER | This session + 2 sub-agent workers (Sonnet) for the tool-surface and guard-layer sweeps | — |
| **JUDGE** | **Separate `claude -p` process**, dispatched via `fw termlink dispatch --name vr2judge`, running in its own tmux session and working directory. Its only inputs were the confirmed yardstick and the evidence file. | **YES — genuinely separate process** |
| HUMAN | **Absent (headless run).** All [ASK] gates recorded verbatim and adopted as PROVISIONAL. | — |

**Model family:** JUDGE ran on the same model family as GATHERER (no second family available
in this environment). Separation is by process and by input restriction, not by model. Per
the prompt's rule, confidence is **not** deducted for role separation — the separation is
real. It *is* noted as a residual weakness: a same-family judge can share a blind spot.

**Note on a gate encountered mid-review.** The framework's `check-agent-dispatch` hook blocked
sub-agent dispatch at "#5" although this session had dispatched none. Investigation (not
worked around) found a real defect, reported in §6 as **F-14**: the counter is documented as
per-session but never resets, and blocked attempts still increment it. The documented
`fw dispatch reset` path was used once, to make the counter reflect this session's actual
zero. The JUDGE was then dispatched through TermLink — the path the gate itself recommends.

---

## 4. Baseline

| Check | Result |
|---|---|
| `cargo test --workspace` | **3618 passed, 0 failed, 4 ignored** — exit 0 (~40 s, 24 binaries) |
| Build | clean |
| `fw audit` (2026-09-19) | pass 39 · warn 8 · **fail 1** |
| `check-cron-install-drift.sh` | 26 of 27 crontabs installed; **1 MISSING** (`substrate-smoke-canary`) |
| `check-vendor-divergence.sh` | clean — 22 vendored-code commits, all registered |
| `check-framework-tracking-drift.sh` | clean |
| `check-stranded-finalized-tasks.sh` | 0 firing (71 legitimate partial-complete) |
| Repo `VERSION` / installed binary | `0.11.1940` / `0.11.1766` (built 2026-09-01) |

**The single most important baseline number is not in that table.** In the 19 days of
September, **1 of 1023 file-touches was to `crates/`** — the product. 927 (90.6%) were to
`.context/` and `.tasks/`. See §5.

---

## 5. Summary

**Counts:** DELETE 3 · REFACTOR 3 · ADD 5 · INVESTIGATE 7 · KEEP 12 — **11 tabled findings**
(F1–F11) plus 7 INVESTIGATE items.

### The one-paragraph read

TermLink's product is in better shape than its project is. Two of the four charter verbs are
genuinely alive and load-bearing — **V4 control terminal sessions** (40 live sessions across
8+ projects) and **V2 exchange durable messages** (4,299 records, with a real external
consumer project). The test suite is green at 3,618 tests. What has gone wrong is not the
code but the **ratio**: in the 19 days of September, **1 of 1023 file-touches went to
`crates/`** and 927 went to `.context/` and `.tasks/`. The mechanism is measurable and
circular — CLAUDE.md (~69.2k tokens) plus the MCP tool descriptions (~26.4k) consume roughly
**48% of a 200k context window before any product code is read**, the budget gate then blocks
source edits at 170k, and 17 commits record work truncated by exactly that. The governance
layer built to protect the product is now crowding it out. Meanwhile the two verbs that are
*not* alive are not unwanted — **V3 claim work** is built, documented, skilled, canaried,
and has never had a production caller (NEVER-WIRED), and its exclusivity guarantee is
unenforced against a spoofable parameter (BROKEN, G-086/G-087 HIGH); the **doorbell rail** is
measured dark and provably fails to become a turn when the recipient is busy (G-083 HIGH).
Both are ADD (REPAIR/WIRE), not DELETE.

### Top 3 per axis

**DELETE**
1. **F2** — 40 deprecated-but-still-compiled MCP tools. The decision was already ratified
   (P4/T-2478); this is unfinished follow-through, and they are still advertised to every
   MCP client (directly observed in this session's own tool list).
2. **F1** — 28 off-charter analytics tools, **0 of 28 with any caller repo-wide**. Tied to
   the open human decision T-2548 — surfaced as SQ-1, not proposed.
3. **F3** — 44% of live topics (20/45) and 61% of identity keys (40/66) are unreaped
   test/demo/probe residue.

**REFACTOR**
1. **F4** — CLAUDE.md's 69k-token fixed per-session cost. The highest-leverage item in the
   review: it is the measured mechanism behind the September collapse.
2. **F5** — 26.4k tokens of MCP tool descriptions. Arc-005 already exists for exactly this
   and has been stalled through 18 consecutive audit warnings.
3. **F6** — `tools.rs`: 46,458 LOC in one file, 390 commits in 6 months, 1,403 functions —
   the repo's #1 hotspot by both churn and size.

**ADD**
1. **F7 (REPAIR)** — bind claim-verb identity to the verified sender fingerprint, as
   `channel.post` already does. Blast radius is nil *today* precisely because nothing uses
   V3 — which makes now the cheap moment, not a safety argument.
2. **F8 (REPAIR)** — make the presence heartbeat reflect real session availability rather
   than process-liveness, so `LIVE + armed` stops being able to lie.
3. **F9 (REPAIR)** — fix the preflight canary's comparator. It has fired **74 consecutive
   days** on a check that is structurally unsatisfiable, which by the project's own T-2685
   doctrine has left that canary *deaf* to any genuine finding for 2½ months.

---

## 6. Findings

Full table with per-row counter-evidence, reversibility and pre-registered expected effects:
**[`.context/working/vr-run2-snapshot/judge-verdict.md`](../../.context/working/vr-run2-snapshot/judge-verdict.md)**. Condensed here.

| ID | Item | Class | Reading | Conf. | Proposal | Size | Rev.? | Expected effect (pre-registered) |
|---|---|---|---|---|---|---|---|---|
| **F2** | 40 deprecated-but-compiled MCP tools (`tools.rs:1077-1217`) | DELETE | E | **HIGH** | Complete the already-ratified P4 removal | medium | Yes | Tool count 260→220; measurable `tools.rs` LOC/fn drop |
| **F1** | 28 off-charter analytics tools, 0 callers | DELETE *(pending SQ-1)* | E | MEDIUM | Expedite T-2548; remove 24 pure-analytics, re-examine 4 `channel_engagement` search/mention tools against V2 separately | medium | Yes | Tool count →~232; re-run parity census |
| **F3** | 20/45 topics + 40/66 identity keys are test residue; `health:ring20-fedprobe` 1,666 recs `forever` @~88/day | DELETE (ops hygiene) | E | **HIGH** | Sweep/bound the residue topics; delete stale keys; **confirm the fed-probe arc's status before deleting that one** | small | Yes | Live topics 45→~25 |
| **F4** | CLAUDE.md ≈69,187 tokens auto-loaded every session | REFACTOR | – | **HIGH** | Split into a small always-loaded governance core + on-demand `docs/operations/*`; keep one-line pointers | needs-design (must preserve the T-2015 clobber-safe boundary) | Yes | Fixed overhead 69k → stated target (<20k); budget-gate-stoppage commits/month trends down |
| **F5** | MCP descriptions ≈26,379 tokens / 261 strings | REFACTOR | – | **HIGH** | Revive **or formally close** arc-005; shorten longest outliers first | medium | Yes | Payload → ~15k; verify parity tests unchanged |
| **F6** | `tools.rs` 46,458 LOC / 390 commits / 1,403 fns | REFACTOR | – | MEDIUM | Mechanical `mod` split along the 29 help categories. **Do not touch the deliberate T-2069 helper duplication** — different decision | needs-design | Yes | No single mcp-crate file >10,000 LOC |
| **F7** | V3 claim verbs trust spoofable `claimer`/`by`/`to_owner` (G-086/G-087 HIGH) | **ADD (REPAIR)** | A | **HIGH** | Bind to verified sender fingerprint as `channel.post` does, all 4+ sites | medium | Yes | G-086/G-087 → resolved; negative-auth tests added (none exist today) |
| **F8** | Doorbell wake doesn't become a turn when recipient busy (G-083 HIGH); rail dark | **ADD (REPAIR)** | A + B | **HIGH** | Use T-2876's DELIVERED/BLOCKED/ENQUEUED vocabulary so the heartbeat reflects true availability | needs-design | Yes | waker-liveness log (33 KB, firing) shows fewer rail-dark entries within a month |
| **F9** | Preflight comparator unsatisfiable — 74 days firing | **ADD (REPAIR)** | A | **HIGH** | Compare against last tagged release or build-vs-source mtime, not the governance-inflated commit counter | **small** | Yes | 80 KB log goes empty after a rebuild and stays empty absent real drift |
| **F10** | A `forever` topic under 50k with sustained daily growth is invisible to both guards built for it | ADD (EXTEND) | – | MEDIUM | Rate-based secondary trigger (records/day over N days) in one existing check | small | Yes | Fixture gains a `rate_flagged` case |
| **F11** | Dispatch gate ratchets shut — blocked attempts increment; no session reset | ADD (REPAIR) — **file upstream only** | A | **HIGH** (observed live) | **Vendored (G-062): do not patch locally.** File upstream per the T-2304/T-2469/T-2813 pattern; record in `.vendor-divergence.yaml` | small | N/A | Upstream fix in next `fw update` |

**Ranking note.** By value per unit of cost and risk, **F9 is the best single move in the
review** — small, reversible, and it restores a monitoring channel that has been dead for
74 days. **F4 has the highest ceiling** but needs design. **F7 is the most time-sensitive**:
it is cheap *only* while V3 has no users.

---

## 7. KEEP

V2 exchange durable messages · V4 control terminal sessions · the 66-script guard/static-check
layer (5 of 6 sampled static-check tasks produced real `crates/` fixes; 39 checks are CI-wired)
· the 3,618-test suite · all 34 `.claude/commands` skills (zero orphans) · the task/governance
system · fleet doctor/governor/verify · vendor-divergence and framework-tracking-drift checks
(both clean — the mechanism demonstrably works) · `channel:learnings` and `framework:pickup`
topics · the 40-subcommand CLI surface · the MCP parity-census ratchet · the preflight
*mechanism* (distinct from F9 — the concept is sound, only its comparator is wrong).

---

## 8. INVESTIGATE

| ID | Item | Data that would decide it |
|---|---|---|
| I1 | V3 has no production caller beyond demos; AEF doesn't call it | Whether AEF has a roadmapped adoption plan — **unreachable from this repo** (T-559) |
| I2 | 313 of 482 bypasses are `focus-drift` (164) + `human_sovereignty` (149) | Per-bypass reason breakdown and bypass **rate** vs commit volume. An Authority-Model question — see SQ-4 |
| I3 | 326/491 fabric cards (66%) have no edges | Whether `fw fabric blast-radius`/`impact` are actually consulted — needs invocation telemetry (§9) |
| I4 | `stale-waker-code` and `stuck-claims` logs are "firing" but **haven't appended in 34–36 days** despite installed daily cron | Per-canary heartbeat freshness — distinguishes "cron stopped" from "condition stable" |
| I5 | Guard-layer runtime vs the 10-min `doc-lint.yml` cap | **Actual CI job duration history.** My local runs were contended and are not a clean measurement — see §9 |
| I6 | 71 human-AC-blocked + 100 tasks ≥3 months old | How many ACs are `[RUBBER-STAMP]` vs `[REVIEW]` — see SQ-4/SQ-5 |
| I7 | Chat-arc 92% single-sender (871/950) among 5 senders | Role of sender `d1993c2c3ec44c94` — expected orchestrator status-posting, or a shortfall in the "fleet" framing? |

---

## 9. Data gaps that capped confidence

Each of these is an **ADD (instrument) candidate in its own right** — the prompt's rule is
that a gap blocking judgement is itself a finding.

| Gap | What it capped | What closing it unlocks |
|---|---|---|
| **No per-verb / per-tool invocation counters anywhere** | Every usage verdict on the 260 MCP tools and 40 CLI verbs. "Used?" was answered by *grepping for callers in this repo*, which cannot see a human or a peer project invoking a tool interactively. | The single highest-value instrument. It would convert the entire surface question from inference to measurement, and it is the one piece of evidence the pending T-2548 decision actually needs. |
| **T-559 boundary blocks reading peer hubs and consumer projects** | The external-consumer check (DELETE check 5) for every tool. This is *exactly* T-2548's blocking question IW-1. | Would let the 30-day-old sovereign decision on the 28 off-charter tools finally be made. |
| No CI run history (no GitHub access; pushing forbidden) | Whether the guard layer actually passes in CI, and whether the 10-minute `doc-lint.yml` cap is ever hit. | Confidence on guard-layer health. |
| No line coverage | Which load-bearing paths are untested — a REFACTOR precondition (behaviour must be covered before restructuring). | Safe refactor of `tools.rs`. |
| No unused-dependency tool (`cargo machete`/`udeps` absent) | Any dependency DELETE verdict. 310 lockfile packages unassessed. | A whole DELETE axis currently invisible. |
| BVP realization log not verified; healing events and per-task token telemetry not sampled | Whether shipped arcs delivered predicted value; where failures concentrate. | Would answer "did the substrate arcs pay off?" — directly relevant to arc-001/002/005/007 being stalled. |
| Guard-layer runtime not cleanly measured | Whether the layer fits its own CI budget. Two concurrent runs plus a live worker made the local timing unusable; a `pkill -f` pattern then matched and killed my own measuring shell. | Recorded honestly as unmeasured rather than asserted. |

**Ground-rule note.** Per "no data is not zero", none of the above is read as evidence of
low value. Where a gap exists the correct class is INVESTIGATE + instrument, not DELETE.

---

## 10. Contradictions found (recorded, not resolved)

1. **"Shipped" vs "live".** The project ships an explicit shipped-equals-live gate (T-2480)
   and three fleet-fitness canaries, yet the push-wake rail is measured DARK and V3 claims
   measure 0 across all 45 topics. The gate exists; the rails it guards carry no traffic.
2. **The preflight canary's own summary contradicts its contract.** Every recent entry ends
   `4 pass, 2 warn, 0 fail — substrate will run`, while the canary's contract treats any
   entry as a firing that dirties a one-bit channel. By the project's own T-2685 reasoning
   the canary has therefore been *deaf* for 74 days.
3. **The guard layer is simultaneously valuable and noisy.** 5 of 6 sampled static-check
   tasks produced genuine `crates/` fixes (real defects, real value); the preflight canary
   produced 74 days of unactioned output on a check that cannot pass.
4. **Binary staleness, two true readings.** Preflight says the binary is 174 versions behind;
   git says 1 product commit behind. Both are correct — they measure different things, and
   the check picked the comparator that cannot distinguish them.
5. **Non-goal #3 vs the live surface.** 28 live off-charter analytics tools remain, held in
   an allowlist the project itself calls "a ledger of an open question, not a permanent
   exemption" — open since 2026-08-20.
6. **Non-goal #2 vs `health:ring20-fedprobe`.** A `forever`-retention topic accruing ~88
   synthetic probe records/day, which both guards built for this class structurally cannot
   see (wrong name pattern for T-2252; 1,666 records against T-2562's 50,000 threshold).
7. **CLAUDE.md's own canary tally** says "all eighteen"; there are 21 canary sections, 21
   logs and 27 crontabs.
8. **The dispatch gate's documentation vs its behaviour** — "per session" vs a counter that
   never resets and that blocked attempts increment (§3, F-14).

---

## 11. Not reviewed

- Healing events, per-task/arc token telemetry, BVP realization log (budget).
- Handover (1702) and episodic (2435) corpora beyond audit-derived summaries (budget).
- Line coverage and dependency analysis (tooling absent — §9).
- Peer hubs' and consumer projects' own state (T-559 boundary).
- `.agentic-framework/` internals as a deletable surface — **vendored (G-062)**; local edits
  are erased by the next re-vendor, so it is upstream's scope, not this review's.
- `docs/reports/` (310 files) as a DELETE surface — these are per-task research artifacts
  mandated by convention C-001 ("the thinking trail IS the artifact"); pruning them is a
  governance decision, not a value finding.

---

## 12. Sovereign questions

These touch the charter, value-driver weights, gates or authority. **This review does not
propose changing them** — each is surfaced as a question with a recommendation.

| # | Question | Evidence | My recommendation |
|---|---|---|---|
| **SQ-1** | **The 28 live off-charter analytics tools: subtract, or amend the charter to sanction them?** | Inception T-2548 completed 2026-08-20 with all three questions `disposition: deferred`; agent recommendation on record is **GO to subtract**; 0 of 28 have any caller in this repo (measured); charter non-goal #3 names them as "removal candidates, not features". Blocking gate IW-1 (external consumers) is unanswerable from this repo. | **Subtract**, following the T-2471/T-2478 precedent — *but only after* IW-1 is cleared, which needs either a fleet-wide check from outside this repo or the per-tool invocation counter in §9. The decision has been open 30 days; the cheapest way to close it is to build the counter. |
| **SQ-2** | **Is V3 "claim work" still a core verb?** | Measured: 0 active claims across 45 topics; the only callers are demo/prover scripts; the AEF orchestrator does not use it. Against that: it is one of four verbs in the human-blessed charter sentence, and its exclusivity guarantee is currently unenforced (G-086/G-087, HIGH). | **Keep the verb, fix the guarantee, and stop extending it.** Its non-use reads as NEVER-WIRED + BROKEN, not NOT-WANTED — so the charter should not change. But a primitive with five observability layers and zero production callers should not receive more surface until something calls it. |
| **SQ-3** | **The budget gate's thresholds, given ~95.5k tokens of fixed overhead.** | CLAUDE.md is ~69.2k tokens and is auto-loaded in full; MCP descriptions are ~26.4k. The gate blocks source edits at 170k. 17 commits and 81 task references record work truncated by it. | **Do not weaken the gate** — it is doing its job. Reduce the overhead instead (§6). Changing a gate to accommodate a cost is the wrong direction; the cost is the defect. |
| **SQ-4** | **The 137 human-owned active tasks, 71 of them fully done and blocked only on human AC verification.** | Measured from frontmatter; all 71 carry `date_finished` and are legitimate T-193 partial-complete. | **A throughput decision only the human can make**: either schedule verification sessions, or narrow which ACs genuinely require human sign-off. An agent must not close these (Human Task Completion Rule). |
| **SQ-5** | **Four stalled arcs (arc-001, 002, 005, 007), the oldest in-progress since 2026-06-07.** | `fw audit` has warned 18× that arc-005 "mcp-slimming" has had no task commits in 30 days, 12× for arc-substrate-fitness. arc-005 is the arc that would reduce the MCP surface. | **Close or re-commit to each explicitly.** An arc that has been `in-progress` for three months with no commits is a recorded intention, not a plan; leaving it open is what makes the audit warning unreadable. |

---

## 13. Reproduction

```bash
# snapshot + baseline
cargo test --workspace
bash scripts/check-cron-install-drift.sh --json
bash scripts/check-task-id-collisions.sh --json

# the usage measurements that carry this report
termlink list --json                                   # V4: 40 ready sessions
termlink channel state agent-presence --json           # V1: 1 sender, 17.4h window
termlink channel claims-summary --all --json           # V3: 0 active claims / 45 topics
termlink channel list --json                           # 45 topics, 20 are test residue

# the activity collapse
git log --all --since=2026-09-01 --oneline -- crates/ | wc -l    # -> 1
git log --since=2026-09-01 --name-only --pretty=format: | grep -v '^$' | sed 's|/.*||' | sort | uniq -c | sort -rn

# the cost
wc -c CLAUDE.md                                        # 276,749 (~69.2k tokens)
```

