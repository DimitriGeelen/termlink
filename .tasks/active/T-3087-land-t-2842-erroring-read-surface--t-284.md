---
id: T-3087
name: "Land T-2842 ERRORING read surface + T-2843 EXIT-trap heartbeat from worktree-governance-canary-signal"
description: >
  Semantic merge, not a cherry-pick. Main has 0 ERRORING and 0 stderr reads in canary-status.sh
  so an erroring canary reads HEALTHY on every operator surface; main has 0 of 40
  check-scripts using an EXIT-trap heartbeat so a hung canary still writes a fresh
  one. Both fixes exist on worktree-governance-canary-signal (pushed to origin). A
  3-way apply produced 5 conflicts because both sides renamed the same concepts: main
  is_cron_scheduled/NOT_SCHED plus T-2763 worktree resolution and T-2975 firing predicate;
  branch crontab_declares/NOT_SCHEDULED plus ERRORING and stderr_size. Must reconcile
  both, not pick a side.

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
# demo_target: true               # T-2286: optional — marks task as reserved for an orchestrated demo
#                                 # worker (e.g. arc-010 HM-A dispatches via mcp__fw__work_on). When set,
#                                 # `fw work-on T-XXX` refuses unless --i-am-demo-orchestrator (CLI) or
#                                 # FW_I_AM_DEMO_ORCHESTRATOR=1 (env) is passed. Prevents the parent
#                                 # session from consuming the captured→started-work transition the demo
#                                 # worker expects to drive. Origin OBS-057.
created: 2026-09-24T18:21:42Z
last_update: 2026-09-25T21:18:42Z
date_finished:
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
bvp_scores_proposed:
  - ts: '2026-09-24T20:18:56Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-24T20:18:57Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=227,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3087: Land T-2842 ERRORING read surface + T-2843 EXIT-trap heartbeat from worktree-governance-canary-signal

## Context

### The renumber mapping — preserved here because removal destroys it

On `worktree-governance-canary-signal` these three tasks collide with three
*unrelated* tasks of the same IDs on main. The renames were **staged but never
committed**, so they exist only in that worktree's index and are NOT in the
branch pushed to origin. Removing the worktree discards them. The mapping:

| On the branch | Rename to | Main's occupant of the old ID |
|---|---|---|
| T-2690 canary stderr sink severs detection | **T-2842** | T-2690 termlink purpose review 4 |
| T-2691 heartbeat proves scheduling not completion | **T-2843** | T-2691 whoami identity auto-resolution |
| T-2692 static-check allowlists untracked | **T-2844** | T-2692 macOS binaries / no CI job |

T-2842–2845 were verified free on main (2026-09-24). A fourth file,
`T-2845-renumber-3-cross-view-task-id-collisions.md`, was untracked in that
worktree and recorded this same mapping — it is superseded by this table.

**T-2844 is a duplicate — do not import it.** Main already landed that fix as
T-2681 (`.context/checks/` tracked allowlists). Only T-2842 and T-2843 carry
content main lacks.



