# PROJECT VALUE REVIEW — TermLink (whole repo)

**Task:** T-2971
**Snapshot taken:** 2026-09-16T21:08:03Z (before the review generated any of its own data)
**Report written:** 2026-09-17
**Evidence file:** `docs/reports/VALUE-REVIEW-repo-2026-09-16-evidence.md` (13 sections, ~447 lines)
**Status:** Phase 5 — awaiting per-item human approval. **Nothing has been deleted, restructured, or built.**

> **Location note.** This report belongs at `docs/reports/VALUE-REVIEW-repo-2026-09-17.md`.
> It is here because the budget gate is latched critical and permits writes only to
> `.context/`, `.tasks/`, `.claude/`. Move it once the gate clears.

---

## 0. METHOD AND PROVENANCE

**Role separation.** GATHERER (Phases 0–3) collected evidence read-only and classified nothing.
JUDGE (Phases 4–5) classified from the evidence file, `docs/CHARTER.md`, `.agentic-framework/policy/value-drivers.yaml`,
and the parked NON-USE DIAGNOSIS section of T-2971 — and re-gathered nothing.

**Separation was contextual, not by model family** (IW-5). Per the review's own rule, **every
confidence below has been dropped one level. No item reaches HIGH.** This is stated up front
because it is the single largest qualifier on the whole report.

**What was deliberately not done:**

- No item was classified before the Phase 1 [ASK] was answered.
- No test, gate, check, or audit is proposed for deletion, weakening, or narrowing.
- No "no telemetry" was converted into "no use."
- No vendored file under `.agentic-framework/` was patched (G-062).
- No gate was bypassed. Work sits staged rather than committed.

**Three self-flagged measurement hazards were honoured** — each was a GATHERER finding that
corrected an earlier GATHERER claim:

1. The **64:1 governance-to-product churn ratio dissolves** on decomposition — 84% of `.context/`
   churn is machine-written, and `crates/` took 247 commits in 90 days. Not used.
2. **"Two methods agree on 75 orphans" is not corroboration** — both searched code/config,
   neither searched arc YAML, so they shared a blind spot. ≥5 of the 75 are provably false
   positives. Not used as a count.
3. Test baseline is **3,618 tests / 24 suites**, not the mid-run read of 2,962 / 10.

---

## 1. YARDSTICK

Confirmed in Phase 1: **`docs/CHARTER.md` plus `.agentic-framework/policy/value-drivers.yaml`.**
Whole repo. No external data (IW-3 = NONE).

The charter's four verbs — **discover · exchange durable messages · claim work · control terminal
sessions** — and five non-goals, of which #2 ("not a durable database / system of record") and
#3 ("not a social / engagement platform") do live work in this report.

Value-driver weights are **§ACD sovereign** and are not touched here. See G3.

---

## 2. SUMMARY

| Class | Count |
|---|---|
| KEEP | 8 (classes, not items) |
| **DELETE** | **1** |
| REFACTOR | 6 |
| ADD | 9 |
| INVESTIGATE | 5 |

**KEEP** — the guard layer itself (109 PASS / **0 ERROR**: every member that should run, ran);
the 3,618-test suite; the 5 zero-debt allowlists; the 81 non-report docs (100% wired); the
`test-` twin convention (not duplication); `docs/reports/` per-task artifacts; the
`*-freshness.sh` family; CHARTER.md and its drift canary.

**The DELETE axis is nearly empty, and that is the finding.** With no per-verb telemetry
anywhere in the repo, reading **D (UNMEASURED)** cannot be ruled out for any of the 214 live
tools. The one DELETE that survives rests on **supersession with a recorded replacement**, not
on absence of use.

---

## 3. TOP 3 PER AXIS

**DELETE**

1. The **46 deprecated-but-live tools** — every one carries a supersession target; a planned
   cut was left half-done. Items 2 and 3 are empty: nothing else qualifies.

**REFACTOR**

1. `.fleet-doorbell-mail` canary **overwrites instead of appends** — a one-line fix that
   restores a history which currently does not exist at all.
2. **Guard-layer runtime**: documented "seconds", observed **>20 min**, wired into every push
   and PR. A guard nobody will run casually is a guard that stops guarding.
