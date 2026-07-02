---
id: T-2309
name: "WS-S3b live CLI push consumer — channel subscribe --push with WS-connect + degrade-to-poll"
description: >
  WS-S3b live CLI push consumer — channel subscribe --push with WS-connect + degrade-to-poll

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs, crates/termlink-session/src/client.rs, crates/termlink-session/src/lib.rs]
related_tasks: [T-2305, T-2306, T-2307, T-2308, T-2303]
arc_id: push-transport            # arc-004 — WS live-transport build arc (GO output of T-2303); S3b consumer
created: 2026-07-02T17:04:22Z
last_update: 2026-07-02T17:27:25Z
date_finished: 2026-07-02T17:27:25Z
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

# T-2309: WS-S3b live CLI push consumer — channel subscribe --push with WS-connect + degrade-to-poll

## Context

Consumer slice (S3b) of arc-004 `push-transport`, carved out of S3 (T-2307). S1–S4 built and
verified the **hub side** of the live WS transport (upgrade endpoint, broadcast→client push,
per-topic `hub.ws_subscribe` filter, durable-offset-through-push). S3b builds the **client
half** so a live agent can actually consume the push stream: `termlink channel subscribe
<topic> --push` opens a WebSocket to a remote hub, authenticates, subscribes, and prints
pushed `hub.event` frames the instant they arrive — replacing the 1s poll floor — then
**degrades to the existing poll loop** if the socket fails or drops (arc invariant IW-5: the
durable substrate stays authoritative; WS is a faster transport, never a new source of truth).

**Design (from the S3b scouting report):** the WS logic lives in a new
`termlink-session::ws_consumer` module (the session crate already has `tokio-rustls`/`rustls`;
add `tokio-tungstenite` + `futures-util` there) so tungstenite stays out of the CLI crate.
The helper connects TCP+TLS (reusing `build_tls_connector`/`build_tofu_connector` to get an
**unsplit** `TlsStream`), `client_async` upgrades over the already-terminated TLS, sends
`hub.auth` then `hub.ws_subscribe`, and forwards each `hub.event` `params` into a
`tokio::sync::mpsc` channel until the socket closes/errors. The CLI (`cmd_channel_subscribe`)
renders from that channel using the same envelope-print path as the poll loop, and on helper
return (WS ended) prints a one-line degrade notice and falls into the existing `--follow`
poll loop at `channel.rs:8407`.

