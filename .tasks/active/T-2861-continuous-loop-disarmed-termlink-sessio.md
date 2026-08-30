---
id: T-2861
name: "Continuous loop disarmed: termlink sessions launch unsupervised, and fw doctor's claude-fw remediation would break the T-2854 router"
description: >
  Continuous loop disarmed: termlink sessions launch unsupervised, and fw doctor's claude-fw remediation would break the T-2854 router

status: work-completed
workflow_type: build
owner: human
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
created: 2026-08-30T18:36:31Z
last_update: 2026-08-30T18:41:55Z
date_finished: 2026-08-30T18:41:55Z
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

# T-2861: Continuous loop disarmed: termlink sessions launch unsupervised, and fw doctor's claude-fw remediation would break the T-2854 router

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

The arc-012 continuous-run loop (budget-critical → auto-handover →
`.restart-requested` → `claude-fw` restart → directive re-injection) has never
fired in this repo. The machinery is complete and correct. The loop is disarmed
for one reason, and misdiagnosed for a second.

**1. Disarmed.** Termlink sessions are launched as plain `claude`, so
`FW_CLAUDE_FW_SUPERVISED` is never exported and nothing consumes the signal the
budget gate writes. The proof is an artefact, not an inference:
`.context/working/.restart-requested` (`2026-08-30T10:16:55Z`, `tokens: 295345`,
session `S-2026-0827-2142`) was still on disk ten hours later. `claude-fw`
removes that file on **every** exit — fresh (restart) or stale (ignore) — so its
survival proves no wrapper has exited in this repo since it was written. `ps`
agrees: claude-fw wrappers are live for 999-AEF, 002-CPN and 050-email-archive,
and none is rooted at /opt/termlink. `.compact-log` records the human
consequence: `[auto]` handovers at 10:14:37 and 10:15:49, the signal at 10:16:55,
then a `[manual]` /compact at 10:19:03 — the operator stepping in because the
loop did not.

**2. Misdiagnosed.** `fw doctor`'s T-2501 drift check (`bin/fw:2369-2385`)
compares `command -v claude-fw` against `$FRAMEWORK_ROOT/bin/claude-fw`. Since
T-2854 the on-PATH artefact is the **router** (3707 b, `claude-fw-router`) and
the repo artefact is the **wrapper** (14971 b). They are *supposed* to differ, so
the check reports drift permanently on a correct install — and its printed
remediation, `rm -f <router> && cp <wrapper> <router-path>`, would delete the
router and reinstate exactly the fixed on-PATH copy T-2854 existed to remove. An
operator following `fw doctor` to fix their loop breaks their launcher and still
has no loop. Sibling-not-migrated: T-2854 shipped the router; T-2501's check was
never updated to know about it.

Both files are vendored (G-062), so neither is patched here — filed upstream.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Root cause established from artefact evidence, not inference: the unconsumed `.restart-requested` plus the absence of any claude-fw process rooted at /opt/termlink proves the sessions run unsupervised
- [x] `fw doctor`'s claude-fw drift WARN confirmed to be a permanent false positive on a correct post-T-2854 install (router vs wrapper), with both file identities and byte sizes recorded
- [x] The WARN's printed remediation confirmed harmful — shown to delete the T-2854 router and reinstate the pinned on-PATH copy that T-2854 existed to remove
- [x] The stale inert `.restart-requested` from session S-2026-0827-2142 cleared, so it cannot later be misread as a live signal
- [x] Both defects filed upstream to `framework:pickup` over termlink (vendored — G-062 forbids a local patch), with file:line citations
- [x] The operator action that actually arms the loop routed to the approvals route, with the review page fetched and confirmed to render before the link is printed

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

