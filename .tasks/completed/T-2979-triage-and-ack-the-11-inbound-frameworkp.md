---
id: T-2979
name: "Triage and ack the 11 inbound framework:pickup filings"
description: >
  S-5/C-22 triage half: 11 unprocessed inbound filings on framework:pickup (G-063
  class). Triage each (file tasks / reply on peer hub), then check-framework-pickup-freshness.sh
  --ack. Evidence: consolidated C-22.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [value-review, arc:arc-009]
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
created: 2026-09-19T22:02:02Z
last_update: 2026-09-24T20:52:37Z
date_finished: 2026-09-24T20:52:37Z
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
  - ts: '2026-09-20T08:45:10Z'
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
  - ts: '2026-09-20T08:45:19Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=207,acs=4)
    rubric_sha: e4a00f38e801
---

# T-2979: Triage and ack the 11 inbound framework:pickup filings

## Context

Filed from value-review S-5/C-22 (2026-09-19): 11 inbound framework:pickup filings
were unprocessed at filing time. By the time this round picked the task up
(2026-09-24), prior sessions had already cleared most of them; only 3 remained
unprocessed (offsets 136, 140, 143 — verified via
`check-framework-pickup-freshness.sh --json`). Triaged each:

- **offset 136** (`pickup-bug-report`, `pickup_id: P-900`) — a synthetic canary/test
  fixture (`summary: "FIXTURE: widget daemon drops every second frame..."`,
  `detail` carries a literal `SENTINEL-DETAIL-9F2A` marker and a `SENTINEL-TAG-RINGBUF`
  tag, `source.project: fixture-peer-project`). Not a real defect — no task filed,
  no reply needed.
- **offset 140** (`proposal` from `1409-sprind`) — "zai-adapter", a proposal to make
  Z.ai/ZCode a first-class AEF citizen. Addressed to whoever owns AEF (explicitly
  scoped per its own G-020 framing: "Scoping, prioritisation and go/no-go are
  yours"), not to termlink specifically, and not a termlink defect or a decision
  termlink has standing to make for the framework. Informational only — no task,
  no reply.
- **offset 143** (`note` from `1409-sprind`) — DEFECT: `fw designer` dead on
  v1.6.769 (`agents/designer/designer.sh` shipped mode 644;
  `exec "$AGENTS_DIR/designer/designer.sh" "$@"` at `bin/fw:5578` requires the
  execute bit). **This one is real and affects us** — independently reproduced
  locally (see Verification), confirmed the peer's own scope claim (1 broken of 19
  direct-exec call sites — same result here), applied the mode-only unblock
  (`chmod +x`), and registered it in `.vendor-divergence.yaml` under the T-2812
  `mode-only` convention so it's flagged for re-check after the next re-vendor.
  Replied on framework:pickup (offset 145) confirming independent reproduction —
  a second-source confirmation, mirroring this project's own value-review emphasis
  on independently-converging evidence.

Cleared the canary: `check-framework-pickup-freshness.sh --ack` → seen_offset=145,
max_offset=145, 0 unprocessed.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] All inbound framework:pickup filings triaged: `check-framework-pickup-freshness.sh
      --json` reports 0 unprocessed (own filings correctly excluded per the T-2816
      self-filter).
- [x] The one genuine defect found (offset 143, `fw designer` dead — mode 644)
      independently reproduced locally before any fix applied.
- [x] Reproduced defect fixed locally (mode-only `chmod +x`) and registered in
      `.vendor-divergence.yaml` (T-2812 convention) so a re-vendor regression is
      flagged, not silent.
- [x] Confirmation reply posted to framework:pickup (offset 145, `--reply-to 143`).

## Verification

bash scripts/check-framework-pickup-freshness.sh --json > /tmp/.t2979a.out && grep -q '"unprocessed": \[\]' /tmp/.t2979a.out
test -x .agentic-framework/agents/designer/designer.sh
grep -q "designer/designer.sh" .vendor-divergence.yaml
bash scripts/check-vendor-divergence.sh > /tmp/.t2979b.out 2>&1; grep -q "all registered" /tmp/.t2979b.out

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

### 2026-09-24 — 11 filed down to 3 by the time of pickup; one genuine, two not
- **What changed:** At filing (2026-09-19) the canary reported 11 unprocessed
  inbound items; by execution time only 3 remained (prior sessions had cleared the
  rest incidentally). Of those 3, only one (offset 143) was a real, actionable
  defect — the other two were a test fixture (offset 136) and an out-of-scope
  framework proposal (offset 140) that neither warranted a task nor a reply.
- **Plan impact:** The task's original framing ("triage each: file tasks / reply")
  undersold how much of triage is *deciding something needs no action* — 2 of 3
  closed with zero output, which is itself the correct outcome, not a shortfall.
- **Triggered:** No new sub-task. The one real defect (offset 143, `fw designer`
  mode-644) was small enough (one-line `chmod +x`) to fix inline as part of this
  task rather than spinning off a separate one; registered per T-2812 convention
  instead.

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

### 2026-09-19T22:02:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2979-triage-and-ack-the-11-inbound-frameworkp.md
- **Context:** Initial task creation

### 2026-09-19T22:08:35Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-24T20:52:31Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-03aedde3
- **Timestamp:** 2026-09-24T20:52:39Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-24T20:52:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
