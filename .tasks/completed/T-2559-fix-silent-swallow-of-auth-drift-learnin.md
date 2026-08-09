---
id: T-2559
name: "fix silent swallow of auth-drift learning/concern write (reliability, remote.rs)"
description: >
  fix silent swallow of auth-drift learning/concern write (reliability, remote.rs)

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
created: 2026-08-09T07:53:26Z
last_update: 2026-08-09T07:53:26Z
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

# T-2559: fix silent swallow of auth-drift learning/concern write (reliability, remote.rs)

## Context

Reliability-lens review (T-2468, Directive #2 "no silent failures") found two
`let _ = ...` swallows on a LIVE path at `crates/termlink-cli/src/commands/remote.rs:2445-2446`:
`maybe_record_auth_mismatch_learning(...)` and `maybe_track_fleet_failure(...)` —
the T-1052/T-1053 auth-drift auto-register that writes the documented "hub rotated
its secret" learning/concern trail. Both return `Result<()>`; their Err is discarded.
The mismatch itself IS surfaced in fleet-doctor output, so this is not a lost
detection — but a FAILED WRITE of the *persistent audit record* (disk full,
permission error, corrupt YAML) is invisible, so the rotation history silently
stops accumulating with no signal. Per Directive #2 that write failure must be
surfaced. This path already uses `eprintln!` for its human diagnostics (167 sites;
line 2456), and stderr is safe even in `--json` mode (JSON goes to stdout).

## Acceptance Criteria

### Agent
- [x] ALL `let _ = maybe_record_auth_mismatch_learning(...)` / `let _ = maybe_track_fleet_failure(...)` swallows in remote.rs (6 sites: the text path at 2445-2446 AND the JSON path at 5273/5301/5303/5324) are replaced with `if let Err(e) = ... { eprintln!(...) }` naming the hub + error
- [x] The warning goes to stderr (not stdout) so `--json` output stays parseable (the JSON-path sites fire even in `--json` mode)
- [x] `cargo build -p termlink` compiles clean
- [x] Load-bearing (structural): zero `let _ = maybe_record_auth_mismatch_learning` / `let _ = maybe_track_fleet_failure` remain in remote.rs; the `if let Err(e) = maybe_...` forms are present (reverting any site to `let _ =` makes this fail)

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
cargo build -p termlink
# structural: zero let _ = swallows remain (revert any site → this fails)
out=$(grep -cE 'let _ = maybe_record_auth_mismatch_learning|let _ = maybe_track_fleet_failure' crates/termlink-cli/src/commands/remote.rs || true); test "$out" -eq 0
# the error-surfacing wiring is present
grep -q 'if let Err(e) = maybe_record_auth_mismatch_learning' crates/termlink-cli/src/commands/remote.rs
out=$(grep -c 'failed to .* for {name}: {e}' crates/termlink-cli/src/commands/remote.rs); test "$out" -ge 4

## RCA

**Symptom:** A failed WRITE of the auth-drift audit record (the T-1052/T-1053
learning + concern trail written when a hub's secret/cert rotates) was invisible —
`let _ =` discarded the `Result<()>`. If disk was full, the file was read-only, or
the YAML was corrupt, the rotation history silently stopped accumulating with no
operator signal, on a charter-adjacent reliability path.

**Root cause:** six `let _ = maybe_record_auth_mismatch_learning(...)` /
`maybe_track_fleet_failure(...)` call sites (the text fleet-doctor path + the JSON
path) discarded the write Result. The functions were designed "best-effort, never
blocks" — correct intent — but "never blocks" was implemented as "never reports",
conflating non-fatal with invisible.

**Why structurally allowed:** the "best-effort side-effect" idiom (`let _ =`) is
indistinguishable at the call site from a deliberate ignore; nothing flags a
discarded `Result` on an audit-write. Directive #2 ("no silent failures") is a
convention, not an enforced lint — the same class the reliability lens keeps finding.

**Prevention:** all six sites now `eprintln!` on Err (stderr, safe in `--json`).
The task's Verification greps assert zero `let _ =` swallows remain — a source-level
load-bearing check (sibling of the alloc-sink/drain-sink static checks): reverting
any site to `let _ =` re-fires the check. A future Level-C step could generalize
this into a `#![warn(unused_must_use)]`-style lint for these audit-write helpers.

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

### 2026-08-09T07:53:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2559-fix-silent-swallow-of-auth-drift-learnin.md
- **Context:** Initial task creation
