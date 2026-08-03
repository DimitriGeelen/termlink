---
id: T-2516
name: "Bound WS upgrade handshake (accept_async) in hub — slot-leak DoS sibling of T-2515"
description: >
  Bound WS upgrade handshake (accept_async) in hub — slot-leak DoS sibling of T-2515

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-03T18:40:48Z
last_update: 2026-08-03T18:40:48Z
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

# T-2516: Bound WS upgrade handshake (accept_async) in hub — slot-leak DoS sibling of T-2515

## Context

Firing #33 of the T-2468 purpose-review campaign — a direct sibling of T-2515 found by hunting
OTHER instances of the same slot-leak class. The hub's WS upgrade handshake
`tokio_tungstenite::accept_async(stream).await` (`crates/termlink-hub/src/server.rs:1067`, inside
`handle_ws_connection`) runs inside the spawned per-connection task that holds the `ConnGovernor`
slot (`ConnSlotGuard`, T-2460) — with **no timeout**. The T-2442 first-byte guard
(`server.rs:~994`, inside `handle_connection`) reads exactly ONE byte into a `[0u8;1]` buffer;
when that byte is `'G'` the path routes to `handle_ws_connection`, where `accept_async` reads the
REST of the HTTP upgrade request (until the `\r\n\r\n` header terminator) off the wire with no
deadline. A peer that completes TCP+TLS, sends the single sniff byte `'G'`, then withholds the
remaining request bytes hangs `accept_async` forever → the task never returns → `ConnSlotGuard::Drop`
never runs → the slot leaks. After `TERMLINK_MAX_CONNECTIONS` (default 256) such stalls, every new
fleet connection is refused with `HUB_AT_CAPACITY` (-32019). Pre-auth (the upgrade precedes
`hub.auth`), reachable from any peer that can open the hub's TCP/TLS listener.

This is the SAME DoS class as T-2515, just one await downstream: T-2515 bounded `tls.accept()`,
T-2442 bounds the first-byte read and the post-upgrade WS idle timer (`server.rs:1103`, which only
arms AFTER `accept_async` returns) — but the WS handshake read between them was left unguarded.
Availability-class; defeats the same conn-cap DoS guard. Fix mirrors T-2515 exactly. Zero breadth added.

### Ready-to-apply fix (turnkey)

In `handle_ws_connection` (server.rs ~1067) wrap `accept_async` in the shared handshake deadline:

```rust
let ws = match tokio::time::timeout(
    conn_handshake_timeout(),
    tokio_tungstenite::accept_async(stream),
)
.await
{
    Ok(Ok(ws)) => ws,
    Ok(Err(e)) => {
        tracing::debug!(error = %e, "Hub: WebSocket handshake failed");
        return;
    }
    Err(_) => {
        tracing::warn!(peer = ?peer_addr, "Hub: WebSocket handshake timed out — slot reclaimed (T-2516 conn-cap DoS guard)");
        return;
    }
};
```

On timeout the `accept_async` future drops (socket closed) and the task returns →
`ConnSlotGuard::Drop` reclaims the slot. `conn_handshake_timeout()` is already in scope (same module).

Test: raw-TCP (None acceptor) accept loop + a client that sends exactly the byte `'G'` then withholds
the rest of the HTTP upgrade; assert the hub closes within the handshake timeout (EOF/reset within 2s).

## Acceptance Criteria

### Agent
- [ ] AC1: `tokio_tungstenite::accept_async(stream)` in `handle_ws_connection` (server.rs ~1067) is wrapped in `tokio::time::timeout(conn_handshake_timeout(), ...)` — a stalled WS upgrade can no longer pin the task or its `ConnSlotGuard`.
- [ ] AC2: The timeout arm emits a `tracing::warn!` carrying the unique marker `T-2516 conn-cap DoS guard`; on timeout the stream is dropped and the governor slot is reclaimed on task return.
- [ ] AC3: Regression test `ws_handshake_timeout_reclaims_slot` — raw-TCP accept loop + a client that sends only `'G'` then withholds the rest of the upgrade — asserts the hub closes the connection within the handshake timeout (EOF/reset within 2s). Proven load-bearing: temp-revert to the un-timed `accept_async` makes it hang to its 2s read-timeout and FAIL.
- [ ] AC4: `cargo build --release -p termlink-hub` clean AND the new test passes (`cargo test -p termlink-hub --lib ws_handshake_timeout_reclaims_slot`).

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
grep -q "T-2516 conn-cap DoS guard" crates/termlink-hub/src/server.rs
cargo build --release -p termlink-hub > /tmp/t2516-build.log 2>&1 && tail -3 /tmp/t2516-build.log
cargo test -p termlink-hub --lib ws_handshake_timeout_reclaims_slot > /tmp/t2516-test.log 2>&1 && grep -q "test result: ok" /tmp/t2516-test.log

## RCA

**Symptom:** A remote peer that completes TCP+TLS to the hub's fleet port, sends the single WS
sniff byte `'G'`, then withholds the rest of the HTTP upgrade request holds a `ConnGovernor` slot
indefinitely. Enough such stalls (default cap 256) exhaust the cap; new fleet connections are then
refused with `HUB_AT_CAPACITY` — a slow-loris availability DoS, identical in effect to T-2515 but
via the WebSocket upgrade path.

**Root cause:** `server.rs:1067` awaits `tokio_tungstenite::accept_async(stream)` with no deadline.
`accept_async` reads the full HTTP upgrade request (until `\r\n\r\n`) off the wire; tungstenite
imposes no handshake timeout. The `ConnSlotGuard` (T-2460) is released only when the spawned task
returns, which never happens while `accept_async` awaits withheld bytes.

**Why structurally allowed:** the connection path has THREE sequential awaits between slot-acquire
and slot-release — `tls.accept()` (now guarded by T-2515), the first-byte read (guarded by T-2442),
and `accept_async` (this gap) — plus the post-upgrade WS idle timer (T-2442, arms only AFTER
`accept_async` returns). T-2442 guarded the read it introduced and the idle loop; T-2515 guarded
`tls.accept()`. The WS handshake read in between was never bounded. The T-2442 first-byte test sends
a full silent-connection scenario but never a partial-`'G'`-then-stall, so the `accept_async` branch
was uncovered.

**Prevention:** New regression test `ws_handshake_timeout_reclaims_slot` drives exactly the
partial-`'G'`-then-withhold scenario against a raw-TCP accept loop and asserts the hub closes within
the handshake timeout — it fails if the timeout wrap is removed. Reinforces the T-2515 PL: a
liveness/DoS guard must bound EVERY await between slot-acquire and slot-release; guarding one or two
of N sequential awaits leaves the rest as live leak sites. A follow-up sweep confirmed these are the
only three handshake awaits on the slot-holding path.

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

### 2026-08-03T18:40:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2516-bound-ws-upgrade-handshake-acceptasync-i.md
- **Context:** Initial task creation
