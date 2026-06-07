---
id: T-2030
name: "Substrate (T-2019 Slice 2): channel.renew RPC + lazy-expiry queries"
description: >
  Slice 2 of 3 from T-2019 GO. Builds on Slice 1's claims table. New RPC verb channel.renew
  (params: claim_id, lease_secs → claimed_until, or claim_expired/not_found error).
  Lazy expiry mechanism (T-1155 no-background-threads compliance): at every claim/list,
  evaluate WHERE claimed_until < now to release stale claims before checking availability
  — no reaper task. Unit tests for renewal flow, expiry-on-stale-attempt, renew-after-expiry-fails.
  ~1d.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-parallel-substrate, slice-2]
components: []
related_tasks: [T-2019, T-2018]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-07T12:12:15Z
last_update: 2026-06-07T13:01:45Z
date_finished:
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
  - ts: '2026-06-07T12:12:34Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
---

# T-2030: Substrate (T-2019 Slice 2): channel.renew RPC + lazy-expiry queries

## Context

Slice 2 of 3 from [T-2019 GO](../../docs/reports/T-2019-claim-semantics-inception.md). Builds on
Slice 1 ([T-2029](T-2029-substrate-t-2019-slice-1-claims-sqlite-t.md)) which delivered the
`claims` SQLite table and `channel.claim`/`channel.release` RPCs. Slice 2 adds the renewal
verb so a worker whose work outlives the default TTL can extend its lease before expiry —
and adds an explicit gate: an expired claim cannot be renewed silently (sweep-on-attempt).

The "lazy-expiry queries" part of the description is satisfied: T-2029 already implemented
sweep-on-claim. Slice 2 extends the same pattern to renew (sweep-then-check) so we don't
ship a path where renew can succeed against a row that's already past `claimed_until`.

## Acceptance Criteria

### Agent
- [x] `CHANNEL_RENEW` method constant added to `crates/termlink-protocol/src/control.rs::method` with doc comment naming `T-2030` and pointing at error codes
- [x] `CLAIM_EXPIRED` error code (`-32018`) added to `crates/termlink-protocol/src/control.rs::error_code` for "renew on expired claim"
- [x] `Bus::renew_claim(claim_id, claimer, additional_ttl_ms) -> Result<ClaimInfo>` public method on `crates/termlink-bus/src/lib.rs::Bus`
- [x] `Meta::renew_claim` implementation: gates on `claimed_until > now_ms` (sweeps + returns `ClaimExpired` otherwise), gates on `claimed_by == claimer` (else `ClaimNotOwned`), updates `claimed_until = now_ms + additional_ttl_ms`, returns the refreshed `ClaimInfo`
- [x] `BusError::ClaimExpired { claim_id }` variant in `crates/termlink-bus/src/error.rs`
- [x] `handle_channel_renew` handler in `crates/termlink-hub/src/channel.rs` — params `{claim_id, claimer, additional_ttl_ms?}`; clamps `additional_ttl_ms` to 1h max, default 30s; returns full refreshed `ClaimInfo` shape
- [x] Router match arm in `crates/termlink-hub/src/router.rs` for `CHANNEL_RENEW`
- [x] `CHANNEL_RENEW` added to the `methods` list in `handle_hub_capabilities`
- [x] Unit test: `renew_claim` extends `claimed_until` past original deadline (claim with short TTL, renew with longer additional_ttl_ms, observe new `claimed_until > old claimed_until + 100ms`)
- [x] Unit test: `renew_claim` on an already-expired claim returns `ClaimExpired` (1ms TTL, sleep 20ms, renew → error)
- [x] Unit test: `renew_claim` by a non-owner returns `ClaimNotOwned`
- [x] `cargo build --release -p termlink-hub` clean
- [x] `cargo test --release -p termlink-bus -p termlink-hub -p termlink-protocol` clean (all existing tests still pass + 3 new tests)

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

cargo build --release -p termlink-hub 2>&1 | tail -5 | grep -qE "Compiling|Finished"
out=$(cargo test --release -p termlink-bus -p termlink-hub -p termlink-protocol 2>&1); echo "$out" | grep -qE "test result: ok"
grep -q "CHANNEL_RENEW" crates/termlink-protocol/src/control.rs
grep -q "CLAIM_EXPIRED" crates/termlink-protocol/src/control.rs
grep -q "fn renew_claim\|renew_claim(" crates/termlink-bus/src/lib.rs
grep -q "handle_channel_renew" crates/termlink-hub/src/channel.rs
grep -q "ClaimExpired" crates/termlink-bus/src/error.rs

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

### 2026-06-07 — Slice 2 closed in the same session as Slice 1
- **What changed:** Slice 2 lands directly behind Slice 1 with no structural surprises:
  the renew path is a textbook UPDATE-WHERE on the same row claim/release operate on,
  guarded by the same `claimed_by==claimer` rule. The one design point that emerged
  during build was naming the post-lapse condition: I chose a separate error code
  `CLAIM_EXPIRED (-32018)` distinct from `CLAIM_NOT_FOUND (-32016)` so the client
  can tell "your lease lapsed, fetch a fresh claim" from "wrong id". Lazy eviction
  of the stale row inside `renew_claim` itself (not a separate query) means the slot
  is reclaimable immediately after the failing renew — no second round-trip required.
- **Plan impact:** None. Slice 3 (T-2031) will surface this distinction in client-side
  helper return types (the `claim_with_renewal` task can branch on ClaimExpired to
  re-claim from the cursor automatically).
- **Triggered:** None.

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-06-07T12:12:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2030-substrate-t-2019-slice-2-channelrenew-rp.md
- **Context:** Initial task creation

### 2026-06-07T13:01:45Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-06-07 — Slice 2 build complete — all 13 Agent ACs ticked
- **Action:** Implemented `Bus::renew_claim` + `Meta::renew_claim` in `crates/termlink-bus/{lib,meta}.rs`, `channel.renew` RPC verb + `handle_channel_renew` in `crates/termlink-hub/src/channel.rs`, router arm + capabilities listing in `crates/termlink-hub/src/router.rs`, `CHANNEL_RENEW` method constant + `CLAIM_EXPIRED (-32018)` error code in `crates/termlink-protocol/src/control.rs`, `BusError::ClaimExpired` variant in `crates/termlink-bus/src/error.rs`.
- **Tests:** 3 new unit tests in `crates/termlink-bus/src/lib.rs` — (1) renew extends `claimed_until` past original deadline by >100ms, (2) renew on expired claim returns `ClaimExpired` AND evicts stale row so reclaim succeeds, (3) renew by non-owner returns `ClaimNotOwned` while original claim remains intact. All 3 pass; full crate suites green: termlink-bus 38/38, termlink-hub 306/306, termlink-protocol 100/100.
- **Verification:** All 7 Verification commands pass (release build, release tests across 3 crates, 5 grep symbol checks).
- **Next step:** Slice 3 (T-2031) — client-side helpers (`claim_with_renewal` async helper with auto-renew background task) + integration tests across the full RPC surface. Slice 3 touches the client crate which I haven't read yet — out of scope for the current chained-slice burst.