3. **CLAUDE.md at 3,283 lines**, 822 of them below an unmarked `fw upgrade` split. The
   always-loaded file grew ~790 lines since August and the destroyed half keeps growing.

**ADD**

1. **Per-verb invocation counters** — the single absence capping the entire DELETE axis. Until
   it exists, this review cannot be repeated with more force than it was run with.
2. **Timestamps on the 6 of 7 canary logs that emit none** — cheapest high-leverage fix in the
   evidence. The one canary that does timestamp is the one whose evidence was usable.
3. **Repair the two dark charter verbs** — `.122` presence-subscribe times out at 30s (verb 1,
   T-2970) and the `agent-chat-arc` rail is NEVER-ACKED at lag 1451 (verb 2).

---

## 4. FINDINGS TABLE

Confidences are **post-drop**. "Reading" is the NON-USE DIAGNOSIS letter (A BROKEN · B NEVER
WIRED · C UNDISCOVERABLE · D UNMEASURED · E NOT WANTED).

| # | Item | Class | Reading | Evidence | Counter-evidence | Conf | Proposal | Size | Rev? | Risk if wrong | Expected effect |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 46 deprecated live tools | **DELETE** (scheduled) | **E via supersession**, *not* non-use | §4; T-2971 §NON-USE | All 46 still callable; no removal date; T-1415/1426/1432 `started-work` | MED | Set a removal date under T-1166; cut after one release of notice | med | yes (git) | External consumer breaks — unmeasurable, IW-3=NONE | `help --json` 260→214; `deprecated`→0 |
| 2 | 28 off-charter live tools | INVESTIGATE | D; intent-E blocked | §4 | Acknowledged pending **T-2548**, an open *human* decision | LOW | Sovereign question **G1** — no agent proposal | — | — | Pre-empting sovereignty | n/a until T-2548 |
| 3 | 75 "true orphan" scripts | INVESTIGATE | **C** broadly; D unfalsifiable | §10a, §12 | **≥5 false positives** (arc `demo_evidence`); 2 origin tasks ACTIVE | LOW | Re-run the sweep **including arc YAML**, then diagnose per family | med | yes | Deleting live guards — hazard 2 already proved this once | Corrected count published; families bucketed A–E |
| 4 | `.fleet-doorbell-mail` overwrites its log | **REFACTOR** | — (defect) | T-2971 §NON-USE | none | MED | `>` → `>>`, per the shared idiom | small | yes | none material | Log accumulates ≥2 entries in a week |
| 5 | 6 of 7 canary logs untimestamped | **ADD** (instrument) | **D** | T-2971 §NON-USE | none | MED | Stamp `=== <ts> ===` on every entry | small | yes | none | "Firing" vs "stale noise" decidable from the file alone |
| 6 | `.substrate-preflight` firing **72 days, worsening** | **ADD (REPAIR)** | **A** | 2026-07-06→09-16; 5 pass/1 warn → 3 pass/3 warn | none | MED | Fix binary staleness, *then* truncate | small | yes | Truncating hides an unfixed cause | Next run 6 pass/0 warn; empty 7d |
| 7 | `.framework-pickup` ack stuck at offset 42, backlog →120 | **ADD (REPAIR/WIRE)** | **A + B** | T-2971 §NON-USE | none | MED | Triage offsets 43–120, *then* `--ack` | med | yes | Acking un-triaged inbound **is** the G-063 miss | Watermark ≥ newest offset |
| 8 | `.waker-liveness` + `.stale-waker-code`: RAIL DARK, 4 dead wakers | **ADD (REPAIR)** | **A** | byte-identical repeats, same 4 pids | none | MED | `fleet-rearm-wakers.sh`, then truncate | small | yes | Re-arm without root cause → recurs | ≥1 LIVE listener carries `pty_session` |
| 9 | `.stuck-claims`: **9 of 11 topics are `substrate-drain-demo*`**, `active=0` | **REFACTOR** | E (for the *topics*, not the guard) | T-2971 §NON-USE | T-2568 (alarm-fatigue tuning) still `captured` | LOW-MED | Reap demo topics; resolve T-2568 — **do not weaken the guard** | small | yes | Narrowing hides a real stuck claim | `stuck_count` 11→≤2 |
| 10 | `agent-chat-arc` **NEVER-ACKED**, lag 1451 | **ADD (REPAIR)** | **A/B** | §12 | none | MED | Find the identity's consumer; wire or retire the rail | med | yes | Retiring a rail someone reads | Lag→0 or identity removed |
| 11 | `.122` presence-subscribe 30s timeout (T-2970) | **ADD (REPAIR)** | A | §8 | none | MED | Fix — **charter verb 1 is dark on a fleet hub** | needs-design | yes | Hub-side change on a stale host | `subscribe agent-presence --hub .122` <5s |
| 12 | Guard layer: "seconds" documented, **>20 min** observed | **REFACTOR** + ADD(timings) | — | §11 | Run completed, 0 ERROR | MED | Emit per-member timings; fix/parallelize the tail | med | yes | Parallelism masks a hang as PASS | Wall <5 min; timings in `--json` |
| 13 | CLAUDE.md 3,283 lines, 822 below unmarked split | **REFACTOR** | — | §10c | Vendored boundary is upstream's (G-062) | MED | Mark the boundary; move project content above it | med | yes | A wrong cut loses operator entry points | Below-split stops growing; `fw upgrade` dry-run loses 0 project lines |
| 14 | `fw task_list --arc <slug>` **silently returns all 245** | **REFACTOR (upstream)** | — | T-2971 §NON-USE | Vendored (G-062) — file, don't patch | MED | File upstream; error on unknown filter | small | yes | none | Filter returns a subset, or errors |
| 15 | No BVP realization log | **ADD** (instrument) | **D** | §7 | none | MED | Write `.context/audits/bvp-realization.jsonl` at arc close | med | yes | Ceremony nobody reads | "Did arc X deliver?" gets a source |
| 16 | **No per-verb telemetry** | **ADD** (instrument) | **D** — the cause of D everywhere | §7 | Must not drift into non-goal #2 — keep counters bounded | MED | Bounded in-memory counters on the existing governor surface | med | yes | Retention creep into "system of record" | A repeat of this review can falsify non-use for 214 tools |
| 17 | Bypass log **40% unattributed**; 93 batch-closes | **ADD** (mandatory attribution) | — | §6 | NOT established whether the 93 were individually evidenced | MED | Require a non-empty reason at write time | small | yes | none — strengthens | New entries 100% attributed |
| 18 | 2 stalled arcs (`mcp-slimming`, `arc-substrate-fitness`) | **REFACTOR** | C/D | §5 | `arc-substrate-fitness` flagged in only 4 of 14 audits | LOW-MED | Close, or re-activate with a dated next slice | small | yes | Closing live work | `fw audit` stops flagging both |
| 19 | `fabric-register-workflow.sh` | INVESTIGATE / ADD(WIRE) | **B** | §10a, §7 (no `*.bpmn` found) | Fabric cards exist | LOW | Decide: wire to a BPMN path, or retire deliberately | small | yes | Deleting a half-built feature someone intends | Referenced by a runner, or retired with a reason |
| 20 | 155 + 155 pipefail / unbounded-rpc allowlist entries | REFACTOR (work down) | — | §3b | Acknowledged and ledgered **on purpose** | LOW-MED | Ratchet — **never bulk-clear** | needs-design | yes | Clearing to look clean | Counts monotonically decrease |
| 21 | 70 partial-complete + 136 human-owned active | INVESTIGATE | D | §5 | All 70 legitimate T-193; `firing_count: 0` | MED | Measure age per task **before** proposing anything | small | yes | Pressure to batch-close (CLAUDE.md forbids) | Age histogram exists; oldest named |
| 22 | `.claude/` 44 GB; `.agentic-framework.rollback/` 25 MB untracked | INVESTIGATE | — | §1 | **Cost is not value evidence** — size alone justifies nothing | LOW | Ask for a transcript retention policy | small | yes | Deleting recoverable session history | A stated retention window exists |

