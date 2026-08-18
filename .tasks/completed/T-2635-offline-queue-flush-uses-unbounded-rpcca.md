---
id: T-2635
name: "Offline-queue flush uses unbounded rpc_call_addr — wedges forever on black-hole
  hub (adopt call_with_timeout T-2354)"
description: >
  BusClient::flush + post call the unbounded rpc_call_addr (client.rs:273); call_with_timeout
  (T-2354) exists but flush never adopted it. A half-open/wedged hub blocks the detached
  flush task forever; queue never drains. Route flush/post through a bounded rpc_call_addr_with_timeout.

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/channel.rs, 
      crates/termlink-session/src/bus_client.rs, 
      crates/termlink-session/src/client.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-12T10:52:13Z
last_update: '2026-08-18T18:59:14Z'
date_finished: 2026-08-12T14:27:01Z
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
  - ts: '2026-08-18T18:56:54Z'
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
  - ts: '2026-08-18T18:59:14Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2635: Offline-queue flush uses unbounded rpc_call_addr — wedges forever on black-hole hub (adopt call_with_timeout T-2354)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] A bounded `rpc_call_addr_with_timeout(addr, method, params, timeout)` exists in `termlink-session/src/client.rs`, routing through the existing `Client::call_with_timeout` (T-2354) primitive rather than the unbounded `call`. **Done in T-2641/T-2639** (bounds connect via `connect_addr_with_timeout` + read via `call_with_timeout`); this task adopts it.
- [x] `BusClient::post` and `BusClient::flush` call the bounded variant with a sane default read timeout (tunable via env to match the T-2354 convention); a wedged/half-open hub causes the flush to error+retry, never to block indefinitely. **Done:** both sites route through `rpc_call_addr_with_timeout(&self.addr, CHANNEL_POST, params, flush_read_timeout())`; `flush_read_timeout()` shares the `TERMLINK_RPC_READ_TIMEOUT_SECS` env (default 30s, clamped 1..=600).
- [x] A test with a black-hole server (accepts the connection but never writes a response line) proves the flush path returns a timeout error within the bound instead of hanging. **Done:** `bus_client::tests::post_and_flush_are_bounded_on_black_hole_hub` (repeated-accept black-hole unix listener; post queues + flush reports `failed` within a 1s bound). Load-bearing: reverting both sites to unbounded `rpc_call_addr` hangs → outer 6s guard trips at 6.11s.
- [x] The detached flush task's shutdown oneshot can interrupt a stuck flush (the `select!` is no longer starved by an unbounded await). **Done:** the detached-task loop now races `c.flush()` against `shutdown_rx` in an inner `select!`, so `shutdown()` interrupts a wedged flush promptly. Test `shutdown_interrupts_a_stuck_flush` (default 30s bound + black-hole; `shutdown()` joins the task within 3s). Load-bearing: reverting to plain `c.flush().await` blocks 30s → 3s join assertion fails at 3.42s.
- [x] `cargo test -p termlink-session` green (427 passed); `cargo build` succeeds (full workspace).

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

cargo test -p termlink-session --lib bus_client::tests
cargo build -p termlink-session

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

**Symptom:** When a hub host completes the TCP+TLS handshake but the hub process
is wedged/half-open (accepts but never writes a response line), the offline-queue's
detached flush task blocks forever inside `c.flush().await`; the durable queue
never drains and the flush task is uninterruptible until the OS/TLS layer errors.

**Root cause:** `BusClient::post`/`flush` call `rpc_call_addr` (client.rs:273),
which uses the unbounded `Client::call` — `call` awaits `reader.next_line()` with
no timeout. The codebase already has the bounded fix `Client::call_with_timeout`
(client.rs:210, T-2354) whose own doc comment names this exact hazard ("blocks
forever if the hub accepts the request but never writes a response line ...
observed in the field"), but the offline-queue flush was never migrated onto it.

**Why structurally allowed:** The bounded primitive was added for one caller
(T-2354) without an audit of the other unbounded `rpc_call_addr` callers; no test
exercised the flush path against a silent/black-hole server, so the hang was
invisible. The resilience mechanism (durable queue for surviving hub outages)
defeats itself precisely during the outage it exists to survive.

**Prevention:** Route all queue RPC through a bounded variant + a black-hole-server
regression test (the T-2354 test `call_with_timeout_errors_on_silent_server`
already proves the primitive; this extends it to the flush caller). Consider a
follow-up audit AC: no `rpc_call_addr` (unbounded) caller remains on a
detached/background path.

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

### 2026-08-12T10:52:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2635-offline-queue-flush-uses-unbounded-rpcca.md
- **Context:** Initial task creation

### 2026-08-12T12:14:59Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-595dc702
- **Timestamp:** 2026-08-12T14:27:06Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Agent)** — A bounded `rpc_call_addr_with_timeout(addr, method, params, timeout)` exists in `termlink-session/src/client.rs`, routing through the existing `Client::call_with_timeout` (T-2354) primitive rather tha
  - **AC-verify-mismatch** (narrow, heuristic) — `path=termlink-session/src/client.rs in: A bounded `rpc_call_addr_with_timeout(addr, method, params, timeout)` exists in `termlink-session/src/client.rs`, routing through the existing `Client`

### 2026-08-12T14:27:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
