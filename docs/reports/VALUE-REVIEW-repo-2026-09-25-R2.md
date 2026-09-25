# Value review — repo — 2026-09-25 — Round 2 (T-3093 R2S1)

Scope: whole repo (/opt/termlink). Round 2 of 4 in the [Review, Audit, procAsFit] × 4
sequence. Fed by R1S3's handback (procAsFit round 1: closed T-3010, T-2998 under arc-009).
Evidence: `docs/reports/VALUE-REVIEW-repo-2026-09-25-R2-evidence.md`.

**This is a delta round, not a from-scratch review** — the ninth value-review pass over
this repo, the second in this orchestrated sequence. It verifies what changed in the ~7
hours since R1S1, closes out R1S1's three explicitly carried-forward INVESTIGATE items, and
reports genuinely new evidence only. See the evidence file's scoping note for the full
rationale.

## 1. Yardstick (confirmed unchanged)

Purpose: cross-terminal / cross-machine agent session control + durable messaging
substrate (docs/CHARTER.md, human-blessed). Value drivers: D1-D4 protected + up to 5 free
(policy/value-drivers.yaml v3). No change since R1S1; no new contradiction found.

## 2. Data availability map (confirmed, delta only)

See evidence file §2. Two rows moved from a better state to a **worse** one this round —
worth flagging plainly rather than only noting improvements: the invocation-audit sink
(live-and-growing at R1S1 → live-but-stalled now) and the G-087-class budget-cache defect
(believed closed three times → recurred in a new concurrency framing this session).

## 3. Role setup

GATHERER and JUDGE are the same worker (single non-interactive `claude -p` step, same
constraint as R1S1). Every confidence rating below is one level lower than the raw evidence
would otherwise support, per the prompt's own rule.

## 4. Baseline

Not re-run this round — no code changed since R1S1 in a way that would invalidate its
decision to skip a fresh `cargo test --workspace`/guard-layer run, and R1S2's own audit
re-runs this session already exercised the relevant surfaces (400/85-89/3-4 pass/warn/fail,
per its handback). Re-running for a delta-scoped pass would cost 10-50+ minutes of
contended wall-clock (T-3090 baseline) for no new signal.

## 5. Summary

| Axis | Count this round | Top item |
|---|---|---|
| DELETE | 0 new (1 carried, unexecuted) | R1-F1/F1b (4 unused Cargo deps) still pending human [ASK] |
| REFACTOR | 0 new | — |
| ADD | 2 | F1 — invocation-audit sink stalled again (REPAIR/WIRE, undetermined); F2 — G-087-class concurrency hazard undocumented in CLAUDE.md (SURFACE) |
| INVESTIGATE | 2 closed-out, 1 opened | Closed: F3 (30/31 mcp-serve processes = legitimate), F4 (revisit_at mechanism confirmed working). Opened: F1's root cause (A vs B undetermined) |
| Process note (not an axis) | 2 | T-3006 NO-GO-recommended inception sitting undecided; T-2486 revisit_at set pre-decision (anomaly, unresolved) |

## 6. Findings table

