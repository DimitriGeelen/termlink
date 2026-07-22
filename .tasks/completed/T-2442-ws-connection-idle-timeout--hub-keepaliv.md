---
id: T-2442
name: "WS connection idle timeout + hub keepalive ping — close unauthenticated conn-cap DoS"
description: >
  WS connection idle timeout + hub keepalive ping — close unauthenticated conn-cap DoS

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-hub/src/governor.rs, crates/termlink-hub/src/server.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-21T21:29:12Z
last_update: 2026-07-21T21:36:41Z
date_finished: 2026-07-21T21:36:41Z
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

# T-2442: WS connection idle timeout + hub keepalive ping — close unauthenticated conn-cap DoS

## Context

Round-8 adversarial review of the arc-004 WS push-transport found a HIGH
unauthenticated DoS: neither the pre-dispatch first-byte read
(`server.rs` `handle_connection`) nor the WS `select!` loop
(`handle_ws_connection`) has any read/idle timeout, and the hub never sends
its own keepalive ping. A TCP client that completes TLS then goes silent (or
half-opens with no FIN/RST) holds a `ConnGovernor` slot forever; ~256 such
sockets exhaust `DEFAULT_MAX_CONNECTIONS` and every new legit client is
refused — no auth required. Fix: bound the first-byte handshake read with a
timeout, and add a hub-initiated keepalive ping + inbound-idle timeout to the
WS loop so dead connections release their slot. Env-tunable, clamped.

## Acceptance Criteria

### Agent
- [x] `handle_connection` bounds the first-byte sniff read with a timeout (`TERMLINK_CONN_HANDSHAKE_TIMEOUT_MS`, default 30000, clamped); a silent connection is dropped and its governor slot released.
- [x] `handle_ws_connection` sends a hub-initiated `Ping` on a periodic timer (`TERMLINK_WS_PING_INTERVAL_MS`, default 30000) and drops the connection when no inbound frame has arrived within the idle window (`TERMLINK_WS_IDLE_TIMEOUT_MS`, default 120000); any inbound frame (incl. Pong) resets the idle clock.
- [x] Integration test: a raw-TCP connection that sends nothing is closed by the hub within the handshake timeout (client observes EOF).
- [x] Integration test: a WS-upgraded connection that goes silent is closed by the hub within the idle timeout.
- [x] `cargo test -p termlink-hub --lib` green (existing + new tests).

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

cargo test -p termlink-hub --lib conn_handshake
cargo test -p termlink-hub --lib ws_idle

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

**Symptom:** A remote peer can exhaust the hub's connection cap
(`DEFAULT_MAX_CONNECTIONS = 256`) with no authentication: open a TCP socket,
complete the TLS handshake, then send nothing (or half-open the link with no
FIN/RST). Each such socket holds a `ConnGovernor` slot indefinitely; once 256
are held, every new legitimate client is refused with `HUB_AT_CAPACITY`.

**Root cause:** The connection lifecycle had no liveness bound. The first-byte
protocol sniff (`handle_connection`, `stream.read(&mut first).await`) awaited
the client's first byte with no timeout, and the WS `select!` loop
(`handle_ws_connection`) awaited `source.next()` with no idle timeout while the
hub only *replied* to client pings and never *sent* its own. The governor slot
is released only when the per-connection task returns — which never happened
for a silent/half-open peer.

**Why structurally allowed:** T-2048 added the connection cap (the enforcement
primitive) but the cap only counts slots; it cannot reclaim a slot held by a
task that never completes. The arc-004 WS transport (T-2305/06/07) introduced a
long-lived read loop without pairing it with a keepalive/idle-timeout — the
review focus was correctness of the push path, not connection liveness. No test
exercised a silent-connection scenario, so the leak was invisible.

**Prevention:** Two integration tests now assert a silent connection is
actively closed by the hub — one on the raw first-byte path, one on the
WS-upgraded path — so any future regression that removes the timeout fails CI.
The timeouts are env-tunable so an operator can tighten them under attack.

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

### 2026-07-21T21:29:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2442-ws-connection-idle-timeout--hub-keepaliv.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-618eddf3
- **Timestamp:** 2026-07-21T21:36:47Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-07-21T21:36:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
