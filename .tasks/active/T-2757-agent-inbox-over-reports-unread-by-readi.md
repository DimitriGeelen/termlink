---
id: T-2757
name: "agent inbox over-reports unread by reading a cursor nothing advances"
description: >
  termlink agent inbox reports 388 unread on agent-chat-arc and 131 on a DM thread where the receipt frontier says 14 and 1. It reads cursors.json, which only subscribe --resume advances, while every conversation-arc tool (check-arc, reply, agent ack, channel ack) advances the receipt frontier instead. Nothing reconciles the two, so the number never clears no matter how diligently an agent reads and acks.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-16T07:04:13Z
last_update: 2026-08-16T10:07:18Z
date_finished: null
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── BVP scoring fields (T-1918, arc-006). See docs/reports/T-1915-bvp-inception.md for semantics. ──
# bvp_scores:                     # confirmed per-driver scores 0-5, set by `fw bvp confirm` (T-1924).
#                                 # Sovereignty boundary — only set after human or agent confirmation.
#                                 # Shape: {D1: <int 0-5>, D2: <int 0-5>, D3: <int 0-5>, D4: <int 0-5>, [<free-driver-id>: <int>]...}
# bvp_scores_proposed:            # estimator-proposed scores (T-1922 worker). Persists when ≥2 delta
#                                 # from bvp_scores: on any driver (M3 v2-delta). Shape: list of timestamped entries.
# cost_estimate:                  # F8 composite: 0.6×blast_radius + 0.3×tier + 0.1×effort.
#                                 # Q2 fallback: T-shirt S/M/L/XL mapped to 2/4/6/8 when blast_radius is not yet computable.
---

# T-2757: agent inbox over-reports unread by reading a cursor nothing advances

## Context

`termlink agent inbox` (CLI T-1553 / MCP `termlink_agent_inbox`) is the
"what needs my attention?" verb — the one an agent calls to decide whether it
has mail. Measured on this host, same identity, same hub, same moment:

| topic | `agent inbox` | `channel unread` (receipts) |
|---|---|---|
| `agent-chat-arc` | **388** | **14** |
| `dm:9219671e...:d1993c2c...` | **131** | **1** |

A 27x and 131x over-report from the verb whose entire job is that number.

### Two independent causes, measured not inferred

**(1) The digest reads a frontier the daily workflow never advances.**
`agent inbox` computes unread from `~/.termlink/cursors.json`, which is written
only by `subscribe --resume`. Every conversation-arc tool — `/check-arc`,
`/reply`, `agent ack`, `channel ack` — records consumption as a *receipt*
envelope (`msg_type=receipt`, `metadata.up_to`) instead. Nothing reconciles the
two. Direct proof on this machine: the last receipt this identity posted to the
DM topic says `up_to=174`, while the cursor for that same topic sits at **44**.
The mail was read and acked; the cursor did not move, and cannot.

That makes the number **unclearable by any amount of diligent reading** — the
PL-340 / T-2709 latch shape. A count that only ever grows regardless of what the
operator does is precisely how a reader learns to stop reading it.

**(2) Count-vs-offset decoupling — PL-293, on a topic that is already trimmed.**
`agent-chat-arc` is `retention messages:2000` and saturated at count=2000, while
its true latest offset is **11966**. `compute_unread_rows_mcp` (tools.rs:3489)
prefers the hub's authoritative `latest_offset` and falls back to `count - 1`.
T-2533 shipped that preference and the hub emits the field at
`channel.rs:1691` — but the **running hub predates T-2533 and does not emit it**
(verified: `channel list` returns no `latest_offset` for any topic), so the
fallback always wins and `latest` reads 1999 instead of 11966.

PL-293 closes with: *"When you fix one count-anchored read, grep for ALL callers
of the count-anchored helper."* This is one that was missed.

Note the two errors point in OPPOSITE directions and do not cancel: cause (2)
makes `latest` too small, cause (1) makes the frontier far too small. Frontier
staleness dominates, so the net is a large over-report.

### Scope

