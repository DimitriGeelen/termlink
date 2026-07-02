---
id: T-2305
name: "V1 WS-S1 hub WebSocket upgrade endpoint — dep + Upgrade handshake + auth reuse"
description: >
  V1 WS-S1 hub WebSocket upgrade endpoint — dep + Upgrade handshake + auth reuse

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-hub/src/server.rs]
related_tasks: []
arc_id: push-transport            # arc-004 — WS live-transport build arc (GO output of T-2303)
created: 2026-07-02T15:47:34Z
last_update: 2026-07-02T15:59:43Z
date_finished: 2026-07-02T15:59:43Z
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

# T-2305: V1 WS-S1 hub WebSocket upgrade endpoint — dep + Upgrade handshake + auth reuse

## Context

First build slice (S1) of arc-004 `push-transport`, the GO(scoped) output of the
T-2303 inception (`docs/reports/T-2303-push-transport-inception.md` §10). Goal of the
whole arc: replace the 15s doorbell-then-poll wake/read floor with a hub→client
WebSocket push stream, degrading to polling if the socket drops, durability layer
(dm: topics / receipts / journal / offline queue) unchanged.

**S1 scope (this task):** stand up the hub-side WebSocket *upgrade endpoint only* —
add the WS dependency, detect + complete an RFC6455 `Upgrade` handshake on the hub's
existing accept path, and authenticate the upgraded socket by **reusing** the existing
once-per-connection HMAC scope verification (server.rs:508) rather than forking a new
auth scheme. The connection is accepted, authenticated, and held open. **Out of scope:**
draining the broadcast channel to the client (that is S2), client-side WS subscribe (S3),
receipts/journal through WS (S4). Feasibility was assessed LOW–MEDIUM in the inception
(hub is pure tokio + generic `handle_connection`; broadcast fan-out already exists at
aggregator.rs:60). Additive + reversible: a plain JSON-RPC line connection must still
work unchanged (back-compat).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] A WebSocket dependency (`tokio-tungstenite`, TLS-feature-free since the WS runs over the already-TLS-terminated stream) is added to the hub crate's `Cargo.toml`, and `cargo build --release -p termlink-hub` succeeds. *(server.rs / Cargo.toml — release build green.)*
- [x] The hub accept path detects an inbound HTTP WebSocket upgrade (first-byte sniff `G` of `GET ` vs `{` of a JSON-RPC line, replayed via `PeekedStream` so no byte is lost) and completes the RFC6455 handshake via `tokio_tungstenite::accept_async`, yielding a WS stream over the same generic `AsyncRead+AsyncWrite` connection type used today. *(Test drives a real `client_async` upgrade to completion.)*
- [x] Back-compat: a connection that speaks the existing newline-delimited JSON-RPC line protocol (first byte `{`) is routed to `handle_line_connection` — the extracted-verbatim old body — so the plain path is unchanged. Proven by all 373 hub unit tests + 4 integration tests still passing.
- [x] The upgraded WS connection is authenticated by **reusing** the existing HMAC scope-verification path: `hub.auth` flows through the *same* `process_request_message` dispatch as the line transport, so an invalid/absent token yields the identical refusal (unauthenticated calls → `AUTH_REQUIRED -32009`; a bad `hub.auth` does not authenticate and does not drop the socket), and a valid token caches the authenticated scope (`granted_scope`) for the connection lifetime — no new/forked auth scheme.
- [x] A unit test (`server::tests::ws_upgrade_auth_and_reuse`) exercises both outcomes over a real WS client: (a) valid token authenticates and the connection stays open for a subsequent authed call that returns a result; (b) unauthenticated call rejected with `AUTH_REQUIRED` and an invalid token fails to authenticate. Passes in CI.

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
cargo test -p termlink-hub ws_upgrade 2>&1 | tail -8

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

### 2026-07-02 — S1 built: one dispatch path, first-byte sniff, TLS-free WS dep
- **What changed:** Rather than duplicate the ~130-line JSON-RPC dispatch loop for
  WS, the old `handle_connection` body was extracted into `process_request_message`
  (returns `Option<String>`), and `handle_connection` now sniffs the first byte and
  routes to `handle_line_connection` or `handle_ws_connection` — both call the one
  shared dispatch. This makes auth-reuse *automatic*: `hub.auth` over WS runs the
  exact same code as over a line, so there is no second auth path to drift.
- **Plan impact:** Confirmed the inception's LOW–MEDIUM feasibility. Two refinements
  over the filed plan: (1) WS runs over the *already-TLS-terminated* stream, so
  `tokio-tungstenite` is taken with `default-features=false, features=["handshake"]`
  — no second rustls pin, sidestepping the version-match risk the AC anticipated;
  (2) auth is a post-handshake `hub.auth` RPC (faithful reuse), NOT a handshake-time
  credential — so "invalid HMAC refused" happens at the RPC layer identically to the
  line path, and the socket stays open (AC wording corrected to match).
- **Triggered:** `PeekedStream<S>` (replays the sniffed byte) is a new reusable
  primitive S2 will build on. S2 (broadcast→client push) plugs a concurrent write
  loop into `handle_ws_connection` alongside the existing read side — the read/write
  split is already there. No scope cuts; arc plan S1→S4 unchanged.

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

### 2026-07-02T15:47:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2305-v1-ws-s1-hub-websocket-upgrade-endpoint-.md
- **Context:** Initial task creation

### 2026-07-02T15:59:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