<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] `scripts/canary-status.sh` classifies a canary whose `<log>.stderr` has content written inside the staleness window as **ERRORING**, and ERRORING outranks FIRING / STALE / HEALTHY — a canary that could not complete its run cannot be trusted to have found or missed anything.
- [ ] ERRORING counts toward `PROBLEMS` (the verb exits 1) and is surfaced on **every** output path: the full human summary line, `--quiet`, the `Action needed:` block naming the stderr sink, and `--json` (`summary.erroring` plus per-canary `stderr_bytes`).
- [ ] When a canary is ERRORING, `latest_entry` is read from the **stderr sink**, not the firing log.
- [ ] All four of main's post-branch hardenings survive the merge unchanged and are re-asserted by fixtures: T-2763 worktree resolution (`RESOLUTION`, refuse-never-fallback), T-2975 `SCOPE_NOTE` on every path, T-2826 `-ge` firing predicate, T-2840 `is_cron_scheduled` NOT_SCHEDULED predicate.
- [ ] The branch's looser `crontab_declares` NOT_SCHEDULED variant is **deliberately not adopted**, with the reason recorded in `## Decisions` (bare-name grep vs `.<name>.log` anchor; fail-closed vs fail-open on an absent cron dir).
- [ ] Every check script carrying the uniform `if [ "$HEARTBEAT" -eq 1 ]; then touch ...; fi` block defers the touch to an `EXIT` trap, so a hung or killed run leaves the heartbeat untouched and surfaces as STALE instead of reading alive.
- [ ] No check script is left un-migrated — the migrated set covers every script matching the uniform block, not only the 24 the branch happened to touch (the "hardened in one place, siblings not migrated" divergence).
- [ ] `bash tests/canary-status-fixtures.sh` and `bash tests/canary-heartbeat-fixtures.sh` both pass, and the canary-status suite carries a **mutant** that removes the ERRORING branch and is caught by it.

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
       `bin/fw reviewer T-XXX > /tmp/.rev 2>&1 && grep -q "Overall:.*PASS" /tmp/.rev`
       added to ## Verification. NEVER `... 2>&1 | grep -q ...` — that is the shape the
       Pipefail/SIGPIPE section below forbids, and this line used to prescribe it.
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
# ── Pipefail/SIGPIPE: grepping a command's output (L-387, T-2090, T-2743, T-2738) ──
#
# THE DEFAULT — redirect to a file, then grep the file:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
#     curl -sf "$(bin/fw watchtower url)/page" -o /tmp/.out && grep -q "PAT" /tmp/.out
# Correct at any output size, and `&&` keeps the PRODUCING command's exit code in
# the verdict. Reach for this first; the alternative below is the special case.
#
# NEVER `cmd | grep -q PAT` (L-387) — why: P-011 runs each line under `set -eo
# pipefail`. When grep matches it exits and closes stdin while cmd is still
# writing, cmd takes SIGPIPE, the pipeline exits 141 — verification "fails" with
# the pattern present. Captured 4× (T-1716, T-1838, T-1862, T-1863).
#
# THE EXCEPTION — capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Valid ONLY while "$out" fits the 65536-byte pipe buffer, and it is on you to
# know that it does. Above that the form inverts and becomes the very failure
# L-387 describes: echo blocks on the full pipe, grep -q exits, echo takes
# SIGPIPE, rc=141 (T-2743 — measured on a 146,366-byte Watchtower page, 3/3 runs,
# deterministic not racy; rendered routes run 50-200KB, so anything that curls a
# page is over the line). It also discards cmd's exit code, so a 404 yields an
# empty capture that grep merely fails to match rather than a failed line.
# If you do use it: single pipe only, no intermediate tail/awk/sed stage between
# capture and grep (T-2090) — the middle stage is what `grep -q` slams its stdin
# on, and grep scans the whole captured string anyway, so the `tail -3` was
# cosmetic. `echo "$out" | grep -q PAT`, nothing between.
#
# TEST RUNNERS need a guard either way (T-2738). `set -e` is suppressed inside the
# `if` condition the gate runs each line in, so in `cmd1; cmd2` only cmd2 is the
# verdict — and the pass marker you grep for survives a partial failure: a suite
# printing "3 failed, 9 passed" satisfies `grep -q "9 passed"`, and generalising
# to `grep -qE "[0-9]+ passed"` matches the same output. Keep the exit code:
#     python3 -m pytest <file> -q > /tmp/.out 2>&1 && grep -q passed /tmp/.out
# or add the guard the exit code used to supply:
#     out=$(python3 -m pytest <file> -q 2>&1); echo "$out" | grep -q passed && ! echo "$out" | grep -q failed
#     out=$(bats <file> 2>&1); echo "$out" | grep -q '^ok 1 ' && ! echo "$out" | grep -q '^not ok'
# The close gate refuses the unguarded form. Bypass: FW_ALLOW_UNJUDGED_TEST_RUN=1.
#
# REHEARSING A LINE BY HAND DOES NOT REHEARSE THE GATE (T-2743). Your interactive
# shell has no `set -eo pipefail`. A line has returned 0 by hand and 141 under
# P-011, from the same directory, the same second. To rehearse for real:
#     bash -c 'set -eo pipefail; <your verification line>'
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

bash -n scripts/canary-status.sh
bash tests/canary-status-fixtures.sh > /tmp/.t3087-cs.out 2>&1 && grep -q ", 0 failed" /tmp/.t3087-cs.out
bash tests/canary-heartbeat-fixtures.sh > /tmp/.t3087-hb.out 2>&1 && grep -q ", 0 failed" /tmp/.t3087-hb.out
# ERRORING landed (T-2842 payload)
grep -q 'status="ERRORING"' scripts/canary-status.sh
grep -q 'stderr_bytes' scripts/canary-status.sh
grep -q 'erroring' scripts/canary-status.sh
# main's four post-branch hardenings survived the reconcile
grep -q 'is_cron_scheduled' scripts/canary-status.sh
grep -q 'RESOLUTION' scripts/canary-status.sh
grep -q 'SCOPE_NOTE' scripts/canary-status.sh
grep -q 'log_mtime" -ge "\$heartbeat_mtime' scripts/canary-status.sh
# the branch's looser variant was NOT adopted
test -f scripts/canary-status.sh && ! grep -q 'crontab_declares' scripts/canary-status.sh
# T-2843 EXIT-trap migration is complete, not partial. The second line is the
# one that matters: the OLD inline guard was a bare `if ...; then` opening a
# multi-line block, and every migrated script now carries the one-line
# `if ...; then trap _canary_hb EXIT; fi` instead. Asserting the absence of the
# touch itself would be wrong — it still exists, inside the helper.
test "$(grep -l 'trap _canary_hb EXIT' scripts/*.sh 2>/dev/null | wc -l)" -ge 31
test -z "$(grep -lE '^if \[ "\$HEARTBEAT" (-eq|=) 1 \]; then$' scripts/*.sh 2>/dev/null)"

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

