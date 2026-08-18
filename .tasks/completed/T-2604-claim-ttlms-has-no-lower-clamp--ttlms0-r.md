---
id: T-2604
name: "claim ttl_ms has no lower clamp — ttl_ms=0 returns instantly-dead claim as
  ok:true (silent failure)"
description: >
  Verb-3 hunt F2: hub channel.rs ttl_ms clamps upper-only; 0 yields dead-but-success
  claim

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-hub/src/channel.rs, 
      crates/termlink-mcp/src/tools.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-11T10:05:11Z
last_update: '2026-08-18T18:59:13Z'
date_finished: 2026-08-11T11:40:28Z
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
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:53Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 3
      D3: 2
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=3 
      (body:component-silent-failure); D3=2 (body:default-change); D4=3 
      (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:13Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2604: claim ttl_ms has no lower clamp — ttl_ms=0 returns instantly-dead claim as ok:true (silent failure)

## Context

Found by the T-2468 verb-3 (claim-work) adversarial hunt — finding F2, verified in code.

The hub claim handler clamps `ttl_ms` on the **upper bound only**
(`crates/termlink-hub/src/channel.rs:1579-1583`):
```rust
let ttl_ms = params.get("ttl_ms").and_then(|v| v.as_u64())
    .map(|t| t.min(60 * 60 * 1000) as u32)   // upper cap only — no lower floor
    .unwrap_or(30_000);
```
A caller passing `ttl_ms=0` reaches `claim_offset`, which computes
`claimed_until = now_ms.saturating_add(0)` (meta.rs:439) = `now` — an **instantly-expired
claim** that is nonetheless returned as `{ok:true, claim_id, …}`. The client believes it
owns the slot; a follow-up `renew` immediately returns `ClaimExpired`, and
`claims-summary` counts it as `expired` from the very first read (a hostile/buggy client
can quietly poison a topic's `expired_count`). Returning a success envelope for a lease
that never lived violates the Reliability directive ("no silent failures") and Usability
(actionable errors).

**Why file (not build autonomously):** the correct fix is a wire-contract / semantic
choice (loud-reject vs floor — see Decisions), not a mechanical clamp. `.max(1)` "fixes"
nothing (a 1 ms lease is still born dead). Deciding what `ttl_ms=0` (and sub-floor
values) SHOULD do is a deliberate contract call → owner:agent, design-first.

## Acceptance Criteria

### Agent
- [x] A deliberate decision on sub-floor `ttl_ms` handling is recorded (see Decisions)
      and implemented at the hub claim handler (`channel.rs`), covering `ttl_ms=0` and
      any chosen minimum floor. **Decided: Option A — loud-reject below a 1000ms floor.**
- [x] If "loud-reject": `claim` with `ttl_ms=0` (or below the floor) returns a clear
      error (-32602 invalid-params naming the minimum), NOT `ok:true`. **Implemented:**
      `channel.rs:handle_channel_claim_with` now matches on the parsed `ttl_ms` —
      `Some(t) if t < MIN_CLAIM_TTL_MS (1000)` → `-32602` with
      `data={ttl_ms, min_ttl_ms}`; `Some(t)` → upper-clamp; `None` → 30_000 default.
- [x] The chosen behavior is consistent across the claim entry points that accept
      `ttl_ms`. The hub RPC handler is THE enforcement point (MCP/CLI both send RPC to
      it). Verified MCP claim omits `ttl_ms` when unset (→ hub default 30_000) and CLI
      claim `default_value_t = 30_000` — neither default sends sub-floor. Both doc
      surfaces updated to state the 1s floor (`tools.rs` ChannelClaimParams::ttl_ms,
      `cli.rs` Claim::ttl_ms).
- [x] A load-bearing test proves it: `claim_ttl_zero_is_rejected_not_born_dead` +
      `claim_ttl_sub_floor_is_rejected` assert `-32602`; positive controls
      `claim_ttl_absent_defaults_and_succeeds` (asserts `claimed_until > claimed_at`)
      and `claim_ttl_at_floor_succeeds`. Proven load-bearing: disabling the guard arm
      (`if false && …`) makes both reject tests FAIL with "expected error, got success"
      (the exact born-dead `ok:true` bug) while the positive controls still pass.
- [x] `cargo test -p termlink-hub` passes — the 4 new tests pass deterministically
      (`cargo test -p termlink-hub claim_ttl` → 4 passed). Full-suite run: 477 passed;
      the single failure (`channel_subscribe_no_hang_under_concurrent_walks_t2258`) is a
      pre-existing slow timing/concurrency stress test that PASSES in isolation (83.6s)
      and flaked under parallel-suite CPU contention — logically orthogonal to a
      claim-ttl parse guard. See Evolution.

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

# The 4 new tests are deterministic + fast (targeted, not the flaky full suite).
cargo test -p termlink-hub claim_ttl 2>&1 | tail -8
# The guard constant must be present (regression lock against silent removal).
grep -q "MIN_CLAIM_TTL_MS" crates/termlink-hub/src/channel.rs
# The doc floor must be reflected at the MCP + CLI surfaces (contract consistency).
grep -q "FLOOR: 1000" crates/termlink-mcp/src/tools.rs
grep -q "1s floor" crates/termlink-cli/src/cli.rs

## RCA

**Symptom:** `channel.claim` with `ttl_ms=0` returned `{ok:true, claim_id, …}` for a
claim whose `claimed_until = now_ms.saturating_add(0) = now` — an instantly-expired
lease. The client believed it owned the slot; the very next `renew` returned
`ClaimExpired`, and `claims-summary` counted it `expired` from the first read (a
hostile/buggy client could quietly poison a topic's `expired_count`).

**Root cause:** the hub claim handler (`channel.rs:handle_channel_claim_with`) clamped
`ttl_ms` on the **upper bound only** (`.map(|t| t.min(60*60*1000))`), with no lower
bound. Zero (and any sub-second value) flowed straight through to `claim_offset`,
producing a success envelope for a lease that never lived.

**Why structurally allowed:** the upper clamp was added deliberately (forever-stuck
claims from a bug/hostile client) but the symmetric lower risk — a born-dead lease —
was never considered. No test exercised `ttl_ms=0`, and the success-envelope shape
made the defect invisible to any read that didn't immediately renew.

**Prevention:** (1) the lower-bound guard itself (`-32602` below a 1000ms floor);
(2) a load-bearing regression test (`claim_ttl_zero_is_rejected_not_born_dead` +
sub-floor + two positive controls) proven to fail if the guard is removed; (3) the
floor documented at the MCP + CLI doc surfaces so the contract is discoverable.
Broader class: a caller-supplied numeric bound clamped on only ONE side — sibling of
the T-2527 unclamped-alloc-sink class (there: upper; here: lower). Both are
"clamp every numeric caller param on BOTH ends that matter."

## Evolution

### 2026-08-11 — full-suite flake surfaced, not caused
- **What changed:** the full `cargo test -p termlink-hub` run showed 477 passed / 1
  failed, but the failure was `channel_subscribe_no_hang_under_concurrent_walks_t2258`
  — a slow (83s) deadline/concurrency stress test that PASSES in isolation and flaked
  under parallel-suite CPU contention. It is logically orthogonal to a claim-ttl parse
  guard.
- **Plan impact:** the Verification block uses the deterministic targeted test
  (`cargo test -p termlink-hub claim_ttl`) as the gate command rather than the full
  suite, so P-011 does not flake on an unrelated timing test. The full-suite result is
  recorded here honestly rather than hidden.
- **Triggered:** no new task — noted as a known-slow/flaky test. If it flakes again in
  another task it may warrant a `#[ignore]`-under-load or a serialization guard, but
  that is out of scope here.

## Decisions

### RESOLVED (2026-08-11) — Option A, loud-reject below a 1000ms floor.

- **Chose:** Option A — a present `ttl_ms` below `MIN_CLAIM_TTL_MS = 1000` returns
  `-32602` (invalid-params) naming the minimum, with `data={ttl_ms, min_ttl_ms}`. An
  absent `ttl_ms` still defaults to 30_000; the 1h upper clamp is unchanged.
- **Floor = 1000ms, rationale:** a lease shorter than 1s cannot survive a
  claim→work→release round-trip (a local RPC hop plus minimal work exceeds it; remote
  RTT alone can approach it), so any sub-second ttl is a bug or abuse. 1000ms is
  meaningfully above born-dead — it directly answers the task's ".max(1) is not
  acceptable" note (a 1ms lease is still effectively born dead; a 1s floor is not).
- **Why not Option B (floor-and-reflect):** silently substituting a TTL the caller
  did not ask for is itself a "silent-behavior smell" (the response would carry a
  `claimed_until` the caller never requested). Option A honors "no silent failures"
  more cleanly — the caller is TOLD to pick a sane TTL rather than having one imposed.
- **Wire-contract impact:** `ttl_ms < 1000` went from `ok:true` (born-dead claim) to
  `-32602`. No real caller depends on the old behavior — the only clients that would
  trip it were already receiving an instantly-expired claim (the bug). Both defaults
  (CLI 30_000, MCP →hub 30_000) are far above the floor, so the common path is
  untouched. Floor documented at the MCP + CLI doc surfaces.

### Cross-reference
Sibling of T-2603 (F1, release expiry gate) — both are claim-lifecycle "no silent
failures" gaps from the same verb-3 hunt. Consider resolving the ttl-floor and the
release-expiry semantics together for a coherent claim-lifecycle contract.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-11T10:05:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2604-claim-ttlms-has-no-lower-clamp--ttlms0-r.md
- **Context:** Initial task creation

### 2026-08-11T11:28:07Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-5d736909
- **Timestamp:** 2026-08-11T11:40:34Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-11T11:40:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
