# Value review — repo — 2026-09-25 — Round 1 evidence file

Round 1 of 4 in the [Review, Audit, procAsFit] × 4 orchestrated sequence (T-3093,
run record `.context/runs/T-3093-review-audit-procasfit-x4.yaml`, step `R1S1`).

**Role note:** GATHERER and JUDGE are the same worker in this run (a single `claude -p`
step). Per the prompt's own rule this drops confidence one level from what is stated below
— already applied in the report file's confidence column.

**Scoping decision (stated up front, not hidden):** This is at least the **eighth**
value-review pass over this repo. Prior runs (2026-09-16, five runs on 2026-09-19
consolidated into 45 findings C-01..C-45, and a partial Phase-0/1 orientation on
2026-09-21) already built the inventory, the data-availability map, and the yardstick
confirmation. Re-deriving all of that from zero this round would violate the ground rule
"Activity, not calendar" (no new activity in most of that surface since the last look) and
"Don't pollute what you measure" (re-running full static analysis repeatedly for the same
answer wastes the budget this exercise is supposed to conserve). This round therefore runs
as a **delta + gap-closing pass**: verify what changed since 2026-09-21, close one
previously-flagged data gap with new tooling, and report fresh findings only where the
evidence is genuinely new. Full phases 2/3 (whole-repo re-inventory) are explicitly **not**
re-run; see "Not reviewed" below.

## 1. Yardstick (re-confirmed, unchanged from 2026-09-21)

- Purpose (docs/CHARTER.md, human-blessed sentence, unchanged): a cross-terminal /
  cross-machine agent session-control and durable-messaging substrate — "...sessions across
  one or many machines."
- `policy/value-drivers.yaml` v3: 4 protected directives (D1-D4, the Constitutional
  Directives) + up to 5 free drivers, cap 9 total. Unchanged since prior review.
- No contradiction found between stated purpose and the two new pieces of evidence
  gathered this round (see §3).

## 2. Data availability map — delta since 2026-09-21

Only rows that changed status or gained new evidence are listed; unlisted rows are
unchanged from `docs/reports/VALUE-REVIEW-repo-2026-09-21-phase01.md` §4.

| Source | 2026-09-21 status | 2026-09-25 status | Evidence |
|---|---|---|---|
| Invocation audit (`invocation-audit.jsonl`, T-2996) | shipped, not live (stale binary, sink absent in all 3 runtime dirs) | **EXISTS, live, short window** | `/var/lib/termlink/invocation-audit.jsonl`, 4 records, all `ts` within the last hour of this check, `surface:"mcp"`, names `termlink_hub_status`/`termlink_doctor`/`termlink_dispatch_status`/`termlink_topics`. Hub pid 3071124 exe resolves to `/root/.cargo/bin/termlink` (current, reinstalled by T-2977 on 2026-09-24). |
| Dead-code / unused-dependency tooling (Rust half) | **ABSENT** (cargo-udeps/cargo-machete not installed — named as a gap in 3 prior reports) | **EXISTS this round** — `cargo machete v0.9.2` installed locally per ground-rule 6 ("local installs for analysis are fine... never add to project dependencies"), not added to any manifest | `timeout 120 cargo machete` run at repo root, full workspace scan, exit clean, findings below. |
| Dead-code tooling (JS/Python half: knip/vulture/jscpd) | ABSENT | still ABSENT | not run this round — out of scope budget; flagged as remaining gap in §6. |
| arc-009 execution register | 3/18 slices complete | **7/18 complete** | task-file `status:` field read directly for all 18 T-2975..T-2992, see §4. |
| Review-queue (Human-AC pending) | not measured in prior reports | **measured this round: 144 tasks** | `fw review-queue` full output, see §4. |

## 3. New findings this round (verified, with citation)

### F1 — Four unused Cargo dependencies, verified by direct grep (not just the tool's say-so)

Command: `cargo machete` (workspace root), output:

```
termlink-protocol -- ./crates/termlink-protocol/Cargo.toml:
	bytes
	ulid
termlink-test-utils -- ./crates/termlink-test-utils/Cargo.toml:
	serde_json
	termlink-protocol
```

