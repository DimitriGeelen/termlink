---
id: T-2528
name: "Line-protocol read loop has no idle timeout — pre-auth slowloris pins governor
  slot forever (WS twin of T-2442 missed)"
description: >
  handle_line_connection reads request lines with no tokio::time::timeout; a peer
  that sends one byte then stalls blocks in fill_buf().await forever, holding its
  ConnSlotGuard; cap-many such idle line connections deny the whole hub. WS path already
  has ws_idle_timeout (T-2442); line path is the missed twin.

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
created: 2026-08-04T12:53:20Z
last_update: '2026-08-18T18:59:12Z'
date_finished: 2026-08-04T13:00:15Z
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
  - ts: '2026-08-18T18:56:50Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 1
      D3: 0
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=1 (body:log-or-error-line); D3=0 
      (no-signal); D4=2 (body:env-class-handled); F-RECALL=0 (no-signal); 
      F-ORCH=0 (no-signal)
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

# T-2528: Line-protocol read loop has no idle timeout — pre-auth slowloris pins governor slot forever (WS twin of T-2442 missed)

## Context

**Reliability defect on the CORE "exchange durable messages" transport (T-2468 incomplete-core lens).**
The hub's legacy line-protocol per-connection handler `handle_line_connection`
(`crates/termlink-hub/src/server.rs:1080`) runs `loop { read_capped_line(...).await }`
(line 1095) with **no `tokio::time::timeout`**. `read_capped_line` blocks in
`reader.fill_buf().await` (line 1050) with no deadline. A remote peer that completes
TLS, sends the single sniff byte `{` at ~t+29s (satisfying the T-2442 first-byte
handshake guard at line 994 and the `{`/`G` transport sniff), then sends nothing more,
parks this task in `fill_buf` **forever**. The task holds its `ConnSlotGuard` (line 951,
RAII, released only on task return — T-2460) for its entire lifetime, so the governor
connection slot is never reclaimed.

**Whole-hub DoS.** `TERMLINK_MAX_CONNECTIONS` (default 256) such idle line connections
exhaust the `ConnGovernor` cap → every further peer is refused `HUB_AT_CAPACITY`
(-32019) and **no timeout ever reclaims a slot**. This is precisely the
"connection-cap-without-a-per-connection-read-timeout" liability: the cap without an
idle timeout is not a defense. Pre-auth reachable (accept spawns with
`initial_scope=None`), so no rate-limit applies yet.

**The WS twin was hardened; the line path was missed.** The WebSocket read loop
(lines 1206-1226) already drops a connection when `last_activity.elapsed() >=
ws_idle_timeout()` (120s, T-2442 keepalive+idle), and the WS *handshake* is bounded by
`conn_handshake_timeout()` (T-2516). The equivalent inbound-idle bound for the
line read loop was never added — a genuine asymmetry, not a design choice.

**Long-poll safety (the critical check).** `read_capped_line` reads a full request
*line*; a line-path `event.subscribe`/channel long-poll blocks later, *inside*
`process_request_message` (line 1122) via the `rx.recv()` timeouts in channel.rs /
router.rs — AFTER the line is fully read. So bounding the read at line 1095 bounds only
the *between-requests / initial-request* gap and does **not** truncate long-polls (a
legit client sends its request bytes immediately). This mirrors why the WS inbound-idle
timeout is safe there.

**Fix (small, in-authority — a resource guard, no threat-model/semantics decision):**
wrap the `read_capped_line` call in `tokio::time::timeout(line_idle_timeout(), ...)`;
on elapse log a warn and `break` (drops the guard → reclaims the slot). Add a
`line_idle_timeout()` helper reading `TERMLINK_LINE_IDLE_TIMEOUT_MS` (default 120_000,
symmetric with `ws_idle_timeout`; clamped via the existing `parse_env_u64_clamped`
convention). Mirrors the existing T-2442/T-2516 pattern verbatim — the missed twin.

Origin: T-2468 subtract-and-deepen campaign, adversarial reliability-hunter firing
(no-timeout blocking-read lens). Predecessors: T-2442 (WS keepalive+idle),
T-2515 (TLS handshake timeout), T-2516 (WS upgrade timeout), T-2518 (line-length cap),
T-2460 (ConnSlotGuard RAII).

## Acceptance Criteria

### Agent
- [x] `handle_line_connection`'s `read_capped_line` call (server.rs:1095) is wrapped in `tokio::time::timeout(line_idle_timeout(), ...)`; on elapse it logs a warn and `break`s (dropping the `ConnSlotGuard` so the governor slot is reclaimed)
- [x] A `line_idle_timeout()` helper is added reading `TERMLINK_LINE_IDLE_TIMEOUT_MS` (default 120_000, clamped via `parse_env_u64_clamped`), symmetric with `ws_idle_timeout()`
- [x] A load-bearing async test drives `handle_line_connection` over a duplex stream that sends `{` then stalls (never a newline), under a short `TERMLINK_LINE_IDLE_TIMEOUT_MS`, and asserts the function RETURNS (slot reclaimed) within a generous outer bound rather than hanging forever — `line_idle_timeout_reclaims_stalled_slot` passes in 0.20s
- [x] Load-bearing proof: temp-reverting the timeout wrap makes that test FAIL (the handler no longer returns — the pre-fix slowloris hang), restore → green — neutralizing the guard (idle→1h) made the test hang 5.00s then fail on its assertion; restored → ok
- [x] `cargo test -p termlink-hub` green (472 + 4 pass, 0 fail); `cargo build -p termlink-hub` clean; long-poll paths unaffected (the wrap covers only the line read, not `process_request_message`)

