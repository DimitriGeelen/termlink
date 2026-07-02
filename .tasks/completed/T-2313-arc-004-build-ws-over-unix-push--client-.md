---
id: T-2313
name: "arc-004 build: WS-over-Unix push — client connect_ws_unix + route Unix --push (T-2312 GO)"
description: >
  arc-004 build: WS-over-Unix push — client connect_ws_unix + route Unix --push (T-2312 GO)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: ["arc:push-transport"]
components: [crates/termlink-cli/src/commands/channel.rs, scripts/demo-ws-push-unix.sh]
related_tasks: ["T-2312", "T-2309", "T-2303"]
arc_id: push-transport
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-02T19:15:30Z
last_update: 2026-07-02T19:37:51Z
date_finished: 2026-07-02T19:37:51Z
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

# T-2313: arc-004 build: WS-over-Unix push — client connect_ws_unix + route Unix --push (T-2312 GO)

## Context

Build slice for the T-2312 GO decision. The hub already supports WS-over-Unix
(the T-2305 first-byte sniff in `handle_connection` routes a Unix `GET` to the
generic `handle_ws_connection`, and Unix connections start at `Execute` scope so
`hub.ws_subscribe` needs no auth — verified in `docs/reports/T-2312-*.md`). The
only blocker is the client: `connect_tls_stream` (`termlink-session/src/client.rs`)
rejects Unix and always wraps TLS, and the CLI `--push` branch degrades immediately
on `Unsupported` for Unix hubs. This task adds a raw `client_async`-over-UDS path
so co-located agents get the same ~90 ms push as remote agents.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `termlink-session` gains a Unix WS connector (Unix arm in `stream_ws_events`) that opens the Unix socket and completes a `tokio_tungstenite::client_async` handshake over the **raw** stream (no TLS), sharing the generic `run_ws_session` loop with the TCP path. **Evidence:** `ws_consumer.rs` Unix match arm + `run_ws_session<S>`.
- [x] The Unix WS path **skips the `hub.auth` token mint** (Unix is peer-cred-trusted; `run_ws_push` mints a token only for TCP) and sends `hub.ws_subscribe` directly; the pure `ws_auth_required` predicate gates auth and is unit-tested (TCP→true, Unix→false). **Evidence:** `ws_consumer::tests::ws_auth_required_by_transport` passes.
- [x] `channel subscribe <topic> --push` against a **Unix** hub no longer degrades immediately — it uses the Unix WS path (`WsPushOutcome::Unsupported` and `WsConsumerError::UnsupportedTransport` removed). **Evidence:** live smoke below.
- [x] `cargo build --release -p termlink && cargo test -p termlink-session ws_` pass (existing WS consumer tests green; new `ws_auth_required_by_transport` included). **Evidence:** build exit 0; 10 session ws tests pass.
- [x] Live smoke: an isolated **Unix-socket** hub + a `--push inbox.queued` consumer over that Unix socket receives an `inbox.queued` push carrying the durable `message_offset` when a DM is posted to an `inbox:*` topic. **Evidence:** `scripts/demo-ws-push-unix.sh` → 42 ms post→push, frame `{"...","message_offset":0,...}`; TCP path re-verified 96 ms (no regression).

<!-- All criteria agent-verifiable; no Human section. -->

<!-- HUMAN-SECTION-REMOVED
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
HUMAN-SECTION-REMOVED -->

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

cargo build --release -p termlink
cargo test -p termlink-session ws_

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

### 2026-07-02 — build slice from T-2312 GO

- **What changed:** T-2312 inception verified the hub already supports WS-over-Unix,
  so this is a client-only change — no hub/protocol work. The build reduces to a raw
  `client_async`-over-UDS connector + routing Unix `--push` through it (skipping auth).
- **Plan impact:** none — matches the T-2312 recommended path exactly.
- **Triggered:** IW-2 (raw Unix `client_async` handshake) worked first-try — the
  hub's generic `handle_ws_connection` accepted the raw-UDS WS with no change.
  IW-3 (no token needed) confirmed — the Unix consumer subscribed and received
  pushes with no `hub.auth`, exactly as the T-2312 code-read predicted. Net: the
  build matched the inception's client-only scope with zero surprises. Also
  removed two now-dead symbols (`WsPushOutcome::Unsupported`,
  `WsConsumerError::UnsupportedTransport`) since Unix is no longer unsupported.

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

### 2026-07-02T19:15:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2313-arc-004-build-ws-over-unix-push--client-.md
- **Context:** Initial task creation

### 2026-07-02 — WS-over-Unix live smoke (AC5)
- **Command:** `bash scripts/demo-ws-push-unix.sh`
- **Output:**
  ```
  === arc-004 WS-over-Unix push demo (T-2313) ===
  hub socket:     <tmp>/hub.sock   (isolated runtime_dir)
  transport:      Unix socket, raw WS (no TLS, no token)
  topic:          inbox:demo-unix-1223403
  post->push:     42 ms
  push frame:     [push] inbox.queued seq=0: {"addressee_session_id":"demo-unix-1223403","channel":"inbox:demo-unix-1223403","enqueued_at":1783020923000,"message_offset":0,"schema_version":"1.0"}
  RESULT: PASS — push arrived over Unix sub-second (42 ms < 1000 ms)
  ```
- **TCP no-regression:** `bash scripts/demo-ws-push.sh` → 96 ms, degrade-to-poll intact.

### 2026-07-02T19:37:51Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
