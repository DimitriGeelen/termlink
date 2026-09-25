# Value review — repo — 2026-09-25 — Round 2 evidence file

Round 2 of 4 in the [Review, Audit, procAsFit] × 4 orchestrated sequence (T-3093,
run record `.context/runs/T-3093-review-audit-procasfit-x4.yaml`, step `R2S1`).

**Role note:** GATHERER and JUDGE are the same worker in this run (a single `claude -p`
step, per this orchestrated sequence's design — no separate worker/model was dispatched
for Phase 4-5). Per the prompt's own rule this drops confidence one level from what is
stated below — already applied in the report file's confidence column.

**Scoping decision (stated up front, not hidden):** This is at least the **ninth**
value-review pass over this repo, and the second in this orchestrated sequence. Round 1
(R1S1, `docs/reports/VALUE-REVIEW-repo-2026-09-25-R1*.md`) already ran Phase 0/1
(yardstick + data-map, unchanged) and Phase 2/3 at delta-scope. This round runs the same
way: verify what changed since R1S1 (~7 hours of elapsed work: R1S2's audit-remediation
cycle 1 and R1S3's procAsFit slice), close out R1S1's explicitly carried-forward
INVESTIGATE items, and report genuinely new evidence only. Full phases 2/3 (whole-repo
re-inventory) are **not** re-run — see "Not reviewed" below.

## 1. Yardstick (re-confirmed, unchanged)

- Purpose (docs/CHARTER.md, human-blessed, unchanged since R1S1): cross-terminal /
  cross-machine agent session-control and durable-messaging substrate.
- `policy/value-drivers.yaml` v3, unchanged.
- No contradiction found against the new evidence gathered this round.

## 2. Data availability map — delta since R1S1 (2026-09-25 ~01:15)

| Source | R1S1 status | R2S1 status | Evidence |
|---|---|---|---|
| Invocation audit (`invocation-audit.jsonl`) | EXISTS, live, 4 records, <1h window | **EXISTS, but STALLED — still exactly 4 records, ~7h later** | see §3 F1 below |
| arc-008 (audit remediation) | not yet created | **EXISTS — created R1S2, 33 tasks, 8 closed cycle 1** | `.context/arcs/arc-008.yaml`, `e0f9158dc`/`a60974684` |
| arc-009 (value-review execution) | 7/18 complete (fixed 18-task scope as of R1S1) | **2 more owner:agent tasks closed by R1S3 (T-2998, T-3010) — see note** | R1S3 handback; `cedd6b514` |
| Human-AC review queue | 144 pending, oldest 147d | **still 144, oldest still 147d — unchanged, as expected** (neither R1S2 nor R1S3 touched human-owned items, correctly) | `fw review-queue` re-run this round |
| Long-lived `termlink mcp serve` processes | ~30 noted, not classified | **classified this round — see §3 F3** | `ps -eo pid,tty,etimes,cmd` |
| DEFER inception `revisit_at` coverage | not measured | **measured this round — see §3 F4** | per-task frontmatter read |
| R1-F1/F1b unused Cargo deps | proposed DELETE, not executed (halted at [ASK]) | **still present, unexecuted** (expected — no approval step ran) | `crates/termlink-{protocol,test-utils}/Cargo.toml` re-read |
| G-087-class "budget cache unsafe to read raw" | 3 prior task closures (T-2950 2026-09-09, T-3018 2026-09-20, T-3034 2026-09-21) covering the *missing subcommand* framing | **recurred a 4th time this session in a NEW framing: concurrent orchestrated `claude -p` workers, not a single interactive session** — see §3 F2 | R1S2 + R1S3 handbacks (independent), this session's own checkpoint.sh read |

## 3. New/updated findings this round (verified, with citation)

### F1 — Invocation-audit sink has gone stale again, ~7 hours after R1S1 confirmed it live

R1S1 (F3) found `/var/lib/termlink/invocation-audit.jsonl` freshly live with 4 records,
all from the hour before that check, and predicted it would hold enough history by R2S1
to start being usable evidence for the C-29/C-30/C-31 "structurally invisible usage"
findings.

This round: `wc -l /var/lib/termlink/invocation-audit.jsonl` → still **4**. `head -1` /
`tail -1` show the SAME two timestamps R1S1 already cited
(`1790289251771`..`1790289258892`, both within the same ~7-second window). **Zero new
records in the ~7 hours since**, despite substantial `fw`/`termlink` CLI and MCP activity
in that window (R1S2's full `fw audit`/`fw doctor` runs, 8 task closures, multiple git
operations; R1S3's task work; this very session's own tool calls).

Non-use diagnosis (applied to the sink, not a downstream consumer — there is no consumer
yet, this is Phase-1 "does the writer even fire" territory):
- **Not the same reading as R1S1's finding.** R1S1 established reading B (never wired) had
  been resolved by a binary reinstall; this is a **new** stall, not a continuation of the
  old one — the writer fired for exactly one MCP session's startup calls
  (`hub_status`/`doctor`/`dispatch_status`/`topics`, all read-only introspection calls
  typical of a single `fleet doctor`-style sweep) and then never fired again despite heavy
  subsequent MCP tool use in this exact terminal (this session alone has made 25+ MCP-tool
  calls). This reads as **A (broken) or B (never wired) again, narrower this time**: either
  the sink only instruments a specific subset of MCP tool names (the four observed) and
  silently no-ops for the rest — consistent with reading B (partial wiring, not full) — or
  it wrote once per hub-process-lifetime rather than per-call — consistent with reading A
  (a bug in the write trigger).
- Could not distinguish A vs B further within this round's budget without reading
  `crates/termlink-mcp/src/` for the instrumentation call sites, which is out of scope for
  a delta pass (Phase-3 evidence gathering, not Phase-6 code archaeology). Recorded as
  **INVESTIGATE** in the report, not classified further.
- This directly affects confidence on any future C-29/C-30/C-31 usage verdict: if the sink
  only ever captures ~4 events total regardless of elapsed time, it cannot become the
  "enough history" data source R1S1 predicted. That prediction is now falsified by direct
  re-measurement — worth stating plainly rather than silently dropping the thread.

Kind: usage / structure (instrumentation reliability). Axis: ADD (REPAIR or WIRE,
undetermined without a code read) — this is exactly the kind of "channel cannot report its
own failure" case the ground rules warn about: nothing failed loudly, the file just stopped
growing.

### F2 — The G-087-class "stale shared budget cache" defect has now recurred a 4th time, in a genuinely new context this time

Three prior task closures already exist for framings of "the only certified-safe budget
read (`checkpoint.sh budget`) doesn't exist, so the documented `/resume` instruction can't
be followed": T-2950 (2026-09-09), T-3018 (2026-09-20), T-3034 (2026-09-21) — all
`work-completed`, all filed upstream per G-062 (vendored code, `.agentic-framework/`), not
patched locally.

This session's run record (`cross_step_findings`) shows R1S2 and R1S3 **independently**
hit a *different* instance of the same underlying class: `.context/working/.budget-status`
read **504K tokens** for both dispatched workers when their real per-session usage
(via `checkpoint.sh status`, which reads the actual transcript) was ~169K. That is not the
"subcommand doesn't exist" framing of the three prior closures — it is a **plausible
false-panic** read (0-vs-huge-number in the wrong direction from the G-087/T-222 precedent
CLAUDE.md cites, which was 0-vs-large the *other* way) surfacing specifically when
**multiple `claude -p` workers are dispatched concurrently under one orchestrated run**
and each writes/reads the same shared cache file without any worker-identity scoping.

Verified this round: `cat .context/working/.budget-status` right now (single active
session, no concurrent dispatch in flight) reads `{"level": "ok", "tokens": 189468, ...}`
— plausible and roughly consistent with this session's own `checkpoint.sh status` read
(~210K at time of check, few tool calls apart) when only one worker is active. The defect
is specifically a **concurrency** hazard, not a permanent brokenness — which is why it did
not reproduce for this single-worker step and why it is a *new* instance of the class
rather than a duplicate of T-2950/T-3018/T-3034 (which were about single-session
unavailability of a documented verb, not about multi-worker races).

Checked: no active task or `.context/project/concerns.yaml` / `gaps.yaml` entry currently
names this concurrency framing specifically (`grep -rn "budget-status" concerns.yaml
gaps.yaml` → no hits; `grep -rl "budget-status" .tasks/active/` → no hits). CLAUDE.md's own
Context Budget Management section (`grep -n "budget-status" CLAUDE.md`) documents the
cache's existence and the `checkpoint.sh status` fallback but does **not** warn that the
cache is unsafe to trust when multiple orchestrated workers run concurrently — the exact
gap that let two separate workers rediscover it independently in one afternoon rather than
one of them finding a warning already there.

Kind: structure / reliability (Directive #2). Axis: **ADD (SURFACE)** — the underlying
mechanism (per-worker transcript read via `checkpoint.sh status`) already works and is
already the documented fallback; what's missing is a loud, discoverable warning at the
point orchestrated/dispatched workers would look (this run record's own "carried_fixes"
section already does this locally for *this* run — the gap is that it is not in CLAUDE.md
itself, so the next orchestration outside this run record starts blind again).

### F3 — Long-lived `termlink mcp serve` processes: 30 of 31 classify as legitimate, 1 anomalous

R1S1 flagged this as an open INVESTIGATE item ("legitimate open sessions or leaked
processes... needs a liveness cross-check"). This round ran that check:
`ps -eo pid,tty,etimes,cmd | grep 'termlink mcp serve'` → **31** processes (up from "~30"
noted in R1S1, consistent with normal session churn on a host this busy, not unbounded
growth — R1S1 also said "noted, not asserted").

- **30 of 31** are each pinned to a **distinct, named `pts/N`** tty. Cross-referenced
  against `who` (426 active login sessions) and `ls /dev/pts/` (848 allocated pts) on this
  host — a pts-attached long-lived process is consistent with a still-open interactive
  terminal session, exactly R1S1's "legitimate open session" reading. **Resolved: not a
  leak** for these 30.
- **1 of 31** (pid 1895596) has **no controlling tty** (`tty` column shows `?`) and the
  longest uptime of the set (`etimes` 115117s ≈ 32 hours). A process with no controlling
  tty cannot be an open interactive terminal session by definition — it is either a
  legitimately headless/detached MCP server (e.g. spawned by a background dispatch,
  systemd unit, or an agent launched via `claude --bg` per the T-2876 prover's own finding
  that such sessions exist and are addressable) or an orphaned leak. Could not distinguish
  further within budget (would need `/proc/1895596/` parent-pid history, which may no
  longer resolve for a 32h-old process) — **not classified**, carried to the report as a
  named INVESTIGATE item rather than asserted as either reading.

Kind: usage / structure. Axis: mostly KEEP (confirms 30/31 as legitimate, closing R1S1's
open question); the 1 anomalous process is INVESTIGATE, not DELETE — a single unclassified
process is not "unbounded growth" and does not meet the DELETE bar (no positive reason
established, and it may simply be a currently-live dispatched worker like this very run).

### F4 — DEFER-inception `revisit_at` coverage: measured directly, mostly N/A rather than missing

R1S1 flagged "whether the 9 DEFER inception decisions... carry `revisit_at`" as a plausible
R2S1 target. Re-run `fw review-queue` this round: **12** pending inception decisions now
(was 12 at R1S1 too, contents shifted — 3 new GO items T-3060/T-3075/T-3076 entered, none
resolved), of which **6 carry a `DEFER` auto-recommended verdict** from `fw task verify`
(T-2486 54d, T-2725 40d, T-2879 22d, T-2918 17d, T-2753 6d, T-2992 5d).

Direct frontmatter read of all 6: only **T-2486** carries `revisit_at` (`2026-11-01`,
correctly formatted per T-1451). The other 5 do not.

**This is not itself a defect** — per CLAUDE.md, `revisit_at` is a field the *human* sets
at the moment of running `fw inception decide T-XXX defer`, and none of these 5 have
actually been decided yet (`fw review-queue`'s "DEFER" here is `fw task verify`'s
**recommendation**, not a recorded decision — the task remains `started-work`/`captured`
pending the human's Tier-0-gated `fw inception decide` call). T-2486 having `revisit_at`
already, while still appearing on the pending-decision list at 54 days old, is the
interesting data point: it suggests T-2486 may have been speculatively annotated with a
`revisit_at` before formal decision, or decided once and reopened — **not resolved within
this round's budget**; flagged as a narrow follow-up (read T-2486's `## Decisions` section)
rather than asserted.

Kind: process / governance. Axis: no class change — R1S1's open question is now answered
("do the review-queue items even have a mechanism to carry revisit_at correctly" — yes,
confirmed on the one item that has gone through the field being set), but surfaces a
smaller, cheaper follow-up (T-2486's apparent revisit_at set pre-decision) worth a single
targeted read next round rather than action now.

## 4. Contradictions / notes (recorded, not classified)

1. Version skew persists and has grown by one more data point: `VERSION` now reads
   `0.12.31` (was `0.12.25` at R1S1), `git describe` reads `v0.12.0-33-g4acd93b94` (was
   `-26-g...`) — both advanced consistently with the 7 intervening commits, so this is
   **not** new drift, just the same pre-existing three-way skew (noted at R1S1 as
   `~/.local/bin/termlink` lagging) moving forward in lockstep on the two sources that do
   track HEAD. Not re-verified whether `~/.local/bin/termlink` itself updated — out of this
   round's budget, low value (R1S1 already established the *hub* binary, the one that
   matters for the invocation-audit and MCP-serving paths, is current).
2. `fw review-queue`'s DECISIONS list includes **T-3006** ("Investigate task
   completion-rate drop") with an auto-recommended **NO-GO** verdict, age 5 days — this is
   the same task R1S3's handback named as arc-009's "one real measured-cost Q2 candidate"
   to prioritize next. A NO-GO recommendation sitting undecided is worth the next audit
   step (R2S2) noting, not resolving here (Tier-0/human-only decide gate).

## 5. Not reviewed this round (explicit, per ground rules)

- Full Phase 2 whole-repo inventory — unchanged from the 2026-09-19 consolidated set and
  R1S1's decision not to re-walk it; no new activity signal in this round's own evidence
  suggests it needs re-walking yet.
- Code-level root cause for F1 (invocation-audit stall) — would require reading
  `crates/termlink-mcp/src/` instrumentation call sites; explicitly out of Phase-3 scope
  for a GATHERER pass (that is Phase-6 execution work, gated on approval).
- `/proc`-level parent-process archaeology for the one anomalous headless `mcp serve`
  process (F3) — 32h-old process, parent chain likely already unwound; not attempted.
- T-2486's `## Decisions` section (F4 follow-up) — flagged, not read, budget discipline.
- JS/Python dead-code tooling (knip/vulture/jscpd) — still ABSENT, unchanged from R1S1,
  not re-flagged as new since no new activity touched that gap this round.
- Full baseline (`cargo test --workspace`, guard-layer run) — not re-run; no code changed
  in a way that would invalidate R1S1's decision to skip it, and R1S2's own audit re-runs
  already exercised the relevant surfaces this session.

Budget consumed this round: light-to-moderate — Phase 0/1 re-confirmation, ~10 targeted
greps/reads, 2 live process/file inspections (`ps`, `wc -l` on the audit sink), one
`fw review-queue` re-run. Well inside the 200k allowance; total session context at write
time ~210K tokens (~26% of window).
