---
id: T-2527
name: "Static check for unclamped caller-param to allocation sinks (G-019 prevention for T-2523/T-2526 class)"
description: >
  Heuristic detector flagging Vec::with_capacity / vec![_;n] / Semaphore::new / String::with_capacity on caller-supplied params without a clamp, in MCP/hub handler files

status: captured
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
created: 2026-08-04T12:22:21Z
last_update: 2026-08-04T12:22:21Z
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

# T-2527: Static check for unclamped caller-param to allocation sinks (G-019 prevention for T-2523/T-2526 class)

## Context

**G-019 structural prevention for the class closed by T-2523 + T-2526.** Both were the
same defect shape: a caller-supplied param (`max_parallel: usize`, `count: u32`) reached an
eager allocation / resource sink (`tokio::sync::Semaphore::new`, `Vec::with_capacity`,
`vec![_; n]`) with only a lower/format guard (`unwrap_or(..)`, `== 0`) and no upper bound —
so `u32::MAX`/`9e18` either panics or pre-allocates ~100 GB, OOM-aborting the MCP server
from one tiny JSON field. The file's *own* convention clamps nearly every numeric caller
param (`clamp_max_parallel`, `max_depth.clamp(1,1024)`, `since_days.clamp(1,365)`), but the
convention is **by discipline, not enforced** — a param with few call sites slips through and
stays invisible until an adversarial hunter reads that exact line. Two instances in one
window means the mechanism will recur. This task builds the check that makes the convention
load-bearing (the same "empty-log = healthy" canary/check pattern used across this repo).

**Scope (deliberately heuristic, best-effort — like the charter-drift / canary checks).** A
precise "does this arg trace to an unclamped caller param" answer needs real dataflow
analysis; that is out of scope. Instead: a grep/AST-lite scanner over the handler files
(`crates/termlink-mcp/src/tools.rs`, `crates/termlink-hub/src/**`, `crates/termlink-session/src/**`)
that flags each `Vec::with_capacity(<x>)`, `String::with_capacity(<x>)`, `vec![_; <x>]`,
`Semaphore::new(<x>)`, and `.repeat(<x>)` whose argument is a **bare identifier or `p.<field>`
expression** (not a literal, not a `.clamp(`/`.min(`/`.take(`-bounded expression, not a
`.len()` of an already-materialized collection). Output is a REVIEW list (candidate sites),
not a hard gate — false positives are acceptable and expected; the value is surfacing new
sinks for a human/agent to confirm-and-clamp. An allowlist file
(`.context/working/.alloc-sink-allowlist`) lets confirmed-safe sites (bounded-by-construction)
be acknowledged so the check trends toward empty. Model it on
`scripts/check-charter-drift-freshness.sh` (exit 0 clean / 1 candidates found / 2 tooling),
optionally wired as a daily canary appending to `.context/working/.alloc-sink-canary.log`.

**Why a check and not just "be careful":** the two fixes prove carefulness already failed
twice. The Post-Fix Root Cause Escalation rule (G-019) requires prevention distinct from the
per-instance fix — "did I fix the symptom, or the reason the framework couldn't detect it?".
This is that prevention. Predecessors: T-2523, T-2526 (the two instances); sibling pattern:
the 13 `*-canary` checks documented in CLAUDE.md.

## Acceptance Criteria

### Agent
- [ ] `scripts/check-alloc-sink-clamps.sh` scans the handler crates and flags `with_capacity` / `vec![_;n]` / `Semaphore::new` / `.repeat(n)` sites whose size arg is a bare identifier or `p.<field>` (not a literal / not `.clamp`/`.min`/`.take`-bounded)
- [ ] Running it against the CURRENT tree (post T-2523/T-2526) reports 0 firing sites, OR reports only sites acknowledged in `.context/working/.alloc-sink-allowlist` — i.e. the known-good baseline is clean
- [ ] A deliberately-planted un-clamped `Vec::with_capacity(some_caller_param)` in a test fixture IS flagged (proving the detector fires), and the same site wrapped in `.clamp(1,256)` is NOT flagged (proving it recognizes the fix)
- [ ] Exit codes: 0 = clean, 1 = unacknowledged candidate(s), 2 = tooling error; `--json` envelope; documented in CLAUDE.md alongside the canary set
- [ ] Load-bearing proof: temp-revert one of the T-2523/T-2526 clamps in a scratch copy and confirm the check flags it (the check would have caught the original defect)

<!-- Human section removed — fully agent-verifiable (a shell check + fixtures). -->

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
bash scripts/check-alloc-sink-clamps.sh
bash tests/alloc-sink-check-fixtures.sh   # (or a cargo/bats test that plants + clamps a fixture)

## RCA

**Symptom (of the class this prevents):** Twice in one window an adversarial hunter found a
caller-supplied param reaching an allocation/resource sink with no upper bound —
`max_parallel`→`Semaphore::new` (T-2523, panic + 0-hang) and `count`→`Vec::with_capacity`
(T-2526, ~100 GB pre-alloc OOM). Each was a one-line omission invisible until someone read
that exact line.

**Root cause (of the blindness):** the repo has a strong "clamp every caller numeric param"
convention but NO mechanism enforces it. New handlers (and old low-traffic ones) can wire a
param straight to `with_capacity`/`Semaphore::new`/`vec![_;n]`/`.repeat` and nothing objects
until it OOMs in the field. A lower/format guard (`== 0`, `unwrap_or`) reads as "validated"
while leaving the ceiling open.

**Why structurally allowed:** no static check, no test, no canary covers this shape — the
existing 13 canaries watch runtime/fleet state, not source-level allocation antipatterns.

**Prevention (this task):** a source scanner that flags candidate sinks + an allowlist that
trends the known-good baseline to empty, optionally a daily canary. Distinct from the two
point fixes (which clamped the two known sites) — this catches the *next* site before it
ships. Closes the G-019 loop the two fixes opened ("fixed the symptom; did I fix why the
framework was blind?" — this is the "no").

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

### 2026-08-04T12:22:21Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2527-static-check-for-unclamped-caller-param-.md
- **Context:** Initial task creation
