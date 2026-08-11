---
id: T-2604
name: "claim ttl_ms has no lower clamp — ttl_ms=0 returns instantly-dead claim as ok:true (silent failure)"
description: >
  Verb-3 hunt F2: hub channel.rs ttl_ms clamps upper-only; 0 yields dead-but-success claim

status: captured
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-11T10:05:11Z
last_update: 2026-08-11T10:05:11Z
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
- [ ] A deliberate decision on sub-floor `ttl_ms` handling is recorded (see Decisions)
      and implemented at the hub claim handler (`channel.rs`), covering `ttl_ms=0` and
      any chosen minimum floor.
- [ ] If "loud-reject": `claim` with `ttl_ms=0` (or below the floor) returns a clear
      error (e.g. -32602 invalid-params naming the minimum), NOT `ok:true`. If "floor to
      a sane minimum": the effective TTL is raised to a documented floor (e.g. 1000 ms)
      AND the response reflects the effective (floored) `claimed_until` so the client is
      not misled — with the floor documented at the hub + MCP + CLI doc surfaces.
- [ ] The chosen behavior is consistent across the claim entry points that accept
      `ttl_ms` (hub RPC handler; verify the MCP/CLI defaults path is unaffected or
      updated to match).
- [ ] A load-bearing test proves it: a claim request with `ttl_ms=0` yields the chosen
      outcome (error, or floored-and-live claim), NOT a born-dead success. Prove
      load-bearing by temp-reverting the clamp and confirming the test FAILS.
- [ ] `cargo test -p termlink-hub` passes.

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

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

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

### OPEN — what should a sub-floor `ttl_ms` (esp. 0) do?

- **Option A — loud-reject.** `ttl_ms` below a minimum floor returns invalid-params
  naming the minimum. Cleanest honoring of "no silent failures"; a zero-lifetime claim
  is nonsensical so refusing it is defensible. Cost: a wire-contract change (previously
  `ok:true`, now an error) — but the prior success was itself the bug.
- **Option B — floor to a sane minimum (e.g. 1000 ms) and reflect it.** Backward-
  compatible (still `ok:true`), but the response MUST carry the effective floored
  `claimed_until` and the floor MUST be documented, else it silently substitutes a value
  the caller didn't ask for (a different silent-behavior smell).

Note: `.max(1)` alone is NOT acceptable — a 1 ms lease is still effectively born dead;
it moves the bug by 1 ms rather than fixing it. Owner to confirm A vs B before coding.

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
