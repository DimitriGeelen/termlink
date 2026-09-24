# Value review — repo — 2026-09-25 — Round 1 (T-3093 R1S1)

Scope: whole repo (/opt/termlink). Round 1 of 4 in the [Review, Audit, procAsFit] × 4
sequence. Evidence: `docs/reports/VALUE-REVIEW-repo-2026-09-25-R1-evidence.md`.

**This is a delta round, not a from-scratch review** — see the evidence file's scoping
note. This is at least the eighth value-review pass over this repo; the prior seven
already produced a full inventory, a 45-finding consolidated register (C-01..C-45, filed
as arc-009), and a Phase 0/1 orientation four days ago. Re-deriving all of that would
violate the ground rules on activity-scoped judgement and not polluting what is measured.
This round verifies what changed, closes one long-standing data gap, and reports genuinely
new evidence only.

## 1. Yardstick (confirmed unchanged)

Purpose: cross-terminal / cross-machine agent session control + durable messaging
substrate (docs/CHARTER.md, human-blessed). Value drivers: D1-D4 protected + up to 5 free
(policy/value-drivers.yaml v3). No change since 2026-09-21; no new contradiction found.

## 2. Data availability map (confirmed, delta only)

See evidence file §2. Two rows moved from a worse state to a better one this round:
invocation-audit sink (shipped-not-live → live, short window) and Rust unused-dependency
tooling (absent → run, workspace-clean scan on record).

## 3. Role setup

GATHERER and JUDGE are the same worker (single non-interactive `claude -p` step). Per the
prompt's own rule, every confidence rating below is one level lower than the raw evidence
would otherwise support.

## 4. Baseline

Not re-run this round (see evidence file §5 — re-running `cargo test --workspace` or the
guard layer for a delta-scoped pass would add multi-minute cost for no new signal; T-3090
established the guard-layer timing baseline yesterday). `fw audit` output from today
(2026-09-25) already on disk at `.context/audits/2026-09-25.yaml`, PASS-heavy, consistent
with the daily cron; R1S2 (audit_remediation, next step) is the step mandated to actually
run and act on it.

## 5. Summary

| Axis | Count this round | Top item |
|---|---|---|
| DELETE | 1 (4 sub-items) | F1 — 4 unused Cargo dependencies across 2 crates |
| REFACTOR | 1 | F2b — G-093 partial-complete/commit-gate dead end (already filed, re-surfaced by fresh measurement) |
| ADD | 0 new | (arc-009 already carries the open ADD items from the 2026-09-19 series) |
| INVESTIGATE / Sovereign question | 2 | F2 queue-clearing UX; F3 usage-data window too short to judge |
| Process note (not an axis) | 2 | F4 arc-009 progress; T-3044 closeable-but-unclosed |

## 6. Findings table

| ID | Item | Location | Class | Non-use reading | Evidence | Counter-evidence | Confidence | Proposal | Size | Reversible? | Risk if wrong | Expected effect |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| R1-F1 | `bytes`, `ulid` deps | `crates/termlink-protocol/Cargo.toml` | DELETE | E (not needed — never used) | 0 grep matches for crate usage anywhere in the crate incl. tests; not a re-export; added 2026-03-08, never touched since | cargo-machete has known macro-only-use false positives, but manual grep found zero references of any kind, including in doc comments describing intent | MEDIUM (self-judged, evidence itself is HIGH — 2 independent checks: tool + manual grep) | Remove both lines from Cargo.toml, run `cargo check -p termlink-protocol` | small | yes — 2-line revert | `cargo build` fails if something depends on a transitive re-export not found by grep (checked, none found) | `Cargo.lock` shrinks by 2 packages' worth of transitive tree (bytes has no deps beyond itself typically; ulid may pull `getrandom`/`rand_core`) |
| R1-F1b | `serde_json`, `termlink-protocol` deps | `crates/termlink-test-utils/Cargo.toml` | DELETE | E (not needed — never used) | 0 grep matches in src/ or tests/; added at crate creation (T-072, March 2026) | none found | MEDIUM | Remove both lines, run `cargo check -p termlink-test-utils` and `cargo test -p termlink-test-utils` | small | yes | same as above, scoped to test-utils | same |
| R1-F2 | Human-AC review queue | `fw review-queue` / Watchtower `/approvals` | Sovereign question, not a class verdict | not resolvable from static evidence (human intent unknown) | 144 pending, 90 GO up to 147d old; only 1 system-wide item is agent-closeable (list-closeable.sh) | G-093 (existing gap) explains part of the friction for items that reach partial-complete, not items untouched since filing | LOW (intent-dependent) | Ask the human: is this backlog's pace acceptable, or should a remediation task build a batch-review aid (e.g. a Watchtower bulk-approve view scoped to GO-verdict-only, still one-click-per-item to preserve T-372/T-373's no-batch-close rule)? | n/a (question) | n/a | acting without asking risks violating the Human Task Completion Rule's explicit "no batch-close" prohibition | clarifies whether R1S2/future rounds should spend budget here |
| R1-F2b | Partial-complete commit-gate dead end | `.agentic-framework/agents/context/check-active-task.sh:506-523`; `.agentic-framework/agents/task-create/update-task.sh:2100-2110` | REFACTOR (already filed as **G-093**, `status: watching`, 2026-09-18) | n/a | re-surfaced by this round's F2 measurement as a live contributor to queue aging, not a new finding | vendored code (G-062) — cannot be patched locally, only filed upstream (already is) | HIGH (pre-existing, independently re-confirmed relevant) | No new action — G-093 already tracks this; flagging the cross-reference so R1S2's audit-remediation pass sees the connection to the queue-size finding above | n/a | n/a | none — informational cross-link | none new |
| R1-F3 | Invocation-audit sink (D-1, prior report) | `/var/lib/termlink/invocation-audit.jsonl` | data-gap-closing, not yet actionable | B confirmed (was never wired due to stale binary, now resolved) | 4 records this hour, current binary | window (<1h) too short for any usage verdict | HIGH (direct measurement) | No action; let it accumulate; R2S1 should re-check size/age | n/a | n/a | none | by R2S1 this should hold enough data to start resolving C-29/C-30/C-31 |

