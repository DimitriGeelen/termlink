---
id: T-2888
name: "fw audit may never complete in full: a bounded run is killed before it emits
  a summary"
description: >
  A full 'fw audit' exceeded a 15-minute bound and was killed at exit 124 inside OE-DAILY/CTL-013,
  having emitted 260 PASS / 29 WARN / 1 FAIL across 19 sections but no SUMMARY or
  PRIORITY ACTIONS. CTL-009 runs a git log per inception (168) and CTL-013 re-runs
  task verification blocks. Both the daily cron and the pre-push hook invoke only
  --sections structure, so the full audit may not run to completion anywhere. Measure
  the true wall time and decide whether this is merely slow or structurally uncompletable.

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
created: 2026-09-03T10:10:30Z
last_update: 2026-09-03T10:11:40Z
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
  - ts: '2026-09-03T10:11:40Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 4
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=4 (body:fw-audit-or-doctor); D3=3
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2888: fw audit may never complete in full: a bounded run is killed before it emits a summary

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

**Verdict: NOT a defect. The premise was wrong and is recorded here so nobody re-opens it.**

The suspicion was that `fw audit` never completes in full, and therefore that the sections
after the cut-off were reporting on nothing. Two interactive runs were killed — the first by
a self-imposed `timeout 900` (exit 124, died in `OE-DAILY/CTL-013` after 260 PASS / 29 WARN /
1 FAIL across 19 sections), the second by the harness at 248 lines in `CTL-009`. Neither
emitted `=== SUMMARY ===`, which looked like evidence for the hypothesis.

It was not. Both were artefacts of running it in a bounded foreground/background slot.

**Measured from the scheduler's own artefacts, which is better evidence than a stopwatch:**
`.context/audits/cron/2026-09-03-0800.yaml` is **45,763 bytes and carries no `sections:`
key** — the signature of the full unsharded run. It began `2026-09-03T06:00:02Z` and the file
was written at 08:07 local. **The full audit completes daily in ~7 minutes**, and it is the
largest artefact of the day.

**It is scheduled, and so is every section independently.** `/etc/cron.d/agentic-audit-termlink`
runs eight jobs: seven sharded by section (`structure,compliance,quality,discovery` every
30m; `traceability,episodic,discovery-trends` hourly; `observations,gaps` 6-hourly;
`oe-fast,oe-research` at :15/:45; `oe-hourly` at :30; `oe-daily` at 07:00; `oe-weekly` Mon
09:00) plus **one unsharded `fw audit --cron` at 08:00**. So the sections beyond the
interactive cut-off run twice over — once in their shard, once in the full run. Nothing is
dark, and AC4's filing branch does not apply.

**What is true, narrowly:** the full audit is slow for interactive use (`CTL-009` runs a
`git log` per inception, 168 of them; `CTL-013` re-runs task verification blocks). That is a
note for anyone invoking it by hand — read
`.context/audits/cron/LATEST-CRON.yaml` or the 08:00 artefact instead of re-running it — not
a governance gap.

**Today's authoritative full-audit result (08:00 run): 348 PASS / 68 WARN / 3 FAIL.** The
three failures are recorded in `## Recommendation` below; note that the interactive run saw
only one of them, because it was killed before reaching the checks that produce the other two.
That is the concrete cost of trusting a truncated run, and the reason this task closes with
the cron artefact — not the interactive output — as the source of record.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The full `fw audit` wall time is MEASURED on this tree, not estimated, and recorded
      here with the exit code — so "slow" and "uncompletable" are distinguishable
      — ~7 min, from the 08:00 cron artefact (start `06:00:02Z`, written 08:07 local).
        Interactive attempts: exit 124 (self-imposed bound) and harness-killed.
- [x] The verdict is stated one way or the other: either the run completes and emits
      `=== SUMMARY ===` (it is merely slow — record the time and close), or it does not
      complete under a generous bound (it is structurally uncompletable — record which
      section it dies in)
      — COMPLETES. `summary: {pass: 348, warn: 68, fail: 3}`. Merely slow.
- [x] Established from the crontab and the pre-push hook whether the FULL audit is invoked
      anywhere on a schedule, or only `--sections structure` — i.e. whether the sections
      after the cutoff have ever run unattended
      — 8 cron jobs: 7 sharded covering every section + 1 unsharded `fw audit --cron` at
        08:00. The post-cutoff sections run twice over. Nothing is dark.
- [x] If the full audit is confirmed uncompletable AND unscheduled, that is the
      shipped-but-dark class in the audit layer itself and is filed rather than silently
      accepted (the checks in question would be reporting on nothing)
      — Branch does not apply: neither condition holds. Nothing filed, deliberately.

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

# 1. The 08:00 artefact is the FULL run: it must carry a summary and NO `sections:` key
#    (a sharded run declares its sections; the unsharded one does not).
python3 -c "import yaml,sys;d=yaml.safe_load(open('.context/audits/cron/2026-09-03-0800.yaml'));s=d.get('summary') or {};sys.exit(0 if ('sections' not in d and all(k in s for k in ('pass','warn','fail'))) else 1)"
# 2. The full unsharded audit is actually SCHEDULED — an `fw audit --cron` line carrying no
#    --section flag. This is the claim that "nothing is dark" rests on; if a future edit
#    shards every job, this fires.
test -n "$(grep -E '^[0-9*/, ]+ +root .*fw\" audit --cron' /etc/cron.d/agentic-audit-termlink 2>/dev/null)"

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

**Recommendation:** CLOSE — no defect. The full audit completes daily in ~7 minutes and every
section is scheduled twice over. Anyone invoking it interactively should read
`.context/audits/cron/LATEST-CRON.yaml` or the 08:00 artefact rather than re-run it.

**The three FAILs in today's 08:00 full run**, none of which this task fixes — recorded so the
truncated interactive run is not mistaken for the audit result:

1. `cron(canary-aliveness-sweep)` — USER-field syntax, no install in `/etc/cron.d`. T-2878
   shipped it and it was never installed, so its own meta-canary fix is dark. Remediation
   writes to `/etc/cron.d` under sudo — **operator's call, deliberately not run.**
2. `D2: Human review queue — 57 tasks waiting >30d`, oldest 125 days (T-1417, T-1419,
   T-1435 …). These carry unticked `### Human` ACs. **An agent must never tick those**, and
   autonomous mode does not delegate completing human-owned tasks, so this can only be
   surfaced, never cleared, from here.
3. `D8: Handover quality — LATEST.md has 5 [TODO] sections`. This one IS in scope and is the
   same defect class as T-2882/T-2883/T-2884 — it is addressed separately rather than here,
   because enriching a handover is its own deliverable with its own evidence trail.

**Note on the interactive run:** it reported only failure 1. Failures 2 and 3 come from checks
it was killed before reaching. A truncated audit does not under-report proportionally — it
under-reports *whatever is at the end*, silently, while still printing hundreds of PASS lines.

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

### 2026-09-03T10:10:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2888-fw-audit-may-never-complete-in-full-a-bo.md
- **Context:** Initial task creation

### 2026-09-03T10:11:40Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
