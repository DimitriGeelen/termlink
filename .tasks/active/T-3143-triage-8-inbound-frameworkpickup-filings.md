---
id: T-3143
name: "Triage 8 inbound framework:pickup filings — record the 1.7.x re-vendor hazards peers measured"
description: >
  Triage 8 inbound framework:pickup filings — record the 1.7.x re-vendor hazards peers measured

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
created: 2026-09-25T11:41:35Z
last_update: 2026-09-25T11:41:35Z
date_finished: null
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

# T-3143: Triage 8 inbound framework:pickup filings — record the 1.7.x re-vendor hazards peers measured

## Context

The framework-pickup canary (T-2231, G-063 prevention) fired with **8 unprocessed inbound
filings** — offsets 147-153 from `832-Workflow-designer` and 156 from `050-email-archive`.

**None of them are addressed to termlink.** `framework:pickup` is the rail peer projects use
to file at AEF, and every one of these is a peer reporting a framework defect upstream. So the
naive triage outcome is "not our work, ack it" — and that would be wrong, because three of
them were measured on a framework version we have not yet taken, and describe what will
happen to us when we do.

We run vendored **fw v1.6.29**. 832 upgraded **1.6.354 → 1.7.68** yesterday on their
operator's instruction and measured the damage. That makes their filings the only empirical
report we will get about our own next re-vendor, from the one place that has already done it.
`.vendor-divergence.yaml` already carries a PRE-RE-VENDOR CHECKLIST for exactly this moment;
it currently protects against losing OUR patches, and knows nothing about hazards in the
version we would be taking.

The asymmetry that makes this worth doing now rather than at upgrade time: the filings are on
a retention-bounded topic and the canary is about to be acked, after which nothing in this
repo remembers any of it. A future session proposing a re-vendor reads the checklist, not a
hub topic's history.

## Triage Record

All 8 read in full or to the point where the ask was unambiguous. Classification:

| off | from | subject | verdict |
|-----|------|---------|---------|
| 147 | 832 | T-2054 exemption: wrapper-blind predicate + unconditional misdiagnosis | **hazard** — H1 family |
| 148 | 832 | `upstream_repo` unreadable on a YAML continuation line; two readers disagree | **hazard H4** — measured clean here |
| 149 | 832 | 1.7.68 REGRESSED the exemption; plus exec-bit stripping; plus secret-key advice | **hazards H1+H2+H3** |
| 150 | 832 | `fw bvp` driver scoring-spec: remedy cannot reach the drivers the audit flags | AEF's work — no termlink action |
| 151 | 832 | Same regression, measured at safe-commands.sh:1207-1213 (`^git` anchor) | **hazard H1** — detail |
| 152 | 832 | Verification legs asserting an ABSENCE with nothing proving the search could succeed | not a request — **but our class** |
| 153 | 832 | Root cause + fix for 152 | not a request — **but our class** |
| 156 | 050 | Six vendored files carrying patches a re-vendor reverts | AEF's work — **finding #1 already fixed here** |

Hazards are recorded in `.vendor-divergence.yaml` under KNOWN HAZARDS IN THE TARGET VERSION,
each with a DETECT line that runs in this tree. Every DETECT line was executed before commit.

**152/153 are explicitly not a request, and are the most relevant thing in the batch.** 832
measured 122 verification legs that assert an absence with nothing establishing the search
could have succeeded — a leg that passes vacuously once its target file is gone. That is the
same defect class as T-2831 (verification misfile / vacuous pass) and is precisely why
T-3142 was parked one commit before this task: its own load-bearing test reported
`clean — 0 references` on a fixture that named an absent verb. We have no census of the shape
here. Filed as **T-3144** (inception, `GO` = run the census and report a number, explicitly
not build a guard member before that number exists) rather than widening this task.

One finding of our own, made while triaging: `fw build` is advertised at `fw help` line 85 and
exits **126 Permission denied** — `bin/fw:7534` does `exec "$FRAMEWORK_ROOT/lib/build.sh"` and
that file is vendored at mode 100644. It is a different root cause from 832's exec-bit
stripping and produces the same symptom, and a mode-drift check cannot see it because the
index AGREES with the wrong value. Filed upstream at **framework:pickup offset 162**, verified
by read-back (4125 bytes, head and tail intact). Not patched locally — it is vendored (G-062).

## Acceptance Criteria

### Agent
- [x] All 8 filings (147-153, 156) are read and individually classified as either
      "AEF's work, no termlink action" or "carries a hazard for termlink's next re-vendor" —
      no filing is left unclassified, and the classification is recorded, not just performed.
- [x] Every hazard-carrying filing is recorded in the PRE-RE-VENDOR CHECKLIST in
      `.vendor-divergence.yaml`, each naming the framework version it was measured on and the
      peer that measured it, so a future reader can weigh it rather than take it on faith.
- [x] Each recorded hazard carries a **detection step runnable in this tree** — what to run to
      find out whether it has actually bitten us — not merely a description of the defect.
      A hazard a future session cannot test for is a rumour.
- [x] Hazards that can be measured here are measured here, and the recorded entry states OUR
      result rather than repeating the peer's (specifically: exec-bit drift on tracked 100755
      files, and whether `.agentic-framework/.context/working/.fw-secret-key` exists/is ignored).
- [x] `.vendor-divergence.yaml` still parses as YAML and `scripts/check-vendor-divergence.sh`
      is still clean — this task adds commentary to the checklist, it must not disturb the
      register the checker reads.
- [x] The canary is acked and goes quiet (exit 0), so the next genuine inbound filing is
      visible against an empty baseline rather than lost in a backlog of eight.

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

# The register the checker reads must still parse and still be clean.
python3 -c "import yaml,sys; yaml.safe_load(open('.vendor-divergence.yaml')); print('parsed')"
bash scripts/check-vendor-divergence.sh > /tmp/.t3143-vd 2>&1 && grep -q . /tmp/.t3143-vd

# Every hazard is recorded against the version it was measured on, and names its source.
grep -q "1.7.68" .vendor-divergence.yaml
grep -q "832-Workflow-designer" .vendor-divergence.yaml

# The load-bearing AC: each recorded hazard carries a runnable detection step, not prose.
# Asserted against the committed file, not the working copy (T-3086 lesson).
git show HEAD:.vendor-divergence.yaml > /tmp/.t3143-head 2>&1 && grep -q "DETECT:" /tmp/.t3143-head
test "$(grep -c 'DETECT:' /tmp/.t3143-head)" -ge 4

# The canary is quiet — nothing inbound left untriaged.
bash scripts/check-framework-pickup-freshness.sh > /tmp/.t3143-canary 2>&1

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

### 2026-09-25T11:41:35Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3143-triage-8-inbound-frameworkpickup-filings.md
- **Context:** Initial task creation
