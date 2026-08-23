---
id: T-2835
name: "Drain fully-ticked tasks stuck in started-work through the real P-011 gate"
description: >
  Drain fully-ticked tasks stuck in started-work through the real P-011 gate

status: work-completed
workflow_type: refactor
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-23T20:01:47Z
last_update: 2026-08-23T20:49:41Z
date_finished: 2026-08-23T20:49:41Z
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

# T-2835: Drain fully-ticked tasks stuck in started-work through the real P-011 gate

## Context

`scripts/check-stranded-finalized-tasks.sh` (T-2833) detects tasks declaring
`status: work-completed` in `active/` with no `date_finished` — the finalize
latch. This task addresses the ADJACENT population that check deliberately does
NOT fire on: tasks still declaring `status: started-work` whose Agent ACs are
all ticked and which carry zero unticked ACs. Their work shipped; only the
register disagrees.

Measured 2026-08-23: 52 such tasks. Not a cosmetic backlog — roughly a quarter
of the HV/LC head is finished work, so the BVP quadrant is ranking completed
tasks as available work and mis-ordering everything behind them.

The remedy is to walk them through the REAL P-011 verification gate. Never
`--force`: a task whose verification genuinely fails is a finding worth having,
and forcing it would convert a visible defect into a silent one — the exact
trade the guard layer exists to reverse. `owner: human` tasks are out of scope
(autonomous mode does not delegate their completion).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The stuck population is enumerated by a reproducible script, not by hand — `started-work` AND every Agent AC ticked AND zero unticked
- [x] Every enumerated non-`owner: human` task is put through `fw task update --status work-completed` with no `--force` anywhere in this task
- [x] Each task that finalizes shows all three finalize effects: `date_finished` stamped, moved to `completed/`, episodic generated
- [x] Any task whose P-011 verification genuinely FAILS is left open in `active/` and recorded in this task's Evolution rather than forced through
- [x] No `owner: human` task is completed and no `### Human` AC is ticked by this task
- [x] The residual is reported: how many landed, how many failed verification, how many were skipped as human-owned

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

test -f /root/.claude/jobs/d638a35c/tmp/drain-results.txt
test "$(grep -c LANDED /root/.claude/jobs/d638a35c/tmp/drain-results.txt)" -ge 11
out=$(grep -E '^status:' .tasks/active/T-1885-fw-independent-review-v01--local-only-or.md); echo "$out" | grep -q "started-work"
out=$(grep 'task update' /root/.claude/jobs/d638a35c/tmp/drain.sh); if echo "$out" | grep -q -- '--force'; then exit 1; else exit 0; fi

## Evolution

### 2026-08-23 — the drain found a hole in the gate it was running through

- **What changed:** The enumerated population was 51, not 52 (T-2713 came off the
  list when it closed earlier the same session), and only **24 were mine to
  touch** — 27 are `owner: human` and were skipped untouched. That is the first
  correction worth banking: the headline "52 stuck tasks" overstated the agent-
  actionable set by more than half.

  Of the 24: **12 landed** through a complete P-011 run, **13 were blocked**
  (T-2684 attempted twice). The blocks are not noise and they are not uniform:

  - **8 blocked on one shared cause** — `run-guard-layer.sh` exits 1 because of
    four PRE-EXISTING, unrelated FAILs (`check-episodic-parse` 29 unreadable
    episodics, `check-framework-tracking-drift` dangling refs,
    `check-pickup-deferred-freshness` P-043 stranded, `check-task-id-collisions`).
    T-2684/85/86/88/93/99, T-2709, T-2758. Their Verification blocks assert
    *whole-layer green*, so they cannot close until those four backlogs drain.
    Worth naming as a coupling problem: a task's completion is gated on findings
    it did not cause and does not own.
  - **1 blocked on its own subject** — T-2711 (`revisit-due-scan.sh` still exits
    non-zero). A real, task-specific failure.
  - **3 blocked on a missing agent artifact**, not a verification failure —
    T-2822 and T-1885 wanted `## Recommendation`, T-2569 wants `## RCA`. Writing
    those is authorship, not a bypass; T-2822 was written and landed to T-193
    partial-complete with its Human AC untouched.
  - **1 not a block at all** — T-2409 routed to the inception review queue (rc=0).

- **Plan impact — the finding that matters.** T-1885 was written a
  `## Recommendation` and then *completed*, and it should not have been. Its gate
  printed `Verification: 3/4 passed ✓` — a checkmark on a fraction that is
  visibly not whole — with **zero** FAIL lines. Command 4 was never executed.

  Cause, confirmed by minimal repro rather than inference: the P-011 loop
  (`update-task.sh:~1145`) drives commands with `done <<< "$verify_cmds"`, so the
  `eval`'d command inherits the herestring as its stdin. Command 3 reads stdin and
  swallows command 4. `verify_total` is computed independently
  (`wc -l`, line ~1139) so the skipped command stays in the denominator while
  incrementing neither counter; the blocking test is `verify_fail -gt 0`, which is
  false, so the gate falls through to the green branch. Adding `< /dev/null` to the
  eval takes the repro from `2/4 passed (fail=0)` to `4/4`.

  Command 4 does not merely go unrun — it **fails**: the classifier's confidence is
  now 60.5% (78/129) against the ≥80% GO threshold T-1885's own AC cites. The
  corpus grew 84→129 and accuracy fell with it. So the gate closed a task whose
  central quality claim is currently false. T-1885 was reverted to `started-work`
  by direct frontmatter repair (`work-completed` has no outgoing CLI transition);
  `owner: human` was left untouched.

  This is the same defect class as T-2830 approached from the opposite side —
  there, commands under the wrong heading meant zero ran and the gate passed
  vacuously; here, a stdin read means the tail never runs and the gate passes
  visibly-partially. Both are the gate asserting a property adjacent to the one it
  claims. Swept this session's 12 finalize logs for the signature (a fraction whose
  halves differ): T-1885 is the only one — the other 11 ran whole.

- **Triggered:** Filed upstream at `framework:pickup` **offset 34** with the repro,
  the observed instance, and a three-part remedy: close stdin on the eval; assert
  `pass + fail == total` and treat a mismatch as a FAILURE so the class cannot
  regress silently; and stop rendering a checkmark on a non-whole fraction.
  `update-task.sh` is vendored, so it was **not** patched here (G-062) — no file
  under `.agentic-framework/` was modified by this task.

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

### 2026-08-23T20:01:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/t2687-pickup-failopen/.tasks/active/T-2835-drain-fully-ticked-tasks-stuck-in-starte.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-a639b0b0
- **Timestamp:** 2026-08-23T20:49:42Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **l387-sigpipe-risk** (partial, heuristic) @ Verification:line 35
     - evidence: `out=$(grep 'task update' /root/.claude/jobs/d638a35c/tmp/drain.sh); if echo "$out" | grep -q -- '--force'; then exit 1; else exit 0; fi`

### 2026-08-23T20:49:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