**S3b v1 scope (this task):** the **remote** (`host:port` + token) push path — exactly what the
hub-side S1–S4 tests exercise (TCP + `hub.auth` token + `hub.ws_subscribe`). Token/secret/addr
resolution reuses the existing `connect_remote_hub` auth logic (`remote.rs`). **Out of scope /
follow-on:** local-Unix-socket push (agents co-located with their hub — a separate target
path); active reconnect-to-WS with backoff (v1 degrades to poll and stays there rather than
re-establishing the socket — poll IS the safe fallback).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `termlink-session` gains a `ws_consumer` module with a helper that, given a `TransportAddr` (TCP), an auth token, and a topic list, connects TCP+TLS (unsplit `TlsStream` via new `Client::connect_tls_stream`), upgrades via `tokio_tungstenite::client_async`, sends `hub.auth` + `hub.ws_subscribe`, and forwards each `hub.event` `params` value into an `mpsc::Sender`. Returns cleanly (Ok/Err) when the socket closes or errors. `cargo build -p termlink-session` succeeds; 4 unit tests (`ws_auth_request_shape`, `ws_subscribe_request_shape`, `ws_hub_event_mapping`, `ws_auth_ack_ok_detection`) cover frame construction + the `hub.event`→forward mapping. **All 4 pass.**
- [x] `channel subscribe` gains a `--push` flag (clap, `cli.rs`) threaded through the dispatch (`main.rs`, both call sites) into `cmd_channel_subscribe`. `termlink channel subscribe --help` lists `--push` (verified). `cargo build --release -p termlink` succeeds.
- [x] With `--push` against a remote hub target, the handler mints a token via the existing remote-auth path (`mint_tcp_hub_token` reusing `resolve_hub_secret_hex` + `auth::create_token`, same logic as `rpc_call_authed`) and streams pushed events live from the WS helper's channel, rendering each readably — `[push] <topic> seq=<n>: <payload>` in human mode, the raw `params` JSON line under `--json`. If the WS path errors at any stage (unix target, connect, TLS, handshake, auth) OR the stream ends, it prints a one-line stderr degrade notice (`[push] WS … — degrading to poll`) and falls through to the existing poll loop — never a hard failure (degrade-to-poll invariant).
- [x] End-to-end smoke against a live hub (isolated test hub on `127.0.0.1:9199`, fresh binary): `channel subscribe inbox.queued --push --follow` received the doorbell **the instant** a `channel post inbox:smoke` landed — `[push] inbox.queued seq=0: {"channel":"inbox:smoke","message_offset":0,...}` — no poll tick. Evidence in Updates below. *(Smoke uses `inbox.queued`, not an arbitrary topic — see Evolution: the WS carries aggregator-injected events, and only `inbox:*` posts inject one. This is the arc's live-DM-doorbell path, exactly what S4 verified.)*
- [x] No regression: `cargo test -p termlink-session` green (WS additions isolated); the release CLI builds; the poll path (`--follow` without `--push`) is unchanged (the `--push` branch runs *before* the untouched poll loop and only when the flag is set).

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
cargo build -p termlink-session 2>&1 | tail -3
cargo test -p termlink-session ws_ 2>&1 | tail -15
cargo build --release -p termlink 2>&1 | tail -3

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

### 2026-07-02 — `--push` carries aggregator events, not arbitrary topic posts
- **What changed:** The first smoke (subscribe `--push smoke:s3b`, post to `smoke:s3b`) received **nothing** — and that is *correct*. The WS push stream (S1–S4) delivers `AggregatedEvent`s from the hub's process-global aggregator. A plain `channel.post` to an arbitrary topic does **not** inject an aggregator event; only an `inbox:*` post injects one (`inbox.queued`, `channel.rs:753`, the T-1637 forward path), plus session-forwarded events via the aggregator's long-poll `add_session` loop. So `channel subscribe <arbitrary-topic> --push` connects and idles silently.
- **Plan impact:** The live-agent value of `--push` is the **DM doorbell**: subscribe to `inbox.queued` (or session-event topics) and get instant wake with the durable `message_offset` — precisely the seam S4 verified, now proven live through the real CLI. The second smoke against `inbox.queued` succeeded instantly. Documented this in the AC + `--push` help; a future refinement could make `--push` on a non-injected topic emit a one-line "note: this topic does not produce push events" hint (would require the hub to advertise which topics inject — out of scope for v1).
- **Triggered:** No new sub-task. Confirmed the two carved-out follow-ons remain: **WS-over-Unix** (co-located agent — `connect_tls_stream` rejects Unix with a clear error, `run_ws_push` degrades to poll with a hint) and **active reconnect-to-WS with backoff** (v1 degrades to poll and stays; poll is the safe fallback). arc-004's build is now complete end-to-end: hub side (S1–S4) + live CLI consumer (S3b), verified live.

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

### 2026-07-02T17:04:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2309-ws-s3b-live-cli-push-consumer--channel-s.md
- **Context:** Initial task creation

### 2026-07-02 — live end-to-end smoke (AC4)
- **Setup:** isolated test hub (fresh release binary) on `127.0.0.1:9199`,
  `TERMLINK_RUNTIME_DIR=/tmp/tl-s3b-smoke`; temp `s3b-smoke` hubs.toml profile
  (removed after). Shared `:9100` hub untouched.
- **Consumer:** `termlink channel subscribe inbox.queued --hub 127.0.0.1:9199 --push --follow`
- **Trigger:** `termlink channel post inbox:smoke --hub 127.0.0.1:9199 --msg-type note --payload "wake-up-live-s3b"` → `Posted … offset=0`
- **Observed (consumer stdout, sub-second, no poll tick):**
  `[push] inbox.queued seq=0: {"addressee_session_id":"smoke","channel":"inbox:smoke","enqueued_at":1783013117803,"message_offset":0,"schema_version":"1.0"}`
- **Proves:** the full live path through the real `termlink` binary — TCP→TLS→WS
  handshake→`hub.auth`→`hub.ws_subscribe`→pushed `hub.event` render — delivers a
  DM doorbell carrying the durable `message_offset` (the S4-verified pointer)
  instantly to a WebSocket-connected CLI. Cleanup verified (hub down, profile
  removed, runtime dir cleaned).

### 2026-07-02T17:27:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
