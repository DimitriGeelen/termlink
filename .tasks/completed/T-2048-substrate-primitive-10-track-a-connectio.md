---
id: T-2048
name: "Substrate primitive #10 Track A: connection cap + rate limit (T-2028 PARTIAL-GO Track A)"
description: >
  Implement T-2028 Track A per docs/reports/T-2028-throughput-retention-inception.md. Hub-side max_connections (default 256) + per-connection rate limit (default 1000 req/sec, token-bucket). Reject-with-retry-after on overflow. Operator-tunable via hub.toml. ~30 LOC scope. Tracks B (observability) + C (retention default tuning) follow as separate tasks.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-parallel-substrate, substrate-primitive, supporting]
components: []
related_tasks: [T-2018, T-2028]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-08T10:49:10Z
last_update: 2026-06-08T15:05:43Z
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

# T-2048: Substrate primitive #10 Track A: connection cap + rate limit (T-2028 PARTIAL-GO Track A)

## Context

Implements T-2028 PARTIAL-GO **Track B** (the inception report at
`docs/reports/T-2028-throughput-retention-inception.md` §4 Track B):
hub-side connection cap + per-sender rate limit + loud-refuse RPC error.
Track A's name on this task is a misnomer from filing — the task
matches inception Track B exactly. (Tracks A "retention audit" and C
"budget observability" remain separate.)

Three vertical slices:
1. Governor primitives in `crates/termlink-hub/src/governor.rs`
   (`ConnGovernor` + per-sender `RateGovernor` token bucket) + unit tests.
2. Wire governor into accept loop (connection-cap check before spawn)
   and per-request RPC dispatch (rate-limit check before route), plus
   protocol constants for structured refuse-with-`retry_after_ms`.
3. Observability — `hub.governor_status` RPC + CLI surfacing +
   docs page + CLAUDE.md Quick Reference row.

Defaults per task description: `max_connections=256`,
`rate_limit_per_sec=1000` per sender. Operator-tunable via
`TERMLINK_MAX_CONNECTIONS` and `TERMLINK_RATE_LIMIT_PER_SEC` env
vars (hub-side, T-1633 pattern).

Refuse semantics (IW-3 — LOUD, never silent): structured `RpcError`
with `code = -32019 HUB_AT_CAPACITY` (new) or `-32008 RATE_LIMITED`
(existing, currently unused) and `data: {retry_after_ms: u64}`.

## Acceptance Criteria

### Agent
- [x] Slice 1 — Governor module: `crates/termlink-hub/src/governor.rs`
      exists with `ConnGovernor` (current/max + `try_acquire`/`release`)
      and `RateGovernor` (per-sender token bucket: tokens, refill rate,
      last_refill_ms + `try_acquire(sender)`); both expose
      `retry_after_ms()` helper for the LOUD-refuse envelope.
- [x] Slice 1 — Unit tests: `cargo test -p termlink-hub governor`
      passes (11 tests: 4 ConnGovernor [admit-up-to-max, reject-past-max,
      release-frees-slot, release-noop-on-zero] + 7 RateGovernor [burst,
      refill, sender-isolation, zero-disables, hint-matches-refill-period,
      refill-clamps-at-capacity, evict-idle]).
- [x] Slice 2 — Protocol constant: `HUB_AT_CAPACITY: i64 = -32019`
      added to `crates/termlink-protocol/src/control.rs` `error_code`
      module with doc-comment naming `retry_after_ms` data field.
      Existing `RATE_LIMITED: i64 = -32008` reused for rate-limit path.
      Wire-value test `hub_at_capacity_const_is_stable_wire_value`
      pins the value.
- [x] Slice 2 — Accept-loop wiring: `run_accept_loop` in
      `crates/termlink-hub/src/server.rs` consults `ConnGovernor`
      before spawning `handle_connection`. On full Unix accept,
      writes one JSON-RPC error envelope via `write_capacity_refusal`
      (HUB_AT_CAPACITY + retry_after_ms) and closes — never silent
      drop. On full TCP accept, closes the raw socket before TLS
      handshake (client sees handshake-fail; server-side surfaces via
      `capacity_hits_total`).
- [x] Slice 2 — Per-RPC wiring: `handle_connection` consults
      `RateGovernor` keyed by sender identity (params.from → peer_addr
      → peer_pid string → "anonymous") before `router::route`. On
      overflow, returns structured RATE_LIMITED error with
      `retry_after_ms` + `sender` in data.
- [x] Slice 2 — Configurable defaults: `TERMLINK_MAX_CONNECTIONS`
      (default 256, `governor::DEFAULT_MAX_CONNECTIONS`) and
      `TERMLINK_RATE_LIMIT_PER_SEC` (default 1000,
      `governor::DEFAULT_RATE_LIMIT_PER_SEC`) env vars read at hub
      start in `governor::init()`. `tracing::info!` emits active
      config at startup ("Hub governors active (T-2048 — ...)").
- [x] Slice 3 — Observability RPC: `hub.governor_status` returns
      `{connections_active, connections_max, capacity_hits_total,
      rate_buckets_active, rate_hits_total, max_rate_per_sec}` and
      requires Observe scope. Registered in `route()` + `hub_method_scope`
      + `handle_hub_capabilities` methods list.
- [x] Slice 3 — MCP parity: `termlink_hub_governor_status` MCP tool
      added (subprocesses via `client::rpc_call` over local UDS), plus
      help-registry entry under "hub" category.
- [x] Slice 3 — Docs: `docs/operations/substrate-governor.md` describes
      the two governors, defaults, override env vars, refuse error
      shapes, operator probe + tighter-limit recipes, and the
      "what this does NOT do" boundary list.
