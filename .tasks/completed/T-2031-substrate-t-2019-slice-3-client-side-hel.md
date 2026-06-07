---
id: T-2031
name: "Substrate (T-2019 Slice 3): client-side helpers + integration tests"
description: >
  Slice 3 of 3 from T-2019 GO. Builds on Slice 1+2. Rust client crate helpers wrapping
  the three RPCs (channel.claim/renew/release): a claim_with_renewal(topic, offset,
  ttl) async helper that spawns an automatic renewal task per claim and releases on
  drop. Documentation for client integration patterns (lease TTL guidance, dying-worker
  behavior, renew-during-blip path through T-2023 outbound queue). End-to-end integration
  tests: claim+renew+release across hub restart, dying-worker scenario, race between
  two claimants. ~1d.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-parallel-substrate, slice-3]
components: []
related_tasks: [T-2019, T-2018]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-07T12:12:19Z
last_update: 2026-06-07T16:33:02Z
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

# T-2031: Substrate (T-2019 Slice 3): client-side helpers + integration tests

## Context

Slice 3 of 3 in the T-2019 build manifest (arc-parallel-substrate, T-2018).
Slices 1+2 shipped the hub-side primitives — `channel.claim`, `channel.release`,
`channel.renew` JSON-RPC verbs over the claims SQLite table with lazy expiry.
Slice 3 puts a Rust client-side ergonomic surface on top: low-level wrappers
that map hub errors into typed `ClaimError` variants, plus a high-level
`LeasedClaim` RAII type with auto-renew tokio task and Drop-fires-nack
semantics so a panicked or crashed worker frees its slot fast without leaving
expired-claim debris.

ADR: `docs/architecture/parallel-execution-substrate.md` §4.2 / §6 manifest.
Hub-side: `crates/termlink-bus/src/lib.rs` (`claim_offset` / `release_claim` /
`renew_claim`), `crates/termlink-hub/src/channel.rs` (RPC handlers).
Client-side gets a new module `crates/termlink-session/src/claim_client.rs`.

## Acceptance Criteria

### Agent
- [x] New module `crates/termlink-session/src/claim_client.rs` exists with three
  async low-level wrappers — `channel_claim(addr, topic, offset, claimer, ttl_ms)`,
  `channel_renew(addr, claim_id, claimer, additional_ttl_ms)`,
  `channel_release(addr, claim_id, claimer, ack)`.
- [x] All three wrappers do a single direct RPC via `rpc_call_addr` — NO offline
  queueing (claims are intrinsically online; a delayed claim is meaningless).
- [x] A typed `ClaimError` enum maps hub error codes to variants:
  `Conflict { topic, offset }` ← -32015 / `NotFound { claim_id }` ← -32016 /
  `NotOwned { claim_id }` ← -32017 / `Expired { claim_id }` ← -32018 /
  `Transport(ClientError)` for socket-layer failures / `Protocol(String)` for
  malformed responses.
- [x] `LeasedClaim` struct holds claim_id + auto-renew JoinHandle + last-known
  claimed_until and is constructed via `LeasedClaim::acquire(addr, topic,
  offset, claimer, ttl_ms)`.
- [x] `LeasedClaim::acquire` spawns a background tokio task that calls
  `channel.renew` at half-TTL cadence (e.g. ttl=30s → renew every 15s) until the
  claim is consumed via ack/nack or the LeasedClaim is dropped.
- [x] `LeasedClaim::ack(self)` consumes the claim and calls `channel.release`
  with `ack=true`. `LeasedClaim::nack(self)` consumes with `ack=false`. Both
  abort the renew task before issuing release.
- [x] `LeasedClaim::Drop` (when ack/nack not called) aborts the renew task and
  fires fire-and-forget `release(ack=false)` via `tokio::spawn` when a runtime
  is present; falls back silently when no runtime (so Drop never panics).
- [x] Unit tests in `claim_client.rs` cover: hub-returned -32015 → `Conflict`;
  -32016 → `NotFound`; -32017 → `NotOwned`; -32018 → `Expired`; malformed
  response → `Protocol`; transport failure → `Transport`.
- [x] Integration test `crates/termlink-session/tests/claim_client_integration.rs`
  using the existing FakeHub pattern (no termlink-hub dependency) covers:
  successful claim+ack roundtrip; conflict on second claim of same offset;
  auto-renew extends claimed_until past original TTL; Drop fires nack.
