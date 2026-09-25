# Value review — repo — 2026-09-25 — Round 3 (T-3093 R3S1)

Scope: whole repo (/opt/termlink). Round 3 of 4 in the [Review, Audit, procAsFit] × 4
sequence. Fed by R2S3's handback (procAsFit round 2). Evidence:
`docs/reports/VALUE-REVIEW-repo-2026-09-25-R3-evidence.md`.

**This is a delta round, not a from-scratch review** — the tenth value-review pass over
this repo, the third in this orchestrated sequence. It closes out R2S1's two open
INVESTIGATE items (one with a full root cause, one resolved to KEEP) and reports that the
one live DELETE candidate has now gone three rounds with zero human engagement.

## 1. Yardstick (confirmed unchanged)

Purpose: cross-terminal / cross-machine agent session control + durable messaging
substrate (docs/CHARTER.md, human-blessed). Value drivers: D1-D4 protected + up to 5 free
(policy/value-drivers.yaml v3). No change since R1S1/R2S1; no new contradiction found.

## 2. Data availability map (confirmed) + snapshot windows

See evidence file §2. Session snapshot: 182,437 tokens (~22%) at start via
`checkpoint.sh status`. One row moved from ABSENT-explanation to fully root-caused
(invocation-audit CLI-surface gap); one row moved from INVESTIGATE to resolved-KEEP
(the no-tty process); one row moved from open-ADD to CLOSED (the `.budget-status`
CLAUDE.md paragraph, delivered by T-3127/T-3128 between R2S1 and this round).

## 3. Role setup

GATHERER and JUDGE are the same worker (single non-interactive `claude -p` step, same
constraint as R1S1/R2S1). Every confidence rating below is one level lower than the raw
evidence would otherwise support, per the prompt's own rule.

## 4. Baseline

Not re-run this round — no code changed since R1S1 in a way that would invalidate the
decision both prior rounds made to skip a fresh `cargo test --workspace`/guard-layer run
for a delta-scoped pass (T-3090's ~945s contended-host baseline still stands as the
authoritative timing figure; re-running it a third time for no new signal would itself be
the kind of measurement-polluting cost the ground rules warn against).

## 5. Summary

| Axis | Count this round | Top item |
|---|---|---|
| DELETE | 0 new (1 carried, unexecuted, now 3 rounds stale) | R1-F1/F1b — still pending human [ASK] |
| REFACTOR | 0 new | — |
| ADD | 0 new findings, 1 EXISTING task strengthened with direct evidence | R3-F1 confirms T-3032 (captured since 2026-09-21, unworked) with a live 3-round, ~10-hour empirical demonstration of the exact failure it describes |
| INVESTIGATE | 1 closed (KEEP) | R3-F2 resolves R2-F4 — legitimate Desktop-app MCP child process, not an orphan |
| Process note (not an axis) | 1 | R2-F2's CLAUDE.md doc fix landed cleanly between rounds — a value-review-sourced ADD-SURFACE finding executed and verified without this review round having to chase it |

## 6. Findings table

| ID | Item | Location | Class | Non-use reading | Evidence | Counter-evidence | Confidence | Proposal | Size | Reversible? | Risk if wrong | Expected effect |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| R3-F1 | Invocation-audit sink undercounts by construction — CLI surface unwired | `crates/termlink-hub/src/invocation_audit.rs:68` (`SURFACE_CLI` defined, never called); `crates/termlink-mcp/src/server.rs:44` (the only production call site, MCP-only). Already tracked at `.tasks/active/T-3032-*.md` | ADD (WIRE) — **not a new finding, an existing captured task (T-3032) confirmed and strengthened** | B, NEVER WIRED, with the module's own doc comment as direct intent evidence ("a future CLI/session-daemon instrument can share the sink") | 3 independent measurements (R1S1, R2S1, R3S1) over ~9.5h, spanning 2 full audit-remediation cycles and 2 procAsFit rounds of heavy CLI/`fw` activity, show the sink frozen at the same 4 records; `grep` confirms zero production `SURFACE_CLI` call sites; every prior step's own handback commands are 100% CLI/`fw`, 0% literal MCP tool calls | none — the absence-of-growth reading was already falsified by R2S1's own re-measurement; this round supplies the mechanism, not just another null result | MEDIUM (2 independent verified facts — the grep and the 3-round measurement — but self-judged JUDGE=GATHERER caps it) | No new task needed — T-3032 already exists, scored (D1=4/D2=2/D3=3/D4=2, tier=2/effort=8), `horizon: next`. Recommend a future procAsFit round promote it to `horizon: now`: it is now the load-bearing blocker for every future review round's ability to use this sink as DELETE evidence for the 260-tool MCP surface, and its own doc comment names UNDERCOUNT as the dangerous failure direction | small-medium (per T-3032's own effort=8 estimate; a CLI dispatch choke point mirroring the MCP one) | yes | leaving it uncorrected risks a future round treating sink silence as a real usage signal and recommending a DELETE that undercounts CLI-only tools | invocation-audit.jsonl should begin recording CLI-surface calls; unblocks the C-29/C-30/C-31 usage verdicts this instrument exists to feed, this time on the surface that actually carries this sequence's own traffic |
| R3-F2 | `termlink mcp serve` pid 1895596, no controlling tty | process table | KEEP (resolves R2-F4) | not applicable — identity question resolved, not a usage question | parent pid 1895454 is a live (non-orphaned, `State: S`) Claude Code **Desktop app** session (`ccd_*`/`computer-use` tool grants); Desktop-launched sessions have no controlling pty by construction | none | HIGH (direct process-table + `/proc` evidence, 2 independent signals: parent liveness + tool-grant signature) | No action — close the R2S1/R1S1 open question as resolved: legitimate, not a leak | n/a | n/a | none | none — informational closure, matches R2-F3's precedent for the other 30 processes |
| R3-F3 (carried, now stale 3 rounds) | 4 unused Cargo deps (`bytes`, `ulid` in termlink-protocol; `serde_json`, `termlink-protocol` in termlink-test-utils) | `crates/termlink-protocol/Cargo.toml`, `crates/termlink-test-utils/Cargo.toml` | DELETE (unchanged from R1-F1/F1b/R2-F5) | E (not needed) | re-verified present, byte-identical to R1S1's original citation; all 7 DELETE CHECKS still pass; **new this round: the silence itself** — 3 review rounds, 2 audit-remediation cycles, 3 procAsFit rounds, ~10h, and neither the human [ASK] nor either audit-remediation step's own "adopt fully-evidenced findings as governed tasks" precedent (which R1S2 exercised for `fw audit`/`fw doctor` findings, just never this one) has touched it | none new | MEDIUM (unchanged from R1S1) | Unchanged proposal: remove the 4 lines, `cargo check` the two crates. Flagging the 3-round silence as a process observation for the human, not a re-litigation of the DELETE itself — Phase 6 still requires per-item human approval regardless of how many rounds pass | small | yes | none beyond R1S1's own analysis | Cargo.lock shrinks; no behavior change; closing this would also demonstrate the [ASK] channel is being read at all, which nothing so far confirms either way |

## 7. KEEP list

Everything scoped by R1S1/R2S1 and the 2026-09-19 consolidated review (C-01..C-45), not
named above, remains at its prior classification. This round adds **R3-F2** (the no-tty
Desktop-app MCP process) to the KEEP list, completing the classification of all 31
long-lived `mcp serve` processes first noted in R1S1 (30 via R2-F3, the 31st via this
round). arc-009's progress since R2S1: 26 of ~49 tagged tasks now `work-completed` (was
9-ish at R2S1); 3 `started-work`; 20 `captured` including T-3032/T-3033, both directly
relevant to this round's F1.

