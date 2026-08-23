---
id: T-2833
name: "task finalize half-runs leaving work-completed tasks in active and deadlocking commits"
description: >
  task finalize half-runs leaving work-completed tasks in active and deadlocking commits

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/check-stranded-finalized-tasks.sh, tests/stranded-finalized-check-fixtures.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-23T17:14:52Z
last_update: 2026-08-23T19:09:44Z
date_finished: 2026-08-23T19:09:44Z
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

### Root cause — found, and it is a latch

`update-task.sh` writes the status field **unconditionally** at line ~1681 and then
runs the finalize block — stamp `date_finished` (~1904), `git mv` to `completed/`
(~1946), clear focus (~2001), generate episodic (~2194) — **221 lines later**,
behind this guard at line ~1902:

```bash
if [ -n "$NEW_STATUS" ] && [ "$NEW_STATUS" = "work-completed" ] \
   && [ "$OLD_STATUS" != "work-completed" ]; then
```

`$OLD_STATUS` is read from the file at startup. The two writes are not atomic and
**the guard reads the value the first write already committed to disk**. So a run
that writes the status and then dies before line 1902 leaves every later
invocation seeing `OLD_STATUS=work-completed`, the third clause FALSE, and the
whole finalize block skipped. Not an error — an `if` that does not match.

`date_finished` is written at exactly one site, inside that block. Once the latch
closes, the field is unreachable by any code path.

**The trigger was almost certainly our own output truncation.** Both prior
observations were piped through `head`, and the reviewer verdict prints
immediately before the status write — so `head -N` closes the pipe right there
and SIGPIPE kills the script inside the two-write window. L-387 again, one layer
up: this time the SIGPIPE did not fail a verification, it corrupted state.

### The partial recovery hides it rather than fixing it

Re-running the command *does* reach a different branch — the partial-complete
re-check at ~1395 — which git-mv's the file to `completed/`. **That branch never
stamps `date_finished`.** Confirmed live on both tasks: each printed
`Moved to completed/` with no `date_finished set to ...` line, and each now sits
in `completed/` with `date_finished: null`.

That is precisely the T-2290 canary's **soft, non-firing** class (work-completed
with empty `date_finished` — informational by default, fires only under
`--strict`). The system self-heals into its own blind spot.

### Vendored — detection built, fix routed upstream

`agents/task-create/update-task.sh` is under `.agentic-framework/`, so per G-062
it was **not patched here**; a local fix is deleted by the next re-vendor. Filed
upstream at `framework:pickup` **offset 32**, with three ordered remediations
(make the trigger idempotent by also firing when the task claims work-completed
but carries no date; or write status last; and either way have the ~1395 branch
stamp the date).

### The first draft of the check was wrong, and that is the durable lesson

Keyed on "work-completed while still in `active/`", it fired **58 times** — every
one a legitimate T-193 partial-complete task (agent ACs pass, human ACs pending,
stays in `active/` by design, `owner: human`; all 58 verified). A permanently-red
check is an unread check — T-2818's fatigue lesson from the other direction, and
it would have shipped that way had the real corpus not been run before the
fixtures were written.

The discriminator that works is `date_finished`: the finalize block stamps it
*before* it branches on `PARTIAL_COMPLETE`, so a partial-complete task always
carries a date and a latched one structurally cannot. Keyed on the date rather
than `owner: human` deliberately — owner is a convention a task could legitimately
vary; the date is produced by the code path itself. The 58 are **counted and
reported**, never fired on, so a green is never ambiguous between "none" and "the
filter ate them".

### Honest variance on AC 5

The AC says "red against the current tree (which has two such tasks), green once
they are resolved". The order actually ran the other way: T-2831 and T-2832 were
unstuck first (they were blocking commits), so by the time the check existed the
live tree was already clean. The red leg is therefore proven against the two files
**as they stood at commit `389488a65`**, extracted straight from git rather than a
synthetic mutant — the same technique T-2831 used, and pinned to an explicit sha
for the same reason (a HEAD-relative fixture silently starts reading the fixed
file and passes for the wrong reason).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The exact failure point in the finalize path is identified by reading the code — which step sets `status`, which sets `date_finished`, which moves the file, and what stops between them
- [x] The finding names whether this is a vendored defect (G-062, route upstream) or locally fixable, with evidence for the call
- [x] T-2831 and T-2832 reach a consistent finalized state — either fully finalized, or explicitly left and documented, never silently half-done
- [x] A check exists that detects a task whose `status: work-completed` while it is still in `.tasks/active/` — the state the T-2290 canary is structurally blind to because it scans `completed/` only
- [x] That check is proven load-bearing: red against the current tree (which has two such tasks), green once they are resolved

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

