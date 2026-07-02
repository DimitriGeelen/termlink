---
id: T-2306
name: "V1 WS-S2 hub broadcast→client push write loop over WebSocket"
description: >
  V1 WS-S2 hub broadcast→client push write loop over WebSocket

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-hub/src/server.rs]
related_tasks: []
arc_id: push-transport            # arc-004 — WS live-transport build arc (GO output of T-2303)
created: 2026-07-02T16:00:40Z
last_update: 2026-07-02T16:08:04Z
date_finished: 2026-07-02T16:08:04Z
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

# T-2306: V1 WS-S2 hub broadcast→client push write loop over WebSocket

## Context

Second build slice (S2) of arc-004 `push-transport`. S1 (T-2305) stood up the hub-side
WebSocket upgrade + auth-reuse over a shared dispatch, but the WS path is still
request/response only (half-duplex). **S2 adds the actual server→client PUSH**: once a
WS connection is authenticated, the hub subscribes to the existing in-process
`tokio::broadcast::Sender<AggregatedEvent>` (aggregator.rs) and streams each event to the
client as a JSON-RPC notification frame — the client receives events with **no poll**.
This is the mechanism that collapses the 15s doorbell-then-poll floor to sub-second push.

**S2 scope (this task):** split the WS stream into read + write halves and run a
`tokio::select!` loop in `handle_ws_connection` so request-frames and pushed events flow
concurrently; drain the aggregator broadcast and forward events as `hub.event`
notifications, gated on the connection being authenticated. **Out of scope:** per-topic
subscription filtering + degrade-to-poll fallback (S3), receipts/journal through WS (S4).
For S2 an authenticated WS connection receives *all* broadcast events; S3 adds the
client-driven topic filter. Rides entirely on the existing durable substrate — this is a
faster transport for the wake/read, not a new source of truth.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `handle_ws_connection` splits the WebSocket into a read half and a write half and runs a `tokio::select!` loop, so the hub can send server-initiated frames concurrently with handling client request frames (no longer strictly request→response). `cargo build --release -p termlink-hub` succeeds.
- [x] After a WS connection authenticates (`granted_scope` becomes `Some`), the hub subscribes to the aggregator `broadcast::Receiver<AggregatedEvent>` and pushes each event to the client as a JSON-RPC notification frame (`{"jsonrpc":"2.0","method":"hub.event","params":<AggregatedEvent>}`) — delivered without the client sending any request. `broadcast::error::Lagged` is handled (skip + continue), not fatal.
- [x] Push is gated on auth: a WS connection that has NOT authenticated receives no `hub.event` pushes (events are drained but dropped until `granted_scope` is `Some`).
- [x] Concurrency preserved: request/response still works over the same WS connection while the push loop runs (a client can call `hub.auth` / `session.discover` and get responses interleaved with pushes). All existing hub tests (line protocol + the S1 WS upgrade test) still pass.
- [x] A unit test (`server::tests::ws_push_*`) proves push end-to-end: connect a WS client, authenticate, inject an `AggregatedEvent` into the aggregator, and assert the client receives the pushed `hub.event` frame carrying that event; also assert a pre-auth connection does not receive the pushed event. Passes in CI.

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
cargo build --release -p termlink-hub 2>&1 | tail -3
cargo test -p termlink-hub ws_ 2>&1 | tail -10

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

## Evolution

### 2026-07-02 — S2 built: split + select! push, gated on auth, dormant-when-no-aggregator
- **What changed:** `handle_ws_connection` now `.split()`s the WS into `sink`/`source`
  and runs a single `tokio::select!` loop — one task multiplexes inbound requests and
  outbound pushes, so `granted_scope` stays a plain local (no `Arc<Mutex>`). Events are
  pushed as `hub.event` JSON-RPC *notifications* (params = the `AggregatedEvent`).
- **Plan impact:** Two design calls beyond the filed plan. (1) **Subscribe before auth,
  gate at send-time on `granted_scope.is_some()`** — this closes the miss-window between
  auth completing and the first push (an event injected the instant after auth still
  lands), and pre-auth events are simply drained+dropped. (2) A `recv_event` helper that
  `std::future::pending()`s when the aggregator receiver is `None`, so the push `select!`
  arm goes *dormant* instead of busy-looping when the aggregator isn't initialized (the
  minimal `run_accept_loop` test harness) or has closed — the loop keeps serving requests.
- **Triggered:** For S2 an authed WS client receives *all* broadcast events. **S3 must add
  the client-driven per-topic subscribe filter** (so a client only gets its `dm:*` /
  `agent-presence`) **plus the degrade-to-poll fallback** when the socket drops. The
  `hub.event` notification envelope + the split/select structure are the seam S3 builds on.
  No scope cuts; arc plan S2→S3→S4 unchanged.

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

### 2026-07-02T16:00:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2306-v1-ws-s2-hub-broadcastclient-push-write-.md
- **Context:** Initial task creation

### 2026-07-02T16:08:04Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