## 8. INVESTIGATE list

- None carried open from this round's own findings — R3-F1 resolved to a confirmed
  existing task (not a new open question), R3-F2 resolved to KEEP.
- Carried, unchanged, low-priority: T-2486's pre-decision `revisit_at` (its `## Decisions`
  section remains template-only — checked this round, not chased further, single data
  point).
- Carried, unchanged: JS/Python dead-code tooling (knip/vulture/jscpd) remains an
  unmeasured gap, not re-flagged as new evidence this round.

## 9. Data gaps that capped confidence

- Self-judged GATHERER=JUDGE — every confidence rating above is capped one level below
  what the raw evidence supports, per the prompt's own rule, same as R1S1/R2S1.
- The `fw review-queue` 144-item backlog figure is carried from R1S1 without a fresh
  re-run this round (budget discipline — an unchanged number three checks running would
  add cost for no new signal); flagged here so it is not mistaken for a re-verified fact
  the way R3-F3's Cargo-deps citation is.

## 10. Contradictions

None new this round (see evidence file §4).

## 11. Not reviewed

See evidence file §5 — chiefly: the full Phase 2 inventory (stable since 2026-09-19, third
consecutive delta-confirmation), JS/Python dead-code scan, a fresh baseline run, and a
`fw review-queue` re-run.

## 12. Sovereign questions

1. **Carried from R1S1/R2S1, now sharpened, not re-litigated as a new decision:** R1-F1's
   original question (is the 144-item, up-to-147-day-old Human-AC review queue an
   acceptable backlog pace?) is unchanged. **New observation this round, worth surfacing
   plainly:** the review-queue question and R3-F3's DELETE [ASK] may share one root cause —
   nothing in this entire 9-step, ~10-hour orchestrated sequence has observed any human
   engagement with a value-review-sourced [ASK] specifically (as distinct from the
   Tier-0 `fw inception decide` GOs on T-3075/T-3076, which the human DID act on via
   Watchtower mid-sequence — so human attention is demonstrably reaching this run, just not
   yet this particular gate). This is a process observation for the human, not a claim that
   anything is broken — Phase 6's per-item approval requirement is working exactly as
   designed; it simply has not been exercised yet for this run's DELETE candidate.
2. **Carried from prior rounds, unchanged:** arc-011's two inceptions (T-3075, T-3076) are
   now decided (GO) and R2S3 already re-pointed their slices to new build tasks
   (T-3134/T-3135) — this Sovereign item is resolved as far as the value-review axis is
   concerned; the remaining open question (S12's framework-agent-systemd allowed-commands
   blocker) is a separate project's config per T-559 and out of this review's scope.
3. R3-F1 and R3-F2 require no Sovereign decision — F1 confirms an already-scoped,
   already-`agent`-owned build task; F2 is informational.

---

**[ASK] — halted here per the known gate.** Per this run's `known_gates` entry ("The
value-review prompt HALTS at PHASE 5 [ASK] for per-item human approval before PHASE 6
EXECUTE. A review step that halts there is COMPLETE, not failed"), this step does not
proceed to Phase 6 execution. R3-F1 and R3-F2 require no Sovereign approval to act on
further (F1 is an existing agent-owned task a future procAsFit round may pick up directly;
F2 is closed, informational). R3-F3 (the carried DELETE candidate) remains gated exactly as
R1S1 left it three rounds ago — this round adds no new urgency to the DELETE itself, only
notes that the [ASK] has now gone unanswered through the whole sequence so far. No item in
this report is executed by this step.
