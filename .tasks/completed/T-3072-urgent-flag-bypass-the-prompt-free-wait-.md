---
id: T-3072
name: "Urgent flag: bypass the prompt-free wait and inject directly"
description: >
  Spec step 9. Operator: not yet defined, note for later. An urgent message should
  skip the wait for a free prompt and inject immediately. Needs the flag semantics
  defined before build.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-011]
components:
  - scripts/notify-injector.sh
  - scripts/be-reachable-pushwaker.sh
  - tests/notify-injector-fixtures.sh
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
created: 2026-09-22T12:54:39Z
last_update: 2026-09-23T17:01:15Z
date_finished: 2026-09-23T17:01:15Z
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
  - ts: '2026-09-22T14:57:19Z'
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
  - ts: '2026-09-22T14:57:42Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=207,acs=4)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-22T14:59:18Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (3-components); tier=2 (workflow:build); effort=8 
      (lines=207,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3072: Urgent flag: bypass the prompt-free wait and inject directly

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **Urgent shortens the WAIT, never the CHECK** (SQ-4, operator). When the queued
      head message is urgent, the injector re-probes the prompt within a bounded window
      instead of deferring on the first non-READY probe. The wait it shortens is the
      cron interval — up to 5 minutes — not the prompt-free test
- [x] **It still NEVER injects into BUSY or UNKNOWN.** After the window expires it
      defers exactly as before (exit 4). This is the criterion the others serve:
      injecting into a busy prompt is the T-2396 loss, and a bypass would silently lose
      exactly the messages most likely to be marked urgent
- [x] Urgency is read from the DECLARED priority band and is bounded by the delivered
      watermark, so an already-delivered urgent message cannot re-trigger the wait
- [x] **Non-urgent behaviour is unchanged** — one probe, immediate defer. Proven by the
      existing 29 assertions staying green, not by inspection
- [x] The threshold and window are declared and overridable (`INJECTOR_URGENT_THRESHOLD`,
      `INJECTOR_URGENT_WAIT`), not magic numbers buried in a condition
- [x] Fixtures pin: urgent waits and then injects when the prompt frees; urgent still
      defers when it never frees; non-urgent does not wait; an already-delivered urgent
      message does not trigger waiting
- [x] Mutation-proven — disabling the urgency arm reddens a named fixture, and so does
      removing the post-window defer

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

bash -n scripts/notify-injector.sh

# 34 assertions. U2 is the criterion SQ-4 exists to protect: urgent must still defer on
# BUSY. The other 29 staying green is the proof that non-urgent behaviour is unchanged.
bash tests/notify-injector-fixtures.sh

# The knobs are declared, not magic numbers buried in a condition.
grep -q 'URGENT_THRESHOLD="${INJECTOR_URGENT_THRESHOLD:-5}"' scripts/notify-injector.sh
grep -q 'URGENT_WAIT="${INJECTOR_URGENT_WAIT:-120}"' scripts/notify-injector.sh

# Urgency is bounded by the delivered watermark — an already-handed-over urgent message
# must not re-arm the wait for ever (T-3082's defect in a different hat).
grep -q 'AND offset > \$delivered' scripts/notify-injector.sh

# The BUSY arm still exits 4 unconditionally. If this line ever grows an urgent
# exemption, the bypass is back and U2 is the fixture that catches it.
grep -q 'prompt BUSY — deferring (the agent is mid-turn)' scripts/notify-injector.sh

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

### 2026-09-23 — the task title says "bypass", and the operator's answer says do not

- **What changed:** this task is literally named *"Urgent flag: bypass the prompt-free
  wait and inject directly"*, and its description says an urgent message "should skip
  the wait for a free prompt and inject immediately". SQ-4 asked what that means when
  the prompt is BUSY, and the operator's answer inverts the title: urgent shortens the
  WAIT, never the CHECK. I built the operator's answer, not the title, and the title is
  now the most misleading thing in the file — worth renaming if this is ever revisited.
- **What "the wait" actually was.** I expected to find a wait loop to shorten. There is
  none: the injector is single-shot and the wait a normal message serves is the CRON
  INTERVAL — defer now, be looked at again in up to five minutes. So urgency could not
  mean "skip a loop"; it had to mean "re-probe within a bounded window instead of
  surrendering the tick". That reframing is the whole design, and it only became
  visible by reading the code rather than the ticket.
- **Most of the slice was already built.** T-3071 shipped the priority column and
  `COALESCE(priority,0) DESC` ordering, so "urgent goes first in the queue" needed
  nothing. What remained was purely the latency, which is why this closed inside a
  budget that could not have carried T-3076.
- **The watermark had to move earlier, and that costs something.** Urgency must be
  judged against what is UNDELIVERED, or an already-handed-over urgent message re-arms
  the wait for ever — T-3082's defect in a different hat. So the hub read now happens
  before the prompt check, which means a deferred tick does one read it previously
  skipped. One read per agent per cron interval, accepted deliberately and recorded
  rather than discovered later by someone profiling it.
- **The mutation that matters:** making the BUSY arm exempt urgent — the tempting
  reading of the title — is caught by U2 with the message *"URGENT INJECTED INTO A BUSY
  PROMPT — the T-2396 loss"*. That is the criterion SQ-4 exists to protect, and it is
  now mechanically defended rather than merely agreed.
- **Cost vs estimate:** BVP 57 / cost 3.2 (hv-lc). Accurate this time — one script, one
  fixture suite, closed within a tight budget. The earlier calibration complaints were
  about tasks whose remaining work was verification; this one was a genuine build and
  the estimate held.
- **Triggered:** nothing new. Selected over the higher-value T-3076 (70) on
  executability at 724k context, declared at selection time rather than reconstructed.

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

### 2026-09-22T12:54:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3072-urgent-flag-bypass-the-prompt-free-wait-.md
- **Context:** Initial task creation

### 2026-09-22T13:10:19Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-011

### 2026-09-23T16:57:32Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-95376312
- **Timestamp:** 2026-09-23T17:01:23Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-23T17:01:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Urgent shortens the wait (re-probe within a bounded window) and never the check (still defers on BUSY/UNKNOWN), per the operator's SQ-4 decision. 34 fixtures, both arms mutation-proven — including the bypass mutation that U2 catches as the T-2396 loss.
