---
id: T-3036
name: "Run substrate-smoke prover and report claim/claim-transfer stage verdicts (AEF
  T-3398 drift check)"
description: >
  AEF's arc-020 circuit design reuses channel claim / claim_transfer / hub start-status
  / fleet verbs as building blocks and asked whether those still behave as their design
  expects. The affirmative prover exists (scripts/substrate-smoke.sh: create - post
  - claim - claim-transfer - worker-loop - verify-clean plus four regression gates)
  but its canary has never been installed in /etc/cron.d (10 consecutive audit FAIL
  recurrences), so no standing evidence exists. Run the prover, report the stage-by-stage
  verdict to AEF, and close the standing audit gap by installing the canary or recording
  why not.

status: work-completed
workflow_type: test
owner: agent
horizon: null
tags: [arc:arc-009]
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
created: 2026-09-21T08:58:02Z
last_update: 2026-09-21T14:36:08Z
date_finished: 2026-09-21T14:36:08Z
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
  - ts: '2026-09-21T14:30:00Z'
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
  - ts: '2026-09-21T14:30:43Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 1
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=1 
      (workflow:test); effort=8 (lines=204,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3036: Run substrate-smoke prover and report claim/claim-transfer stage verdicts (AEF T-3398 drift check)

## Context

AEF's arc-020 circuit design reuses `channel claim` / `claim-transfer` / hub
start-status / fleet verbs as building blocks and asked whether those still
behave as its design expects. `scripts/substrate-smoke.sh` (T-2151) is the
affirmative prover for exactly that composition, but its canary has never been
installed to `/etc/cron.d` (audit FAIL, 10 consecutive recurrences), so **no
standing evidence exists** — the prover has never run on a schedule. This task
produces the missing evidence once, by hand, and reports it to the peer who
asked.

Reachability is prechecked separately BEFORE invoking the prover. This is
load-bearing, not ceremony: T-2696 measured that smoke's `create` stage on an
unreachable hub is a `stage_fail`, so smoke exits **1** — "substrate broken" —
on a host whose hub is merely down. Without the precheck a verdict of 1 is
ambiguous between "the composition regressed" and "nothing was listening", and
reporting the former when it was the latter is precisely the kind of confident
wrong answer that would mislead the peer who asked.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Hub reachability prechecked and its result recorded BEFORE the prover runs, so the exit code is interpretable (T-2696 remap: unreachable ⇒ tooling, not a stage regression)
- [x] `scripts/substrate-smoke.sh` executed against the live hub and its **stage-by-stage** verdict recorded verbatim in this task — every stage named, not an aggregate pass/fail
- [x] The claim / claim-transfer stages AEF specifically asked about are reported individually, since those are the verbs arc-020 reuses
- [x] Verdict filed to the `framework:pickup` topic and **proven received by reading it back from the hub at a recorded offset** — not by trusting the `delivered` response (T-2876: delivered means queued, not received)
- [x] The standing canary-install audit gap is resolved in this task by recording WHY it was not installed here plus the exact operator command, since installing to `/etc/cron.d` needs sudo and is operator action this session must not take

**Measured evidence (2026-09-21).**

*Reachability precheck, run BEFORE the prover.* `termlink channel list` -> rc=0.
This is load-bearing, not ceremony: T-2696 measured that smoke's `create` stage on
an unreachable hub is a `stage_fail`, so the prover exits **1** on a host whose hub
is merely down. Without the precheck an exit of 1 cannot be told apart from a real
regression, and reporting "substrate broken" to a peer when the real answer was
"nothing was listening" is the confident-wrong-answer failure this project keeps
finding in its own guards.

*Prover result.* `scripts/substrate-smoke.sh --json` -> rc=0, `ok:true`,
`stages_failed: []`. All ten stages passed, named individually because an aggregate
verdict does not answer what AEF asked:

    1.  create
    2.  post (offset=0)
    3.  claim                        <-- arc-020 building block
    4.  transfer                     <-- arc-020 building block (claim-transfer)
    5.  worker-loop (adopted-claim path)
    6.  verify-clean (active=0 expired=0)
    7.  drain-demo (work-stealing race, exclusive delivery)
    8.  handoff-demo (CLAIM_NOT_OWNED ownership gates)
    9.  lease-expiry-demo (worker-death auto-reclaim, lapsed-owner lockout)
    10. resilience-demo (hub-blip queue absorb + exactly-once drain)

Stage 8 is worth more than its bare pass: it exercises the CLAIM_NOT_OWNED gate,
proving `transfer` still REFUSES what it is supposed to refuse. A transfer that had
lost its ownership check would pass stage 4 and fail stage 8.

*The qualification, which is load-bearing.* The running binary is **termlink
0.11.1766**; this tree's VERSION is **0.11.2062** — about 296 commits newer. So the
verdict covers 0.11.1766 and NOT current HEAD. This is the T-2181 stale-binary class
(preflight Check 4). The green is real; it is green about a specific binary, and the
report says so.

*Filed and proven received.* Posted to `framework:pickup` at **offset 139**. The post
response itself said `[delivered-unconfirmed: hub accepted it; no consumer receipt
yet]`, which is exactly why acceptance required a read-back: `channel subscribe
--cursor 139 --limit 1` returned 4477 bytes carrying `pickup_id: P-TL-3036`, the
claim/transfer building-block lines, and the 0.11.1766 qualification. Per T-2876,
`delivered` means queued, not received — the read-back is the evidence, the response
is not.

*Why there was no standing evidence to report, and why that is not fixed here.* The
prover's canary (`.context/cron/substrate-smoke-canary.crontab`, T-2696) has NEVER
been installed to `/etc/cron.d`; our audit has recorded that FAIL for ten consecutive
runs. So when AEF asked, the truthful answer was "nobody knows" — an un-executed
prover is not evidence. This task produces the one missing data point by hand; it
does **not** make the evidence standing, and the next person to ask will be in the
same position unless the canary is installed. Same shape as T-2683 (static checks
nothing ran) and T-2686 (a parity test failing undetected since 2026-08-12).

Installing it requires `sudo` to `/etc/cron.d`, which is operator action this session
must not take. Exact command, for the operator:

    sudo cp /opt/termlink/.context/cron/substrate-smoke-canary.crontab /etc/cron.d/termlink-substrate-smoke-canary && sudo systemctl reload cron

This is deliberately NOT also filed as a Human AC here: the daily audit already names
it every run, and duplicating a signal that already fires daily into a review queue
that T-2940 measured at 57 items waiting over 30 days would add noise, not signal.

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

# T-3036 verification. Rehearsed under `bash -c 'set -eo pipefail; <line>'` (T-2743)
# and each confirmed able to go RED with a wrong sentinel before being committed.
# Safe redirect form throughout, never `cmd | grep -q` (L-387 SIGPIPE).
timeout 30 termlink channel subscribe framework:pickup --cursor 139 --limit 1 > /tmp/.t3036v 2>&1 && grep -q 'pickup_id: P-TL-3036' /tmp/.t3036v
grep -q 'arc-020 building block' /tmp/.t3036v
grep -q '0.11.1766' /tmp/.t3036v
grep -rq 'handoff-demo (CLAIM_NOT_OWNED ownership gates)' .tasks/
grep -rq 'sudo cp /opt/termlink/.context/cron/substrate-smoke-canary.crontab' .tasks/
test -f scripts/substrate-smoke.sh


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

- date: 2026-09-21
  task: T-3036
  note: >
    Ran the substrate-smoke prover for AEF's arc-020 drift check: 10/10 stages
    PASS including claim and claim-transfer, and stage 8 (handoff-demo) proving
    transfer still REFUSES via the CLAIM_NOT_OWNED gate — a transfer that had lost
    its ownership check would pass stage 4 and fail stage 8. Filed at
    framework:pickup offset 139 and proven by hub read-back, not by the
    `delivered` response (T-2876).
- date: 2026-09-21
  task: T-3036
  note: >
    Two things this run refused to overstate. (1) The verdict is about binary
    0.11.1766 while the tree is VERSION 0.11.2062, ~296 commits newer — the green
    covers the installed binary, not HEAD (T-2181 Check 4 class), and the report
    says so rather than letting a peer read it as broader than it is. (2) The
    prover's canary has never been installed, so this is a one-shot data point and
    NOT standing evidence; the honest answer to "what did the schedule show" is
    still "nobody knows". An un-executed prover is not a guard — same shape as
    T-2683 and T-2686.


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

### 2026-09-21T08:58:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3036-run-substrate-smoke-prover-and-report-cl.md
- **Context:** Initial task creation

### 2026-09-21T14:31:40Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-139facdc
- **Timestamp:** 2026-09-21T14:36:09Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#5 (Agent)** — The standing canary-install audit gap is resolved in this task by recording WHY it was not installed here plus the exact operator command, since installing to `/etc/cron.d` needs sudo and is operator 
  - **AC-verify-mismatch** (narrow, heuristic) — `path=etc/cron.d in: The standing canary-install audit gap is resolved in this task by recording WHY it was not installed here plus the exact operator command, since insta`

### 2026-09-21T14:36:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
