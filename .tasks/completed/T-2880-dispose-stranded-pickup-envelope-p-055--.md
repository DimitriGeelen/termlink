---
id: T-2880
name: "Dispose stranded pickup envelope P-055 — its three requested actions were already
  done"
description: >
  check-pickup-deferred-freshness.sh (T-2801) fires: P-055-bug-report.yaml sits in
  .context/pickup/auto-deferred/ with no breadcrumb, so it is STRANDED - un-promotable
  by construction and invisible to fw pickup auto-deferred list, which shows it as
  blocked-by=? and says nothing is wrong. Read it: it is a high-priority request from
  the voxtype side for three privileged host actions (add dimitri-mint-dev to the
  input group, restart the voxtype daemon under sg input, verify /dev/input/event3
  is readable in that context). Measured on this host, all three are already satisfied
  - so the envelope needs disposing with evidence, not acting on. Disposal is to .context/pickup/processed/
  matching P-050..P-054, with the verification recorded in this task so the drain
  is auditable rather than silent.

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
# demo_target: true               # T-2286: optional — marks task as reserved for an orchestrated demo
#                                 # worker (e.g. arc-010 HM-A dispatches via mcp__fw__work_on). When set,
#                                 # `fw work-on T-XXX` refuses unless --i-am-demo-orchestrator (CLI) or
#                                 # FW_I_AM_DEMO_ORCHESTRATOR=1 (env) is passed. Prevents the parent
#                                 # session from consuming the captured→started-work transition the demo
#                                 # worker expects to drive. Origin OBS-057.
created: 2026-09-02T06:36:39Z
last_update: 2026-09-02T06:39:47Z
date_finished: 2026-09-02T06:39:47Z
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
  - ts: '2026-09-02T06:37:50Z'
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

# T-2880: Dispose stranded pickup envelope P-055 — its three requested actions were already done

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **The envelope's three asks are verified satisfied on this host, not assumed.** Each requested privileged action is checked against live host state with the reading recorded, so the disposal rests on measurement rather than on "it looks done".
- [x] **The envelope is disposed to `processed/`, matching P-050..P-054.** It leaves `auto-deferred/` — where it is un-promotable by construction — and lands where every previously-actioned envelope lives, so `fw pickup status` counts it correctly.
- [x] **The drain is auditable, not silent.** The reasoning and the live readings are recorded in this task's RCA before the move, so a future reader can tell an actioned envelope from one quietly deleted. This is the specific trade T-2801 warns about: a checker that auto-drains converts a visible backlog into a silent one.
- [x] **The guard returns to clean, and did not start clean.** `check-pickup-deferred-freshness.sh` exits 1 before the move and 0 after — so the disposal is load-bearing rather than coincidental.

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

test ! -e .context/pickup/auto-deferred/P-055-bug-report.yaml
test -f .context/pickup/processed/P-055-bug-report.yaml
id -nG dimitri-mint-dev > /tmp/.t2880-groups.txt 2>&1 && grep -qw input /tmp/.t2880-groups.txt
timeout 240 bash scripts/check-pickup-deferred-freshness.sh > /tmp/.t2880-pickup.txt 2>&1

## RCA

**Symptom.** `check-pickup-deferred-freshness.sh` (T-2801) exits 1:
`1 envelope(s) STRANDED — no breadcrumb`. The envelope is
`.context/pickup/auto-deferred/P-055-bug-report.yaml`, filed 2026-09-01T20:24:35Z,
priority `high`.

**What was in it.** A request from the voxtype side for three privileged host actions:
(1) `usermod -aG input dimitri-mint-dev`, (2) restart the voxtype daemon under
`sg input` so evdev works without a re-login, (3) confirm `/dev/input/event3` is
readable in that context.

**Measured on this host, 2026-09-02 — all three already satisfied:**

| ask | reading | verdict |
|---|---|---|
| (1) user in `input` | `getent group input` → `input:x:995:dimitri-mint-dev`; `id` lists `995(input)` | **done** |
| (2) daemon restarted under `sg input` | `/proc/751189/status` → `Uid: 1000`, **`Gid: 995`**, `Groups: … 995 …`; started `Tue Sep 1 22:28:52 2026` | **done** |
| (3) `event3` readable in that context | `crw-rw---- root input 13,67 /dev/input/event3` — group-readable, and the daemon's gid IS `input` | **done** |

The daemon's **primary** gid is 995, not merely a supplementary group, which is the
signature of a `sg input`-wrapped launch — exactly the mechanism ask (2) requested. Its
start time is **~2h after** the envelope was filed, so the work was done out-of-band
while the envelope sat unread.

**Why the framework was blind (G-019).** The envelope carried no
`.breadcrumb.yaml`, so it was un-promotable by construction: `fw pickup
promote-deferred` resolves the blocking task from that file and there was none, and
`fw pickup auto-deferred list` renders it as `blocked-by=? reason=? at=?` while
reporting nothing wrong. So the one surface that showed it degraded silently — the
precise failure T-2801 was built to catch, and it caught it. The breadcrumb hole
itself is in vendored `lib/pickup.sh` and is already filed upstream per G-062; it is
not re-fixed here.

**The cost was small this time, and that is luck, not design.** The requested work
happened anyway. Had it not, a high-priority request would have sat invisible
indefinitely — the P-043 shape, where a stranded envelope's contents were
independently re-discovered and re-fixed 73 days later.

**Disposal, not deletion.** The envelope moves to `.context/pickup/processed/`
alongside P-050..P-054 rather than being removed, and this RCA records the readings
that justify it. An auto-drain would have converted a visible backlog into a silent
one, which is the trade T-2801 exists to reverse.

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

### 2026-09-02T06:36:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2880-dispose-stranded-pickup-envelope-p-055--.md
- **Context:** Initial task creation

### 2026-09-02T06:37:50Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-75890e87
- **Timestamp:** 2026-09-02T06:39:49Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-02T06:39:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
