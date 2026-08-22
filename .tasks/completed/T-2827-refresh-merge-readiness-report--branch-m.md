---
id: T-2827
name: "Refresh merge-readiness report — branch moved to 87 commits, tools.rs now conflicts and main already has the fix"
description: >
  Refresh merge-readiness report — branch moved to 87 commits, tools.rs now conflicts and main already has the fix

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-22T10:07:32Z
last_update: 2026-08-22T10:12:40Z
date_finished: 2026-08-22T10:12:40Z
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

# T-2827: Refresh merge-readiness report — branch moved to 87 commits, tools.rs now conflicts and main already has the fix

## Context

`.context/merge-plan-t2687.md` was produced by a trial merge on 2026-08-20 at `c8607a501`.
Nine commits have landed since, so its header table (78 commits / 37 conflicts / 377 files)
is wrong on every figure — and a merge plan that is confidently wrong about its own scope is
worse than none, because the reader has no reason to re-check it.

Re-measuring turned up one substantive change rather than just new numbers. The single new
conflict, `crates/termlink-mcp/src/tools.rs`, is **duplicated work**: main fixed the
`termlink_topics` silent-partial-inventory defect as T-2687, and this branch fixed the same
defect again as T-2824 eight days later. The two are functionally identical; main's is
better factored. So the resolution is take-main, and the branch's version is dropped.

Also recorded: why `check-task-id-collisions.sh` axis C could not see it (`--diff-filter=A`
only examines *added* files, and main is excluded from the branch set as BASE). That is a
detector gap, captured separately — not fixed here.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The report states the CURRENT branch head, merge base, commit count and conflict
      count, each produced by a command re-run at write time — not carried over from the
      2026-08-20 trial (which said 78 commits / 37 conflicts and is now wrong on both)
- [x] `crates/termlink-mcp/src/tools.rs` — the one conflict that is new since the trial —
      has a written resolution with its reason, and the reason cites the actual content of
      both sides rather than a guess about which is newer
- [x] The report records that main already carries an equivalent fix for the same defect,
      naming both task IDs, so the duplicated work is visible rather than silently dropped
- [x] The stale figures in the existing report are corrected in place, so a reader cannot
      pick up the superseded 78/37 numbers from the same file

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

# The report exists and names the current head, not the superseded one.
# head -30 rather than the whole file: the point is that the header table itself is
# correct, not that the right sha appears somewhere further down next to the old one.
out=$(head -30 .context/merge-plan-t2687.md 2>&1); echo "$out" | grep -q 'fdb70a144'
# The superseded commit count is corrected in the header table, not merely contradicted later.
out=$(head -30 .context/merge-plan-t2687.md 2>&1); echo "$out" | grep -q '| commits landing | 87 |'
# The new tools.rs conflict has a written resolution.
out=$(cat .context/merge-plan-t2687.md 2>&1); echo "$out" | grep -q 'termlink-mcp/src/tools.rs'
# The duplicate-work finding names both task IDs.
out=$(cat .context/merge-plan-t2687.md 2>&1); echo "$out" | grep -q 'T-2824'
# The conflict count in the report matches what git actually reports right now.
git merge-tree --write-tree --name-only HEAD origin/main > /root/.claude/jobs/d638a35c/tmp/v.txt 2>&1; true
n=$(sed -n '2,$p' /root/.claude/jobs/d638a35c/tmp/v.txt | sed '/^$/,$d' | grep -c .); test "$n" -eq 38

## RCA

**Symptom:** The same defect in `termlink_topics` — a chained `if let` that swallowed a
timeout, a transport error, an error response and a missing `topics` array identically, so a
caller got a partial topic inventory with no way to know it was partial — was fixed twice.
Main fixed it as T-2687. This branch fixed it again as T-2824, eight days later. The
duplication only surfaced as a merge conflict.

**Root cause:** No check compares a long-lived feature branch's work against what has
already landed on main. This branch diverged on 2026-08-13 and main took 230 commits after
that, so the window for someone else to fix the same thing was wide, and nothing was
watching it.

**Why structurally allowed:** `check-task-id-collisions.sh` was built specifically to catch
duplicated work, and axis C could not see this instance for two independent reasons, either
of which alone would have been sufficient:

1. It runs `git diff --name-only --diff-filter=A`, so it only considers files **added** on a
   branch. `tools.rs` existed at the merge base and both sides *modified* it. Axis C detects
   duplicated **new files**; it is structurally blind to duplicated **fixes to existing
   files**, which is the more common shape.
2. Its branch list is `[b for b in git branch ... if b != BASE]` — **main is excluded**. Even
   a future axis that compared modifications would not have looked at main, which is exactly
   where the competing fix lived.

Axis B (near-duplicate titles) would not have helped either: it compares titles *across
branches*, and main is not in that set for the same reason.

This is the second instance of the duplicated-work class in this repo. The first — two
independent `check-verification-pipefail.sh` implementations — *was* caught, but only
because it happened to be a new file on two feature branches, which is precisely the narrow
case axis C covers. The catch was luck about the shape, not coverage.

**Prevention:** Not fixed here, and deliberately so — the fix is a new axis rather than a
tweak, and "two branches modified the same file" is far too noisy to fire on without a
false-positive story first (nearly every pair of branches touches some common file). It
needs to compare at a finer grain than the file, and to earn its threshold, or it will be
switched off the first week. Captured as its own task.

Two things do exist in the meantime. The finding is written into
`.context/merge-plan-t2687.md` beside the resolution, so whoever performs the merge reads it
at the moment it is actionable. And the cheap manual mitigation is recorded there too:
before implementing a fix on a long-lived branch, run `git log origin/main -- <path>` on the
file about to be changed.

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

### 2026-08-22T10:07:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/t2687-pickup-failopen/.tasks/active/T-2827-refresh-merge-readiness-report--branch-m.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-9e010339
- **Timestamp:** 2026-08-22T10:12:41Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#2 (Agent)** — `crates/termlink-mcp/src/tools.rs` — the one conflict that is new since the trial —
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-mcp/src/tools.rs in: `crates/termlink-mcp/src/tools.rs` — the one conflict that is new since the trial —`

### 2026-08-22T10:12:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
