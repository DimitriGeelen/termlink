# T-3093 R3S1 handback (Review, round 3 of 4)

**Status:** COMPLETE — halted at Phase 5 [ASK] per this run's own `known_gates` entry
("The value-review prompt HALTS at PHASE 5 [ASK] for per-item human approval before
PHASE 6 EXECUTE. A review step that halts there is COMPLETE, not failed"). Same shape as
R1S1/R2S1.

## Orientation

- Re-read the run record + `git log --oneline -20` at start of this step (carried fix #2).
  Confirmed nothing landed between R2S3's close (`fdcedbe02`) and this step's dispatch
  other than the orchestrator's own bookkeeping commit — no moving target to reconcile.
- Read R2S3's fed handback plus R1S1's and R2S1's own value-review reports/evidence files
  in full before gathering anything new, per the delta-round convention both prior rounds
  established (this is at least the 10th value-review pass over this repo, 3rd in this
  sequence — re-deriving the full Phase 2 inventory a 3rd time would violate the ground
  rules on activity-scoped judgement).
- Session context at start: 182,437 tokens (~22%) via `checkpoint.sh status` (the
  per-session reader, not the shared `.budget-status` file — see R2-F2/T-3127 below).

## What this round is

Per the run record, R3S1 = the value_review prompt again, round 3 of 4, fed by R2S3's
procAsFit handback. Consistent with R1S1/R2S1's own precedent, ran this round as a delta:
reconfirm the yardstick/data-map (unchanged), then focus on **closing out R2S1's two open
carry-forwards** (R2-F1 invocation-audit stall, R2-F4 the anomalous no-tty process) and
checking whether R2-F2/R2-F5's fates had moved.

## Execution

Ran Phase 0-3 (GATHERER) and Phase 4-5 (JUDGE — same worker, confidence dropped one level
per the prompt's own rule) as a single pass:

1. **Re-confirmed yardstick/data-map** — unchanged since R1S1/R2S1, no new contradiction.
2. **Root-caused R2-F1 (invocation-audit sink stall) — the one thing both prior rounds
   explicitly deferred as "needs a code read, out of scope for Phase 3."** Re-verified the
   sink is STILL exactly 4 records, same 2 timestamps, now ~9.5h since first observed at
   R1S1 (3rd independent confirmation). Read `crates/termlink-mcp/src/server.rs` (the
   `call_tool` choke point — confirmed it correctly records every literal MCP tool call)
   and `crates/termlink-hub/src/invocation_audit.rs` (confirmed `SURFACE_CLI` is defined
   but has **zero production call sites** anywhere in the workspace — `grep` across all
   crates found only the MCP call site and the module's own test block). Cross-checked
   against every prior step's own handback commands: 100% CLI/`fw`, 0% literal
   `mcp__termlink__*` tool calls — which is exactly the surface this instrument cannot see.
   **Then found this is not a new gap: `.tasks/active/T-3032-*.md` (created 2026-09-21, 4
   days before this sequence started) already states the identical root cause verbatim**,
   is `captured`/`horizon: next`, BVP-scored, never worked. A sibling, T-3033, covers the
   same gap for the `kv.*`/`session.*` daemon surfaces. Filed as **R3-F1**: not a new
   finding, but a 3-round/~10-hour empirical confirmation strengthening an already-captured
   task, with a recommendation to promote it to `horizon: now` given it now demonstrably
   blocks this very review series from getting real usage signal on the surface this
   sequence actually exercises.
3. **Resolved R2-F4 (anomalous no-tty `mcp serve` process, pid 1895596) to KEEP.**
   Confirmed via `ps`/`/proc` that the pid is still alive (~33.65h uptime, up from R2S1's
   ~32h reading — consistent growth), its parent (pid 1895454) is a LIVE, non-orphaned
   Claude Code **Desktop app** session (`ccd_*`/`computer-use` tool-grant signature), and
   `/proc/1895596/status` shows `PPid: 1895454`, `State: S` — not reparented to init. A
   Desktop-launched session has no controlling tty by construction, fully explaining the
   `tty=?` without any leak. Filed as **R3-F2**, closing R2S1's own explicitly-named
   resolving test (persistent + growing + parent-archaeology).
4. **Checked R2-F2's fate** (the CLAUDE.md concurrency-warning ADD-SURFACE finding) —
   found it CLOSED cleanly between rounds: `grep -n "T-3127" CLAUDE.md` shows the paragraph
   landed (T-3127 filed, T-3128 as its audit-side sibling), matching R2-F2's proposed
   wording in substance. No action needed; noted as a process win (a value-review-sourced
   finding executed and verified without this round having to chase it).
5. **Re-verified R1-F1/F1b (the 4 unused Cargo deps)** — still present, byte-identical,
   unexecuted. This is now the **3rd consecutive round** reporting this unchanged, spanning
   ~10h, 2 audit-remediation cycles, and 3 procAsFit rounds, with zero human engagement on
   the [ASK] specifically (as distinct from the Tier-0 `fw inception decide` GOs on
   T-3075/T-3076, which the human DID act on mid-sequence via Watchtower — so attention is
   reaching this run, just not this gate yet). Filed as **R3-F3**, carried unchanged, with
   the 3-round silence itself surfaced as a plain process observation rather than a
   re-litigation of the DELETE (Phase 6 still requires per-item human approval regardless).
6. **Quick check on T-2486's pre-decision `revisit_at` anomaly** (a low-priority carried
   INVESTIGATE) — its `## Decisions` section is still template-only, no recorded rationale.
   Not chased further; single data point, unchanged priority.

Wrote the evidence file (`docs/reports/VALUE-REVIEW-repo-2026-09-25-R3-evidence.md`) and
the report (`docs/reports/VALUE-REVIEW-repo-2026-09-25-R3.md`), following R1S1/R2S1's own
structure so the three rounds read as a continuous series.

## Findings summary (see report for full table)

| ID | Class | One-line |
|---|---|---|
| R3-F1 | ADD (WIRE) — confirms existing T-3032, not a new task | Root-caused the invocation-audit stall: `SURFACE_CLI` defined, zero call sites; the CLI surface this whole sequence actually uses is structurally invisible to the sink. Already captured as T-3032/T-3033, unworked since 2026-09-21 |
| R3-F2 | KEEP (resolves R2-F4) | The anomalous no-tty `mcp serve` process is a legitimate Desktop-app MCP child, not an orphan — confirmed via live, non-orphaned parent process |
| R3-F3 | DELETE (carried, unchanged, now 3 rounds stale) | 4 unused Cargo deps — still pending R1S1's original [ASK]; flagging the 3-round silence as a process observation, not reopening the DELETE decision itself |
| (process win, not a finding ID) | — | R2-F2's CLAUDE.md concurrency-warning paragraph landed and verified between rounds — a value-review finding executed cleanly without this round chasing it |

No DELETE/REFACTOR item was executed this step (Phase 6 gate, per known_gates — same as
R1S1/R2S1). Nothing here requires a NEW Sovereign decision; R3-F1/F2 are agent-actionable
(F1 via the existing T-3032/T-3033 tasks, F2 by simply closing the question) and R3-F3's
existing [ASK] is not reopened or duplicated.

## Stop condition

Context at write time: ~240K tokens (~30% of window) via `checkpoint.sh status`. Slightly
above this prompt's own "200k tokens" gathering-budget parameter (spent extra on the F1
code-read, judged worthwhile since it closed a 2-round-old open question with a concrete
root cause rather than deferring it a 3rd time), well inside the session's ~300k mandate
ceiling. No early stop was needed; this round's evidence-gathering (2 targeted code reads,
2 process-table checks, ~6 grep/read passes over existing task files) was proportionate to
closing exactly the two items R2S1 left open.

## Gates / governance notes

- No Tier-0, G-020, or P-002 gate fired this step — read-only investigation (source reads,
  `ps`/`/proc` checks, task-file reads) plus two new report-file writes under the
  already-focused, already-started-work T-3093 umbrella task, consistent with how
  R1S1/R2S1 operated (value-review is a GATHERER/JUDGE role producing a report, not code
  changes).
- Confirmed via direct re-read at the moment of checking (not from memory) that no
  live/shared infrastructure was touched or mutated: every command this step ran (`grep`,
  `ps`, `cat /proc/.../status`, file reads) is read-only, matching the GATHERER role's
  constraint and this run's own carried fix #2.
- Did not create a duplicate task for R3-F1 — verified T-3032/T-3033 already exist and
  already state the identical root cause before writing anything, in keeping with the
  T-2800 collision-avoidance concern this run's CLAUDE.md carries (three-workers-
  independently-discover-the-same-defect is exactly the failure mode a quick existing-task
  search prevents).

## Suggested next step (for R3S2 / audit-remediation)

- **T-3032/T-3033 are strong candidates to promote from `horizon: next` to `horizon: now`**
  given this round's direct empirical confirmation that they are actively degrading this
  very sequence's ability to use the invocation-audit sink as evidence. If R3S2 has Q1/Q2
  capacity after its own fresh audit findings, working T-3032 (the CLI choke point, tier=2/
  effort=8 per its existing estimate) would be a genuine value-review-driven remediation,
  not just an `fw audit` finding.
- R3-F3 (the Cargo deps DELETE) remains the same low-risk, fully-evidenced item it has been
  for 3 rounds — still gated on human [ASK], not adoptable by an audit-remediation step
  without that approval (Phase 6's rule, not a technical blocker).

## Commits this step

`e11430230` — run record update + both report files + this handback (committed together;
this step's own final commit hash is not self-referencing by construction, recorded here
after the fact, matching R1S1/R2S1's `131c864a6`/`bd54592f8` convention).