## Recommendation

<!-- T-2945: same shape as inception.md's block — the gate that reads it
     (audit_inception_recommendation, lib/task-audit.sh:117) is shared, so the
     shape is copied rather than reinvented.

     REQUIRED once this task reaches partial-complete: Agent ACs done, at least
     one `### Human` AC still unticked. `lib/review.sh:205-211` (T-2421) BLOCKS
     `fw task review` emission for build/refactor/test/decommission tasks in that
     state with no substantive block here — the operator would otherwise open
     /review/<id> to a blank Recommendation card and be asked to approve a form.

     Not required while every Human AC is ticked or the task has none: the gate
     only fires on the partial-complete transition. It is here from the start so
     you write it while you still have the evidence, not when the gate refuses.

     Format (the parser wants the `**Recommendation:**` line at the start of a
     line; a leading `-` or `*` bullet is also accepted):
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence — what shipped, what was proven, what remains)
     **Evidence:**
     - Finding 1
     - Finding 2

     DEFER is for evidence gaps, not confidence gaps (CLAUDE.md §Presenting Work
     for Human Review). If the artefact is complete and you still don't want to
     commit, that is a calibration failure — recommend GO or NO-GO.
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

### 2026-09-25 — NOT_SCHEDULED: which of the two predicates survives

Both sides independently invented the same class and named it almost the same
thing, which is what produced the conflict. They are not equivalent.

- **Chose:** main's `is_cron_scheduled()` — greps the crontabs for `\.<name>\.log`,
  **fails OPEN** on an absent cron dir, and is consulted only inside the STALE
  branch (heartbeat ancient AND log never written AND nothing schedules it).
- **Why:** three independent reasons, all pointing the same way. (1) The anchor is
  the LOG FILENAME, which is what a crontab necessarily names; the branch's bare
  `grep -F <name>` also matches a canary merely *mentioned* in a comment, so it
  would silence a genuinely stale canary on a coincidence. (2) Fail direction: on
  an absent cron dir main keeps the old behaviour, while the branch's variant
  returns "not declared" for everything — mass-downgrading every real STALE canary
  to "not a problem" precisely when the check can no longer tell. A guard that
  goes quiet when it loses its footing is the vacuous-pass class (T-2831). (3)
  Placement: main only downgrades something already STALE, so the blast radius is
  the narrower one.
- **Rejected:** the branch's `crontab_declares()`. Nothing in it is unavailable in
  main's version, so adopting it would have traded strictness for nothing. Case 8
  pins the fail-open direction and Case 13 pins that the rejected predicate has not
  crept back — if it ever does, two predicates would disagree silently.

### 2026-09-25 — migrate 31 scripts, not the branch's 24

- **Chose:** apply the EXIT-trap heartbeat to every script that writes a heartbeat.
- **Why:** main gained six such scripts after the branch forked, and
  `check-addressed-posts.sh` used a fourth block shape (a local `$hb` rather than
  `$HEARTBEAT_FILE`) that the sweep's anchor did not reach. Landing only the
  branch's 24 would have reproduced the "hardened in one place, siblings not
  migrated" divergence that the guard layer exists to catch.
- **Rejected:** a literal `git apply` of the branch patch. It would have been
  cleaner to review but would have left the six newer scripts and the fourth shape
  behind, and would have missed that the heartbeat regions were byte-identical
  anyway — which is what made a surgical transform safe.

### 2026-09-25 — the chained-trap hazard

- **Chose:** for the four scripts that install their own cleanup `trap ... EXIT`
  later in the file, use a self-guarding helper armed early and chained onto that
  later trap.
- **Why:** `trap ... EXIT` REPLACES any previous EXIT trap. A naive
  `trap _canary_hb EXIT` at the heartbeat site would have been silently overwritten
  by the cleanup trap installed further down, stopping those heartbeats entirely
  and pinning four canaries permanently STALE — a worse failure than the one being
  fixed, and invisible until someone read /canaries a day later.
- **Rejected:** moving the heartbeat call into each cleanup trap only. That leaves
  an early `exit 2` (before the cleanup trap is installed) with no heartbeat at all.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-09-24T18:21:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3087-land-t-2842-erroring-read-surface--t-284.md
- **Context:** Initial task creation

### 2026-09-25T21:11:25Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