Fix cause (1) — reconcile the frontier — in both the MCP tool and its CLI twin.
Cause (2) is already fixed in-tree; it is inert only because the local hub
binary is stale, which is a deploy concern (G-069) and not a code change here.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] A pure helper reconciles the two frontiers by taking the max of the
      subscribe cursor and the receipt `up_to` for the same (topic, identity) —
      consumption recorded by EITHER mechanism counts as consumed
- [x] The helper is unit-tested for: receipt ahead of cursor (the live bug),
      cursor ahead of receipt (subscribe-only workflow, unchanged), no receipt
      at all (pre-existing behaviour preserved), and equal frontiers
- [x] Both `termlink_agent_inbox` (MCP) and the `termlink agent inbox` CLI twin
      use the reconciled frontier, so the two surfaces cannot disagree
- [x] A topic whose receipt frontier covers every message is omitted from the
      digest entirely rather than reported with a stale non-zero count
- [x] Receipt lookup degrades safely: a hub that does not serve the receipts
      fast-path falls back rather than erroring the whole digest
- [x] The reconciliation cannot silently UNDER-report: when the hub gives no
      authoritative `latest_offset` and the receipt frontier exceeds the
      count-derived `latest` (proving the two are different units), the topic is
      reported as indeterminate with the cause and remedy named — never dropped
      as "caught up" and never given a fabricated number
- [x] Live proof against the hub that produced the original numbers: the DM
      reports the same value as `channel unread`, and the retention-saturated
      topic reports indeterminate rather than vanishing
- [x] `cargo test --workspace` passes
- [x] Guard layer passes (`bash scripts/run-guard-layer.sh`)

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `## Verification` instead of a Human AC here. Only keep [REVIEW] if
     verification genuinely needs human taste (tone, feel, layout rhythm).
     See CLAUDE.md §AC Classification Guidance for the conversion rule.

     [REVIEW] example (genuine human judgment):
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error

     [REVIEWER] example (static-scan-verifiable — convert to Agent AC + Verification):
       - [ ] [REVIEWER] Block message names both bypass mechanisms
         **Steps:**
         1. Run `bin/fw reviewer T-XXX`
         **Expected:** Verdict: PASS; no findings on `block-message-completeness`
         **If not:** Inspect hook block-message string and add missing mechanism
       Conversion: this AC should be moved to ### Agent and
       `bin/fw reviewer T-XXX 2>&1 | grep -q "Overall:.*PASS"` added to ## Verification.
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# Pipefail/SIGPIPE hint (L-387): P-011 runs each command under `set -eo pipefail`.
# `cmd | grep -q PATTERN` exits 141 (SIGPIPE) when grep matches and closes stdin
# while the upstream is still writing — verification then "fails" even though
# the pattern was present. Safe pattern: capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Or:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before this hint.
#
# Single pipe only — no intermediate tail/awk/sed stages between capture and grep
# (T-2090): `echo "$out" | tail -3 | grep -q PAT` re-introduces the SIGPIPE risk
# the capture step closed off — the middle stage is what `grep -q` slams its
# stdin on. `echo "$out"` is small and immediate; grep scans the whole captured
# string anyway, so the tail-3 was cosmetic. Drop it: `echo "$out" | grep -q PAT`.
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

# The four reconciliation orderings, CLI side. Asserted by NAME rather than by a
# count, so adding a test later does not break verification while a deleted or
# renamed test still does.
out=$(cargo test -p termlink reconcile_frontier 2>&1); echo "$out" | grep -q "0 failed"
out=$(cargo test -p termlink reconcile_frontier 2>&1); echo "$out" | grep -q "reconcile_frontier_receipt_ahead_of_cursor_wins ... ok"
out=$(cargo test -p termlink reconcile_frontier 2>&1); echo "$out" | grep -q "reconcile_frontier_cursor_ahead_of_receipt_wins ... ok"
out=$(cargo test -p termlink reconcile_frontier 2>&1); echo "$out" | grep -q "reconcile_frontier_no_receipt_is_cursor ... ok"

# The measured production defect, CLI side. This test pins BOTH numbers — the
# reconciled 1 and the pre-fix 131 — so it cannot pass against a helper that
# just returns 0.
out=$(cargo test -p termlink compute_unread_rows_receipt_frontier_collapses_the_overreport 2>&1); echo "$out" | grep -q "1 passed"

