---
id: T-2887
name: "Housekeeping 2026-09-03: re-stamp the enforcement baseline after proving the
  drift is additive-only"
description: >
  fw doctor FAILs on 'Enforcement baseline CHANGED', a single opaque bit that cannot
  name what changed (T-2909). Diffed it: the stored sha matches .claude/settings.json
  at commit 96c160467 exactly, and the delta since is 8 hooks ADDED, 0 removed. Re-stamp
  with the delta recorded in the commit, so the stamp is auditable rather than laundered.

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
created: 2026-09-03T09:56:13Z
last_update: 2026-09-03T09:57:43Z
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
  - ts: '2026-09-03T09:57:43Z'
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
---

# T-2887: Housekeeping 2026-09-03: re-stamp the enforcement baseline after proving the drift is additive-only

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

`fw doctor` FAILed with `Enforcement baseline CHANGED — settings.json hooks differ from
baseline`. That message is a **single opaque bit**: it reports that the hook set moved
but cannot say which way. `docs/reports/T-2909-enforcement-baseline-laundering.md` (an
in-progress exploration on the `charter-review-2026-0814` branch) documents precisely
this — and that the prescribed remedy, `fw enforcement baseline`, *launders* the loss by
overwriting the record without ever naming what left. So the remedy was NOT run until the
delta had been established independently.

**Measured.** The stored baseline `5ba1d36e8d39eae6` recomputes byte-identically from
`.claude/settings.json` as it stood at commit `96c160467` (T-1638, 2026-05-15) — proving
which file the stamp was taken from. Diffing that against the live file:

```
REMOVED since baseline commit:   0
ADDED since baseline commit:     9
  PreToolUse  Write|Edit                             check-active-completed-dup
  PreToolUse  Write|Edit                             check-arc-id
  PreToolUse  Write|Edit                             check-heredoc-cmd-sub
  PreToolUse  Write|Edit                             check-inception-decisions
  PreToolUse  Write|Edit                             check-inception-schema
  PreToolUse  Write|Edit                             check-onboarding-gate
  PreToolUse  mcp__termlink__termlink_channel_post   check-rail-mcp-label
  PostToolUse Write|Edit                             check-settings-edit
  SessionStart startup                               post-compact-resume
```

**Zero removals.** The drift is strictly additive — enforcement was strengthened and the
baseline simply was never re-stamped after those nine bindings landed. This is the one
condition under which re-stamping is legitimate rather than laundering, and it is why the
delta is recorded here and in the commit message: the stamp is auditable after the fact.

**The first measurement was wrong twice, and the strict verification is what caught it.**
The initial diff matched only tokens containing `check-` / `gate` / `guard`, which (a)
under-counted the additions at 8 — `post-compact-resume` on `SessionStart/startup` carries
none of those tokens — and (b) was structurally blind to a path migration. Verification
line 2, comparing whole command strings, failed and reported 18 removals. Those 18 turned
out to be every pre-existing hook re-pathed from a hardcoded absolute
`/opt/termlink/.agentic-framework/bin/fw` to `${CLAUDE_PROJECT_DIR}/...` — each with an
exact counterpart in the added set. Normalising only that prefix gives 0 removed / 9 added.
Recorded because the near-miss is the point: a laxer verification line would have
confirmed the conclusion I had already reached, which is the failure mode this task is
about.

**Same fact, second surface.** Eight of the nine are exactly the eight `fw doctor` reports
as `missing` in all five linked worktrees. The worktrees branched before the gates landed,
so "baseline drifted" and "worktrees are missing 8 hooks" are one event seen from two
sides, not two problems.

**Bearing on T-2886.** The path migration means `.claude/settings.json` already uses the
portable `${CLAUDE_PROJECT_DIR}` idiom, while `.mcp.json` — and the framework's own shipped
`framework-mcp.mcp-fragment.json` — still use a bare cwd-relative path. The framework
therefore already has the correct idiom in one file and not the other, which is a
materially stronger form of the T-2886 argument than the filing at offset 85 made.

**Left deliberately undone:** the worktree hook drift itself. `fw upgrade <worktree-path>`
would refresh them, but four of the five carry other sessions' in-flight branches, and
T-2881 already established that reaching into a worktree from here is the wrong move.
Whoever next works each branch should run it there.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The baseline delta is established by measurement, not assumption: the stored sha is
      shown to recompute from a named commit's `settings.json`, and the add/remove sets are
      enumerated
      — stored `5ba1d36e8d39eae6` recomputes from `96c160467`; delta 0 removed / 9 added
- [x] The removal set is empty — re-stamping is only performed because nothing was lost
      — verified by whole-command comparison with only the fw-path prefix normalised, and
        mutation-tested: deleting `check-tier0` makes the check fire
- [x] `fw enforcement baseline` re-stamped, and `fw doctor` reports `Enforcement baseline
      intact`
      — new hash `9bc136177d40fe76`
- [ ] The enumerated delta is recorded in the task and the commit message, so the stamp can
      be audited later rather than being an opaque overwrite

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
#
# T-2887 note: that is exactly what recurred here, with eight more hooks. L-398
# exists and is even printed in this very template, so the knowledge was present
# and still did not bind — nothing MAKES a hook-adding task re-stamp. The learning
# has now failed to prevent its own restatement twice (T-1886, T-2887), which is
# the T-2746 argument for a structural check over more documentation.

# 1. The stored baseline matches the live settings.json (the re-stamp actually landed).
test "$(cat .context/project/enforcement-baseline.sha256)" = "$(python3 -c "import json,hashlib;d=json.load(open('.claude/settings.json'));print(hashlib.sha256(json.dumps(d.get('hooks',{}),sort_keys=True).encode()).hexdigest())")"
# 2. THE LOAD-BEARING ONE: nothing present at the baseline commit was removed. This is the
#    only condition under which re-stamping is legitimate; if a future edit drops a hook,
#    this line fires and the stamp must not be refreshed until that is explained.
#    ONLY the fw-binary path prefix is normalised (absolute vs ${CLAUDE_PROJECT_DIR}) — the
#    event, matcher and hook verb are all compared literally, so a genuine removal fires.
#    Comparing raw command strings instead reports all 18 pre-existing hooks as removed on
#    the path migration alone; that is a false positive, not extra strictness.
python3 -c "import json,re,subprocess,sys;n=lambda c:re.sub(r'^(\S*?|\\\$\{CLAUDE_PROJECT_DIR\})/\.agentic-framework/bin/fw\b','FW',c).strip();f=lambda t:{(e,g.get('matcher',''),n(h.get('command',''))) for e,gs in json.loads(t).get('hooks',{}).items() for g in gs for h in g.get('hooks',[])};old=f(subprocess.run(['git','show','96c160467:.claude/settings.json'],capture_output=True,text=True).stdout);new=f(open('.claude/settings.json').read());sys.exit(0 if not (old-new) else 1)"
# 3. fw doctor agrees the baseline is intact (doctor exits non-zero on unrelated warnings,
#    so its rc is captured rather than gating the line — see the pipefail notes above).
rc=0; .agentic-framework/bin/fw doctor --quick > /tmp/.t2887-doctor.out 2>&1 || rc=$?; grep -q "Enforcement baseline intact" /tmp/.t2887-doctor.out

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

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-09-03T09:56:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2887-housekeeping-2026-09-03-re-stamp-the-enf.md
- **Context:** Initial task creation

### 2026-09-03T09:57:43Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
