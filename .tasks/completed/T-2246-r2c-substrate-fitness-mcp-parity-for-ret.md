---
id: T-2246
name: "R2c substrate-fitness: MCP parity for retention-management verbs (channel.sweep + set_retention)"
description: >
  R2c substrate-fitness: MCP parity for retention-management verbs (channel.sweep + set_retention)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-substrate-fitness]
arc_id: arc-substrate-fitness
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: [T-2245, T-2244]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-22T21:21:12Z
last_update: 2026-06-22T21:28:41Z
date_finished: 2026-06-22T21:28:41Z
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

# T-2246: R2c substrate-fitness: MCP parity for retention-management verbs (channel.sweep + set_retention)

## Context

R2c of arc-substrate-fitness (arc-002). T-2245 (R2b) added the `channel.sweep` RPC + CLI
verb and the `latest_per_cv_key` retention mode; T-2244 (R2a) added `channel.set_retention`.
Neither has an MCP twin — yet `channel.create` does (`termlink_channel_create`). This closes
the MCP-parity gap so orchestrator agents can manage + enforce retention without shelling out
(PL-172: a CLI feature missing from its MCP wrapper is a silent-strip bug). Mechanical mirror
of the existing `termlink_channel_create` MCP tool.

## Acceptance Criteria

### Agent
- [x] `termlink_channel_sweep` MCP tool exists (param `topic`), calls `CHANNEL_SWEEP`,
      returns the hub's `{ok, topic, pruned}` envelope; registered in the tool list.
- [x] `termlink_channel_set_retention` MCP tool exists (params `name`, `retention_kind`,
      optional `retention_value`), calls `CHANNEL_SET_RETENTION`; registered in the tool list.
- [x] Both MCP tools accept `latest_per_cv_key` as a retention kind, and the existing
      `termlink_channel_create` MCP handler is extended to accept it too (shared `retention_json`
      helper — PL-172: no silent strip; one place to add the next kind).
- [x] `cargo build -p termlink-mcp` compiles; MCP tool registration list includes both
      new tool names (grep: 6 matches across struct/tool/registry).
- [x] No regression: `cargo test -p termlink-mcp --lib` passes 867/867. The full
      `cargo test -p termlink-mcp` shows 6 `mcp_integration` failures (list_sessions /
      discover / topics) — confirmed IDENTICAL on clean HEAD via `git stash`, i.e.
      pre-existing + environmental (sandbox has no live sessions/hub), NOT an R2c regression.

## Verification

cargo build -p termlink-mcp
cargo test -p termlink-mcp --lib

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

## Evolution

### 2026-06-22 — R2a (set_retention) also lacked MCP parity; folded in
- **What changed:** Scoping R2c for `channel.sweep`'s MCP twin, found `channel.set_retention`
  (R2a / T-2244) never got one either — though `channel.create` has had `termlink_channel_create`
  since T-1160. Both retention-management verbs were CLI-only.
- **Plan impact:** Treated "retention-management MCP surface" as one deliverable (set_retention +
  sweep) rather than minting a separate R2a-MCP fixup. Added a shared `retention_json` helper so
  create/set_retention can't drift on supported kinds (PL-172 silent-strip prevention) — adding
  the next retention kind is now a one-line change in one place.
- **Triggered:** None minted.

### 2026-06-22 — 6 pre-existing mcp_integration failures surfaced (not mine)
- **What changed:** `cargo test -p termlink-mcp` (full) shows 6 failing `mcp_integration` tests
  (list_sessions / discover / topics: "invalid type: map, expected a sequence"). Verified via
  `git stash` that they fail IDENTICALLY on clean HEAD — pre-existing, and environmental (the
  sandbox has no live sessions/hub for the integration harness to enumerate).
- **Plan impact:** Verification scoped to `--lib` (867/867 pass). These are NOT an R2c
  regression. Flagged to the operator; whether they indicate real contract drift vs a
  test-harness env assumption is a separate investigation (candidate gap, out of R2c scope).
- **Triggered:** Surfaced to user for disposition (not auto-filed — needs a "how long failing /
  is it env-only" check first).

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

### 2026-06-22T21:21:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2246-r2c-substrate-fitness-mcp-parity-for-ret.md
- **Context:** Initial task creation

### 2026-06-22T21:28:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
