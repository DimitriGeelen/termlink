---
id: T-2955
name: "rail project label falls back to basename PWD, mis-attributing 11 own filings
  as root"
description: >
  rail_project_label() falls back to basename ${PROJECT_ROOT:-$PWD} when RAIL_PROJECT_LABEL
  is unset. RAIL_PROJECT_LABEL is unset here, so attribution depends on the cwd at
  invocation: 11 of our own framework:pickup filings (P-064..P-073, P-076) carry from_project=root
  instead of 010-termlink, defeating the T-2816 self-filter.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-008]
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
created: 2026-09-11T19:45:34Z
last_update: 2026-09-11T19:50:21Z
date_finished: 2026-09-11T19:50:21Z
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
  - ts: '2026-09-11T19:46:45Z'
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

# T-2955: rail project label falls back to basename PWD, mis-attributing 11 own filings as root

## Context

Found 2026-09-11 while fixing T-2954, which made the rail readable past offset 99
for the first time and so made this measurable at all.

`.agentic-framework/lib/rail-identity.sh::rail_project_label()`:

    [ -n "$label" ] || label="$(basename "${PROJECT_ROOT:-$PWD}")"

`RAIL_PROJECT_LABEL` is UNSET in this project (`fw config get` returns empty), so
every rail post takes the fallback. The fallback is path-derived and, when
PROJECT_ROOT is unset, derives from $PWD -- an ambient value depending on where
the command happened to be invoked.

MEASURED on framework:pickup: 11 of our own filings carry `from_project: root`
-- offsets 106-114, 116, 120 (P-064..P-073 and P-076) -- while 6 carry the
correct `010-termlink`. Same project, same rail, two labels, decided by cwd.

The consequence is not cosmetic. T-2816's canary self-filter keys on
FW_PICKUP_SELF_PROJECT (default `010-termlink`), so the root-labelled half of our
own filings is indistinguishable from a peer's. T-2816's own rationale states the
cost precisely: our filings make the canary fire at us, and the --ack used to
clear that echo also acks any genuine inbound filing that landed in between --
reintroducing the G-063 miss the canary exists to prevent. There are genuine
inbound filings on that rail right now (five from 050-email-archive, see T-2956),
so the collision is live, not hypothetical.

Note `basename` is ALREADY known-wrong for attribution here. CLAUDE.md states it
for the canary: "Attribution is a constant, deliberately not basename
PROJECT_ROOT -- a path-derived slug is wrong inside a git worktree (that is the
T-2815 defect filed upstream)." T-2905 centralised the label to one emitter so it
could not be spelled three ways; the emitter's FALLBACK was left path-derived, so
the class T-2815 names survives in it -- and $PWD is strictly worse than
PROJECT_ROOT, because it varies per invocation rather than per checkout.

lib/rail-identity.sh is VENDORED, so G-062 applies: the fallback is not patched
here. The local remediation is to set the label explicitly so the fallback is
never reached.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `RAIL_PROJECT_LABEL` is set explicitly for this project, so
      `rail_project_label()` returns the canonical label from config and the
      path-derived fallback is never reached regardless of cwd.
- [x] The configured value matches the constant the T-2816 self-filter keys on
      (`FW_PICKUP_SELF_PROJECT`, default `010-termlink`), so the two attribution
      schemes agree instead of disagreeing silently.
- [x] The mis-attribution is recorded as measured fact — which offsets, which
      label, how the split arose — so a future reader is not left inferring it
      from the rail.
- [x] The vendored fallback is recorded locally rather than patched (G-062), as
      a learning through the supported verb plus the census entry. It is NOT
      filed upstream in this task, and that is deliberate: F-I (census) found our
      filings are posted in two forms and the `note`-typed ones may never be
      processed as filings at all, so filing now would assert a delivery that is
      not established. The filing decision is left explicit rather than performed
      blind. (`fw gaps` is read-only in this build — there is no `gaps add` verb
      to register a concern through, so no such verb is invented here.)

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

```bash
# The canonical label is configured and is not empty.
test -n "$(.agentic-framework/bin/fw config get RAIL_PROJECT_LABEL 2>/dev/null)"

# It agrees with the constant the pickup self-filter keys on.
test "$(.agentic-framework/bin/fw config get RAIL_PROJECT_LABEL 2>/dev/null)" = "010-termlink"

# The vendored fallback is untouched -- G-062, this is not patched locally.
grep -q 'PROJECT_ROOT:-\$PWD' .agentic-framework/lib/rail-identity.sh

# The measurement is recorded durably in the census, not only in the task file.
grep -q 'from_project: root' .context/audits/arc-008-cycle1-census.md

# The reasoning is registered through the supported verb, not only in prose.
grep -q 'PL-373' .context/project/learnings.yaml
```

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

The escalation is Level C, and the interesting part is WHY this was invisible
until today.

It was invisible because the canary that reads this rail could not see past
offset 99 (T-2954). Nine of the eleven mis-attributed filings sit at offsets
106-120 -- behind the truncation. So one guard being blind concealed a second
defect, and fixing the first is what surfaced the second. That is the argument for
fixing guard-layer blindness before anything it guards: a blind guard's cost is
never only the class it was built to catch.

G-019 asks why the framework was blind. It was blind because attribution was
DERIVED rather than DECLARED. A derived value is correct whenever the derivation's
inputs happen to be right, wrong silently whenever they are not, and nothing about
the wrong answer looks different from the right one -- `root` is a perfectly
well-formed project label. T-2905 already reached this conclusion for the typed
case ("the label is EMITTED from a single source and never typed at a call site")
but kept a derived fallback underneath, reintroducing exactly the property the
centralisation removed.

The lesson worth carrying: a fallback that GUESSES an identity is worse than one
that refuses. The file's own reasoning says a label present but inconsistent
"buys nothing over absence" -- an absent label is honestly unknown and filters as
unknown, whereas `root` is confidently wrong and filters as somebody else.
Unknown attribution already has a defined safe behaviour here (CLAUDE.md:
"Unknown attribution still fires ... a false fire is cheap while a false silence
is the whole point of G-063"); a wrong attribution routes around it.

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

### 2026-09-11T19:45:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2955-rail-project-label-falls-back-to-basenam.md
- **Context:** Initial task creation

### 2026-09-11T19:46:44Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-09-11T19:50:17Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-008

## Reviewer Verdict (v1.5)

- **Scan ID:** R-94a172a8
- **Timestamp:** 2026-09-11T19:50:22Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-11T19:50:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
