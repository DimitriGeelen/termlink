---
id: T-2247
name: "R2d substrate-fitness: update presence-retention runbook to safe set-retention+sweep path (deprecate sqlite3 footgun)"
description: >
  R2d substrate-fitness: update presence-retention runbook to safe set-retention+sweep path (deprecate sqlite3 footgun)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-substrate-fitness]
arc_id: arc-substrate-fitness
components: []
related_tasks: [T-2244, T-2245, T-2246]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-22T21:30:28Z
last_update: 2026-06-22T21:30:28Z
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

# T-2247: R2d substrate-fitness: update presence-retention runbook to safe set-retention+sweep path (deprecate sqlite3 footgun)

## Context

R2d of arc-substrate-fitness (arc-002) — the operator-facing close-out. The R2a/R2b/R2c
verticals (set_retention RPC+CLI+MCP, sweep RPC+CLI+MCP, latest_per_cv_key mode) made
`docs/operations/agent-presence-retention-reset.md` (T-2059) actively WRONG and dangerous:
§2 asserts "TermLink has no channel.set-retention RPC", §4 instructs operators to do raw
`sqlite3 UPDATE topics` edits and stop the hub, and §6 lists set-retention as an unfiled
"future task". Following that runbook today means bypassing the hub and risking a metadata
race when a clean, online, race-free verb path now exists. This task rewrites the runbook to
the safe path and makes the sweep operational (a cron recipe), preventing PL-168 (a shipped
verb nobody runs because it isn't documented).

## Acceptance Criteria

### Agent
- [x] `agent-presence-retention-reset.md` §2 no longer claims "no channel.set-retention RPC";
      it documents `termlink channel set-retention` + `termlink channel sweep` (+ MCP twins).
- [x] The safe online path (`set-retention latest-per-cv-key` → `sweep`) is the PRIMARY
      recommendation for `agent-presence`; `latest-per-cv-key` is explained as the proper
      T-1991 fix (record count tracks agent COUNT, not heartbeat count).
- [x] The `sqlite3 UPDATE topics` + hub-stop path is explicitly marked LEGACY/deprecated
      (kept only as an emergency fallback), not the default.
- [x] A periodic-sweep cron recipe is included (the trigger that makes any retention policy
      actually enforce — the bus runs no background sweep thread).
- [x] §6 "future task: channel.set-retention RPC ... Not filed" note is corrected to point at
      T-2244/T-2245/T-2246 as shipped.
- [x] Doc is internally consistent (no remaining "no such verb" statements) and the verb
      spellings match the CLI (`set-retention`, `sweep`, `latest-per-cv-key`) — grep-verified.

## Verification

test -f docs/operations/agent-presence-retention-reset.md
grep -q 'set-retention' docs/operations/agent-presence-retention-reset.md
grep -q 'latest-per-cv-key' docs/operations/agent-presence-retention-reset.md
grep -q 'channel sweep agent-presence' docs/operations/agent-presence-retention-reset.md
! grep -q 'no .channel.set-retention. RPC' docs/operations/agent-presence-retention-reset.md

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

### 2026-06-22 — runbook was actively dangerous, not just stale
- **What changed:** This wasn't a cosmetic doc refresh. The T-2059 runbook's §4 told operators
  to `sqlite3 UPDATE topics` and stop the hub — a footgun that bypasses hub invariants and
  races in-flight writes. R2a/R2b/R2c made that unnecessary AND made the doc's §2 factually
  false ("no channel.set-retention RPC"). An operator trusting the doc today would take a
  riskier action than the substrate now requires.
- **Plan impact:** Reframed R2d from "document the new verbs" to "deprecate the footgun + make
  the verbs the default path". Added the periodic-sweep cron recipe (the trigger) so the
  capability is operational, not just documented (PL-168 dormant-tooling prevention).
- **Triggered:** None. Closes the autonomously-buildable portion of arc-002's R2 line
  (R2a→R2b→R2c→R2d). Remaining arc work (R3/R5/R6 Sovereign inceptions, R7 live-host ops) is
  not agent-startable.

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

### 2026-06-22T21:30:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2247-r2d-substrate-fitness-update-presence-re.md
- **Context:** Initial task creation