- [x] Slice 3 — CLAUDE.md Quick Reference row "Hub governor status
      (BACKPRESSURE)" added between Transfer-claim and Cross-host-handoff
      rows in the substrate-primitive cluster.
- [x] Live smoke (2026-06-08 17:23Z): deployed via
      `cargo install --path crates/termlink-cli --force --offline`
      + `systemctl restart termlink-hub`; production hub logs
      "Hub governors active (T-2048 — ...) max_connections=256
      rate_limit_per_sec=1000" at startup; `hub.governor_status`
      returns full shape with expected baselines. Cap smoke
      (separate test hub at /tmp/termlink-test-T2048,
      `TERMLINK_MAX_CONNECTIONS=4`): 5th simultaneous connection
      gets `{"id":null,"error":{"code":-32019,"message":"Hub at
      capacity (retry in 1000ms)","data":{"retry_after_ms":1000}}}`
      and `capacity_hits_total` increments to 1. Rate smoke
      (`TERMLINK_RATE_LIMIT_PER_SEC=10`): burst of 15 from same
      sender admits 10 + denies 5 with `code=-32008`
      `data={"retry_after_ms":100,"sender":"burst-test-sender"}`;
      `rate_hits_total` increments to 5. Both LOUD-refuse paths
      validated; structured envelopes match the doc spec exactly.

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

cargo check --workspace
cargo test -p termlink-hub governor
test -f crates/termlink-hub/src/governor.rs
out=$(cargo run -p termlink-cli --bin termlink --offline -- --help 2>&1); echo "$out" | head -n 5 > /dev/null
out=$(grep -c 'HUB_AT_CAPACITY' crates/termlink-protocol/src/control.rs); test "$out" -ge 1
out=$(grep -c 'RATE_LIMITED' crates/termlink-protocol/src/control.rs); test "$out" -ge 1
test -f docs/operations/substrate-governor.md
grep -q 'substrate-governor\|hub.governor_status\|hub_governor_status' CLAUDE.md
grep -q 'termlink_hub_governor_status' crates/termlink-mcp/src/tools.rs

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

### 2026-06-08 — slice 1 governor primitives + 11 unit tests
- **What changed:** Filing-time "Track A" name is a misnomer — task
  exactly matches T-2028 inception **Track B**. Inception Track A
  (retention audit) is a separate AUDIT-class task, not a build.
  Filed-time "~30 LOC" estimate is also low; inception §4 Track B
  bounds at ~150 LOC for three slices, which matches what slice 1
  alone landed (~210 LOC including unit tests).
- **Plan impact:** Treating this task as Track B exactly per inception.
  Filed AC for "operator-tunable via hub.toml" relaxed to env-var
  (TERMLINK_MAX_CONNECTIONS, TERMLINK_RATE_LIMIT_PER_SEC) per T-1633
  precedent — no hub.toml exists yet, env vars are the established
  hub-side config surface.
- **Triggered:** Updated task Context section to disambiguate Track B
  scope before slice 2.

### 2026-06-08 — slice 3 observability RPC + live smoke
- **What changed:** Hub-side governor state observable via
  `hub.governor_status` (read-only Tier-A JSON-RPC, scope=Observe).
  MCP parity tool `termlink_hub_governor_status` calls the same RPC
  via local UDS. Inception §4 Track B sub-task "observability into
  hub status" deliberately scoped narrowly: a dedicated RPC instead
  of grafting governor counters onto `hub.bus_state` (which is
  T-1446's G-050 audit telemetry — different concerns, distinct
  rollups in `fleet doctor`).
- **Plan impact:** None — slice 3 went per-plan. Cap smoke + rate
  smoke both passed first try. Production hub remains on defaults
  (256/1000) and the separate test-hub instance at
  `/tmp/termlink-test-T2048` with `max=4, rate=10` validated both
  refuse paths without disrupting any production state.
- **Triggered:** Track B is now shippable. Track A (retention audit)
  and Track C (budget visibility in `channel info`) remain as
  filed-but-uncaptured follow-ups per the inception §6
  recommendation. No new task filed — they're in the inception
  artifact and will fan out when T-2027 lands.

### 2026-06-08 — slice 2 wiring + drain-counter dual-tracker
- **What changed:** Naive substitution of the per-loop
  `active_connections: Arc<AtomicU32>` with the global `ConnGovernor`
  broke `graceful_shutdown_stops_accept_loop`. Root cause: under
  cargo-test parallel harness, multiple `run_accept_loop` instances
  share the same OnceLock-installed governor; counts pollute across
  tests and the drain loop blocks waiting for "other tests'" handlers
  to exit. The global counter is correct for cap-enforcement (any
  test can refuse beyond max=256) but wrong for per-loop drain
  termination.
- **Plan impact:** Slice 2 ships with a **dual-tracker** pattern:
  `ConnGovernor` (process-wide, enforces cap on accept) +
  `active_connections` (per-loop, drives drain). Both increment on
  accept, both decrement after handler exit. Adds ~6 LOC of bookkeeping
  but keeps the per-loop lifecycle clean. Documented inline in the
  accept-loop header comment so the dual-tracker shape isn't
  mistaken for redundancy.
- **Triggered:** No new sub-task. The pattern is captured in-place
  for the next governor-like primitive (Track C observability) to
  follow.

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

### 2026-06-08T10:49:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2048-substrate-primitive-10-track-a-connectio.md
- **Context:** Initial task creation

### 2026-06-08T14:53:39Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