---

## 5. INVESTIGATE — what would decide each

- **#2 (28 off-charter tools)** — T-2548's human decision. Nothing else decides it.
- **#3 (orphans)** — a sweep that searches `.context/arcs/*.yaml` as well as code and config,
  plus origin-task status per family. Both prior methods missed arc YAML.
- **#19 (`fabric-register-workflow.sh`)** — does any ratified BPMN workflow exist or is one
  planned? §7 records `*.bpmn` ABSENT. If none is planned this is reading B and should be
  retired deliberately, not left ambiguous.
- **#21 (human-review backlog)** — `date_finished` age per partial-complete task. Until then,
  "the human step is the bottleneck" is a shape, not a measurement.
- **#22 (disk)** — a stated retention intent. Size is cost, not value.

---

## 6. DATA GAPS THAT CAPPED CONFIDENCE

1. **No per-verb telemetry (§7).** Reading D cannot be ruled out for **any** of the 214 live
   tools. This is why there is exactly one DELETE in this report, and why it rests on
   supersession rather than absence of use.
2. **No BVP realization log (§7).** "Did shipped arcs deliver?" has no source, so no item could
   be ranked on realized value — only on charter fit and cost.
3. **IW-3 = NONE.** Consumer breakage on any DELETE is unmeasurable from inside the repo.
   Item 1's risk column is therefore honest, not reassuring.
