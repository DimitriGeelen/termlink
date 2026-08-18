---
id: T-2515
name: "Bound TLS handshake in hub TCP accept path — slot-leak DoS twin of T-2442"
description: >
  Bound TLS handshake in hub TCP accept path — slot-leak DoS twin of T-2442

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-hub/src/server.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-03T18:29:13Z
last_update: '2026-08-18T18:59:12Z'
date_finished: 2026-08-03T18:34:38Z
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
  - ts: '2026-08-18T18:56:49Z'
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
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:12Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2515: Bound TLS handshake in hub TCP accept path — slot-leak DoS twin of T-2442

## Context

Firing #32 of the T-2468 purpose-review campaign (correctness/resource lens). The hub's
TCP accept loop (`crates/termlink-hub/src/server.rs`) acquires a `ConnGovernor` slot
(`ConnSlotGuard`, T-2460) at line 795 — released only when the spawned per-connection
task RETURNS — and then, at line 807, awaits `tls.accept(tcp_stream)` with **no timeout**.
`tokio-rustls` imposes no accept deadline. A peer that completes the TCP handshake but
stalls the TLS ClientHello therefore pins the task (and its governor slot) forever. After
`TERMLINK_MAX_CONNECTIONS` (default 256) such stalls, every NEW legitimate TCP connection
is refused with `HUB_AT_CAPACITY` at line 776 — the hub goes dark to new fleet peers while
spending zero CPU. This is a slot-leak DoS on the network-facing fleet port (default 9100)
that serves all four charter verbs (discover / durable-message exchange / claim / exec).

This is the **pre-handshake twin of T-2442**: T-2442 (line 884 doc-comment) explicitly names
"A silent TCP client that completes TLS but never speaks" holding "a `ConnGovernor` slot
forever" and guards it with `conn_handshake_timeout()` — but that guard is a *first-byte read*
timeout at line 979, INSIDE `handle_connection`, which only runs AFTER `tls.accept()` returns.
The strictly-earlier "completes TCP but never finishes TLS" window is the unguarded twin.
The Unix-socket path (line 740) and the raw-TCP-no-TLS test branch (line 824) both reach the
first-byte guard directly, so only the TLS accept branch leaks. Availability-class (no durable
state loss) but defeats the exact DoS guard T-2442 exists to close — in-authority by that
feature's own stated intent, and squarely the "incomplete core — deepen the guard" the T-2468
verdict points at. Zero breadth added.

### Ready-to-apply fix (turnkey)

In the `if let Some(tls) = acceptor.as_ref()` arm of the spawned task (server.rs ~806),
replace the un-timed `match tls.accept(tcp_stream).await { ... }` with:

```rust
if let Some(tls) = acceptor.as_ref() {
    // T-2515: bound the TLS handshake itself. `tls.accept().await` has no
    // built-in deadline — a peer that completes TCP but stalls the TLS
    // ClientHello would pin this task (and its ConnGovernor slot) forever,
    // exhausting the cap and DoS-ing new fleet connections. Pre-handshake
    // twin of T-2442's first-byte timeout (which is inside handle_connection,
    // AFTER accept returns).
    let hs_timeout = conn_handshake_timeout();
    match tokio::time::timeout(hs_timeout, tls.accept(tcp_stream)).await {
        Ok(Ok(tls_stream)) => {
            handle_connection(tls_stream, None, (*secret).clone(), None, Some(peer_addr_str)).await;
        }
        Ok(Err(e)) => {
            tracing::warn!(%peer_addr, error = %e, "Hub: TLS handshake failed");
        }
        Err(_) => {
            tracing::warn!(%peer_addr, timeout_ms = hs_timeout.as_millis(),
                "Hub: TLS handshake timed out — slot reclaimed (T-2515 conn-cap DoS guard)");
        }
    }
} else { /* unchanged raw-TCP test branch */ }
```

