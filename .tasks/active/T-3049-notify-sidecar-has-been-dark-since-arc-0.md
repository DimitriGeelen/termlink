---
id: T-3049
name: "Notify sidecar has been dark since arc-003 closed - restore the deterministic
  wake rail"
description: >
  arc-003 reliable-comms closed 2026-07-02 declaring 'no silent loss'. The deterministic
  notify sidecar (T-2294, V3a) that the claim rests on last wrote a heartbeat 2026-07-01
  - the day before - and has not run since. Measured 2026-09-22: no notify-sidecar
  process alive, every file under ~/.termlink/notify/ is a July test agent (s3g/s3probe/s3smoke),
  no cron entry among 24 crontabs, no skill to start it, no canary to notice it is
  dark. Neither is the arc-004 push-waker running and there is no be-reachable state
  file, so this host is currently unreachable by peers through ANY rail while the
  arc records the capability as shipped. This is the G-069 shipped-not-live class
  in the one rail agent-to-agent comms depends on. Deliverable: prove the script still
  functions after 82 days of drift via its own hermetic harness, then bring the rail
  live for this agent and measure both the CLEAR and DEAF verdicts against a real
  listener. Detection so it cannot silently die again is deliberately a separate task
  - one task, one deliverable.

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
created: 2026-09-21T22:06:20Z
last_update: 2026-09-21T22:07:25Z
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
  - ts: '2026-09-21T22:07:25Z'
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

# T-3049: Notify sidecar has been dark since arc-003 closed - restore the deterministic wake rail

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The shipped hermetic harness `scripts/test-notify-sidecar.sh` passes in full against
      the current tree, establishing that an 82-day-dark script still functions before any
      of it is relied on — measured, not assumed
- [x] A notify sidecar is RUNNING for this agent: a live process, and a `.heartbeat` under
      `~/.termlink/notify/` whose age is under the deaf threshold rather than 82 days
- [x] `notify-check.sh` is measured returning BOTH live verdicts against that real listener:
      `0` CLEAR while the heartbeat is fresh, and `3` DEAF once it is stale. A rail that can
      only be observed passing is not evidence it can detect its own failure
- [x] The start path is recorded in the task so it is reproducible by the next session, and
      the ~82-day gap between the arc's "shipped" claim and any listener actually running is
      written into the RCA — the finding is the dark rail, not just the restart

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

# AC1 — the 82-day-dark script still functions. Hermetic (TERMLINK_NOTIFY_TEST_UNREAD),
# no hub needed, and it covers BOTH DEAF paths (T10 stale, T11 missing).
bash scripts/test-notify-sidecar.sh > /tmp/.t3049-h 2>&1 && grep -qE "0 fail" /tmp/.t3049-h

# AC2 — a listener is genuinely alive: heartbeat age in seconds, not 82 days.
# This asserts LIVE HOST STATE by design, which is unusual for this repo (PL-213)
# and is the point: the task's whole finding is that the rail was shipped and dark.
# On a host with no sidecar running this line SHOULD fail.
python3 -c "import time,pathlib; p=pathlib.Path.home()/\".termlink/notify/claude-termlink.heartbeat\"; hb=int(p.read_text().strip()); age=time.time()-hb/1000.0; assert age < 120, (\"heartbeat stale\", age)"

# AC3 (live half) — the check returns a LIVE verdict (0 CLEAR or 10 MAIL), not 3 DEAF.
# The DEAF half was measured by stopping the real listener and is recorded in the RCA;
# it is not re-run here because asserting it would require killing the rail this task
# exists to keep running.
rc=0; bash scripts/notify-check.sh --agent-id claude-termlink --deaf-after 45 > /tmp/.t3049-c 2>&1 || rc=$?; test "$rc" = "0" -o "$rc" = "10"

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

**Symptom:** arc-003 closed 2026-07-02 recording *"reliable cross-agent comms shipped
… no silent loss."* Measured 2026-09-22: no notify sidecar process alive, every file
under `~/.termlink/notify/` a July test agent, no cron entry among 24 crontabs, no
skill to start it, no canary to notice. The arc-004 push-waker was not running either
and no `be-reachable` state file existed, so this host was unreachable by peers
through **any** rail while the register recorded the capability as shipped.

**Root cause:** the sidecar is a long-lived process with no launcher. Nothing starts
it at boot, at session open, or on a schedule. Its last heartbeat is 2026-07-01 — the
day BEFORE the arc closed — so it was very likely never running outside the slice's
own test runs. "Shipped" meant the code merged and the tests passed.

**Why structurally allowed:** this is G-069 (shipped ≠ live) in the comms rail, and
the framework had the lesson already — PL-168, surfaced by `fw work-on` when this task
started: *"Canary scripts without a trigger are not prevention — they are dormant
tooling."* The repo has eighteen cron canaries and a T-2480 live-probe gate precisely
for this, and none of them watch the notify rail. A capability nothing executes and
nothing watches decays to zero silently, and the arc's closing claim keeps asserting
otherwise.

**What the dark rail cost, measured:** switching the ears on surfaced **69 pending
DMs** across 10 topics. Roughly 30 are July test residue (`deadbeefdeadbeef`, `s3t*`,
self-to-self), but the rest sit on real peer identities. On the largest,
`dm:9219671e28054458:d1993c2c3ec44c94`, the peer posted **27 messages**, we posted 4,
and `receipts: []` is **empty** — no acknowledgement has ever been recorded. That is
the G-063 write-only-sink shape on a live peer conversation: the transport worked, and
nobody was listening.

**Prevention (distinct from the fix):** restarting the process by hand is exactly what
decayed. Detection and autostart are deliberately NOT in this task — one task, one
deliverable — and are the immediate follow-ups: a canary that fires when no listener
is alive, and a start path that does not depend on someone remembering.

**Reproducible start path:**
```
nohup setsid bash scripts/notify-sidecar.sh --agent-id claude-termlink \
    --self-fp d1993c2c3ec44c94 --interval 15 \
    >> .context/working/notify-sidecar-claude-termlink.log 2>&1 &
```
`--self-fp` must be passed explicitly on this host: `termlink whoami` is ambiguous
here (18 candidate sessions), so the sidecar's default identity resolution cannot
settle it. The fingerprint comes from `termlink agent identity`.

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

### 2026-09-21T22:06:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3049-notify-sidecar-has-been-dark-since-arc-0.md
- **Context:** Initial task creation

### 2026-09-21T22:07:25Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
