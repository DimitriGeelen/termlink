---
id: T-2833
name: "task finalize half-runs leaving work-completed tasks in active and deadlocking commits"
description: >
  task finalize half-runs leaving work-completed tasks in active and deadlocking commits

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
created: 2026-08-23T17:14:52Z
last_update: 2026-08-23T17:14:52Z
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

# T-2833: task finalize half-runs leaving work-completed tasks in active and deadlocking commits

## Context

`fw task update --status work-completed` **half-runs**: it writes
`status: work-completed` but leaves `date_finished: null` and never moves the file
out of `.tasks/active/`. Observed twice in one session, on T-2831 and T-2832 —
both of which passed every gate first (4/4 and 8/8 ACs, 9/9 and 5/5 P-011
commands, reviewer PASS).

**Why it matters more than a cosmetic status field:** the half-finalized state
**deadlocks commits**. P-002 refuses a commit while focus is on a completed task;
the T-1730 focus-drift gate refuses one while focus is on any *other* task. The
only exits are a logged Tier-2 bypass or attributing the commit to a different
task — which is what commits `2dabb2da5` and `9798e715a` had to do, named
explicitly in their messages. Every subsequent task inherits the problem.

**It is invisible to the existing guard.** The T-2290 task-finalization canary
scans `.tasks/completed/` only, so a task that never leaves `active/` is
structurally unreachable by it. Same end-state as the G-066 bypass class (work
done, register disagrees) arrived at from the opposite direction.

### Diagnosis so far — narrowing, not yet root cause

- **NOT the documented T-2806 path.** CLAUDE.md attributes this class to
  `lib/evolution_log.sh:52` sourcing `lib/arc_membership.sh` unguarded, which
  exits 1 after all gates pass when that file is absent. The unguarded source is
  confirmed present at line 52 — but **`arc_membership.sh` and `arc_membership.py`
  both exist** in this worktree (7258 / 6103 bytes), so that specific dependency
  is satisfied and cannot be the cause here.
- **The stop is between two writes.** A successful run (T-2830, same session)
  prints, in order: AC check → reviewer scan → `Status: started-work →
  work-completed` → `date_finished set to ...` → `Moved to completed/` → focus
  cleared → components resolved → episodic generation. T-2831 and T-2832 reached
  the status write and produced none of the four steps after it.
- **Three dangling framework refs are live in this worktree** —
  `$FRAMEWORK_ROOT/tools/corpus_{explain,lint,spec}.py`, reported by
  `check-framework-tracking-drift.sh`. They are referenced by
  `agents/designer/designer.sh` and `bin/fw`, **not** by the finalize path, so
  they are a separate finding and most likely not this cause. Worth re-checking
  only after the finalize path itself has been read.

### Next step for whoever picks this up

Read `agents/task-create/update-task.sh` at the finalize block — locate the
`date_finished` write and the `completed/` move, and find what sits between them
and the status write that can exit non-zero without printing. Capture a FULL
run (`fw task update <a fresh task> --status work-completed` with no `head`
truncation) — both observations here were truncated at the reviewer verdict, so
the failing line has never actually been seen. If the culprit is in
`.agentic-framework/`, it is vendored: route upstream per G-062 and register it
in `.vendor-divergence.yaml` rather than patching in place.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] The exact failure point in the finalize path is identified by reading the code — which step sets `status`, which sets `date_finished`, which moves the file, and what stops between them
- [ ] The finding names whether this is a vendored defect (G-062, route upstream) or locally fixable, with evidence for the call
- [ ] T-2831 and T-2832 reach a consistent finalized state — either fully finalized, or explicitly left and documented, never silently half-done
- [ ] A check exists that detects a task whose `status: work-completed` while it is still in `.tasks/active/` — the state the T-2290 canary is structurally blind to because it scans `completed/` only
- [ ] That check is proven load-bearing: red against the current tree (which has two such tasks), green once they are resolved

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

## Updates

### 2026-08-23T17:14:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/t2687-pickup-failopen/.tasks/active/T-2833-task-finalize-half-runs-leaving-work-com.md
- **Context:** Initial task creation