On timeout the `tls.accept()` future is dropped (dropping `tcp_stream` → socket closed) and the
task returns → `ConnSlotGuard::Drop` reclaims the slot. Test mirrors
`conn_handshake_timeout_closes_silent_connection` but WITH a real TLS acceptor
(`crate::tls::load_or_generate_cert()` into a temp `TERMLINK_RUNTIME_DIR`) and a silent client.

## Acceptance Criteria

### Agent
- [x] AC1: `tls.accept(tcp_stream)` in the accept-loop spawned task (server.rs ~807) is wrapped in `tokio::time::timeout(conn_handshake_timeout(), ...)` — a stalled TLS handshake can no longer pin the task or its `ConnSlotGuard` indefinitely.
- [x] AC2: The timeout arm emits a `tracing::warn!` carrying the unique marker `T-2515 conn-cap DoS guard`; on timeout the `tcp_stream` is dropped and the governor slot is reclaimed on task return (via `ConnSlotGuard::Drop`).
- [x] AC3: Regression test `tls_handshake_timeout_reclaims_slot` — TLS acceptor configured + a silent TCP client — asserts the hub closes the connection within the handshake timeout (EOF/reset within 2s). Proven load-bearing: temp-revert to the un-timed `tls.accept()` makes the test hang to its 2s read-timeout and FAIL.
- [x] AC4: `cargo build --release -p termlink-hub` clean AND the new test passes (`cargo test -p termlink-hub --lib tls_handshake_timeout_reclaims_slot`).

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
grep -q "T-2515 conn-cap DoS guard" crates/termlink-hub/src/server.rs
cargo build --release -p termlink-hub > /tmp/t2515-build.log 2>&1 && tail -3 /tmp/t2515-build.log
cargo test -p termlink-hub --lib tls_handshake_timeout_reclaims_slot > /tmp/t2515-test.log 2>&1 && grep -q "test result: ok" /tmp/t2515-test.log

## RCA

**Symptom:** A remote peer that completes the TCP handshake to the hub's fleet-facing port
(default 9100) but stalls the TLS ClientHello holds a `ConnGovernor` connection slot
indefinitely. Enough such stalls (default cap 256) exhaust the cap; every new legitimate
fleet connection is then refused with `HUB_AT_CAPACITY` — the hub goes dark to new peers
while burning zero CPU (a slow-loris / half-open availability DoS).

**Root cause:** `server.rs:807` awaits `tls.accept(tcp_stream)` with no deadline.
`tokio-rustls` imposes no accept timeout and OS TCP keepalive defaults to hours/off, so the
handshake future can await forever. The `ConnSlotGuard` (T-2460) is released only when the
spawned task returns — which never happens while the accept future is pending.

**Why structurally allowed:** T-2442 introduced `conn_handshake_timeout()` specifically to
close this DoS class, but placed the guard as a *first-byte read* timeout INSIDE
`handle_connection` (line 979) — which only executes AFTER `tls.accept()` returns. The
strictly-earlier pre-TLS-handshake window was left unguarded. The T-2442 regression test
runs with `None` TLS acceptor (raw-TCP branch), so it exercises the first-byte guard and
never touches the `tls.accept()` branch — the test coverage matched the fix's placement, not
the full attack surface, so the gap stayed invisible.

**Prevention:** New regression test `tls_handshake_timeout_reclaims_slot` runs the accept loop
WITH a real TLS acceptor and a silent client, asserting the connection is closed within the
handshake timeout — it exercises exactly the previously-uncovered `tls.accept()` branch and
fails if the timeout wrap is removed. PL captured: a liveness/DoS guard must bound EVERY await
between slot-acquire and slot-release, and its regression test must drive the same transport
mode (TLS vs raw) as the guarded path.

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

### 2026-08-03T18:29:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2515-bound-tls-handshake-in-hub-tcp-accept-pa.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-b7e14cfa
- **Timestamp:** 2026-08-03T18:35:06Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-03T18:34:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
