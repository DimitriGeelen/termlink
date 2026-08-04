---
id: T-2523
name: "batch_exec/batch_run max_parallel unclamped — Semaphore panic + zero-permit hang (MCP DoS)"
description: >
  batch_exec/batch_run max_parallel unclamped — Semaphore panic + zero-permit hang (MCP DoS)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-04T10:51:42Z
last_update: 2026-08-04T10:57:34Z
date_finished: 2026-08-04T10:57:34Z
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

# T-2523: batch_exec/batch_run max_parallel unclamped — Semaphore panic + zero-permit hang (MCP DoS)

## Context

The MCP tools `termlink_batch_exec` (handler ~L14396) and `termlink_batch_run`
(handler ~L14722) accept a caller-supplied `max_parallel: Option<usize>`
(struct field `crates/termlink-mcp/src/tools.rs:9571`) and feed it **unclamped**
into `tokio::sync::Semaphore::new(max_parallel)` (L14403 / L14727). Two failure
modes, both reachable from untrusted MCP params:

1. **Panic (DoS):** tokio 1.50.0 `Semaphore::new` does
   `assert!(permits <= MAX_PERMITS)` where `MAX_PERMITS == usize::MAX >> 3`
   (≈2.3e18 on 64-bit). A caller passing e.g. `{"max_parallel": 9000000000000000000}`
   (9e18 > MAX_PERMITS) panics the async tool handler.
2. **Zero-permit hang:** `{"max_parallel": 0}` yields a zero-permit semaphore;
   each spawned task's `sem.acquire().await.unwrap()` (L14417 / L14740) never
   resolves. The acquire sits **outside** the per-command timeout (the timeout
   wraps only the rpc_call / `executor::execute`), so the batch call hangs
   forever and leaks the spawned tasks. This is the more certain harm — no
   dependency on the exact MAX_PERMITS value or panic-isolation behavior.

Both batch tools execute shell commands, so the surface is consequential.
This is the clear outlier: every other numeric param in this file is clamped —
the direct sibling `max_depth` is `p.max_depth.unwrap_or(100).clamp(1, 1024)`
(L19794), `timeout_ms` is `.clamp(100, 30_000)` (L18178). `max_parallel` has
only these 2 sites and neither clamps.

Fix: a shared pure helper `clamp_max_parallel(v) = v.unwrap_or(10).clamp(1, 256)`
used at both sites. Lower bound 1 fixes the hang; upper bound 256 fixes the
panic (and matches the fleet 256 connection-cap convention). An upper bound
above the actual item count is harmless — the semaphore simply never blocks.

## Acceptance Criteria

### Agent
- [x] `clamp_max_parallel` helper added at module scope in `tools.rs`, clamping to `1..=256` with a default of 10
- [x] Both call sites (batch_exec ~L14397, batch_run ~L14723) route `p.max_parallel` through the helper — no remaining unclamped `p.max_parallel.unwrap_or(...)` feeding `Semaphore::new`
- [x] Unit test `clamp_max_parallel_*` asserts `Some(0)=>1`, `Some(9e18)=>256`, `None=>10`, `Some(50)=>50`; proven load-bearing via temp-revert (reverting the clamp makes the test FAIL)
- [x] `cargo test -p termlink-mcp` passes; `cargo build -p termlink-mcp` clean

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
cargo test -p termlink-mcp clamp_max_parallel
cargo build -p termlink-mcp

## RCA

**Symptom:** An MCP agent calling `termlink_batch_exec` / `termlink_batch_run`
with `max_parallel: 0` hangs the tool call forever (leaking spawned tasks); with
a value `> usize::MAX>>3` (~2.3e18) panics the async tool handler (DoS). Both
tools run shell commands, so the surface is consequential.

**Root cause:** `max_parallel` was read as `p.max_parallel.unwrap_or(10)` and
passed straight into `tokio::sync::Semaphore::new()` at both call sites
(tools.rs:14403, :14727) with no clamp. `Semaphore::new` `assert!`s
`permits <= MAX_PERMITS` (panic on huge), and a 0-permit semaphore makes the
per-task `acquire().await` — which sits outside the per-command timeout — block
forever.

**Why structurally allowed:** The file has a near-universal clamp convention for
caller-supplied numeric params (`max_depth.clamp(1,1024)`, `timeout_ms.clamp(100,30_000)`,
`since_days.clamp(1,365)`, watch intervals `[5,3600]`), but the convention is by
discipline, not enforced — a param with only 2 sites slipped through. No test
exercised the extreme `max_parallel` values because the batch handlers require a
live hub/sessions, so the pure clamp logic was never isolated or asserted.

**Prevention:** Extracted the clamp into a pure module-level helper
`clamp_max_parallel` (the only path both sites now use), with a unit test pinning
`0 => 1` and `9e18 => 256`. The test is proven load-bearing via temp-revert
(reverting the clamp makes it FAIL), so a future edit that drops the clamp is
caught in CI rather than in the field. Extraction-to-a-single-guarded-helper is
the same pattern used for `max_depth`/receive-filename sanitization elsewhere in
the campaign.

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

### 2026-08-04T10:51:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2523-batchexecbatchrun-maxparallel-unclamped-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-40a114f9
- **Timestamp:** 2026-08-04T10:58:12Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-04T10:57:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
