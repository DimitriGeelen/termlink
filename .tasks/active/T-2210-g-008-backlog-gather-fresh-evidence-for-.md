---
id: T-2210
name: "G-008 backlog: gather fresh evidence for partial-complete Human ACs"
description: >
  For the ~48 partial-complete tasks, run the mechanically-verifiable Human-AC verification steps, capture fresh evidence, and inject timestamped Updates entries so the human can batch-confirm. Does NOT tick Human ACs (sovereignty). Surfaces operator-bound ACs separately. Batch-injection per G-008 pattern.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [G-008, backlog, human-review]
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-13T12:37:56Z
last_update: 2026-06-13T13:54:51Z
date_finished: 2026-06-13T13:54:17Z
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

# T-2210: G-008 backlog: gather fresh evidence for partial-complete Human ACs

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Triage script buckets every partial-complete task (active/ with status=work-completed OR owner=human) by Human-AC marker type ([REVIEW]/[RUBBER-STAMP]/[REVIEWER]/none) and local-verifiability; table written to report
- [x] For the locally-verifiable cluster, the Human-AC Steps are re-run and fresh command output captured per task
- [x] Each processed task receives a timestamped "G-008 fresh evidence" entry in its `## Updates` (no `### Human` AC is ever ticked — sovereignty)
- [x] Master evidence report written to `docs/reports/T-2210-human-review-evidence.md` and the user notified (in-session response; no user PTY session for `termlink inject` in this bg job)

### Human
- [ ] [REVIEW] The captured evidence is sufficient to batch-confirm the rubber-stampable ACs
  **Steps:**
  1. Read `docs/reports/T-2210-human-review-evidence.md`
  2. For each "READY" task, confirm the captured output satisfies its Human AC
  **Expected:** you can tick the Human ACs of the READY tasks from the report alone, without re-running anything
  **If not:** note which tasks lack sufficient evidence; they move to the "needs operator environment" bucket

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

test -f docs/reports/T-2210-human-review-evidence.md
grep -rq "G-008 fresh evidence" .tasks/active/

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

## Recommendation

**Recommendation:** GO (partial-complete — Agent ACs met; one Human [REVIEW] AC awaits you)

**Rationale:** All four Agent ACs are satisfied. 48 partial-complete backlog tasks now
carry timestamped "G-008 fresh evidence" entries in their `## Updates`, captured by
re-running each task's mechanically-verifiable Human-AC Steps against today's binary
(0.11.1293) and live hub. Sovereignty was preserved — zero `### Human` checkboxes were
ticked (verified: `grep -c '[x]'` in each Human section = 0). The remaining Human AC is
your judgment call: confirm the captured evidence is sufficient to batch-tick the READY
tasks. Operator-env tasks (ssh/GitHub/remote) were honestly flagged, not faked.

**Evidence:**
- `docs/reports/T-2210-human-review-evidence.md` — triage (84 tasks) + per-cluster READY table + flagged items + operator-env bucket
- `docs/reports/T-2210-evidence/{cluster-A..D,batch2-a,batch2-b}.md` — raw command/exit/output per task
- 48 task files with fresh `## Updates` evidence entries (commits 4481b78a, 0f756046)
- Flagged for your eye before confirming: T-1485 (divergent-but-loud error), T-1529-1532 (mutating verbs, parse-confirmed only)

## Updates

### 2026-06-13T12:37:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2210-g-008-backlog-gather-fresh-evidence-for-.md
- **Context:** Initial task creation

### 2026-06-13T13:54:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
