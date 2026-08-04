---
id: T-2526
name: "termlink_dispatch count unbounded — u32 pre-allocates ~100GB Vec, OOMs MCP server before spawn"
description: >
  termlink_dispatch count unbounded — u32 pre-allocates ~100GB Vec, OOMs MCP server before spawn

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
created: 2026-08-04T11:20:35Z
last_update: 2026-08-04T11:20:35Z
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

# T-2526: termlink_dispatch count unbounded — u32 pre-allocates ~100GB Vec, OOMs MCP server before spawn

## Context

The `termlink_dispatch` MCP tool takes a caller-supplied `count: u32`
(`DispatchParams`, tools.rs:9444) with the **only** validation being a lower guard
`if p.count == 0` (tools.rs:12860). `count` then flows unbounded into two eager
allocations: `Vec::with_capacity(count as usize)` (tools.rs:12914, `count` String
slots) and `vec![false; count as usize]` (tools.rs:13001). A caller passing
`{"count": 4294967295, ...}` reaches `Vec::<String>::with_capacity(4.29e9)` ≈
**103 GB** reservation → OOM-abort of the MCP server **before any worker spawns**.

Amplified (one small JSON int → gigabytes), reachable from untrusted MCP input.
This is the string/collection analog of T-2523's `max_parallel`→`Semaphore::new`
panic, in the SAME file, and the clear outlier against that file's own clamp
convention (`clamp_max_parallel(v)=v.unwrap_or(10).clamp(1,256)`, tools.rs:403,
"Mirrors the max_depth.clamp(1,1024) convention used everywhere else").

**Fix (in-authority, resource guard):** an upper bound `MAX_DISPATCH_COUNT = 256`
(mirrors the fleet 256-worker/connection cap). Unlike `max_parallel` — a
concurrency *hint* where a silent clamp is harmless — `dispatch count` **is** the
number of workers actually spawned, so silently clamping 300→256 would do less
work than the caller asked without telling them. Therefore **loud-reject**
(`json_err`) counts above the max rather than clamp, folding the existing `== 0`
check into one pure `validate_dispatch_count` helper. This prevents the OOM while
keeping the caller's intent honest (they resubmit with a sane count or loop).

## Acceptance Criteria

### Agent
- [x] Pure helper `validate_dispatch_count(count: u32) -> Option<String>` returns an error message for `count == 0` OR `count > MAX_DISPATCH_COUNT` (256), `None` otherwise
- [x] `termlink_dispatch` calls the helper and returns its message via `json_err` before reaching the `Vec::with_capacity(count)` / `vec![false; count]` allocations — no remaining unbounded `count` reaching those sinks
- [x] Unit test asserts `0`, `257`, and `u32::MAX` are rejected and `1`/`256` accepted; proven load-bearing via temp-revert (dropping the upper-bound branch makes the `u32::MAX` assertion FAIL)
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
cargo test -p termlink-mcp validate_dispatch_count
cargo build -p termlink-mcp

## RCA

**Symptom:** An MCP agent calling `termlink_dispatch` with a large `count`
(e.g. `4294967295`) OOM-aborts the MCP server before any worker spawns — a single
tiny JSON field kills the process (amplified DoS).

**Root cause:** `count: u32` (DispatchParams) was validated only for `== 0`
(tools.rs:12860), then used eagerly in `Vec::with_capacity(count as usize)`
(tools.rs:12914, `count` × 24 B for `String`) and `vec![false; count as usize]`
(tools.rs:13001). No upper bound → `u32::MAX` reserves ~103 GB up front.

**Why structurally allowed:** Same by-discipline-not-enforced clamp convention gap
as T-2523: this file clamps nearly every caller count (`clamp_max_parallel`,
`max_depth.clamp(1,1024)`, `since_days.clamp(1,365)`), but `dispatch.count` had a
`== 0` guard that *looked* like validation while leaving the ceiling open. A
pre-allocation from a caller count is invisible to any test that doesn't pass an
extreme value, and the dispatch handler needs a live hub so the pure bound was
never isolated.

**Prevention:** A pure `validate_dispatch_count(u32) -> Option<String>` folding the
`== 0` check and a new `MAX_DISPATCH_COUNT = 256` upper bound, called before the
allocations. Loud-reject (not silent clamp) because `count` == workers actually
spawned. Unit-tested for 0/1/256/257/u32::MAX, proven load-bearing via temp-revert
(drop the upper-bound branch → the `u32::MAX` assertion FAILs). Third
unclamped-caller-count→sink fix this window (T-2523 Semaphore, T-2526 Vec) — the
recurring class is "a lower/format guard mistaken for full validation".

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

### 2026-08-04T11:20:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2526-termlinkdispatch-count-unbounded--u32-pr.md
- **Context:** Initial task creation