## 7. KEEP list

Everything scoped by the 2026-09-19 consolidated review and not named above remains at its
prior classification; this round did not re-derive or overturn any of the 45 C-01..C-45
findings. Arc-009's 7 completed slices (T-2975, T-2976, T-2977, T-2979, T-2982, T-2986,
T-2989) are KEEP-and-closed.

## 8. INVESTIGATE list

- **R1-F2** (above) — needs a human answer on intent, not more static evidence.
- Whether the ~30 long-lived `termlink mcp serve` processes noted in the evidence file §4.2
  are legitimate open sessions or leaked processes — needs a liveness cross-check
  (`ps` pts vs active terminal sessions) not attempted this round.
- Whether the 9 DEFER inception decisions in the queue (ages 5-53 days) carry `revisit_at`
  and whether G-053's daily scan is actually surfacing them — plausible target for R2S1.

## 9. Data gaps that capped confidence

- **JS/Python dead-code tooling absent** (knip/vulture/jscpd) — the Rust half of this gap
  closed this round (cargo-machete); the Python `web/` Flask blueprints and shell scripts
  under `agents/`/`scripts/` remain unscanned. Closing it is itself an ADD candidate
  (instrumentation) for a future round.
- Per-tool MCP/CLI usage remains UNMEASURED pending F3's window growing (see above).
- Self-judged GATHERER=JUDGE — every confidence rating above is capped one level below
  what the raw evidence supports, per the prompt's own rule.

## 10. Contradictions

See evidence file §4 (three-way version-number mismatch; long-lived MCP-serve process
count noted but not classified).

## 11. Not reviewed

See evidence file §5 (full re-list omitted here to avoid duplication) — chiefly: the full
Phase 2 inventory, the 45 prior findings' content re-verification, JS/Python dead-code
scan, `revisit_at` audit of the 9 deferred inceptions, and a fresh baseline run.

## 12. Sovereign questions

1. **R1-F2** — is the 144-item, up-to-147-day-old Human-AC review queue an acceptable
   backlog pace, or does it warrant a batch-review UX investment? This is the human's call
   under the Authority Model; agents may propose, never decide, and CLAUDE.md explicitly
   forbids agent batch-closing without per-item evidence regardless of the answer.
2. Carried from prior rounds, unchanged: arc-011's two open inceptions (T-3075, T-3076)
   remain `fw inception decide`-blocked (human-only, Tier-0-adjacent). Not re-litigated
   here — see `.context/runs/T-3089-*-handback.md` for full history; the operator rulings
   at the top of this run's dispatch prompt already cover T-3075/T-3076's IW-4 (portable
   fallback required) and are not reopened by this review.

---

**[ASK] — halted here per the known gate.** Per this run's `known_gates` entry ("The
value-review prompt HALTS at PHASE 5 [ASK] for per-item human approval before PHASE 6
EXECUTE. A review step that halts there is COMPLETE, not failed — the T-3044 precedent."),
this step does not proceed to Phase 6 execution. The two DELETE candidates (R1-F1, R1-F1b)
are small, reversible, and evidence-complete against all 7 DELETE CHECKS; they are ready
for a one-line human approval or for R1S2 (audit_remediation) to pick up as its own
governed task if the human/operator treats a value-review-sourced, fully-evidenced DELETE
finding as in scope for that step's "convert every finding into a governed task" mandate.
No item in this report is executed by this step.