<!-- Human section removed — fully agent-verifiable (a Rust guard + async duplex test). -->

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
cargo build -p termlink-hub
cargo test -p termlink-hub line_idle

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

**Symptom:** A remote peer completes TLS to the hub, sends the single byte `{` (or `G`)
to pass the T-2442 first-byte guard, then sends nothing. The per-connection task parks
forever in `read_capped_line`'s `fill_buf().await` (server.rs:1050), holding its
`ConnSlotGuard`. `TERMLINK_MAX_CONNECTIONS` (default 256) such connections exhaust the
`ConnGovernor` cap; all further peers are refused `HUB_AT_CAPACITY` (-32019) and no
timeout ever reclaims a slot — a whole-hub, pre-auth denial of service.

**Root cause:** `handle_line_connection`'s read loop (server.rs:1094-1132) has no
per-read idle timeout. The read blocks indefinitely on a peer that opens the connection
but withholds bytes (or trickles < the 16 MiB line cap without a newline). The T-2518
line-*length* cap bounds memory but not *time*; the T-2442 first-byte cap bounds only
the very first byte, defeated by sending exactly one.

**Why structurally allowed:** the connection-cap governor (T-2048/T-2460) was added
without a matching per-connection read timeout on the line path. The WS path *did* get
both (T-2442 keepalive+idle at lines 1206-1226, T-2516 upgrade timeout) — but the line
path's idle-read twin was never added. An asymmetry between two transports carrying the
same dispatch: the hardening applied to one and not the other, with nothing testing that
a stalled line connection is reclaimed. A cap without a per-connection read timeout reads
as "protected" while leaving every slot indefinitely wedgeable.

**Prevention:** (1) the fix — bound the line read with `line_idle_timeout()`, reclaiming
the slot on elapse; (2) a load-bearing async test that drives `handle_line_connection`
over a stalling duplex stream and asserts it RETURNS (the guard fires) rather than hanging
— so a future regression that drops the timeout fails CI. Broader structural note: the
class is "a slot-holding await with no deadline on a pre-auth path"; the three handshake
awaits (TLS/first-byte/WS-upgrade) and the WS read loop were each hardened one-by-one
(T-2515/T-2442/T-2516) — this closes the last one (the line read loop). A grep for
slot-holding `.await` on the accept→dispatch path with no `tokio::time::timeout` wrapper
would surface the whole class (candidate future canary, sibling of T-2527's alloc-sink
check — logged as a follow-up thought, not built here to keep one-bug-one-task).

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

### 2026-08-04T12:53:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2528-line-protocol-read-loop-has-no-idle-time.md
- **Context:** Initial task creation

### 2026-08-04T12:53:34Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-08-04 — built + verified [T-2468 subtract-and-deepen campaign, reliability-hunter firing]
- **Fix:** `line_idle_timeout()` helper (`TERMLINK_LINE_IDLE_TIMEOUT_MS`, default 120_000, `parse_env_u64_clamped` [200, 3_600_000]) + wrapped the `read_capped_line` call in `handle_line_connection` (server.rs) in `tokio::time::timeout(line_idle_timeout(), ...)`; on elapse logs a warn and `break`s → drops the `ConnSlotGuard` → reclaims the governor slot. The missed twin of the WS idle timeout (T-2442).
- **Verified in code first (hunter can be wrong):** confirmed line 1095 read had no timeout, WS loop (1206-1226) did, `ConnSlotGuard` (951) is RAII released only on task return, T-2442 first-byte guard is defeated by one byte, and the long-poll wait is inside `process_request_message` (1122) AFTER the read — so bounding the read cannot truncate a long-poll.
- **Load-bearing:** `line_idle_timeout_reclaims_stalled_slot` drives `handle_line_connection` over a duplex stream that sends `{` then keeps the client half ALIVE (Pending, not EOF — an EOF would let the handler return even without the fix, a false pass) and asserts it returns within 5s. Neutralizing the guard (idle→1h) → test hangs 5.00s → FAILS on its assertion; restored → passes in 0.20s.
- **No regressions:** full `cargo test -p termlink-hub` = 472 + 4 pass, 0 fail; build clean.
- **Follow-up thought (NOT built, one-bug-one-task):** the class is "a slot-holding `.await` with no deadline on a pre-auth path"; TLS/first-byte/WS-upgrade/WS-read were each hardened one-by-one (T-2515/T-2442/T-2516) and this closes the line-read loop — a grep-canary for un-timeout'd slot-holding awaits on the accept→dispatch path would surface the whole class (sibling of T-2527's alloc-sink check).

## Reviewer Verdict (v1.5)

- **Scan ID:** R-550b4eaf
- **Timestamp:** 2026-08-04T13:00:29Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-04T13:00:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