4. **6 of 7 canary logs untimestamped.** "Firing continuously" vs "accumulated noise" was
   undecidable from the files; contents had to be read to split the 7 into three classes.
5. **A channel cannot report its own failures (§7).** TermLink delivery evidence comes from
   TermLink's own bus. Items 10 and 11 are bounded by this.

---

## 7. CONTRADICTIONS FOUND

1. **Docs vs reality** — CLAUDE.md says the guard layer runs in "seconds"; observed >20 min
   across two invocations, and CI runs it on every push and PR.
2. **Rule vs behaviour** — CLAUDE.md: *"No 'batch-close stale tasks' — each task needs
   individual evidence."* The bypass log holds **93 `Phase A batch close`** entries. Whether
   they were individually evidenced is **not established**: the log records a reason string,
   not evidence.
3. **Convention vs implementation** — "empty log = healthy" is documented 21 times, yet **7 of
   21 logs are non-empty**, and T-2685 (same file) states a dirty log is *"deaf until someone
   truncates it by hand."*
4. **Convention vs implementation** — `.fleet-doorbell-mail` overwrites where every sibling
   appends. It structurally cannot show history.
5. **Charter vs registry** — non-goal #3 says "not a social / engagement platform"; the
   registry still names `agent_engagement_metrics`, `agent_rankings`, `agent_stats`,
   `agent_poll`. Acknowledged, pending T-2548.
6. **Directive #2 vs tooling** — `fw task_list --arc` silently returns all 245 tasks. A wrong
   answer, not an error.
7. **Charter verbs vs state** — verb 1 (discover) is dark on `.122`; verb 2's `agent-chat-arc`
   rail is NEVER-ACKED at lag 1451.

---

## 8. SOVEREIGN QUESTIONS

Questions only. No agent action is requested or implied by any of these.

- **G1 — the 28 off-charter tools.** T-2548 is your open decision. *Recommendation:* deprecate
  on item 1's schedule rather than delete outright, so the ledger empties without an
  unmeasurable consumer break.
- **G2 — the 93 batch closes.** Were they individually evidenced, or is the written rule the
  one that should change? *Recommendation:* keep the rule, add mandatory attribution (item 17)
  so the question stops being re-askable.
- **G3 — value-driver weights.** F-RECALL 6 and F-AUTONOMY 4 both carry `retire_when`
  conditions that are free text and not auto-enforced. Worth checking whether either has now
  been met. §ACD-gated — **nothing proposed.**
- **G4 — item 1's removal date.** Cutting 46 tools is a compatibility decision with
  unmeasurable external blast radius. *Recommendation:* one release of notice. Yours to set.
- **G5 — transcript retention.** 44 GB of `.claude/`. *Recommendation:* state a window. Size
  was not treated as a value argument anywhere in this report.

---

## 9. WHAT REMAINS UNREVIEWED

Stopped at budget, per the protocol. Not reviewed:

- **Per-crate source quality** inside `crates/` — the review reached the tool surface and the
  guard layer, not the implementation behind them.
- **The 155 + 155 allowlist entries individually** (item 20) — counted, never read one by one.
- **Task bodies** — 245 tasks were counted and bucketed by status, not read.
- **`.agentic-framework/` internals** — out of scope by G-062; only its *interface* to this
  repo was examined.
- **Anything requiring external data** — IW-3 = NONE was the confirmed parameter.