# The near-miss guard: a receipt beyond a count-derived latest must NOT be read
# as caught-up. Without this the fix silently hides real unread.
out=$(cargo test -p termlink compute_unread_rows_stale_hub_receipt_beyond_count_is_indeterminate_not_dropped 2>&1); echo "$out" | grep -q "1 passed"
out=$(cargo test -p termlink unread_verdict_indeterminate_only_on_the_fallback_path 2>&1); echo "$out" | grep -q "1 passed"

# Same defect and same guard, MCP side, plus the CLI/MCP agreement pins.
out=$(cargo test -p termlink-mcp mcp_reconcile_frontier 2>&1); echo "$out" | grep -q "0 failed"
out=$(cargo test -p termlink-mcp mcp_agent_inbox 2>&1); echo "$out" | grep -q "0 failed"
out=$(cargo test -p termlink-mcp mcp_agent_inbox_stale_hub_receipt_beyond_count_is_indeterminate 2>&1); echo "$out" | grep -q "1 passed"
out=$(cargo test -p termlink-mcp mcp_reconcile_frontier_matches_cli_semantics 2>&1); echo "$out" | grep -q "1 passed"
out=$(cargo test -p termlink-mcp mcp_unread_verdict_matches_cli_semantics 2>&1); echo "$out" | grep -q "1 passed"

# Both surfaces must actually CALL the reconciliation — a helper that exists but
# is unwired is the T-2699 shape (covered builder, zero callers).
grep -q "reconcile_consumption_frontier(\*cursor, receipt_up_to)" crates/termlink-cli/src/commands/channel.rs
grep -q "reconcile_consumption_frontier_mcp(\*cursor, receipt_up_to)" crates/termlink-mcp/src/tools.rs

# The digest must fetch receipts, not just accept them as a parameter.
grep -q "CHANNEL_RECEIPTS" crates/termlink-cli/src/commands/channel.rs
grep -q "fetch_receipt_frontier(&sock, topic, &fp)" crates/termlink-cli/src/commands/channel.rs

# The reconciliation must stay auditable: both inputs reported, not just the result.
grep -q "receipt_up_to" crates/termlink-cli/src/commands/channel.rs
grep -q '"frontier": r.frontier' crates/termlink-mcp/src/tools.rs

# The CLI help must no longer assert the un-reconciled behaviour as the design.
grep -q "RECONCILED consumption frontier" crates/termlink-cli/src/cli.rs

# Full suite + guard layer.
cargo test --workspace
bash scripts/run-guard-layer.sh

## RCA

**Symptom:** `termlink agent inbox` / `termlink_agent_inbox` reported 388 unread
on `agent-chat-arc` and 131 on a DM topic, at a moment when the receipt-based
`channel unread` reported 14 and 1 for the same identity on the same hub. The
number did not move in response to reading and acking mail, because nothing it
reads is advanced by reading and acking mail.

**Root cause:** two independently-maintained consumption frontiers with no join.
`cursors.json` is advanced only by `subscribe --resume`; the receipt frontier
(`msg_type=receipt` / `metadata.up_to`) is advanced by `channel ack` / `agent
ack` and every conversation-arc tool sitting on them. The digest keyed on the
first while the daily workflow advanced only the second. Measured on the origin
host: receipt `up_to=174` against cursor `44` on the same DM topic.

**Why structurally allowed:** the split was *documented as intentional* — the
CLI help for `Inbox` said "Distinct from `channel unread <topic>` which is
single-topic + receipt-based". That sentence describes the mechanism accurately
and says nothing about the consequence, so it reads as a design note rather than
a warning. Two verbs answering "have I read this?" with answers differing by
27x is not a difference in scope; it is a contradiction, and documenting the
mechanism made it look intended. No test compared the two, because each was
correct against its own store — the defect existed only in the relationship
between them, which nothing owned.

Compounding it, the count could only ever rise. A number that no operator
action can clear is the PL-340 / T-2709 shape: the reader stops believing it,
so the wrongness stops being reported. That is why this survived rather than
being noticed the first time the two disagreed.