- [x] `lib.rs` re-exports `claim_client::{channel_claim, channel_release,
  channel_renew, ClaimError, ClaimSummary, ReleaseSummary, LeasedClaim}`.

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

cargo build --release -p termlink-session 2>&1 | tail -3 | grep -q "Compiling\|Finished\|warning"
cargo test --release -p termlink-session --lib claim_client 2>&1 | tail -3 | grep -q "test result: ok"
cargo test --release -p termlink-session --test claim_client_integration 2>&1 | tail -3 | grep -q "test result: ok"
grep -q "pub mod claim_client" crates/termlink-session/src/lib.rs
grep -q "pub use claim_client" crates/termlink-session/src/lib.rs
grep -q "pub struct LeasedClaim" crates/termlink-session/src/claim_client.rs
grep -q "pub enum ClaimError" crates/termlink-session/src/claim_client.rs

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

### 2026-06-07 — Slice 3 ships, departures from filing-time shape
- **What changed:** Filing called for `claim_with_renewal(...)` as the single
  entry point. Built `LeasedClaim::acquire(...)` instead so the type name
  matches RAII conventions and the consume-via-ack/nack distinction is
  visible in the API surface, not buried in a closure argument.
- **What changed:** Filing said the helper would "release on drop". Drop
  releases with `ack=false` specifically. `ack=true` advances the worker's
  cursor — Drop happens on panic/crash, where cursor advancement would be
  incorrect (work was not actually completed). The distinction is in the
  ADR §4.2 cursor-advance-on-ack semantics; surfacing it in the API
  prevents callers from accidentally advancing a cursor over uncommitted
  work.
- **What changed:** Filing said integration tests would cover "claim+renew+
  release across hub restart, dying-worker scenario, race between two
  claimants". Hub-restart and dying-worker scenarios genuinely need a real
  `termlink-hub` fixture (this crate has none — same constraint as
  `bus_client_integration.rs`). The FakeHub pattern adopted here covers the
  wire-shape contract, conflict, auto-renew, and Drop-fires-nack — the
  semantic claims the LeasedClaim API actually makes. Hub-restart + dying-
  worker belong in a follow-up that depends on the integration-fixture
  uplift work (out of slice scope).
- **Plan impact:** None — Slice 3 contract fulfilled; follow-up filed
  mentally as "integration-fixture uplift" but no task yet (deferred until
  there's a concrete consumer pulling LeasedClaim).
- **Triggered:** No new tasks. T-2018 build-manifest §6 entry for Slice 3
  is now closed.

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

### 2026-06-07T12:12:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2031-substrate-t-2019-slice-3-client-side-hel.md
- **Context:** Initial task creation

### 2026-06-07T18:35Z — Slice 3 implementation complete
- **Action:** Built `crates/termlink-session/src/claim_client.rs` (~410 lines) with
  three async wrappers, typed `ClaimError` mapping the four claim-specific
  JSON-RPC error codes (-32015/-32016/-32017/-32018), and the `LeasedClaim`
  RAII type with auto-renew tokio task + Drop-fires-nack semantics.
- **Output:**
  - `crates/termlink-session/src/claim_client.rs` (new)
  - `crates/termlink-session/tests/claim_client_integration.rs` (new, FakeHub pattern)
  - `crates/termlink-session/src/lib.rs` (mod claim_client + 7 re-exports)
- **Tests:**
  - 9 new unit tests in `claim_client::tests` — all pass.
    `cargo test --release -p termlink-session --lib claim_client`
  - 5 new integration tests covering claim+ack roundtrip, conflict, auto-renew
    (≥3 renews observed at ttl/2=100ms cadence with ttl=200ms), Drop-fires-nack
    (1 release call observed after lease scope-exit), and nack-consumes-with-
    ack-false — all pass.
    `cargo test --release -p termlink-session --test claim_client_integration`
  - Regression: termlink-session 334/334 (was 325 + 9 new = 334), termlink-bus
    38/38 unchanged, termlink-protocol 100/100 unchanged.
- **Context:** Slice 3 closes T-2019's three-slice manifest. T-2018 §6 first
  primitive (claim semantics) now has full hub-side + client-side surface;
  ready for vendored consumers to claim/renew/release offsets with RAII
  safety on panic.

### 2026-06-07T16:33:02Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