---

## 10. APPROVAL LEDGER

**Research is not authorization.** Nothing proceeds without a per-item decision here.

| # | Item (short) | Class | Decision | Reason |
|---|---|---|---|---|
| 1 | 46 deprecated tools — set removal date | DELETE | ☐ approve ☐ reject ☐ modify | |
| 4 | `.fleet-doorbell-mail` `>` → `>>` | REFACTOR | ☐ approve ☐ reject ☐ modify | |
| 5 | Timestamp the 6 canary logs | ADD | ☐ approve ☐ reject ☐ modify | |
| 6 | Repair substrate-preflight (72d) | ADD | ☐ approve ☐ reject ☐ modify | |
| 7 | Triage pickup 43–120, then ack | ADD | ☐ approve ☐ reject ☐ modify | |
| 8 | Re-arm the 4 dead wakers | ADD | ☐ approve ☐ reject ☐ modify | |
| 9 | Reap `substrate-drain-demo*` topics | REFACTOR | ☐ approve ☐ reject ☐ modify | |
| 10 | `agent-chat-arc` rail: wire or retire | ADD | ☐ approve ☐ reject ☐ modify | |
| 11 | `.122` subscribe timeout (T-2970) | ADD | ☐ approve ☐ reject ☐ modify | |
| 12 | Guard-layer timings + runtime | REFACTOR | ☐ approve ☐ reject ☐ modify | |
| 13 | CLAUDE.md split boundary | REFACTOR | ☐ approve ☐ reject ☐ modify | |
| 14 | File `--arc` silent-filter upstream | REFACTOR | ☐ approve ☐ reject ☐ modify | |
| 15 | BVP realization log | ADD | ☐ approve ☐ reject ☐ modify | |
| 16 | Per-verb telemetry | ADD | ☐ approve ☐ reject ☐ modify | |
| 17 | Mandatory bypass attribution | ADD | ☐ approve ☐ reject ☐ modify | |
| 18 | Close or re-activate 2 stalled arcs | REFACTOR | ☐ approve ☐ reject ☐ modify | |
| 20 | Ratchet the two 155-entry allowlists | REFACTOR | ☐ approve ☐ reject ☐ modify | |
| 3 | Re-run orphan sweep incl. arc YAML | INVESTIGATE | ☐ approve ☐ reject ☐ modify | |
| 19 | Decide `fabric-register-workflow.sh` | INVESTIGATE | ☐ approve ☐ reject ☐ modify | |
| 21 | Age histogram for the 70 | INVESTIGATE | ☐ approve ☐ reject ☐ modify | |
| 22 | Transcript retention policy | INVESTIGATE | ☐ approve ☐ reject ☐ modify | |
| 2 | 28 off-charter tools | — | **sovereign, G1** | routed to T-2548 |

---

## 11. BLOCKERS ON THIS REVIEW ITSELF

Both need a human command; neither was bypassed.

1. **The budget gate is latched critical and cannot clear itself.** It reported 294,525 →
   300,486 → 319,736 tokens, rising *across* a `/compact` boundary, because it sizes the
   session transcript — which only grows. It cannot distinguish "context full" from "long
   session already compacted". The remedy is unreachable from inside: `checkpoint.sh reset` is
   itself Bash, which the gate blocks. This is the inverse of G-087, and more expensive — a
   falsely-healthy read costs a surprise; a falsely-critical latch costs the session.
2. **Three-gate commit deadlock.** The budget gate permits only commit/handover; the inception
   commit-limit refuses further exploration commits until a decision is recorded; recording it
   is Tier 0. Told to commit, forbidden to commit.

```
cd /opt/termlink && .agentic-framework/agents/context/checkpoint.sh reset
cd /opt/termlink && fw inception decide T-2971 go --rationale 'Review proceeds under confirmed scope; authorizes no delete/refactor/add — Phase 5 proposals remain individually gated.'
cd /opt/termlink && git commit -m "T-2971: guard baseline + Phase 1 dispositions + Phase 5 report"
```

Also observed, same family of documented-but-absent affordances: `checkpoint.sh budget` (the
verb `/resume` instructs) does not exist in this build, and `--no-heartbeat` is unknown to
`check-pickup-deferred-freshness.sh` and `check-receiver-ack-lag.sh`.