# The new check is green against the live tree
bash scripts/check-stranded-finalized-tasks.sh --no-heartbeat
# ...and its own fixtures pass, so it was not weakened to go green
out=$(bash tests/stranded-finalized-check-fixtures.sh 2>&1); echo "$out" | grep -q "0 failed"
# RED (load-bearing): it fires on T-2831 as it stood at the pinned pre-repair sha.
# Pinned to an explicit sha, NOT HEAD — a HEAD-relative fixture silently starts
# reading the REPAIRED file and the red leg passes for the wrong reason (T-2831).
rm -rf .tmp-stranded-fixture && mkdir -p .tmp-stranded-fixture/active && git show 389488a65:.tasks/active/T-2831-repair-t-2830-verification-block-landed-.md > .tmp-stranded-fixture/active/a.md && ! bash scripts/check-stranded-finalized-tasks.sh --no-heartbeat --quiet --tasks-dir .tmp-stranded-fixture
rm -rf .tmp-stranded-fixture
# The VENDORED file was NOT patched — G-062, the fix went upstream instead
test -z "$(git status --porcelain .agentic-framework/agents/task-create/update-task.sh)"
# Both previously-stuck tasks are now out of active/ and in completed/
test -f .tasks/completed/T-2831-repair-t-2830-verification-block-landed-.md
test -f .tasks/completed/T-2832-register-vendored-framework-divergences-.md
test ! -f .tasks/active/T-2831-repair-t-2830-verification-block-landed-.md
test ! -f .tasks/active/T-2832-register-vendored-framework-divergences-.md
# The allowlist is git-tracked (T-2681) and carries no acknowledgement entries
git ls-files --error-unmatch .context/checks/stranded-finalized-allowlist
test -z "$(grep -v '^#' .context/checks/stranded-finalized-allowlist | grep -v '^[[:space:]]*$' || true)"
# The check is a guard-layer member, so CI runs it on every push
out=$(bash scripts/run-guard-layer.sh --list 2>&1); echo "$out" | grep -q "check-stranded-finalized-tasks.sh"
# CLAUDE.md documents the defect and the check
grep -q "Stranded-finalized task check (T-2833" CLAUDE.md

## RCA

**Symptom:** `fw task update <id> --status work-completed` wrote `status:
work-completed` but left `date_finished: null` and never moved the file out of
`.tasks/active/`. Re-running it changed nothing. The resulting half-state
deadlocked commits: P-002 refuses a commit while focus is on a completed task and
the T-1730 focus-drift gate refuses one while focus is on any other task, so two
commits (`2dabb2da5`, `9798e715a`) had to be attributed to unrelated tasks.

**Root cause:** the status write (`update-task.sh` ~1681) and the finalize block
(~1902) are not atomic, and the finalize block is guarded on `[ "$OLD_STATUS" !=
"work-completed" ]` where `$OLD_STATUS` was read from the file at startup. Any
interruption between the two writes — SIGPIPE from piping the output through
`head` being the likely trigger, since the reviewer verdict prints immediately
before the status write — commits the status to disk. Every later invocation then
reads `OLD_STATUS=work-completed`, the guard is false, and the entire finalize
block is skipped silently. `date_finished` is written at exactly one site inside
that block, so the field becomes unreachable. It is a latch, not a transient.

**Why structurally allowed:** three failures compounding. (1) The guard reads
remembered state rather than observed state, so it cannot distinguish "already
finalized" from "status written but finalize never ran". (2) Skipping the block
produces no output at all — the Directive #2 shape, where a no-op and a success
are indistinguishable. (3) The T-2290 task-finalization canary scans
`.tasks/completed/` only, so a task that never leaves `active/` is outside its
corpus by construction; and the partial-complete re-check branch (~1395) then
moves the file WITHOUT stamping the date, landing it in exactly that canary's
soft, non-firing class. The system self-heals into its own blind spot.

**Prevention:** `scripts/check-stranded-finalized-tasks.sh` fires on any task in
`.tasks/active/` declaring `status: work-completed` with no `date_finished` — the
state no healthy code path produces, and the one the T-2290 canary cannot see. It
is a guard-layer member (`# guard-layer: source`), so CI runs it on every push.
Fixtures: `tests/stranded-finalized-check-fixtures.sh`, 29 assertions, red leg
proven against the real pre-repair files at commit `389488a65`. The fix itself is
vendored and was routed upstream per G-062 (`framework:pickup` offset 32) rather
than patched locally, so the detection is what survives a re-vendor.

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

## Reviewer Verdict (v1.5)

- **Scan ID:** R-c7525e38
- **Timestamp:** 2026-08-23T19:09:47Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **destructive-action** (high) — Destructive operation in verification or AC
     - matched: `rm -rf`

### 2026-08-23T19:09:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
