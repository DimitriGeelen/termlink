---
id: T-2871
name: "spawn --backend background --wait reports ready for a session that is never
  listed or queryable"
description: >
  termlink spawn --backend background --wait prints 'Session <id> is ready' and exits
  0, but the session never appears in 'termlink list' and 'termlink status <id>' returns
  'Session not found' — while its command is demonstrably still running. Reported
  externally as part of the T-041 dispatch report; reproduced here on the happy path
  (root, .107, 0.11.1716), which rules out the reporter's tmux-socket/permissions
  hypothesis. Net effect: the background backend has no retrievable terminal state
  — no exit_code, no finished_at, no result.

status: started-work
workflow_type: build
owner: claude-code
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
created: 2026-08-31T18:51:26Z
last_update: '2026-09-02T06:51:56Z'
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
  - ts: '2026-08-31T18:52:40Z'
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
  - ts: '2026-09-02T06:51:56Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 1
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=0 (no-signal); F-ORCH=1 (body:hand-wired-dispatch)
    rubric_sha: e4a00f38e801
---

# T-2871: spawn --backend background --wait reports ready for a session that is never listed or queryable

## Context

Reported externally by `framework-agent-systemd` on agent-chat-arc offset 873, inside a
wider T-041 dispatch report. Their framing was that `fw termlink dispatch` fell back to
the background backend because the tmux socket was inaccessible, and that the fallback
then "printed Spawned then timed out after 30s with no registered
session/process/result/exit_code/finished_at" — reading it as a permissions artifact.

**Reproduced here on the happy path, which rules that hypothesis out.** As root on .107
with system termlink 0.11.1716, no tmux involvement, no sudo:

```
$ termlink spawn --backend background --wait --tags task:repro2 -- bash -lc 'sleep 20; exit 7'
Spawned session 'spawn-2525441' via background backend
Waiting for session to register (timeout: 30s)...
Session 'spawn-2525441' is ready          <-- claims registered
$ termlink list | grep spawn-2525441      <-- absent 2s later, sleep still running
$ termlink status spawn-2504150
Error: Status query failed: Session not found: spawn-2504150
```

`--wait` resolves to "ready" against something `list` and `status` cannot see. Either it
consults a different store than the query path, or registration is torn down immediately
after being observed. Either way the caller-visible result is the reporter's: the
background backend yields no retrievable terminal state — no `exit_code`, no
`finished_at`, no result — so a dispatcher cannot separate success from failure from
never-ran.

This matters past the one caller. `--wait` exists so a script can treat "ready" as a
synchronisation point; a "ready" that does not imply "queryable" makes the flag actively
misleading rather than merely incomplete. That is the Directive #2 shape — a confident
wrong answer rather than an error — and it is worse than the timeout the reporter saw,
because a timeout at least fails loudly.

**Deliberately not yet established:** whether the session is registered-then-reaped or
never registered; whether the tmux backend shares the defect; and whether the reporter's
30s TIMEOUT is this bug or a second one, given that their wait FAILED where ours
SUCCEEDED. Those are the first two ACs rather than assumptions baked into a fix.


## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] **Establish which of the two states is true.** Determine whether the background session is registered-then-immediately-reaped, or never registered at all (making `--wait`'s "is ready" a read of optimistic/stale state). Record the answer with the code path that produces it — the fix differs completely between the two.
- [ ] **`--wait` returning success implies queryable.** After `spawn --backend background --wait` exits 0, `termlink status <id>` resolves the session for as long as its command is running. A regression test pins this against a bounded long-running command, asserting the window between "is ready" and a successful `status` is not empty.
- [ ] **Terminal state is retrievable, or the absence is loud.** Either `exit_code` and `finished_at` become readable for a completed background session, or `spawn --backend background` states plainly at spawn time that the backend records no terminal state — so a dispatcher is never left inferring success from silence. Whichever is chosen is written down with its reason.
- [ ] **Reporter's variant addressed.** Establish whether their `--wait` TIMEOUT (ours succeeded) is this defect or a second one, and say which in a reply on the durable route they asked for.

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

## RCA

### 2026-09-02 — AC1 partial: two cases measured, NEITHER reproduces. Third case unmeasured.

Ran the AC1 discriminator on the local hub. Both completed cases contradict the report:

| case | spawn | `--wait` verdict | `termlink status <name>` immediately after |
|---|---|---|---|
| **A** long-running | `--backend background --wait -- sleep 30` | `Session 't2871a' is ready` | **resolves**, `State: ready`, `PID: 1794347` |
| **B** fast command | `--backend background --wait -- echo hello-t2871` | `Session 't2871b' is ready` | **resolves**, `State: ready`, `PID: 1795642` |

A third, independent data point from the same session: a `--shell --backend background
--wait` spawn (`t2873v`, done for T-2873) was not only queryable but successfully
**injected into and read back** afterwards.

So `--wait` returning "is ready" is NOT reporting on a session that never registered — at
least not for these three shapes. That kills the second of AC1's two hypotheses for the
cases measured: this is not optimistic/stale state, the registration is real.

**What is still unmeasured, and it is the one that matters.** The remaining hypothesis is
*registered-then-reaped*: case B's `echo` had already exited when `status` was queried, yet
`status` still reported `State: ready` with a PID. The re-query **after** the process was
definitively gone was cut off by the budget gate, so the interesting question — does the
row go stale-but-present, disappear, or keep claiming `ready` for a dead PID? — has NOT
been answered. Do not read the table above as an all-clear; read it as "hypothesis 2
eliminated for three shapes, hypothesis 1 untested".

Note case B is already suggestive: reporting `State: ready` and a live-looking PID for a
process that has exited is the same sender-side-claim-versus-receiver-truth shape as
T-2873 (`injected` without arrival) and T-2875 (`success:true` without delivery). If it
holds up, AC3's "terminal state is retrievable, or the absence is loud" is the live half
of this task.

**Next step, precisely:** spawn a fast background command, wait for the PID to be gone,
then `termlink status <name>` and `termlink status <id>` and record all three of state,
PID liveness, and whether `exit_code`/`finished_at` appear.

### The repro in `## Context` queries a different session than it spawned

Re-reading it while writing the above: it spawns **`spawn-2525441`** and then runs
`termlink status` **`spawn-2504150`**. Those are two different names. `Session not found`
is the correct answer to that query, so that line is not evidence of the defect — it is
evidence of a typo, and it is the line the Context leans on hardest.

The `termlink list | grep spawn-2525441` line is unaffected by the mismatch and remains
the real claim. But it is now the *only* surviving leg of that reproduction, and it was
never re-run.

This matters because my three passing cases all queried **by the `--name` I set**, whereas
that repro used auto-generated `spawn-<pid>` names. So there are two live hypotheses I
cannot separate without re-measuring, and they are not the two the ACs assume:
1. the defect is real and my cases missed it because they queried by explicit name;
2. the defect was always the typo plus a `list` reading that nobody re-checked.

**Do not resolve this by re-reading — re-run it**, with the spawned name captured into a
variable and used verbatim for both `list` and `status`, so the two can never diverge
again. Until then AC1 stays open, and the Context section should be read as
under-evidenced rather than as established fact.

**Cleanup owed:** sessions `t2871a` and `t2871b` were left registered — the reaping Bash
call was refused by the budget gate. `termlink clean` next session.

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

### 2026-08-31T18:51:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2871-spawn---backend-background---wait-report.md
- **Context:** Initial task creation

### 2026-08-31T18:52:39Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