**Prevention:**
- `reconcile_consumption_frontier` (CLI) / `_mcp` (MCP) is now the single place
  the join happens, unit-tested in both crates for all four orderings.
- The regression tests assert BOTH numbers: they pin the reconciled result (1)
  *and* that the pre-fix input still reproduces the old value (131). A test that
  only pinned the new number would pass against a helper that always returned 0.
- `mcp_reconcile_frontier_matches_cli_semantics` pins CLI/MCP agreement, so the
  two surfaces cannot silently drift apart again in either direction.
- Rows now carry `cursor`, `receipt_up_to` and `frontier` together, so a future
  divergence is visible in the output itself rather than only in a comparison
  nobody runs.
- PL-350 records the general shape: when two stores can answer the same
  question, something must join them or they will disagree silently — and
  documenting the split is not a join.
- PL-351 records the near-miss: the first version of this fix silently
  under-reported, and every unit test passed, because the tests encoded the
  same assumption as the code. Only a live run against real data caught it.

**Known remaining gap (not closed here).** `termlink_agent_inbox` has NO
MCP/CLI parity assertion — it is line 60 of
`.context/checks/mcp-parity-census-allowlist`, one of the 236 tools T-2747
enumerated as unexamined and T-2748 is working down. This task changed both
surfaces in lockstep by hand, which is exactly the situation a parity test
exists to police. The compensating control is
`mcp_reconcile_frontier_matches_cli_semantics`, a unit-level pin that the two
reconciliation helpers agree across five input orderings — narrower than a real
parity case (it pins the helper, not the emitted JSON), and deliberately noted
rather than left implied. Promoting this tool out of the allowlist is a good
first target for T-2748, since both surfaces were just touched and are fresh.

**Not fixed here (deliberate):** the second, smaller cause. `agent-chat-arc` is
retention-bounded at 2000 while its true latest offset is 11966, so the
`count - 1` fallback resolves `latest` to 1999. T-2533 already fixed that in
tree — the hub emits `latest_offset` at `channel.rs:1691` and both helpers
prefer it — but the *running* hub predates T-2533 and omits the field, so the
fallback wins. That is a stale-binary deploy concern (G-069), not a code change,
and it is exactly what the T-2359 fleet-binary canary exists to surface.

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

## Decisions

### 2026-08-16 — reconcile the two frontiers rather than document the split

- **Chose:** compute unread from `max(cursor, receipt up_to)`.
- **Why:** both values are claims by the same identity that it consumed up to
  that offset, so consumption recorded by either counts. `max` is also safe in
  one direction only — it can lower a reported count, never raise one — so it
  cannot manufacture a false alarm.