| ID | Item | Location | Class | Non-use reading | Evidence | Counter-evidence | Confidence | Proposal | Size | Reversible? | Risk if wrong | Expected effect |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| R2-F1 | Invocation-audit sink (`invocation-audit.jsonl`) | `/var/lib/termlink/invocation-audit.jsonl`, writer in `crates/termlink-mcp/src/` (not read this round) | ADD (REPAIR or WIRE — undetermined) | A or B, not distinguished (see evidence §3 F1) | 4 records at R1S1, still exactly 4 records ~7h later despite 25+ MCP calls in this very session; only 4 introspection tool names ever recorded | none against — the file is real, was growing once, stopped growing; not simply "too short a window" any more (R1S1's own prediction that growth would resume is now falsified) | MEDIUM (direct re-measurement, self-judged JUDGE=GATHERER) | R2S2 (audit-remediation) or a future round: read the writer's call sites in `termlink-mcp` to determine whether it instruments a fixed allowlist of tool names (WIRE — extend the allowlist) or has a per-process-lifetime bug (REPAIR) | small (once root cause known) | yes | none from investigating; a wrong REPAIR could mask the real bug if applied without reading the call sites first | invocation-audit.jsonl should start growing per-call again; unlocks C-29/C-30/C-31 usage verdicts this was meant to feed |
| R2-F2 | G-087-class stale/shared `.budget-status` cache, concurrency framing | CLAUDE.md § Context Budget Management (~line 2932); `.context/working/.budget-status` | ADD (SURFACE) | C (undiscoverable in this framing) — the single-session framing (T-2950/T-3018/T-3034) is documented and fixed 3×; the concurrent-orchestrated-worker framing is not | R1S2 and R1S3 independently read `.budget-status` as ~504K when their real per-session usage (`checkpoint.sh status`) was ~169K, discovered this run's own dispatch model creates the hazard, and both worked around it the same way without either one finding a warning already in CLAUDE.md | this round's own single-worker read of `.budget-status` was plausible (189468, consistent with a concurrent `checkpoint.sh status` read) — confirming the defect is concurrency-specific, not a permanent break, which somewhat narrows (not negates) urgency | MEDIUM (2 independent worker reports this session + 1 direct re-check; self-judged JUDGE=GATHERER) | Add one paragraph to CLAUDE.md's Context Budget Management section: "when multiple orchestrated/dispatched `claude -p` workers run concurrently, `.context/working/.budget-status` reflects whichever worker wrote it last and MUST NOT be trusted by a different worker — always use `checkpoint.sh status`, which reads your own transcript" | small (doc-only, 1 paragraph) | yes — trivial revert | none — it's additive documentation | the next orchestrated run (R3S1..R4S3, or any future one) starts with the warning already in place instead of rediscovering it a 4th/5th time |
| R2-F3 | 30 of 31 long-lived `termlink mcp serve` processes | process table, all `pts/N`-attached | KEEP (resolves R1S1's open question) | not applicable — use confirmed | cross-referenced against `who` (426 sessions) and `/dev/pts` (848 allocated) on a host T-3090 already established runs at high, sustained concurrency | none | HIGH (direct measurement, 2 independent signals: pts attachment + host-activity baseline) | no action — close the R1S1 INVESTIGATE item as resolved: legitimate | n/a | n/a | none | none — informational closure |
| R2-F4 | 1 anomalous `termlink mcp serve` process, no controlling tty, ~32h uptime (pid 1895596 at check time) | process table | INVESTIGATE (not DELETE — single instance, no positive reason found, could be a live legitimate headless worker) | D (unmeasured — cannot tell orphan from legitimate headless dispatch without parent-pid history that may no longer resolve) | `tty` column reads `?`; all 30 siblings have named ptys; a controlling-tty-less long-lived process cannot be an open interactive terminal by definition, but CAN be a legitimate detached/headless dispatch (this very run is evidence such things exist and are normal) | none decisive either way | LOW (single data point, inconclusive by design) | If seen again next round at a similar or growing uptime with still no tty, treat as a stronger orphan signal; a single instance does not meet any DELETE check (no positive reason, no repeated pattern yet) | n/a | n/a | killing a live legitimate process would be a real outage; doing nothing costs nothing if it is genuinely a leak (single stray process, not unbounded growth) | R3S1 should re-check whether pid 1895596 (or another `?`-tty process) still exists and how its uptime has grown |
| R2-F5 (carried, unchanged) | 4 unused Cargo deps (`bytes`, `ulid` in termlink-protocol; `serde_json`, `termlink-protocol` in termlink-test-utils) | `crates/termlink-protocol/Cargo.toml`, `crates/termlink-test-utils/Cargo.toml` | DELETE (unchanged from R1-F1/F1b) | E (not needed) | re-verified present this round — still unremoved; all 7 DELETE CHECKS from R1S1 still pass, nothing regressed them | none new | MEDIUM (unchanged from R1S1) | Same proposal as R1S1: remove the 4 lines, `cargo check` the two crates. Still awaiting human [ASK] approval or an audit-remediation step adopting it as a governed task | small | yes | none beyond R1S1's own analysis | Cargo.lock shrinks; no behavior change |

## 7. KEEP list

Everything scoped by R1S1 and the prior 45 C-01..C-45 findings, not named above, remains at
its prior classification. This round adds **R2-F3** (30 legitimate mcp-serve processes) to
the KEEP list as a newly-resolved item (was an open question, not previously classified).
arc-009's now-9/19-ish completed slices (T-2975, T-2976, T-2977, T-2979, T-2982, T-2986,
T-2989, plus R1S3's T-2998 and the intentionally-still-open T-3010) are KEEP-and-progressing.

## 8. INVESTIGATE list

- **R2-F1** (above) — needs a code read of `crates/termlink-mcp/src/` invocation-audit call
  sites to distinguish REPAIR from WIRE. Cheapest next step: `grep -rn
  "invocation.audit\|invocation_audit" crates/termlink-mcp/src/` in a future round or in
  R2S2's remediation pass.
- **R2-F4** (above) — the single `?`-tty long-lived process. Needs a repeat check next
  round (growing/persistent uptime with still no tty would strengthen an orphan reading;
  disappearing would resolve it as a transient headless worker).
- **T-2486's pre-decision `revisit_at`** (evidence §3 F4) — a single targeted read of its
  `## Decisions` section would resolve whether this is a legitimate speculative-annotation
  pattern worth documenting, or a one-off anomaly. Not attempted this round (budget
  discipline — single data point, low urgency).

## 9. Data gaps that capped confidence

- **R2-F1's root cause** — could not be determined from Phase-3 evidence-gathering alone;
  needs a code read that this GATHERER pass deliberately did not do (out of scope for
  Phase 3, would blur into Phase 6 execution work).
- Self-judged GATHERER=JUDGE — every confidence rating above is capped one level below what
  the raw evidence supports, per the prompt's own rule, same as R1S1.
- JS/Python dead-code tooling (knip/vulture/jscpd) remains ABSENT, unchanged since R1S1 —
  not re-flagged as new evidence this round, still an open data gap for a future round.

## 10. Contradictions

See evidence file §4: version-skew triplet advanced consistently (not new drift, same
pre-existing gap); T-3006 carries a NO-GO auto-recommendation while sitting undecided on
the review queue (a routing note for R2S2, not a value-review verdict).

## 11. Not reviewed

See evidence file §5 — chiefly: the full Phase 2 inventory (unchanged since 2026-09-19),
code-level root cause for R2-F1, `/proc` archaeology for R2-F4's anomalous process, T-2486's
`## Decisions` section, JS/Python dead-code tooling, and a fresh baseline run.

## 12. Sovereign questions

1. **Carried from R1S1, unchanged, not re-litigated:** the 144-item Human-AC review queue's
   acceptable-backlog-pace question (R1S1's F2) — still 144, still 147d oldest, no new
   evidence either way this round. Still the human's call under the Authority Model.
2. **Carried from prior rounds, unchanged:** arc-011's operator-decision-blocked build
   slices (S1 portable-respawn design, S12 systemd allowlist) — not touched or re-litigated
   this round; the operator rulings already carried into this run's dispatch prompt stand.
3. **New this round, low-stakes:** none of R2-F1..F5 requires a Sovereign decision — F1/F2
   are ADD-SURFACE/investigate items an agent can act on directly once root-caused, F3/F4
   are informational, and F5 (the DELETE candidate) already has its own explicit [ASK] gate
   from R1S1 that this round does not reopen or duplicate.

---

**[ASK] — halted here per the known gate.** Per this run's `known_gates` entry, this step
does not proceed to Phase 6 execution. Nothing in R2-F1..F4 requires Sovereign approval to
investigate further (they are ADD/INVESTIGATE, not DELETE/REFACTOR touching protected
surfaces); R2-F5 (the carried DELETE candidate) remains gated exactly as R1S1 left it —
this round adds no new urgency or evidence to it, just confirms it hasn't regressed. R2S2
(audit-remediation, next step) may adopt any of R2-F1/F2/F5 as governed tasks if it judges
them in scope for its own "convert findings into tasks" mandate, per the same precedent
R1S1 set for R1S2. No item in this report is executed by this step.