Independent verification (cargo-machete has known false-positive classes — macro-only use,
re-exports — so each was checked by hand rather than trusted on the tool's word alone):

- `grep -rn '\bbytes::\|use bytes' crates/termlink-protocol/src/` → **0 matches**. Every
  occurrence of the string "bytes" in that crate (`to_be_bytes()`, doc comments, the word
  "bytes") is the std-lib method or English word, not the `bytes` crate — confirmed by
  reading the matched lines (`crates/termlink-protocol/src/data.rs`,
  `crates/termlink-protocol/src/control.rs`, `crates/termlink-protocol/src/error.rs`).
- `grep -rn 'ulid' crates/termlink-protocol/ --include='*.rs'` → **0 matches anywhere in
  the crate**, tests included.
- `grep -rln 'serde_json\|termlink_protocol' crates/termlink-test-utils/ --include='*.rs'`
  → **0 matches** in any file in the crate (src and tests).
- `grep -rn 'pub use bytes\|pub use ulid\|bytes::Bytes\|ulid::Ulid' crates/` (whole
  workspace) → **0 matches** — ruling out a re-export that would make these part of another
  crate's public API surface via `termlink_protocol::bytes::...`.
- Age: `git log --oneline -- crates/termlink-test-utils/Cargo.toml` bottoms out at `T-014`
  (workspace scaffold) and `T-072` (crate creation) — both from the March 2026 window.
  `crates/termlink-protocol/Cargo.toml`'s last-touch date for these lines is 2026-03-08.
  Not a recent addition (rules out the "recently added, give it time" DELETE-against
  signal).

Kind: structure / cost. Axis: DELETE. This is genuinely new evidence — three prior reports
(2026-09-19 run2, run5; 2026-09-21 phase01 §4) named the ABSENT unused-dependency-tooling
row as a whole invisible DELETE axis ("30 workspace deps / 310 lockfile packages"); this is
the first round with an actual read on it.

### F2 — Human-AC review queue: 144 tasks pending, oldest GO-verdict items 147 days old

Command: `.agentic-framework/bin/fw review-queue`. Output (full, not sampled):
**12 pending inception decisions** (5 GO, ~7 DEFER-or-unclear at various ages up to 55d) +
**144 tasks awaiting human review** on their Human ACs, broken down: **90 GO / 9 DEFER /
2 NO-GO / 26 `?` (undetermined) / 17 NO-REC** (recommendation withdrawn/superseded).
Oldest sampled GO-verdict ages seen in the listing: 147, 146, 145, 143(×9) days.

Cross-reference: `bash scripts/list-closeable.sh` (the AGENT-side companion queue) finds
only **1** agent-closeable task system-wide right now (`T-3044`, itself the predecessor of
this very orchestrated sequence — 4/4 Agent ACs checked, 0 Human ACs blocking, not yet
closed). So the 144-item backlog is overwhelmingly **not** an agent-actionable gap; the
`fw task verify` mechanism has already done its job (a GO verdict means the tool found
evidence the Human ACs are satisfied) and the remaining step is a human clicking through
`fw task update T-XXX --status work-completed` (or the Watchtower `/approvals` page).

Non-use diagnosis, applied to the *review-queue mechanism itself* rather than to any one
task in it (the prompt's NON-USE framework is written per-item; the queue as a whole is
better read this way):
- **Not C (undiscoverable):** the command exists, runs, and prints a direct Watchtower URL
  (`http://192.168.10.107:3003/approvals`). Findable.
- **Not A (broken):** it produces a correct-looking, well-formed listing.
- **Partial B-adjacent (a related, already-documented structural friction):**
  `.context/project/concerns.yaml` **G-093** (filed 2026-09-18, `status: watching`)
  documents a real dead end in the SAME mechanism: closing a task to `partial-complete`
  pins focus on it, and the write-gate then refuses the very commit that would persist the
  Human-AC tick, under that task's own focus — the only exits being a logged override, a
  commit misattributed to a different task, or leaving the edit uncommitted. This is a
  plausible *contributing* mechanism to why GO-verdict items age past 140 days once a human
  starts the close-out and hits the gate, but it does not explain items that were never
  started at all, so it only partially accounts for the backlog.
- **Not E (not wanted):** the Human-AC split (T-193/T-372/T-373) is core governance, and
  CLAUDE.md is explicit that batch-closing without per-item evidence is prohibited — so a
  large, aging queue is the EXPECTED shape of a working, sovereignty-respecting gate, not
  evidence the gate should be removed.
- Reading not fully resolvable from static evidence: whether the human considers a
  147-day-old GO item "fine to leave, I'll get to it" (in which case this is not a defect)
  or "I didn't know these were sitting there" (in which case a digest/batch-review surface
  would help). This is recorded as a **Sovereign question**, not a verdict, in the report.

Kind: friction / process. Axis: touches REFACTOR (G-093's gate interaction) and a
Sovereign question (queue-clearing UX), not DELETE or ADD in the strict sense — the
mechanism works as designed; whether its throughput needs help is the human's call.

### F3 — D-1 (invocation-audit sink) evidence updated, verdict still not ready

The 2026-09-21 report's finding D-1 (shipped-but-dark instrumentation, distinguishing
NON-USE reading A/BROKEN from B/NEVER-WIRED) predicted that a binary reinstall would begin
producing data. `/var/lib/termlink/invocation-audit.jsonl` now exists with 4 records, all
from the current hour, confirming **reading B was correct, not A** — the writer was never
broken, it was serving from a stale binary. **Per-tool usage remains UNMEASURED** — a
4-event, <1-hour window is not enough to support any usage verdict on any of the 214+ MCP
tools it will eventually cover. Recommendation carried to R2S1 (round 2's review step):
re-check this file's growth; by round 2 it should hold enough history to start being
usable evidence for the C-29/C-30/C-31 "structurally invisible usage" findings from the
2026-09-19 consolidated report.

### F4 — arc-009 execution progress (measured, not estimated)

Direct read of `status:` frontmatter for all 18 tasks named in `.context/arcs/arc-009.yaml`
scope (T-2975..T-2992):

| Status | Count | Tasks |
|---|---|---|
| work-completed | 7 | T-2975, T-2976, T-2977, T-2979, T-2982, T-2986, T-2989 |
| started-work | 1 | T-2984 (owner: human, inception) |
| captured | 10 | T-2978, T-2980, T-2981, T-2983, T-2985, T-2987, T-2988, T-2990, T-2991, T-2992 |

Up from 3/18 complete on 2026-09-21 (T-2975, T-2982, T-2989 at that time — so T-2976,
T-2977, T-2979, T-2986 closed in the intervening 4 days, consistent with the T-3089
handback series' own claimed closures).

## 4. Contradictions / notes (recorded, not classified)

1. Version skew persists: `VERSION` file reads `0.12.25`; `git describe` at HEAD reads
   `v0.12.0-26-gc9cefa531` (26 commits past the v0.12.0 tag); the installed
   `~/.local/bin/termlink` reports `0.12.13`. Three different numbers for "what version is
   this" in one terminal. Same class as the 2026-09-21 D-1 root cause, smaller magnitude
   this time (hub binary itself, at `/root/.cargo/bin/termlink`, IS current per F3 above —
   the skew is in a *different* installed copy, `~/.local/bin/termlink`, apparently not the
   one serving the running hub or the running MCP processes).
2. ~30 long-lived `termlink mcp serve` processes observed in the process table, oldest from
   2026-09-21, none obviously orphaned (each pinned to a distinct pts, consistent with
   still-open Claude Code sessions per this host's normal load) — noted but **not**
   classified as a leak without further evidence; flagged as a possible future-round check
   (compare pts liveness against `who`/`ps` at review time) rather than asserted here.

## 5. Not reviewed this round (explicit, per ground rules)

- Full Phase 2 whole-repo inventory (features/verbs/skills/prompts/workflows/modules) —
  unchanged from the 2026-09-19 consolidated 45-finding set; not re-walked.
- The 45 C-01..C-45 findings' current disposition beyond arc-009's task-status field (i.e.
  whether the *content* of each remaining open finding still holds against today's code) —
  out of this round's budget; each `fw inception decide`/task-close event already carries
  its own re-verification per CLAUDE.md's Human Task Completion Rule.
- JS/Python dead-code tooling (knip/vulture/jscpd) — Rust half closed this round (F1); the
  Python `web/` Flask app and shell scripts remain unscanned by dedicated tooling.
- Individual re-verification of the 12 pending inception decisions' `revisit_at` fields
  against the DEFER ages shown in F2 (whether G-053's daily scan is actually catching them)
  — flagged as a good target for a future round, not attempted here.
- Full baseline (`cargo test --workspace`, guard-layer run) — not re-run; T-3090 (closed
  2026-09-24) already established the guard-layer's current timing baseline (945s
  contended), and re-running a multi-minute suite for a delta-scoped review would violate
  "don't pollute what you measure" for no new signal.

Budget consumed this round: light — Phase 0/1 verification, one tool install + workspace
scan, and ~6 targeted greps/reads. Well inside the 200k allowance.
