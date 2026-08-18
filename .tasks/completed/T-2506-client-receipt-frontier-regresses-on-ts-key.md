---
id: T-2506
name: "Client-side receipt-frontier reducers key on latest ts, regressing the delivery
  frontier"
description: "The CLI + MCP client-side receipt aggregators keep the receipt with
  the latest ts instead of the highest up_to. A later-but-lower receipt (out-of-order,
  or an operator ack --up-to smaller) regresses the monotonic delivery frontier →
  check-outbox/awaiting-ack over-report already-read offsets as unread. Regression-of-omission:
  the hub reducer was fixed for this exact class in T-2456; the 5 client-side copies
  were never brought in line."
status: work-completed
workflow_type: build
horizon:
owner: agent
created: 2026-08-03
last_update: '2026-08-18T18:59:12Z'
tags: [reliability, correctness, receipts, delivery-confirmation, regression]
components: [crates/termlink-cli/src/commands/channel.rs, 
      crates/termlink-mcp/src/tools.rs]
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:48Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 3
      D2: 5
      D3: 0
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=3 (body:test-or-audit-check); D2=5 
      (body:silent-class-removed); D3=0 (no-signal); D4=3 
      (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:12Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

## Context

Correctness class (not silent-failure — the silent-failure class is exhausted). Found
by a broadened adversarial sweep (19th) after two clean silent-failure sweeps.

A read-receipt's `up_to` is a MONOTONIC delivery frontier ("I have received up to
offset N"). The hub's authoritative reducer `walk_receipt_records`
(termlink-hub/src/channel.rs:1277-1278) keeps the HIGHER `up_to`, ties broken by newer
`ts` (fixed in T-2456). But five client-side reducers still carried the pre-T-2456
predicate — "keep latest `ts`" — so a receipt arriving with a later `ts` but a smaller
`up_to` overwrites the higher frontier with the lower one.

Downstream, `check-outbox` / `awaiting-ack` compute
`outbound_unread = count-1 - max(peer_receipt.up_to)`, so a regressed frontier
over-reports already-confirmed offsets as unread — the delivery-confirmation charter
path producing a wrong, plausible answer.

## RCA

- **Symptom:** Sender S posts receipt {up_to=9, ts=100}, then {up_to=4, ts=200}
  (out-of-order arrival, or an operator `channel ack --up-to 4`). Correct merged
  frontier = 9; the buggy reducers yield 4. `awaiting-ack` then reports offsets 5–9
  as unread though the peer confirmed them.
- **Root cause:** Five client-side receipt reducers keyed the merge on `ts`
  (`prev.ts > ts`), not on the monotonic `up_to`. One site (channel.rs:8560) had NO
  `up_to` comparison at all.
- **Why the framework was blind:** T-2456 fixed the class in the HUB reducer only; the
  CLI + MCP client-side copies (walker fallbacks + topic-summary aggregators) are
  independent duplicated logic and were never swept. The existing MCP tests asserted
  only cases where both predicates agree (they never exercised later-ts-lower-up_to),
  so they passed and masked the bug.
- **Fix class:** Level C — extract one pure `receipt_frontier_replaces` helper per
  crate (T-2069 duplicated-not-shared convention), route all sites through it, mirror
  the blessed T-2456 semantics. Prevents re-drift (a revert breaks a unit test).
- **Learning tie-in:** direct application of PL-291 (paired/duplicated logic must be
  swept at ALL sites in one pass) and PL-259 (parse/reduce-boundary correctness).

## Acceptance Criteria

### Agent
- [x] A pure `receipt_frontier_replaces(prev_up_to, prev_ts, up_to, ts)` helper added to CLI (channel.rs) and MCP (tools.rs), mirroring the hub T-2456 semantics (higher up_to wins; ties → newer ts)
- [x] All 3 CLI client-side receipt reducers (channel.rs receipts-walker, topic-summary, awaiting-ack fallback) routed through the helper
- [x] Both MCP client-side receipt reducers (tools.rs walker fallback + topic-summary) routed through the helper
- [x] Regression test proves a later-ts-lower-up_to receipt does NOT regress the frontier (the core bug), in both crates
- [x] The two stale MCP tests that re-implemented the buggy ts-keyed predicate are replaced with helper-routed tests
- [x] `cargo test -p termlink --bin termlink` passes
- [x] `cargo test -p termlink-mcp --lib` passes

## Verification

# CLI tests pass (includes new receipt_frontier tests)
out=$(cargo test -p termlink --bin termlink 2>&1); echo "$out" | tail -3; echo "$out" | grep -q "test result: ok"
# MCP tests pass
out=$(cargo test -p termlink-mcp --lib 2>&1); echo "$out" | tail -3; echo "$out" | grep -q "test result: ok"
# No client-side receipt reducer still keys on prev.ts
! grep -rn 'prev.ts > ts' crates/termlink-cli/src/commands/channel.rs crates/termlink-mcp/src/tools.rs

## Decisions

Extract-one-helper-per-crate rather than a shared crate export — follows the existing
T-2069 duplicated-not-shared convention for tiny pure helpers already used throughout
termlink-mcp/tools.rs. Semantics are already blessed and shipped (T-2456 hub reducer);
this only brings the client-side copies into line, so there is no user-facing behavior
change (output values become correct).

## Reviewer Verdict (v1.5)

- **Scan ID:** R-4a8709df
- **Timestamp:** 2026-08-02T22:42:44Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-02T22:41:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