- [ ] [REVIEW] The next termlink session is launched under `claude-fw`, so the continuous-run loop can actually fire

  **Steps:**
  1. Let this session finish and close it.
  2. Start the next one from the project root with the wrapper, not bare `claude`:
     `claude-fw -c`
  3. Inside that session, confirm supervision is armed:
     `cd /opt/termlink && .agentic-framework/bin/fw doctor 2>&1 | grep -i -e supervis -e drift`

  **Expected:** the first line reads
  `OK  Session supervised by claude-fw — budget auto-restart armed`.
  A second line, `WARN  Installed claude-fw drifted from repo source`, is a KNOWN
  FALSE POSITIVE on a correct install — filed upstream at `framework:pickup`
  offset 75. **Do not run its `Refresh:` line**: it deletes the T-2854 router and
  reinstates the pinned copy that router replaced, which makes future
  `fw upgrade`s unable to refresh on-PATH behaviour, and does not arm the loop.

  **If not:** if the line still says `Unsupervised session`, the router did not
  resolve this project. Check that `.agentic-framework/bin/claude-fw` exists and
  is executable, and that `command -v claude-fw` reports
  `/root/.local/bin/claude-fw`. Nothing needs reinstalling — the router resolves
  per-invocation from `$PWD`, so launching from a directory outside the project
  is the usual cause.

  **Scope note — this arms interactive sessions only.** A background-job session
  (this one) is launched as plain `claude` by the job harness, with no wrapper in
  the process tree, so the loop can never fire for it regardless of what is on
  PATH. That is a separate gap, not something this AC fixes.

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

# --- T-2861 ---
# The on-PATH artefact is the T-2854 router, not a copy of the wrapper.
grep -q "claude-fw-router" /root/.local/bin/claude-fw
# The wrapper (what the router execs) is what exports the supervision marker.
grep -q "export FW_CLAUDE_FW_SUPERVISED=1" .agentic-framework/bin/claude-fw
# The drift check still byte-compares the on-PATH file against the WRAPPER path —
# this is the defect. If upstream fixes it, this line fails and the task is re-read.
grep -q '_cfw_src="$FRAMEWORK_ROOT/bin/claude-fw"' .agentic-framework/bin/fw
# The harmful remediation is still the one printed (same re-read trigger as above).
grep -q 'Refresh: rm -f' .agentic-framework/bin/fw
# The stale S-2026-0827-2142 signal is gone. Written so a NEW signal from a later
# session does not fail this line — only the specific stale one is asserted absent.
! grep -q "S-2026-0827-2142" .context/working/.restart-requested 2>/dev/null
# The upstream filing landed on the topic.
timeout 60 termlink channel info framework:pickup --json > /tmp/.t2861-pickup.out 2>&1 && grep -q '"count"' /tmp/.t2861-pickup.out

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

**Recommendation:** GO — launch termlink sessions under `claude-fw`.

**Rationale:** The continuous-run loop is not broken; it is unarmed. Every link
in the chain exists and is correct — the budget gate detects critical and writes
the signal (it did, at 295,345 tokens), the T-2373 terminator ends the foreground
claude, the wrapper restarts `claude -c`, and the T-2376 sentinel lets the
SessionStart hook tell an auto-restart continuation from a cold start. Only link
zero is missing: nothing exports `FW_CLAUDE_FW_SUPERVISED`, because the sessions
are launched as bare `claude`. One launch command changes that. Nothing needs
installing or repairing first, which is why this is GO rather than a build task.

**Evidence:**
- `.restart-requested` written `2026-08-30T10:16:55Z` at 295,345 tokens survived
  ten hours on disk. `claude-fw` unlinks it on every exit, fresh or stale — so no
  wrapper has exited in this repo since it was written.
- `ps` shows claude-fw wrappers live for 999-AEF, 002-CPN and 050-email-archive;
  none rooted at /opt/termlink.
- `.compact-log`: `[auto]` handovers 10:14:37 and 10:15:49, signal 10:16:55, then
  `[manual]` /compact 10:19:03. The operator did by hand what the loop exists to do.
- `fw doctor` in this session reports both `Unsupervised session` and the drift
  WARN, live.
- The drift WARN is a false positive: on-PATH is `claude-fw-router` (3707 b),
  repo is the wrapper (14971 b); `bin/fw:2373` byte-compares them, so it can never
  match on a correct post-T-2854 install.

**Scope caveat — this is GO on arming interactive sessions, and nothing more.**
It does not arm background-job sessions: the job harness launches plain `claude`
with no wrapper in the process tree, so the loop cannot fire for them whatever is
on PATH. It also does not fix the drift WARN, which is vendored and filed upstream
at `framework:pickup` offset 75 — the WARN will keep firing, and its `Refresh:`
line must not be run.

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

### 2026-08-30T18:36:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2861-continuous-loop-disarmed-termlink-sessio.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-ed1fbd10
- **Timestamp:** 2026-08-30T18:41:56Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **destructive-action** (high) — Destructive operation in verification or AC
     - matched: `rm -f`

### 2026-08-30T18:41:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