- **Rejected:** *leave it and document the divergence more loudly.* The split
  was ALREADY documented (`cli.rs`: "Distinct from `channel unread <topic>`
  which is single-topic + receipt-based") and that documentation is precisely
  what made a 27x contradiction read as intended design. More prose describing
  the mechanism would not have helped; the two answers still contradict.
- **Rejected:** *advance `cursors.json` on ack.* That makes acking write to a
  store owned by the subscribe path, coupling two mechanisms at the write side
  where a partial failure corrupts state. Reconciling at READ time is
  recoverable and touches nothing durable.

### 2026-08-16 — report indeterminate instead of guessing (found by live run)

- **Chose:** when `latest` came from the `count - 1` fallback AND the receipt
  frontier exceeds it, emit `unread: null` + `indeterminate: true` with the
  cause and remedy named.
- **Why:** the first version of this fix DROPPED `agent-chat-arc` from the
  digest — frontier 11952 vs a count-derived latest of 1999 looked caught-up —
  silently hiding 14 genuinely unread messages. That trades a loud over-report
  for a silent under-report, which is strictly worse and violates Directive #2.
  A receipt cannot ack an offset that does not exist, so `receipt > latest` is
  PROOF the fallback is not an offset; that proof is available at exactly the
  point the wrong answer would be produced.
- **Rejected:** *use the receipt frontier as `latest` too.* It is a lower bound
  on the true latest, so unread would read 0 whenever the frontier is the
  highest offset we know — a confident wrong answer in the same place.
- **Note:** only a live run surfaced this. Every unit test passed against the
  naive version, because the tests I had written encoded the same assumption as
  the code. The regression tests now pin BOTH numbers (reconciled AND pre-fix)
  so a helper that always returns 0 cannot satisfy them.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-16 — state at handover: work complete, NOT closed

Committed and pushed as `ac38b06b4`. Deliberately left at `started-work`.

The budget gate hit critical (~95%) immediately after the workspace suite
returned, which blocks `cargo` — and the P-011 gate in `## Verification` is
mostly cargo invocations. Closing would have required `--force` or
`--skip-verification`, i.e. a gate bypass, which is not the agent's to take.
The gate exists precisely for this moment.

**Evidence already gathered (all green), so the next session can close quickly:**
- `cargo test --workspace` → **exit 0**, every suite `0 failed` (parity 24/24
  in 825s; MCP lib 923 passed, up from 921 pre-change).
- `bash scripts/run-guard-layer.sh` → **PASS 36/36**, re-run after the doc and
  script edits, not just the code.
- `cargo test -p termlink reconcile_frontier` → 4 passed, confirmed by NAME.
- Earlier dedicated `cargo test -p termlink-mcp` run → all new MCP tests passed.
- Live hub (the one that produced the original numbers): DM reports `1`,
  matching `channel unread` exactly; `agent-chat-arc` reports UNKNOWN naming
  the stale pre-T-2533 hub instead of vanishing.
- `fw audit` → exit 0.

**To close:** re-run the `## Verification` block (it is self-contained), then
`fw task update T-2757 --status work-completed`. The only unverified-in-this-
session lines are the MCP `cargo test` ones, which were green in a dedicated
run before the last two edits and are covered by the workspace run afterwards.

**Not in scope, still open:** the local hub predates T-2533 and does not emit
`latest_offset`, which is what forces the indeterminate path on
`agent-chat-arc`. That is a stale-binary deploy concern (G-069) for the
operator, not a code change — upgrading the hub turns that row into a real
count (the `compute_unread_rows_authoritative_latest_resolves_the_same_case`
test pins that it resolves to 14).

### 2026-08-16T07:04:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2757-agent-inbox-over-reports-unread-by-readi.md
- **Context:** Initial task creation

### 2026-08-16 — gate run, 19/21 legs green

All 9 Agent ACs are checked and the P-011 gate ran cleanly through **19 of its
21 verification commands** (every targeted `cargo test`, every structural grep).
The two outstanding legs are `cargo test --workspace` and
`bash scripts/run-guard-layer.sh` — both of which were run INDEPENDENTLY this
session and were green (`cargo test --workspace` exit 0, all suites `0 failed`;
guard layer `PASS 36/36`, run twice). The gate leg was still executing when the
session hit its budget horizon.

**CORRECTION (same session, after the gate finished):** the gate did NOT pass.
It reached leg 20 and reported `FAIL: cargo test --workspace (exit 101)`, so
T-2757 stays `started-work`. The sentence that stood here — "it is expected to
pass; nothing is known to be outstanding" — was written while the leg was still
running and was wrong.

This is worth reading carefully, because the SAME command exited 0 earlier in
this same session with every suite reporting `0 failed`. Two runs of one
command disagreeing means one of the two results is not trustworthy, and the
green one is not automatically the honest one. Do not close this task on the
strength of the earlier green.

Candidate causes, none verified — do not assume:
  - a build-directory race: a `cargo test` and the guard layer's own cargo work
    were in flight concurrently at points during this session
  - a genuinely flaky test surfacing under different scheduling
  - a real failure that the earlier run somehow did not reach

**To close:** first reproduce `cargo test --workspace` on a quiet tree with no
other cargo process running, and get the ACTUAL failing test name — the P-011
gate truncates failure output to 5 lines, which was only compile chatter here.
Fix or explain what it names, THEN re-run the gate. Do NOT reach for `--force`
or `--skip-verification`: a gate that failed is exactly the signal those flags
would destroy.

**Note on a flaky background run:** an earlier invocation of the same command
reported exit 0 with an EMPTY output file while leaving the task at
`started-work`. That is a harness/backgrounding artifact, not a gate failure —
the re-run reached 19/21 PASS. Worth knowing if a future session sees the same
silent no-op.
